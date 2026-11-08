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

async fn setup_test_env() -> (axum::Router, String, String, String) {
    let temp_dir =
        std::env::temp_dir().join(format!("genealogy_dq_places_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Data Quality & Place Hierarchy Test Tree".to_string(),
        enable_public_access: false,
        backup_path: temp_dir.to_string_lossy().to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    // Clear seeded demo records so our tests operate on a clean deterministic store
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

    let editor_auth = create_role("editor", "editor@levshits.family", "Elena").await;
    let viewer_auth = create_role("viewer", "viewer@levshits.family", "Viktor").await;
    let app = create_router(state);

    (app, admin_auth, editor_auth, viewer_auth)
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
async fn test_relationship_centric_duplicate_detection_kinship_disqualifications_and_detached_scoring(
) {
    let (app, admin_auth, _, _) = setup_test_env().await;

    // 1. Father and Son with identical given name & surname ("Пётр Щеглов" / "Pyotr Shcheglov")
    let (_, father) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Пётр",
            "surname": "Щеглов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    let father_id = father["id"].as_str().unwrap().to_string();

    let (_, mother) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Пётр",
            "surname": "Щеглова",
            "sex": "F",
            "is_living": false
        }),
    )
    .await;
    let mother_id = mother["id"].as_str().unwrap().to_string();

    let (_, son1) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Пётр",
            "patronymic": "Петрович",
            "surname": "Щеглов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    let son1_id = son1["id"].as_str().unwrap().to_string();

    // 2. Confirmed distinct sibling in the same family (born in a different year)
    let (_, son2_sibling) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Пётр",
            "patronymic": "Петрович",
            "surname": "Щеглов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    let son2_id = son2_sibling["id"].as_str().unwrap().to_string();

    // Add distinct birth years for son1 (1910) and son2 (1915)
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": son1_id,
            "event_type": "birth",
            "gedcom_date": "10 MAR 1910"
        }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": son2_id,
            "event_type": "birth",
            "gedcom_date": "15 MAY 1915"
        }),
    )
    .await;

    // Link father, mother, son1, and son2 in Family 1
    let (fam_status, fam1) = post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": father_id,
            "partner2_id": mother_id,
            "children_ids": [son1_id, son2_id]
        }),
    )
    .await;
    assert_eq!(fam_status, StatusCode::CREATED, "fam1: {fam1}");

    // 3. Same-component person with identical name ("Пётр Щеглов") but disjoint confirmed biological parents
    // Connect father's brother ("Иван Щеглов") into the same connected component, and give him a son named "Пётр Щеглов"
    let (_, grandfather) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Николай",
            "surname": "Щеглов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    let grandfather_id = grandfather["id"].as_str().unwrap().to_string();

    let (_, uncle) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Иван",
            "surname": "Щеглов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    let uncle_id = uncle["id"].as_str().unwrap().to_string();

    let (_, aunt) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Мария",
            "surname": "Щеглова",
            "sex": "F",
            "is_living": false
        }),
    )
    .await;
    let aunt_id = aunt["id"].as_str().unwrap().to_string();

    let (_, cousin_pyotr) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Пётр",
            "surname": "Щеглов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    let cousin_id = cousin_pyotr["id"].as_str().unwrap().to_string();

    // Connect grandfather -> [father, uncle]
    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": grandfather_id,
            "children_ids": [father_id, uncle_id]
        }),
    )
    .await;

    // Connect uncle + aunt -> [cousin_pyotr]
    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": uncle_id,
            "partner2_id": aunt_id,
            "children_ids": [cousin_id]
        }),
    )
    .await;

    // Verify ZERO false-positive duplicates inside the connected component!
    // - Father & Son (direct ancestor/descendant) -> 0
    // - Father & Mother (spouses in same family) -> 0
    // - Son1 & Son2 (siblings with distinct birth dates 1910 vs 1915) -> 0
    // - Son1 & Cousin Pyotr (same component, disjoint confirmed parents) -> 0
    let (dup_status, dup_resp) =
        get_json(&app, "/api/v1/quality/duplicates?min_score=1", &admin_auth).await;
    assert_eq!(dup_status, StatusCode::OK);
    let pairs = dup_resp["items"].as_array().unwrap();
    for pair in pairs {
        let a = pair["person_a"]["id"].as_str().unwrap();
        let b = pair["person_b"]["id"].as_str().unwrap();
        let is_pair = |x: &str, y: &str| (a == x && b == y) || (a == y && b == x);

        assert!(
            !is_pair(&father_id, &son1_id),
            "Father and son must NEVER be flagged as duplicates: {pair}"
        );
        assert!(
            !is_pair(&grandfather_id, &son1_id),
            "Grandfather and grandson must NEVER be flagged as duplicates: {pair}"
        );
        assert!(
            !is_pair(&father_id, &mother_id),
            "Spouses must NEVER be flagged as duplicates: {pair}"
        );
        assert!(
            !is_pair(&son1_id, &son2_id),
            "Confirmed distinct siblings (1910 vs 1915) must NEVER be flagged as duplicates: {pair}"
        );
        assert!(
            !is_pair(&son1_id, &cousin_id),
            "Same-component cousins with disjoint confirmed parents must NEVER be flagged: {pair}"
        );
    }

    // Also verify that with default min_score=80, total is 0 before adding detached duplicates
    let (_, default_dups) = get_json(&app, "/api/v1/quality/duplicates", &admin_auth).await;
    assert_eq!(
        default_dups["total"].as_u64().unwrap(),
        0,
        "Expected 0 duplicate pairs in clean connected component, got: {default_dups}"
    );

    // 4. Now create a detached singleton duplicate of son1 ("Пётр Петрович Щеглов", b. 1910)
    let (_, detached_singleton) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Пётр",
            "patronymic": "Петрович",
            "surname": "Щеглов",
            "sex": "M",
            "is_living": false
        }),
    )
    .await;
    let singleton_id = detached_singleton["id"].as_str().unwrap().to_string();
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": singleton_id,
            "event_type": "birth",
            "gedcom_date": "1910"
        }),
    )
    .await;

    let (dup_status2, dup_resp2) = get_json(&app, "/api/v1/quality/duplicates", &admin_auth).await;
    assert_eq!(dup_status2, StatusCode::OK);
    let flagged = dup_resp2["items"].as_array().unwrap();
    let singleton_match = flagged.iter().find(|pair| {
        let a = pair["person_a"]["id"].as_str().unwrap();
        let b = pair["person_b"]["id"].as_str().unwrap();
        (a == son1_id && b == singleton_id) || (a == singleton_id && b == son1_id)
    });
    assert!(
        singleton_match.is_some(),
        "Detached singleton with matching name, patronymic, and birth year 1910 should score >= 80: {dup_resp2}"
    );
    let sm = singleton_match.unwrap();
    assert!(sm["score"].as_u64().unwrap() >= 80);
    assert_eq!(sm["is_detached"], json!(true));
    let reasons: Vec<&str> = sm["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        reasons.contains(&"detached_singleton"),
        "Expected detached_singleton reason in {reasons:?}"
    );
    assert!(
        reasons.contains(&"same_birth_year"),
        "Expected same_birth_year reason in {reasons:?}"
    );
}

#[tokio::test]
async fn test_place_hierarchy_analysis_unparsed_paths_haversine_orphans_batch_fix_and_undo() {
    let (app, admin_auth, _, _) = setup_test_env().await;

    // 1. Create an unparsed comma-separated 4-level place:
    // "Покровка, Новоузенский уезд, Самарская губерния, Российская империя"
    let (st1, pokrovka_place) = post_json(
        &app,
        "/api/v1/places",
        &admin_auth,
        json!({
            "name": "Покровка, Новоузенский уезд, Самарская губерния, Российская империя",
            "place_type": "other",
            "latitude": 50.85,
            "longitude": 48.20
        }),
    )
    .await;
    assert_eq!(st1, StatusCode::CREATED);
    let pokrovka_id = pokrovka_place["id"].as_str().unwrap().to_string();

    // Attach a life event to Pokrovka so we can verify events stay attached after restructuring
    let (_, person) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Анна",
            "surname": "Иванова",
            "sex": "F",
            "is_living": false
        }),
    )
    .await;
    let person_id = person["id"].as_str().unwrap().to_string();
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": person_id,
            "event_type": "birth",
            "gedcom_date": "1892",
            "place_id": pokrovka_id
        }),
    )
    .await;

    // 2. Create a structured parent district ("Тверской уезд", latitude 56.85, longitude 35.91)
    // and an orphan village ("с. Медное", latitude 56.90, longitude 35.48 -> ~27 km away, well within 40 km!)
    let (_, tver_region) = post_json(
        &app,
        "/api/v1/places",
        &admin_auth,
        json!({
            "name": "Тверская губерния",
            "place_type": "region",
            "latitude": 56.85,
            "longitude": 35.91
        }),
    )
    .await;
    let tver_region_id = tver_region["id"].as_str().unwrap().to_string();

    let (_, tver_district) = post_json(
        &app,
        "/api/v1/places",
        &admin_auth,
        json!({
            "name": "Тверской уезд",
            "place_type": "district",
            "parent_id": tver_region_id,
            "latitude": 56.85,
            "longitude": 35.91
        }),
    )
    .await;
    let tver_district_id = tver_district["id"].as_str().unwrap().to_string();

    let (_, mednoye_orphan) = post_json(
        &app,
        "/api/v1/places",
        &admin_auth,
        json!({
            "name": "с. Медное",
            "place_type": "other",
            "latitude": 56.90,
            "longitude": 35.48
        }),
    )
    .await;
    let mednoye_id = mednoye_orphan["id"].as_str().unwrap().to_string();

    // 3. Inspect `GET /api/v1/quality/places/analysis` and `GET /api/v1/quality/summary`
    let (sum_st, summary) = get_json(&app, "/api/v1/quality/summary", &admin_auth).await;
    assert_eq!(sum_st, StatusCode::OK);
    assert!(
        summary["place_hierarchy_issues"].as_u64().unwrap() >= 2,
        "Expected place_hierarchy_issues >= 2 in summary: {summary}"
    );
    assert!(
        summary["place_hierarchy_high_confidence"].as_u64().unwrap() >= 2,
        "Expected place_hierarchy_high_confidence >= 2 in summary: {summary}"
    );

    let (an_st, analysis) = get_json(&app, "/api/v1/quality/places/analysis", &admin_auth).await;
    assert_eq!(an_st, StatusCode::OK);
    assert_eq!(analysis["counts"]["unparsed_paths"], 1);
    assert!(analysis["counts"]["orphans"].as_u64().unwrap() >= 1);
    assert!(analysis["counts"]["high_confidence"].as_u64().unwrap() >= 2);
    let issues = analysis["items"].as_array().unwrap();

    // Check Pokrovka unparsed path issue
    let pokrovka_issue = issues
        .iter()
        .find(|i| i["place_id"].as_str() == Some(pokrovka_id.as_str()))
        .expect("Pokrovka unparsed path issue should be detected");
    assert_eq!(pokrovka_issue["issue_type"], "unparsed_path");
    assert_eq!(pokrovka_issue["category"], "unparsed_paths");
    assert_eq!(pokrovka_issue["rule_code"], "PLACE_UNPARSED_PATH");
    assert!(pokrovka_issue["confidence"].as_u64().unwrap() >= 90);
    assert_eq!(
        pokrovka_issue["confidence_score"],
        pokrovka_issue["confidence"]
    );
    assert!(!pokrovka_issue["rationale"].as_str().unwrap().is_empty());
    assert_eq!(pokrovka_issue["proposed_clean_name"], "Покровка");
    let chain = pokrovka_issue["proposed_chain"].as_array().unwrap();
    assert_eq!(chain.len(), 4);
    assert_eq!(chain[0]["name"], "Российская империя");
    assert_eq!(chain[0]["place_type"], "country");
    assert_eq!(chain[0]["action"], "create");
    assert_eq!(chain[1]["name"], "Самарская губерния");
    assert_eq!(chain[1]["place_type"], "region");
    assert_eq!(chain[1]["action"], "create");
    assert_eq!(chain[2]["name"], "Новоузенский уезд");
    assert_eq!(chain[2]["place_type"], "district");
    assert_eq!(chain[2]["action"], "create");
    assert_eq!(chain[3]["name"], "Покровка");
    assert_eq!(chain[3]["place_type"], "village");
    assert_eq!(chain[3]["action"], "clean_leaf");

    // Check single-place preview endpoint (`GET /api/v1/quality/places/preview?id=...`)
    let (prev_st, prev_dto) = get_json(
        &app,
        &format!("/api/v1/quality/places/preview?id={pokrovka_id}"),
        &admin_auth,
    )
    .await;
    assert_eq!(prev_st, StatusCode::OK);
    assert_eq!(prev_dto["place_id"], pokrovka_id);
    assert_eq!(prev_dto["events_count"], 1);

    // Check Mednoye orphan hierarchy issue matched by Haversine distance (<= 40 km to Тверской уезд)
    let mednoye_issue = issues
        .iter()
        .find(|i| i["place_id"].as_str() == Some(mednoye_id.as_str()))
        .expect("Mednoye orphan issue should be detected");
    assert_eq!(mednoye_issue["issue_type"], "orphan_hierarchy");
    assert_eq!(mednoye_issue["category"], "orphans");
    assert_eq!(mednoye_issue["rule_code"], "PLACE_ORPHAN_HIERARCHY");
    assert_eq!(mednoye_issue["proposed_parent_id"], tver_district_id);
    assert_eq!(mednoye_issue["proposed_clean_name"], "Медное");
    assert_eq!(mednoye_issue["proposed_place_type"], "village");
    let med_chain = mednoye_issue["proposed_chain"].as_array().unwrap();
    assert_eq!(med_chain[0]["action"], "reuse");
    assert_eq!(med_chain.last().unwrap()["action"], "clean_leaf");
    let dist_km = mednoye_issue["distance_km"].as_f64().unwrap();
    assert!(
        dist_km > 15.0 && dist_km <= 40.0,
        "Expected Haversine distance ~26.7 km, got {dist_km}"
    );

    // 4. Execute Batch Auto-Fix (`POST /api/v1/quality/places/batch-fix`)
    let (bf_st, bf_resp) = post_json(
        &app,
        "/api/v1/quality/places/batch-fix",
        &admin_auth,
        json!({
            "min_confidence": 85
        }),
    )
    .await;
    assert_eq!(bf_st, StatusCode::OK, "batch-fix failed: {bf_resp}");
    let tx_id = bf_resp["tx_id"].as_str().unwrap().to_string();
    assert!(bf_resp["fixed_count"].as_u64().unwrap() >= 2);
    assert!(
        bf_resp["created_parents_count"].as_u64().unwrap() >= 3,
        "Should have created Российская империя, Самарская губерния, Новоузенский уезд: {bf_resp}"
    );

    // Verify Pokrovka is now clean leaf "Покровка" with full hierarchy and original path archived in alt_names
    let (pk_st, pk_detail) =
        get_json(&app, &format!("/api/v1/places/{pokrovka_id}"), &admin_auth).await;
    assert_eq!(pk_st, StatusCode::OK);
    assert_eq!(pk_detail["place"]["name"], "Покровка");
    assert_eq!(pk_detail["place"]["place_type"], "village");
    assert_eq!(
        pk_detail["place"]["full_name"],
        "Покровка, Новоузенский уезд, Самарская губерния, Российская империя"
    );
    let breadcrumbs = pk_detail["breadcrumbs"].as_array().unwrap();
    assert_eq!(breadcrumbs.len(), 4);
    assert_eq!(breadcrumbs[0]["name"], "Российская империя");
    assert_eq!(breadcrumbs[1]["name"], "Самарская губерния");
    assert_eq!(breadcrumbs[2]["name"], "Новоузенский уезд");
    assert_eq!(breadcrumbs[3]["name"], "Покровка");

    let alt_names = pk_detail["place"]["alt_names"].as_array().unwrap();
    assert!(
        alt_names.iter().any(|a| {
            a["name"] == "Покровка, Новоузенский уезд, Самарская губерния, Российская империя"
        }),
        "Original unparsed path must be preserved in alt_names: {alt_names:?}"
    );
    assert_eq!(pk_detail["place"]["events_count"], 1);

    // Verify Mednoye is now linked under Тверской уезд -> Тверская губерния
    let (_, md_detail) = get_json(&app, &format!("/api/v1/places/{mednoye_id}"), &admin_auth).await;
    assert_eq!(md_detail["place"]["name"], "Медное");
    assert_eq!(md_detail["place"]["place_type"], "village");
    assert_eq!(md_detail["place"]["parent_id"], tver_district_id);
    assert_eq!(
        md_detail["place"]["full_name"],
        "Медное, Тверской уезд, Тверская губерния"
    );

    // 5. Execute 1-Click Atomic Undo (`POST /api/v1/history/tx/{tx_id}/undo`)
    let (undo_st, undo_resp) = post_json(
        &app,
        &format!("/api/v1/history/tx/{tx_id}/undo"),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(undo_st, StatusCode::OK, "undo failed: {undo_resp}");
    assert!(
        undo_resp["undone_count"].as_u64().unwrap() >= 5,
        "Expected >= 5 undone entries in {undo_resp}"
    );

    // Verify Pokrovka and Mednoye are restored to their original unparsed/orphan state
    let (_, pk_after_undo) =
        get_json(&app, &format!("/api/v1/places/{pokrovka_id}"), &admin_auth).await;
    assert_eq!(
        pk_after_undo["place"]["name"],
        "Покровка, Новоузенский уезд, Самарская губерния, Российская империя"
    );
    assert!(pk_after_undo["place"]["parent_id"].is_null());

    let (_, md_after_undo) =
        get_json(&app, &format!("/api/v1/places/{mednoye_id}"), &admin_auth).await;
    assert_eq!(md_after_undo["place"]["name"], "с. Медное");
    assert!(md_after_undo["place"]["parent_id"].is_null());

    // 6. Test Single-Place Fix (`POST /api/v1/quality/places/fix`) and Duplicate Node Consolidation
    let (single_st, single_resp) = post_json(
        &app,
        "/api/v1/quality/places/fix",
        &admin_auth,
        json!({ "place_id": pokrovka_id }),
    )
    .await;
    assert_eq!(
        single_st,
        StatusCode::OK,
        "single fix failed: {single_resp}"
    );
    assert_eq!(single_resp["place"]["name"], "Покровка");
    assert_eq!(single_resp["breadcrumbs"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn test_place_hierarchy_rbac_guards_and_dismiss() {
    let (app, admin_auth, editor_auth, viewer_auth) = setup_test_env().await;

    let (_, place) = post_json(
        &app,
        "/api/v1/places",
        &admin_auth,
        json!({
            "name": "Тверь, Тверская губерния, Российская империя",
            "place_type": "other"
        }),
    )
    .await;
    let place_id = place["id"].as_str().unwrap().to_string();

    // Viewer and Editor must receive 403 Forbidden on all `/api/v1/quality/places/*` endpoints
    for auth in [&viewer_auth, &editor_auth] {
        let (s1, _) = get_json(&app, "/api/v1/quality/places/analysis", auth).await;
        assert_eq!(s1, StatusCode::FORBIDDEN);

        let (s2, _) = get_json(
            &app,
            &format!("/api/v1/quality/places/preview?id={place_id}"),
            auth,
        )
        .await;
        assert_eq!(s2, StatusCode::FORBIDDEN);

        let (s3, _) = post_json(
            &app,
            "/api/v1/quality/places/fix",
            auth,
            json!({ "place_id": place_id }),
        )
        .await;
        assert_eq!(s3, StatusCode::FORBIDDEN);

        let (s4, _) = post_json(&app, "/api/v1/quality/places/batch-fix", auth, json!({})).await;
        assert_eq!(s4, StatusCode::FORBIDDEN);

        let (s5, _) = post_json(
            &app,
            "/api/v1/quality/places/dismiss",
            auth,
            json!({ "place_id": place_id }),
        )
        .await;
        assert_eq!(s5, StatusCode::FORBIDDEN);
    }

    // Admin can dismiss the place hierarchy suggestion
    let (dis_st, dis_resp) = post_json(
        &app,
        "/api/v1/quality/places/dismiss",
        &admin_auth,
        json!({ "place_id": place_id }),
    )
    .await;
    assert_eq!(dis_st, StatusCode::OK);
    assert_eq!(dis_resp["status"], "dismissed");
    assert_eq!(dis_resp["dismissed"]["entity_type"], "place");
    assert_eq!(dis_resp["dismissed"]["entity_id"], place_id);

    let (_, analysis_after) = get_json(&app, "/api/v1/quality/places/analysis", &admin_auth).await;
    assert_eq!(analysis_after["total"], 0);
}
