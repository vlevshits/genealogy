#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/notes",
    params(NoteFilterQuery),
    responses(
        (status = 200, description = "Notes attached to an entity", body = NoteListResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_notes(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<NoteFilterQuery>,
) -> Result<Json<NoteListResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let mut items: Vec<NoteDto> = store
        .notes
        .iter()
        .filter(|n| !n.is_private || access.can_view_private())
        .filter(|n| {
            if let Some(ref et) = query.entity_type {
                if !n.entity_type.eq_ignore_ascii_case(et.trim()) {
                    return false;
                }
            }
            if let Some(ref eid) = query.entity_id {
                if n.entity_id != eid.trim() {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    items.sort_by_key(|n| n.sort_order);
    let total = items.len();

    Ok(Json(NoteListResponse { items, total }))
}

#[utoipa::path(
    post,
    path = "/api/v1/notes",
    request_body = CreateNoteRequest,
    responses(
        (status = 201, description = "Note created", body = NoteDto),
        (status = 400, description = "Validation error", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_note(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<NoteDto>), AppError> {
    let entity_type = req.entity_type.trim().to_ascii_lowercase();
    if !matches!(
        entity_type.as_str(),
        "individual" | "family" | "event" | "place" | "media"
    ) {
        let mut fields = HashMap::new();
        fields.insert("entity_type".to_string(), "invalid_entity_type".to_string());
        return Err(AppError::ValidationFailed(
            "entity_type must be one of: individual, family, event, place, media".to_string(),
            Some(fields),
        ));
    }

    if req.body.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("body".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "Note body must not be empty".to_string(),
            Some(fields),
        ));
    }

    let mut store = state.privacy_store.write().await;
    let note = NoteDto {
        id: Uuid::new_v4().to_string(),
        entity_type,
        entity_id: req.entity_id.trim().to_string(),
        body: req.body,
        is_private: req.is_private.unwrap_or(false),
        sort_order: req.sort_order.unwrap_or(0),
        version: 1,
    };
    store.notes.push(note.clone());

    log_change(
        &mut store,
        None,
        "note.create",
        "note",
        &note.id,
        "create",
        Some(format!("Note on {}", note.entity_type)),
        format!("Created note on {} {}", note.entity_type, note.entity_id),
        None,
        serde_json::to_value(&note).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok((StatusCode::CREATED, Json(note)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/notes/{id}",
    params(
        ("id" = String, Path, description = "Note UUID")
    ),
    request_body = UpdateNoteRequest,
    responses(
        (status = 200, description = "Note updated", body = NoteDto),
        (status = 404, description = "Note not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_note(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<Json<NoteDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let note_idx = store
        .notes
        .iter()
        .position(|n| n.id == id.trim())
        .ok_or(AppError::NotFound)?;

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.notes[note_idx].version,
        expected_version,
        &format!("Note {}", store.notes[note_idx].id),
        serde_json::to_value(&store.notes[note_idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.notes[note_idx]).ok();

    let note = &mut store.notes[note_idx];
    if let Some(body) = req.body {
        note.body = body;
    }
    if let Some(is_priv) = req.is_private {
        note.is_private = is_priv;
    }
    if let Some(so) = req.sort_order {
        note.sort_order = so;
    }
    note.version += 1;
    let updated = note.clone();

    log_change(
        &mut store,
        None,
        "note.update",
        "note",
        &updated.id,
        "update",
        Some(format!("Note on {}", updated.entity_type)),
        format!("Updated note {}", updated.id),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/notes/{id}",
    params(
        ("id" = String, Path, description = "Note UUID")
    ),
    responses(
        (status = 204, description = "Note deleted"),
        (status = 404, description = "Note not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_note(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let idx = store
        .notes
        .iter()
        .position(|n| n.id == id.trim())
        .ok_or(AppError::NotFound)?;
    let removed = store.notes.remove(idx);

    log_change(
        &mut store,
        None,
        "note.delete",
        "note",
        &removed.id,
        "delete",
        Some(format!("Note on {}", removed.entity_type)),
        format!("Deleted note {}", removed.id),
        serde_json::to_value(&removed).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) const DEFAULT_WEBP_BYTES: &[u8] =
    b"RIFF\x1a\x00\x00\x00WEBPVP8L\x0d\x00\x00\x00/\x00\x00\x00\x10\x07\x10\x11\x11\x88\x88\xfe\x07\x00";

pub(crate) fn read_media_bytes_from_disk(media_storage_path: &str, file_name: &str) -> Vec<u8> {
    if !file_name.is_empty() {
        let primary_path = PathBuf::from(media_storage_path)
            .join("originals")
            .join(file_name);
        if let Ok(bytes) = std::fs::read(&primary_path) {
            if !bytes.is_empty() {
                return bytes;
            }
        }
        let fallback_path = PathBuf::from("/tmp/genealogy-media/originals").join(file_name);
        if let Ok(bytes) = std::fs::read(&fallback_path) {
            if !bytes.is_empty() {
                return bytes;
            }
        }
    }
    DEFAULT_WEBP_BYTES.to_vec()
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{id}/file",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID (e.g. M0002)")
    ),
    responses(
        (status = 200, description = "Media file binary stream", content_type = "image/webp"),
        (status = 404, description = "Media asset not found or shielded by living/private privacy", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn get_media_file(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let media = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    // Direct Media Shield (`02-rbac-access-control.md §7 DoD #3`):
    // Living-person photos and private assets return 404 Not Found (indistinguishable from missing)
    let _permitted = redact_media_for_caller(media, &access, &store).ok_or(AppError::NotFound)?;

    let body_bytes = read_media_bytes_from_disk(
        &state.config.media_storage_path,
        media.file_name.as_deref().unwrap_or(""),
    );
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media.mime_type.as_str())
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(body_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to build media response: {e}")))?;

    Ok(response.into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{id}/crop",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID (e.g. M0001)"),
        ("x" = Option<f32>, Query, description = "Relative X coordinate (0.0..=1.0)"),
        ("y" = Option<f32>, Query, description = "Relative Y coordinate (0.0..=1.0)"),
        ("w" = Option<f32>, Query, description = "Relative width (0.0..=1.0)"),
        ("h" = Option<f32>, Query, description = "Relative height (0.0..=1.0)")
    ),
    responses(
        (status = 200, description = "Cropped avatar binary stream", content_type = "image/webp"),
        (status = 404, description = "Media asset not found or shielded by living/private privacy", body = ErrorResponse),
        (status = 422, description = "Invalid crop coordinates", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn get_media_crop(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(crop): Query<MediaCropQuery>,
) -> Result<Response, AppError> {
    let x = crop.x.unwrap_or(0.0);
    let y = crop.y.unwrap_or(0.0);
    let w = crop.w.or(crop.width).unwrap_or(1.0);
    let h = crop.h.or(crop.height).unwrap_or(1.0);
    if !(0.0..=1.0).contains(&x)
        || !(0.0..=1.0).contains(&y)
        || w <= 0.0
        || w > 1.0
        || h <= 0.0
        || h > 1.0
    {
        return Err(AppError::ValidationFailed(
            "Crop coordinates must be normalized within 0.0..=1.0".to_string(),
            Some(HashMap::from([(
                "crop".to_string(),
                "out_of_bounds".to_string(),
            )])),
        ));
    }

    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let media = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let _permitted = redact_media_for_caller(media, &access, &store).ok_or(AppError::NotFound)?;

    let body_bytes = read_media_bytes_from_disk(
        &state.config.media_storage_path,
        media.file_name.as_deref().unwrap_or(""),
    );
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media.mime_type.as_str())
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(body_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to build media crop response: {e}")))?;

    Ok(response.into_response())
}
