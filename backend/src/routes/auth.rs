use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, ErrorResponse},
    services::{
        auth::{
            AdminSignInLinkResponse, AdminUserListResponse, AdminUserSummaryDto, AuthService,
            AuthSessionResponse, CreateInviteRequest, InvitationListResponse,
            InviteCreatedResponse, InvitePreviewResponse, InviteRegisterRequest,
            LoginChallengeRequest, LoginChallengeResponse, MagicLinkRequest,
            MagicLinkRequestResponse, MagicLinkVerifyRequest, PasskeyCredentialDto,
            PasskeyListResponse, PasskeyLoginVerifyRequest, PasskeyRegistrationInput,
            PasswordLoginRequest, RecoveryCodeLoginRequest, RecoveryCodeLoginResponse,
            RegenerateRecoveryCodesResponse, RenamePasskeyRequest, SessionDto, SessionListResponse,
            SessionRecord, SetHomePersonRequest, SetPasswordRequest, SetupCompleteResponse,
            SetupRequest, SetupStatusResponse, UpdateAdminUserRequest, UpdateProfileRequest,
            UserDto, UserRecord, WebAuthnChallengeResponse,
        },
        rbac::RequireAdmin,
        security::RequestContext,
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct MagicLinkQuery {
    pub token: String,
}

pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    crate::services::rbac::extract_session_token(headers)
}

async fn require_authenticated(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(UserRecord, SessionRecord), AppError> {
    let token = extract_session_token(headers).ok_or(AppError::NotAuthenticated)?;
    AuthService::authenticate_session(state, &token).await
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/setup/status",
    responses(
        (status = 200, description = "First-run setup status and WebAuthn RP config", body = SetupStatusResponse)
    ),
    tag = "Auth"
)]
pub async fn get_setup_status(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
) -> Json<SetupStatusResponse> {
    let is_initialized = AuthService::is_initialized(&state).await;
    let settings = state.resolve_instance_settings().await;
    Json(SetupStatusResponse {
        is_initialized,
        tree_title: settings.tree_title,
        default_locale: settings.default_locale,
        enable_public_access: settings.enable_public_access,
        auth_password_enabled: state.config.is_password_auth_effectively_enabled(),
        rp_id: ctx.effective_rp_id,
        rp_name: state.config.rp_name.clone(),
        smtp_enabled: state.config.smtp_enabled,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register/challenge",
    responses(
        (status = 200, description = "WebAuthn registration challenge", body = WebAuthnChallengeResponse),
        (status = 429, description = "Rate limited", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn create_register_challenge(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Json<WebAuthnChallengeResponse>, AppError> {
    state.rate_limiter.check_auth_ip(&ctx.client_ip).await?;
    let challenge = AuthService::create_challenge(&state, None).await?;
    Ok(Json(WebAuthnChallengeResponse {
        challenge,
        rp_id: ctx.effective_rp_id,
        rp_name: state.config.rp_name.clone(),
        timeout_ms: 60_000,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/setup",
    request_body = SetupRequest,
    responses(
        (status = 201, description = "Initial admin account created with 10 recovery codes", body = SetupCompleteResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Setup already completed", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn complete_initial_setup(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<SetupRequest>,
) -> Result<Response, AppError> {
    state.rate_limiter.check_auth_ip(&ctx.client_ip).await?;

    let (resp, cookie) = AuthService::setup_initial_admin(
        &state,
        req,
        ctx.user_agent,
        Some(ctx.client_ip),
        ctx.is_https,
    )
    .await?;
    {
        let mut store = state.privacy_store.write().await;
        if store.is_demo_seed {
            *store = crate::services::privacy::GenealogyPrivacyStore::empty();
        }
    }
    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie)],
        Json(resp),
    )
        .into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/invites/{token}",
    params(
        ("token" = String, Path, description = "One-time invitation token")
    ),
    responses(
        (status = 200, description = "Invitation preview details", body = InvitePreviewResponse),
        (status = 404, description = "Invalid or expired invitation", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn preview_invitation(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<InvitePreviewResponse>, AppError> {
    let preview = AuthService::preview_invitation(&state, &token).await?;
    Ok(Json(preview))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register/verify",
    request_body = InviteRegisterRequest,
    responses(
        (status = 201, description = "Invited user registered and signed in", body = SetupCompleteResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Invalid or expired invitation", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn verify_invite_registration(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<InviteRegisterRequest>,
) -> Result<Response, AppError> {
    state.rate_limiter.check_auth_ip(&ctx.client_ip).await?;

    let (resp, cookie) = AuthService::register_with_invitation(
        &state,
        req,
        ctx.user_agent,
        Some(ctx.client_ip),
        ctx.is_https,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie)],
        Json(resp),
    )
        .into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login/challenge",
    request_body(content = Option<LoginChallengeRequest>, description = "Optional user email for allowCredentials"),
    responses(
        (status = 200, description = "WebAuthn assertion challenge", body = LoginChallengeResponse),
        (status = 429, description = "Rate limited", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn create_login_challenge(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    payload: Option<Json<LoginChallengeRequest>>,
) -> Result<Json<LoginChallengeResponse>, AppError> {
    state.rate_limiter.check_auth_ip(&ctx.client_ip).await?;

    let email_opt = payload
        .and_then(|Json(p)| p.email)
        .map(|e| e.trim().to_ascii_lowercase())
        .filter(|e| !e.is_empty());

    let mut allow_credentials = Vec::new();
    let mut user_id_opt = None;

    if let Some(ref email) = email_opt {
        if let Ok(Some(user)) = AuthService::ensure_user_and_passkeys_by_email(&state, email).await
        {
            user_id_opt = Some(user.id);
            let store = state.auth_store.read().await;
            allow_credentials = store
                .passkeys
                .iter()
                .filter(|p| p.user_id == user.id)
                .map(|p| p.credential_id.clone())
                .collect();
        }
    }

    let challenge = AuthService::create_challenge(&state, user_id_opt).await?;
    Ok(Json(LoginChallengeResponse {
        challenge,
        rp_id: ctx.effective_rp_id,
        timeout_ms: 60_000,
        allow_credentials,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login/verify",
    request_body = PasskeyLoginVerifyRequest,
    responses(
        (status = 200, description = "Passkey assertion verified and session created", body = AuthSessionResponse),
        (status = 400, description = "Invalid or expired challenge", body = ErrorResponse),
        (status = 401, description = "Invalid passkey or replay detected", body = ErrorResponse),
        (status = 429, description = "Rate limited or account locked", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn verify_passkey_login(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<PasskeyLoginVerifyRequest>,
) -> Result<Response, AppError> {
    state.rate_limiter.check_auth_ip(&ctx.client_ip).await?;

    let (resp, cookie) = AuthService::verify_passkey_login(
        &state,
        req,
        ctx.user_agent,
        Some(ctx.client_ip),
        Some(ctx.effective_rp_id),
        Some(ctx.host),
        ctx.is_https,
    )
    .await?;
    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie)], Json(resp)).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/password/login",
    request_body = PasswordLoginRequest,
    responses(
        (status = 200, description = "Signed in with password", body = AuthSessionResponse),
        (status = 401, description = "Invalid email or password", body = ErrorResponse),
        (status = 403, description = "Password authentication disabled", body = ErrorResponse),
        (status = 429, description = "Rate limited or account locked", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn login_with_password(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<PasswordLoginRequest>,
) -> Result<Response, AppError> {
    state.rate_limiter.check_auth_ip(&ctx.client_ip).await?;

    let (resp, cookie) = AuthService::password_login(
        &state,
        req,
        ctx.user_agent,
        Some(ctx.client_ip),
        ctx.is_https,
    )
    .await?;
    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie)], Json(resp)).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/magic-link/request",
    request_body = MagicLinkRequest,
    responses(
        (status = 200, description = "Magic sign-in link generated", body = MagicLinkRequestResponse),
        (status = 429, description = "Rate limited (max 3 per 15 minutes per email)", body = ErrorResponse),
        (status = 503, description = "SMTP email delivery is not configured on this server", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn request_magic_link(
    State(state): State<AppState>,
    Json(req): Json<MagicLinkRequest>,
) -> Result<Json<MagicLinkRequestResponse>, AppError> {
    let resp = AuthService::request_magic_link(&state, &req.email).await?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/magic-link/verify",
    params(
        ("token" = String, Query, description = "One-time magic link token")
    ),
    responses(
        (status = 200, description = "Magic link verified and session issued", body = AuthSessionResponse),
        (status = 400, description = "Invalid or expired magic link token", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn verify_magic_link_get(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Query(query): Query<MagicLinkQuery>,
) -> Result<Response, AppError> {
    let (resp, cookie) = AuthService::verify_magic_link(
        &state,
        &query.token,
        ctx.user_agent,
        Some(ctx.client_ip),
        ctx.is_https,
    )
    .await?;
    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie)], Json(resp)).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/magic-link/verify",
    request_body = MagicLinkVerifyRequest,
    responses(
        (status = 200, description = "Magic link verified and session issued", body = AuthSessionResponse),
        (status = 400, description = "Invalid or expired magic link token", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn verify_magic_link_post(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<MagicLinkVerifyRequest>,
) -> Result<Response, AppError> {
    let (resp, cookie) = AuthService::verify_magic_link(
        &state,
        &req.token,
        ctx.user_agent,
        Some(ctx.client_ip),
        ctx.is_https,
    )
    .await?;
    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie)], Json(resp)).into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/recovery-code",
    request_body = RecoveryCodeLoginRequest,
    responses(
        (status = 200, description = "Recovery code validated and session issued", body = RecoveryCodeLoginResponse),
        (status = 401, description = "Invalid recovery code", body = ErrorResponse),
        (status = 429, description = "Rate limited or account locked", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn login_with_recovery_code(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    Json(req): Json<RecoveryCodeLoginRequest>,
) -> Result<Response, AppError> {
    state.rate_limiter.check_auth_ip(&ctx.client_ip).await?;

    let (resp, cookie) = AuthService::login_with_recovery_code(
        &state,
        req,
        ctx.user_agent,
        Some(ctx.client_ip),
        ctx.is_https,
    )
    .await?;
    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie)], Json(resp)).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current authenticated user profile", body = UserDto),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserDto>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let dto = AuthService::build_user_dto(&state, &user).await;
    Ok(Json(dto))
}

#[utoipa::path(
    patch,
    path = "/api/v1/auth/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Updated current user profile", body = UserDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse),
        (status = 404, description = "Referenced home person not found", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn update_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserDto>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let dto = AuthService::update_profile(&state, user.id, req).await?;
    Ok(Json(dto))
}

#[utoipa::path(
    patch,
    path = "/api/v1/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Updated current user preferences (start_page, digest_opt_in, locale, theme, home_person_id)", body = UserDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse),
        (status = 404, description = "Referenced home person not found", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn update_me_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserDto>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let dto = AuthService::update_profile(&state, user.id, req).await?;
    Ok(Json(dto))
}

#[utoipa::path(
    put,
    path = "/api/v1/me/home-person",
    request_body = SetHomePersonRequest,
    responses(
        (status = 200, description = "Set or cleared personal home person override", body = UserDto),
        (status = 401, description = "Not authenticated", body = ErrorResponse),
        (status = 404, description = "Referenced person not found or hidden", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn set_me_home_person(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SetHomePersonRequest>,
) -> Result<Json<UserDto>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let dto = AuthService::set_home_person(&state, user.id, req.person_id).await?;
    Ok(Json(dto))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 204, description = "Session invalidated and cookie cleared")
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    if let Some(token) = extract_session_token(&headers) {
        if let Ok((_, sess)) = AuthService::authenticate_session(&state, &token).await {
            state
                .auth_store
                .write()
                .await
                .sessions
                .retain(|s| s.id != sess.id);
            if let Some(pool) = &state.db {
                let _ = sqlx::query("DELETE FROM sessions WHERE id = $1")
                    .bind(sess.id)
                    .execute(pool)
                    .await;
            }
        }
    }

    let is_configured_https = state
        .config
        .app_origin
        .split([',', ';'])
        .any(|o| o.trim().starts_with("https://"));
    let secure_flag = if ctx.is_https || is_configured_https {
        "; Secure"
    } else {
        ""
    };
    let clear_cookie = format!("session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{secure_flag}");
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, clear_cookie)]).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/passkeys",
    responses(
        (status = 200, description = "Registered passkey credentials for current user", body = PasskeyListResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn list_passkeys(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PasskeyListResponse>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let passkeys = AuthService::ensure_passkeys_by_user_id(&state, user.id).await?;
    let items = passkeys.iter().map(AuthService::to_passkey_dto).collect();
    Ok(Json(PasskeyListResponse { items }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/passkeys/add/challenge",
    responses(
        (status = 200, description = "WebAuthn challenge for adding a passkey", body = WebAuthnChallengeResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn create_add_passkey_challenge(
    State(state): State<AppState>,
    Extension(ctx): Extension<RequestContext>,
    headers: HeaderMap,
) -> Result<Json<WebAuthnChallengeResponse>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let challenge = AuthService::create_challenge(&state, Some(user.id)).await?;
    Ok(Json(WebAuthnChallengeResponse {
        challenge,
        rp_id: ctx.effective_rp_id,
        rp_name: state.config.rp_name.clone(),
        timeout_ms: 60_000,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/passkeys/add/verify",
    request_body = PasskeyRegistrationInput,
    responses(
        (status = 201, description = "Additional passkey registered", body = PasskeyCredentialDto),
        (status = 400, description = "Invalid challenge or credential", body = ErrorResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn verify_add_passkey(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PasskeyRegistrationInput>,
) -> Result<(StatusCode, Json<PasskeyCredentialDto>), AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let dto = AuthService::add_passkey(&state, user.id, req).await?;
    Ok((StatusCode::CREATED, Json(dto)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/auth/passkeys/{id}",
    params(
        ("id" = String, Path, description = "Passkey credential UUID")
    ),
    request_body = RenamePasskeyRequest,
    responses(
        (status = 200, description = "Passkey renamed", body = PasskeyCredentialDto),
        (status = 404, description = "Passkey not found", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn rename_passkey(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<RenamePasskeyRequest>,
) -> Result<Json<PasskeyCredentialDto>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let pk_id = Uuid::parse_str(&id).map_err(|_| AppError::NotFound)?;
    let dto = AuthService::rename_passkey(&state, user.id, pk_id, &req.device_name).await?;
    Ok(Json(dto))
}

#[utoipa::path(
    delete,
    path = "/api/v1/auth/passkeys/{id}",
    params(
        ("id" = String, Path, description = "Passkey credential UUID")
    ),
    responses(
        (status = 204, description = "Passkey removed"),
        (status = 400, description = "Cannot remove last remaining authentication method", body = ErrorResponse),
        (status = 404, description = "Passkey not found", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn delete_passkey(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let pk_id = Uuid::parse_str(&id).map_err(|_| AppError::NotFound)?;
    AuthService::delete_passkey(&state, user.id, pk_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/recovery-codes/regenerate",
    responses(
        (status = 200, description = "10 new single-use recovery codes generated", body = RegenerateRecoveryCodesResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn regenerate_recovery_codes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RegenerateRecoveryCodesResponse>, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let codes = AuthService::regenerate_recovery_codes(&state, user.id).await?;
    Ok(Json(RegenerateRecoveryCodesResponse {
        recovery_codes: codes,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/auth/password",
    request_body = SetPasswordRequest,
    responses(
        (status = 204, description = "Password updated"),
        (status = 400, description = "Password too short (< 10 chars)", body = ErrorResponse),
        (status = 401, description = "Not authenticated or current password mismatch", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn set_user_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    AuthService::set_password(&state, user.id, req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/sessions",
    responses(
        (status = 200, description = "Active sessions for current user", body = SessionListResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SessionListResponse>, AppError> {
    let (user, current_sess) = require_authenticated(&state, &headers).await?;
    let store = state.auth_store.read().await;
    let items = store
        .sessions
        .iter()
        .filter(|s| s.user_id == user.id)
        .map(|s| SessionDto {
            id: s.id.to_string(),
            user_agent: s.user_agent.clone(),
            ip_address: s.ip_address.clone(),
            created_at: s.created_at.to_rfc3339(),
            last_seen_at: s.last_seen_at.to_rfc3339(),
            expires_at: s.expires_at.to_rfc3339(),
            is_current: s.id == current_sess.id,
        })
        .collect();
    Ok(Json(SessionListResponse { items }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/auth/sessions/{id}",
    params(
        ("id" = String, Path, description = "Session UUID to revoke")
    ),
    responses(
        (status = 204, description = "Session revoked"),
        (status = 404, description = "Session not found", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn revoke_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let (user, _) = require_authenticated(&state, &headers).await?;
    let sess_id = Uuid::parse_str(&id).map_err(|_| AppError::NotFound)?;

    let mut store = state.auth_store.write().await;
    let before = store.sessions.len();
    store
        .sessions
        .retain(|s| !(s.id == sess_id && s.user_id == user.id));
    if store.sessions.len() == before {
        return Err(AppError::NotFound);
    }
    drop(store);

    if let Some(pool) = &state.db {
        let _ = sqlx::query("DELETE FROM sessions WHERE id = $1 AND user_id = $2")
            .bind(sess_id)
            .bind(user.id)
            .execute(pool)
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/auth/sessions",
    responses(
        (status = 204, description = "All other sessions revoked")
    ),
    tag = "Auth"
)]
pub async fn revoke_other_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let (user, current_sess) = require_authenticated(&state, &headers).await?;
    state
        .auth_store
        .write()
        .await
        .sessions
        .retain(|s| s.user_id != user.id || s.id == current_sess.id);

    if let Some(pool) = &state.db {
        let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id <> $2")
            .bind(user.id)
            .bind(current_sess.id)
            .execute(pool)
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Admin User & Invitation Management Endpoints
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    responses(
        (status = 200, description = "List of family members and roles", body = AdminUserListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn list_admin_users(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<AdminUserListResponse>, AppError> {
    let store = state.auth_store.read().await;
    let items = store
        .users
        .iter()
        .map(|u| {
            let passkeys_count = store.passkeys.iter().filter(|p| p.user_id == u.id).count();
            AuthService::to_admin_user_summary(u, passkeys_count)
        })
        .collect();
    Ok(Json(AdminUserListResponse { items }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/admin/users/{id}",
    params(
        ("id" = String, Path, description = "Target user UUID")
    ),
    request_body = UpdateAdminUserRequest,
    responses(
        (status = 200, description = "Updated user role or activation status", body = AdminUserSummaryDto),
        (status = 400, description = "Validation error or sole admin protection", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn update_admin_user(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAdminUserRequest>,
) -> Result<Json<AdminUserSummaryDto>, AppError> {
    let target_id = Uuid::parse_str(&id).map_err(|_| AppError::NotFound)?;
    let updated = AuthService::update_admin_user(&state, target_id, req).await?;
    Ok(Json(updated))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/invites",
    request_body = CreateInviteRequest,
    responses(
        (status = 201, description = "One-time invitation created", body = InviteCreatedResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn create_admin_invite(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<InviteCreatedResponse>), AppError> {
    let created = AuthService::create_invitation(&state, Some(admin.user.id), req).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/invites",
    responses(
        (status = 200, description = "List of pending invitations", body = InvitationListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn list_admin_invites(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<InvitationListResponse>, AppError> {
    let store = state.auth_store.read().await;
    let items = store
        .invitations
        .iter()
        .filter(|i| i.used_at.is_none() && i.revoked_at.is_none())
        .map(AuthService::to_invitation_dto)
        .collect();
    Ok(Json(InvitationListResponse { items }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/invites/{id}",
    params(
        ("id" = String, Path, description = "Invitation UUID")
    ),
    responses(
        (status = 204, description = "Invitation revoked"),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Invitation not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn revoke_admin_invite(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let inv_id = Uuid::parse_str(&id).map_err(|_| AppError::NotFound)?;

    let mut store = state.auth_store.write().await;
    let inv = store
        .invitations
        .iter_mut()
        .find(|i| i.id == inv_id && i.revoked_at.is_none())
        .ok_or(AppError::NotFound)?;
    inv.revoked_at = Some(chrono::Utc::now());
    drop(store);

    if let Some(pool) = &state.db {
        let _ = sqlx::query("UPDATE invitations SET revoked_at = NOW() WHERE id = $1")
            .bind(inv_id)
            .execute(pool)
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{id}/sign-in-link",
    params(
        ("id" = String, Path, description = "Target user UUID")
    ),
    responses(
        (status = 201, description = "24-hour one-time sign-in link created", body = AdminSignInLinkResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn create_user_sign_in_link(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<AdminSignInLinkResponse>), AppError> {
    let target_id = Uuid::parse_str(&id).map_err(|_| AppError::NotFound)?;
    let link = AuthService::create_admin_sign_in_link(&state, target_id).await?;
    Ok((StatusCode::CREATED, Json(link)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{id}/sessions",
    params(
        ("id" = String, Path, description = "Target user UUID")
    ),
    responses(
        (status = 204, description = "All sessions for user revoked"),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn revoke_all_user_sessions(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let target_id = Uuid::parse_str(&id).map_err(|_| AppError::NotFound)?;

    let mut store = state.auth_store.write().await;
    if !store.users.iter().any(|u| u.id == target_id) {
        return Err(AppError::NotFound);
    }
    store.sessions.retain(|s| s.user_id != target_id);
    drop(store);

    if let Some(pool) = &state.db {
        let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(target_id)
            .execute(pool)
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}
