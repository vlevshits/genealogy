pub mod health;

use axum::{
    http::{header, Method},
    routing::get,
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    error::{ErrorDetail, ErrorResponse},
    routes::health::{health_check, HealthResponse},
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
    ),
    components(
        schemas(
            HealthResponse,
            ErrorResponse,
            ErrorDetail,
        )
    ),
    tags(
        (name = "System", description = "System operations and health monitoring")
    ),
    info(
        title = "Genealogy Family Tree API",
        version = "0.1.0",
        description = "Lightweight, privacy-first family tree archival API backend"
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::HeaderName::from_static("x-requested-with"),
        ])
        .allow_origin(Any);

    let api_v1 = Router::new()
        .route("/health", get(health_check));

    Router::new()
        .nest("/api/v1", api_v1)
        .merge(SwaggerUi::new("/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
