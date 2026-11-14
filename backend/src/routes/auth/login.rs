#[allow(unused_imports)]
use super::*;

#[derive(Debug, Deserialize)]
pub struct MagicLinkQuery {
    pub token: String,
}

pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    crate::services::rbac::extract_session_token(headers)
}

pub(crate) async fn require_authenticated(
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
            let ai_cfg = store.ai_config.clone();
            let preserved_tokens = std::mem::take(&mut store.personal_access_tokens);
            *store = crate::services::privacy::GenealogyPrivacyStore::empty();
            store.ai_config = ai_cfg;
            store.personal_access_tokens = preserved_tokens;
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
