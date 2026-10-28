use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use genealogy_backend::{
    config::AppConfig,
    routes::create_router,
    services::{
        auth::{AuthService, CreateInviteRequest, InviteRegisterRequest, SetupRequest},
        backup::CURRENT_SCHEMA_VERSION,
    },
    state::AppState,
};

fn test_config() -> AppConfig {
    AppConfig {
        app_env: "test".to_string(),
        tree_title: "Onboarding Test Tree".to_string(),
        default_locale: "en".to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    }
}

async fn setup_empty_state_with_roles() -> (AppState, String, String, String) {
    let state = AppState::new_empty(None, test_config());

    let (_, admin_cookie) = AuthService::setup_initial_admin(
        &state,
        SetupRequest {
            tree_title: Some("Romanov Family Tree".to_string()),
            first_name: "Anna".to_string(),
            last_name: "Admin".to_string(),
            email: "admin@example.com".to_string(),
            passkey: None,
            password: Some("AdminPass123!".to_string()),
            emergency_token: None,
        },
        None,
        Some("127.0.0.1".to_string()),
    )
    .await
    .unwrap();
    let admin_id = state.auth_store.read().await.users[0].id;

    // Create Editor
    let editor_inv = AuthService::create_invitation(
        &state,
        Some(admin_id),
        CreateInviteRequest {
            role: "editor".to_string(),
            email: Some("editor@example.com".to_string()),
            invitee_name: Some("Elena Editor".to_string()),
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (_, editor_cookie) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: editor_inv.token,
            email: Some("editor@example.com".to_string()),
            first_name: "Elena".to_string(),
            last_name: "Editor".to_string(),
            passkey: None,
            password: Some("EditorPass123!".to_string()),
        },
        None,
        Some("127.0.0.1".to_string()),
    )
    .await
    .unwrap();

    // Create Viewer
    let viewer_inv = AuthService::create_invitation(
        &state,
        Some(admin_id),
        CreateInviteRequest {
            role: "viewer".to_string(),
            email: Some("viewer@example.com".to_string()),
            invitee_name: Some("Victor Viewer".to_string()),
            person_id: None,
        },
    )
    .await
    .unwrap();
    let (_, viewer_cookie) = AuthService::register_with_invitation(
        &state,
        InviteRegisterRequest {
            invite_token: viewer_inv.token,
            email: Some("viewer@example.com".to_string()),
            first_name: "Victor".to_string(),
            last_name: "Viewer".to_string(),
            passkey: None,
            password: Some("ViewerPass123!".to_string()),
        },
        None,
        Some("127.0.0.1".to_string()),
    )
    .await
    .unwrap();

    (state, admin_cookie, editor_cookie, viewer_cookie)
}

async fn read_json(resp: axum::response::Response) -> Value {
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

#[tokio::test]
async fn test_onboarding_status_gatekeeper_and_role_permissions() {
    let (state, admin_cookie, editor_cookie, viewer_cookie) = setup_empty_state_with_roles().await;
    let app = create_router(state.clone());

    // 1. Unauthenticated request when enable_public_access == false returns 401
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 2. Admin on empty tree sees is_empty: true, total_individuals: 0, can_onboard: true
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["is_empty"], true);
    assert_eq!(body["total_individuals"], 0);
    assert_eq!(body["can_onboard"], true);
    assert!(body["resolved_home_person_id"].is_null());
    assert!(body["default_home_person_id"].is_null());
    assert_eq!(body["start_page"], "dashboard");

    // 3. Editor on empty tree sees can_onboard: true
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &editor_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["is_empty"], true);
    assert_eq!(body["can_onboard"], true);

    // 4. Viewer on empty tree sees is_empty: true, can_onboard: false
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &viewer_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json(resp).await;
    assert_eq!(body["is_empty"], true);
    assert_eq!(body["can_onboard"], false);

    // 5. Viewer is forbidden (403) from calling POST /api/v1/onboarding/start-with-self
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &viewer_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Victor",
                        "surname": "Viewer",
                        "sex": "M"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_start_with_self_atomic_creation_and_conflict() {
    let (state, admin_cookie, editor_cookie, _) = setup_empty_state_with_roles().await;
    let app = create_router(state.clone());

    // 1. Validation error (400) if both given_name and surname are blank
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "   ",
                        "surname": ""
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // 2. Happy path: Admin creates root self person with birth date and place chain
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Anna",
                        "patronymic": "Nikolaevna",
                        "surname": "Romanova",
                        "sex": "F",
                        "birth_date": "15 MAY 1990",
                        "birth_place": "Saint Petersburg, Russia"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = read_json(resp).await;

    let person_id = body["individual"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["individual"]["gid"], "I0100");
    assert_eq!(body["individual"]["primary_name"]["given_name"], "Anna");
    assert_eq!(
        body["individual"]["primary_name"]["patronymic"],
        "Nikolaevna"
    );
    assert_eq!(body["individual"]["primary_name"]["surname"], "Romanova");
    assert_eq!(body["individual"]["sex"], "F");
    assert_eq!(body["individual"]["is_living"], true);
    assert_eq!(body["individual"]["birth_sort"], 19900515);
    assert_eq!(
        body["individual"]["birth"]["place"],
        "Saint Petersburg, Russia"
    );

    // User profile linked atomically
    assert_eq!(body["user"]["person_id"], person_id);
    assert_eq!(body["user"]["home_person_id"], person_id);

    // Verify birth event E0100 and change_log entry were created
    {
        let store = state.privacy_store.read().await;
        assert_eq!(store.persons.len(), 1);
        assert_eq!(store.events.len(), 1);
        assert_eq!(store.events[0].gid, "E0100");
        assert_eq!(store.events[0].event_type, "birth");
        assert_eq!(store.events[0].date_sort, Some(19900515));
        assert_eq!(store.places.len(), 2); // Saint Petersburg + Russia
        assert!(!store.change_log.is_empty());
        assert_eq!(store.change_log.last().unwrap().op, "create");
        assert_eq!(store.change_log.last().unwrap().entity_id, person_id);
    }

    // Verify onboarding status after onboarding
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let status_body = read_json(resp).await;
    assert_eq!(status_body["is_empty"], false);
    assert_eq!(status_body["total_individuals"], 1);
    assert_eq!(status_body["can_onboard"], false);
    assert_eq!(status_body["resolved_home_person_id"], person_id);
    assert_eq!(status_body["default_home_person_id"], person_id);
    assert_eq!(status_body["user_home_person_id"], person_id);
    assert_eq!(status_body["user_person_id"], person_id);

    // 3. Non-admin Editor calling POST /api/v1/onboarding/start-with-self when tree is already non-empty receives 409 Conflict (already_onboarded)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &editor_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Elena",
                        "surname": "Editor",
                        "sex": "F"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let err_body = read_json(resp).await;
    assert_eq!(err_body["error"]["code"], "already_onboarded");
}

#[tokio::test]
async fn test_home_person_resolution_hierarchy_and_user_preferences() {
    let (state, admin_cookie, editor_cookie, viewer_cookie) = setup_empty_state_with_roles().await;
    let app = create_router(state.clone());

    // 1. Admin creates root person P1 via start-with-self -> sets default_home_person_id = P1, admin.person_id = P1, admin.home_person_id = P1
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/onboarding/start-with-self")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Anna",
                        "surname": "Root",
                        "sex": "F"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let p1_id = read_json(resp).await["individual"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Create two additional people: P2 and P3
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Elena",
                        "surname": "Branch",
                        "sex": "F"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let p2_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/persons")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "given_name": "Mikhail",
                        "surname": "Ancestor",
                        "sex": "M",
                        "birth_date": "1880",
                        "death_date": "1955"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let p3_id = read_json(resp).await["id"].as_str().unwrap().to_string();

    // 3. Viewer has neither home_person_id nor person_id -> falls back to instance_settings.default_home_person_id (P1)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree")
                .header(header::COOKIE, &viewer_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let tree_body = read_json(resp).await;
    assert_eq!(tree_body["resolved_home_person_id"], p1_id);

    // 4. Admin links Editor's person_id to P2 via PATCH /api/v1/admin/users/{id}
    let editor_user_id = state
        .auth_store
        .read()
        .await
        .users
        .iter()
        .find(|u| u.email == "editor@example.com")
        .unwrap()
        .id;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/v1/admin/users/{editor_user_id}"))
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "person_id": p2_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Editor now resolves to P2
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &editor_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_body = read_json(resp).await;
    assert_eq!(status_body["resolved_home_person_id"], p2_id);
    assert_eq!(status_body["user_person_id"], p2_id);

    // 5. Editor sets personal override home_person_id to P3 via PUT /api/v1/me/home-person
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/api/v1/me/home-person")
                .header(header::COOKIE, &editor_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "person_id": p3_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let me_body = read_json(resp).await;
    assert_eq!(me_body["home_person_id"], p3_id);
    assert_eq!(me_body["person_id"], p2_id);

    // Editor now resolves to P3 (user.home_person_id takes priority over user.person_id and default_home_person_id)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/tree")
                .header(header::COOKIE, &editor_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let tree_body = read_json(resp).await;
    assert_eq!(tree_body["resolved_home_person_id"], p3_id);

    // 6. Editor clears personal override (person_id: null) -> falls back to user.person_id (P2)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/api/v1/me/home-person")
                .header(header::COOKIE, &editor_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "person_id": null
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let me_body = read_json(resp).await;
    assert!(me_body["home_person_id"].is_null());

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &editor_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_body = read_json(resp).await;
    assert_eq!(status_body["resolved_home_person_id"], p2_id);

    // 7. Setting a non-existent person in PUT /api/v1/me/home-person returns 404
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/api/v1/me/home-person")
                .header(header::COOKIE, &editor_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "person_id": "I9999"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // 8. Admin updates default_home_person_id to P3 via PATCH /api/v1/admin/settings -> Viewer now resolves to P3
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "default_home_person_id": p3_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let settings_body = read_json(resp).await;
    assert_eq!(settings_body["default_home_person_id"], p3_id);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &viewer_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let viewer_status = read_json(resp).await;
    assert_eq!(viewer_status["resolved_home_person_id"], p3_id);

    // 9. Admin clears default_home_person_id -> Viewer resolves to None (null)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::COOKIE, &admin_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "default_home_person_id": null
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/onboarding/status")
                .header(header::COOKIE, &viewer_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let viewer_status = read_json(resp).await;
    assert!(viewer_status["resolved_home_person_id"].is_null());

    // 10. PATCH /api/v1/me updates start_page and digest_opt_in, and rejects invalid start_page with 400
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/me")
                .header(header::COOKIE, &editor_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "start_page": "dashboard",
                        "digest_opt_in": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated_me = read_json(resp).await;
    assert_eq!(updated_me["start_page"], "dashboard");
    assert_eq!(updated_me["digest_opt_in"], true);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/me")
                .header(header::COOKIE, &editor_cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-requested-with", "genealogy")
                .body(Body::from(
                    json!({
                        "start_page": "invalid_page"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    assert_eq!(CURRENT_SCHEMA_VERSION, 6);
}
