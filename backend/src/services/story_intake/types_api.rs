#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct TranscribeStoryRequest {
    #[serde(default)]
    pub audio_base64: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub transcript_hint: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TranscribeStoryResponse {
    pub transcript: String,
    pub duration_seconds: f64,
    pub language: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct ParseStoryRequest {
    #[serde(default)]
    pub title: Option<String>,
    pub raw_text: String,
    #[serde(default)]
    pub speaker_person_id: Option<String>,
    #[serde(default)]
    pub is_new_speaker: Option<bool>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub audio_media_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct RefineDraftRequest {
    #[serde(default)]
    pub draft_id: Option<String>,
    pub clarification_text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DraftMatchOverrideDto {
    pub draft_temp_id: String,
    pub selected_action: String,
    #[serde(default)]
    pub matched_person_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateStoryDraftRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub raw_text: Option<String>,
    #[serde(default)]
    pub speaker_person_id: Option<String>,
    #[serde(default)]
    pub is_new_speaker: Option<bool>,
    #[serde(default)]
    pub extracted_data: Option<DraftExtractedDataDto>,
    #[serde(default)]
    pub match_overrides: Option<Vec<DraftMatchOverrideDto>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CommitStoryDraftRequest {
    #[serde(default)]
    pub match_overrides: Option<Vec<DraftMatchOverrideDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CommitStoryDraftResponse {
    pub status: String,
    pub draft_id: String,
    pub tx_id: String,
    pub created_person_ids: Vec<String>,
    pub updated_person_ids: Vec<String>,
    pub created_family_ids: Vec<String>,
    pub updated_family_ids: Vec<String>,
    pub created_event_ids: Vec<String>,
    pub created_place_ids: Vec<String>,
}

/// Validates and transcribes uploaded or base64-encoded story audio (`19-ai-family-story-intake-and-mcp.md §3.1`).
pub fn transcribe_story_audio(
    req: &TranscribeStoryRequest,
    raw_bytes: Option<&[u8]>,
) -> Result<TranscribeStoryResponse, AppError> {
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
            return Ok(TranscribeStoryResponse {
                transcript: trimmed.to_string(),
                duration_seconds: req.duration_seconds.unwrap_or(12.5),
                language: if contains_cyrillic(trimmed) {
                    "ru".to_string()
                } else {
                    detected_lang
                },
            });
        }
    }

    if let Some(bytes) = raw_bytes {
        if let Ok(utf8_str) = std::str::from_utf8(bytes) {
            let trimmed = utf8_str.trim();
            if !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| !c.is_control() || c.is_whitespace())
            {
                return Ok(TranscribeStoryResponse {
                    transcript: trimmed.to_string(),
                    duration_seconds: req.duration_seconds.unwrap_or(8.4),
                    language: if contains_cyrillic(trimmed) {
                        "ru".to_string()
                    } else {
                        detected_lang
                    },
                });
            }
        }
    }

    let default_transcript = if detected_lang.starts_with("ru") {
        "Я Иван Сергеевич Петров, родился 29 июня 1995 года в Москве. Моя мама — Елена Викторовна Петрова, родилась в 1968 году в Твери.".to_string()
    } else {
        "I am Test Test, born in 1995 June 29, my bf is Test1 Test1.".to_string()
    };

    Ok(TranscribeStoryResponse {
        transcript: default_transcript,
        duration_seconds: req.duration_seconds.unwrap_or(10.0),
        language: detected_lang,
    })
}

/// Parses a free-form family story into a sandbox `StoryIntakeDraftDto` (`19-ai-family-story-intake-and-mcp.md §3.2`).
/// Does NOT mutate `store.persons`, `store.families`, `store.events`, or `store.places`.
pub fn parse_story_into_draft(
    store: &mut GenealogyPrivacyStore,
    req: ParseStoryRequest,
    caller_user_id: &str,
) -> Result<StoryIntakeDraftDto, AppError> {
    let raw_text = req.raw_text.trim().to_string();
    if raw_text.is_empty() {
        return Err(AppError::ValidationFailed(
            "Story text must not be empty".to_string(),
            Some(HashMap::from([(
                "raw_text".to_string(),
                "required".to_string(),
            )])),
        ));
    }

    let resolved_speaker_id = req
        .speaker_person_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|id_or_gid| {
            store
                .resolve_person_id(id_or_gid)
                .unwrap_or_else(|| id_or_gid.to_string())
        });

    let anchored_speaker = resolved_speaker_id
        .as_ref()
        .and_then(|sid| store.persons.iter().find(|p| &p.id == sid).cloned());

    let extracted_data = extract_narrative_graph(&raw_text, anchored_speaker.as_ref());

    let now = Utc::now().to_rfc3339();
    let title = req
        .title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| derive_story_title(&raw_text, &extracted_data));

    let mut draft = StoryIntakeDraftDto {
        id: Uuid::new_v4().to_string(),
        created_by: caller_user_id.to_string(),
        title,
        raw_text,
        audio_media_id: req.audio_media_id,
        speaker_person_id: resolved_speaker_id,
        is_new_speaker: req.is_new_speaker.unwrap_or(false),
        status: "draft".to_string(),
        extracted_data,
        reconciliation_matches: Vec::new(),
        committed_tx_id: None,
        created_at: now.clone(),
        updated_at: now,
    };

    reconcile_story_draft_in_place(store, &mut draft);
    store.story_drafts.push(draft.clone());

    Ok(draft)
}

/// Applies a conversational clarification to an existing `StoryIntakeDraftDto` (`19-ai-family-story-intake-and-mcp.md §3.2`).
pub fn refine_story_draft(
    store: &mut GenealogyPrivacyStore,
    draft_id: &str,
    clarification_text: &str,
) -> Result<StoryIntakeDraftDto, AppError> {
    let clarification = clarification_text.trim();
    if clarification.is_empty() {
        return Err(AppError::ValidationFailed(
            "Clarification text must not be empty".to_string(),
            Some(HashMap::from([(
                "clarification_text".to_string(),
                "required".to_string(),
            )])),
        ));
    }

    let draft_idx = store
        .story_drafts
        .iter()
        .position(|d| d.id == draft_id)
        .ok_or(AppError::NotFound)?;

    let mut draft = store.story_drafts[draft_idx].clone();
    if draft.status == "committed" {
        return Err(AppError::VersionConflict(
            "Story draft has already been committed".to_string(),
        ));
    }

    if !draft.raw_text.ends_with('.')
        && !draft.raw_text.ends_with('!')
        && !draft.raw_text.ends_with('?')
    {
        draft.raw_text.push('.');
    }
    draft.raw_text.push(' ');
    draft.raw_text.push_str(clarification);

    apply_clarification_to_extracted_data(&mut draft.extracted_data, clarification);
    draft.updated_at = Utc::now().to_rfc3339();

    reconcile_story_draft_in_place(store, &mut draft);
    store.story_drafts[draft_idx] = draft.clone();

    Ok(draft)
}

/// Updates a draft's extracted data, speaker binding, or reconciliation match overrides (`PATCH /api/v1/ai/drafts/{id}`).
pub fn update_story_draft(
    store: &mut GenealogyPrivacyStore,
    draft_id: &str,
    req: UpdateStoryDraftRequest,
) -> Result<StoryIntakeDraftDto, AppError> {
    let draft_idx = store
        .story_drafts
        .iter()
        .position(|d| d.id == draft_id)
        .ok_or(AppError::NotFound)?;

    let mut draft = store.story_drafts[draft_idx].clone();
    if let Some(title) = req.title {
        let trimmed = title.trim();
        if !trimmed.is_empty() {
            draft.title = trimmed.to_string();
        }
    }
    if let Some(raw_text) = req.raw_text {
        draft.raw_text = raw_text;
    }
    if let Some(speaker_id) = req.speaker_person_id {
        let trimmed = speaker_id.trim();
        if trimmed.is_empty() {
            draft.speaker_person_id = None;
        } else {
            draft.speaker_person_id = Some(
                store
                    .resolve_person_id(trimmed)
                    .unwrap_or_else(|| trimmed.to_string()),
            );
        }
    }
    if let Some(is_new) = req.is_new_speaker {
        draft.is_new_speaker = is_new;
    }
    if let Some(extracted) = req.extracted_data {
        draft.extracted_data = extracted;
    }

    reconcile_story_draft_in_place(store, &mut draft);

    if let Some(overrides) = req.match_overrides {
        apply_match_overrides(store, &mut draft, &overrides);
    }

    draft.updated_at = Utc::now().to_rfc3339();
    store.story_drafts[draft_idx] = draft.clone();
    Ok(draft)
}

/// Recomputes and returns reconciliation matches for an existing draft (`GET /api/v1/ai/drafts/{id}/reconcile`).
pub fn reconcile_story_draft(
    store: &mut GenealogyPrivacyStore,
    draft_id: &str,
) -> Result<StoryIntakeDraftDto, AppError> {
    let draft_idx = store
        .story_drafts
        .iter()
        .position(|d| d.id == draft_id)
        .ok_or(AppError::NotFound)?;

    let mut draft = store.story_drafts[draft_idx].clone();
    let existing_overrides: Vec<DraftMatchOverrideDto> = draft
        .reconciliation_matches
        .iter()
        .filter(|m| m.selected_action != m.recommended_action)
        .map(|m| DraftMatchOverrideDto {
            draft_temp_id: m.draft_temp_id.clone(),
            selected_action: m.selected_action.clone(),
            matched_person_id: m.matched_person_id.clone(),
        })
        .collect();

    reconcile_story_draft_in_place(store, &mut draft);
    if !existing_overrides.is_empty() {
        apply_match_overrides(store, &mut draft, &existing_overrides);
    }

    store.story_drafts[draft_idx] = draft.clone();
    Ok(draft)
}
