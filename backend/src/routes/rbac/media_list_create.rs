#[allow(unused_imports)]
use super::*;

pub(crate) fn extract_year_from_media(m: &MediaAssetSummaryDto) -> Option<i32> {
    if let Some(ds) = m.date_sort {
        let yr = ds / 10_000;
        if (1000..=2999).contains(&yr) {
            return Some(yr);
        }
    }
    for text in [m.date_taken.as_deref().unwrap_or(""), m.title.as_str()] {
        for token in text.split(|c: char| !c.is_ascii_digit()) {
            if token.len() == 4 {
                if let Ok(yr) = token.parse::<i32>() {
                    if (1500..=2100).contains(&yr) {
                        return Some(yr);
                    }
                }
            }
        }
    }
    None
}

pub fn sync_media_links_and_living(store: &mut GenealogyPrivacyStore, media_id: &str) {
    let person_map: HashMap<String, (String, bool)> = store
        .persons
        .iter()
        .map(|p| (p.id.clone(), (format_person_display_name(p), p.is_living)))
        .collect();

    let (media_id_str, media_gid_str) = match store
        .media
        .iter()
        .find(|m| m.id == media_id || m.gid.eq_ignore_ascii_case(media_id))
    {
        Some(m) => (m.id.clone(), m.gid.clone()),
        None => return,
    };

    let avatar_persons: Vec<String> = store
        .persons
        .iter()
        .filter(|p| {
            p.avatar_media_id.as_deref() == Some(media_id_str.as_str())
                || p.avatar_media_id.as_deref() == Some(media_gid_str.as_str())
        })
        .map(|p| p.id.clone())
        .collect();

    if let Some(media) = store.media.iter_mut().find(|m| m.id == media_id_str) {
        for tag in &mut media.face_tags {
            if let Some((pname, living)) = person_map.get(&tag.person_id) {
                tag.person_name = pname.clone();
                tag.is_living = *living;
            }
            if !media.linked_person_ids.contains(&tag.person_id) {
                media.linked_person_ids.push(tag.person_id.clone());
            }
            if !media
                .links
                .iter()
                .any(|l| l.entity_type == "individual" && l.entity_id == tag.person_id)
            {
                media.links.push(MediaLinkDto {
                    id: Uuid::new_v4().to_string(),
                    media_id: media.id.clone(),
                    entity_type: "individual".to_string(),
                    entity_id: tag.person_id.clone(),
                    role: Some("depicted".to_string()),
                });
            }
        }

        for avatar_person_id in &avatar_persons {
            if !media.linked_person_ids.contains(avatar_person_id) {
                media.linked_person_ids.push(avatar_person_id.clone());
            }
            if let Some(link) = media
                .links
                .iter_mut()
                .find(|l| l.entity_type == "individual" && &l.entity_id == avatar_person_id)
            {
                link.role = Some("primary_avatar".to_string());
            } else {
                media.links.push(MediaLinkDto {
                    id: Uuid::new_v4().to_string(),
                    media_id: media.id.clone(),
                    entity_type: "individual".to_string(),
                    entity_id: avatar_person_id.clone(),
                    role: Some("primary_avatar".to_string()),
                });
            }
        }

        for link in &mut media.links {
            if link.entity_type == "individual"
                && link.role.as_deref() == Some("primary_avatar")
                && !avatar_persons.contains(&link.entity_id)
            {
                link.role = Some("depicted".to_string());
            }
        }

        for link in &media.links {
            if link.entity_type == "individual"
                && !media.linked_person_ids.contains(&link.entity_id)
            {
                media.linked_person_ids.push(link.entity_id.clone());
            }
        }

        if media.linked_person_ids.is_empty() {
            media.only_living_persons = false;
        } else {
            media.only_living_persons = media
                .linked_person_ids
                .iter()
                .all(|pid| person_map.get(pid).is_some_and(|(_, living)| *living));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_primary_avatar_for_person(
    store: &mut GenealogyPrivacyStore,
    person_id: &str,
    media_id: &str,
    tag_id: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    for m in &mut store.media {
        for t in &mut m.face_tags {
            if t.person_id == person_id {
                t.is_primary_avatar = m.id == media_id && t.id == tag_id;
            }
        }
    }
    if let Some(person) = store.persons.iter_mut().find(|p| p.id == person_id) {
        person.avatar_media_id = Some(media_id.to_string());
        person.avatar_crop = Some(AvatarCropDto {
            x,
            y,
            width: w,
            height: h,
        });
        person.avatar_url = Some(format!(
            "/api/v1/media/{media_id}/crop?x={x:.4}&y={y:.4}&w={w:.4}&h={h:.4}"
        ));
    }
}

pub(crate) fn clear_primary_avatar_for_person(store: &mut GenealogyPrivacyStore, person_id: &str) {
    for m in &mut store.media {
        for t in &mut m.face_tags {
            if t.person_id == person_id {
                t.is_primary_avatar = false;
            }
        }
    }
    if let Some(person) = store.persons.iter_mut().find(|p| p.id == person_id) {
        person.avatar_media_id = None;
        person.avatar_crop = None;
        person.avatar_url = None;
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/media",
    params(
        ("q" = Option<String>, Query, description = "Search query across title, description, place, or tagged person"),
        ("person_id" = Option<String>, Query, description = "Filter by tagged/linked person UUID or GID"),
        ("decade" = Option<String>, Query, description = "Filter by decade (e.g. '1940s' or '1940')"),
        ("type" = Option<String>, Query, description = "Filter by media category ('photos', 'documents', 'audio')"),
        ("untagged" = Option<bool>, Query, description = "Filter untagged media (true = untagged only, false = tagged only)"),
        ("is_missing" = Option<bool>, Query, description = "Filter by missing media stubs pending bundle upload"),
        ("cursor" = Option<String>, Query, description = "Pagination cursor"),
        ("limit" = Option<usize>, Query, description = "Max items per page (default 50, max 100)")
    ),
    responses(
        (status = 200, description = "Media archive & gallery (authenticated users only)", body = MediaArchiveResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn list_media_archive(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<MediaFilterQuery>,
) -> Result<Json<MediaArchiveResponse>, AppError> {
    let access = auth.access();
    let store = state.privacy_store.read().await;

    let unfiltered_items: Vec<MediaAssetSummaryDto> = store
        .media
        .iter()
        .filter_map(|m| redact_media_for_caller(m, &access, &store))
        .collect();
    let total_unfiltered = unfiltered_items.len();

    let resolved_person_filter = if let Some(ref pid_raw) = query.person_id {
        let trimmed = pid_raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(
                resolve_person_id_in_store(&store.persons, trimmed)
                    .unwrap_or_else(|| "__nonexistent__".to_string()),
            )
        }
    } else {
        None
    };

    let decade_start_year: Option<i32> = query.decade.as_deref().and_then(|d| {
        let digits: String = d
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if digits.len() == 4 {
            digits.parse::<i32>().ok().map(|yr| (yr / 10) * 10)
        } else {
            None
        }
    });

    let q_tokens: Vec<String> = query
        .q
        .as_deref()
        .map(|q| q.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default();

    let filtered: Vec<MediaAssetSummaryDto> = unfiltered_items
        .into_iter()
        .filter(|m| {
            if let Some(missing_filter) = query.is_missing {
                if m.is_missing != missing_filter {
                    return false;
                }
            }

            if let Some(ref target_pid) = resolved_person_filter {
                let matches_person = m.linked_person_ids.iter().any(|id| id == target_pid)
                    || m.face_tags.iter().any(|t| &t.person_id == target_pid)
                    || m.links
                        .iter()
                        .any(|l| l.entity_type == "individual" && &l.entity_id == target_pid);
                if !matches_person {
                    return false;
                }
            }

            if let Some(dec_yr) = decade_start_year {
                let Some(media_yr) = extract_year_from_media(m) else {
                    return false;
                };
                if media_yr < dec_yr || media_yr > dec_yr + 9 {
                    return false;
                }
            }

            if let Some(ref mtype) = query.media_type {
                let mt = mtype.trim().to_lowercase();
                if !mt.is_empty() && mt != "all" {
                    let mime_matches = match mt.as_str() {
                        "photos" | "photo" | "image" | "images" => {
                            m.mime_type.starts_with("image/")
                        }
                        "documents" | "document" | "pdf" | "doc" => {
                            m.mime_type == "application/pdf" || m.mime_type.starts_with("text/")
                        }
                        "audio" => m.mime_type.starts_with("audio/"),
                        "video" => m.mime_type.starts_with("video/"),
                        other => m.mime_type.eq_ignore_ascii_case(other),
                    };
                    if !mime_matches {
                        return false;
                    }
                }
            }

            if let Some(untagged_filter) = query.untagged {
                let is_untagged = m.face_tags.is_empty() && m.linked_person_ids.is_empty();
                if untagged_filter != is_untagged {
                    return false;
                }
            }

            if !q_tokens.is_empty() {
                let mut searchable_fields = vec![
                    m.title.clone(),
                    m.gid.clone(),
                    m.original_file_name.clone().unwrap_or_default(),
                    m.description.clone().unwrap_or_default(),
                    m.place_full_name.clone().unwrap_or_default(),
                ];
                for tag in &m.face_tags {
                    searchable_fields.push(tag.person_name.clone());
                }
                let combined_raw = searchable_fields.join(" ").to_lowercase();
                let combined_ortho = searchable_fields
                    .iter()
                    .flat_map(|s| s.split_whitespace())
                    .map(normalize_russian_orthography_word)
                    .collect::<Vec<_>>()
                    .join(" ");
                let combined_lat = transliterate_to_latin(&combined_ortho);

                for tok in &q_tokens {
                    let tok_lower = tok.to_lowercase();
                    let tok_ortho = normalize_russian_orthography_word(tok);
                    let tok_lat = transliterate_to_latin(&tok_ortho);
                    let matched = combined_raw.contains(&tok_lower)
                        || combined_ortho.contains(&tok_ortho)
                        || (!tok_lat.is_empty() && combined_lat.contains(&tok_lat));
                    if !matched {
                        return false;
                    }
                }
            }

            true
        })
        .collect();

    let total = filtered.len();
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let start_idx = if let Some(ref cur) = query.cursor {
        let cur_trimmed = cur.trim();
        if let Ok(offset) = cur_trimmed.parse::<usize>() {
            offset.min(total)
        } else {
            filtered
                .iter()
                .position(|m| m.id == cur_trimmed || m.gid.eq_ignore_ascii_case(cur_trimmed))
                .map(|idx| idx + 1)
                .unwrap_or(0)
        }
    } else {
        0
    };

    let items: Vec<MediaAssetSummaryDto> =
        filtered.into_iter().skip(start_idx).take(limit).collect();
    let next_cursor = if start_idx + items.len() < total {
        items.last().map(|m| m.id.clone())
    } else {
        None
    };

    Ok(Json(MediaArchiveResponse {
        items,
        next_cursor,
        total,
        total_unfiltered,
    }))
}

pub(crate) const ALLOWED_MEDIA_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/tiff",
    "application/pdf",
    "audio/mpeg",
    "audio/ogg",
    "audio/wav",
    "audio/mp4",
    "video/mp4",
];

pub(crate) fn extension_for_media(file_name: &str, mime_type: &str) -> &'static str {
    let lower_name = file_name.to_lowercase();
    if lower_name.ends_with(".jpg") || lower_name.ends_with(".jpeg") {
        return "jpg";
    }
    if lower_name.ends_with(".png") {
        return "png";
    }
    if lower_name.ends_with(".webp") {
        return "webp";
    }
    if lower_name.ends_with(".gif") {
        return "gif";
    }
    if lower_name.ends_with(".tif") || lower_name.ends_with(".tiff") {
        return "tiff";
    }
    if lower_name.ends_with(".pdf") {
        return "pdf";
    }
    if lower_name.ends_with(".mp3") {
        return "mp3";
    }
    if lower_name.ends_with(".ogg") {
        return "ogg";
    }
    if lower_name.ends_with(".wav") {
        return "wav";
    }
    match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/tiff" => "tiff",
        "application/pdf" => "pdf",
        "audio/mpeg" => "mp3",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/mp4" | "video/mp4" => "mp4",
        _ => "webp",
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/media",
    request_body = CreateMediaRequest,
    responses(
        (status = 201, description = "Media asset uploaded and indexed", body = MediaAssetSummaryDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 422, description = "Validation error (unsupported MIME type or invalid payload)", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn create_media(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateMediaRequest>,
) -> Result<(StatusCode, Json<MediaAssetSummaryDto>), AppError> {
    let orig_name = req.file_name.as_deref().unwrap_or("").trim();
    if orig_name.is_empty() {
        return Err(AppError::ValidationFailed(
            "file_name is required".to_string(),
            Some(HashMap::from([(
                "file_name".to_string(),
                "required".to_string(),
            )])),
        ));
    }

    let mime_trimmed = req.mime_type.as_deref().unwrap_or("").trim().to_lowercase();
    let mime_type = if mime_trimmed.is_empty() {
        "image/webp".to_string()
    } else {
        mime_trimmed
    };

    if !ALLOWED_MEDIA_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(AppError::ValidationFailed(
            format!("Unsupported media MIME type: {mime_type}"),
            Some(HashMap::from([(
                "mime_type".to_string(),
                "unsupported_mime_type".to_string(),
            )])),
        ));
    }

    let raw_bytes: Vec<u8> = if let Some(ref b64_raw) = req.data {
        let stripped = b64_raw
            .trim()
            .split_once(";base64,")
            .map(|(_, payload)| payload)
            .unwrap_or_else(|| b64_raw.trim());
        if stripped.is_empty() {
            DEFAULT_WEBP_BYTES.to_vec()
        } else {
            BASE64_STANDARD.decode(stripped).map_err(|_| {
                AppError::ValidationFailed(
                    "Invalid base64 media payload".to_string(),
                    Some(HashMap::from([(
                        "data".to_string(),
                        "invalid_base64".to_string(),
                    )])),
                )
            })?
        }
    } else {
        DEFAULT_WEBP_BYTES.to_vec()
    };

    let sha_digest = Sha256::digest(&raw_bytes);
    let sha256_hex: String = sha_digest.iter().map(|b| format!("{b:02x}")).collect();
    let file_size_bytes = raw_bytes.len() as u64;

    let (date_taken, date_sort) = if let Some(ref dt_raw) = req.date_taken {
        let dt_trimmed = dt_raw.trim();
        if dt_trimmed.is_empty() {
            (None, None)
        } else {
            let parsed = GenDate::parse(dt_trimmed);
            let sort = parsed.sort_key();
            (Some(dt_trimmed.to_string()), sort)
        }
    } else {
        (None, None)
    };

    let mut store = state.privacy_store.write().await;

    let (place_id, place_full_name) = if let Some(ref pid_raw) = req.place_id {
        if !pid_raw.trim().is_empty() {
            let pid =
                resolve_place_id_in_store(&store.places, pid_raw).ok_or(AppError::NotFound)?;
            let fname = store
                .places
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.full_name.clone());
            (Some(pid), fname)
        } else {
            (None, None)
        }
    } else if let Some(ref ppath) = req.place_path {
        if !ppath.trim().is_empty() {
            let place = resolve_or_create_place_chain(&mut store, ppath, None, None, None, None)?;
            (Some(place.id), Some(place.full_name))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let is_image = mime_type.starts_with("image/");
    let width = req.width.or(if is_image { Some(1200) } else { None });
    let height = req.height.or(if is_image { Some(900) } else { None });
    let description = req
        .description
        .or(req.caption)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Check if an existing missing GEDCOM media stub matches by base filename (`03 §2.2 #4`)
    let upload_base_name = orig_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(orig_name)
        .trim();
    let matching_stub_id = if !upload_base_name.is_empty() {
        store
            .media
            .iter()
            .find(|m| {
                m.is_missing
                    && (m
                        .original_file_name
                        .as_deref()
                        .is_some_and(|ofn| ofn.eq_ignore_ascii_case(upload_base_name))
                        || m.file_name
                            .as_deref()
                            .is_some_and(|fnm| fnm.eq_ignore_ascii_case(upload_base_name))
                        || m.external_path
                            .as_deref()
                            .and_then(|p| p.rsplit(['/', '\\']).next())
                            .is_some_and(|bn| bn.eq_ignore_ascii_case(upload_base_name)))
            })
            .map(|m| m.id.clone())
    } else {
        None
    };

    let mut resolved_linked_persons = Vec::new();
    if let Some(ref person_ids) = req.linked_person_ids {
        for raw_pid in person_ids {
            let pid = resolve_person_id_in_store(&store.persons, raw_pid.trim())
                .ok_or(AppError::NotFound)?;
            if !resolved_linked_persons.contains(&pid) {
                resolved_linked_persons.push(pid);
            }
        }
    }

    if let Some(stub_id) = matching_stub_id {
        let ext = extension_for_media(orig_name, &mime_type);
        let stored_file_name = format!("{stub_id}.{ext}");
        save_media_bytes_to_disk(
            &state.config.media_storage_path,
            &stored_file_name,
            &raw_bytes,
        );

        if let Some(stub) = store.media.iter_mut().find(|m| m.id == stub_id) {
            stub.mime_type = mime_type;
            stub.file_url = format!("/api/v1/media/{stub_id}/file");
            stub.file_name = Some(stored_file_name);
            stub.original_file_name = Some(upload_base_name.to_string());
            stub.file_size_bytes = Some(file_size_bytes);
            stub.sha256 = Some(sha256_hex);
            stub.width = width;
            stub.height = height;
            stub.is_missing = false;
            stub.version += 1;

            if let Some(explicit_title) = req
                .title
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                stub.title = explicit_title.to_string();
            }
            if description.is_some() {
                stub.description = description;
            }
            if date_taken.is_some() {
                stub.date_taken = date_taken;
                stub.date_sort = date_sort;
            }
            if place_id.is_some() {
                stub.place_id = place_id;
                stub.place_full_name = place_full_name;
            }
            if let Some(priv_flag) = req.is_private {
                stub.is_private = priv_flag;
            }
            for pid in resolved_linked_persons {
                if !stub.linked_person_ids.contains(&pid) {
                    stub.linked_person_ids.push(pid.clone());
                    stub.links.push(MediaLinkDto {
                        id: Uuid::new_v4().to_string(),
                        media_id: stub_id.clone(),
                        entity_type: "individual".to_string(),
                        entity_id: pid,
                        role: Some("gallery".to_string()),
                    });
                }
            }
        }

        sync_media_links_and_living(&mut store, &stub_id);

        let reconciled = store
            .media
            .iter()
            .find(|m| m.id == stub_id)
            .cloned()
            .ok_or(AppError::NotFound)?;

        log_change(
            &mut store,
            None,
            "media.reconcile_stub",
            "media",
            &reconciled.id,
            "update",
            Some(reconciled.title.clone()),
            format!(
                "Reconciled media stub {} ({})",
                reconciled.gid, reconciled.title
            ),
            None,
            serde_json::to_value(&reconciled).ok(),
            Some(editor.user.id.to_string()),
            Some(editor.user.display_name()),
        );

        return Ok((StatusCode::CREATED, Json(reconciled)));
    }

    let id = Uuid::new_v4().to_string();
    let ext = extension_for_media(orig_name, &mime_type);
    let stored_file_name = format!("{id}.{ext}");

    save_media_bytes_to_disk(
        &state.config.media_storage_path,
        &stored_file_name,
        &raw_bytes,
    );

    let existing_gids: HashSet<String> = store.media.iter().map(|m| m.gid.clone()).collect();
    let gid = allocate_gid(
        'M',
        &mut store.next_media_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );

    let title = req
        .title
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| orig_name.to_string());

    let links: Vec<MediaLinkDto> = resolved_linked_persons
        .iter()
        .map(|pid| MediaLinkDto {
            id: Uuid::new_v4().to_string(),
            media_id: id.clone(),
            entity_type: "individual".to_string(),
            entity_id: pid.clone(),
            role: Some("gallery".to_string()),
        })
        .collect();

    let media = MediaAssetSummaryDto {
        id: id.clone(),
        gid,
        title,
        mime_type,
        file_url: format!("/api/v1/media/{id}/file"),
        file_name: Some(stored_file_name),
        original_file_name: Some(orig_name.to_string()),
        file_size_bytes: Some(file_size_bytes),
        sha256: Some(sha256_hex),
        width,
        height,
        description,
        date_taken,
        date_sort,
        place_id,
        place_full_name,
        is_missing: false,
        external_path: None,
        is_private: req.is_private.unwrap_or(false),
        only_living_persons: false,
        linked_person_ids: resolved_linked_persons,
        face_tags: vec![],
        links,
        version: 1,
    };

    store.media.push(media);
    sync_media_links_and_living(&mut store, &id);

    let created = store
        .media
        .iter()
        .find(|m| m.id == id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    log_change(
        &mut store,
        None,
        "media.create",
        "media",
        &created.id,
        "create",
        Some(created.title.clone()),
        format!("Uploaded media {} ({})", created.gid, created.title),
        None,
        serde_json::to_value(&created).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok((StatusCode::CREATED, Json(created)))
}
