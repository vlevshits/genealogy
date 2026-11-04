#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/places",
    params(PlacesFilterQuery),
    responses(
        (status = 200, description = "Places directory & coordinates (authenticated users only)", body = PlacesDirectoryResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_places_directory(
    RequireAuth(_auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<PlacesFilterQuery>,
) -> Result<Json<PlacesDirectoryResponse>, AppError> {
    let store = state.privacy_store.read().await;

    let q_raw = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());
    let q_norm = q_raw.as_deref().map(normalize_search_tokens);

    let resolved_parent_filter = query
        .parent_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|raw| {
            resolve_place_id_in_store(&store.places, raw).unwrap_or_else(|| raw.to_string())
        });

    let items: Vec<PlaceSummaryDto> = store
        .places
        .iter()
        .filter(|p| {
            if let Some(ref pid) = resolved_parent_filter {
                if p.parent_id.as_deref() != Some(pid.as_str()) {
                    return false;
                }
            }
            if query.missing_coords == Some(true) && p.latitude.is_some() && p.longitude.is_some() {
                return false;
            }
            if let (Some(ref qr), Some(ref qn)) = (&q_raw, &q_norm) {
                let name_matches = p.name.to_lowercase().contains(qr)
                    || normalize_search_tokens(&p.name).contains(qn);
                let full_matches = p.full_name.to_lowercase().contains(qr)
                    || normalize_search_tokens(&p.full_name).contains(qn);
                let alt_matches = p.alt_names.iter().any(|a| {
                    a.name.to_lowercase().contains(qr)
                        || normalize_search_tokens(&a.name).contains(qn)
                });
                if !name_matches && !full_matches && !alt_matches {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    let total = items.len();

    Ok(Json(PlacesDirectoryResponse {
        items,
        next_cursor: None,
        total,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/places",
    request_body = CreatePlaceRequest,
    responses(
        (status = 201, description = "Place or comma-separated place hierarchy created/resolved", body = PlaceSummaryDto),
        (status = 400, description = "Validation error", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_place(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreatePlaceRequest>,
) -> Result<(StatusCode, Json<PlaceSummaryDto>), AppError> {
    let mut store = state.privacy_store.write().await;

    let raw_path = req
        .path
        .as_deref()
        .or(req.name.as_deref())
        .map(str::trim)
        .unwrap_or("");

    if raw_path.is_empty() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "Place name or path is required".to_string(),
            Some(fields),
        ));
    }

    // If `path` is explicitly provided, resolve/create the hierarchy top-down (`12-places.md §3.1, §7 DoD #1 & #5`).
    if req.path.is_some() {
        let leaf = resolve_or_create_place_chain(
            &mut store,
            raw_path,
            req.place_type.as_deref(),
            req.latitude,
            req.longitude,
            req.alt_names,
        )?;
        log_change(
            &mut store,
            None,
            "place.create",
            "place",
            &leaf.id,
            "create",
            Some(leaf.full_name.clone()),
            format!("Created place {} ({})", leaf.gid, leaf.full_name),
            None,
            serde_json::to_value(&leaf).ok(),
            Some(editor.user.id.to_string()),
            Some(editor.user.display_name()),
        );
        return Ok((StatusCode::CREATED, Json(leaf)));
    }

    let resolved_parent_id = match req
        .parent_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => Some(resolve_place_id_in_store(&store.places, raw).ok_or(AppError::NotFound)?),
        None => None,
    };

    let existing_gids: HashSet<String> = store.places.iter().map(|p| p.gid.clone()).collect();
    let gid = allocate_gid(
        'P',
        &mut store.next_place_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );
    let new_id = Uuid::new_v4().to_string();

    let alt_names = req
        .alt_names
        .unwrap_or_default()
        .into_iter()
        .map(|mut a| {
            if a.id.is_empty() {
                a.id = Uuid::new_v4().to_string();
            }
            a
        })
        .collect();

    let new_place = PlaceSummaryDto {
        id: new_id.clone(),
        gid,
        name: raw_path.to_string(),
        place_type: req.place_type.unwrap_or_else(|| "other".to_string()),
        parent_id: resolved_parent_id,
        full_name: raw_path.to_string(),
        country: None,
        latitude: req.latitude,
        longitude: req.longitude,
        events_count: 0,
        children_count: 0,
        alt_names,
        version: 1,
    };

    store.places.push(new_place);
    recompute_all_places_metadata(&mut store);

    let created = store
        .places
        .iter()
        .find(|p| p.id == new_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    log_change(
        &mut store,
        None,
        "place.create",
        "place",
        &created.id,
        "create",
        Some(created.full_name.clone()),
        format!("Created place {} ({})", created.gid, created.full_name),
        None,
        serde_json::to_value(&created).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/v1/places/{id}",
    params(
        ("id" = String, Path, description = "Place UUID or GID (e.g. P0001)")
    ),
    responses(
        (status = 200, description = "Place detail with breadcrumbs, child places, and events", body = PlaceDetailDto),
        (status = 404, description = "Place not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_place_detail(
    RequireAuth(_auth): RequireAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PlaceDetailDto>, AppError> {
    let store = state.privacy_store.read().await;
    let place_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;
    let place = store
        .places
        .iter()
        .find(|p| p.id == place_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let breadcrumbs = build_place_breadcrumbs(&store.places, &place_id);
    let children: Vec<PlaceSummaryDto> = store
        .places
        .iter()
        .filter(|p| p.parent_id.as_deref() == Some(place_id.as_str()))
        .cloned()
        .collect();
    let events: Vec<LifeEventDto> = store
        .events
        .iter()
        .filter(|e| e.place_id.as_deref() == Some(place_id.as_str()))
        .cloned()
        .collect();

    Ok(Json(PlaceDetailDto {
        place,
        breadcrumbs,
        children,
        events,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/places/{id}",
    params(
        ("id" = String, Path, description = "Place UUID or GID")
    ),
    request_body = UpdatePlaceRequest,
    responses(
        (status = 200, description = "Place updated and descendant full_name values recomputed", body = PlaceSummaryDto),
        (status = 400, description = "Validation error or place hierarchy cycle detected", body = ErrorResponse),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_place(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdatePlaceRequest>,
) -> Result<Json<PlaceSummaryDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let place_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;
    let idx = store
        .places
        .iter()
        .position(|p| p.id == place_id)
        .ok_or(AppError::NotFound)?;

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.places[idx].version,
        expected_version,
        &format!("Place {}", store.places[idx].gid),
        serde_json::to_value(&store.places[idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.places[idx]).ok();

    if req.clear_parent == Some(true) {
        store.places[idx].parent_id = None;
    } else if let Some(ref raw_parent) = req.parent_id {
        if raw_parent.trim().is_empty() {
            store.places[idx].parent_id = None;
        } else {
            let resolved_parent =
                resolve_place_id_in_store(&store.places, raw_parent).ok_or(AppError::NotFound)?;
            ensure_no_place_cycle(&store.places, &place_id, Some(&resolved_parent))?;
            store.places[idx].parent_id = Some(resolved_parent);
        }
    }

    if let Some(name) = req.name {
        let trimmed = name.trim().to_string();
        if !trimmed.is_empty() {
            store.places[idx].name = trimmed;
        }
    }
    if let Some(pt) = req.place_type {
        store.places[idx].place_type = pt;
    }
    if let Some(lat) = req.latitude {
        store.places[idx].latitude = Some(lat);
    }
    if let Some(lon) = req.longitude {
        store.places[idx].longitude = Some(lon);
    }
    if let Some(alts) = req.alt_names {
        store.places[idx].alt_names = alts
            .into_iter()
            .map(|mut a| {
                if a.id.is_empty() {
                    a.id = Uuid::new_v4().to_string();
                }
                a
            })
            .collect();
    }

    store.places[idx].version += 1;
    recompute_all_places_metadata(&mut store);

    // Refresh cached `place_full_name` on events
    let full_name_map: HashMap<String, String> = store
        .places
        .iter()
        .map(|p| (p.id.clone(), p.full_name.clone()))
        .collect();
    for ev in &mut store.events {
        if let Some(ref pid) = ev.place_id {
            if let Some(fname) = full_name_map.get(pid) {
                ev.place_full_name = Some(fname.clone());
            }
        }
    }
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let updated = store
        .places
        .iter()
        .find(|p| p.id == place_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    log_change(
        &mut store,
        None,
        "place.update",
        "place",
        &updated.id,
        "update",
        Some(updated.full_name.clone()),
        format!("Updated place {} ({})", updated.gid, updated.full_name),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/places/{id}",
    params(
        ("id" = String, Path, description = "Place UUID or GID")
    ),
    responses(
        (status = 204, description = "Unused place deleted"),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 409, description = "Place is referenced by events or child places (place_in_use)", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_place(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let place_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;

    let has_child_places = store
        .places
        .iter()
        .any(|p| p.parent_id.as_deref() == Some(place_id.as_str()));
    let has_referencing_events = store
        .events
        .iter()
        .any(|e| e.place_id.as_deref() == Some(place_id.as_str()));

    if has_child_places || has_referencing_events {
        return Err(AppError::PlaceInUse);
    }

    let idx = store
        .places
        .iter()
        .position(|p| p.id == place_id)
        .ok_or(AppError::NotFound)?;
    let removed = store.places.remove(idx);
    recompute_all_places_metadata(&mut store);

    log_change(
        &mut store,
        None,
        "place.delete",
        "place",
        &removed.id,
        "delete",
        Some(removed.full_name.clone()),
        format!("Deleted place {} ({})", removed.gid, removed.full_name),
        serde_json::to_value(&removed).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/places/{id}/merge",
    params(
        ("id" = String, Path, description = "Target place UUID or GID to keep")
    ),
    request_body = MergePlaceRequest,
    responses(
        (status = 200, description = "Source place merged into target place", body = PlaceSummaryDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Target or source place not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn merge_place(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MergePlaceRequest>,
) -> Result<Json<PlaceSummaryDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;

    let source_raw = req
        .other_id
        .as_deref()
        .or(req.source_place_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            let mut fields = HashMap::new();
            fields.insert("other_id".to_string(), "required".to_string());
            AppError::ValidationFailed(
                "other_id (source place to merge) is required".to_string(),
                Some(fields),
            )
        })?;

    let source_id =
        resolve_place_id_in_store(&store.places, source_raw).ok_or(AppError::NotFound)?;

    if target_id == source_id {
        let mut fields = HashMap::new();
        fields.insert("other_id".to_string(), "same_as_target".to_string());
        return Err(AppError::ValidationFailed(
            "Cannot merge a place with itself".to_string(),
            Some(fields),
        ));
    }

    let before_target_json = store
        .places
        .iter()
        .find(|p| p.id == target_id)
        .and_then(|p| serde_json::to_value(p).ok());

    let source_idx = store
        .places
        .iter()
        .position(|p| p.id == source_id)
        .ok_or(AppError::NotFound)?;
    let source_place = store.places.remove(source_idx);

    // Re-point events referencing source_id to target_id
    for ev in &mut store.events {
        if ev.place_id.as_deref() == Some(source_id.as_str()) {
            ev.place_id = Some(target_id.clone());
        }
    }

    // Re-point child places referencing source_id to target_id
    for p in &mut store.places {
        if p.parent_id.as_deref() == Some(source_id.as_str()) && p.id != target_id {
            p.parent_id = Some(target_id.clone());
        }
    }

    // Re-point notes referencing source_id to target_id
    for n in &mut store.notes {
        if n.entity_type == "place" && n.entity_id == source_id {
            n.entity_id = target_id.clone();
        }
    }

    // Merge alternate names + old primary name into target place (`12-places.md §3.3, §7 DoD #4`)
    if let Some(target_place) = store.places.iter_mut().find(|p| p.id == target_id) {
        if target_place.latitude.is_none() {
            target_place.latitude = source_place.latitude;
        }
        if target_place.longitude.is_none() {
            target_place.longitude = source_place.longitude;
        }

        let mut candidate_alts = vec![PlaceAltNameDto {
            id: Uuid::new_v4().to_string(),
            name: source_place.name.clone(),
            lang: None,
            note: Some(format!("Merged from {}", source_place.gid)),
        }];
        candidate_alts.extend(source_place.alt_names.clone());

        for cand in candidate_alts {
            let matches_primary = target_place.name.eq_ignore_ascii_case(&cand.name);
            let already_in_alts = target_place
                .alt_names
                .iter()
                .any(|a| a.name.eq_ignore_ascii_case(&cand.name));
            if !matches_primary && !already_in_alts {
                target_place.alt_names.push(cand);
            }
        }

        target_place.events_count += source_place.events_count;
        target_place.version += 1;
    }

    recompute_all_places_metadata(&mut store);

    let full_name_map: HashMap<String, String> = store
        .places
        .iter()
        .map(|p| (p.id.clone(), p.full_name.clone()))
        .collect();
    for ev in &mut store.events {
        if let Some(ref pid) = ev.place_id {
            if let Some(fname) = full_name_map.get(pid) {
                ev.place_full_name = Some(fname.clone());
            }
        }
    }
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let merged = store
        .places
        .iter()
        .find(|p| p.id == target_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let tx_id = Uuid::new_v4().to_string();
    log_change(
        &mut store,
        Some(tx_id.clone()),
        "place.merge",
        "place",
        &source_place.id,
        "delete",
        Some(source_place.full_name.clone()),
        format!(
            "Merged place {} ({}) into {} ({})",
            source_place.gid, source_place.full_name, merged.gid, merged.full_name
        ),
        serde_json::to_value(&source_place).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );
    log_change(
        &mut store,
        Some(tx_id),
        "place.merge",
        "place",
        &merged.id,
        "update",
        Some(merged.full_name.clone()),
        format!(
            "Merged {} into place {} ({})",
            source_place.gid, merged.gid, merged.full_name
        ),
        before_target_json,
        serde_json::to_value(&merged).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(merged))
}
