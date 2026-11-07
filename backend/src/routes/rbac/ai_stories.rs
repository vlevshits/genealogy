#[allow(unused_imports)]
use super::*;

// ============================================================================
// Milestone 3.2: AI Family Story Intake & AI Settings Handlers
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

async fn process_story_transcription(
    state: &AppState,
    req: &crate::services::story_intake::TranscribeStoryRequest,
    audio_bytes: Option<&[u8]>,
) -> Result<crate::services::story_intake::TranscribeStoryResponse, AppError> {
    if let Some(filename) = &req.filename {
        let lower = filename.trim().to_ascii_lowercase();
        if !lower.is_empty()
            && !lower.ends_with(".webm")
            && !lower.ends_with(".mp3")
            && !lower.ends_with(".wav")
            && !lower.ends_with(".m4a")
            && !lower.ends_with(".ogg")
        {
            return Err(AppError::ValidationFailed(
                "Unsupported audio format; expected .webm, .mp3, .wav, .m4a, or .ogg".to_string(),
                Some(HashMap::from([(
                    "filename".to_string(),
                    "unsupported_audio_format".to_string(),
                )])),
            ));
        }
    }

    let detected_lang = req
        .lang
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("en")
        .to_string();

    if let Some(hint) = &req.transcript_hint {
        let trimmed = hint.trim();
        if !trimmed.is_empty() {
            return Ok(crate::services::story_intake::TranscribeStoryResponse {
                transcript: trimmed.to_string(),
                text: trimmed.to_string(),
                duration_seconds: req.duration_seconds.unwrap_or(12.5),
                language: if crate::services::story_intake::contains_cyrillic(trimmed) {
                    "ru".to_string()
                } else {
                    detected_lang
                },
            });
        }
    }

    if let Some(bytes) = audio_bytes {
        if let Ok(utf8_str) = std::str::from_utf8(bytes) {
            let trimmed = utf8_str.trim();
            if !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| !c.is_control() || c.is_whitespace())
            {
                return Ok(crate::services::story_intake::TranscribeStoryResponse {
                    transcript: trimmed.to_string(),
                    text: trimmed.to_string(),
                    duration_seconds: req.duration_seconds.unwrap_or(8.4),
                    language: if crate::services::story_intake::contains_cyrillic(trimmed) {
                        "ru".to_string()
                    } else {
                        detected_lang
                    },
                });
            }
        }
    }

    let gemini_key = {
        let key_from_config = state
            .config
            .gemini_api_key
            .as_deref()
            .filter(|k| !k.trim().is_empty());
        if key_from_config.is_some() {
            key_from_config.map(ToString::to_string)
        } else {
            let store = state.privacy_store.read().await;
            store
                .ai_config
                .llm_api_key
                .clone()
                .filter(|k| !k.trim().is_empty())
        }
    };

    if let Some(bytes) = audio_bytes {
        if let Some(key) = gemini_key {
            let mime_type = match req.filename.as_deref() {
                Some(f) if f.to_ascii_lowercase().ends_with(".mp3") => "audio/mp3",
                Some(f) if f.to_ascii_lowercase().ends_with(".wav") => "audio/wav",
                Some(f) if f.to_ascii_lowercase().ends_with(".m4a") => "audio/mp4",
                Some(f) if f.to_ascii_lowercase().ends_with(".ogg") => "audio/ogg",
                _ => "audio/webm",
            };

            let transcript = crate::services::story_intake::transcribe_audio_with_gemini(
                &key,
                &state.config.gemini_model,
                bytes,
                mime_type,
                req.lang.as_deref(),
            )
            .await?;

            return Ok(crate::services::story_intake::TranscribeStoryResponse {
                transcript: transcript.clone(),
                text: transcript.clone(),
                duration_seconds: req.duration_seconds.unwrap_or(15.0),
                language: if crate::services::story_intake::contains_cyrillic(&transcript) {
                    "ru".to_string()
                } else {
                    detected_lang
                },
            });
        }
    }

    if audio_bytes.is_some() {
        return Err(AppError::ValidationFailed(
            if detected_lang.starts_with("ru") {
                "Серверное распознавание аудиофайлов не настроено (в .env не задан GEMINI_API_KEY). Пожалуйста, запишите рассказ через микрофон в браузере (Chrome / Safari / Edge) или введите текст рассказа."
            } else {
                "Server-side audio transcription is not configured (GEMINI_API_KEY is not set in .env). Please record via microphone in a supported browser (Chrome, Safari, Edge) or enter the story as text."
            }
            .to_string(),
            Some(HashMap::from([(
                "audio".to_string(),
                "gemini_api_key_not_configured".to_string(),
            )])),
        ));
    }

    Err(AppError::ValidationFailed(
        "No audio data or transcript hint provided".to_string(),
        Some(HashMap::from([(
            "audio".to_string(),
            "required".to_string(),
        )])),
    ))
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
    State(state): State<AppState>,
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
        let res =
            process_story_transcription(&state, &transcribe_req, raw_bytes.as_deref()).await?;
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

    let res = process_story_transcription(&state, &payload, decoded_bytes.as_deref()).await?;
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
    Json(mut req): Json<crate::services::story_intake::ParseStoryRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::services::privacy::StoryIntakeDraftDto>,
    ),
    AppError,
> {
    if req.raw_text.trim().is_empty() {
        return Err(AppError::ValidationFailed(
            "Story text must not be empty".to_string(),
            Some(HashMap::from([(
                "raw_text".to_string(),
                "required".to_string(),
            )])),
        ));
    }

    if req.extracted_data.is_none() && req.engine.as_deref() != Some("builtin") {
        let gemini_key = {
            let key_from_config = state
                .config
                .gemini_api_key
                .as_deref()
                .filter(|k| !k.trim().is_empty());
            if key_from_config.is_some() {
                key_from_config.map(ToString::to_string)
            } else {
                let store = state.privacy_store.read().await;
                store
                    .ai_config
                    .llm_api_key
                    .clone()
                    .filter(|k| !k.trim().is_empty())
            }
        };

        if let Some(key) = gemini_key {
            let speaker_context = {
                let store = state.privacy_store.read().await;
                let resolved_id = req
                    .speaker_person_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .and_then(|id_or_gid| store.resolve_person_id(id_or_gid));
                resolved_id
                    .as_ref()
                    .and_then(|sid| store.persons.iter().find(|p| &p.id == sid))
                    .map(|sp| crate::services::privacy::DraftPersonDto {
                        temp_id: "draft-p-speaker".to_string(),
                        given_name: sp.primary_name.given_name.clone(),
                        patronymic: sp.primary_name.patronymic.clone(),
                        surname: sp.primary_name.surname.clone(),
                        maiden_name: sp.birth_surname.clone(),
                        sex: sp.sex.clone(),
                        relation_to_speaker: "self".to_string(),
                        birth_date_text: sp.birth.as_ref().and_then(|b| b.date.clone()),
                        birth_year: crate::services::story_intake::extract_person_birth_year(sp),
                        birth_place: sp.birth.as_ref().and_then(|b| b.place.clone()),
                        death_date_text: sp.death.as_ref().and_then(|d| d.date.clone()),
                        death_year: None,
                        death_place: sp.death.as_ref().and_then(|d| d.place.clone()),
                        is_living: sp.is_living,
                        occupation: None,
                        biography_notes: None,
                        confidence: 98,
                        grounding_quote: None,
                    })
            };

            match crate::services::story_intake::extract_story_with_gemini(
                &key,
                &state.config.gemini_model,
                &req.raw_text,
                speaker_context.as_ref(),
            )
            .await
            {
                Ok(gemini_extracted) => {
                    req.extracted_data = Some(gemini_extracted);
                }
                Err(err) => {
                    tracing::warn!(
                        "Gemini extraction failed, falling back to built-in NLP engine: {err}"
                    );
                }
            }
        }
    }

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
    post,
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
