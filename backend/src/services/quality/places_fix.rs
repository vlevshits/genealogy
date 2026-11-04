#[allow(unused_imports)]
use super::*;

/// Inspects all places in `store` and returns categorized hierarchy issues (`GET /api/v1/quality/places/analysis`).
pub fn analyze_place_hierarchies(
    store: &GenealogyPrivacyStore,
    query: &PlaceHierarchyAnalysisQuery,
) -> PlaceHierarchyAnalysisResponseDto {
    let include_dismissed = query.include_dismissed.unwrap_or(false);
    let min_conf = query.min_confidence.unwrap_or(0);
    let type_filter = query
        .issue_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let mut all_issues = Vec::new();
    for place in &store.places {
        if let Some(issue) = analyze_single_place_issue(place, store, include_dismissed) {
            all_issues.push(issue);
        }
    }

    all_issues.sort_by(|a, b| {
        b.confidence
            .cmp(&a.confidence)
            .then_with(|| b.events_count.cmp(&a.events_count))
            .then_with(|| a.place_gid.cmp(&b.place_gid))
    });

    let mut counts = PlaceHierarchyAnalysisCountsDto {
        total_places: store.places.len(),
        total_issues: all_issues.len(),
        ..Default::default()
    };

    for iss in &all_issues {
        if iss.confidence >= 85 {
            counts.high_confidence_count += 1;
        } else if iss.confidence >= 60 {
            counts.medium_confidence_count += 1;
        } else {
            counts.low_confidence_count += 1;
        }
        match iss.issue_type.as_str() {
            "unparsed_path" => counts.unparsed_paths_count += 1,
            "orphan_hierarchy" => counts.orphan_places_count += 1,
            "type_mismatch" => counts.type_mismatches_count += 1,
            "duplicate_node" => counts.duplicate_nodes_count += 1,
            _ => {}
        }
    }

    let filtered: Vec<PlaceHierarchyIssueDto> = all_issues
        .into_iter()
        .filter(|iss| {
            if iss.confidence < min_conf {
                return false;
            }
            if let Some(tf) = type_filter {
                if !iss.issue_type.eq_ignore_ascii_case(tf)
                    && !iss.rule_code.eq_ignore_ascii_case(tf)
                {
                    return false;
                }
            }
            true
        })
        .collect();

    let total = filtered.len();
    PlaceHierarchyAnalysisResponseDto {
        status: "ok".to_string(),
        counts,
        items: filtered.clone(),
        issues: filtered,
        total,
    }
}

/// Returns a detailed hierarchy preview for a single place (`GET /api/v1/quality/places/preview?id=`).
pub fn preview_place_hierarchy_issue(
    store: &GenealogyPrivacyStore,
    raw_id: &str,
) -> Result<PlaceHierarchyIssueDto, AppError> {
    let place_id = store.resolve_place_id(raw_id).ok_or(AppError::NotFound)?;
    let place = store
        .places
        .iter()
        .find(|p| p.id == place_id)
        .ok_or(AppError::NotFound)?;

    if let Some(issue) = analyze_single_place_issue(place, store, true) {
        return Ok(issue);
    }

    // Even if the place already has a clean hierarchy, return a preview representation
    let mut chain = if let Some(ref pid) = place.parent_id {
        build_ancestor_steps_for_parent(&store.places, pid)
    } else {
        vec![]
    };
    chain.push(ProposedHierarchyStepDto {
        name: place.name.clone(),
        place_type: place.place_type.clone(),
        existing_place_id: Some(place.id.clone()),
        existing_place_gid: Some(place.gid.clone()),
        is_new: false,
    });
    let parent_name = place
        .parent_id
        .as_deref()
        .and_then(|pid| store.places.iter().find(|p| p.id == pid))
        .map(|p| p.full_name.clone());

    Ok(PlaceHierarchyIssueDto {
        id: format!("PLACE_PREVIEW:{}", place.id),
        place_id: place.id.clone(),
        place_gid: place.gid.clone(),
        issue_type: "none".to_string(),
        rule_code: "NONE".to_string(),
        severity: "info".to_string(),
        confidence: 100,
        confidence_tier: "high".to_string(),
        current_name: place.name.clone(),
        current_full_name: place.full_name.clone(),
        current_place_type: place.place_type.clone(),
        current_parent_id: place.parent_id.clone(),
        current_parent_name: parent_name.clone(),
        latitude: place.latitude,
        longitude: place.longitude,
        events_count: place.events_count,
        children_count: place.children_count,
        proposed_leaf_name: place.name.clone(),
        proposed_clean_name: place.name.clone(),
        proposed_place_type: place.place_type.clone(),
        proposed_parent_id: place.parent_id.clone(),
        proposed_parent_name: parent_name,
        proposed_full_name: place.full_name.clone(),
        proposed_chain: chain,
        parent_candidates: vec![],
        consolidate_with_place_id: None,
        consolidate_with_name: None,
        distance_km: None,
        reason: "Place hierarchy is already structured.".to_string(),
        preserve_alt_name: None,
    })
}

pub(crate) fn fix_single_place_in_tx(
    store: &mut GenealogyPrivacyStore,
    req: &FixPlaceHierarchyRequest,
    tx_id: &str,
    tx_label: &str,
    actor_id: Option<String>,
    actor_name: Option<String>,
) -> Result<(PlaceSummaryDto, Vec<PlaceSummaryDto>, bool, Option<String>), AppError> {
    let raw_id = req
        .place_id
        .as_deref()
        .or(req.id.as_deref())
        .unwrap_or_default()
        .trim();
    let place_id = store.resolve_place_id(raw_id).ok_or(AppError::NotFound)?;

    let place_snapshot = store
        .places
        .iter()
        .find(|p| p.id == place_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let analyzed_issue = analyze_single_place_issue(&place_snapshot, store, true);

    let mut created_parents: Vec<PlaceSummaryDto> = Vec::new();
    let mut resolved_parent_id: Option<String> = None;

    // 1. Resolve explicit parent_id, parent_path, or analyzed proposed_chain ancestors
    if let Some(ref explicit_parent_raw) = req.parent_id {
        if !explicit_parent_raw.trim().is_empty() {
            let pid = store
                .resolve_place_id(explicit_parent_raw)
                .ok_or(AppError::NotFound)?;
            ensure_no_place_cycle(&store.places, &place_id, Some(&pid))?;
            resolved_parent_id = Some(pid);
        }
    } else if let Some(ref parent_path_raw) = req.parent_path {
        let segs = split_jurisdiction_segments(parent_path_raw);
        let mut curr_pid: Option<String> = None;
        for (depth_from_top, seg) in segs.iter().rev().enumerate() {
            let (clean_seg, inferred_opt) = infer_place_type_and_clean_name(seg);
            let seg_type = inferred_opt.unwrap_or_else(|| {
                if depth_from_top == 0 && segs.len() >= 2 {
                    "country".to_string()
                } else if depth_from_top <= 1 {
                    "region".to_string()
                } else {
                    "district".to_string()
                }
            });
            if let Some(existing) = find_matching_place_in_store(
                &store.places,
                &clean_seg,
                curr_pid.as_deref(),
                &place_id,
            ) {
                curr_pid = Some(existing.id.clone());
            } else {
                let existing_gids: HashSet<String> =
                    store.places.iter().map(|p| p.gid.clone()).collect();
                let gid = allocate_gid('P', &mut store.next_place_gid_seq, None, &existing_gids);
                let new_id = Uuid::new_v4().to_string();
                let new_place = PlaceSummaryDto {
                    id: new_id.clone(),
                    gid: gid.clone(),
                    name: clean_seg.clone(),
                    place_type: seg_type,
                    parent_id: curr_pid.clone(),
                    full_name: clean_seg.clone(),
                    country: None,
                    latitude: None,
                    longitude: None,
                    events_count: 0,
                    children_count: 0,
                    alt_names: vec![],
                    version: 1,
                };
                store.places.push(new_place.clone());
                recompute_all_places_metadata(store);
                let created_after = store
                    .places
                    .iter()
                    .find(|p| p.id == new_id)
                    .cloned()
                    .unwrap_or(new_place);
                log_change(
                    store,
                    Some(tx_id.to_string()),
                    tx_label,
                    "place",
                    &new_id,
                    "create",
                    Some(format!("{} ({})", created_after.name, created_after.gid)),
                    format!(
                        "Created parent jurisdiction {} ({})",
                        created_after.full_name, created_after.gid
                    ),
                    None,
                    serde_json::to_value(&created_after).ok(),
                    actor_id.clone(),
                    actor_name.clone(),
                );
                created_parents.push(created_after);
                curr_pid = Some(new_id);
            }
        }
        if let Some(ref pid) = curr_pid {
            ensure_no_place_cycle(&store.places, &place_id, Some(pid))?;
        }
        resolved_parent_id = curr_pid;
    } else if let Some(ref issue) = analyzed_issue {
        if issue.proposed_chain.len() >= 2 {
            let ancestor_steps = &issue.proposed_chain[..issue.proposed_chain.len() - 1];
            let mut curr_pid: Option<String> = None;
            for step in ancestor_steps {
                if let Some(ref ex_id) = step.existing_place_id {
                    if store.places.iter().any(|p| &p.id == ex_id) {
                        curr_pid = Some(ex_id.clone());
                        continue;
                    }
                }
                if let Some(existing) = find_matching_place_in_store(
                    &store.places,
                    &step.name,
                    curr_pid.as_deref(),
                    &place_id,
                ) {
                    let found_id = existing.id.clone();
                    // If existing top-level place had no parent_id yet and we now have a higher parent, link it
                    if existing.parent_id.is_none()
                        && curr_pid.is_some()
                        && ensure_no_place_cycle(&store.places, &found_id, curr_pid.as_deref())
                            .is_ok()
                    {
                        if let Some(ex_mut) = store.places.iter_mut().find(|p| p.id == found_id) {
                            let before_val = serde_json::to_value(&*ex_mut).ok();
                            ex_mut.parent_id = curr_pid.clone();
                            ex_mut.version += 1;
                            let after_val = serde_json::to_value(&*ex_mut).ok();
                            let ex_name = ex_mut.name.clone();
                            let ex_gid = ex_mut.gid.clone();
                            log_change(
                                store,
                                Some(tx_id.to_string()),
                                tx_label,
                                "place",
                                &found_id,
                                "update",
                                Some(format!("{ex_name} ({ex_gid})")),
                                format!("Linked parent jurisdiction for {ex_name} ({ex_gid})"),
                                before_val,
                                after_val,
                                actor_id.clone(),
                                actor_name.clone(),
                            );
                        }
                    }
                    curr_pid = Some(found_id);
                } else {
                    let existing_gids: HashSet<String> =
                        store.places.iter().map(|p| p.gid.clone()).collect();
                    let gid =
                        allocate_gid('P', &mut store.next_place_gid_seq, None, &existing_gids);
                    let new_id = Uuid::new_v4().to_string();
                    let new_place = PlaceSummaryDto {
                        id: new_id.clone(),
                        gid: gid.clone(),
                        name: step.name.clone(),
                        place_type: step.place_type.clone(),
                        parent_id: curr_pid.clone(),
                        full_name: step.name.clone(),
                        country: None,
                        latitude: None,
                        longitude: None,
                        events_count: 0,
                        children_count: 0,
                        alt_names: vec![],
                        version: 1,
                    };
                    store.places.push(new_place.clone());
                    recompute_all_places_metadata(store);
                    let created_after = store
                        .places
                        .iter()
                        .find(|p| p.id == new_id)
                        .cloned()
                        .unwrap_or(new_place);
                    log_change(
                        store,
                        Some(tx_id.to_string()),
                        tx_label,
                        "place",
                        &new_id,
                        "create",
                        Some(format!("{} ({})", created_after.name, created_after.gid)),
                        format!(
                            "Created intermediate parent place {} ({})",
                            created_after.full_name, created_after.gid
                        ),
                        None,
                        serde_json::to_value(&created_after).ok(),
                        actor_id.clone(),
                        actor_name.clone(),
                    );
                    created_parents.push(created_after);
                    curr_pid = Some(new_id);
                }
            }
            resolved_parent_id = curr_pid;
        } else {
            resolved_parent_id = issue.proposed_parent_id.clone();
        }
    } else {
        resolved_parent_id = place_snapshot.parent_id.clone();
    }

    if let Some(ref pid) = resolved_parent_id {
        ensure_no_place_cycle(&store.places, &place_id, Some(pid))?;
    }

    let target_leaf_name = req
        .clean_name
        .as_deref()
        .or(req.proposed_leaf_name.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| {
            analyzed_issue
                .as_ref()
                .map(|i| i.proposed_leaf_name.clone())
        })
        .unwrap_or_else(|| infer_place_type_and_clean_name(&place_snapshot.name).0);

    let target_place_type = req
        .place_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .or_else(|| {
            analyzed_issue
                .as_ref()
                .map(|i| i.proposed_place_type.clone())
        })
        .unwrap_or_else(|| place_snapshot.place_type.clone());

    // 2. Check if consolidating into an existing duplicate target place under the same parent
    let explicit_consolidate_id = req
        .consolidate_with_place_id
        .as_deref()
        .and_then(|raw| store.resolve_place_id(raw))
        .or_else(|| {
            analyzed_issue
                .as_ref()
                .and_then(|i| i.consolidate_with_place_id.clone())
        })
        .or_else(|| {
            let norm_leaf = normalize_place_match_key(&target_leaf_name);
            store
                .places
                .iter()
                .find(|p| {
                    p.id != place_id
                        && p.parent_id == resolved_parent_id
                        && !p.name.contains(',')
                        && normalize_place_match_key(&p.name) == norm_leaf
                })
                .map(|p| p.id.clone())
        });

    if let Some(target_dup_id) = explicit_consolidate_id {
        if target_dup_id != place_id && store.places.iter().any(|p| p.id == target_dup_id) {
            ensure_no_place_cycle(&store.places, &target_dup_id, resolved_parent_id.as_deref())?;

            // Re-point events from `place_id` to `target_dup_id`
            let affected_event_ids: Vec<String> = store
                .events
                .iter()
                .filter(|e| e.place_id.as_deref() == Some(place_id.as_str()))
                .map(|e| e.id.clone())
                .collect();
            for eid in affected_event_ids {
                if let Some(ev) = store.events.iter_mut().find(|e| e.id == eid) {
                    let ev_before = serde_json::to_value(&*ev).ok();
                    ev.place_id = Some(target_dup_id.clone());
                    ev.version += 1;
                    let ev_after = serde_json::to_value(&*ev).ok();
                    let ev_gid = ev.gid.clone();
                    log_change(
                        store,
                        Some(tx_id.to_string()),
                        tx_label,
                        "event",
                        &eid,
                        "update",
                        Some(ev_gid),
                        format!("Re-pointed event place to {target_dup_id}"),
                        ev_before,
                        ev_after,
                        actor_id.clone(),
                        actor_name.clone(),
                    );
                }
            }

            // Re-point child places from `place_id` to `target_dup_id`
            let affected_child_place_ids: Vec<String> = store
                .places
                .iter()
                .filter(|p| {
                    p.parent_id.as_deref() == Some(place_id.as_str()) && p.id != target_dup_id
                })
                .map(|p| p.id.clone())
                .collect();
            for cpid in affected_child_place_ids {
                if let Some(ch_pl) = store.places.iter_mut().find(|p| p.id == cpid) {
                    let ch_before = serde_json::to_value(&*ch_pl).ok();
                    ch_pl.parent_id = Some(target_dup_id.clone());
                    ch_pl.version += 1;
                    let ch_after = serde_json::to_value(&*ch_pl).ok();
                    let ch_name = ch_pl.name.clone();
                    log_change(
                        store,
                        Some(tx_id.to_string()),
                        tx_label,
                        "place",
                        &cpid,
                        "update",
                        Some(ch_name),
                        format!("Re-pointed child place parent to {target_dup_id}"),
                        ch_before,
                        ch_after,
                        actor_id.clone(),
                        actor_name.clone(),
                    );
                }
            }

            // Re-point notes from `place_id` to `target_dup_id`
            let affected_note_ids: Vec<String> = store
                .notes
                .iter()
                .filter(|n| n.entity_type == "place" && n.entity_id == place_id)
                .map(|n| n.id.clone())
                .collect();
            for nid in affected_note_ids {
                if let Some(note) = store.notes.iter_mut().find(|n| n.id == nid) {
                    let n_before = serde_json::to_value(&*note).ok();
                    note.entity_id = target_dup_id.clone();
                    note.version += 1;
                    let n_after = serde_json::to_value(&*note).ok();
                    log_change(
                        store,
                        Some(tx_id.to_string()),
                        tx_label,
                        "note",
                        &nid,
                        "update",
                        Some("Place note".to_string()),
                        format!("Re-pointed place note to {target_dup_id}"),
                        n_before,
                        n_after,
                        actor_id.clone(),
                        actor_name.clone(),
                    );
                }
            }

            // Update surviving target place (`alt_names` + coordinates)
            if let Some(target_pl) = store.places.iter_mut().find(|p| p.id == target_dup_id) {
                let target_before = serde_json::to_value(&*target_pl).ok();
                if target_pl.latitude.is_none() && place_snapshot.latitude.is_some() {
                    target_pl.latitude = place_snapshot.latitude;
                    target_pl.longitude = place_snapshot.longitude;
                }
                if !place_snapshot.name.eq_ignore_ascii_case(&target_pl.name)
                    && !target_pl
                        .alt_names
                        .iter()
                        .any(|a| a.name.eq_ignore_ascii_case(&place_snapshot.name))
                {
                    target_pl.alt_names.push(PlaceAltNameDto {
                        id: Uuid::new_v4().to_string(),
                        name: place_snapshot.name.clone(),
                        lang: None,
                        note: Some("Original unparsed path".to_string()),
                    });
                }
                for alt in &place_snapshot.alt_names {
                    if !target_pl
                        .alt_names
                        .iter()
                        .any(|a| a.name.eq_ignore_ascii_case(&alt.name))
                    {
                        target_pl.alt_names.push(alt.clone());
                    }
                }
                target_pl.version += 1;
                let target_after = serde_json::to_value(&*target_pl).ok();
                let t_name = target_pl.name.clone();
                let t_gid = target_pl.gid.clone();
                log_change(
                    store,
                    Some(tx_id.to_string()),
                    tx_label,
                    "place",
                    &target_dup_id,
                    "update",
                    Some(format!("{t_name} ({t_gid})")),
                    format!(
                        "Consolidated duplicate place {} ({}) into {} ({})",
                        place_snapshot.name, place_snapshot.gid, t_name, t_gid
                    ),
                    target_before,
                    target_after,
                    actor_id.clone(),
                    actor_name.clone(),
                );
            }

            // Record GID alias & delete redundant source place
            store
                .gid_aliases
                .insert(place_snapshot.gid.clone(), target_dup_id.clone());
            store.places.retain(|p| p.id != place_id);
            log_change(
                store,
                Some(tx_id.to_string()),
                tx_label,
                "place",
                &place_id,
                "delete",
                Some(format!("{} ({})", place_snapshot.name, place_snapshot.gid)),
                format!(
                    "Removed redundant duplicate place {} ({}) after consolidation",
                    place_snapshot.name, place_snapshot.gid
                ),
                serde_json::to_value(&place_snapshot).ok(),
                None,
                actor_id,
                actor_name,
            );

            recompute_all_places_metadata(store);
            let surviving = store
                .places
                .iter()
                .find(|p| p.id == target_dup_id)
                .cloned()
                .ok_or(AppError::NotFound)?;
            return Ok((surviving, created_parents, true, Some(place_id)));
        }
    }

    // 3. Standard in-place hierarchy restructuring on `place_id`
    let place_idx = store
        .places
        .iter()
        .position(|p| p.id == place_id)
        .ok_or(AppError::NotFound)?;
    let before_json = serde_json::to_value(&store.places[place_idx]).ok();
    let original_name = store.places[place_idx].name.clone();

    if original_name != target_leaf_name {
        let already_in_alt = store.places[place_idx]
            .alt_names
            .iter()
            .any(|a| a.name.eq_ignore_ascii_case(&original_name));
        if !already_in_alt {
            store.places[place_idx].alt_names.push(PlaceAltNameDto {
                id: Uuid::new_v4().to_string(),
                name: original_name.clone(),
                lang: None,
                note: Some("Original unparsed path".to_string()),
            });
        }
        store.places[place_idx].name = target_leaf_name;
    }

    store.places[place_idx].place_type = target_place_type;
    store.places[place_idx].parent_id = resolved_parent_id;
    store.places[place_idx].version += 1;

    recompute_all_places_metadata(store);
    let updated_place = store.places[place_idx].clone();

    log_change(
        store,
        Some(tx_id.to_string()),
        tx_label,
        "place",
        &updated_place.id,
        "update",
        Some(format!("{} ({})", updated_place.name, updated_place.gid)),
        format!(
            "Restructured place hierarchy for '{}' -> '{}'",
            original_name, updated_place.full_name
        ),
        before_json,
        serde_json::to_value(&updated_place).ok(),
        actor_id,
        actor_name,
    );

    Ok((updated_place, created_parents, false, None))
}
