use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap},
};

use crate::{
    error::AppError,
    services::auth::{AuthService, SessionRecord, UserRecord},
    state::AppState,
};

/// Canonical 4-tier RBAC caller classification (`docs/features/02-rbac-access-control.md §1–§3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallerAccess {
    /// Tier 2: Unauthenticated visitor when `ENABLE_PUBLIC_ACCESS=true`.
    AnonymousPublic,
    /// Tier 3: Authenticated family member with read-only access (including living relatives).
    Viewer,
    /// Tier 4: Authenticated curator with CRUD access to tree & media (including `is_private=true`).
    Editor,
    /// Tier 5: Instance owner with full administrative, bulk export/import, audit & merge authority.
    Admin,
}

impl CallerAccess {
    pub fn from_role_str(role: &str) -> Self {
        match role.trim().to_ascii_lowercase().as_str() {
            "admin" => Self::Admin,
            "editor" => Self::Editor,
            _ => Self::Viewer,
        }
    }

    pub fn as_tier_str(&self) -> &'static str {
        match self {
            Self::AnonymousPublic => "public_anonymous",
            Self::Viewer => "viewer",
            Self::Editor => "editor",
            Self::Admin => "admin",
        }
    }

    pub fn can_view_private(&self) -> bool {
        matches!(self, Self::Editor | Self::Admin)
    }

    pub fn can_view_living_details(&self) -> bool {
        matches!(self, Self::Viewer | Self::Editor | Self::Admin)
    }

    pub fn can_edit(&self) -> bool {
        matches!(self, Self::Editor | Self::Admin)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

/// Extracts session token from `Authorization: Bearer <token>` or `Cookie: session=<token>` / `genealogy_session=<token>`.
pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    if let Some(cookie_str) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        for part in cookie_str.split(';') {
            let trimmed = part.trim();
            if let Some(val) = trimmed
                .strip_prefix("session=")
                .or_else(|| trimmed.strip_prefix("genealogy_session="))
            {
                let token = val.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: UserRecord,
    pub session: SessionRecord,
}

impl AuthenticatedUser {
    pub fn access(&self) -> CallerAccess {
        CallerAccess::from_role_str(&self.user.role)
    }
}

/// Requires any valid authenticated user (`Viewer`, `Editor`, or `Admin`).
/// Rejects unauthenticated or deactivated users with `401 Unauthorized`.
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthenticatedUser);

/// Requires an authenticated `Editor` or `Admin`.
/// Rejects unauthenticated users with `401 Unauthorized` and `Viewer` with `403 Forbidden`.
#[derive(Debug, Clone)]
pub struct RequireEditor(pub AuthenticatedUser);

/// Requires an authenticated `Admin`.
/// Rejects unauthenticated users with `401 Unauthorized` and `Viewer`/`Editor` with `403 Forbidden`.
#[derive(Debug, Clone)]
pub struct RequireAdmin(pub AuthenticatedUser);

/// Resolves an authenticated user if a valid session is present; otherwise allows anonymous
/// public access ONLY when `enable_public_access == true` (`02-rbac-access-control.md §6.1`).
#[derive(Debug, Clone)]
pub enum PublicOrAuth {
    Authenticated(Box<AuthenticatedUser>),
    AnonymousPublic,
}

impl PublicOrAuth {
    pub fn caller_access(&self) -> CallerAccess {
        match self {
            Self::Authenticated(auth) => auth.access(),
            Self::AnonymousPublic => CallerAccess::AnonymousPublic,
        }
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_session_token(&parts.headers).ok_or(AppError::NotAuthenticated)?;
        let (user, session) = AuthService::authenticate_session(state, &token).await?;
        if !user.is_active {
            return Err(AppError::NotAuthenticated);
        }
        Ok(AuthenticatedUser { user, session })
    }
}

impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthenticatedUser::from_request_parts(parts, state).await?;
        Ok(RequireAuth(auth))
    }
}

impl FromRequestParts<AppState> for RequireEditor {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthenticatedUser::from_request_parts(parts, state).await?;
        if !auth.access().can_edit() {
            return Err(AppError::Forbidden(
                "Editor or Administrator privileges required".to_string(),
            ));
        }
        Ok(RequireEditor(auth))
    }
}

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthenticatedUser::from_request_parts(parts, state).await?;
        if !auth.access().is_admin() {
            return Err(AppError::Forbidden(
                "Administrator privileges required".to_string(),
            ));
        }
        Ok(RequireAdmin(auth))
    }
}

impl FromRequestParts<AppState> for PublicOrAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(token) = extract_session_token(&parts.headers) {
            let (user, session) = AuthService::authenticate_session(state, &token).await?;
            if !user.is_active {
                return Err(AppError::NotAuthenticated);
            }
            return Ok(PublicOrAuth::Authenticated(Box::new(AuthenticatedUser {
                user,
                session,
            })));
        }

        let settings = state.resolve_instance_settings().await;
        if settings.enable_public_access {
            Ok(PublicOrAuth::AnonymousPublic)
        } else {
            Err(AppError::NotAuthenticated)
        }
    }
}
