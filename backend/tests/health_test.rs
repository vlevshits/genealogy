use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use genealogy_backend::{config::AppConfig, routes::create_router, state::AppState};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check_endpoint() {
    let config = AppConfig {
        app_env: "test".to_string(),
        enable_public_access: true,
        ..AppConfig::default()
    };

    let state = AppState::new(None, config);
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_openapi_endpoint() {
    let config = AppConfig {
        app_env: "test".to_string(),
        ..AppConfig::default()
    };

    let state = AppState::new(None, config);
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
