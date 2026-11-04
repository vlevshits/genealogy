#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/media/bundle",
    params(
        MediaBundleQuery
    ),
    responses(
        (status = 200, description = "Media library archive bundle ingested and reconciled against stubs", body = MediaBundleReconciliationReport),
        (status = 400, description = "Invalid archive, zip slip traversal or decompression limit exceeded", body = ErrorResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 413, description = "Payload too large (> 500MB)", body = ErrorResponse),
        (status = 415, description = "Unsupported archive format (not a ZIP archive)", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn upload_media_bundle(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Query(query): Query<MediaBundleQuery>,
    multipart: Multipart,
) -> Result<Json<MediaBundleReconciliationReport>, AppError> {
    let ingest_unmatched = query.ingest_unmatched.unwrap_or(true);
    let temp_file = spool_multipart_to_temp_file(multipart).await?;
    let mut store = state.privacy_store.write().await;
    let report = process_media_bundle_archive(
        temp_file,
        &mut store,
        &state.config.media_storage_path,
        ingest_unmatched,
    )?;

    if report.reconciled_stubs > 0 || report.newly_created > 0 {
        log_change(
            &mut store,
            None,
            "media.bundle",
            "media",
            "bundle",
            "update",
            Some("Media Library Bundle".to_string()),
            format!(
                "Reconciled {} stubs and ingested {} new media assets from media bundle",
                report.reconciled_stubs, report.newly_created
            ),
            None,
            None,
            Some(editor.user.id.to_string()),
            Some(editor.user.display_name()),
        );
    }

    Ok(Json(report))
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID (e.g. M0001)")
    ),
    responses(
        (status = 200, description = "Media asset metadata, face tags, and entity links", body = MediaAssetSummaryDto),
        (status = 404, description = "Media asset not found or shielded by privacy", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn get_media_detail(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MediaAssetSummaryDto>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let media = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let redacted = redact_media_for_caller(media, &access, &store).ok_or(AppError::NotFound)?;
    Ok(Json(redacted))
}

#[utoipa::path(
    patch,
    path = "/api/v1/media/{id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    request_body = UpdateMediaRequest,
    responses(
        (status = 200, description = "Media asset metadata updated", body = MediaAssetSummaryDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset not found", body = ErrorResponse),
        (status = 409, description = "Optimistic concurrency version conflict", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn update_media(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateMediaRequest>,
) -> Result<Json<MediaAssetSummaryDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let resolved_place = if let Some(ref pid_raw) = req.place_id {
        if pid_raw.trim().is_empty() {
            Some((None, None))
        } else {
            let pid =
                resolve_place_id_in_store(&store.places, pid_raw).ok_or(AppError::NotFound)?;
            let fname = store
                .places
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.full_name.clone());
            Some((Some(pid), fname))
        }
    } else if let Some(ref ppath) = req.place_path {
        if ppath.trim().is_empty() {
            Some((None, None))
        } else {
            let place = resolve_or_create_place_chain(&mut store, ppath, None, None, None, None)?;
            Some((Some(place.id), Some(place.full_name)))
        }
    } else {
        None
    };

    let (updated, before_json) = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;

        let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
        verify_optimistic_version(
            media.version,
            expected_version,
            &format!("Media {}", media.gid),
            serde_json::to_value(&*media).unwrap_or_default(),
        )?;

        let before_snapshot = serde_json::to_value(&*media).ok();

        if let Some(ref title) = req.title {
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                media.title = trimmed.to_string();
            }
        }

        if let Some(desc) = req.description.or(req.caption) {
            let trimmed = desc.trim();
            media.description = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
        }

        if let Some(ref dt_raw) = req.date_taken {
            let trimmed = dt_raw.trim();
            if trimmed.is_empty() {
                media.date_taken = None;
                media.date_sort = None;
            } else {
                let parsed = GenDate::parse(trimmed);
                media.date_sort = parsed.sort_key();
                media.date_taken = Some(trimmed.to_string());
            }
        }

        if let Some((pid, fname)) = resolved_place {
            media.place_id = pid;
            media.place_full_name = fname;
        }

        if let Some(is_priv) = req.is_private {
            media.is_private = is_priv;
        }

        media.version += 1;
        (media.clone(), before_snapshot)
    };

    log_change(
        &mut store,
        None,
        "media.update",
        "media",
        &updated.id,
        "update",
        Some(updated.title.clone()),
        format!("Updated media {} ({})", updated.gid, updated.title),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    sync_media_links_and_living(&mut store, &media_id);
    let final_media = store
        .media
        .iter()
        .find(|m| m.id == media_id)
        .cloned()
        .unwrap_or(updated);

    Ok(Json(final_media))
}

#[utoipa::path(
    delete,
    path = "/api/v1/media/{id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    responses(
        (status = 204, description = "Media asset deleted"),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset not found", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn delete_media(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_idx = store
        .media
        .iter()
        .position(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed = store.media.remove(media_idx);

    for ev in &mut store.events {
        ev.media_ids.retain(|mid| mid != &removed.id);
    }

    for person in &mut store.persons {
        if person.avatar_media_id.as_deref() == Some(removed.id.as_str()) {
            person.avatar_media_id = None;
            person.avatar_crop = None;
            person.avatar_url = None;
        }
    }

    log_change(
        &mut store,
        None,
        "media.delete",
        "media",
        &removed.id,
        "delete",
        Some(removed.title.clone()),
        format!("Deleted media {} ({})", removed.gid, removed.title),
        serde_json::to_value(&removed).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/media/{id}/tags",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    request_body = CreateFaceTagRequest,
    responses(
        (status = 201, description = "Face tag created on media asset", body = FaceTagDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or person not found", body = ErrorResponse),
        (status = 422, description = "Invalid normalized bounding box coordinates", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn create_face_tag(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateFaceTagRequest>,
) -> Result<(StatusCode, Json<FaceTagDto>), AppError> {
    let w = req.w.or(req.width).unwrap_or(0.0);
    let h = req.h.or(req.height).unwrap_or(0.0);
    if !(0.0..=1.0).contains(&req.x)
        || !(0.0..=1.0).contains(&req.y)
        || w <= 0.0
        || w > 1.0
        || h <= 0.0
        || h > 1.0
    {
        return Err(AppError::ValidationFailed(
            "Face tag coordinates (x, y, w, h) must be normalized within 0.0..=1.0".to_string(),
            Some(HashMap::from([(
                "coordinates".to_string(),
                "out_of_bounds".to_string(),
            )])),
        ));
    }

    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let person_id = resolve_person_id_in_store(&store.persons, req.person_id.trim())
        .ok_or(AppError::NotFound)?;
    let (person_name, is_living) = store
        .persons
        .iter()
        .find(|p| p.id == person_id)
        .map(|p| (format_person_display_name(p), p.is_living))
        .ok_or(AppError::NotFound)?;

    let tag_id = Uuid::new_v4().to_string();
    let is_primary = req.is_primary_avatar.unwrap_or(false);

    let new_tag = FaceTagDto {
        id: tag_id.clone(),
        person_id: person_id.clone(),
        person_name,
        is_living,
        x: req.x,
        y: req.y,
        width: w,
        height: h,
        is_primary_avatar: is_primary,
    };

    if let Some(media) = store.media.iter_mut().find(|m| m.id == media_id) {
        media.face_tags.push(new_tag);
        media.version += 1;
    }

    if is_primary {
        apply_primary_avatar_for_person(
            &mut store, &person_id, &media_id, &tag_id, req.x, req.y, w, h,
        );
    }

    sync_media_links_and_living(&mut store, &media_id);

    let created_tag = store
        .media
        .iter()
        .find(|m| m.id == media_id)
        .and_then(|m| m.face_tags.iter().find(|t| t.id == tag_id).cloned())
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::CREATED, Json(created_tag)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/media/{id}/tags/{tag_id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID"),
        ("tag_id" = String, Path, description = "Face tag ID")
    ),
    request_body = UpdateFaceTagRequest,
    responses(
        (status = 200, description = "Face tag updated", body = FaceTagDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset, face tag, or person not found", body = ErrorResponse),
        (status = 422, description = "Invalid normalized bounding box coordinates", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn update_face_tag(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, tag_id)): Path<(String, String)>,
    Json(req): Json<UpdateFaceTagRequest>,
) -> Result<Json<FaceTagDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let target_tag = tag_id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let new_person_info = if let Some(ref raw_pid) = req.person_id {
        let pid =
            resolve_person_id_in_store(&store.persons, raw_pid.trim()).ok_or(AppError::NotFound)?;
        let (pname, living) = store
            .persons
            .iter()
            .find(|p| p.id == pid)
            .map(|p| (format_person_display_name(p), p.is_living))
            .ok_or(AppError::NotFound)?;
        Some((pid, pname, living))
    } else {
        None
    };

    let (person_id, x, y, w, h, was_primary, now_primary) = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;
        let tag = media
            .face_tags
            .iter_mut()
            .find(|t| t.id == target_tag)
            .ok_or(AppError::NotFound)?;

        let next_x = req.x.unwrap_or(tag.x);
        let next_y = req.y.unwrap_or(tag.y);
        let next_w = req.w.or(req.width).unwrap_or(tag.width);
        let next_h = req.h.or(req.height).unwrap_or(tag.height);
        if !(0.0..=1.0).contains(&next_x)
            || !(0.0..=1.0).contains(&next_y)
            || next_w <= 0.0
            || next_w > 1.0
            || next_h <= 0.0
            || next_h > 1.0
        {
            return Err(AppError::ValidationFailed(
                "Face tag coordinates (x, y, w, h) must be normalized within 0.0..=1.0".to_string(),
                Some(HashMap::from([(
                    "coordinates".to_string(),
                    "out_of_bounds".to_string(),
                )])),
            ));
        }

        if let Some((pid, pname, living)) = new_person_info {
            tag.person_id = pid;
            tag.person_name = pname;
            tag.is_living = living;
        }
        tag.x = next_x;
        tag.y = next_y;
        tag.width = next_w;
        tag.height = next_h;

        let was_primary = tag.is_primary_avatar;
        let now_primary = req.is_primary_avatar.unwrap_or(was_primary);
        tag.is_primary_avatar = now_primary;
        media.version += 1;

        (
            tag.person_id.clone(),
            tag.x,
            tag.y,
            tag.width,
            tag.height,
            was_primary,
            now_primary,
        )
    };

    if now_primary {
        apply_primary_avatar_for_person(&mut store, &person_id, &media_id, target_tag, x, y, w, h);
    } else if was_primary && !now_primary {
        clear_primary_avatar_for_person(&mut store, &person_id);
    }

    sync_media_links_and_living(&mut store, &media_id);

    let updated = store
        .media
        .iter()
        .find(|m| m.id == media_id)
        .and_then(|m| m.face_tags.iter().find(|t| t.id == target_tag).cloned())
        .ok_or(AppError::NotFound)?;

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/media/{id}/tags/{tag_id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID"),
        ("tag_id" = String, Path, description = "Face tag ID")
    ),
    responses(
        (status = 204, description = "Face tag deleted"),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or face tag not found", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn delete_face_tag(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, tag_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let target_tag = tag_id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let removed_tag = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;
        let idx = media
            .face_tags
            .iter()
            .position(|t| t.id == target_tag)
            .ok_or(AppError::NotFound)?;
        let removed = media.face_tags.remove(idx);

        if !media
            .links
            .iter()
            .any(|l| l.entity_type == "individual" && l.entity_id == removed.person_id)
        {
            media.links.push(MediaLinkDto {
                id: Uuid::new_v4().to_string(),
                media_id: media.id.clone(),
                entity_type: "individual".to_string(),
                entity_id: removed.person_id.clone(),
                role: Some("depicted".to_string()),
            });
        }
        if !media.linked_person_ids.contains(&removed.person_id) {
            media.linked_person_ids.push(removed.person_id.clone());
        }

        media.version += 1;
        removed
    };

    if removed_tag.is_primary_avatar {
        clear_primary_avatar_for_person(&mut store, &removed_tag.person_id);
    }

    sync_media_links_and_living(&mut store, &media_id);

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/media/{id}/links",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    request_body = CreateMediaLinkRequest,
    responses(
        (status = 201, description = "Entity linked to media asset", body = MediaLinkDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or target entity not found", body = ErrorResponse),
        (status = 422, description = "Invalid entity_type", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn create_media_link(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateMediaLinkRequest>,
) -> Result<(StatusCode, Json<MediaLinkDto>), AppError> {
    let entity_type = req.entity_type.trim().to_lowercase();
    if !matches!(entity_type.as_str(), "individual" | "family" | "event") {
        return Err(AppError::ValidationFailed(
            "entity_type must be one of: individual, family, event".to_string(),
            Some(HashMap::from([(
                "entity_type".to_string(),
                "invalid_entity_type".to_string(),
            )])),
        ));
    }

    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let raw_eid = req.entity_id.trim();
    let resolved_entity_id = match entity_type.as_str() {
        "individual" => {
            resolve_person_id_in_store(&store.persons, raw_eid).ok_or(AppError::NotFound)?
        }
        "family" => store
            .families
            .iter()
            .find(|f| f.id == raw_eid || f.gid.eq_ignore_ascii_case(raw_eid))
            .map(|f| f.id.clone())
            .ok_or(AppError::NotFound)?,
        "event" => store
            .events
            .iter()
            .find(|e| e.id == raw_eid || e.gid.eq_ignore_ascii_case(raw_eid))
            .map(|e| e.id.clone())
            .ok_or(AppError::NotFound)?,
        _ => unreachable!(),
    };

    let role = req
        .role
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("gallery")
        .to_string();

    let link = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;

        if let Some(existing) = media
            .links
            .iter()
            .find(|l| l.entity_type == entity_type && l.entity_id == resolved_entity_id)
        {
            existing.clone()
        } else {
            let new_link = MediaLinkDto {
                id: Uuid::new_v4().to_string(),
                media_id: media_id.clone(),
                entity_type: entity_type.clone(),
                entity_id: resolved_entity_id.clone(),
                role: Some(role),
            };
            media.links.push(new_link.clone());
            media.version += 1;
            new_link
        }
    };

    if entity_type == "event" {
        if let Some(ev) = store.events.iter_mut().find(|e| e.id == resolved_entity_id) {
            if !ev.media_ids.contains(&media_id) {
                ev.media_ids.push(media_id.clone());
            }
        }
    }

    sync_media_links_and_living(&mut store, &media_id);

    Ok((StatusCode::CREATED, Json(link)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/media/{id}/links/{link_id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID"),
        ("link_id" = String, Path, description = "Media link ID or linked entity ID")
    ),
    responses(
        (status = 204, description = "Media link removed"),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or link not found", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn delete_media_link(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, link_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let target_link = link_id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let removed_link = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;
        let idx = media
            .links
            .iter()
            .position(|l| l.id == target_link || l.entity_id == target_link)
            .ok_or(AppError::NotFound)?;
        let removed = media.links.remove(idx);
        if removed.entity_type == "individual" {
            let still_tagged = media
                .face_tags
                .iter()
                .any(|t| t.person_id == removed.entity_id);
            let still_linked = media
                .links
                .iter()
                .any(|l| l.entity_type == "individual" && l.entity_id == removed.entity_id);
            if !still_tagged && !still_linked {
                media
                    .linked_person_ids
                    .retain(|pid| pid != &removed.entity_id);
            }
        }
        media.version += 1;
        removed
    };

    if removed_link.entity_type == "event" {
        if let Some(ev) = store
            .events
            .iter_mut()
            .find(|e| e.id == removed_link.entity_id)
        {
            ev.media_ids.retain(|mid| mid != &media_id);
        }
    }

    if removed_link.entity_type == "individual" {
        if let Some(person) = store
            .persons
            .iter_mut()
            .find(|p| p.id == removed_link.entity_id)
        {
            if person.avatar_media_id.as_deref() == Some(media_id.as_str()) {
                person.avatar_media_id = None;
                person.avatar_crop = None;
                person.avatar_url = None;
            }
        }
    }

    sync_media_links_and_living(&mut store, &media_id);

    Ok(StatusCode::NO_CONTENT)
}
