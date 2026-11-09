#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current authenticated user profile", body = UserDto),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn get_current_user(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match require_authenticated(&state, &headers).await {
        Ok((user, _)) => {
            let dto = AuthService::build_user_dto(&state, &user).await;
            Json(dto).into_response()
        }
        Err(err @ (AppError::NotAuthenticated | AppError::SessionExpired)) => {
            let is_configured_https = state
                .config
                .app_origin
                .split([',', ';'])
                .any(|o| o.trim().starts_with("https://"));
            let is_forwarded_https = headers
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .is_some_and(|proto| proto.trim().eq_ignore_ascii_case("https"));
            let secure_flag = if is_forwarded_https || is_configured_https {
                "; Secure"
            } else {
                ""
            };
            let clear_cookie =
                format!("session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{secure_flag}");

            let mut resp = err.into_response();
            if let Ok(cookie_val) = header::HeaderValue::from_str(&clear_cookie) {
                resp.headers_mut().insert(header::SET_COOKIE, cookie_val);
            }
            resp
        }
        Err(err) => err.into_response(),
    }
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
