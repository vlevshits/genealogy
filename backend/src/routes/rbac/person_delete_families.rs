#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/persons/{id}/delete-impact",
    params(
        ("id" = String, Path, description = "Person UUID or GID")
    ),
    responses(
        (status = 200, description = "Preview of records affected by deleting this person", body = DeleteImpactDto),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_person_delete_impact(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteImpactDto>, AppError> {
    let store = state.privacy_store.read().await;
    let target = id.trim();
    let person = store
        .persons
        .iter()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let events_count = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(person.id.as_str()))
        .count();

    let notes_count = store
        .notes
        .iter()
        .filter(|n| n.entity_type == "individual" && n.entity_id == person.id)
        .count();

    let media_links_count = store
        .media
        .iter()
        .filter(|m| m.linked_person_ids.contains(&person.id))
        .count();

    let mut families_unlinked_count = 0usize;
    let mut empty_families_deleted_count = 0usize;

    for fam in &store.families {
        let is_p1 = fam.partner1_id.as_deref() == Some(person.id.as_str());
        let is_p2 = fam.partner2_id.as_deref() == Some(person.id.as_str());
        let is_child = fam.children.iter().any(|c| c.child_id == person.id);
        if is_p1 || is_p2 || is_child {
            families_unlinked_count += 1;
            let rem_p1 = if is_p1 {
                None
            } else {
                fam.partner1_id.as_ref()
            };
            let rem_p2 = if is_p2 {
                None
            } else {
                fam.partner2_id.as_ref()
            };
            let rem_children = fam
                .children
                .iter()
                .filter(|c| c.child_id != person.id)
                .count();
            if rem_p1.is_none() && rem_p2.is_none() && rem_children == 0 {
                empty_families_deleted_count += 1;
            }
        }
    }

    let display_name = format!(
        "{} {}",
        person.primary_name.given_name, person.primary_name.surname
    )
    .trim()
    .to_string();

    Ok(Json(DeleteImpactDto {
        person_id: person.id.clone(),
        gid: person.gid.clone(),
        display_name,
        events_count,
        families_unlinked_count,
        empty_families_deleted_count,
        notes_count,
        media_links_count,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/persons/{id}",
    params(
        ("id" = String, Path, description = "Person UUID or GID")
    ),
    responses(
        (status = 200, description = "Person safely deleted and snapshot logged to change_log", body = serde_json::Value),
        (status = 403, description = "Viewer role is read-only", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_person(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let person_idx = store
        .persons
        .iter()
        .position(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed_person = store.persons.remove(person_idx);
    let person_id = removed_person.id.clone();

    let mut removed_events = Vec::new();
    store.events.retain(|e| {
        if e.individual_id.as_deref() == Some(person_id.as_str()) {
            removed_events.push(e.clone());
            false
        } else {
            true
        }
    });

    let mut removed_notes = Vec::new();
    store.notes.retain(|n| {
        if n.entity_type == "individual" && n.entity_id == person_id {
            removed_notes.push(n.clone());
            false
        } else {
            true
        }
    });

    let families_before: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| {
            f.partner1_id.as_deref() == Some(person_id.as_str())
                || f.partner2_id.as_deref() == Some(person_id.as_str())
                || f.children.iter().any(|c| c.child_id == person_id)
        })
        .cloned()
        .collect();

    // Unlink from families and delete any family left with 0 partners and 0 children (`10 §6.2`)
    let mut empty_family_ids: HashSet<String> = HashSet::new();
    for fam in &mut store.families {
        let mut touched = false;
        if fam.partner1_id.as_deref() == Some(person_id.as_str()) {
            fam.partner1_id = None;
            touched = true;
        }
        if fam.partner2_id.as_deref() == Some(person_id.as_str()) {
            fam.partner2_id = None;
            touched = true;
        }
        let before_children = fam.children.len();
        fam.children.retain(|c| c.child_id != person_id);
        if fam.children.len() != before_children {
            touched = true;
        }
        if touched {
            fam.version += 1;
        }
        if fam.partner1_id.is_none() && fam.partner2_id.is_none() && fam.children.is_empty() {
            empty_family_ids.insert(fam.id.clone());
        }
    }

    if !empty_family_ids.is_empty() {
        store.families.retain(|f| !empty_family_ids.contains(&f.id));
        store.events.retain(|e| {
            e.family_id
                .as_ref()
                .is_none_or(|fid| !empty_family_ids.contains(fid))
        });
    }

    // Unlink from media assets
    for m in &mut store.media {
        m.linked_person_ids.retain(|pid| pid != &person_id);
        m.face_tags.retain(|ft| ft.person_id != person_id);
    }

    // Record full snapshot in change_log so deletion can be undone from History (`07 §4, §5 DoD #5`)
    let snapshot = DeletedPersonSnapshot {
        person: removed_person.clone(),
        events: removed_events,
        notes: removed_notes,
        families_before,
    };

    let entity_label = format_person_display_name(&removed_person);
    let summary = format!(
        "Deleted individual {} ({} {})",
        removed_person.gid,
        removed_person.primary_name.given_name,
        removed_person.primary_name.surname
    )
    .trim()
    .to_string();

    let entry = log_change(
        &mut store,
        None,
        "person.delete",
        "individual",
        &person_id,
        "delete",
        Some(entity_label),
        summary,
        serde_json::to_value(snapshot).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);
    drop(store);

    // Match ON DELETE SET NULL for users.person_id / users.home_person_id and default_home_person_id
    if let Ok(deleted_uuid) = Uuid::parse_str(&person_id) {
        let mut auth_store = state.auth_store.write().await;
        for u in &mut auth_store.users {
            if u.person_id == Some(deleted_uuid) {
                u.person_id = None;
            }
            if u.home_person_id == Some(deleted_uuid) {
                u.home_person_id = None;
            }
        }
    }
    {
        let mut settings_guard = state.settings_override.write().await;
        if settings_guard.default_home_person_id.as_deref() == Some(person_id.as_str()) {
            settings_guard.default_home_person_id = None;
        }
    }

    Ok(Json(serde_json::json!({
        "status": "deleted",
        "deleted_id": person_id,
        "tx_id": entry.tx_id
    })))
}

#[utoipa::path(
    post,
    path = "/api/v1/history/{id}/restore",
    params(
        ("id" = String, Path, description = "Change log entry ID or deleted entity UUID/GID")
    ),
    responses(
        (status = 200, description = "Deleted or modified record restored from change_log", body = TreePersonNodeDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Change log entry or snapshot not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn restore_deleted_person(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let restored = restore_change(
        &mut store,
        &id,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
        current_year,
        age_threshold,
    )?;

    Ok(Json(restored))
}

#[utoipa::path(
    get,
    path = "/api/v1/families",
    responses(
        (status = 200, description = "List of family unions", body = FamilyListResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_families(
    caller: PublicOrAuth,
    State(state): State<AppState>,
) -> Result<Json<FamilyListResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let items: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| !f.is_private || access.can_view_private())
        .map(|f| enrich_family_dto_marriage(f.clone(), &store.events))
        .collect();
    let total = items.len();

    Ok(Json(FamilyListResponse { items, total }))
}

#[utoipa::path(
    get,
    path = "/api/v1/families/{id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID (e.g. F0100)")
    ),
    responses(
        (status = 200, description = "Family union record", body = FamilyDto),
        (status = 404, description = "Family not found or private", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_family(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<FamilyDto>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let fam = store
        .families
        .iter()
        .find(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    if fam.is_private && !access.can_view_private() {
        return Err(AppError::NotFound);
    }

    Ok(Json(enrich_family_dto_marriage(fam.clone(), &store.events)))
}

#[utoipa::path(
    post,
    path = "/api/v1/families",
    request_body = CreateFamilyRequest,
    responses(
        (status = 201, description = "Family union created", body = FamilyDto),
        (status = 400, description = "Validation error or ancestor cycle detected", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_family(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateFamilyRequest>,
) -> Result<(StatusCode, Json<FamilyDto>), AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;

    let p1_id = match req
        .partner1_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
        None => None,
    };
    let p2_id = match req
        .partner2_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
        None => None,
    };

    if p1_id.is_some() && p1_id == p2_id {
        let mut fields = HashMap::new();
        fields.insert("partner2_id".to_string(), "duplicate_partner".to_string());
        return Err(AppError::ValidationFailed(
            "partner1_id and partner2_id must be distinct individuals".to_string(),
            Some(fields),
        ));
    }

    let mut resolved_children = Vec::new();
    let mut input_children = req.children.unwrap_or_default();
    for cid in req.children_ids.unwrap_or_default() {
        if !input_children.iter().any(|c| c.child_id == cid) {
            input_children.push(FamilyChildDto {
                child_id: cid,
                rel_to_partner1: "biological".to_string(),
                rel_to_partner2: "biological".to_string(),
                sort_order: 0,
            });
        }
    }
    for (idx, mut ch) in input_children.into_iter().enumerate() {
        let resolved_cid =
            resolve_person_id_in_store(&store.persons, &ch.child_id).ok_or(AppError::NotFound)?;
        for parent_id in [p1_id.as_deref(), p2_id.as_deref()].into_iter().flatten() {
            ensure_no_ancestor_cycle(&store.families, parent_id, &resolved_cid)?;
        }
        ch.child_id = resolved_cid;
        if ch.sort_order == 0 {
            ch.sort_order = idx as i32;
        }
        resolved_children.push(ch);
    }

    let existing_gids: HashSet<String> = store.families.iter().map(|f| f.gid.clone()).collect();
    let gid = allocate_gid(
        'F',
        &mut store.next_family_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );

    let fam = FamilyDto {
        id: Uuid::new_v4().to_string(),
        gid,
        partner1_id: p1_id,
        partner2_id: p2_id,
        relationship_type: req
            .relationship_type
            .unwrap_or_else(|| "married".to_string()),
        is_private: req.is_private.unwrap_or(false),
        children: resolved_children,
        external_ids: HashMap::new(),
        version: 1,
        marriage_date: None,
        marriage_place: None,
    };

    store.families.push(fam.clone());
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    log_change(
        &mut store,
        None,
        "family.create",
        "family",
        &fam.id,
        "create",
        Some(fam.gid.clone()),
        format!("Created family union {}", fam.gid),
        None,
        serde_json::to_value(&fam).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok((StatusCode::CREATED, Json(fam)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/families/{id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID")
    ),
    request_body = UpdateFamilyRequest,
    responses(
        (status = 200, description = "Family union updated", body = FamilyDto),
        (status = 400, description = "Validation error or ancestor cycle detected", body = ErrorResponse),
        (status = 404, description = "Family not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_family(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateFamilyRequest>,
) -> Result<Json<FamilyDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let fam_idx = store
        .families
        .iter()
        .position(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.families[fam_idx].version,
        expected_version,
        &format!("Family {}", store.families[fam_idx].gid),
        serde_json::to_value(&store.families[fam_idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.families[fam_idx]).ok();

    let next_p1 = if let Some(ref raw) = req.partner1_id {
        if raw.trim().is_empty() {
            None
        } else {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
    } else {
        store.families[fam_idx].partner1_id.clone()
    };

    let next_p2 = if let Some(ref raw) = req.partner2_id {
        if raw.trim().is_empty() {
            None
        } else {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
    } else {
        store.families[fam_idx].partner2_id.clone()
    };

    if next_p1.is_some() && next_p1 == next_p2 {
        let mut fields = HashMap::new();
        fields.insert("partner2_id".to_string(), "duplicate_partner".to_string());
        return Err(AppError::ValidationFailed(
            "partner1_id and partner2_id must be distinct individuals".to_string(),
            Some(fields),
        ));
    }

    let next_children = if let Some(children) = req.children {
        let mut resolved = Vec::with_capacity(children.len());
        for mut ch in children {
            let cid = resolve_person_id_in_store(&store.persons, &ch.child_id)
                .ok_or(AppError::NotFound)?;
            ch.child_id = cid;
            resolved.push(ch);
        }
        resolved
    } else {
        store.families[fam_idx].children.clone()
    };

    // Validate cycle-free graph with other families + this updated family
    let other_families: Vec<FamilyDto> = store
        .families
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != fam_idx)
        .map(|(_, f)| f.clone())
        .collect();

    for ch in &next_children {
        for parent_id in [next_p1.as_deref(), next_p2.as_deref()]
            .into_iter()
            .flatten()
        {
            ensure_no_ancestor_cycle(&other_families, parent_id, &ch.child_id)?;
        }
    }

    let fam = &mut store.families[fam_idx];
    fam.partner1_id = next_p1;
    fam.partner2_id = next_p2;
    fam.children = next_children;
    if let Some(rel_type) = req.relationship_type {
        fam.relationship_type = rel_type;
    }
    if let Some(is_priv) = req.is_private {
        fam.is_private = is_priv;
    }
    fam.version += 1;

    let updated = fam.clone();
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    log_change(
        &mut store,
        None,
        "family.update",
        "family",
        &updated.id,
        "update",
        Some(updated.gid.clone()),
        format!("Updated family union {}", updated.gid),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/families/{id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID")
    ),
    responses(
        (status = 204, description = "Family union deleted while keeping partner and child individuals intact"),
        (status = 404, description = "Family not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_family(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let fam_idx = store
        .families
        .iter()
        .position(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed_fam = store.families.remove(fam_idx);
    store
        .events
        .retain(|e| e.family_id.as_deref() != Some(removed_fam.id.as_str()));

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    log_change(
        &mut store,
        None,
        "family.delete",
        "family",
        &removed_fam.id,
        "delete",
        Some(removed_fam.gid.clone()),
        format!("Deleted family union {}", removed_fam.gid),
        serde_json::to_value(&removed_fam).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(StatusCode::NO_CONTENT)
}
