use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
    state::AppState,
};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn setup_test_env() -> (axum::Router, AppState, String, String, String) {
    let temp_dir =
        std::env::temp_dir().join(format!("genealogy_dq_events_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Duplicate Events Test Tree".to_string(),
        enable_public_access: false,
        backup_path: temp_dir.to_string_lossy().to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    {
        let mut store = state.privacy_store.write().await;
        store.persons.clear();
        store.families.clear();
        store.events.clear();
        store.places.clear();
        store.notes.clear();
        store.media.clear();
        store.dismissed_warnings.clear();
        store.not_duplicate_pairs.clear();
        store.change_log.clear();
    }

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
                    role: role_str,
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

    let editor_auth = create_role("editor", "editor@levshits.family", "Elena").await;
    let viewer_auth = create_role("viewer", "viewer@levshits.family", "Viktor").await;
    let app = create_router(state.clone());

    (app, state, admin_auth, editor_auth, viewer_auth)
}

async fn post_json(
    app: &axum::Router,
    uri: &str,
    auth: &str,
    payload: Value,
) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(uri)
                .header(header::AUTHORIZATION, auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let val = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, val)
}

async fn get_json(app: &axum::Router, uri: &str, auth: &str) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header(header::AUTHORIZATION, auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let val = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, val)
}

#[tokio::test]
async fn test_duplicate_birth_death_detection_merge_vitals_and_undo() {
    let (app, _, admin_auth, _, _) = setup_test_env().await;

    let (st_p, person) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Иван",
            "surname": "Иванов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st_p, StatusCode::CREATED);
    let pid = person["id"].as_str().unwrap().to_string();

    // Two compatible birth events (exact Y-M-D + broader year/city)
    let (s1, b1) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "birth",
            "date_text": "1890-05-12",
            "place_path": "Москва, Российская империя",
            "source_text": "Метрическая книга",
            "description": "Church parish entry"
        }),
    )
    .await;
    assert_eq!(s1, StatusCode::CREATED);
    let b1_id = b1["id"].as_str().unwrap().to_string();

    let (s2, b2) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "BIRT",
            "date_text": "1890",
            "place_path": "Москва",
            "description": "Family Bible margin note"
        }),
    )
    .await;
    assert_eq!(s2, StatusCode::CREATED);
    let b2_id = b2["id"].as_str().unwrap().to_string();

    // Two compatible death events (fuzzy ABT 1960 overlaps 1961-02-10; place prefix matches)
    let (_, d1) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "death",
            "date_text": "ABT 1960",
            "place_path": "Ленинград"
        }),
    )
    .await;
    let d1_id = d1["id"].as_str().unwrap().to_string();

    let (_, d2) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "death",
            "date_text": "1961-02-10",
            "place_path": "Ленинград, СССР",
            "source_text": "ЗАГС запись №44"
        }),
    )
    .await;
    let d2_id = d2["id"].as_str().unwrap().to_string();

    let (st_list, list) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(st_list, StatusCode::OK);
    assert_eq!(list["total"], 2);

    let items = list["items"].as_array().unwrap();
    let birth_group = items.iter().find(|g| g["event_type"] == "birth").unwrap();
    assert_eq!(birth_group["scope"], "individual");
    assert_eq!(birth_group["has_conflict"], false);
    assert_eq!(birth_group["confidence"], "high");
    assert_eq!(birth_group["proposed"]["keep_event_id"], b1_id);
    assert_eq!(birth_group["proposed"]["date_text"], "1890-05-12");
    assert!(birth_group["proposed"]["description"]
        .as_str()
        .unwrap()
        .contains("Church parish entry"));
    assert!(birth_group["proposed"]["description"]
        .as_str()
        .unwrap()
        .contains("Family Bible margin note"));

    let death_group = items.iter().find(|g| g["event_type"] == "death").unwrap();
    assert_eq!(death_group["has_conflict"], false);
    assert_eq!(death_group["confidence"], "high");
    assert_eq!(death_group["proposed"]["keep_event_id"], d2_id);

    // Check quality summary & warnings reflect DUPLICATE_EVENT
    let (_, summary) = get_json(&app, "/api/v1/quality/summary", &admin_auth).await;
    assert_eq!(summary["duplicate_events_count"], 2);
    assert_eq!(summary["by_rule"]["DUPLICATE_EVENT"], 2);

    let (_, warn_list) = get_json(
        &app,
        "/api/v1/quality/warnings?rule_code=DUPLICATE_EVENT",
        &admin_auth,
    )
    .await;
    assert_eq!(warn_list["total"], 2);

    // Merge birth group using group_id
    let (st_m1, m1) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({ "group_id": birth_group["id"] }),
    )
    .await;
    assert_eq!(st_m1, StatusCode::OK, "birth merge failed: {m1}");
    assert_eq!(m1["status"], "merged");
    assert_eq!(m1["surviving_event"]["id"], b1_id);
    assert_eq!(m1["merged_event_ids"], json!([b2_id]));
    let birth_tx_id = m1["tx_id"].as_str().unwrap().to_string();

    // Merge death group explicitly using alias fields + explicit override
    let (st_m2, m2) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({
            "target_event_id": d2_id,
            "duplicate_event_ids": [d1_id],
            "description": "Confirmed archival death",
            "source_text": "ЗАГС №44 / семейный архив"
        }),
    )
    .await;
    assert_eq!(st_m2, StatusCode::OK, "death merge failed: {m2}");
    assert_eq!(m2["surviving_event"]["id"], d2_id);
    assert_eq!(
        m2["surviving_event"]["description"],
        "Confirmed archival death"
    );

    // Quality duplicates for events should now be 0
    let (_, list_after) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(list_after["total"], 0);

    // Check individual profile vitals updated
    let (_, prof) = get_json(&app, &format!("/api/v1/persons/{pid}/profile"), &admin_auth).await;
    assert_eq!(prof["person"]["has_death_event"], true);

    // 1-click Undo birth merge
    let (st_undo, _) = post_json(
        &app,
        &format!("/api/v1/history/tx/{birth_tx_id}/undo"),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(st_undo, StatusCode::OK);

    // Duplicate birth group reappears cleanly with both events restored
    let (_, list_restored) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(list_restored["total"], 1);
    assert_eq!(list_restored["items"][0]["event_type"], "birth");
    assert_eq!(
        list_restored["items"][0]["events"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn test_duplicate_marriage_couple_folding_conflict_and_family_consolidation() {
    let (app, state, admin_auth, _, _) = setup_test_env().await;

    let (_, husb) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Алексей", "surname": "Петров", "sex": "M", "is_living": false }),
    )
    .await;
    let (_, wife) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Мария", "surname": "Сидорова", "sex": "F", "is_living": false }),
    )
    .await;
    let (_, child1) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Николай", "surname": "Петров", "sex": "M", "is_living": false }),
    )
    .await;
    let (_, child2) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Ольга", "surname": "Петрова", "sex": "F", "is_living": false }),
    )
    .await;

    let (_, child3) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Екатерина", "surname": "Петрова", "sex": "F", "is_living": false }),
    )
    .await;

    let h_id = husb["id"].as_str().unwrap().to_string();
    let w_id = wife["id"].as_str().unwrap().to_string();
    let c1_id = child1["id"].as_str().unwrap().to_string();
    let c2_id = child2["id"].as_str().unwrap().to_string();
    let c3_id = child3["id"].as_str().unwrap().to_string();

    // Two duplicate couple families (with reversed partner order and distinct child-parent relations)
    let (st_f1, f1) = post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": h_id,
            "partner2_id": w_id,
            "relationship_type": "married",
            "children_ids": [c1_id]
        }),
    )
    .await;
    assert_eq!(st_f1, StatusCode::CREATED);
    let f1_id = f1["id"].as_str().unwrap().to_string();

    let (st_f2, f2) = post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": w_id,
            "partner2_id": h_id,
            "relationship_type": "married",
            "children": [{
                "child_id": c2_id,
                "rel_to_partner1": "adopted",
                "rel_to_partner2": "foster"
            }]
        }),
    )
    .await;
    assert_eq!(st_f2, StatusCode::CREATED);
    let f2_id = f2["id"].as_str().unwrap().to_string();

    // Conflicting marriage events on F1 and F2, plus compatible individual marriage event on h_id
    let (_, m1) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "family_id": f1_id,
            "event_type": "marriage",
            "date_text": "1914-07-10",
            "place_path": "Киев, Российская империя",
            "description": "Church wedding"
        }),
    )
    .await;
    let m1_id = m1["id"].as_str().unwrap().to_string();

    let (_, m2) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "family_id": f2_id,
            "event_type": "MARR",
            "date_text": "1918-03-01",
            "place_path": "Одесса, УССР",
            "source_text": "Civil register"
        }),
    )
    .await;
    let m2_id = m2["id"].as_str().unwrap().to_string();

    // Single-partner family for folding & child-relation slot remapping test (`F3` has partner1=w_id only, whereas `F1` has partner2=w_id)
    let (st_f3, f3) = post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": w_id,
            "relationship_type": "married",
            "children": [{
                "child_id": c3_id,
                "rel_to_partner1": "step",
                "rel_to_partner2": "birth"
            }]
        }),
    )
    .await;
    assert_eq!(st_f3, StatusCode::CREATED);
    let f3_id = f3["id"].as_str().unwrap().to_string();

    let (st_m3, m3_ind) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "family_id": f3_id,
            "event_type": "marriage",
            "date_text": "1914",
            "place_path": "Киев"
        }),
    )
    .await;
    assert_eq!(st_m3, StatusCode::CREATED);
    let m3_id = m3_ind["id"].as_str().unwrap().to_string();

    // Attach note on m2 so we verify note consolidation
    post_json(
        &app,
        "/api/v1/notes",
        &admin_auth,
        json!({ "entity_type": "event", "entity_id": m2_id, "body": "Signed by witness", "is_private": false }),
    )
    .await;

    let (_, list) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(list["total"], 1);
    let grp = &list["items"][0];
    assert_eq!(grp["scope"], "family");
    assert_eq!(grp["event_type"], "marriage");
    assert_eq!(grp["has_conflict"], true);
    assert_eq!(grp["confidence"], "medium");
    let conflicts = grp["conflict_fields"].as_array().unwrap();
    assert!(conflicts.contains(&json!("date")));
    assert!(conflicts.contains(&json!("place")));
    // Single-partner/individual marriage folded into couple bucket
    assert_eq!(grp["events"].as_array().unwrap().len(), 3);

    // Merge keeping m1_id with explicit field overrides (place_name auto-resolves/creates place chain)
    let (st_merge, merge_resp) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({
            "group_id": grp["id"],
            "keep_event_id": m1_id,
            "date_text": "1914-07-10",
            "place_name": "Киев, Украина",
            "description": "Consolidated church and civil marriage",
            "source_text": "Combined parish and ZAGS archive"
        }),
    )
    .await;
    assert_eq!(
        st_merge,
        StatusCode::OK,
        "marriage merge failed: {merge_resp}"
    );
    assert_eq!(merge_resp["surviving_event"]["id"], m1_id);
    assert_eq!(
        merge_resp["surviving_event"]["place_full_name"],
        "Киев, Украина"
    );
    assert_eq!(
        merge_resp["surviving_event"]["description"],
        "Consolidated church and civil marriage"
    );
    let merged_fams = merge_resp["merged_family_ids"].as_array().unwrap();
    assert!(merged_fams.contains(&json!(f2_id)));
    assert!(merged_fams.contains(&json!(f3_id)));
    let merged_evs = merge_resp["merged_event_ids"].as_array().unwrap();
    assert!(merged_evs.contains(&json!(m2_id)));
    assert!(merged_evs.contains(&json!(m3_id)));

    // Verify F1 absorbed Child 2 & Child 3 with partner-slot relation remapping, and note moved to m1_id
    {
        let store = state.privacy_store.read().await;
        assert_eq!(store.families.len(), 1);
        let fam = &store.families[0];
        assert_eq!(fam.id, f1_id);
        assert_eq!(fam.children.len(), 3);
        let c2_rel = fam.children.iter().find(|c| c.child_id == c2_id).unwrap();
        assert_eq!(c2_rel.rel_to_partner1, "foster"); // h_id was partner2 on f2, partner1 on f1
        assert_eq!(c2_rel.rel_to_partner2, "adopted"); // w_id was partner1 on f2, partner2 on f1
        let c3_rel = fam.children.iter().find(|c| c.child_id == c3_id).unwrap();
        assert_eq!(c3_rel.rel_to_partner1, "birth"); // h_id absent from single-partner f3
        assert_eq!(c3_rel.rel_to_partner2, "step"); // w_id was partner1 on f3, partner2 on f1
        assert!(store.notes.iter().any(|n| n.entity_type == "event"
            && n.entity_id == m1_id
            && n.body == "Signed by witness"));
    }

    // Undo and verify F2/F3 + children + all 3 marriage events are restored
    let tx_id = merge_resp["tx_id"].as_str().unwrap();
    let (st_undo, _) = post_json(
        &app,
        &format!("/api/v1/history/tx/{tx_id}/undo"),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(st_undo, StatusCode::OK);
    {
        let store = state.privacy_store.read().await;
        assert_eq!(store.families.len(), 3);
    }
    let (_, list_after_undo) =
        get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(list_after_undo["total"], 1);
    assert_eq!(
        list_after_undo["items"][0]["events"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[tokio::test]
async fn test_repeatable_events_dismissal_batch_merge_undo_rbac_and_mcp() {
    let (app, state, admin_auth, editor_auth, viewer_auth) = setup_test_env().await;

    let (_, p1) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Борис", "surname": "Волков", "sex": "M", "is_living": false }),
    )
    .await;
    let (_, p2) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Анна", "surname": "Волкова", "sex": "F", "is_living": false }),
    )
    .await;
    let p1_id = p1["id"].as_str().unwrap().to_string();
    let p2_id = p2["id"].as_str().unwrap().to_string();

    // P1: 2 compatible residence events (cluster 0) + 1 unrelated 1930 Odessa residence event
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p1_id, "event_type": "residence", "date_text": "1905", "place_path": "Москва" }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p1_id, "event_type": "residence", "date_text": "1905-03-01", "place_path": "Москва, Россия" }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p1_id, "event_type": "residence", "date_text": "1930", "place_path": "Одесса, УССР" }),
    )
    .await;

    // P2: 2 compatible birth events (high) + 2 conflicting death events (medium)
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p2_id, "event_type": "birth", "date_text": "1901", "place_path": "Тверь" }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p2_id, "event_type": "birth", "date_text": "1901-04-15", "place_path": "Тверь, Россия" }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p2_id, "event_type": "death", "date_text": "1960", "place_path": "Тверь" }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p2_id, "event_type": "death", "date_text": "1968", "place_path": "Тверь" }),
    )
    .await;

    // RBAC: Viewer & Editor get 403 Forbidden on all /api/v1/quality/events/* endpoints
    for auth in [&viewer_auth, &editor_auth] {
        let (s1, _) = get_json(&app, "/api/v1/quality/events/duplicates", auth).await;
        assert_eq!(s1, StatusCode::FORBIDDEN);
        let (s2, _) = post_json(&app, "/api/v1/quality/events/merge", auth, json!({})).await;
        assert_eq!(s2, StatusCode::FORBIDDEN);
        let (s3, _) = post_json(&app, "/api/v1/quality/events/batch-merge", auth, json!({})).await;
        assert_eq!(s3, StatusCode::FORBIDDEN);
    }

    let (_, list) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(list["total"], 3);
    let items = list["items"].as_array().unwrap();
    let res_group = items
        .iter()
        .find(|g| g["event_type"] == "residence")
        .unwrap();
    // Unrelated 1930 Odessa event stayed outside the 1905 Moscow cluster
    assert_eq!(res_group["events"].as_array().unwrap().len(), 2);

    // Dismiss residence duplicate group
    let res_gid = res_group["id"].as_str().unwrap().to_string();
    let (st_dis, dis) = post_json(
        &app,
        "/api/v1/quality/warnings/dismiss",
        &admin_auth,
        json!({ "rule_code": "DUPLICATE_EVENT", "entity_id": res_gid }),
    )
    .await;
    assert_eq!(st_dis, StatusCode::OK);
    assert_eq!(dis["dismissed"]["entity_type"], "event");
    assert_eq!(dis["dismissed"]["entity_id"], res_gid);

    let (_, active_list) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(active_list["total"], 2);
    let (_, all_list) = get_json(
        &app,
        "/api/v1/quality/events/duplicates?include_dismissed=true",
        &admin_auth,
    )
    .await;
    assert_eq!(all_list["total"], 3);

    // Batch-merge high-confidence groups: only P2 birth should merge (residence is dismissed; death is medium)
    let (st_batch, batch) = post_json(
        &app,
        "/api/v1/quality/events/batch-merge",
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(st_batch, StatusCode::OK);
    assert_eq!(batch["merged_groups_count"], 1);
    assert_eq!(batch["removed_events_count"], 1);
    let batch_tx = batch["tx_id"].as_str().unwrap().to_string();

    let (_, remaining) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(remaining["total"], 1);
    assert_eq!(remaining["items"][0]["event_type"], "death");
    assert_eq!(remaining["items"][0]["confidence"], "medium");

    // Single Undo reverts batch-merge cleanly
    let (st_undo, _) = post_json(
        &app,
        &format!("/api/v1/history/tx/{batch_tx}/undo"),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(st_undo, StatusCode::OK);
    let (_, restored) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(restored["total"], 2);

    // MCP support: Admin PAT with `admin:maintenance` can call list_duplicate_events & merge_duplicate_events
    let (tok_st, tok_resp) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &admin_auth,
        json!({
            "name": "Quality Agent",
            "scopes": ["read:tree", "admin:maintenance"]
        }),
    )
    .await;
    assert_eq!(tok_st, StatusCode::CREATED, "PAT create failed: {tok_resp}");
    let pat = format!("Bearer {}", tok_resp["raw_token"].as_str().unwrap());

    let (tl_st, tl_resp) = post_json(
        &app,
        "/api/v1/mcp",
        &pat,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} }),
    )
    .await;
    assert_eq!(tl_st, StatusCode::OK);
    let tool_names: Vec<&str> = tl_resp["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(tool_names.contains(&"list_duplicate_events"));
    assert!(tool_names.contains(&"merge_duplicate_events"));

    let (ml_st, ml_resp) = post_json(
        &app,
        "/api/v1/mcp",
        &pat,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "list_duplicate_events",
                "arguments": { "confidence": "high" }
            }
        }),
    )
    .await;
    assert_eq!(ml_st, StatusCode::OK);
    let p2_birth_gid = ml_resp["result"]["structuredContent"]["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (mm_st, mm_resp) = post_json(
        &app,
        "/api/v1/mcp",
        &pat,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_events",
                "arguments": { "group_id": p2_birth_gid }
            }
        }),
    )
    .await;
    assert_eq!(mm_st, StatusCode::OK);
    assert_eq!(mm_resp["result"]["structuredContent"]["status"], "merged");

    // Viewer PAT cannot invoke Tier C list_duplicate_events
    let (_, vtok) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &viewer_auth,
        json!({ "name": "Viewer Reader", "scopes": ["read:tree"] }),
    )
    .await;
    let v_pat = format!("Bearer {}", vtok["raw_token"].as_str().unwrap());
    let (v_st, _) = post_json(
        &app,
        "/api/v1/mcp",
        &v_pat,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": { "name": "list_duplicate_events", "arguments": {} }
        }),
    )
    .await;
    assert_eq!(v_st, StatusCode::FORBIDDEN);

    // OpenAPI completeness check
    let (_, spec) = get_json(&app, "/api/v1/openapi.json", &admin_auth).await;
    assert!(spec["paths"]["/api/v1/quality/events/duplicates"].is_object());
    assert!(spec["paths"]["/api/v1/quality/events/merge"].is_object());
    assert!(spec["paths"]["/api/v1/quality/events/batch-merge"].is_object());
    assert!(spec["components"]["schemas"]["DuplicateEventGroupDto"].is_object());
    let _ = state;
}

#[tokio::test]
async fn test_individual_marriage_folding_media_union_and_validation_errors() {
    use genealogy_backend::{models::gendate::GenDate, services::privacy::LifeEventDto};

    let (app, state, admin_auth, _, _) = setup_test_env().await;

    let (_, h) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Сергей", "surname": "Орлов", "sex": "M", "is_living": false }),
    )
    .await;
    let (_, w) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Татьяна", "surname": "Белова", "sex": "F", "is_living": false }),
    )
    .await;
    let h_id = h["id"].as_str().unwrap().to_string();
    let w_id = w["id"].as_str().unwrap().to_string();

    let (_, fam) = post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({ "partner1_id": h_id, "partner2_id": w_id, "relationship_type": "married" }),
    )
    .await;
    let fam_id = fam["id"].as_str().unwrap().to_string();

    let (_, fm_ev) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "family_id": fam_id, "event_type": "marriage", "date_text": "1920-08-15", "place_path": "Казань" }),
    )
    .await;
    let fm_ev_id = fm_ev["id"].as_str().unwrap().to_string();

    let (_, birth_ev) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": h_id, "event_type": "birth", "date_text": "1895" }),
    )
    .await;
    let birth_ev_id = birth_ev["id"].as_str().unwrap().to_string();

    // Seed individual-level marriage event AND legacy FAMC birth event (both individual_id + family_id)
    let ind_marr_id = uuid::Uuid::new_v4().to_string();
    let famc_birth_id = uuid::Uuid::new_v4().to_string();
    {
        let mut store = state.privacy_store.write().await;
        if let Some(ev) = store.events.iter_mut().find(|e| e.id == fm_ev_id) {
            ev.media_ids = vec!["media-1".to_string()];
        }
        let gd = GenDate::parse("1920");
        store.events.push(LifeEventDto {
            id: ind_marr_id.clone(),
            gid: "E0999".to_string(),
            individual_id: Some(h_id.clone()),
            family_id: None,
            event_type: "marriage".to_string(),
            custom_type: None,
            date_sort: gd.sort_key(),
            gedcom_date: Some(gd.to_gedcom()),
            date: Some(gd),
            place_id: None,
            place_full_name: Some("Казань, Татарская АССР".to_string()),
            description: Some("Legacy import record".to_string()),
            source_text: None,
            media_ids: vec!["media-2".to_string()],
            is_private: false,
            version: 1,
        });
        let bgd = GenDate::parse("1895-04-01");
        store.events.push(LifeEventDto {
            id: famc_birth_id.clone(),
            gid: "E0998".to_string(),
            individual_id: Some(h_id.clone()),
            family_id: Some(fam_id.clone()),
            event_type: "birth".to_string(),
            custom_type: None,
            date_sort: bgd.sort_key(),
            gedcom_date: Some(bgd.to_gedcom()),
            date: Some(bgd),
            place_id: None,
            place_full_name: None,
            description: Some("Legacy FAMC birth".to_string()),
            source_text: None,
            media_ids: vec![],
            is_private: false,
            version: 1,
        });
    }

    let (_, list) = get_json(&app, "/api/v1/quality/events/duplicates", &admin_auth).await;
    assert_eq!(list["total"], 2);
    let items = list["items"].as_array().unwrap();
    let grp = items
        .iter()
        .find(|g| g["event_type"] == "marriage")
        .unwrap();
    assert_eq!(grp["scope"], "family");
    assert_eq!(grp["proposed"]["combined_media_count"], 2);
    assert_eq!(grp["has_conflict"], false);
    let ind_birth_grp = items.iter().find(|g| g["event_type"] == "birth").unwrap();
    assert_eq!(ind_birth_grp["scope"], "individual");
    assert_eq!(ind_birth_grp["id"], format!("dup_event:ind:{h_id}:birth"));
    assert_eq!(ind_birth_grp["events"].as_array().unwrap().len(), 2);

    // Validation errors: missing group -> 404; mismatched event types -> 400
    let (s404, _) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({ "group_id": "dup_event:ind:missing:birth" }),
    )
    .await;
    assert_eq!(s404, StatusCode::NOT_FOUND);

    let (s400, _) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({ "keep_event_id": fm_ev_id, "merge_event_ids": [birth_ev_id] }),
    )
    .await;
    assert_eq!(s400, StatusCode::BAD_REQUEST);

    // Execute merge cleanly
    let (s_ok, merged) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({ "group_id": grp["id"] }),
    )
    .await;
    assert_eq!(s_ok, StatusCode::OK);
    assert_eq!(merged["surviving_event"]["id"], fm_ev_id);
    assert_eq!(merged["surviving_event"]["family_id"], fam_id);
    assert!(merged["surviving_event"]["individual_id"].is_null());
    assert_eq!(
        merged["surviving_event"]["media_ids"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(merged["merged_event_ids"], json!([ind_marr_id]));

    // Merging the individual birth group containing the legacy FAMC birth event must preserve
    // surviving.individual_id and leave fam_id intact without family consolidation (DEF-002).
    let (b_st, b_merged) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({ "group_id": ind_birth_grp["id"] }),
    )
    .await;
    assert_eq!(b_st, StatusCode::OK, "Birth merge failed: {b_merged}");
    assert_eq!(b_merged["surviving_event"]["id"], famc_birth_id);
    assert_eq!(b_merged["surviving_event"]["individual_id"], h_id);
    assert_eq!(b_merged["merged_event_ids"], json!([birth_ev_id]));
    assert_eq!(b_merged["merged_family_ids"], json!([]));
    {
        let store = state.privacy_store.read().await;
        let fam = store.families.iter().find(|f| f.id == fam_id).unwrap();
        assert_eq!(fam.marriage_date.as_deref(), Some("1920-08-15"));
        let ev = store.events.iter().find(|e| e.id == famc_birth_id).unwrap();
        assert_eq!(ev.individual_id.as_deref(), Some(h_id.as_str()));
    }
}

#[tokio::test]
async fn test_security_explicit_clearing_and_additional_event_types() {
    let (app, state, admin_auth, _editor_auth, _viewer_auth) = setup_test_env().await;

    // 1. Anonymous callers receive 401 Unauthorized with standard error envelope
    for (method, path) in [
        (Method::GET, "/api/v1/quality/events/duplicates"),
        (Method::POST, "/api/v1/quality/events/merge"),
        (Method::POST, "/api/v1/quality/events/batch-merge"),
    ] {
        let mut builder = Request::builder().method(method.clone()).uri(path);
        if method == Method::POST {
            builder = builder
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy");
        }
        let res = app
            .clone()
            .oneshot(builder.body(Body::from("{}")).unwrap())
            .await
            .unwrap();
        let st = res.status();
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        assert_eq!(st, StatusCode::UNAUTHORIZED, "Expected 401 on {path}");
        assert!(
            body["error"]["code"].is_string(),
            "Expected error.code on {path}"
        );
    }

    // 2. CSRF protection rejects missing X-Requested-With and foreign Origin with 403
    for path in [
        "/api/v1/quality/events/merge",
        "/api/v1/quality/events/batch-merge",
    ] {
        let no_xrw = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .header(header::AUTHORIZATION, &admin_auth)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            no_xrw.status(),
            StatusCode::FORBIDDEN,
            "Missing XRW on {path}"
        );

        let bad_origin = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .header(header::AUTHORIZATION, &admin_auth)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-requested-with", "genealogy")
                    .header("origin", "https://evil.example")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            bad_origin.status(),
            StatusCode::FORBIDDEN,
            "Foreign Origin on {path}"
        );
    }

    // 3. Additional singleton vital (baptism/burial/cremation), union (divorce/engagement),
    // repeatable occupation clustering, genealogical dates (BEF / BET..AND), and private/living access
    let (_, person) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Нина", "surname": "Кузнецова", "sex": "F", "is_living": true }),
    )
    .await;
    let pid = person["id"].as_str().unwrap().to_string();

    let (_, partner) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Виктор", "surname": "Орлов", "sex": "M", "is_living": false }),
    )
    .await;
    let partner_id = partner["id"].as_str().unwrap().to_string();

    let (_, fam) = post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({ "partner1_id": partner_id, "partner2_id": pid }),
    )
    .await;
    let fam_id = fam["id"].as_str().unwrap().to_string();

    let (bap_st, bap1) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "baptism",
            "date_text": "BEF 1890",
            "place_path": "Рязань, Российская империя",
            "description": "Parish christening",
            "source_text": "ГАРО ф.10",
            "is_private": true
        }),
    )
    .await;
    assert_eq!(bap_st, StatusCode::CREATED);
    let bap1_id = bap1["id"].as_str().unwrap().to_string();
    let (_, bap2) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "baptism",
            "date_text": "1889-12-01",
            "place_path": "Рязань, Российская империя",
            "description": "Duplicate baptism",
            "source_text": "Family register"
        }),
    )
    .await;
    let bap2_id = bap2["id"].as_str().unwrap().to_string();

    // Repeatable occupation events: compatible ABT 1920 + 1921 cluster; 1935 stays separate
    let (_, occ1) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "occupation",
            "custom_type": "Teacher",
            "date_text": "ABT 1920",
            "place_path": "Рязань",
            "description": "Gymnasium teacher"
        }),
    )
    .await;
    let occ1_id = occ1["id"].as_str().unwrap().to_string();
    let (_, occ2) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "occupation",
            "date_text": "1921",
            "custom_type": "Teacher",
            "place_path": "Рязань"
        }),
    )
    .await;
    let occ2_id = occ2["id"].as_str().unwrap().to_string();
    let (_, _) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": pid,
            "event_type": "occupation",
            "custom_type": "Teacher",
            "date_text": "1935"
        }),
    )
    .await;

    // Couple divorce duplicate using BET..AND range
    let (_, div1) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "family_id": fam_id,
            "event_type": "divorce",
            "date_text": "BET 1918 AND 1920",
            "place_path": "Москва"
        }),
    )
    .await;
    let div1_id = div1["id"].as_str().unwrap().to_string();
    let (_, div2) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "family_id": fam_id,
            "event_type": "divorce",
            "date_text": "1919-06-15",
            "place_path": "Москва"
        }),
    )
    .await;
    let div2_id = div2["id"].as_str().unwrap().to_string();

    // Filter queries work accurately
    let (f1_st, f_occ) = get_json(
        &app,
        "/api/v1/quality/events/duplicates?event_type=occupation&scope=individual",
        &admin_auth,
    )
    .await;
    assert_eq!(f1_st, StatusCode::OK);
    assert_eq!(f_occ["total"], 1);
    assert_eq!(f_occ["items"][0]["events"].as_array().unwrap().len(), 2);

    let (_, f_div) = get_json(
        &app,
        "/api/v1/quality/events/duplicates?event_type=divorce&scope=family&confidence=high",
        &admin_auth,
    )
    .await;
    assert_eq!(f_div["total"], 1);
    assert_eq!(f_div["items"][0]["scope"], "family");

    // 4. Explicit empty-string "" clearing clears date, place_id/name, description & source_text; Undo restores
    let (clear_st, cleared) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({
            "group_id": format!("dup_event:ind:{pid}:baptism"),
            "keep_event_id": bap1_id,
            "merge_event_ids": [bap2_id],
            "date_text": "",
            "place_name": "",
            "description": "",
            "source_text": ""
        }),
    )
    .await;
    assert_eq!(
        clear_st,
        StatusCode::OK,
        "Explicit clear merge failed: {cleared}"
    );
    let surv = &cleared["surviving_event"];
    assert!(surv["date"].is_null(), "date must clear on empty date_text");
    assert!(
        surv["date_sort"].is_null(),
        "date_sort must clear on empty date_text"
    );
    assert!(
        surv["gedcom_date"].is_null(),
        "gedcom_date must clear on empty date_text"
    );
    assert!(
        surv["place_id"].is_null(),
        "place_id must clear on empty place_name"
    );
    assert!(
        surv["place_full_name"].is_null(),
        "place_full_name must clear"
    );
    assert!(
        surv["description"].is_null(),
        "description must clear on empty string"
    );
    assert!(
        surv["source_text"].is_null(),
        "source_text must clear on empty string"
    );

    let clear_tx = cleared["tx_id"].as_str().unwrap().to_string();
    let (undo_clear_st, _) = post_json(
        &app,
        &format!("/api/v1/history/tx/{clear_tx}/undo"),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(undo_clear_st, StatusCode::OK);
    {
        let store = state.privacy_store.read().await;
        let orig_bap1 = store.events.iter().find(|e| e.id == bap1_id).unwrap();
        assert_eq!(orig_bap1.description.as_deref(), Some("Parish christening"));
        assert!(orig_bap1.place_full_name.is_some());
    }

    // 5. MCP `merge_duplicate_events` explicit `target_event_id` + `source_event_ids` aliases and `batch: true`
    let (_, tok_resp) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &admin_auth,
        json!({ "name": "Quality Alias Agent", "scopes": ["read:tree", "admin:maintenance"] }),
    )
    .await;
    let pat = format!("Bearer {}", tok_resp["raw_token"].as_str().unwrap());

    let (alias_st, alias_resp) = post_json(
        &app,
        "/api/v1/mcp",
        &pat,
        json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_events",
                "arguments": {
                    "target_event_id": occ1_id,
                    "source_event_ids": [occ2_id]
                }
            }
        }),
    )
    .await;
    assert_eq!(alias_st, StatusCode::OK, "Alias merge failed: {alias_resp}");
    assert_eq!(
        alias_resp["result"]["structuredContent"]["surviving_event"]["id"],
        occ1_id
    );

    let (batch_mcp_st, batch_mcp_resp) = post_json(
        &app,
        "/api/v1/mcp",
        &pat,
        json!({
            "jsonrpc": "2.0",
            "id": 12,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_events",
                "arguments": {
                    "batch": true,
                    "only_high_confidence": false
                }
            }
        }),
    )
    .await;
    assert_eq!(
        batch_mcp_st,
        StatusCode::OK,
        "MCP batch merge failed: {batch_mcp_resp}"
    );
    assert_eq!(
        batch_mcp_resp["result"]["structuredContent"]["merged_groups_count"],
        2
    );
    let _ = (div1_id, div2_id);
}

#[tokio::test]
async fn test_regression_mcp_undo_vitals_famc_individual_id_and_private_merge() {
    use genealogy_backend::{models::gendate::GenDate, services::privacy::LifeEventDto};

    let (app, state, admin_auth, _, _) = setup_test_env().await;

    // 1. DEF-001: 1850-born person with NO death event stays is_living=false after MCP undo_transaction
    let (_, p1850) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Матвей", "surname": "Историков", "sex": "M" }),
    )
    .await;
    let p1850_id = p1850["id"].as_str().unwrap().to_string();
    let (_, b1850_1) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p1850_id, "event_type": "birth", "date_text": "1850-01-10" }),
    )
    .await;
    let (_, b1850_2) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": p1850_id, "event_type": "birth", "date_text": "1850" }),
    )
    .await;
    let _ = (b1850_1, b1850_2);
    {
        let store = state.privacy_store.read().await;
        let person = store.persons.iter().find(|p| p.id == p1850_id).unwrap();
        assert!(
            !person.is_living,
            "1850-born person without death event must be deceased initially"
        );
    }

    let (_, tok_resp) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &admin_auth,
        json!({ "name": "Undo Regression Agent", "scopes": ["read:tree", "admin:maintenance"] }),
    )
    .await;
    let pat = format!("Bearer {}", tok_resp["raw_token"].as_str().unwrap());

    let (m_st, m_resp) = post_json(
        &app,
        "/api/v1/mcp",
        &pat,
        json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_events",
                "arguments": { "group_id": format!("dup_event:ind:{p1850_id}:birth") }
            }
        }),
    )
    .await;
    assert_eq!(m_st, StatusCode::OK, "MCP merge failed: {m_resp}");
    let tx_id = m_resp["result"]["structuredContent"]["tx_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (u_st, u_resp) = post_json(
        &app,
        "/api/v1/mcp",
        &pat,
        json!({
            "jsonrpc": "2.0",
            "id": 101,
            "method": "tools/call",
            "params": {
                "name": "undo_transaction",
                "arguments": { "tx_id": tx_id }
            }
        }),
    )
    .await;
    assert_eq!(
        u_st,
        StatusCode::OK,
        "MCP undo_transaction failed: {u_resp}"
    );
    {
        let store = state.privacy_store.read().await;
        let person = store.persons.iter().find(|p| p.id == p1850_id).unwrap();
        assert!(
            !person.is_living,
            "DEF-001: 1850-born person must remain is_living=false after MCP undo_transaction"
        );
    }

    // 2. DEF-002: Merging regular birth + legacy FAMC birth (whether regular or FAMC is kept)
    // preserves surviving.individual_id and keeps FAMC family intact, while rejecting cross-person merge
    let (_, dad) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Пётр", "surname": "Соколов", "sex": "M", "is_living": false }),
    )
    .await;
    let (_, mom) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Ольга", "surname": "Соколова", "sex": "F", "is_living": false }),
    )
    .await;
    let (_, child) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({ "given_name": "Дарья", "surname": "Соколова", "sex": "F", "is_living": false }),
    )
    .await;
    let dad_id = dad["id"].as_str().unwrap().to_string();
    let mom_id = mom["id"].as_str().unwrap().to_string();
    let child_id = child["id"].as_str().unwrap().to_string();

    let (_, famc) = post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": dad_id,
            "partner2_id": mom_id,
            "children_ids": [child_id]
        }),
    )
    .await;
    let famc_id = famc["id"].as_str().unwrap().to_string();

    let (_, child_birth_reg) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({ "individual_id": child_id, "event_type": "birth", "date_text": "1910-05-02" }),
    )
    .await;
    let child_birth_reg_id = child_birth_reg["id"].as_str().unwrap().to_string();

    let child_birth_famc_id = uuid::Uuid::new_v4().to_string();
    let mom_birth_famc_id = uuid::Uuid::new_v4().to_string();
    {
        let mut store = state.privacy_store.write().await;
        let gd_c = GenDate::parse("1910");
        store.events.push(LifeEventDto {
            id: child_birth_famc_id.clone(),
            gid: "E0701".to_string(),
            individual_id: Some(child_id.clone()),
            family_id: Some(famc_id.clone()),
            event_type: "birth".to_string(),
            custom_type: None,
            date_sort: gd_c.sort_key(),
            gedcom_date: Some(gd_c.to_gedcom()),
            date: Some(gd_c),
            place_id: None,
            place_full_name: None,
            description: Some("Legacy FAMC birth".to_string()),
            source_text: None,
            media_ids: vec![],
            is_private: false,
            version: 1,
        });
        let gd_m = GenDate::parse("1885");
        store.events.push(LifeEventDto {
            id: mom_birth_famc_id.clone(),
            gid: "E0702".to_string(),
            individual_id: Some(mom_id.clone()),
            family_id: Some(famc_id.clone()),
            event_type: "birth".to_string(),
            custom_type: None,
            date_sort: gd_m.sort_key(),
            gedcom_date: Some(gd_m.to_gedcom()),
            date: Some(gd_m),
            place_id: None,
            place_full_name: None,
            description: None,
            source_text: None,
            media_ids: vec![],
            is_private: false,
            version: 1,
        });
    }

    // Explicit cross-individual merge with shared legacy FAMC family_id must still fail validation (400)
    let (cross_st, _) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({ "keep_event_id": child_birth_famc_id, "merge_event_ids": [mom_birth_famc_id] }),
    )
    .await;
    assert_eq!(cross_st, StatusCode::BAD_REQUEST);

    // Explicit merge keeping regular birth and merging legacy FAMC birth preserves individual_id & famc_id
    let (famc_st, famc_merged) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({
            "keep_event_id": child_birth_reg_id,
            "merge_event_ids": [child_birth_famc_id]
        }),
    )
    .await;
    assert_eq!(
        famc_st,
        StatusCode::OK,
        "FAMC birth merge failed: {famc_merged}"
    );
    assert_eq!(famc_merged["surviving_event"]["individual_id"], child_id);
    assert_eq!(famc_merged["merged_family_ids"], json!([]));
    {
        let store = state.privacy_store.read().await;
        assert!(
            store.families.iter().any(|f| f.id == famc_id),
            "DEF-002: FAMC family must remain intact after individual birth merge"
        );
        let surviving_ev = store
            .events
            .iter()
            .find(|e| e.id == child_birth_reg_id)
            .unwrap();
        assert_eq!(
            surviving_ev.individual_id.as_deref(),
            Some(child_id.as_str())
        );
    }

    // 3. DEF-004: Merging non-private surviving event + is_private=true duplicate preserves is_private=true
    let (_, res_pub) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": child_id,
            "event_type": "residence",
            "date_text": "1932",
            "place_path": "Новгород",
            "description": "Publicly documented address",
            "source_text": "Census 1932",
            "is_private": false
        }),
    )
    .await;
    let res_pub_id = res_pub["id"].as_str().unwrap().to_string();

    let (_, res_priv) = post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": child_id,
            "event_type": "residence",
            "date_text": "1932",
            "is_private": true
        }),
    )
    .await;
    let res_priv_id = res_priv["id"].as_str().unwrap().to_string();

    let (priv_st, priv_merged) = post_json(
        &app,
        "/api/v1/quality/events/merge",
        &admin_auth,
        json!({
            "keep_event_id": res_pub_id,
            "merge_event_ids": [res_priv_id]
        }),
    )
    .await;
    assert_eq!(
        priv_st,
        StatusCode::OK,
        "Private merge failed: {priv_merged}"
    );
    assert_eq!(priv_merged["surviving_event"]["id"], res_pub_id);
    assert_eq!(
        priv_merged["surviving_event"]["is_private"], true,
        "DEF-004: merging a private event into a public event must mark surviving event private"
    );
    {
        let store = state.privacy_store.read().await;
        let ev = store.events.iter().find(|e| e.id == res_pub_id).unwrap();
        assert!(ev.is_private);
    }
}
