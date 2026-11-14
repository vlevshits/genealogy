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

async fn setup_test_env_with_state() -> (AppState, axum::Router, String, String, String) {
    let temp_dir =
        std::env::temp_dir().join(format!("genealogy_story_mcp_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "AI Story Intake & MCP Test Tree".to_string(),
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
        store.story_drafts.clear();
        store.personal_access_tokens.clear();
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
    let app = create_router(state.clone());

    (state, app, admin_auth, editor_auth, viewer_auth)
}

async fn setup_test_env() -> (axum::Router, String, String, String) {
    let (_state, app, admin_auth, editor_auth, viewer_auth) = setup_test_env_with_state().await;
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
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
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
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

async fn delete_req(app: &axum::Router, uri: &str, auth: &str) -> StatusCode {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(uri)
                .header(header::AUTHORIZATION, auth)
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    res.status()
}

#[tokio::test]
async fn test_bilingual_story_parsing_refinement_and_sandbox_isolation() {
    let (app, _admin_auth, editor_auth, viewer_auth) = setup_test_env().await;

    // 1. Viewer is forbidden from parsing stories
    let (v_status, _) = post_json(
        &app,
        "/api/v1/ai/parse-story",
        &viewer_auth,
        json!({
            "raw_text": "I am Test Test, born in 1995 June 29, my bf is Test1 Test1"
        }),
    )
    .await;
    assert_eq!(v_status, StatusCode::FORBIDDEN);

    // 2. Editor parses an English family story
    let (en_status, en_draft) = post_json(
        &app,
        "/api/v1/ai/parse-story",
        &editor_auth,
        json!({
            "raw_text": "I am Test Test, born in 1995 June 29, my bf is Test1 Test1, born in 1993.",
            "is_new_speaker": true
        }),
    )
    .await;
    assert_eq!(en_status, StatusCode::CREATED, "en_draft: {en_draft}");
    let draft_id = en_draft["id"].as_str().unwrap().to_string();
    assert_eq!(en_draft["status"], "draft");

    let persons = en_draft["extracted_data"]["persons"].as_array().unwrap();
    assert_eq!(persons.len(), 2);
    assert_eq!(persons[0]["given_name"], "Test");
    assert_eq!(persons[0]["surname"], "Test");
    assert_eq!(persons[0]["relation_to_speaker"], "self");
    assert_eq!(persons[0]["birth_year"], 1995);
    assert_eq!(persons[0]["birth_date_text"], "29 JUN 1995");

    assert_eq!(persons[1]["given_name"], "Test1");
    assert_eq!(persons[1]["surname"], "Test1");
    assert_eq!(persons[1]["relation_to_speaker"], "partner");
    assert_eq!(persons[1]["sex"], "M");
    assert_eq!(persons[1]["birth_year"], 1993);

    let unions = en_draft["extracted_data"]["unions"].as_array().unwrap();
    assert_eq!(unions.len(), 1);

    // Verify sandbox isolation: live tree still has 0 individuals
    let (_, people_list) = get_json(&app, "/api/v1/people", &editor_auth).await;
    assert_eq!(people_list["items"].as_array().unwrap().len(), 0);

    // 3. Conversational refinement updates existing partner and adds a sister
    let (ref_status, refined) = post_json(
        &app,
        "/api/v1/ai/refine-draft",
        &editor_auth,
        json!({
            "draft_id": draft_id,
            "clarification_text": "My boyfriend's surname is Smith and he was born in 1993 in London. Also my sister Maria Test was born in 1998."
        }),
    )
    .await;
    assert_eq!(ref_status, StatusCode::OK, "refined: {refined}");
    let ref_persons = refined["extracted_data"]["persons"].as_array().unwrap();
    assert_eq!(ref_persons.len(), 3);
    let partner = ref_persons
        .iter()
        .find(|p| p["relation_to_speaker"] == "partner")
        .unwrap();
    assert_eq!(partner["surname"], "Smith");
    assert_eq!(partner["birth_place"], "London");
    let sister = ref_persons
        .iter()
        .find(|p| p["relation_to_speaker"] == "sister")
        .unwrap();
    assert_eq!(sister["given_name"], "Maria");
    assert_eq!(sister["birth_year"], 1998);

    // 4. Audio transcription endpoint works with Russian hint
    let (tr_status, tr_body) = post_json(
        &app,
        "/api/v1/ai/transcribe",
        &editor_auth,
        json!({
            "lang": "ru",
            "transcript_hint": "Я Иван Сергеевич Петров, родился 29 июня 1995 в Москве."
        }),
    )
    .await;
    assert_eq!(tr_status, StatusCode::OK);
    assert_eq!(tr_body["language"], "ru");
    assert!(tr_body["transcript"]
        .as_str()
        .unwrap()
        .contains("Иван Сергеевич Петров"));
}

#[tokio::test]
async fn test_russian_story_reconciliation_atomic_commit_and_one_click_undo() {
    let (app, admin_auth, editor_auth, _viewer_auth) = setup_test_env().await;

    // 1. Seed existing speaker (Иван Петров, born 1995, no patronymic yet) and his mother (Елена Петрова, born 1970)
    let (_, ivan_res) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "M",
            "given_name": "Иван",
            "surname": "Петров",
            "is_living": true
        }),
    )
    .await;
    let ivan_id = ivan_res["id"].as_str().unwrap().to_string();
    post_json(
        &app,
        "/api/v1/events",
        &editor_auth,
        json!({
            "individual_id": ivan_id,
            "event_type": "birth",
            "gedcom_date": "29 JUN 1995"
        }),
    )
    .await;

    let (_, elena_res) = post_json(
        &app,
        "/api/v1/persons",
        &editor_auth,
        json!({
            "sex": "F",
            "given_name": "Елена",
            "surname": "Петрова",
            "is_living": true
        }),
    )
    .await;
    let elena_id = elena_res["id"].as_str().unwrap().to_string();
    post_json(
        &app,
        "/api/v1/events",
        &editor_auth,
        json!({
            "individual_id": elena_id,
            "event_type": "birth",
            "gedcom_date": "12 MAY 1970"
        }),
    )
    .await;

    // Link Elena as Ivan's mother in an existing parent family
    let (fam_status, _) = post_json(
        &app,
        "/api/v1/families",
        &editor_auth,
        json!({
            "partner2_id": elena_id,
            "relationship_type": "married",
            "children_ids": [ivan_id]
        }),
    )
    .await;
    assert_eq!(fam_status, StatusCode::CREATED);

    // 2. Parse a rich Russian narrative anchored to Ivan (`speaker_id` alias for `speaker_person_id`)
    let ru_story = "Я Иван Сергеевич Петров, родился 29 июня 1995 в Москве. Моя девушка — Анна Смирнова, родилась в 1997 году. Моя мама — Елена Ивановна Петрова (в девичестве Соколова), родилась 12 мая 1970 в Твери. Мой папа — Сергей Петров, родился в 1968 году. У нас есть дочка София, родилась в 2022 году.";
    let (parse_status, draft) = post_json(
        &app,
        "/api/v1/ai/parse-story",
        &editor_auth,
        json!({
            "raw_text": ru_story,
            "speaker_id": ivan_id
        }),
    )
    .await;
    assert_eq!(parse_status, StatusCode::CREATED, "draft: {draft}");
    assert_eq!(draft["speaker_person_id"], ivan_id);
    let draft_id = draft["id"].as_str().unwrap().to_string();

    // 2b. Verify that passing both speaker_person_id AND speaker_id does not cause Serde duplicate field error
    let (both_keys_status, draft_both) = post_json(
        &app,
        "/api/v1/ai/parse-story",
        &editor_auth,
        json!({
            "raw_text": "Story with both speaker keys.",
            "speaker_id": ivan_id,
            "speaker_person_id": ivan_id
        }),
    )
    .await;
    assert_eq!(
        both_keys_status,
        StatusCode::CREATED,
        "draft_both: {draft_both}"
    );
    assert_eq!(draft_both["speaker_person_id"], ivan_id);

    // 3. Verify GET on /reconcile is disallowed (POST-only mutation route) and POST relationship-centric reconciliation matches speaker (100%) and mother (>= 90%)
    let (get_rec_status, _) = get_json(
        &app,
        &format!("/api/v1/ai/drafts/{draft_id}/reconcile"),
        &editor_auth,
    )
    .await;
    assert_eq!(get_rec_status, StatusCode::METHOD_NOT_ALLOWED);

    let (rec_status, reconciled) = post_json(
        &app,
        &format!("/api/v1/ai/drafts/{draft_id}/reconcile"),
        &editor_auth,
        json!({}),
    )
    .await;
    assert_eq!(rec_status, StatusCode::OK);
    let matches = reconciled["reconciliation_matches"].as_array().unwrap();
    let draft_persons = reconciled["extracted_data"]["persons"].as_array().unwrap();
    assert_eq!(draft_persons.len(), 5);

    let speaker_temp_id = draft_persons
        .iter()
        .find(|p| p["relation_to_speaker"] == "self")
        .unwrap()["temp_id"]
        .as_str()
        .unwrap();
    let mother_temp_id = draft_persons
        .iter()
        .find(|p| p["relation_to_speaker"] == "mother")
        .unwrap()["temp_id"]
        .as_str()
        .unwrap();
    let partner_temp_id = draft_persons
        .iter()
        .find(|p| p["relation_to_speaker"] == "partner")
        .unwrap()["temp_id"]
        .as_str()
        .unwrap();

    let speaker_match = matches
        .iter()
        .find(|m| m["draft_temp_id"] == speaker_temp_id)
        .unwrap();
    assert_eq!(speaker_match["selected_action"], "link_enrich");
    assert_eq!(speaker_match["matched_person_id"], ivan_id);
    assert_eq!(speaker_match["confidence"], 100);

    let mother_match = matches
        .iter()
        .find(|m| m["draft_temp_id"] == mother_temp_id)
        .unwrap();
    assert_eq!(mother_match["selected_action"], "link_enrich");
    assert_eq!(mother_match["matched_person_id"], elena_id);
    assert!(
        mother_match["confidence"].as_u64().unwrap() >= 90,
        "Expected mother confidence >= 90, got: {mother_match}"
    );

    let partner_match = matches
        .iter()
        .find(|m| m["draft_temp_id"] == partner_temp_id)
        .unwrap();
    assert_eq!(partner_match["selected_action"], "create_new");

    // 4. Atomically commit the draft
    let (commit_status, commit_res) = post_json(
        &app,
        &format!("/api/v1/ai/drafts/{draft_id}/commit"),
        &editor_auth,
        json!({}),
    )
    .await;
    assert_eq!(commit_status, StatusCode::OK, "commit_res: {commit_res}");
    let tx_id = commit_res["tx_id"].as_str().unwrap().to_string();
    assert!(!tx_id.is_empty());
    assert_eq!(
        commit_res["created_person_ids"].as_array().unwrap().len(),
        3,
        "Expected 3 net-new people (partner Anna, father Sergey, daughter Sofia)"
    );
    assert_eq!(
        commit_res["updated_person_ids"].as_array().unwrap().len(),
        2,
        "Expected 2 enriched people (Ivan and Elena)"
    );

    // Verify Ivan was enriched with patronymic Сергеевич and Elena with patronymic Ивановна + maiden name Соколова
    let (_, ivan_after) = get_json(&app, &format!("/api/v1/persons/{ivan_id}"), &editor_auth).await;
    assert_eq!(ivan_after["primary_name"]["patronymic"], "Сергеевич");

    let (_, elena_after) =
        get_json(&app, &format!("/api/v1/persons/{elena_id}"), &editor_auth).await;
    assert_eq!(elena_after["primary_name"]["patronymic"], "Ивановна");
    assert_eq!(elena_after["birth_surname"], "Соколова");

    // Verify total people count is now 5
    let (_, all_people) = get_json(&app, "/api/v1/people", &editor_auth).await;
    assert_eq!(all_people["items"].as_array().unwrap().len(), 5);

    // 5. Undo the entire story commit in 1 click using the shared tx_id
    let (undo_status, undo_res) = post_json(
        &app,
        &format!("/api/v1/history/tx/{tx_id}/undo"),
        &admin_auth,
        json!({}),
    )
    .await;
    assert_eq!(undo_status, StatusCode::OK, "undo_res: {undo_res}");

    // Verify all 3 net-new individuals were removed and Ivan/Elena were restored
    let (_, people_after_undo) = get_json(&app, "/api/v1/people", &editor_auth).await;
    assert_eq!(people_after_undo["items"].as_array().unwrap().len(), 2);

    let (_, ivan_restored) =
        get_json(&app, &format!("/api/v1/persons/{ivan_id}"), &editor_auth).await;
    assert!(ivan_restored["primary_name"]["patronymic"].is_null());

    // Verify the story draft status was reset back to "draft"
    let (_, draft_after_undo) =
        get_json(&app, &format!("/api/v1/ai/drafts/{draft_id}"), &editor_auth).await;
    assert_eq!(draft_after_undo["status"], "draft");
}

#[tokio::test]
async fn test_protected_mcp_server_pat_scopes_rbac_and_living_person_privacy() {
    let (app, admin_auth, _editor_auth, viewer_auth) = setup_test_env().await;

    // Seed one living person (John Doe, born 1995) and one deceased ancestor (Arthur Doe, born 1910, died 1985)
    let (_, living_res) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "sex": "M",
            "given_name": "John",
            "surname": "Doe",
            "is_living": true
        }),
    )
    .await;
    let living_id = living_res["id"].as_str().unwrap().to_string();
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": living_id,
            "event_type": "birth",
            "date_text": "15 MAR 1995",
            "place_path": "Boston"
        }),
    )
    .await;

    let (_, deceased_res) = post_json(
        &app,
        "/api/v1/persons",
        &admin_auth,
        json!({
            "sex": "M",
            "given_name": "Arthur",
            "surname": "Doe",
            "is_living": false
        }),
    )
    .await;
    let deceased_id = deceased_res["id"].as_str().unwrap().to_string();
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": deceased_id,
            "event_type": "birth",
            "date_text": "10 JAN 1910",
            "place_path": "London"
        }),
    )
    .await;
    post_json(
        &app,
        "/api/v1/events",
        &admin_auth,
        json!({
            "individual_id": deceased_id,
            "event_type": "death",
            "date_text": "20 NOV 1985",
            "place_path": "London"
        }),
    )
    .await;

    // 1. Unauthenticated request to /api/v1/mcp is rejected with 401
    let unauth_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/mcp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth_res.status(), StatusCode::UNAUTHORIZED);

    // 2. Viewer cannot mint a PAT with write:tree scope (403 Forbidden)
    let (bad_pat_status, _) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &viewer_auth,
        json!({
            "name": "Escalated Token",
            "scopes": ["read:tree", "write:tree"]
        }),
    )
    .await;
    assert_eq!(bad_pat_status, StatusCode::FORBIDDEN);

    // 3. Viewer mints a read-only PAT (read:tree)
    let (v_pat_status, v_pat_body) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &viewer_auth,
        json!({
            "name": "Viewer Claude Desktop",
            "scopes": ["read:tree"],
            "expires_in_days": 30
        }),
    )
    .await;
    assert_eq!(v_pat_status, StatusCode::CREATED);
    let viewer_raw_pat = v_pat_body["token"].as_str().unwrap().to_string();
    let viewer_pat_id = v_pat_body["token_info"]["id"].as_str().unwrap().to_string();
    assert!(viewer_raw_pat.starts_with("mcp_pat_"));

    // Verify GET /api/v1/mcp/tokens lists the token metadata without exposing the raw token
    let (list_status, list_body) = get_json(&app, "/api/v1/mcp/tokens", &viewer_auth).await;
    assert_eq!(list_status, StatusCode::OK);
    let listed = list_body["items"].as_array().unwrap();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].get("token").is_none());
    assert!(listed[0].get("token_hash").is_none());

    // 4. Viewer PAT calls tools/list -> only Tier A tools are returned
    let (v_tools_status, v_tools_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {viewer_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }),
    )
    .await;
    assert_eq!(v_tools_status, StatusCode::OK);
    let v_tools = v_tools_res["result"]["tools"].as_array().unwrap();
    assert!(
        v_tools.iter().all(|t| t["tier"] == "A"),
        "Viewer PAT must only list Tier A tools"
    );

    // 5. Viewer PAT calls get_individual on living person -> given_name is masked to initial "J." and birth date is year-only
    let (v_get_status, v_get_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {viewer_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_individual",
                "arguments": { "person_id": living_id }
            }
        }),
    )
    .await;
    assert_eq!(v_get_status, StatusCode::OK);
    let v_person = &v_get_res["result"]["structuredContent"];
    assert_eq!(
        v_person["primary_name"]["given_name"], "J.",
        "Viewer MCP call must mask living given name to initial"
    );
    assert_eq!(v_person["birth"]["date"], "1995");
    assert!(v_person["birth"]["place"].is_null());

    // 5b. Viewer PAT calls list_events_timeline -> living person's events are redacted/excluded while deceased events are returned
    let (v_ev_status, v_ev_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {viewer_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 31,
            "method": "tools/call",
            "params": {
                "name": "list_events_timeline",
                "arguments": {}
            }
        }),
    )
    .await;
    assert_eq!(v_ev_status, StatusCode::OK);
    let v_events = v_ev_res["result"]["structuredContent"]["items"]
        .as_array()
        .unwrap();
    assert_eq!(
        v_events.len(),
        2,
        "Viewer MCP list_events_timeline must exclude living person events and return only deceased events"
    );
    assert!(v_events
        .iter()
        .all(|e| e["individual_id"].as_str() == Some(deceased_id.as_str())));

    // Viewer PAT calling list_events_timeline specifically for living_id returns 0 events
    let (_, v_living_ev_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {viewer_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 32,
            "method": "tools/call",
            "params": {
                "name": "list_events_timeline",
                "arguments": { "person_id": living_id }
            }
        }),
    )
    .await;
    assert_eq!(
        v_living_ev_res["result"]["structuredContent"]["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    // 5c. Viewer PAT calls get_ancestors_descendants -> living individuals in persons list are redacted
    let (_, v_sub_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {viewer_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 33,
            "method": "tools/call",
            "params": {
                "name": "get_ancestors_descendants",
                "arguments": { "id": living_id }
            }
        }),
    )
    .await;
    let sub_persons = v_sub_res["result"]["structuredContent"]["persons"]
        .as_array()
        .unwrap();
    let sub_living = sub_persons.iter().find(|p| p["id"] == living_id).unwrap();
    assert_eq!(sub_living["primary_name"]["given_name"], "J.");

    // 6. Viewer PAT attempting Tier B (create_individual) or Tier C (merge_duplicate_individuals) is rejected with 403 / -32600
    let (v_mut_status, v_mut_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {viewer_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "create_individual",
                "arguments": { "given_name": "Unauthorized", "surname": "Person" }
            }
        }),
    )
    .await;
    assert_eq!(v_mut_status, StatusCode::FORBIDDEN);
    assert_eq!(v_mut_res["error"]["code"], -32600);

    // 7. Admin mints a full-access PAT and invokes Tier A, B, and C tools
    let (a_pat_status, a_pat_body) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &admin_auth,
        json!({
            "name": "Admin MCP Agent",
            "scopes": ["read:tree", "write:tree", "write:ai_drafts", "admin:maintenance"]
        }),
    )
    .await;
    assert_eq!(a_pat_status, StatusCode::CREATED);
    let admin_raw_pat = a_pat_body["token"].as_str().unwrap().to_string();

    // Admin PAT sees full unredacted living person
    let (a_get_status, a_get_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "get_individual",
                "arguments": { "person_id": living_id }
            }
        }),
    )
    .await;
    assert_eq!(a_get_status, StatusCode::OK);
    assert_eq!(
        a_get_res["result"]["structuredContent"]["primary_name"]["given_name"],
        "John"
    );

    // Admin PAT sees living person's events in list_events_timeline
    let (_, a_ev_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 51,
            "method": "tools/call",
            "params": {
                "name": "list_events_timeline",
                "arguments": { "person_id": living_id }
            }
        }),
    )
    .await;
    let a_living_events = a_ev_res["result"]["structuredContent"]["items"]
        .as_array()
        .unwrap();
    assert_eq!(a_living_events.len(), 1);
    assert_eq!(a_living_events[0]["place_full_name"], "Boston");

    // Admin PAT creates an individual via MCP tool create_individual
    let (a_create_status, a_create_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "create_individual",
                "arguments": {
                    "given_name": "Clara",
                    "surname": "Doe",
                    "sex": "female",
                    "birth_date": "1998"
                }
            }
        }),
    )
    .await;
    assert_eq!(a_create_status, StatusCode::OK);
    let clara_id = a_create_res["result"]["structuredContent"]["person"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        a_create_res["result"]["structuredContent"]["person"]["primary_name"]["given_name"],
        "Clara"
    );
    assert_eq!(
        a_create_res["result"]["structuredContent"]["person"]["sex"],
        "F"
    );
    assert_eq!(
        a_create_res["result"]["structuredContent"]["person"]["is_living"],
        true
    );

    // Admin PAT creates a BIRT and DEATH event via MCP tool create_event with uppercase/GEDCOM tags
    let (a_birt_status, a_birt_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 61,
            "method": "tools/call",
            "params": {
                "name": "create_event",
                "arguments": {
                    "person_id": clara_id,
                    "event_type": "BIRT",
                    "date": "14 FEB 1925",
                    "place_path": "Brista, Chausy, Mogilev, Belarus"
                }
            }
        }),
    )
    .await;
    assert_eq!(a_birt_status, StatusCode::OK);
    assert_eq!(
        a_birt_res["result"]["structuredContent"]["event_type"],
        "birth"
    );

    let (a_deat_status, a_deat_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 62,
            "method": "tools/call",
            "params": {
                "name": "create_event",
                "arguments": {
                    "person_id": clara_id,
                    "event_type": "DEATH",
                    "date_text": "18 OCT 2013",
                    "place": "Minsk, Belarus"
                }
            }
        }),
    )
    .await;
    assert_eq!(a_deat_status, StatusCode::OK);
    assert_eq!(
        a_deat_res["result"]["structuredContent"]["event_type"],
        "death"
    );

    // Verify Clara's birth, death, and is_living were recomputed from the normalized events
    let (_, clara_after) =
        get_json(&app, &format!("/api/v1/persons/{clara_id}"), &admin_auth).await;
    assert_eq!(clara_after["birth"]["date"], "14 FEB 1925");
    assert_eq!(clara_after["death"]["date"], "18 OCT 2013");
    assert_eq!(clara_after["is_living"], false);
    assert_eq!(clara_after["has_death_event"], true);

    // Verify MCP create_or_resolve_place normalizes uppercase place_type ("VILLAGE" -> "village")
    let (a_place_status, a_place_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 63,
            "method": "tools/call",
            "params": {
                "name": "create_or_resolve_place",
                "arguments": {
                    "name": "Samotevichi",
                    "place_type": "VILLAGE"
                }
            }
        }),
    )
    .await;
    assert_eq!(a_place_status, StatusCode::OK);
    assert_eq!(
        a_place_res["result"]["structuredContent"]["place_type"],
        "village"
    );

    // 8. Revoke Viewer PAT and verify subsequent MCP call returns 401 Unauthorized
    let rev_status = delete_req(
        &app,
        &format!("/api/v1/mcp/tokens/{viewer_pat_id}"),
        &viewer_auth,
    )
    .await;
    assert_eq!(rev_status, StatusCode::NO_CONTENT);

    let (after_rev_status, _) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {viewer_raw_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/list"
        }),
    )
    .await;
    assert_eq!(after_rev_status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_parse_story_with_provided_extracted_data_and_derived_title() {
    let (app, _admin_auth, editor_auth, _viewer_auth) = setup_test_env().await;

    // 1. Post a story with pre-extracted data and no title -> title should be derived from extracted persons
    let custom_extracted = json!({
        "persons": [
            {
                "temp_id": "temp_p1",
                "given_name": "Arthur",
                "surname": "Pendelton",
                "sex": "M",
                "birth_year": 1920,
                "birth_place": "Bristol",
                "relation_to_speaker": "grandfather",
                "confidence": 95,
                "biography_notes": "Served in WWII"
            },
            {
                "temp_id": "temp_p2",
                "given_name": "Beatrice",
                "surname": "Pendelton",
                "sex": "F",
                "birth_year": 1924,
                "birth_place": "Bath",
                "relation_to_speaker": "grandmother",
                "confidence": 90,
                "biography_notes": "Met Arthur in 1945"
            }
        ],
        "unions": [
            {
                "temp_id": "temp_u1",
                "partner1_temp_id": "temp_p1",
                "partner2_temp_id": "temp_p2",
                "union_type": "MARR",
                "marriage_date_text": "1946",
                "marriage_place": "Bristol",
                "confidence": 95
            }
        ],
        "parent_child_links": [],
        "events": []
    });

    let (status, draft) = post_json(
        &app,
        "/api/v1/ai/parse-story",
        &editor_auth,
        json!({
            "raw_text": "Arthur and Beatrice met right after the war.",
            "extracted_data": custom_extracted
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "draft response: {draft}");
    assert_eq!(draft["status"], "draft");

    // Title should be derived via derive_story_title(&raw_text, &extracted_data)
    assert_eq!(draft["title"], "Family Story: Arthur Pendelton");

    // Extracted data matches what was passed in directly
    let persons = draft["extracted_data"]["persons"].as_array().unwrap();
    assert_eq!(persons.len(), 2);
    assert_eq!(persons[0]["temp_id"], "temp_p1");
    assert_eq!(persons[0]["given_name"], "Arthur");
    assert_eq!(persons[0]["surname"], "Pendelton");
    assert_eq!(persons[0]["biography_notes"], "Served in WWII");

    assert_eq!(persons[1]["temp_id"], "temp_p2");
    assert_eq!(persons[1]["given_name"], "Beatrice");

    let unions = draft["extracted_data"]["unions"].as_array().unwrap();
    assert_eq!(unions.len(), 1);
    assert_eq!(unions[0]["temp_id"], "temp_u1");

    // Reconciliation matches should be run seamlessly on the provided extracted_data
    let reconciliations = draft["reconciliation_matches"].as_array().unwrap();
    assert_eq!(reconciliations.len(), 2);
    assert_eq!(reconciliations[0]["draft_temp_id"], "temp_p1");
    assert_eq!(reconciliations[0]["recommended_action"], "create_new");
    assert_eq!(reconciliations[1]["draft_temp_id"], "temp_p2");
    assert_eq!(reconciliations[1]["recommended_action"], "create_new");

    // 2. Post a story with provided title alongside extracted_data -> provided title is preserved
    let (status2, draft2) = post_json(
        &app,
        "/api/v1/ai/parse-story",
        &editor_auth,
        json!({
            "title": "Memoirs of Bristol 1945",
            "raw_text": "Arthur and Beatrice met right after the war.",
            "extracted_data": {
                "persons": [
                    {
                        "temp_id": "temp_p1",
                        "given_name": "Arthur",
                        "surname": "Pendelton",
                        "sex": "M",
                        "relation_to_speaker": "relative",
                        "confidence": 90
                    }
                ],
                "unions": [],
                "parent_child_links": [],
                "events": []
            }
        }),
    )
    .await;

    assert_eq!(status2, StatusCode::CREATED);
    assert_eq!(draft2["title"], "Memoirs of Bristol 1945");
}

#[tokio::test]
async fn test_gemini_config_system_info_and_missing_api_key_audio_transcription_error() {
    let (app, _admin_auth, editor_auth, _viewer_auth) = setup_test_env().await;

    // 1. GET /api/v1/system/info should show ai_enabled: false when no key is set
    let (info_status, info_body) = get_json(&app, "/api/v1/system/info", "").await;
    assert_eq!(info_status, StatusCode::OK);
    assert_eq!(info_body["ai_enabled"], false);

    // 2. Setting GEMINI_API_KEY in state reflects ai_enabled: true
    let custom_cfg = AppConfig {
        app_env: "test".to_string(),
        gemini_api_key: Some("test-gemini-key-12345".to_string()),
        gemini_model: "gemini-3.8-flash".to_string(),
        ..AppConfig::default()
    };
    let app2_state = AppState::new(None, custom_cfg);
    let app2 = create_router(app2_state);
    let (info_status2, info_body2) = get_json(&app2, "/api/v1/system/info", "").await;
    assert_eq!(info_status2, StatusCode::OK);
    assert_eq!(info_body2["ai_enabled"], true);

    // 3. Audio transcription without GEMINI_API_KEY returns clear validation error
    let raw_fake_audio_bytes = vec![0x1a, 0x45, 0xdf, 0xa3, 0x00, 0x01, 0x02];
    let base64_audio = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &raw_fake_audio_bytes,
    );

    let (tr_status, tr_err) = post_json(
        &app,
        "/api/v1/ai/transcribe",
        &editor_auth,
        json!({
            "audio_base64": base64_audio,
            "filename": "recording.webm",
            "lang": "en"
        }),
    )
    .await;
    assert_eq!(tr_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        tr_err["error"]["field_errors"]["audio"],
        "gemini_api_key_not_configured"
    );

    // 4. Story parsing with engine="builtin" extracts genealogical graph using built-in NLP
    let (p_status, p_draft) = post_json(
        &app,
        "/api/v1/ai/parse-story",
        &editor_auth,
        json!({
            "raw_text": "I am William Wallace born in 1270 in Elderslie.",
            "engine": "builtin"
        }),
    )
    .await;
    assert_eq!(p_status, StatusCode::CREATED);
    assert_eq!(p_draft["status"], "draft");
    let persons = p_draft["extracted_data"]["persons"].as_array().unwrap();
    assert!(!persons.is_empty());
    assert_eq!(persons[0]["given_name"], "William");
}

#[tokio::test]
async fn test_mcp_merge_duplicate_individuals_executes_real_merge_with_aliases_and_rejects_empty() {
    let (_state, app, admin_auth, editor_auth, viewer_auth) = setup_test_env_with_state().await;

    // Role-aware empty-scopes defaults when `scopes: []` or omitted
    let (adm_st, adm_pat) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &admin_auth,
        json!({ "name": "Admin Default Scopes", "scopes": [] }),
    )
    .await;
    assert_eq!(adm_st, StatusCode::CREATED);
    assert_eq!(
        adm_pat["token_info"]["scopes"],
        json!([
            "read:tree",
            "write:draft",
            "write:direct",
            "admin:maintenance"
        ])
    );
    let admin_pat = adm_pat["token"].as_str().unwrap().to_string();

    let (ed_st, ed_pat) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &editor_auth,
        json!({ "name": "Editor Default Scopes", "scopes": [] }),
    )
    .await;
    assert_eq!(ed_st, StatusCode::CREATED);
    assert_eq!(
        ed_pat["token_info"]["scopes"],
        json!(["read:tree", "write:draft", "write:direct"])
    );

    let (vw_st, vw_pat) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &viewer_auth,
        json!({ "name": "Viewer Default Scopes", "scopes": [] }),
    )
    .await;
    assert_eq!(vw_st, StatusCode::CREATED);
    assert_eq!(vw_pat["token_info"]["scopes"], json!(["read:tree"]));

    // 1. Empty arguments to MCP `merge_duplicate_individuals` MUST return validation error (-32602),
    // never the REST smoke-test no-op `"Duplicate person merge authorized for administrator"`.
    let (empty_st, empty_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 100,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_individuals",
                "arguments": {}
            }
        }),
    )
    .await;
    assert_eq!(empty_st, StatusCode::OK);
    assert_eq!(empty_res["error"]["code"], -32602);
    assert!(
        empty_res.get("result").is_none(),
        "Empty MCP merge must fail validation rather than returning a fake ok result"
    );

    // Missing one ID also fails validation (-32602)
    let (partial_st, partial_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 101,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_individuals",
                "arguments": { "surviving_id": "I0100" }
            }
        }),
    )
    .await;
    assert_eq!(partial_st, StatusCode::OK);
    assert_eq!(partial_res["error"]["code"], -32602);

    // Helper closure to create two duplicate test individuals via REST API
    let create_person = |given: &str, surname: &str, birth_date: Option<&str>| {
        let app_c = app.clone();
        let auth_c = admin_auth.clone();
        let given_s = given.to_string();
        let surname_s = surname.to_string();
        let birth_s = birth_date.map(ToString::to_string);
        async move {
            let mut body = json!({
                "given_name": given_s,
                "surname": surname_s,
                "sex": "M",
                "is_living": false
            });
            if let Some(bd) = birth_s {
                body["birth_date"] = json!(bd);
            }
            let (st, created) = post_json(&app_c, "/api/v1/persons", &auth_c, body).await;
            assert_eq!(st, StatusCode::CREATED);
            created["id"].as_str().unwrap().to_string()
        }
    };

    // Case A: Standard `surviving_id` + `duplicate_id` with `field_choices` & `field_overrides`
    let p1 = create_person("Ivan", "Petrov", Some("1900")).await;
    let p2 = create_person("Ivan", "Petroff", Some("12 MAY 1900")).await;
    let (m1_st, m1_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 102,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_individuals",
                "arguments": {
                    "surviving_id": p1,
                    "duplicate_id": p2,
                    "field_choices": { "surname": "merge", "birth_date": "merge" },
                    "field_overrides": { "surname": "Petroff" }
                }
            }
        }),
    )
    .await;
    assert_eq!(m1_st, StatusCode::OK);
    let m1_out = &m1_res["result"]["structuredContent"];
    assert_eq!(m1_out["status"], "merged");
    assert_eq!(m1_out["surviving_person_id"], p1);
    assert_eq!(m1_out["merged_id"], p2);
    assert_eq!(m1_out["merged_person_id"], p2);
    assert!(m1_out["tx_id"].as_str().is_some_and(|s| !s.is_empty()));

    let (gone_st, _) = get_json(&app, &format!("/api/v1/persons/{p2}"), &admin_auth).await;
    assert_eq!(gone_st, StatusCode::NOT_FOUND);
    let (kept_st, kept_body) = get_json(&app, &format!("/api/v1/persons/{p1}"), &admin_auth).await;
    assert_eq!(kept_st, StatusCode::OK);
    assert_eq!(kept_body["primary_name"]["surname"], "Petroff");
    assert_eq!(kept_body["birth"]["date"], "12 MAY 1900");

    // Case B: `keep_id` + `merge_id` aliases
    let p3 = create_person("Boris", "Sidorov", None).await;
    let p4 = create_person("Boris", "Sidorov", Some("1910")).await;
    let (m2_st, m2_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 103,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_individuals",
                "arguments": { "keep_id": p3, "merge_id": p4 }
            }
        }),
    )
    .await;
    assert_eq!(m2_st, StatusCode::OK);
    assert_eq!(m2_res["result"]["structuredContent"]["status"], "merged");
    assert_eq!(m2_res["result"]["structuredContent"]["merged_id"], p4);

    // Case C: `args` stringified JSON + `target_id` / `source_id` aliases
    let p5 = create_person("Sergey", "Volkov", None).await;
    let p6 = create_person("Sergey", "Volkov", None).await;
    let stringified = json!({ "target_id": p5, "source_id": p6 }).to_string();
    let (m3_st, m3_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 104,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_individuals",
                "args": stringified
            }
        }),
    )
    .await;
    assert_eq!(m3_st, StatusCode::OK);
    assert_eq!(m3_res["result"]["structuredContent"]["status"], "merged");
    assert_eq!(
        m3_res["result"]["structuredContent"]["merged_person_id"],
        p6
    );

    // Case D: Top-level tool params (`arguments`/`args` omitted) + `person_a_id` / `person_b_id`
    let p7 = create_person("Nikolai", "Morozov", None).await;
    let p8 = create_person("Nikolai", "Morozov", None).await;
    let (m4_st, m4_res) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {admin_pat}"),
        json!({
            "jsonrpc": "2.0",
            "id": 105,
            "method": "tools/call",
            "params": {
                "name": "merge_duplicate_individuals",
                "person_a_id": p7,
                "person_b_id": p8
            }
        }),
    )
    .await;
    assert_eq!(m4_st, StatusCode::OK);
    assert_eq!(m4_res["result"]["structuredContent"]["status"], "merged");
    assert_eq!(
        m4_res["result"]["structuredContent"]["surviving_person_id"],
        p7
    );
}

#[tokio::test]
async fn test_mcp_token_expiration_and_persistence_across_resets() {
    let (state, app, admin_auth, _editor_auth, _viewer_auth) = setup_test_env_with_state().await;

    // 1. Mint never-expiring token (`expires_in_days: None`) and future offset token
    let (st_never, body_never) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &admin_auth,
        json!({ "name": "Never Expire", "expires_in_days": null }),
    )
    .await;
    assert_eq!(st_never, StatusCode::CREATED);
    assert!(body_never["token_info"].get("expires_at").is_none());
    let never_pat = body_never["token"].as_str().unwrap().to_string();

    let (st_exp, body_exp) = post_json(
        &app,
        "/api/v1/mcp/tokens",
        &admin_auth,
        json!({ "name": "Expired Soon", "expires_in_days": 7 }),
    )
    .await;
    assert_eq!(st_exp, StatusCode::CREATED);
    let exp_pat_id = body_exp["token_info"]["id"].as_str().unwrap().to_string();
    let exp_pat = body_exp["token"].as_str().unwrap().to_string();

    // Force `exp_pat` to a past RFC-3339 timestamp with timezone offset (+03:00 vs Z check)
    {
        let mut store = state.privacy_store.write().await;
        let rec = store
            .personal_access_tokens
            .iter_mut()
            .find(|t| t.id == exp_pat_id)
            .unwrap();
        rec.expires_at = Some("2020-01-01T00:00:00+03:00".to_string());
    }

    let (exp_call_st, _) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {exp_pat}"),
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
    )
    .await;
    assert_eq!(exp_call_st, StatusCode::UNAUTHORIZED);

    let (ok_call_st, _) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {never_pat}"),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
    )
    .await;
    assert_eq!(ok_call_st, StatusCode::OK);

    // 2. Preserve PATs across GEDCOM `replace` import
    let gedcom = "0 HEAD\n1 SOUR TEST\n1 GEDC\n2 VERS 5.5.1\n1 CHAR UTF-8\n0 @I1@ INDI\n1 NAME Anton /Kuznetsov/\n1 SEX M\n0 TRLR\n";
    let (imp_st, _) = post_json(
        &app,
        "/api/v1/gedcom/import",
        &admin_auth,
        json!({
            "data": gedcom,
            "mode": "replace",
            "confirm_text": "REPLACE"
        }),
    )
    .await;
    assert_eq!(imp_st, StatusCode::OK);

    let (after_imp_st, _) = post_json(
        &app,
        "/api/v1/mcp",
        &format!("Bearer {never_pat}"),
        json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/list" }),
    )
    .await;
    assert_eq!(after_imp_st, StatusCode::OK);

    // 3. Production backup persistence & simulated restart recovery (`mcp-tokens.json` + `live-store.json`)
    let restart_temp =
        std::env::temp_dir().join(format!("genealogy_mcp_restart_{}", uuid::Uuid::new_v4()));
    let prod_cfg = AppConfig {
        app_env: "production".to_string(),
        backup_path: restart_temp.to_string_lossy().to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    };
    let prod_state = AppState::new(None, prod_cfg.clone());
    let (setup_res, _) = AuthService::setup_initial_admin(
        &prod_state,
        SetupRequest {
            tree_title: None,
            first_name: "Prod".to_string(),
            last_name: "Admin".to_string(),
            email: "prodadmin@example.com".to_string(),
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
    let prod_admin_auth = format!("Bearer {}", setup_res.session_token);
    let prod_app = create_router(prod_state.clone());

    let (mint_st, mint_body) = post_json(
        &prod_app,
        "/api/v1/mcp/tokens",
        &prod_admin_auth,
        json!({ "name": "Survives Restart" }),
    )
    .await;
    assert_eq!(mint_st, StatusCode::CREATED);
    let persistent_pat = mint_body["token"].as_str().unwrap().to_string();
    assert!(restart_temp.join("mcp-tokens.json").exists());
    assert!(restart_temp.join("live-store.json").exists());

    // Simulate brand-new AppState loading tokens back from disk backup
    let restarted_state = AppState::new(None, prod_cfg);
    {
        let users = prod_state.auth_store.read().await.users.clone();
        let mut auth_store = restarted_state.auth_store.write().await;
        auth_store.users = users;
    }
    {
        let file_tokens = genealogy_backend::services::mcp::load_mcp_tokens_from_dir(&restart_temp);
        let mut pstore = restarted_state.privacy_store.write().await;
        genealogy_backend::services::mcp::merge_mcp_tokens_into_store(&mut pstore, file_tokens);
    }
    let restarted_app = create_router(restarted_state);
    let (restarted_st, _) = post_json(
        &restarted_app,
        "/api/v1/mcp",
        &format!("Bearer {persistent_pat}"),
        json!({ "jsonrpc": "2.0", "id": 4, "method": "tools/list" }),
    )
    .await;
    assert_eq!(restarted_st, StatusCode::OK);
    let _ = std::fs::remove_dir_all(&restart_temp);

    // 4. Demo-seed initial setup reset preserves previously added PATs
    let demo_state = AppState::new_demo(
        None,
        AppConfig {
            app_env: "test".to_string(),
            enable_public_access: false,
            ..AppConfig::default()
        },
    );
    let demo_admin_id = uuid::Uuid::new_v4();
    let demo_raw = "mcp_pat_preserved_demo_secret_123456";
    let demo_hash = genealogy_backend::services::mcp::hash_mcp_token(demo_raw);
    let seed_raw = "mcp_pat_seed_agent_secret_abcdef";
    let seed_hash = genealogy_backend::services::mcp::hash_mcp_token(seed_raw);
    {
        let mut pstore = demo_state.privacy_store.write().await;
        pstore.personal_access_tokens.push(
            genealogy_backend::services::privacy::PersonalAccessTokenDto {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: demo_admin_id.to_string(),
                name: "Pre-Setup Regular".to_string(),
                token_prefix: "mcp_pat_preserv".to_string(),
                token_hash: demo_hash,
                scopes: vec!["read:tree".to_string()],
                last_used_at: None,
                expires_at: None,
                revoked_at: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        );
        pstore.personal_access_tokens.push(
            genealogy_backend::services::privacy::PersonalAccessTokenDto {
                id: genealogy_backend::services::mcp::deterministic_seed_pat_id(&seed_hash),
                user_id: "seed-admin".to_string(),
                name: "Production MCP Agent".to_string(),
                token_prefix: "mcp_pat_seed_age".to_string(),
                token_hash: seed_hash,
                scopes: vec!["read:tree".to_string(), "admin:maintenance".to_string()],
                last_used_at: None,
                expires_at: None,
                revoked_at: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        );
    }
    let demo_app = create_router(demo_state.clone());
    let (setup_st, _) = post_json(
        &demo_app,
        "/api/v1/auth/setup",
        "",
        json!({
            "first_name": "Fresh",
            "last_name": "Owner",
            "email": "fresh@example.com",
            "password": "FreshOwnerPass123!"
        }),
    )
    .await;
    assert_eq!(setup_st, StatusCode::CREATED);
    assert!(
        demo_state
            .privacy_store
            .read()
            .await
            .personal_access_tokens
            .iter()
            .any(|t| t.token_prefix == "mcp_pat_preserv"),
        "Initial setup clearing demo seed must preserve personal_access_tokens"
    );

    // Non-seed token belonging to an absent user MUST NOT escalate to active admin
    let (reg_missing_st, _) = post_json(
        &demo_app,
        "/api/v1/mcp",
        &format!("Bearer {demo_raw}"),
        json!({ "jsonrpc": "2.0", "id": 5, "method": "tools/list" }),
    )
    .await;
    assert_eq!(reg_missing_st, StatusCode::UNAUTHORIZED);

    // Seeded admin agent token authenticates cleanly after initial setup
    let (seed_ok_st, _) = post_json(
        &demo_app,
        "/api/v1/mcp",
        &format!("Bearer {seed_raw}"),
        json!({ "jsonrpc": "2.0", "id": 6, "method": "tools/list" }),
    )
    .await;
    assert_eq!(seed_ok_st, StatusCode::OK);

    // Deactivated user's token MUST fail with 401 even if named "Production MCP Agent"
    let owner_id = demo_state.auth_store.read().await.users[0].id;
    {
        let mut astore = demo_state.auth_store.write().await;
        astore.users[0].is_active = false;
        let mut pstore = demo_state.privacy_store.write().await;
        pstore.personal_access_tokens[1].user_id = owner_id.to_string();
    }
    let (deactivated_st, _) = post_json(
        &demo_app,
        "/api/v1/mcp",
        &format!("Bearer {seed_raw}"),
        json!({ "jsonrpc": "2.0", "id": 7, "method": "tools/list" }),
    )
    .await;
    assert_eq!(deactivated_st, StatusCode::UNAUTHORIZED);
}
