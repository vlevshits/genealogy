#[allow(unused_imports)]
use super::*;

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
