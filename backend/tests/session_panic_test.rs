use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
    routing::get,
    Router,
};
use genealogy_backend::{
    config::AppConfig,
    error::ErrorResponse,
    routes::{create_router, handle_panic, rbac::TreeResponse},
    services::auth::{AuthService, SetupRequest},
    state::AppState,
};
use tower::ServiceExt;
use tower_http::catch_panic::CatchPanicLayer;

async fn panic_string_handler() -> &'static str {
    panic!("explicit panic message");
}

async fn panic_str_handler() -> &'static str {
    panic!("static str panic");
}

#[tokio::test]
async fn test_catch_panic_layer_returns_500_error_response() {
    let app = Router::new()
        .route("/panic-string", get(panic_string_handler))
        .route("/panic-str", get(panic_str_handler))
        .layer(CatchPanicLayer::custom(handle_panic));

    // 1. String panic
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/panic-string")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err_body: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err_body.error.code, "internal_error");
    assert_eq!(err_body.error.message, "An internal server error occurred");

    // 2. &'static str panic
    let res = app
        .oneshot(
            Request::builder()
                .uri("/panic-str")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let err_body: ErrorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err_body.error.code, "internal_error");
    assert_eq!(err_body.error.message, "An internal server error occurred");
}

#[tokio::test]
async fn test_auth_me_evicts_dead_cookie_on_401() {
    let config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "Cookie Eviction Test".to_string(),
        ..AppConfig::default()
    };
    let state = AppState::new(None, config);
    let app = create_router(state);

    // 1. GET /api/v1/auth/me without session returns 401 with Set-Cookie eviction
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let set_cookie = res
        .headers()
        .get(header::SET_COOKIE)
        .expect("Set-Cookie header must be present on 401")
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("session="));
    assert!(set_cookie.contains("Max-Age=0"));
    assert!(set_cookie.contains("Path=/"));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(!set_cookie.contains("Secure"));

    // 2. GET /api/v1/auth/me with invalid / corrupted cookie returns 401 with Set-Cookie eviction
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header(header::COOKIE, "session=corrupted-dead-token-12345")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let set_cookie = res
        .headers()
        .get(header::SET_COOKIE)
        .expect("Set-Cookie header must be present on 401")
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("session="));
    assert!(set_cookie.contains("Max-Age=0"));

    // 3. GET /api/v1/auth/me behind HTTPS proxy (X-Forwarded-Proto: https) adds Secure flag
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header("x-forwarded-proto", "https")
                .header("x-requested-with", "genealogy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let set_cookie = res
        .headers()
        .get(header::SET_COOKIE)
        .expect("Set-Cookie header must be present on 401")
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("Max-Age=0"));
    assert!(set_cookie.contains("; Secure"));
}

#[tokio::test]
async fn test_public_or_auth_extractor_fallback() {
    let mut config = AppConfig {
        app_env: "test".to_string(),
        tree_title: "PublicOrAuth Fallback Test".to_string(),
        enable_public_access: false,
        ..AppConfig::default()
    };
    config.auth_password_enabled = "true".to_string();
    let state = AppState::new(None, config);
    let app = create_router(state.clone());

    // Setup initial admin
    let setup_req = SetupRequest {
        tree_title: Some("PublicOrAuth Fallback Test".to_string()),
        first_name: "Admin".to_string(),
        last_name: "User".to_string(),
        email: "admin@example.com".to_string(),
        passkey: None,
        password: Some("AdminStrongPass123!".to_string()),
        emergency_token: None,
    };
    let (setup_resp, _admin_cookie) =
        AuthService::setup_initial_admin(&state, setup_req, None, None, false)
            .await
            .unwrap();

    let admin_auth = format!("Bearer {}", setup_resp.session_token);

    // 1. Private Mode (enable_public_access = false):
    // Stale/invalid token returns 401 Unauthorized
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .header(header::AUTHORIZATION, "Bearer invalid-token-xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .header(header::COOKIE, "session=dead-token-xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Enable public access via admin settings PATCH
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/v1/admin/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, &admin_auth)
                .header("x-requested-with", "genealogy")
                .body(Body::from(r#"{"enable_public_access":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Public Mode (enable_public_access = true):
    // Stale/invalid token now gracefully falls back to AnonymousPublic (200 OK)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .header(header::AUTHORIZATION, "Bearer invalid-token-xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let tree: TreeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(tree.access_tier, "public_anonymous");

    // Stale/corrupted cookie also gracefully falls back to AnonymousPublic (200 OK)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .header(header::COOKIE, "session=dead-token-xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let tree: TreeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(tree.access_tier, "public_anonymous");

    // Valid admin token still authenticates as admin
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tree")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let tree: TreeResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(tree.access_tier, "admin");
}
