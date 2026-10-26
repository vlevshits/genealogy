use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String, Option<HashMap<String, String>>),

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Session expired")]
    SessionExpired,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found")]
    NotFound,

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ValidationFailed(_, _) => "validation_failed",
            Self::NotAuthenticated => "not_authenticated",
            Self::SessionExpired => "session_expired",
            Self::Forbidden(_) => "forbidden",
            Self::NotFound => "not_found",
            Self::VersionConflict(_) => "version_conflict",
            Self::RateLimited => "rate_limited",
            Self::DatabaseError(_) => "database_error",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationFailed(_, _) => StatusCode::BAD_REQUEST,
            Self::NotAuthenticated | Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::VersionConflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code().to_string();
        let message = self.to_string();

        let field_errors = match &self {
            Self::ValidationFailed(_, fields) => fields.clone(),
            _ => None,
        };

        let body = ErrorResponse {
            error: ErrorDetail {
                code,
                message,
                details: None,
                field_errors,
            },
        };

        (status, Json(body)).into_response()
    }
}
