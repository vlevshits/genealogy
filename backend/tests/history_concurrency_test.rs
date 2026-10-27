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
    let state = AppState::new(None, config);
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
}
