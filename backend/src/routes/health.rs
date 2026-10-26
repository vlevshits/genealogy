use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: String,
    pub environment: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "System is healthy", body = HealthResponse),
        (status = 503, description = "Database unavailable or system degraded", body = HealthResponse)
    ),
    tag = "System"
)]
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let (db_status, is_healthy) = if let Some(pool) = &state.db {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => ("connected".to_string(), true),
            Err(err) => {
                tracing::warn!("Database healthcheck ping failed: {err}");
                ("error".to_string(), false)
            }
        }
    } else {
        // In local development or standalone without DB yet, report standalone_ready
        ("standalone_ready".to_string(), true)
    };

    let response = HealthResponse {
        status: if is_healthy { "ok".to_string() } else { "degraded".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status,
        environment: state.config.app_env.clone(),
    };

    if is_healthy {
        (StatusCode::OK, Json(response)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response()
    }
}
