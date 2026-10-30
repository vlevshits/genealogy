use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use base64::Engine;
use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::{
        auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
        dashboard::{AnniversariesListResponse, DashboardResponse, TreeStatsResponse},
        genealogy::{FanChartResponse, KinshipResponse, PlacesMapResponse},
        history::UndoTransactionResponse,
        privacy::TreePersonNodeDto,
        quality::{
            DismissWarningResponse, DuplicateCandidatesListResponse, MergePreviewResponse,
            NotDuplicatePairResponse, PersonMatchResponse, PersonMergeResponse,
            QualitySummaryResponse, QualityWarningsListResponse,
        },
    },
    state::AppState,
};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn setup_phase2_env(enable_public_access: bool) -> (axum::Router, String, String, String) {
    let temp_dir =
        std::env::temp_dir().join(format!("genealogy_phase2_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Phase 2 Delight Test Tree".to_string(),
        enable_public_access,
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
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

async fn get_json(app: &axum::Router, uri: &str, auth: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder().method(Method::GET).uri(uri);
    if let Some(a) = auth {
        req = req.header(header::AUTHORIZATION, a);
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

#[tokio::test]
async fn test_all_10_quality_rules_fuzzy_dates_and_dismissal_memory() {
    let (app, admin_auth, editor_auth, viewer_auth) = setup_phase2_env(false).await;

    // 1. DEATH_BEFORE_BIRTH (hard chronological inversion: born 1950, died 1940)
    let (st, p_death_before_birth) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Chrono",
            "surname": "Inverted",
            "birth_date": "1950",
            "death_date": "1940",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let p_dbb_id = p_death_before_birth["id"].as_str().unwrap().to_string();

    // Fuzzy date non-false-positive check: born "ABT 1950" (window 1945..=1955), died "ABT 1949" (window 1944..=1954)
    let (st, p_fuzzy_ok) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Fuzzy",
            "surname": "Plausible",
            "birth_date": "ABT 1950",
            "death_date": "ABT 1949",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let p_fuzzy_id = p_fuzzy_ok["id"].as_str().unwrap().to_string();

    // 2. TOO_OLD (born 1870, died 1995 -> age 125 > 110)
    let (st, p_too_old) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Methuselah",
            "surname": "Ancient",
            "birth_date": "1870",
            "death_date": "1995",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let p_too_old_id = p_too_old["id"].as_str().unwrap().to_string();

    // 3. PARENT_TOO_YOUNG & 4. MOTHER_TOO_OLD & 5. BORN_AFTER_PARENT_DEATH & 6. MARRIED_TOO_YOUNG
    // Young father: born 1930, married in 1941 (age 11 -> MARRIED_TOO_YOUNG), child born 1940 (age 10 -> PARENT_TOO_YOUNG)
    let (_, p_young_father) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Young",
            "surname": "Father",
            "birth_date": "1930",
            "death_date": "1990",
            "is_living": false
        }),
    )
    .await;
    let young_father_id = p_young_father["id"].as_str().unwrap().to_string();

    // Old mother who also died before child birth: born 1880, died 1935, child born 1940 (age 60 -> MOTHER_TOO_OLD, died 1935 < 1940 -> BORN_AFTER_PARENT_DEATH)
    let (_, p_old_mother) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Old",
            "surname": "Mother",
            "birth_date": "1880",
            "death_date": "1935",
            "is_living": false
        }),
    )
    .await;
    let old_mother_id = p_old_mother["id"].as_str().unwrap().to_string();

    let (_, p_child) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Miracle",
            "surname": "Child",
            "birth_date": "1940",
            "death_date": "1980",
            "is_living": false
        }),
    )
    .await;
    let child_id = p_child["id"].as_str().unwrap().to_string();

    // Family linking young_father + old_mother + child, with marriage in 1941 (young_father age 11)
    let (st, fam) = post_json(
        &app,
        "/api/v1/families",
        &editor_auth,
        json!({
            "partner1_id": young_father_id,
            "partner2_id": old_mother_id,
            "children_ids": [child_id]
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);
    let fam_id = fam["id"].as_str().unwrap().to_string();

    let (st, _) = post_json(
        &app,
        "/api/v1/events",
        &editor_auth,
        json!({
            "family_id": fam_id,
            "event_type": "marriage",
            "date_text": "1941"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);

    // 7. EVENT_AFTER_DEATH: add residence event in 1985 for p_child (died 1980), plus burial in 1981 (which is allowed!)
    let (st, _) = post_json(
        &app,
        "/api/v1/events",
        &editor_auth,
        json!({
            "individual_id": child_id,
            "event_type": "residence",
            "date_text": "1985"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);

    let (st, _) = post_json(
        &app,
        "/api/v1/events",
        &editor_auth,
        json!({
            "individual_id": child_id,
            "event_type": "burial",
            "date_text": "1981"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED);

    // 8. POSSIBLE_DUPLICATE & 9. MISSING_BIRTH & 10. UNCONNECTED
    // Create two duplicate deceased individuals without birth dates (triggers MISSING_BIRTH, UNCONNECTED, and POSSIBLE_DUPLICATE when given matching birth year)
    let (_, p_missing_birth) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Nadezhda",
            "surname": "Bezdatova",
            "death_date": "1975",
            "is_living": false
        }),
    )
    .await;
    let missing_birth_id = p_missing_birth["id"].as_str().unwrap().to_string();

    let (_, p_dup1) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Nikolai",
            "patronymic": "Petrovich",
            "surname": "Sokolov",
            "birth_date": "1892",
            "birth_place": "Tula",
            "death_date": "1960",
            "is_living": false
        }),
    )
    .await;
    let dup1_id = p_dup1["id"].as_str().unwrap().to_string();

    let (_, p_dup2) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Nikolai",
            "patronymic": "Petrovich",
            "surname": "Sokolov",
            "birth_date": "1892",
            "birth_place": "Tula",
            "death_date": "1960",
            "is_living": false
        }),
    )
    .await;
    let dup2_id = p_dup2["id"].as_str().unwrap().to_string();

    // Viewer AND Editor should be forbidden from Admin-only /api/v1/quality/* (`15-data-quality-duplicates.md §4, §6`)
    let (st_viewer, _) = get_json(&app, "/api/v1/quality/warnings", Some(&viewer_auth)).await;
    assert_eq!(st_viewer, StatusCode::FORBIDDEN);
    let (st_editor_q, _) = get_json(&app, "/api/v1/quality/summary", Some(&editor_auth)).await;
    assert_eq!(st_editor_q, StatusCode::FORBIDDEN);

    // Admin queries /api/v1/quality/summary
    let (st_sum, sum_val) = get_json(&app, "/api/v1/quality/summary", Some(&admin_auth)).await;
    assert_eq!(st_sum, StatusCode::OK);
    let summary: QualitySummaryResponse = serde_json::from_value(sum_val).unwrap();
    assert_eq!(summary.operation, "data_quality");
    assert!(summary.total_open_warnings >= 10);
    assert!(summary.duplicate_candidates_count >= 1);

    let rule_codes: Vec<&str> = summary.warnings.iter().map(|w| w.code.as_str()).collect();
    for expected_rule in [
        "DEATH_BEFORE_BIRTH",
        "TOO_OLD",
        "PARENT_TOO_YOUNG",
        "MOTHER_TOO_OLD",
        "BORN_AFTER_PARENT_DEATH",
        "MARRIED_TOO_YOUNG",
        "EVENT_AFTER_DEATH",
        "POSSIBLE_DUPLICATE",
        "MISSING_BIRTH",
        "UNCONNECTED",
    ] {
        assert!(
            rule_codes.contains(&expected_rule),
            "Expected quality rule {} to be triggered, found {:?}",
            expected_rule,
            rule_codes
        );
    }

    // Verify fuzzy date person did NOT trigger DEATH_BEFORE_BIRTH
    assert!(
        !summary
            .warnings
            .iter()
            .any(|w| w.code == "DEATH_BEFORE_BIRTH" && w.person.id == p_fuzzy_id),
        "Fuzzy date person should not trigger false-positive DEATH_BEFORE_BIRTH"
    );

    // Verify p_dbb_id DID trigger DEATH_BEFORE_BIRTH
    let dbb_warning = summary
        .warnings
        .iter()
        .find(|w| w.code == "DEATH_BEFORE_BIRTH" && w.person.id == p_dbb_id)
        .expect("Expected DEATH_BEFORE_BIRTH on p_dbb_id");

    // Verify p_too_old_id triggered TOO_OLD
    assert!(summary
        .warnings
        .iter()
        .any(|w| w.code == "TOO_OLD" && w.person.id == p_too_old_id));

    // Verify missing_birth_id triggered MISSING_BIRTH
    assert!(summary
        .warnings
        .iter()
        .any(|w| w.code == "MISSING_BIRTH" && w.person.id == missing_birth_id));

    // Test dismissing the DEATH_BEFORE_BIRTH warning by ID (Admin)
    let (st_dis, dis_val) = post_json(
        &app,
        &format!("/api/v1/quality/warnings/{}/dismiss", dbb_warning.id),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(st_dis, StatusCode::OK);
    let dis_resp: DismissWarningResponse = serde_json::from_value(dis_val).unwrap();
    assert_eq!(dis_resp.dismissed.rule_code, "DEATH_BEFORE_BIRTH");

    // Verify dismissed warning is hidden by default in GET /api/v1/quality/warnings?rule=DEATH_BEFORE_BIRTH
    let (st_w, w_val) = get_json(
        &app,
        "/api/v1/quality/warnings?rule=DEATH_BEFORE_BIRTH",
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st_w, StatusCode::OK);
    let w_list: QualityWarningsListResponse = serde_json::from_value(w_val).unwrap();
    assert!(!w_list.items.iter().any(|w| w.id == dbb_warning.id));

    // Verify it is returned with is_dismissed=true when include_dismissed=true
    let (_, w_inc_val) = get_json(
        &app,
        "/api/v1/quality/warnings?rule=DEATH_BEFORE_BIRTH&include_dismissed=true",
        Some(&admin_auth),
    )
    .await;
    let w_inc_list: QualityWarningsListResponse = serde_json::from_value(w_inc_val).unwrap();
    assert!(w_inc_list
        .items
        .iter()
        .any(|w| w.id == dbb_warning.id && w.is_dismissed));

    // Test marking dup1 and dup2 as not-duplicate (Admin)
    let (st_nd, nd_val) = post_json(
        &app,
        "/api/v1/quality/duplicates/not-duplicate",
        &admin_auth,
        json!({
            "person_a_id": dup1_id,
            "person_b_id": dup2_id
        }),
    )
    .await;
    assert_eq!(st_nd, StatusCode::OK);
    let _nd_resp: NotDuplicatePairResponse = serde_json::from_value(nd_val).unwrap();

    // Verify dup1+dup2 no longer appear in /api/v1/quality/duplicates
    let (_, dups_val) = get_json(&app, "/api/v1/quality/duplicates", Some(&admin_auth)).await;
    let dups_list: DuplicateCandidatesListResponse = serde_json::from_value(dups_val).unwrap();
    assert!(!dups_list.items.iter().any(|d| {
        (d.person_a.id == dup1_id && d.person_b.id == dup2_id)
            || (d.person_a.id == dup2_id && d.person_b.id == dup1_id)
    }));
}

#[tokio::test]
async fn test_inline_match_cross_script_and_atomic_person_merge_with_gid_alias_and_undo() {
    let (app, admin_auth, editor_auth, _viewer_auth) = setup_phase2_env(false).await;

    // Create Person A (keep_person): Cyrillic "Пётр Иванович Левшиц", born 1885 in Vitebsk
    let (_, person_a) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Пётр",
            "patronymic": "Иванович",
            "surname": "Левшиц",
            "birth_date": "1885",
            "birth_place": "Vitebsk",
            "is_living": false
        }),
    )
    .await;
    let id_a = person_a["id"].as_str().unwrap().to_string();
    let gid_a = person_a["gid"].as_str().unwrap().to_string();

    // Test inline duplicate matching (`GET /api/v1/people/match` — accessible to Editors & Admins) with Latin transliterated name
    let (st_match, match_val) = get_json(
        &app,
        "/api/v1/people/match?given_name=Petr&surname=Levshits&birth_year=1885&sex=M",
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_match, StatusCode::OK);
    let match_resp: PersonMatchResponse = serde_json::from_value(match_val).unwrap();
    assert!(
        match_resp
            .items
            .iter()
            .any(|c| c.person.id == id_a && c.score >= 60),
        "Expected cross-script inline match for Пётр Левшиц / Petr Levshits, got {:?}",
        match_resp.items
    );

    // Create Person B (merge_person): Latin "Petr Levshits", born 1885, died 1952 in Leningrad, with a note and event
    let (_, person_b) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Petr",
            "patronymic": "Ivanovich",
            "surname": "Levshits",
            "birth_date": "1885",
            "death_date": "1952",
            "death_place": "Leningrad",
            "is_living": false
        }),
    )
    .await;
    let id_b = person_b["id"].as_str().unwrap().to_string();
    let gid_b = person_b["gid"].as_str().unwrap().to_string();

    // Add a research note and an occupation event to Person B
    let (st_note, _) = post_json(
        &app,
        "/api/v1/notes",
        &editor_auth,
        json!({
            "entity_type": "individual",
            "entity_id": id_b,
            "kind": "research",
            "body": "Archival record from Vitebsk governorate."
        }),
    )
    .await;
    assert_eq!(st_note, StatusCode::CREATED);

    let (st_ev, _) = post_json(
        &app,
        "/api/v1/events",
        &editor_auth,
        json!({
            "individual_id": id_b,
            "event_type": "occupation",
            "date_text": "1912",
            "description": "Watchmaker"
        }),
    )
    .await;
    assert_eq!(st_ev, StatusCode::CREATED);

    // Create a child of Person B to verify family relinking during merge
    let (_, child_b) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Sofia",
            "surname": "Levshits",
            "birth_date": "1915",
            "death_date": "1990",
            "is_living": false
        }),
    )
    .await;
    let child_b_id = child_b["id"].as_str().unwrap().to_string();

    let (st_fam, _) = post_json(
        &app,
        "/api/v1/families",
        &editor_auth,
        json!({
            "partner1_id": id_b,
            "children_ids": [child_b_id]
        }),
    )
    .await;
    assert_eq!(st_fam, StatusCode::CREATED);

    // Verify Editor receives 403 Forbidden on Admin-only merge preview (`15-data-quality-duplicates.md §5, §6`)
    let (st_editor_prev, _) = get_json(
        &app,
        &format!(
            "/api/v1/people/merge/preview?keep_id={}&merge_id={}",
            id_a, id_b
        ),
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_editor_prev, StatusCode::FORBIDDEN);

    // 1. Test cycle prevention (Admin): previewing or merging Person B with his own child `child_b_id` must detect cycle!
    let (st_prev_cycle, prev_cycle_val) = get_json(
        &app,
        &format!(
            "/api/v1/people/merge/preview?keep_id={}&merge_id={}",
            id_b, child_b_id
        ),
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st_prev_cycle, StatusCode::OK);
    let prev_cycle: MergePreviewResponse = serde_json::from_value(prev_cycle_val).unwrap();
    assert!(!prev_cycle.can_merge);
    assert!(prev_cycle.cycle_detected);

    let (st_merge_cycle, err_cycle) = post_json(
        &app,
        "/api/v1/people/merge",
        &admin_auth,
        json!({
            "keep_id": id_b,
            "merge_id": child_b_id
        }),
    )
    .await;
    assert_eq!(st_merge_cycle, StatusCode::BAD_REQUEST);
    assert_eq!(err_cycle["error"]["code"], "cycle_detected");

    // 2. Preview valid merge between Person A and Person B (Admin)
    let (st_prev, prev_val) = get_json(
        &app,
        &format!(
            "/api/v1/people/merge/preview?target_id={}&source_id={}",
            id_a, id_b
        ),
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st_prev, StatusCode::OK);
    let preview: MergePreviewResponse = serde_json::from_value(prev_val).unwrap();
    assert!(preview.can_merge);
    assert!(!preview.cycle_detected);
    assert_eq!(preview.target.id, id_a);
    assert_eq!(preview.source.id, id_b);
    assert!(preview.notes_to_transfer >= 1);
    assert!(preview.families_affected >= 1);
    assert_eq!(preview.combined_counts.notes, preview.notes_to_transfer);
    assert_eq!(preview.combined_counts.families, preview.families_affected);

    // 3. Execute atomic merge keeping Person A, choosing death from Person B (Admin)
    let (st_merge, merge_val) = post_json(
        &app,
        "/api/v1/people/merge",
        &admin_auth,
        json!({
            "target_id": id_a,
            "source_id": id_b,
            "field_choices": {
                "name": "target",
                "death_date": "source"
            }
        }),
    )
    .await;
    assert_eq!(st_merge, StatusCode::OK);
    let merge_resp: PersonMergeResponse = serde_json::from_value(merge_val).unwrap();
    assert_eq!(merge_resp.operation, "person_merge");
    assert_eq!(merge_resp.merged_gid.as_deref(), Some(gid_b.as_str()));
    assert_eq!(
        merge_resp.surviving_person_id.as_deref(),
        Some(id_a.as_str())
    );
    let tx_id = merge_resp.tx_id.expect("Expected tx_id from person merge");
    assert!(merge_resp.snapshot_id.is_some());

    let surviving = merge_resp
        .surviving_person
        .expect("Expected surviving_person");
    assert_eq!(surviving.id, id_a);
    assert_eq!(surviving.gid, gid_a);
    // Non-chosen primary name ("Petr Levshits") should be preserved as an alternate name (`name_type == "aka"`)
    assert!(
        surviving
            .alt_names
            .iter()
            .any(|a| a.name_type == "aka" && a.given_name.as_deref() == Some("Petr")),
        "Expected merged primary name to be saved in alt_names as aka, got {:?}",
        surviving.alt_names
    );
    // Death date from Person B ("1952") should be on surviving person
    assert_eq!(
        surviving.death.as_ref().and_then(|d| d.date.as_deref()),
        Some("1952")
    );

    // 4. Verify GID alias redirection: fetching `/api/v1/persons/{gid_b}` resolves to surviving Person A!
    let (st_alias, alias_val) = get_json(
        &app,
        &format!("/api/v1/persons/{}", gid_b),
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_alias, StatusCode::OK);
    let aliased_person: TreePersonNodeDto = serde_json::from_value(alias_val).unwrap();
    assert_eq!(aliased_person.id, id_a);

    // 5. Undo the merge transaction (`POST /api/v1/history/tx/{tx_id}/undo`) and verify both records are restored!
    let (st_undo, undo_val) = post_json(
        &app,
        &format!("/api/v1/history/tx/{}/undo", tx_id),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(st_undo, StatusCode::OK);
    let undo_resp: UndoTransactionResponse = serde_json::from_value(undo_val).unwrap();
    assert_eq!(undo_resp.status, "ok");
    assert!(undo_resp.undone_count >= 1);

    // Fetching `/api/v1/persons/{gid_b}` now returns the restored Person B with its original ID `id_b`!
    let (st_restored, restored_val) = get_json(
        &app,
        &format!("/api/v1/persons/{}", gid_b),
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_restored, StatusCode::OK);
    let restored_b: TreePersonNodeDto = serde_json::from_value(restored_val).unwrap();
    assert_eq!(restored_b.id, id_b);
}

#[tokio::test]
async fn test_dashboard_julian_anniversaries_fan_chart_kinship_and_places_map() {
    let (app, admin_auth, editor_auth, viewer_auth) = setup_phase2_env(true).await;

    // Create a 3-generation family tree with geocoded places and Julian calendar birth date:
    // Paternal Grandfather (born @#DJULIAN@ 20 APR 1890 -> +12 days = May 2 Gregorian; died 1976-05-10 -> 50th memorial milestone in 2026!)
    let (_, gf) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Ilya",
            "surname": "Levshits",
            "birth_date": "@#DJULIAN@ 20 APR 1890",
            "birth_place": "Minsk",
            "death_date": "10 MAY 1976",
            "is_living": false
        }),
    )
    .await;
    let gf_id = gf["id"].as_str().unwrap().to_string();

    let (_, gm) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Hannah",
            "surname": "Levshits",
            "birth_date": "1895",
            "death_date": "1980",
            "is_living": false
        }),
    )
    .await;
    let gm_id = gm["id"].as_str().unwrap().to_string();

    let (_, father) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Boris",
            "surname": "Levshits",
            "birth_date": "1920",
            "death_date": "1995",
            "is_living": false
        }),
    )
    .await;
    let father_id = father["id"].as_str().unwrap().to_string();

    let (_, aunt) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Raisa",
            "surname": "Levshits",
            "birth_date": "1922",
            "death_date": "2001",
            "is_living": false
        }),
    )
    .await;
    let aunt_id = aunt["id"].as_str().unwrap().to_string();

    // Living root person (born 15 MAY 1990)
    let (_, root_person) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Maxim",
            "surname": "Levshits",
            "birth_date": "15 MAY 1990",
            "is_living": true
        }),
    )
    .await;
    let root_id = root_person["id"].as_str().unwrap().to_string();

    // Private person (is_private: true)
    let (_, priv_person) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Secret",
            "surname": "Relative",
            "birth_date": "1950",
            "is_private": true
        }),
    )
    .await;
    let priv_id = priv_person["id"].as_str().unwrap().to_string();

    // Link GF + GM -> [Father, Aunt]
    post_json(
        &app,
        "/api/v1/families",
        &editor_auth,
        json!({
            "partner1_id": gf_id,
            "partner2_id": gm_id,
            "children_ids": [father_id, aunt_id]
        }),
    )
    .await;

    // Link Father -> [Root]
    post_json(
        &app,
        "/api/v1/families",
        &editor_auth,
        json!({
            "partner1_id": father_id,
            "children_ids": [root_id]
        }),
    )
    .await;

    // Create a geocoded place ("Minsk", 53.9006, 27.5590)
    let (st_place, _) = post_json(
        &app,
        "/api/v1/places",
        &editor_auth,
        json!({
            "name": "Minsk",
            "place_type": "city",
            "latitude": 53.9006,
            "longitude": 27.5590
        }),
    )
    .await;
    assert_eq!(st_place, StatusCode::CREATED);

    // 1. Test Anniversaries with Julian-to-Gregorian conversion (`reference_date=2026-05-01`, `days=30`)
    // GF born `@#DJULIAN@ 20 APR 1890` -> Gregorian May 2 (`next_occurrence_date: "2026-05-02"`, `days_until: 1`)
    // GF died `10 MAY 1976` -> 50th memorial milestone on `2026-05-10` (`is_milestone: true`, `years_elapsed: 50`)
    // Living Root born `15 MAY 1990` -> visible to authenticated Editor, hidden from anonymous Guest!
    let (st_ann, ann_val) = get_json(
        &app,
        "/api/v1/anniversaries?days=30&reference_date=2026-05-01",
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_ann, StatusCode::OK);
    let ann_resp: AnniversariesListResponse = serde_json::from_value(ann_val).unwrap();

    let gf_bday = ann_resp
        .items
        .iter()
        .find(|i| i.person.id == gf_id && i.kind == "birthday")
        .expect("Expected Julian-shifted birthday for Grandfather Ilya");
    assert_eq!(gf_bday.calendar, "julian");
    assert_eq!(gf_bday.month, 5);
    assert_eq!(gf_bday.day, 2);
    assert_eq!(gf_bday.next_occurrence_date, "2026-05-02");
    assert_eq!(gf_bday.days_until, 1);

    let gf_memorial = ann_resp
        .items
        .iter()
        .find(|i| i.person.id == gf_id && i.kind == "memorial")
        .expect("Expected 50-year milestone memorial for Grandfather Ilya");
    assert_eq!(gf_memorial.years_elapsed, 50);
    assert!(gf_memorial.is_milestone);

    assert!(
        ann_resp.items.iter().any(|i| i.person.id == root_id),
        "Authenticated editor should see living person's birthday"
    );

    // Anonymous public guest should NOT see living person's birthday in /api/v1/anniversaries
    let (st_guest_ann, guest_ann_val) = get_json(
        &app,
        "/api/v1/anniversaries?days=30&reference_date=2026-05-01",
        None,
    )
    .await;
    assert_eq!(st_guest_ann, StatusCode::OK);
    let guest_ann: AnniversariesListResponse = serde_json::from_value(guest_ann_val).unwrap();
    assert!(
        !guest_ann.items.iter().any(|i| i.person.id == root_id),
        "Public guest must never see living person's birthday"
    );

    // 2. Test Dashboard (`GET /api/v1/dashboard`) for Admin vs Editor vs Anonymous Guest
    let (st_admin_dash, admin_dash_val) = get_json(
        &app,
        "/api/v1/dashboard?reference_date=2026-05-01",
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st_admin_dash, StatusCode::OK);
    let admin_dash: DashboardResponse = serde_json::from_value(admin_dash_val).unwrap();
    assert!(!admin_dash.is_empty);
    assert!(admin_dash.quality_summary.is_some());
    assert!(!admin_dash.recent_activity.is_empty());
    assert_eq!(admin_dash.anniversaries, admin_dash.upcoming_anniversaries);

    let (st_dash, dash_val) = get_json(
        &app,
        "/api/v1/dashboard?reference_date=2026-05-01",
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_dash, StatusCode::OK);
    let dash: DashboardResponse = serde_json::from_value(dash_val).unwrap();
    assert!(!dash.is_empty);
    assert!(dash.quality_summary.is_none());
    assert!(!dash.recent_activity.is_empty());

    let (st_guest_dash, guest_dash_val) =
        get_json(&app, "/api/v1/dashboard?reference_date=2026-05-01", None).await;
    assert_eq!(st_guest_dash, StatusCode::OK);
    let guest_dash: DashboardResponse = serde_json::from_value(guest_dash_val).unwrap();
    assert!(
        guest_dash.quality_summary.is_none(),
        "Guest dashboard must not expose quality_summary"
    );
    assert!(
        guest_dash.recent_activity.is_empty(),
        "Guest dashboard must not expose recent_activity"
    );
    assert!(
        guest_dash.latest_photos.is_empty(),
        "Guest dashboard must not expose latest_photos"
    );

    // 3. Test Stats (`GET /api/v1/stats`)
    let (st_stats, stats_val) = get_json(&app, "/api/v1/stats", Some(&editor_auth)).await;
    assert_eq!(st_stats, StatusCode::OK);
    let stats: TreeStatsResponse = serde_json::from_value(stats_val).unwrap();
    assert!(stats.total_persons >= 5);
    assert!(stats.living_count >= 1);
    assert!(stats.deceased_count >= 4);
    assert!(stats.generations_count >= 3);
    assert!(stats.oldest_ancestor.is_some());

    // 4. Test Fan Chart (`GET /api/v1/tree/fan?root={root_id}&generations=5`)
    let (st_fan, fan_val) = get_json(
        &app,
        &format!("/api/v1/tree/fan?root={}&generations=5", root_id),
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_fan, StatusCode::OK);
    let fan: FanChartResponse = serde_json::from_value(fan_val).unwrap();
    assert_eq!(fan.root_id, root_id);
    assert_eq!(fan.total_nodes, 4); // root (1), father (2), paternal_gf (4), paternal_gm (5)
    assert!(fan
        .nodes
        .iter()
        .any(|n| n.ahnentafel == 1 && n.person.id == root_id));
    assert!(fan
        .nodes
        .iter()
        .any(|n| n.ahnentafel == 2 && n.person.id == father_id && n.quadrant == "paternal"));
    assert!(fan.nodes.iter().any(|n| n.ahnentafel == 4
        && n.person.id == gf_id
        && n.quadrant == "paternal_grandfather"));
    assert!(fan.nodes.iter().any(|n| n.ahnentafel == 5
        && n.person.id == gm_id
        && n.quadrant == "paternal_grandmother"));

    // 5. Test Kinship Finder (`GET /api/v1/kinship?from={root_id}&to={aunt_id}`)
    let (st_kin, kin_val) = get_json(
        &app,
        &format!("/api/v1/kinship?from={}&to={}", root_id, aunt_id),
        Some(&editor_auth),
    )
    .await;
    assert_eq!(st_kin, StatusCode::OK);
    let kin: KinshipResponse = serde_json::from_value(kin_val).unwrap();
    assert!(kin.connected);
    assert_eq!(kin.relationship_key, "paternal_aunt");
    assert_eq!(kin.degree_of_kinship, 3);
    assert_eq!(kin.up_steps, 2);
    assert_eq!(kin.down_steps, 1);

    // Public guest querying kinship involving living `root_id` gets masked living node (`M.`)
    let (st_guest_kin, guest_kin_val) = get_json(
        &app,
        &format!("/api/v1/kinship?from={}&to={}", root_id, aunt_id),
        None,
    )
    .await;
    assert_eq!(st_guest_kin, StatusCode::OK);
    let guest_kin: KinshipResponse = serde_json::from_value(guest_kin_val).unwrap();
    assert_eq!(guest_kin.path_nodes[0].primary_name.given_name, "M.");
    assert!(guest_kin.path_nodes[0].birth.is_none());

    // Viewer or Guest querying kinship involving private `priv_id` must get 404!
    let (st_priv_kin, _) = get_json(
        &app,
        &format!("/api/v1/kinship?from={}&to={}", priv_id, aunt_id),
        Some(&viewer_auth),
    )
    .await;
    assert_eq!(st_priv_kin, StatusCode::NOT_FOUND);

    // 6. Test Clustered Places Map (`GET /api/v1/places/map`)
    // Unauthenticated guest must be rejected with 401 Unauthorized even when public tree sharing is enabled!
    let (st_guest_map, _) = get_json(&app, "/api/v1/places/map", None).await;
    assert_eq!(st_guest_map, StatusCode::UNAUTHORIZED);

    let (st_map, map_val) = get_json(&app, "/api/v1/places/map", Some(&editor_auth)).await;
    assert_eq!(st_map, StatusCode::OK);
    let map_resp: PlacesMapResponse = serde_json::from_value(map_val).unwrap();
    assert!(map_resp.total >= 1);
    let minsk_marker = map_resp
        .items
        .iter()
        .find(|m| m.name == "Minsk" && (m.latitude - 53.9006).abs() < 0.01)
        .expect("Expected geocoded Minsk marker in /api/v1/places/map");
    assert_eq!(minsk_marker.person_count, 1);
    assert_eq!(minsk_marker.persons[0].id, gf_id);

    // 7. Verify default GenealogyPrivacyStore and AppState::new start completely empty (0 dummy records)
    let default_store = genealogy_backend::services::privacy::GenealogyPrivacyStore::default();
    assert!(default_store.persons.is_empty());
    assert!(default_store.families.is_empty());
    assert!(default_store.events.is_empty());
    assert!(default_store.media.is_empty());
    assert!(default_store.places.is_empty());
    assert!(!default_store.is_demo_seed);
}

#[tokio::test]
async fn test_guest_dashboard_privacy_suppression_with_media_and_activity() {
    let (app, admin_auth, editor_auth, _) = setup_phase2_env(true).await;

    // 1. Create a person to generate recent_activity
    let (st_p, _) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Ivan",
            "surname": "Smirnov",
            "birth_date": "1930-01-01",
            "death_date": "1990-01-01",
            "is_living": false
        }),
    )
    .await;
    assert_eq!(st_p, StatusCode::CREATED);

    // 2. Upload a public photo via editor
    let custom_payload = b"RIFF\x1a\x00\x00\x00WEBPVP8L\x0d\x00\x00\x00/\x00\x00\x00\x10\x07\x10\x11\x11\x88\x88\xfe\x07\x00TEST_PHOTO";
    let b64_data = base64::engine::general_purpose::STANDARD.encode(custom_payload);
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/media")
                .header(header::AUTHORIZATION, &editor_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "file_name": "historic_photo.webp",
                        "mime_type": "image/webp",
                        "title": "Historic Photo",
                        "data": format!("data:image/webp;base64,{b64_data}")
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 3. Admin dashboard should have recent_activity and latest_photos
    let (st_admin, admin_val) = get_json(&app, "/api/v1/dashboard", Some(&admin_auth)).await;
    assert_eq!(st_admin, StatusCode::OK);
    let admin_dash: DashboardResponse = serde_json::from_value(admin_val).unwrap();
    assert!(
        !admin_dash.recent_activity.is_empty(),
        "Admin must see recent activity"
    );
    assert!(
        !admin_dash.latest_photos.is_empty(),
        "Admin must see latest photos"
    );

    // 4. Anonymous Guest dashboard MUST suppress both recent_activity AND latest_photos
    let (st_guest, guest_val) = get_json(&app, "/api/v1/dashboard", None).await;
    assert_eq!(st_guest, StatusCode::OK);
    let guest_dash: DashboardResponse = serde_json::from_value(guest_val).unwrap();
    assert!(
        guest_dash.recent_activity.is_empty(),
        "Guest dashboard must suppress recent_activity completely"
    );
    assert!(
        guest_dash.latest_photos.is_empty(),
        "Guest dashboard must suppress latest_photos completely"
    );
    assert!(
        guest_dash.quality_summary.is_none(),
        "Guest dashboard must suppress quality_summary"
    );
}

#[tokio::test]
async fn test_multi_trace_kinship_ranking_and_max_traces() {
    let (app, admin_auth, _, _) = setup_phase2_env(true).await;

    // 1. Create Grandfather (Pyotr)
    let (_, p_gf) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Pyotr",
            "surname": "Smirnov",
            "sex": "M",
            "birth_date": "1930",
            "death_date": "2000"
        }),
    )
    .await;
    let gf_id = p_gf["id"].as_str().unwrap().to_string();

    // 2. Create Parent 1 (Dmitry, son of Pyotr)
    let (_, p_dmitry) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Dmitry",
            "surname": "Smirnov",
            "sex": "M",
            "birth_date": "1955",
            "death_date": "2020"
        }),
    )
    .await;
    let dmitry_id = p_dmitry["id"].as_str().unwrap().to_string();

    // 3. Create Parent 2 (Elena, daughter of Pyotr)
    let (_, p_elena) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Elena",
            "surname": "Smirnova",
            "sex": "F",
            "birth_date": "1958"
        }),
    )
    .await;
    let elena_id = p_elena["id"].as_str().unwrap().to_string();

    // Link Pyotr -> [Dmitry, Elena]
    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": gf_id,
            "children_ids": [dmitry_id.clone(), elena_id.clone()]
        }),
    )
    .await;

    // 4. Create Person A (Alexey, son of Dmitry -> grandson of Pyotr)
    let (_, p_alexey) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Alexey",
            "surname": "Smirnov",
            "sex": "M",
            "birth_date": "1980"
        }),
    )
    .await;
    let alexey_id = p_alexey["id"].as_str().unwrap().to_string();

    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": dmitry_id,
            "children_ids": [alexey_id.clone()]
        }),
    )
    .await;

    // 5. Create Person B (Olga, daughter of Elena -> granddaughter of Pyotr)
    let (_, p_olga) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Olga",
            "surname": "Kuznetsova",
            "sex": "F",
            "birth_date": "1982"
        }),
    )
    .await;
    let olga_id = p_olga["id"].as_str().unwrap().to_string();

    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": elena_id,
            "children_ids": [olga_id.clone()]
        }),
    )
    .await;

    // 6. Also marry Alexey and Olga (marital connection between first cousins)
    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": alexey_id,
            "partner2_id": olga_id
        }),
    )
    .await;

    // Query kinship with max_traces = 5
    let (st, kin_val) = get_json(
        &app,
        &format!("/api/v1/kinship?from={alexey_id}&to={olga_id}&max_traces=5"),
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let kin: KinshipResponse = serde_json::from_value(kin_val).unwrap();

    // Verify multiple traces were discovered!
    assert!(kin.connected);
    assert_eq!(kin.traces_count, 2);
    assert_eq!(kin.traces.len(), 2);

    // Trace 1 MUST be the biological blood relationship (First cousin), ranked FIRST!
    let trace1 = &kin.traces[0];
    assert!(
        trace1.is_blood_relation,
        "Biological blood relationship must be ranked first"
    );
    assert_eq!(trace1.relationship_key, "first_cousin");
    assert_eq!(trace1.degree_of_kinship, 4);
    assert_eq!(trace1.common_ancestor_ids, vec![gf_id.clone()]);
    assert_eq!(trace1.steps.len(), 5);
    assert_eq!(trace1.steps[0].relation_to_next.as_deref(), Some("Father"));
    assert_eq!(trace1.steps[0].relation_to_next_ru.as_deref(), Some("Отец"));

    // Trace 2 is the affinal/marital relationship (Wife)
    let trace2 = &kin.traces[1];
    assert!(
        !trace2.is_blood_relation,
        "Affinal/marital path must come after blood path"
    );
    assert_eq!(trace2.relationship_key, "wife");
    assert_eq!(trace2.degree_of_kinship, 1);
    assert_eq!(trace2.steps.len(), 2);
    assert_eq!(trace2.steps[0].relation_to_next.as_deref(), Some("Wife"));
    assert_eq!(trace2.steps[0].relation_to_next_ru.as_deref(), Some("Жена"));

    // Top-level fields MUST match Trace 1 for backward compatibility
    assert_eq!(kin.relationship_key, trace1.relationship_key);
    assert_eq!(kin.label_en, trace1.label_en);
    assert_eq!(kin.label_ru, trace1.label_ru);
    assert_eq!(kin.degree_of_kinship, trace1.degree_of_kinship);
    assert_eq!(kin.is_blood_relation, trace1.is_blood_relation);
    assert_eq!(kin.path_person_ids, trace1.path_person_ids);

    // Query with max_traces = 1: should return only 1 trace
    let (st1, kin_val1) = get_json(
        &app,
        &format!("/api/v1/kinship?from={alexey_id}&to={olga_id}&max_traces=1"),
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st1, StatusCode::OK);
    let kin1: KinshipResponse = serde_json::from_value(kin_val1).unwrap();
    assert_eq!(kin1.traces_count, 1);
    assert_eq!(kin1.traces.len(), 1);
    assert_eq!(kin1.traces[0].id, "trace-1");
    assert!(kin1.traces[0].is_blood_relation);
}

#[tokio::test]
async fn test_kinship_prunes_spurious_detours_and_extra_nodes_for_simple_cases() {
    let (app, admin_auth, _, _) = setup_phase2_env(true).await;

    // Viktor (Father) & Lyubov (Mother)
    let (_, p_viktor) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Viktor",
            "surname": "Levshits",
            "sex": "M",
            "birth_date": "1960"
        }),
    )
    .await;
    let viktor_id = p_viktor["id"].as_str().unwrap().to_string();

    let (_, p_lyubov) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Lyubov",
            "surname": "Levshits",
            "sex": "F",
            "birth_date": "1965"
        }),
    )
    .await;
    let lyubov_id = p_lyubov["id"].as_str().unwrap().to_string();

    // Pavel (Son 1) & Valentin (Son 2)
    let (_, p_pavel) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Pavel",
            "surname": "Levshits",
            "sex": "M",
            "birth_date": "1990"
        }),
    )
    .await;
    let pavel_id = p_pavel["id"].as_str().unwrap().to_string();

    let (_, p_valentin) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Valentin",
            "surname": "Levshits",
            "sex": "M",
            "birth_date": "1995"
        }),
    )
    .await;
    let valentin_id = p_valentin["id"].as_str().unwrap().to_string();

    // Family: Viktor & Lyubov -> [Pavel, Valentin]
    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": viktor_id,
            "partner2_id": lyubov_id,
            "children_ids": [pavel_id.clone(), valentin_id.clone()]
        }),
    )
    .await;

    // Test 1: From Viktor (father) to Valentin (son) with max_traces=5
    // Must NOT return fake "stepson" or "relative by marriage" detours through Lyubov or Pavel!
    let (st_father_son, resp_val) = get_json(
        &app,
        &format!("/api/v1/kinship?from={viktor_id}&to={valentin_id}&max_traces=5"),
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st_father_son, StatusCode::OK);
    let father_son: KinshipResponse = serde_json::from_value(resp_val).unwrap();

    // Must have exactly 1 direct trace: Son (degree 1, steps count 1)
    assert!(father_son.connected);
    assert_eq!(
        father_son.traces_count, 1,
        "Father to biological son must have exactly 1 trace"
    );
    assert_eq!(father_son.traces.len(), 1);
    let tr = &father_son.traces[0];
    assert_eq!(tr.relationship_key, "son");
    assert_eq!(tr.degree_of_kinship, 1);
    assert_eq!(
        tr.path_person_ids,
        vec![viktor_id.clone(), valentin_id.clone()]
    );
    assert!(tr.is_blood_relation);

    // Test 2: Between Pavel and Valentin (two full brothers)
    // Must only return direct biological sibling traces via parents, no detours through spouse edge
    let (st_bros, resp_bros) = get_json(
        &app,
        &format!("/api/v1/kinship?from={pavel_id}&to={valentin_id}&max_traces=5"),
        Some(&admin_auth),
    )
    .await;
    assert_eq!(st_bros, StatusCode::OK);
    let brothers: KinshipResponse = serde_json::from_value(resp_bros).unwrap();
    assert!(brothers.connected);
    // Traces must all be biological blood relationship (brother), degree 2
    for b_tr in &brothers.traces {
        assert!(b_tr.is_blood_relation);
        assert_eq!(b_tr.relationship_key, "brother");
        assert_eq!(b_tr.degree_of_kinship, 2);
        assert_eq!(b_tr.steps.len(), 3); // [Pavel, Parent, Valentin]
    }
}

#[tokio::test]
async fn test_duplicate_matching_accuracy_and_false_positive_prevention() {
    let (app, admin_auth, _, _) = setup_phase2_env(false).await;

    // 1. Create two individuals with different given names born in 1987:
    // "Татьяна Шурубейко" (F, 20 NOV 1987) and "Валентина Левшиц" (F, 1 APR 1987)
    let (_, p_tatiana) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Татьяна",
            "surname": "Шурубейко",
            "sex": "F",
            "birth_date": "20 NOV 1987",
            "is_living": false
        }),
    )
    .await;
    let tatiana_id = p_tatiana["id"].as_str().unwrap().to_string();

    let (_, p_valentina) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Валентина",
            "surname": "Левшиц",
            "sex": "F",
            "birth_date": "1 APR 1987",
            "is_living": false
        }),
    )
    .await;
    let valentina_id = p_valentina["id"].as_str().unwrap().to_string();

    // Create a shared relative and attach both to families sharing this relative
    let (_, p_rel) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Иван",
            "surname": "Левшиц",
            "sex": "M",
            "birth_date": "1985",
            "is_living": false
        }),
    )
    .await;
    let rel_id = p_rel["id"].as_str().unwrap().to_string();

    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": tatiana_id,
            "partner2_id": rel_id,
            "relationship_type": "married"
        }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/families",
        &admin_auth,
        json!({
            "partner1_id": valentina_id,
            "partner2_id": rel_id,
            "relationship_type": "married"
        }),
    )
    .await;

    // 2. Create actual duplicate candidates:
    // "Petr Shcheglov" (M, 1910) vs "Pyotr Fyodorovich Shcheglov" (M, 1910)
    let (_, p_petr) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Petr",
            "surname": "Shcheglov",
            "sex": "M",
            "birth_date": "1910",
            "is_living": false
        }),
    )
    .await;
    let petr_id = p_petr["id"].as_str().unwrap().to_string();

    let (_, p_pyotr) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "given_name": "Pyotr",
            "patronymic": "Fyodorovich",
            "surname": "Shcheglov",
            "sex": "M",
            "birth_date": "1910",
            "is_living": false
        }),
    )
    .await;
    let pyotr_id = p_pyotr["id"].as_str().unwrap().to_string();

    // Query duplicate candidates (default threshold is now 75)
    let (st_dups, dups_val) = get_json(&app, "/api/v1/quality/duplicates", Some(&admin_auth)).await;
    assert_eq!(st_dups, StatusCode::OK);
    let dups_resp: DuplicateCandidatesListResponse = serde_json::from_value(dups_val).unwrap();

    // Verify Tatiana and Valentina do NOT trigger as duplicate candidates
    let tatiana_valentina_candidate = dups_resp.items.iter().find(|d| {
        (d.person_a.id == tatiana_id && d.person_b.id == valentina_id)
            || (d.person_a.id == valentina_id && d.person_b.id == tatiana_id)
    });
    assert!(
        tatiana_valentina_candidate.is_none(),
        "Tatiana and Valentina must not be flagged as duplicates"
    );

    // Verify Petr Shcheglov and Pyotr Fyodorovich Shcheglov ARE flagged with score >= 80
    let petr_pyotr_candidate = dups_resp.items.iter().find(|d| {
        (d.person_a.id == petr_id && d.person_b.id == pyotr_id)
            || (d.person_a.id == pyotr_id && d.person_b.id == petr_id)
    });
    assert!(
        petr_pyotr_candidate.is_some(),
        "Petr and Pyotr Shcheglov must be flagged as duplicate candidates"
    );
    let pair = petr_pyotr_candidate.unwrap();
    assert!(
        pair.score >= 80,
        "Expected Petr vs Pyotr score >= 80, got {}",
        pair.score
    );
}
