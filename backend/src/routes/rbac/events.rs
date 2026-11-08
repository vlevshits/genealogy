#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/events",
    params(EventFilterQuery),
    responses(
        (status = 200, description = "Chronological life events list", body = EventListResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_events(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<EventFilterQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let filter_ind_id = query.individual_id.as_deref().and_then(|raw| {
        resolve_person_id_in_store(&store.persons, raw).or_else(|| Some(raw.to_string()))
    });

    let filter_fam_id = query.family_id.as_deref().map(|raw| {
        store
            .families
            .iter()
            .find(|f| f.id == raw.trim() || f.gid.eq_ignore_ascii_case(raw.trim()))
            .map(|f| f.id.clone())
            .unwrap_or_else(|| raw.trim().to_string())
    });

    let mut items: Vec<LifeEventDto> = store
        .events
        .iter()
        .filter(|e| !e.is_private || access.can_view_private())
        .filter(|e| {
            if let Some(ref ind_id) = filter_ind_id {
                if e.individual_id.as_deref() != Some(ind_id.as_str()) {
                    return false;
                }
            }
            if let Some(ref fam_id) = filter_fam_id {
                if e.family_id.as_deref() != Some(fam_id.as_str()) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    items.sort_by_key(|e| e.date_sort.unwrap_or(i32::MAX));
    let total = items.len();

    Ok(Json(EventListResponse { items, total }))
}

#[utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Life event created and vitals/living status recomputed", body = LifeEventDto),
        (status = 400, description = "Validation error (e.g. family-only event on individual)", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_event(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<LifeEventDto>), AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;

    let ind_id = match req
        .individual_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
        None => None,
    };

    let fam_id = match req
        .family_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => Some(
            store
                .families
                .iter()
                .find(|f| f.id == raw || f.gid.eq_ignore_ascii_case(raw))
                .map(|f| f.id.clone())
                .ok_or(AppError::NotFound)?,
        ),
        None => None,
    };

    // Exclusive owner check (`10-core-data-model.md §4.3`)
    if ind_id.is_some() == fam_id.is_some() {
        let mut fields = HashMap::new();
        fields.insert(
            "owner".to_string(),
            "exactly_one_of_individual_id_or_family_id_required".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "An event must belong to either an individual or a family, never both or neither"
                .to_string(),
            Some(fields),
        ));
    }

    let event_type = normalize_event_type(&req.event_type);
    if ind_id.is_some() && is_family_only_event_type(&event_type) {
        let mut fields = HashMap::new();
        fields.insert(
            "event_type".to_string(),
            "family_only_event_on_individual".to_string(),
        );
        return Err(AppError::ValidationFailed(
            format!(
                "Event type '{event_type}' can only be attached to a family record, not an individual"
            ),
            Some(fields),
        ));
    }

    let parsed_date = if let Some(d) = req.date {
        Some(d)
    } else {
        req.date_text
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(GenDate::parse)
    };
    let date_sort = parsed_date.as_ref().and_then(|d| d.sort_key());
    let gedcom_date = parsed_date.as_ref().map(|d| d.to_gedcom());

    let (resolved_place_id, resolved_place_full_name) = if let Some(ref pid_raw) = req.place_id {
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

    let existing_gids: HashSet<String> = store.events.iter().map(|e| e.gid.clone()).collect();
    let gid = allocate_gid(
        'E',
        &mut store.next_event_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );

    let ev = LifeEventDto {
        id: Uuid::new_v4().to_string(),
        gid,
        individual_id: ind_id,
        family_id: fam_id,
        event_type,
        custom_type: req.custom_type,
        date: parsed_date,
        date_sort,
        gedcom_date,
        place_id: resolved_place_id,
        place_full_name: resolved_place_full_name,
        description: req.description,
        source_text: req
            .source_text
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        media_ids: req.media_ids.unwrap_or_default(),
        is_private: req.is_private.unwrap_or(false),
        version: 1,
    };

    store.events.push(ev.clone());
    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    log_change(
        &mut store,
        None,
        "event.create",
        "event",
        &ev.id,
        "create",
        Some(format!("{} ({})", ev.event_type, ev.gid)),
        format!("Created {} event {}", ev.event_type, ev.gid),
        None,
        serde_json::to_value(&ev).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok((StatusCode::CREATED, Json(ev)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/events/{id}",
    params(
        ("id" = String, Path, description = "Event UUID or GID (e.g. E0100)")
    ),
    request_body = UpdateEventRequest,
    responses(
        (status = 200, description = "Life event updated", body = LifeEventDto),
        (status = 404, description = "Event not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_event(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateEventRequest>,
) -> Result<Json<LifeEventDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let ev_idx = store
        .events
        .iter()
        .position(|e| e.id == target || e.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.events[ev_idx].version,
        expected_version,
        &format!("Event {}", store.events[ev_idx].gid),
        serde_json::to_value(&store.events[ev_idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.events[ev_idx]).ok();

    if let Some(ref et) = req.event_type {
        let normalized_et = normalize_event_type(et);
        if store.events[ev_idx].individual_id.is_some() && is_family_only_event_type(&normalized_et)
        {
            let mut fields = HashMap::new();
            fields.insert(
                "event_type".to_string(),
                "family_only_event_on_individual".to_string(),
            );
            return Err(AppError::ValidationFailed(
                format!("Event type '{normalized_et}' can only be attached to a family record"),
                Some(fields),
            ));
        }
        store.events[ev_idx].event_type = normalized_et;
    }

    if let Some(ct) = req.custom_type {
        store.events[ev_idx].custom_type = Some(ct);
    }

    if let Some(d) = req.date {
        store.events[ev_idx].date_sort = d.sort_key();
        store.events[ev_idx].gedcom_date = Some(d.to_gedcom());
        store.events[ev_idx].date = Some(d);
    } else if let Some(ref raw_date) = req.date_text {
        if raw_date.trim().is_empty() {
            store.events[ev_idx].date = None;
            store.events[ev_idx].date_sort = None;
            store.events[ev_idx].gedcom_date = None;
        } else {
            let parsed = GenDate::parse(raw_date);
            store.events[ev_idx].date_sort = parsed.sort_key();
            store.events[ev_idx].gedcom_date = Some(parsed.to_gedcom());
            store.events[ev_idx].date = Some(parsed);
        }
    }

    if let Some(ref pid_raw) = req.place_id {
        if pid_raw.trim().is_empty() {
            store.events[ev_idx].place_id = None;
            store.events[ev_idx].place_full_name = None;
        } else {
            let pid =
                resolve_place_id_in_store(&store.places, pid_raw).ok_or(AppError::NotFound)?;
            let fname = store
                .places
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.full_name.clone());
            store.events[ev_idx].place_id = Some(pid);
            store.events[ev_idx].place_full_name = fname;
        }
    } else if let Some(ref ppath) = req.place_path {
        if !ppath.trim().is_empty() {
            let place = resolve_or_create_place_chain(&mut store, ppath, None, None, None, None)?;
            store.events[ev_idx].place_id = Some(place.id);
            store.events[ev_idx].place_full_name = Some(place.full_name);
        }
    }

    if let Some(desc) = req.description {
        store.events[ev_idx].description = Some(desc);
    }
    if let Some(src) = req.source_text {
        let trimmed = src.trim().to_string();
        store.events[ev_idx].source_text = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    }
    if let Some(mids) = req.media_ids {
        store.events[ev_idx].media_ids = mids;
    }
    if let Some(is_priv) = req.is_private {
        store.events[ev_idx].is_private = is_priv;
    }

    store.events[ev_idx].version += 1;
    let updated = store.events[ev_idx].clone();

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    log_change(
        &mut store,
        None,
        "event.update",
        "event",
        &updated.id,
        "update",
        Some(format!("{} ({})", updated.event_type, updated.gid)),
        format!("Updated {} event {}", updated.event_type, updated.gid),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/events/{id}",
    params(
        ("id" = String, Path, description = "Event UUID or GID")
    ),
    responses(
        (status = 204, description = "Life event deleted and vitals recomputed"),
        (status = 404, description = "Event not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_event(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let ev_idx = store
        .events
        .iter()
        .position(|e| e.id == target || e.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed_ev = store.events.remove(ev_idx);
    if let Some(ref ind_id) = removed_ev.individual_id {
        let has_remaining_death = store.events.iter().any(|e| {
            e.individual_id.as_deref() == Some(ind_id.as_str())
                && matches!(e.event_type.as_str(), "death" | "burial" | "cremation")
        });
        let has_remaining_birth = store.events.iter().any(|e| {
            e.individual_id.as_deref() == Some(ind_id.as_str())
                && matches!(e.event_type.as_str(), "birth" | "baptism")
        });

        if let Some(person) = store.persons.iter_mut().find(|p| &p.id == ind_id) {
            if !has_remaining_death
                && matches!(
                    removed_ev.event_type.as_str(),
                    "death" | "burial" | "cremation"
                )
            {
                person.has_death_event = false;
                person.death_sort = None;
                person.death = None;
            }
            if !has_remaining_birth && matches!(removed_ev.event_type.as_str(), "birth" | "baptism")
            {
                person.birth_sort = None;
                person.birth = None;
            }
        }
    }

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    log_change(
        &mut store,
        None,
        "event.delete",
        "event",
        &removed_ev.id,
        "delete",
        Some(format!("{} ({})", removed_ev.event_type, removed_ev.gid)),
        format!("Deleted {} event {}", removed_ev.event_type, removed_ev.gid),
        serde_json::to_value(&removed_ev).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/events/sources",
    params(SourceSearchQuery),
    responses(
        (status = 200, description = "Distinct source citation strings for autocomplete", body = SourceAutocompleteResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_event_sources(
    RequireAuth(_auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<SourceSearchQuery>,
) -> Result<Json<SourceAutocompleteResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let q_norm = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_search_tokens);

    let mut seen = HashSet::new();
    let mut items = Vec::new();

    for ev in &store.events {
        if let Some(ref src) = ev.source_text {
            let trimmed = src.trim();
            if !trimmed.is_empty() && seen.insert(trimmed.to_lowercase()) {
                if let Some(ref qn) = q_norm {
                    let src_norm = normalize_search_tokens(trimmed);
                    if !src_norm.contains(qn) {
                        continue;
                    }
                }
                items.push(trimmed.to_string());
            }
        }
    }

    Ok(Json(SourceAutocompleteResponse { items }))
}
