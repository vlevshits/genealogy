#[allow(unused_imports)]
use super::*;

// ============================================================================
// Milestone 3.2: AI Family Story Intake & Protected MCP Server Handlers
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoryDraftListResponse {
    pub items: Vec<crate::services::privacy::StoryIntakeDraftDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateAiSettingsRequest {
    #[serde(default)]
    pub llm_provider: Option<String>,
    #[serde(default)]
    pub llm_base_url: Option<String>,
    #[serde(default)]
    pub llm_model: Option<String>,
    #[serde(default)]
    pub llm_api_key: Option<String>,
    #[serde(default)]
    pub stt_provider: Option<String>,
    #[serde(default)]
    pub stt_model: Option<String>,
    #[serde(default)]
    pub mcp_enabled: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/transcribe",
    request_body = crate::services::story_intake::TranscribeStoryRequest,
    responses(
        (status = 200, description = "Audio transcribed into text narrative", body = crate::services::story_intake::TranscribeStoryResponse),
        (status = 400, description = "Unsupported audio format or invalid payload", body = ErrorResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn transcribe_ai_story_handler(
    RequireEditor(_editor): RequireEditor,
    headers: HeaderMap,
    req: axum::extract::Request,
) -> Result<Json<crate::services::story_intake::TranscribeStoryResponse>, AppError> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    if content_type.starts_with("multipart/form-data") {
        let mut mp = <Multipart as axum::extract::FromRequest<()>>::from_request(req, &())
            .await
            .map_err(|e| {
                AppError::ValidationFailed(format!("Invalid multipart payload: {e}"), None)
            })?;

        let mut filename: Option<String> = None;
        let mut lang: Option<String> = None;
        let mut transcript_hint: Option<String> = None;
        let mut raw_bytes: Option<Vec<u8>> = None;

        while let Ok(Some(field)) = mp.next_field().await {
            let name = field.name().unwrap_or("").to_string();
            if name == "audio" || name == "file" {
                if let Some(fname) = field.file_name() {
                    filename = Some(fname.to_string());
                }
                if let Ok(bytes) = field.bytes().await {
                    raw_bytes = Some(bytes.to_vec());
                }
            } else if name == "lang" || name == "language" {
                if let Ok(txt) = field.text().await {
                    lang = Some(txt);
                }
            } else if name == "transcript_hint" || name == "transcript" {
                if let Ok(txt) = field.text().await {
                    transcript_hint = Some(txt);
                }
            }
        }

        let transcribe_req = crate::services::story_intake::TranscribeStoryRequest {
            audio_base64: None,
            filename,
            transcript_hint,
            lang,
            duration_seconds: None,
        };
        let res = crate::services::story_intake::transcribe_story_audio(
            &transcribe_req,
            raw_bytes.as_deref(),
        )?;
        return Ok(Json(res));
    }

    let bytes = axum::body::to_bytes(req.into_body(), 15 * 1024 * 1024)
        .await
        .map_err(|_| AppError::ValidationFailed("Audio payload too large".to_string(), None))?;

    let payload: crate::services::story_intake::TranscribeStoryRequest = if bytes.is_empty() {
        crate::services::story_intake::TranscribeStoryRequest::default()
    } else {
        serde_json::from_slice(&bytes)
            .map_err(|e| AppError::ValidationFailed(format!("Invalid JSON payload: {e}"), None))?
    };

    let decoded_bytes = payload
        .audio_base64
        .as_deref()
        .and_then(|b64| BASE64_STANDARD.decode(b64.trim()).ok());

    let res =
        crate::services::story_intake::transcribe_story_audio(&payload, decoded_bytes.as_deref())?;
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/parse-story",
    request_body = crate::services::story_intake::ParseStoryRequest,
    responses(
        (status = 201, description = "Narrative parsed into sandbox story draft", body = crate::services::privacy::StoryIntakeDraftDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn parse_ai_story_handler(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<crate::services::story_intake::ParseStoryRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::services::privacy::StoryIntakeDraftDto>,
    ),
    AppError,
> {
    let mut store = state.privacy_store.write().await;
    let draft = crate::services::story_intake::parse_story_into_draft(
        &mut store,
        req,
        &editor.user.id.to_string(),
    )?;
    Ok((StatusCode::CREATED, Json(draft)))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/refine-draft",
    request_body = crate::services::story_intake::RefineDraftRequest,
    responses(
        (status = 200, description = "Story draft refined with follow-up clarification", body = crate::services::privacy::StoryIntakeDraftDto),
        (status = 404, description = "Draft not found", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn refine_ai_story_draft_handler(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<crate::services::story_intake::RefineDraftRequest>,
) -> Result<Json<crate::services::privacy::StoryIntakeDraftDto>, AppError> {
    let draft_id = req.draft_id.as_deref().unwrap_or("").trim();
    if draft_id.is_empty() {
        return Err(AppError::ValidationFailed(
            "draft_id is required".to_string(),
            Some(HashMap::from([(
                "draft_id".to_string(),
                "required".to_string(),
            )])),
        ));
    }
    let mut store = state.privacy_store.write().await;
    let updated = crate::services::story_intake::refine_story_draft(
        &mut store,
        draft_id,
        &req.clarification_text,
    )?;
    Ok(Json(updated))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/drafts/{id}/refine",
    params(
        ("id" = String, Path, description = "Story draft ID")
    ),
    request_body = crate::services::story_intake::RefineDraftRequest,
    responses(
        (status = 200, description = "Story draft refined with follow-up clarification", body = crate::services::privacy::StoryIntakeDraftDto),
        (status = 404, description = "Draft not found", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn refine_ai_story_draft_by_id_handler(
    RequireEditor(_editor): RequireEditor,
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<crate::services::story_intake::RefineDraftRequest>,
) -> Result<Json<crate::services::privacy::StoryIntakeDraftDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let updated = crate::services::story_intake::refine_story_draft(
        &mut store,
        id.trim(),
        &req.clarification_text,
    )?;
    Ok(Json(updated))
}

#[utoipa::path(
    get,
    path = "/api/v1/ai/drafts",
    responses(
        (status = 200, description = "List of story intake drafts", body = StoryDraftListResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn list_ai_story_drafts_handler(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
) -> Result<Json<StoryDraftListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let uid = editor.user.id.to_string();
    let is_admin = editor.access().is_admin();
    let items: Vec<_> = store
        .story_drafts
        .iter()
        .filter(|d| is_admin || d.created_by == uid)
        .cloned()
        .collect();
    let total = items.len();
    Ok(Json(StoryDraftListResponse { items, total }))
}

#[utoipa::path(
    get,
    path = "/api/v1/ai/drafts/{id}",
    params(
        ("id" = String, Path, description = "Story draft ID")
    ),
    responses(
        (status = 200, description = "Story intake draft detail", body = crate::services::privacy::StoryIntakeDraftDto),
        (status = 404, description = "Draft not found", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn get_ai_story_draft_handler(
    RequireEditor(_editor): RequireEditor,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::services::privacy::StoryIntakeDraftDto>, AppError> {
    let store = state.privacy_store.read().await;
    let draft = store
        .story_drafts
        .iter()
        .find(|d| d.id == id.trim())
        .cloned()
        .ok_or(AppError::NotFound)?;
    Ok(Json(draft))
}

#[utoipa::path(
    patch,
    path = "/api/v1/ai/drafts/{id}",
    params(
        ("id" = String, Path, description = "Story draft ID")
    ),
    request_body = crate::services::story_intake::UpdateStoryDraftRequest,
    responses(
        (status = 200, description = "Updated story intake draft", body = crate::services::privacy::StoryIntakeDraftDto),
        (status = 404, description = "Draft not found", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn update_ai_story_draft_handler(
    RequireEditor(_editor): RequireEditor,
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<crate::services::story_intake::UpdateStoryDraftRequest>,
) -> Result<Json<crate::services::privacy::StoryIntakeDraftDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let draft = crate::services::story_intake::update_story_draft(&mut store, id.trim(), req)?;
    Ok(Json(draft))
}

#[utoipa::path(
    delete,
    path = "/api/v1/ai/drafts/{id}",
    params(
        ("id" = String, Path, description = "Story draft ID")
    ),
    responses(
        (status = 204, description = "Story draft discarded"),
        (status = 404, description = "Draft not found", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn delete_ai_story_draft_handler(
    RequireEditor(_editor): RequireEditor,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let pos = store
        .story_drafts
        .iter()
        .position(|d| d.id == id.trim())
        .ok_or(AppError::NotFound)?;
    store.story_drafts.remove(pos);
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/ai/drafts/{id}/reconcile",
    params(
        ("id" = String, Path, description = "Story draft ID")
    ),
    responses(
        (status = 200, description = "Reconciled story draft with candidate matches", body = crate::services::privacy::StoryIntakeDraftDto),
        (status = 404, description = "Draft not found", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn reconcile_ai_story_draft_handler(
    RequireEditor(_editor): RequireEditor,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::services::privacy::StoryIntakeDraftDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let draft = crate::services::story_intake::reconcile_story_draft(&mut store, id.trim())?;
    Ok(Json(draft))
}

#[utoipa::path(
    post,
    path = "/api/v1/ai/drafts/{id}/commit",
    params(
        ("id" = String, Path, description = "Story draft ID")
    ),
    request_body = Option<crate::services::story_intake::CommitStoryDraftRequest>,
    responses(
        (status = 200, description = "Story draft atomically committed into live family tree", body = crate::services::story_intake::CommitStoryDraftResponse),
        (status = 404, description = "Draft not found", body = ErrorResponse),
        (status = 409, description = "Draft already committed", body = ErrorResponse)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn commit_ai_story_draft_handler(
    RequireEditor(editor): RequireEditor,
    Path(id): Path<String>,
    State(state): State<AppState>,
    body: Option<Json<crate::services::story_intake::CommitStoryDraftRequest>>,
) -> Result<Json<crate::services::story_intake::CommitStoryDraftResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let req = body.map(|Json(r)| r).unwrap_or_default();
    let mut store = state.privacy_store.write().await;
    let res = crate::services::story_intake::commit_story_draft(
        &mut store,
        id.trim(),
        req,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
        current_year,
        settings.living_person_age_threshold as i32,
    )?;
    Ok(Json(res))
}

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

#[utoipa::path(
    get,
    path = "/api/v1/admin/ai-settings",
    responses(
        (status = 200, description = "AI & MCP instance configuration", body = crate::services::privacy::AiInstanceConfigDto)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn get_admin_ai_settings_handler(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<crate::services::privacy::AiInstanceConfigDto>, AppError> {
    let store = state.privacy_store.read().await;
    Ok(Json(store.ai_config.clone()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/admin/ai-settings",
    request_body = UpdateAiSettingsRequest,
    responses(
        (status = 200, description = "Updated AI & MCP instance configuration", body = crate::services::privacy::AiInstanceConfigDto)
    ),
    tag = "AI Story Intake & MCP"
)]
pub async fn update_admin_ai_settings_handler(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<UpdateAiSettingsRequest>,
) -> Result<Json<crate::services::privacy::AiInstanceConfigDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    if let Some(p) = req.llm_provider {
        store.ai_config.llm_provider = p.trim().to_string();
    }
    if let Some(url) = req.llm_base_url {
        store.ai_config.llm_base_url = url.trim().to_string();
    }
    if let Some(m) = req.llm_model {
        store.ai_config.llm_model = m.trim().to_string();
    }
    if let Some(key) = req.llm_api_key {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            store.ai_config.llm_api_key = None;
            store.ai_config.has_api_key = false;
        } else {
            store.ai_config.llm_api_key = Some(trimmed.to_string());
            store.ai_config.has_api_key = true;
        }
    }
    if let Some(sp) = req.stt_provider {
        store.ai_config.stt_provider = sp.trim().to_string();
    }
    if let Some(sm) = req.stt_model {
        store.ai_config.stt_model = sm.trim().to_string();
    }
    if let Some(enabled) = req.mcp_enabled {
        store.ai_config.mcp_enabled = enabled;
    }
    Ok(Json(store.ai_config.clone()))
}
