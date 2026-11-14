use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::{
        auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
        history::{
            EntityVersionResponse, HistoryFeedResponse, RecordHistoryResponse,
            RollbackSnapshotResponse, SnapshotListResponse, UndoTransactionResponse,
        },
        privacy::{TreePersonNodeDto, TreeSnapshotDto},
    },
    state::AppState,
};
use serde_json::json;
use tower::ServiceExt;

async fn setup_test_env() -> (axum::Router, String, String, String, String) {
    let temp_dir =
        std::env::temp_dir().join(format!("genealogy_snapshots_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "History & Concurrency Test Tree".to_string(),
        backup_path: temp_dir.to_string_lossy().to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new_demo(None, config);
    let (admin_setup, _) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: None,
            first_name: "Aleksandr".to_string(),
            last_name: "Levshits".to_string(),
            email: "admin@levshits.family".to_string(),
            passkey: None,
            password: Some("AdminStrongPass123!".to_string()),
            emergency_token: None,
        },
        None,
        None,
        false,
    )
    .await
    .unwrap();
    let admin_auth = format!("Bearer {}", admin_setup.session_token);
    let admin_id = uuid::Uuid::parse_str(&admin_setup.user.id).unwrap();

    let create_role = |role: &str, email: &str, first_name: &str| {
        let state_clone = state.clone();
        let role_str = role.to_string();
        let email_str = email.to_string();
        let first_str = first_name.to_string();
        async move {
            let inv = AuthService::create_invitation(
                &state_clone,
                Some(admin_id),
                CreateInviteRequest {
                    role: role_str.clone(),
                    email: Some(email_str.clone()),
                    invitee_name: None,
                    person_id: None,
                },
            )
            .await
            .unwrap();
            let (reg, _) = AuthService::register_with_invitation(
                &state_clone,
                InviteRegisterRequest {
                    invite_token: inv.token,
                    first_name: first_str,
                    last_name: "Levshits".to_string(),
                    email: Some(email_str),
                    passkey: None,
                    password: Some("UserStrongPass123!".to_string()),
                },
                None,
                None,
                false,
            )
            .await
            .unwrap();
            format!("Bearer {}", reg.session_token)
        }
    };

    let editor1_auth = create_role("editor", "editor1@levshits.family", "Elena").await;
    let editor2_auth = create_role("editor", "editor2@levshits.family", "Dmitry").await;
    let viewer_auth = create_role("viewer", "viewer@levshits.family", "Viktor").await;
    let app = create_router(state);

    (app, admin_auth, editor1_auth, editor2_auth, viewer_auth)
}

#[tokio::test]
async fn test_change_log_feed_and_per_record_field_diffs() {
    let (app, admin_auth, editor1_auth, _editor2_auth, viewer_auth) = setup_test_env().await;

    // 1. Editor creates a person
    let create_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "sex": "M",
                        "given_name": "Mikhail",
                        "surname": "Levshits",
                        "birth_date": "1920",
                        "death_date": "1995",
                        "is_living": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let created: TreePersonNodeDto =
        serde_json::from_slice(&to_bytes(create_res.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    // 2. Editor updates the person's given_name and birth_place
    let update_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/persons/{}", created.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Mikhail Borisovich",
                        "birth_place": "Vitebsk",
                        "version": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);

    // 3. Editor uses quick-add to add a child (multi-record transaction sharing tx_id)
    let qa_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{}/quick-add", created.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "child",
                        "sex": "F",
                        "given_name": "Anna",
                        "surname": "Levshits",
                        "birth_date": "1950",
                        "death_date": "2015",
                        "is_living": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(qa_res.status(), StatusCode::CREATED);

    // 4. RBAC check on GET /api/v1/history: Viewer and Editor get 403, Admin gets 200
    let editor_hist_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/history")
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(editor_hist_res.status(), StatusCode::FORBIDDEN);

    let admin_hist_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/history")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_hist_res.status(), StatusCode::OK);
    let feed: HistoryFeedResponse = serde_json::from_slice(
        &to_bytes(admin_hist_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(feed.total >= 4);
    assert!(feed.transactions.iter().any(|g| g.changes.len() == 2
        && g.entries.len() == 2
        && g.tx_label == "person.quick_add_child"));

    // 5. Per-record history with field-level diffs (accessible to Editor via UUID or GID)
    let viewer_rec_hist = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/history/individual/{}", created.gid))
                .header(header::AUTHORIZATION, &viewer_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(viewer_rec_hist.status(), StatusCode::FORBIDDEN);

    let rec_hist_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/history/individual/{}", created.gid))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rec_hist_res.status(), StatusCode::OK);
    let rec_hist: RecordHistoryResponse = serde_json::from_slice(
        &to_bytes(rec_hist_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(rec_hist.entity_id, created.id);
    assert_eq!(rec_hist.total, 2);
    let update_entry = rec_hist
        .items
        .iter()
        .find(|i| i.action == "update")
        .expect("should have update entry");
    assert!(!update_entry.diffs.is_empty());
}

#[tokio::test]
async fn test_transaction_undo_and_restore_deleted() {
    let (app, admin_auth, editor1_auth, editor2_auth, _viewer_auth) = setup_test_env().await;

    // Create anchor person
    let create_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "sex": "M",
                        "given_name": "Boris",
                        "surname": "Levshits",
                        "birth_date": "1900",
                        "death_date": "1970",
                        "is_living": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let anchor: TreePersonNodeDto =
        serde_json::from_slice(&to_bytes(create_res.into_body(), usize::MAX).await.unwrap())
            .unwrap();

    // Quick-add a child (creates person + family in one tx_id)
    let qa_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/persons/{}/quick-add", anchor.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "relation": "child",
                        "sex": "M",
                        "given_name": "Pavel",
                        "surname": "Levshits",
                        "birth_date": "1930",
                        "death_date": "1999",
                        "is_living": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(qa_res.status(), StatusCode::CREATED);
    let qa_body: serde_json::Value =
        serde_json::from_slice(&to_bytes(qa_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let child_id = qa_body["person"]["id"].as_str().unwrap().to_string();

    // Get the tx_id of the quick-add transaction
    let hist_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/history")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let feed: HistoryFeedResponse =
        serde_json::from_slice(&to_bytes(hist_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let qa_tx_id = feed
        .transactions
        .iter()
        .find(|g| g.tx_label == "person.quick_add_child")
        .unwrap()
        .tx_id
        .clone();

    // Editor 2 (not the author, not Admin) cannot undo Editor 1's transaction
    let forbidden_undo = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/history/tx/{}/undo", qa_tx_id))
                .header(header::AUTHORIZATION, &editor2_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden_undo.status(), StatusCode::FORBIDDEN);

    // Editor 1 (author within 10 minutes) undoes the multi-record transaction
    let undo_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/history/tx/{}/undo", qa_tx_id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(undo_res.status(), StatusCode::OK);
    let undo_body: UndoTransactionResponse =
        serde_json::from_slice(&to_bytes(undo_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(undo_body.undone_count, 2);

    // Verify quick-added child is gone after undo
    let get_child = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}", child_id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_child.status(), StatusCode::NOT_FOUND);

    // Delete anchor person and restore via POST /api/v1/history/{change_id}/restore
    let del_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/persons/{}", anchor.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_res.status(), StatusCode::OK);
    let del_json: serde_json::Value =
        serde_json::from_slice(&to_bytes(del_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(del_json["status"], "deleted");
    assert_eq!(del_json["deleted_id"], anchor.id);
    assert!(del_json["tx_id"].is_string());

    let del_hist_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/history?action=delete")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let del_feed: HistoryFeedResponse = serde_json::from_slice(
        &to_bytes(del_hist_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let delete_change_id = del_feed
        .items
        .iter()
        .find(|e| e.entity_id == anchor.id && e.action == "delete")
        .unwrap()
        .id
        .clone();

    let restore_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/history/{}/restore", delete_change_id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restore_res.status(), StatusCode::OK);

    // Anchor person is accessible again
    let get_anchor = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}", anchor.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_anchor.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_optimistic_concurrency_409_conflict_and_version_polling() {
    let (app, _admin_auth, editor1_auth, editor2_auth, _viewer_auth) = setup_test_env().await;

    // 1. Editor 1 creates a person (version 1)
    let create_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "sex": "F",
                        "given_name": "Sofiya",
                        "surname": "Levshits",
                        "birth_date": "1910",
                        "death_date": "1985",
                        "is_living": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let person: TreePersonNodeDto =
        serde_json::from_slice(&to_bytes(create_res.into_body(), usize::MAX).await.unwrap())
            .unwrap();
    assert_eq!(person.version, 1);

    // 2. Editor 1 polls lightweight version endpoint by GID
    let ver_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}/version", person.gid))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ver_res.status(), StatusCode::OK);
    let ver_dto: EntityVersionResponse =
        serde_json::from_slice(&to_bytes(ver_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(ver_dto.version, 1);
    assert_eq!(ver_dto.updated_by_name.as_deref(), Some("Elena Levshits"));

    // 3. Editor 2 updates the person with version: 1 -> succeeds, version becomes 2
    let e2_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/persons/{}", person.id))
                .header(header::AUTHORIZATION, &editor2_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Sofiya Iosifovna",
                        "biography": "Born in Minsk",
                        "version": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(e2_res.status(), StatusCode::OK);
    let updated_by_e2: TreePersonNodeDto =
        serde_json::from_slice(&to_bytes(e2_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(updated_by_e2.version, 2);

    // 4. Editor 1 tries to save with stale version: 1 in JSON body -> 409 Conflict with their_data
    let conflict_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/persons/{}", person.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "nickname": "Sonya",
                        "version": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(conflict_res.status(), StatusCode::CONFLICT);
    let conflict_body: serde_json::Value = serde_json::from_slice(
        &to_bytes(conflict_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(conflict_body["error"]["code"], "version_conflict");
    assert_eq!(conflict_body["error"]["details"]["expected_version"], 1);
    assert_eq!(conflict_body["error"]["details"]["current_version"], 2);
    assert!(conflict_body["error"]["details"]["their_data"].is_object());

    // 5. Editor 1 tries to save with stale If-Match: "1" header -> 409 Conflict
    let if_match_conflict = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/persons/{}", person.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::IF_MATCH, "\"1\"")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "nickname": "Sonya"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(if_match_conflict.status(), StatusCode::CONFLICT);

    // 6. Editor 1 resolves the conflict and saves with If-Match: "2" -> 200 OK, version becomes 3
    let resolved_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/persons/{}", person.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::IF_MATCH, "\"2\"")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Sofiya Iosifovna",
                        "biography": "Born in Minsk, lived in Leningrad",
                        "nickname": "Sonya"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resolved_res.status(), StatusCode::OK);
    let resolved_person: TreePersonNodeDto = serde_json::from_slice(
        &to_bytes(resolved_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(resolved_person.version, 3);
    assert_eq!(resolved_person.nickname.as_deref(), Some("Sonya"));
}

#[tokio::test]
async fn test_tree_snapshots_and_safe_rollback() {
    let (app, admin_auth, editor1_auth, _editor2_auth, _viewer_auth) = setup_test_env().await;

    // 1. Editor creates an initial person
    let p1_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "sex": "M",
                        "given_name": "Isaak",
                        "surname": "Levshits",
                        "birth_date": "1890",
                        "death_date": "1960",
                        "is_living": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let p1: TreePersonNodeDto =
        serde_json::from_slice(&to_bytes(p1_res.into_body(), usize::MAX).await.unwrap()).unwrap();

    // 2. Editor cannot create or list snapshots (403 Forbidden)
    let editor_snap_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/snapshots")
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(json!({ "label": "Unauthorized" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(editor_snap_res.status(), StatusCode::FORBIDDEN);

    // 3. Admin creates a manual snapshot
    let create_snap_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/snapshots")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "label": "Clean baseline with Isaak",
                        "reason": "manual"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_snap_res.status(), StatusCode::CREATED);
    let snap: TreeSnapshotDto = serde_json::from_slice(
        &to_bytes(create_snap_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(snap.label, "Clean baseline with Isaak");
    assert!(snap.record_counts.individuals >= 1);

    // 4. Editor deletes p1 and creates a new person p2
    let del_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/persons/{}", p1.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_res.status(), StatusCode::OK);

    let p2_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::AUTHORIZATION, &editor1_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "sex": "M",
                        "given_name": "AccidentalPerson",
                        "surname": "WrongBranch",
                        "is_living": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let p2: TreePersonNodeDto =
        serde_json::from_slice(&to_bytes(p2_res.into_body(), usize::MAX).await.unwrap()).unwrap();

    // 5. Rollback without confirm_text: "ROLLBACK" fails with 400 Bad Request
    let bad_rb_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/snapshots/{}/rollback", snap.id))
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(json!({ "confirm_text": "wrong" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad_rb_res.status(), StatusCode::BAD_REQUEST);

    // 6. Admin rolls back with confirm_text: "ROLLBACK" -> takes pre_rollback safety snapshot and restores tree
    let rb_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/snapshots/{}/rollback", snap.id))
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({ "confirm_text": "ROLLBACK" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rb_res.status(), StatusCode::OK);
    let rb_body: RollbackSnapshotResponse =
        serde_json::from_slice(&to_bytes(rb_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(rb_body.restored_snapshot_id, snap.id);
    assert!(!rb_body.safety_snapshot_id.is_empty());

    // 7. Verify p1 is restored and p2 is removed
    let get_p1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}", p1.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_p1.status(), StatusCode::OK);

    let get_p2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/persons/{}", p2.id))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_p2.status(), StatusCode::NOT_FOUND);

    // 8. Verify GET /api/v1/snapshots lists both the baseline snapshot and the pre_rollback safety snapshot
    let list_snaps_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/snapshots")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_snaps_res.status(), StatusCode::OK);
    let snap_list: SnapshotListResponse = serde_json::from_slice(
        &to_bytes(list_snaps_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(snap_list.total, 2);

    // Verify `snapshot.rollback` emits structured integer `record_counts` (not hardcoded English text)
    let rb_hist_res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/history?page_size=20")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rb_hist_res.status(), StatusCode::OK);
    let rb_hist: HistoryFeedResponse =
        serde_json::from_slice(&to_bytes(rb_hist_res.into_body(), usize::MAX).await.unwrap())
            .unwrap();
    let rb_entry = rb_hist
        .items
        .iter()
        .find(|e| e.tx_label == "snapshot.rollback")
        .expect("snapshot.rollback entry in history");
    let rc_diff = rb_entry
        .diffs
        .iter()
        .find(|d| d.field == "record_counts")
        .expect("record_counts diff on snapshot.rollback");
    let rc_obj = rc_diff
        .after
        .as_object()
        .expect("record_counts.after should be a structured JSON object");
    assert!(rc_obj.get("individuals").and_then(|v| v.as_u64()).is_some());
    assert!(rc_obj.get("families").and_then(|v| v.as_u64()).is_some());
    assert!(rc_obj.get("events").and_then(|v| v.as_u64()).is_some());
    assert!(rc_obj.get("places").and_then(|v| v.as_u64()).is_some());
    assert!(rc_obj.get("media").and_then(|v| v.as_u64()).is_some());
    assert!(rc_obj.get("notes").and_then(|v| v.as_u64()).is_some());
}

fn contains_uuid(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() < 36 {
        return false;
    }
    for i in 0..=bytes.len() - 36 {
        if !text.is_char_boundary(i) || !text.is_char_boundary(i + 36) {
            continue;
        }
        let slice = &text[i..i + 36];
        if uuid::Uuid::parse_str(slice).is_ok() {
            return true;
        }
    }
    false
}

fn assert_clean_diffs(diffs: &[genealogy_backend::services::history::FieldDiffDto], context: &str) {
    assert!(!diffs.is_empty(), "{context}: expected non-empty diffs");
    for d in diffs {
        assert!(
            !matches!(
                d.field.as_str(),
                "id" | "gid"
                    | "version"
                    | "tx_id"
                    | "created_at"
                    | "updated_at"
                    | "birth_sort"
                    | "death_sort"
                    | "date_sort"
                    | "primary_name"
                    | "keep_before"
                    | "merge_person"
                    | "families_before"
                    | "events_before"
                    | "events_count"
                    | "children_count"
            ),
            "{context}: unexpected noisy diff field `{}`",
            d.field
        );
        let b_str = d.before.to_string();
        let a_str = d.after.to_string();
        assert!(
            !contains_uuid(&b_str) && !contains_uuid(&a_str),
            "{context}: raw UUID leaked in diff `{}`: before={b_str} after={a_str}",
            d.field
        );
        for raw in [&b_str, &a_str] {
            assert!(
                !raw.contains("\"child_id\"")
                    && !raw.contains("\"rel_to_partner1\"")
                    && !raw.contains("\"cal\"")
                    && !raw.contains("\"mod\""),
                "{context}: raw nested GenDate/FamilyChildDto JSON in `{}`: {raw}",
                d.field
            );
        }
    }
}

#[tokio::test]
async fn test_history_enrichment_labels_gids_and_clean_diffs() {
    let (app, admin_auth, editor1_auth, _editor2_auth, _viewer_auth) = setup_test_env().await;

    let post_json = |uri: String, auth: String, body: serde_json::Value| {
        let app = app.clone();
        async move {
            let res = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri(&uri)
                        .header(header::AUTHORIZATION, &auth)
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("x-requested-with", "genealogy")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = res.status();
            let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
            (
                status,
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default(),
            )
        }
    };

    let patch_json = |uri: String, auth: String, body: serde_json::Value| {
        let app = app.clone();
        async move {
            let res = app
                .oneshot(
                    Request::builder()
                        .method(Method::PATCH)
                        .uri(&uri)
                        .header(header::AUTHORIZATION, &auth)
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("x-requested-with", "genealogy")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = res.status();
            let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
            (
                status,
                serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default(),
            )
        }
    };

    // 1. Create people (with Russian/Slavic names) and edit one person
    let (st, p1) = post_json(
        "/api/v1/persons".to_string(),
        editor1_auth.clone(),
        json!({
            "sex": "M",
            "given_name": "Иван",
            "surname": "Морозов",
            "birth_date": "1900",
            "death_date": "1970",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let p1_id = p1["id"].as_str().unwrap().to_string();
    let p1_gid = p1["gid"].as_str().unwrap().to_string();

    let (st, p2) = post_json(
        "/api/v1/persons".to_string(),
        editor1_auth.clone(),
        json!({
            "sex": "F",
            "given_name": "Елена",
            "surname": "Петрова",
            "birth_date": "1904",
            "death_date": "1978",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let p2_id = p2["id"].as_str().unwrap().to_string();
    let p2_gid = p2["gid"].as_str().unwrap().to_string();

    let (st, child) = post_json(
        "/api/v1/persons".to_string(),
        editor1_auth.clone(),
        json!({
            "sex": "M",
            "given_name": "Сергей",
            "surname": "Морозов",
            "birth_date": "1930",
            "death_date": "1995",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let child_id = child["id"].as_str().unwrap().to_string();
    let child_gid = child["gid"].as_str().unwrap().to_string();

    let (st, _) = patch_json(
        format!("/api/v1/persons/{p1_id}"),
        editor1_auth.clone(),
        json!({
            "given_name": "Иван-Младший",
            "biography": "Born in Vitebsk"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // 2. Family create + partner/children edit (with non-"birth" pedigree relation)
    let (st, fam) = post_json(
        "/api/v1/families".to_string(),
        editor1_auth.clone(),
        json!({ "partner1_id": p1_id }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let fam_id = fam["id"].as_str().unwrap().to_string();
    let fam_gid = fam["gid"].as_str().unwrap().to_string();

    let (st, _) = patch_json(
        format!("/api/v1/families/{fam_id}"),
        editor1_auth.clone(),
        json!({
            "partner2_id": p2_id,
            "children": [{
                "child_id": child_id,
                "rel_to_partner1": "birth",
                "rel_to_partner2": "adopted"
            }]
        }),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // 3. Place + Event create + date/place edit
    let (st, place) = post_json(
        "/api/v1/places".to_string(),
        editor1_auth.clone(),
        json!({ "name": "Витебск", "place_type": "city" }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let place_id = place["id"].as_str().unwrap().to_string();

    let (st, ev_occ) = post_json(
        "/api/v1/events".to_string(),
        editor1_auth.clone(),
        json!({
            "individual_id": p1_id,
            "event_type": "occupation",
            "date_text": "1925",
            "description": "Blacksmith"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let ev_occ_id = ev_occ["id"].as_str().unwrap().to_string();
    let ev_occ_gid = ev_occ["gid"].as_str().unwrap().to_string();

    let (st, _) = patch_json(
        format!("/api/v1/events/{ev_occ_id}"),
        editor1_auth.clone(),
        json!({
            "date_text": "abt 1914",
            "place_id": place_id
        }),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // 4. Note create + Undo transaction
    let (st, _note) = post_json(
        "/api/v1/notes".to_string(),
        editor1_auth.clone(),
        json!({
            "entity_type": "individual",
            "entity_id": p1_id,
            "body": "Family archive reference"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);

    // 5. Duplicate birth events + merge (event_merge_bundle)
    let (st, b1) = post_json(
        "/api/v1/events".to_string(),
        editor1_auth.clone(),
        json!({
            "individual_id": p2_id,
            "event_type": "birth",
            "date_text": "1904",
            "source_text": "Parish record"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let b1_id = b1["id"].as_str().unwrap().to_string();
    let b1_gid = b1["gid"].as_str().unwrap().to_string();
    let (st, b2) = post_json(
        "/api/v1/events".to_string(),
        editor1_auth.clone(),
        json!({
            "individual_id": p2_id,
            "event_type": "birth",
            "date_text": "1904",
            "description": "Winter morning"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let b2_id = b2["id"].as_str().unwrap().to_string();
    let b2_gid = b2["gid"].as_str().unwrap().to_string();

    let (st, _) = post_json(
        "/api/v1/quality/events/merge".to_string(),
        admin_auth.clone(),
        json!({
            "keep_event_id": b1_id,
            "merge_event_ids": [b2_id]
        }),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // 5b. Batch merge duplicate death events
    let (st, _) = post_json(
        "/api/v1/events".to_string(),
        editor1_auth.clone(),
        json!({
            "individual_id": p1_id,
            "event_type": "death",
            "date_text": "1970",
            "source_text": "Civil registry"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let (st, d2) = post_json(
        "/api/v1/events".to_string(),
        editor1_auth.clone(),
        json!({
            "individual_id": p1_id,
            "event_type": "death",
            "date_text": "1970",
            "description": "Winter"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let d2_gid = d2["gid"].as_str().unwrap().to_string();

    let (st, _) = post_json(
        "/api/v1/quality/events/batch-merge".to_string(),
        admin_auth.clone(),
        json!({}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // 6. Create and delete a temporary person (`DeletedPersonSnapshotPayload`)
    let (st, doomed) = post_json(
        "/api/v1/persons".to_string(),
        editor1_auth.clone(),
        json!({
            "sex": "M",
            "given_name": "Пётр",
            "surname": "Удаляемый",
            "birth_date": "1910",
            "death_date": "1980",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let doomed_id = doomed["id"].as_str().unwrap().to_string();
    let doomed_gid = doomed["gid"].as_str().unwrap().to_string();

    let del_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/persons/{doomed_id}"))
                .header(header::AUTHORIZATION, &editor1_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_res.status(), StatusCode::OK);

    // Undo the `ev_occ` update to also generate a `history.undo_transaction` entry
    let ev_hist_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/history/event/{ev_occ_gid}"))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ev_hist_res.status(), StatusCode::OK);
    let ev_hist: RecordHistoryResponse =
        serde_json::from_slice(&to_bytes(ev_hist_res.into_body(), usize::MAX).await.unwrap())
            .unwrap();
    let ev_upd = ev_hist.items.iter().find(|i| i.action == "update").unwrap();
    let (st, _) = post_json(
        format!("/api/v1/history/tx/{}/undo", ev_upd.tx_id),
        editor1_auth.clone(),
        json!({}),
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    // Verify Global Feed (`/api/v1/history`) enrichment
    let feed_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/history?page_size=200")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feed_res.status(), StatusCode::OK);
    let feed: HistoryFeedResponse =
        serde_json::from_slice(&to_bytes(feed_res.into_body(), usize::MAX).await.unwrap()).unwrap();

    // Check person edit in feed
    let person_upd = feed
        .items
        .iter()
        .find(|e| e.entity_id == p1_id && e.op == "update")
        .expect("person update in feed");
    assert_eq!(person_upd.entity_gid.as_deref(), Some(p1_gid.as_str()));
    assert_clean_diffs(&person_upd.diffs, "feed person update");
    assert!(person_upd
        .diffs
        .iter()
        .any(|d| d.field == "given_name" && d.after == "Иван-Младший"));

    // Check family update in feed
    let fam_upd = feed
        .items
        .iter()
        .find(|e| e.entity_id == fam_id && e.op == "update")
        .expect("family update in feed");
    assert_eq!(fam_upd.entity_gid.as_deref(), Some(fam_gid.as_str()));
    let fam_lbl = fam_upd.entity_label.as_deref().unwrap_or("");
    assert!(
        fam_lbl.contains("Морозов")
            && fam_lbl.contains("Елена Петрова")
            && !fam_lbl.contains(&fam_gid),
        "unexpected family label (should omit separate entity_gid): {fam_lbl}"
    );
    assert_clean_diffs(&fam_upd.diffs, "feed family update");
    let p2_diff = fam_upd
        .diffs
        .iter()
        .find(|d| d.field == "partner2_id")
        .unwrap();
    assert!(p2_diff.after.as_str().unwrap().contains(&p2_gid));
    let ch_diff = fam_upd
        .diffs
        .iter()
        .find(|d| d.field == "children")
        .unwrap();
    let ch_arr = ch_diff.after.as_array().unwrap();
    assert_eq!(ch_arr.len(), 1);
    let ch_str = ch_arr[0].as_str().unwrap();
    assert!(
        ch_str.contains(&child_gid) && ch_str.contains("[adopted]"),
        "got {ch_str}"
    );

    // Check event edit in feed
    let ev_feed_upd = feed
        .items
        .iter()
        .find(|e| e.entity_id == ev_occ_id && e.op == "update")
        .expect("event update in feed");
    assert_eq!(ev_feed_upd.entity_gid.as_deref(), Some(ev_occ_gid.as_str()));
    let ev_lbl = ev_feed_upd.entity_label.as_deref().unwrap_or("");
    assert!(
        ev_lbl.contains("occupation — ")
            && ev_lbl.contains("Морозов")
            && !ev_lbl.contains(&ev_occ_gid),
        "unexpected event label (should omit separate entity_gid): {ev_lbl}"
    );
    assert_clean_diffs(&ev_feed_upd.diffs, "feed event update");
    assert!(ev_feed_upd.diffs.iter().any(|d| d.field == "date"
        && d.after
            .as_str()
            .is_some_and(|s| s.eq_ignore_ascii_case("abt 1914"))));
    assert!(!ev_feed_upd
        .diffs
        .iter()
        .any(|d| d.field == "gedcom_date" || d.field == "place_id"));

    // Check note in feed
    let note_create = feed
        .items
        .iter()
        .find(|e| e.entity_type == "note" && e.op == "create")
        .expect("note create in feed");
    assert!(note_create.entity_gid.is_none());
    let note_lbl = note_create.entity_label.as_deref().unwrap_or("");
    assert!(
        note_lbl.starts_with("Note — ") && note_lbl.contains("Морозов"),
        "got {note_lbl}"
    );
    assert_clean_diffs(&note_create.diffs, "feed note create");

    // Check person deletion in feed (`DeletedPersonSnapshotPayload` unwrapped)
    let del_entry = feed
        .items
        .iter()
        .find(|e| e.entity_id == doomed_id && e.op == "delete")
        .expect("deleted person in feed");
    assert_eq!(del_entry.entity_gid.as_deref(), Some(doomed_gid.as_str()));
    assert_clean_diffs(&del_entry.diffs, "feed person delete");
    assert!(del_entry
        .diffs
        .iter()
        .any(|d| d.field == "given_name" && d.before == "Пётр"));
    assert!(!del_entry.diffs.iter().any(|d| d.field == "person"));

    // Check event merge & batch merge in feed (`event_merge_bundle`)
    let merge_entry = feed
        .items
        .iter()
        .find(|e| e.tx_label == "quality.events.merge")
        .expect("event merge in feed");
    assert_eq!(merge_entry.entity_gid.as_deref(), Some(b1_gid.as_str()));
    let merge_lbl = merge_entry.entity_label.as_deref().unwrap_or("");
    assert!(
        merge_lbl.contains("birth — Елена Петрова") && !merge_lbl.contains(&b1_gid),
        "unexpected merge label (should omit separate entity_gid): {merge_lbl}"
    );
    assert_clean_diffs(&merge_entry.diffs, "feed event merge");
    let merged_diff = merge_entry
        .diffs
        .iter()
        .find(|d| d.field == "merged_events")
        .unwrap();
    assert!(merged_diff.before.to_string().contains(&b2_gid));
    assert!(merged_diff.after.to_string().contains(&b1_gid));

    let batch_entry = feed
        .items
        .iter()
        .find(|e| e.tx_label == "quality.events.batch_merge")
        .expect("batch event merge in feed");
    assert_clean_diffs(&batch_entry.diffs, "feed batch event merge");
    let batch_diff = batch_entry
        .diffs
        .iter()
        .find(|d| d.field == "merged_events")
        .unwrap();
    assert!(
        batch_diff.before.to_string().contains(&d2_gid)
            || batch_diff.after.to_string().contains("death"),
        "unexpected batch diff: {:?}",
        batch_diff
    );

    // Check person.create diffs and transaction-grouped entries are also enriched
    let person_create = feed
        .items
        .iter()
        .find(|e| e.entity_id == p1_id && e.op == "create")
        .expect("person create in feed");
    assert_eq!(person_create.entity_gid.as_deref(), Some(p1_gid.as_str()));
    assert_clean_diffs(&person_create.diffs, "feed person create");
    assert!(feed
        .transactions
        .iter()
        .flat_map(|t| t.entries.iter())
        .any(|e| e.entity_id == fam_id
            && e.op == "update"
            && e.entity_gid.as_deref() == Some(fam_gid.as_str())
            && !e.diffs.is_empty()));

    // Check transaction undo entry in feed
    let undo_entry = feed
        .items
        .iter()
        .find(|e| e.tx_label == "history.undo_transaction")
        .expect("undo tx in feed");
    assert!(undo_entry.entity_gid.is_none());
    let undo_lbl = undo_entry.entity_label.as_deref().unwrap_or("");
    assert!(
        !undo_lbl.is_empty() && !contains_uuid(undo_lbl),
        "got {undo_lbl}"
    );
    assert_clean_diffs(&undo_entry.diffs, "feed undo transaction");

    // Verify per-record history (`/api/v1/history/{entity_type}/{entity_id}`) via GID, including deleted person!
    let doomed_hist_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/history/individual/{doomed_gid}"))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(doomed_hist_res.status(), StatusCode::OK);
    let doomed_hist: RecordHistoryResponse = serde_json::from_slice(
        &to_bytes(doomed_hist_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(doomed_hist.entity_id, doomed_id);
    assert_eq!(doomed_hist.items.len(), 2);
    for item in &doomed_hist.items {
        assert_eq!(item.entity_gid.as_deref(), Some(doomed_gid.as_str()));
        assert_clean_diffs(&item.diffs, "per-record deleted person");
    }

    let fam_hist_res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/v1/history/family/{fam_gid}"))
                .header(header::AUTHORIZATION, &editor1_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fam_hist_res.status(), StatusCode::OK);
    let fam_hist: RecordHistoryResponse = serde_json::from_slice(
        &to_bytes(fam_hist_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(fam_hist.entity_id, fam_id);
    if let Some(ref lbl) = fam_hist.entity_label {
        assert!(!lbl.contains(&fam_gid), "got {lbl}");
    }
    for item in &fam_hist.items {
        assert_eq!(item.entity_gid.as_deref(), Some(fam_gid.as_str()));
        if let Some(ref lbl) = item.entity_label {
            assert!(!lbl.contains(&fam_gid), "got {lbl}");
        }
        assert_clean_diffs(&item.diffs, "per-record family");
    }
}
