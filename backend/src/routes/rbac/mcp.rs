#[allow(unused_imports)]
use super::*;

// ============================================================================
// Milestone 3.2: Protected MCP Server & Personal Access Token Handlers
// ============================================================================

#[utoipa::path(
    get,
    path = "/api/v1/mcp/tokens",
    responses(
        (status = 200, description = "List of active Personal Access Tokens for the caller", body = crate::services::mcp::McpTokenListResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn list_mcp_tokens_handler(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<crate::services::mcp::McpTokenListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let res = crate::services::mcp::list_mcp_tokens(&store, &auth.user.id.to_string());
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/mcp/tokens",
    request_body = crate::services::mcp::CreateMcpTokenRequest,
    responses(
        (status = 201, description = "Created Personal Access Token (raw secret returned once)", body = crate::services::mcp::CreateMcpTokenResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn create_mcp_token_handler(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Json(req): Json<crate::services::mcp::CreateMcpTokenRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::services::mcp::CreateMcpTokenResponse>,
    ),
    AppError,
> {
    let mut store = state.privacy_store.write().await;
    let res = crate::services::mcp::create_mcp_token(
        &mut store,
        &auth.user.id.to_string(),
        auth.access(),
        req,
    )?;
    Ok((StatusCode::CREATED, Json(res)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/mcp/tokens/{id}",
    params(
        ("id" = String, Path, description = "Personal Access Token ID")
    ),
    responses(
        (status = 204, description = "Personal Access Token revoked"),
        (status = 404, description = "Token not found", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn revoke_mcp_token_handler(
    RequireAuth(auth): RequireAuth,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    crate::services::mcp::revoke_mcp_token(
        &mut store,
        id.trim(),
        &auth.user.id.to_string(),
        auth.access(),
    )?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn resolve_mcp_caller(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::services::mcp::McpCallerContext, AppError> {
    {
        let store = state.privacy_store.read().await;
        if !store.ai_config.mcp_enabled {
            return Err(AppError::Forbidden("MCP server is disabled".to_string()));
        }
    }

    // 1. Check Authorization: Bearer mcp_pat_...
    if let Some(auth_val) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        let trimmed = auth_val.trim();
        if let Some(raw_token) = trimmed
            .strip_prefix("Bearer ")
            .or_else(|| trimmed.strip_prefix("bearer "))
        {
            let raw_token = raw_token.trim();
            if raw_token.is_empty() {
                return Err(AppError::NotAuthenticated);
            }
            if raw_token.starts_with("mcp_pat_") {
                let hash = crate::services::mcp::hash_mcp_token(raw_token);
                let now_rfc3339 = Utc::now().to_rfc3339();

                let (user_id_str, scopes) = {
                    let mut store = state.privacy_store.write().await;
                    let pat = store
                        .personal_access_tokens
                        .iter_mut()
                        .find(|t| t.token_hash == hash && t.revoked_at.is_none())
                        .ok_or(AppError::NotAuthenticated)?;

                    if let Some(exp) = &pat.expires_at {
                        if exp.as_str() < now_rfc3339.as_str() {
                            return Err(AppError::NotAuthenticated);
                        }
                    }
                    pat.last_used_at = Some(now_rfc3339);
                    (pat.user_id.clone(), pat.scopes.clone())
                };

                let auth_store = state.auth_store.read().await;
                let user = auth_store
                    .users
                    .iter()
                    .find(|u| u.id.to_string() == user_id_str && u.is_active)
                    .ok_or(AppError::NotAuthenticated)?;
                let access = crate::services::rbac::CallerAccess::from_role_str(&user.role);

                return Ok(crate::services::mcp::McpCallerContext {
                    user_id: user.id.to_string(),
                    username: user.display_name(),
                    access,
                    scopes,
                });
            }
        }
    }

    // 2. Fallback to active session cookie or session Bearer token if present
    if let Some(token) = crate::services::rbac::extract_session_token(headers) {
        if let Ok((user, _session_id)) = AuthService::authenticate_session(state, &token).await {
            let access = crate::services::rbac::CallerAccess::from_role_str(&user.role);
            let scopes = crate::services::mcp::allowed_scopes_for_access(access)
                .into_iter()
                .map(ToString::to_string)
                .collect();
            return Ok(crate::services::mcp::McpCallerContext {
                user_id: user.id.to_string(),
                username: user.display_name(),
                access,
                scopes,
            });
        }
    }

    Err(AppError::NotAuthenticated)
}

#[utoipa::path(
    post,
    path = "/api/v1/mcp",
    request_body = crate::services::mcp::McpJsonRpcRequest,
    responses(
        (status = 200, description = "MCP JSON-RPC 2.0 response"),
        (status = 401, description = "Missing, expired, or revoked Bearer PAT or session", body = ErrorResponse),
        (status = 403, description = "Insufficient role or token scope for requested MCP tool", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn mcp_jsonrpc_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::services::mcp::McpJsonRpcRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let caller = resolve_mcp_caller(&state, &headers).await?;
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let mut store = state.privacy_store.write().await;

    let (status_u16, body) = crate::services::mcp::handle_mcp_jsonrpc(
        &mut store,
        &caller,
        req,
        current_year,
        settings.living_person_age_threshold as i32,
    );
    let status = StatusCode::from_u16(status_u16).unwrap_or(StatusCode::OK);
    Ok((status, Json(body)))
}

#[utoipa::path(
    get,
    path = "/api/v1/mcp/sse",
    responses(
        (status = 200, description = "MCP Server-Sent Events (SSE) endpoint discovery stream"),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn mcp_sse_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let _caller = resolve_mcp_caller(&state, &headers).await?;
    let sse_payload = "event: endpoint\ndata: /api/v1/mcp\n\n";
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(sse_payload))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    Ok(resp)
}
