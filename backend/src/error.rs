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

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Passkey signature counter replay detected")]
    PasskeyReplayDetected,

    #[error("User verification (biometrics or device PIN) is required.")]
    UserVerificationRequired,

    #[error("Passkey assertion verification failed.")]
    PasskeyVerificationFailed,

    #[error("Invalid or expired WebAuthn challenge")]
    InvalidOrExpiredChallenge,

    #[error("Invalid or expired invitation link")]
    InvalidOrExpiredInvite,

    #[error("Invalid or expired authentication token")]
    InvalidOrExpiredToken,

    #[error("Cannot remove the last remaining authentication method")]
    LastAuthMethod,

    #[error("Cannot demote or deactivate the sole remaining active administrator")]
    SoleAdminProtected,

    #[error("Password authentication is disabled on this instance")]
    PasswordAuthDisabled,

    #[error("CSRF or Origin header validation failed")]
    CsrfValidationFailed,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Settings are locked by server environment configuration")]
    SettingsLockedFromEnv,

    #[error("Not found")]
    NotFound,

    #[error("Person not found")]
    PersonNotFound,

    #[error("First-time instance setup has already been completed")]
    SetupAlreadyCompleted,

    #[error("Family tree has already been onboarded")]
    AlreadyOnboarded,

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("Version conflict: {0}")]
    VersionConflictWithRecord(String, serde_json::Value),

    #[error("Relationship cycle detected: {0}")]
    CycleDetected(String),

    #[error("Place is currently referenced by events or sub-places and cannot be deleted")]
    PlaceInUse,

    #[error("Rate limited")]
    RateLimited,

    #[error("Account is temporarily locked due to repeated failed sign-in attempts")]
    AccountLocked,

    #[error("Backup archive checksum verification failed: {0}")]
    ChecksumMismatch(String),

    #[error("Backup archive schema version is incompatible: {0}")]
    IncompatibleSchema(String),

    #[error("Invalid archive: {0}")]
    InvalidArchive(String),

    #[error("Zip slip path traversal detected: {0}")]
    ZipSlipDetected(String),

    #[error("Decompression limit exceeded: {0}")]
    DecompressionLimitExceeded(String),

    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    #[error("Unsupported archive format: {0}")]
    UnsupportedArchiveFormat(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Email delivery is not configured on this server")]
    SmtpNotConfigured,

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ValidationFailed(_, _) => "validation_failed",
            Self::NotAuthenticated => "not_authenticated",
            Self::SessionExpired => "session_expired",
            Self::InvalidCredentials => "invalid_credentials",
            Self::PasskeyReplayDetected => "passkey_replay_detected",
            Self::UserVerificationRequired => "user_verification_required",
            Self::PasskeyVerificationFailed => "passkey_verification_failed",
            Self::InvalidOrExpiredChallenge => "invalid_or_expired_challenge",
            Self::InvalidOrExpiredInvite => "invalid_or_expired_invite",
            Self::InvalidOrExpiredToken => "invalid_or_expired_token",
            Self::LastAuthMethod => "last_auth_method",
            Self::SoleAdminProtected => "sole_admin_protected",
            Self::PasswordAuthDisabled => "password_auth_disabled",
            Self::CsrfValidationFailed => "csrf_validation_failed",
            Self::Forbidden(_) => "forbidden",
            Self::SettingsLockedFromEnv => "settings_locked_from_env",
            Self::NotFound => "not_found",
            Self::PersonNotFound => "person_not_found",
            Self::SetupAlreadyCompleted => "setup_already_completed",
            Self::AlreadyOnboarded => "already_onboarded",
            Self::VersionConflict(_) | Self::VersionConflictWithRecord(_, _) => "version_conflict",
            Self::CycleDetected(_) => "cycle_detected",
            Self::PlaceInUse => "place_in_use",
            Self::RateLimited => "rate_limited",
            Self::AccountLocked => "account_locked",
            Self::ChecksumMismatch(_) => "checksum_mismatch",
            Self::IncompatibleSchema(_) => "incompatible_schema",
            Self::InvalidArchive(_) => "invalid_archive",
            Self::ZipSlipDetected(_) => "zip_slip_detected",
            Self::DecompressionLimitExceeded(_) => "decompression_limit_exceeded",
            Self::PayloadTooLarge(_) => "upload_too_large",
            Self::UnsupportedArchiveFormat(_) => "unsupported_archive_format",
            Self::SmtpNotConfigured => "smtp_not_configured",
            Self::DatabaseError(_) => "database_error",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationFailed(_, _)
            | Self::UserVerificationRequired
            | Self::PasskeyVerificationFailed
            | Self::InvalidOrExpiredChallenge
            | Self::InvalidOrExpiredToken
            | Self::LastAuthMethod
            | Self::SoleAdminProtected
            | Self::CycleDetected(_)
            | Self::ChecksumMismatch(_)
            | Self::IncompatibleSchema(_)
            | Self::InvalidArchive(_)
            | Self::ZipSlipDetected(_)
            | Self::DecompressionLimitExceeded(_) => StatusCode::BAD_REQUEST,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedArchiveFormat(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::NotAuthenticated
            | Self::SessionExpired
            | Self::InvalidCredentials
            | Self::PasskeyReplayDetected => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_)
            | Self::SettingsLockedFromEnv
            | Self::PasswordAuthDisabled
            | Self::CsrfValidationFailed => StatusCode::FORBIDDEN,
            Self::NotFound | Self::PersonNotFound | Self::InvalidOrExpiredInvite => {
                StatusCode::NOT_FOUND
            }
            Self::VersionConflict(_)
            | Self::VersionConflictWithRecord(_, _)
            | Self::SetupAlreadyCompleted
            | Self::AlreadyOnboarded
            | Self::PlaceInUse => StatusCode::CONFLICT,
            Self::RateLimited | Self::AccountLocked => StatusCode::TOO_MANY_REQUESTS,
            Self::SmtpNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
            Self::DatabaseError(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn client_message(&self) -> String {
        match self {
            Self::DatabaseError(err) => {
                tracing::error!(error = %err, "Database operation failed");
                "A database error occurred".to_string()
            }
            Self::Internal(err) => {
                tracing::error!(error = %err, "Internal server error");
                "An internal server error occurred".to_string()
            }
            other => other.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code().to_string();
        let message = self.client_message();

        let field_errors = match &self {
            Self::ValidationFailed(_, fields) => fields.clone(),
            _ => None,
        };

        let details = match &self {
            Self::VersionConflictWithRecord(_, record_details) => Some(record_details.clone()),
            _ => None,
        };

        let body = ErrorResponse {
            error: ErrorDetail {
                code,
                message,
                details,
                field_errors,
            },
        };

        (status, Json(body)).into_response()
    }
}
