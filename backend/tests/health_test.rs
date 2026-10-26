use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use genealogy_backend::{config::AppConfig, routes::create_router, state::AppState};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check_endpoint() {
    let config = AppConfig {
        port: 3000,
        app_env: "test".to_string(),
        rust_log: "info".to_string(),
        app_origin: "http://localhost:8080".to_string(),
        rp_id: "localhost".to_string(),
        rp_name: "Test Family".to_string(),
        tree_title: "Test Archive".to_string(),
        default_locale: "en".to_string(),
        database_url: "postgresql://localhost/test".to_string(),
        media_storage_path: "/tmp/media".to_string(),
        backup_path: "/tmp/backups".to_string(),
        enable_public_access: true,
        enable_registration: false,
        living_person_age_threshold: 110,
        max_upload_size_mb: 100,
        auth_password_enabled: "auto".to_string(),
        session_ttl_days: 30,
        job_workers: 1,
        map_tile_url: None,
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
        port: 3000,
        app_env: "test".to_string(),
        rust_log: "info".to_string(),
        app_origin: "http://localhost:8080".to_string(),
        rp_id: "localhost".to_string(),
        rp_name: "Test Family".to_string(),
        tree_title: "Test Archive".to_string(),
        default_locale: "en".to_string(),
        database_url: "postgresql://localhost/test".to_string(),
        media_storage_path: "/tmp/media".to_string(),
        backup_path: "/tmp/backups".to_string(),
        enable_public_access: true,
        enable_registration: false,
        living_person_age_threshold: 110,
        max_upload_size_mb: 100,
        auth_password_enabled: "auto".to_string(),
        session_ttl_days: 30,
        job_workers: 1,
        map_tile_url: None,
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
