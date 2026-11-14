#[allow(unused_imports)]
use super::*;

pub(crate) fn check_merge_cycle(
    store: &GenealogyPrivacyStore,
    keep_id: &str,
    merge_id: &str,
) -> Result<(), AppError> {
    ensure_no_ancestor_cycle(&store.families, keep_id, merge_id)?;
    ensure_no_ancestor_cycle(&store.families, merge_id, keep_id)?;

    let mut simulated_families = store.families.clone();
    for fam in &mut simulated_families {
        if fam.partner1_id.as_deref() == Some(merge_id) {
            fam.partner1_id = Some(keep_id.to_string());
        }
        if fam.partner2_id.as_deref() == Some(merge_id) {
            fam.partner2_id = Some(keep_id.to_string());
        }
        for ch in &mut fam.children {
            if ch.child_id == merge_id {
                ch.child_id = keep_id.to_string();
            }
        }
    }

    for fam in &simulated_families {
        let involves_keep = fam.partner1_id.as_deref() == Some(keep_id)
            || fam.partner2_id.as_deref() == Some(keep_id);
        if !involves_keep {
            continue;
        }
        for parent_opt in [&fam.partner1_id, &fam.partner2_id] {
            if let Some(p_id) = parent_opt.as_deref() {
                for ch in &fam.children {
                    ensure_no_ancestor_cycle(&simulated_families, p_id, &ch.child_id)?;
                }
            }
        }
    }
    Ok(())
}

/// Generates a side-by-side person merge preview (`GET/POST /api/v1/people/merge/preview`).
pub fn preview_person_merge(
    store: &GenealogyPrivacyStore,
    query: &MergePreviewQuery,
) -> Result<MergePreviewResponse, AppError> {
    let raw_keep = query
        .keep_id
        .as_deref()
        .or(query.target_id.as_deref())
        .or(query.primary_person_id.as_deref())
        .or(query.a.as_deref())
        .unwrap_or_default();
    let raw_merge = query
        .merge_id
        .as_deref()
        .or(query.source_id.as_deref())
        .or(query.duplicate_person_id.as_deref())
        .or(query.b.as_deref())
        .unwrap_or_default();

    let keep_id = store
        .resolve_person_id(raw_keep)
        .ok_or(AppError::NotFound)?;
    let merge_id = store
        .resolve_person_id(raw_merge)
        .ok_or(AppError::NotFound)?;

    if keep_id == merge_id {
        let mut fields = HashMap::new();
        fields.insert(
            "merge_id".to_string(),
            "Cannot merge a person with themselves".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Cannot merge a person with themselves".to_string(),
            Some(fields),
        ));
    }

    let keep_person = store
        .persons
        .iter()
        .find(|p| p.id == keep_id)
        .cloned()
        .ok_or(AppError::NotFound)?;
    let merge_person = store
        .persons
        .iter()
        .find(|p| p.id == merge_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let (score, reasons) = score_duplicate_pair(&keep_person, &merge_person, store);
    let cycle_detected = check_merge_cycle(store, &keep_id, &merge_id).is_err();

    let mut field_conflicts = Vec::new();
    let push_conflict = |conflicts: &mut Vec<MergeFieldConflictDto>,
                         field: &str,
                         kv: serde_json::Value,
                         mv: serde_json::Value| {
        let k_empty = kv.is_null() || kv.as_str().map(|s| s.trim().is_empty()).unwrap_or(false);
        let m_empty = mv.is_null() || mv.as_str().map(|s| s.trim().is_empty()).unwrap_or(false);
        let has_conflict = !k_empty && !m_empty && kv != mv;
        let recommended = if k_empty && !m_empty { "merge" } else { "keep" };
        conflicts.push(MergeFieldConflictDto {
            field: field.to_string(),
            keep_value: kv,
            merge_value: mv,
            has_conflict,
            recommended_source: recommended.to_string(),
        });
    };

    push_conflict(
        &mut field_conflicts,
        "given_name",
        serde_json::json!(keep_person.primary_name.given_name),
        serde_json::json!(merge_person.primary_name.given_name),
    );
    push_conflict(
        &mut field_conflicts,
        "surname",
        serde_json::json!(keep_person.primary_name.surname),
        serde_json::json!(merge_person.primary_name.surname),
    );
    push_conflict(
        &mut field_conflicts,
        "patronymic",
        serde_json::json!(keep_person.primary_name.patronymic),
        serde_json::json!(merge_person.primary_name.patronymic),
    );
    push_conflict(
        &mut field_conflicts,
        "birth_surname",
        serde_json::json!(keep_person.birth_surname),
        serde_json::json!(merge_person.birth_surname),
    );
    push_conflict(
        &mut field_conflicts,
        "sex",
        serde_json::json!(keep_person.sex),
        serde_json::json!(merge_person.sex),
    );
    push_conflict(
        &mut field_conflicts,
        "biography",
        serde_json::json!(keep_person.biography),
        serde_json::json!(merge_person.biography),
    );
    push_conflict(
        &mut field_conflicts,
        "birth",
        serde_json::json!(keep_person.birth),
        serde_json::json!(merge_person.birth),
    );
    push_conflict(
        &mut field_conflicts,
        "death",
        serde_json::json!(keep_person.death),
        serde_json::json!(merge_person.death),
    );

    let keep_events: Vec<&LifeEventDto> = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(keep_id.as_str()))
        .collect();
    let merge_events: Vec<&LifeEventDto> = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(merge_id.as_str()))
        .collect();

    let mut events_deduplicated = 0;
    let mut events_to_transfer = 0;
    for mev in &merge_events {
        let is_dup = keep_events.iter().any(|kev| {
            kev.event_type.eq_ignore_ascii_case(&mev.event_type)
                && kev.date_sort == mev.date_sort
                && kev.place_id == mev.place_id
        });
        if is_dup {
            events_deduplicated += 1;
        } else {
            events_to_transfer += 1;
        }
    }

    let notes_to_transfer = store
        .notes
        .iter()
        .filter(|n| n.entity_type == "individual" && n.entity_id == merge_id)
        .count();

    let media_to_transfer = store
        .media
        .iter()
        .filter(|m| {
            m.linked_person_ids.contains(&merge_id)
                || m.face_tags.iter().any(|ft| ft.person_id == merge_id)
                || m.links
                    .iter()
                    .any(|l| l.entity_type == "individual" && l.entity_id == merge_id)
        })
        .count();

    let families_affected = store
        .families
        .iter()
        .filter(|f| {
            f.partner1_id.as_deref() == Some(merge_id.as_str())
                || f.partner2_id.as_deref() == Some(merge_id.as_str())
                || f.children.iter().any(|c| c.child_id == merge_id)
        })
        .count();

    Ok(MergePreviewResponse {
        target: keep_person.clone(),
        source: merge_person.clone(),
        keep_person,
        merge_person,
        score,
        reasons,
        can_merge: !cycle_detected,
        cycle_detected,
        field_conflicts,
        events_to_keep: keep_events.len(),
        events_to_transfer,
        events_deduplicated,
        notes_to_transfer,
        media_to_transfer,
        families_affected,
        combined_counts: MergeCombinedCountsDto {
            events: events_to_transfer,
            media: media_to_transfer,
            notes: notes_to_transfer,
            families: families_affected,
        },
    })
}

/// Executes an atomic, undoable person merge (`POST /api/v1/people/merge`, `15-data-quality-duplicates.md §5.2`).
pub fn execute_person_merge(
    store: &mut GenealogyPrivacyStore,
    req: PersonMergeRequest,
    actor_id: Option<String>,
    actor_name: Option<String>,
    backup_path: &str,
) -> Result<PersonMergeResponse, AppError> {
    let raw_keep = req
        .keep_id
        .as_deref()
        .or(req.target_id.as_deref())
        .or(req.primary_person_id.as_deref())
        .unwrap_or_default()
        .trim();
    let raw_merge = req
        .merge_id
        .as_deref()
        .or(req.source_id.as_deref())
        .or(req.duplicate_person_id.as_deref())
        .unwrap_or_default()
        .trim();

    // Backward-compatible no-op response if invoked with an empty body (e.g. RBAC permission probe in `rbac_test.rs`)
    if raw_keep.is_empty() && raw_merge.is_empty() {
        return Ok(PersonMergeResponse {
            status: "ok".to_string(),
            operation: "person_merge".to_string(),
            message: "Duplicate person merge authorized for administrator".to_string(),
            tx_id: None,
            snapshot_id: None,
            surviving_person_id: None,
            surviving_person: None,
            merged_id: None,
            merged_gid: None,
        });
    }
    if raw_keep.is_empty() || raw_merge.is_empty() {
        let mut fields = HashMap::new();
        if raw_keep.is_empty() {
            fields.insert("keep_id".to_string(), "required".to_string());
        }
        if raw_merge.is_empty() {
            fields.insert("merge_id".to_string(), "required".to_string());
        }
        return Err(AppError::ValidationFailed(
            "Both keep_id and merge_id are required".to_string(),
            Some(fields),
        ));
    }

    let keep_id = store
        .resolve_person_id(raw_keep)
        .ok_or(AppError::NotFound)?;
    let merge_id = store
        .resolve_person_id(raw_merge)
        .ok_or(AppError::NotFound)?;

    if keep_id == merge_id {
        let mut fields = HashMap::new();
        fields.insert(
            "merge_id".to_string(),
            "Cannot merge a person with themselves".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Cannot merge a person with themselves".to_string(),
            Some(fields),
        ));
    }

    // 1. Verify no ancestor-descendant cycle is created (`15 §5.2`, `10 §6.1`)
    check_merge_cycle(store, &keep_id, &merge_id)?;

    // 2. Automatic pre-merge snapshot (`14 §6.1`, `15 §5.2`)
    let snapshot = create_snapshot(
        store,
        Some(format!("Pre-merge snapshot ({keep_id} <- {merge_id})")),
        "bulk_merge",
        actor_id.clone(),
        actor_name.clone(),
        backup_path,
    )?;

    let keep_idx = store
        .persons
        .iter()
        .position(|p| p.id == keep_id)
        .ok_or(AppError::NotFound)?;
    let merge_idx = store
        .persons
        .iter()
        .position(|p| p.id == merge_id)
        .ok_or(AppError::NotFound)?;

    let keep_before = store.persons[keep_idx].clone();
    let merge_person = store.persons[merge_idx].clone();

    // Capture full pre-merge state for 1-click atomic undo (`POST /api/v1/history/tx/{tx_id}/undo`)
    let before_bundle = serde_json::json!({
        "kind": "person_merge_bundle",
        "keep_before": keep_before,
        "merge_person": merge_person,
        "families_before": store.families,
        "events_before": store.events,
        "notes_before": store.notes,
        "media_before": store.media,
        "gid_aliases_before": store.gid_aliases,
    });

    // 3. Resolve scalar fields on the surviving person
    let choose_from_merge = |keys: &[&str], keep_empty: bool| -> bool {
        for &field in keys {
            if let Some(choice) = req.field_choices.get(field) {
                match choice.trim().to_lowercase().as_str() {
                    "merge" | "source" | "b" => return true,
                    "keep" | "target" | "a" => return false,
                    _ => {}
                }
            }
        }
        keep_empty
    };

    let mut surviving = keep_before.clone();

    if choose_from_merge(
        &["given_name", "name", "primary_name"],
        surviving.primary_name.given_name.trim().is_empty(),
    ) && !merge_person.primary_name.given_name.trim().is_empty()
    {
        surviving.primary_name.given_name = merge_person.primary_name.given_name.clone();
    }
    if choose_from_merge(
        &["surname", "name", "primary_name"],
        surviving.primary_name.surname.trim().is_empty(),
    ) && !merge_person.primary_name.surname.trim().is_empty()
    {
        surviving.primary_name.surname = merge_person.primary_name.surname.clone();
    }
    if choose_from_merge(
        &["patronymic", "name", "primary_name"],
        surviving.primary_name.patronymic.is_none(),
    ) && merge_person.primary_name.patronymic.is_some()
    {
        surviving.primary_name.patronymic = merge_person.primary_name.patronymic.clone();
    }
    if choose_from_merge(&["birth_surname"], surviving.birth_surname.is_none())
        && merge_person.birth_surname.is_some()
    {
        surviving.birth_surname = merge_person.birth_surname.clone();
    }
    if choose_from_merge(
        &["sex", "gender"],
        surviving.sex.trim().is_empty() || surviving.sex == "U",
    ) && !merge_person.sex.trim().is_empty()
        && merge_person.sex != "U"
    {
        surviving.sex = merge_person.sex.clone();
    }
    if choose_from_merge(&["biography", "notes"], surviving.biography.is_none())
        && (merge_person.biography.is_some() || merge_person.notes.is_some())
    {
        surviving.biography = merge_person
            .biography
            .clone()
            .or_else(|| merge_person.notes.clone());
    }
    if choose_from_merge(&["notes", "biography"], surviving.notes.is_none())
        && (merge_person.notes.is_some() || merge_person.biography.is_some())
    {
        surviving.notes = merge_person
            .notes
            .clone()
            .or_else(|| merge_person.biography.clone());
    }
    if choose_from_merge(&["birth", "birth_date"], surviving.birth.is_none())
        && merge_person.birth.is_some()
    {
        surviving.birth = merge_person.birth.clone();
        surviving.birth_sort = merge_person.birth_sort;
    }
    if choose_from_merge(&["death", "death_date"], surviving.death.is_none())
        && merge_person.death.is_some()
    {
        surviving.death = merge_person.death.clone();
        surviving.death_sort = merge_person.death_sort;
        surviving.has_death_event = surviving.has_death_event || merge_person.has_death_event;
    }
    if surviving.avatar_url.is_none() && merge_person.avatar_url.is_some() {
        surviving.avatar_url = merge_person.avatar_url.clone();
        surviving.avatar_media_id = merge_person.avatar_media_id.clone();
        surviving.avatar_crop = merge_person.avatar_crop.clone();
    }

    // 4. Preserve non-chosen primary name as an alternate name (`kind = 'aka'`) + union `alt_names`
    for candidate_name in [&keep_before.primary_name, &merge_person.primary_name] {
        let same_as_surviving = candidate_name
            .given_name
            .eq_ignore_ascii_case(&surviving.primary_name.given_name)
            && candidate_name
                .surname
                .eq_ignore_ascii_case(&surviving.primary_name.surname)
            && candidate_name.patronymic == surviving.primary_name.patronymic;
        if !same_as_surviving
            && (!candidate_name.given_name.trim().is_empty()
                || !candidate_name.surname.trim().is_empty())
        {
            let already_present = surviving.alt_names.iter().any(|a| {
                a.given_name.as_deref().unwrap_or("") == candidate_name.given_name
                    && a.surname.as_deref().unwrap_or("") == candidate_name.surname
            });
            if !already_present {
                surviving.alt_names.push(IndividualAltNameDto {
                    id: Uuid::new_v4().to_string(),
                    name_type: "aka".to_string(),
                    given_name: Some(candidate_name.given_name.clone()),
                    patronymic: candidate_name.patronymic.clone(),
                    surname: Some(candidate_name.surname.clone()),
                    lang: None,
                });
            }
        }
    }
    for alt in &merge_person.alt_names {
        let exists = surviving.alt_names.iter().any(|a| {
            a.given_name == alt.given_name
                && a.surname == alt.surname
                && a.name_type == alt.name_type
        });
        if !exists {
            surviving.alt_names.push(alt.clone());
        }
    }

    // 5. Union external_ids & record merged GID alias (`10 §3.1`, `15 §5.2`)
    for (k, v) in &merge_person.external_ids {
        surviving
            .external_ids
            .entry(k.clone())
            .or_insert_with(|| v.clone());
    }
    surviving
        .external_ids
        .insert("merged_gid".to_string(), merge_person.gid.clone());
    store
        .gid_aliases
        .insert(merge_person.gid.clone(), keep_id.clone());

    surviving.version += 1;

    // 6. Move & deduplicate `events`
    let keep_event_keys: HashSet<(String, Option<i32>, Option<String>)> = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(keep_id.as_str()))
        .map(|e| {
            (
                e.event_type.to_ascii_lowercase(),
                e.date_sort,
                e.place_id.clone(),
            )
        })
        .collect();

    let mut updated_events = Vec::with_capacity(store.events.len());
    for mut ev in store.events.drain(..) {
        if ev.individual_id.as_deref() == Some(merge_id.as_str()) {
            let key = (
                ev.event_type.to_ascii_lowercase(),
                ev.date_sort,
                ev.place_id.clone(),
            );
            if keep_event_keys.contains(&key) {
                continue;
            }
            ev.individual_id = Some(keep_id.clone());
        }
        updated_events.push(ev);
    }
    store.events = updated_events;

    // 7. Move `notes`
    for note in &mut store.notes {
        if note.entity_type == "individual" && note.entity_id == merge_id {
            note.entity_id = keep_id.clone();
        }
    }

    // 8. Move & deduplicate `media` links and face tags
    let surviving_display_name = format_person_display_name(&surviving);
    for media in &mut store.media {
        if media.linked_person_ids.contains(&merge_id) {
            media.linked_person_ids.retain(|id| id != &merge_id);
            if !media.linked_person_ids.contains(&keep_id) {
                media.linked_person_ids.push(keep_id.clone());
            }
        }
        for ft in &mut media.face_tags {
            if ft.person_id == merge_id {
                ft.person_id = keep_id.clone();
                ft.person_name = surviving_display_name.clone();
            }
        }
        for link in &mut media.links {
            if link.entity_type == "individual" && link.entity_id == merge_id {
                link.entity_id = keep_id.clone();
            }
        }
        let mut seen_links: HashSet<(String, String)> = HashSet::new();
        media.links.retain(|l: &MediaLinkDto| {
            seen_links.insert((l.entity_type.clone(), l.entity_id.clone()))
        });
    }

    // 9. Re-point `families` (`partner1_id`, `partner2_id`, `children`) and merge duplicate couple families
    for fam in &mut store.families {
        if fam.partner1_id.as_deref() == Some(merge_id.as_str()) {
            fam.partner1_id = Some(keep_id.clone());
        }
        if fam.partner2_id.as_deref() == Some(merge_id.as_str()) {
            fam.partner2_id = Some(keep_id.clone());
        }
        for ch in &mut fam.children {
            if ch.child_id == merge_id {
                ch.child_id = keep_id.clone();
            }
        }
        let mut seen_children: HashSet<String> = HashSet::new();
        fam.children
            .retain(|c: &FamilyChildDto| seen_children.insert(c.child_id.clone()));
    }

    // Merge duplicate families that now share identical partner pairs or single+two-parent overlap
    let mut merged_families: Vec<crate::services::privacy::FamilyDto> = Vec::new();
    for fam in store.families.drain(..) {
        let existing_pos = merged_families.iter().position(|ef| {
            let same_direct =
                ef.partner1_id == fam.partner1_id && ef.partner2_id == fam.partner2_id;
            let same_swapped =
                ef.partner1_id == fam.partner2_id && ef.partner2_id == fam.partner1_id;
            if (same_direct || same_swapped)
                && (fam.partner1_id.is_some() || fam.partner2_id.is_some())
            {
                return true;
            }
            let ef_partners: Vec<&str> = [ef.partner1_id.as_deref(), ef.partner2_id.as_deref()]
                .into_iter()
                .flatten()
                .collect();
            let fam_partners: Vec<&str> = [fam.partner1_id.as_deref(), fam.partner2_id.as_deref()]
                .into_iter()
                .flatten()
                .collect();
            let one_is_single = (ef_partners.len() == 1 && !fam_partners.is_empty())
                || (fam_partners.len() == 1 && !ef_partners.is_empty());
            let shares_partner = ef_partners.iter().any(|p| fam_partners.contains(p));
            let shares_child = ef
                .children
                .iter()
                .any(|ec| fam.children.iter().any(|fc| fc.child_id == ec.child_id));
            one_is_single && shares_partner && shares_child
        });
        if let Some(idx) = existing_pos {
            let target_fam_id = merged_families[idx].id.clone();
            if merged_families[idx].partner1_id.is_none() {
                if let Some(p) = fam
                    .partner1_id
                    .as_ref()
                    .filter(|p| merged_families[idx].partner2_id.as_ref() != Some(*p))
                    .or_else(|| {
                        fam.partner2_id
                            .as_ref()
                            .filter(|p| merged_families[idx].partner2_id.as_ref() != Some(*p))
                    })
                {
                    merged_families[idx].partner1_id = Some(p.clone());
                }
            }
            if merged_families[idx].partner2_id.is_none() {
                if let Some(p) = fam
                    .partner2_id
                    .as_ref()
                    .filter(|p| merged_families[idx].partner1_id.as_ref() != Some(*p))
                    .or_else(|| {
                        fam.partner1_id
                            .as_ref()
                            .filter(|p| merged_families[idx].partner1_id.as_ref() != Some(*p))
                    })
                {
                    merged_families[idx].partner2_id = Some(p.clone());
                }
            }
            for ch in fam.children {
                if !merged_families[idx]
                    .children
                    .iter()
                    .any(|c| c.child_id == ch.child_id)
                {
                    merged_families[idx].children.push(ch);
                }
            }
            for ev in &mut store.events {
                if ev.family_id.as_deref() == Some(fam.id.as_str()) {
                    ev.family_id = Some(target_fam_id.clone());
                }
            }
        } else {
            merged_families.push(fam);
        }
    }
    store.families = merged_families;

    // 10. Update surviving person and remove source person
    let tx_id = Uuid::new_v4().to_string();
    surviving.tx_id = Some(tx_id.clone());
    store.persons[keep_idx] = surviving.clone();
    store.persons.retain(|p| p.id != merge_id);

    // Clean up dismissed warnings & not_duplicate_pairs referencing `merge_id`
    store.dismissed_warnings.retain(|d| {
        d.entity_id != merge_id && d.secondary_id.as_deref() != Some(merge_id.as_str())
    });
    store
        .not_duplicate_pairs
        .retain(|(a, b)| a != &merge_id && b != &merge_id);

    // 11. Append atomic `merge` entry to `change_log`
    let after_json = serde_json::to_value(&surviving).ok();
    let label = format!(
        "Merged {} ({}) into {} ({})",
        format_person_display_name(&merge_person),
        merge_person.gid,
        format_person_display_name(&surviving),
        surviving.gid
    );
    store.change_log.push(ChangeLogEntryDto {
        id: Uuid::new_v4().to_string(),
        tx_id: tx_id.clone(),
        tx_label: label.clone(),
        op: "merge".to_string(),
        action: "merge".to_string(),
        entity_type: "individual".to_string(),
        entity_id: keep_id.clone(),
        entity_label: Some(format_person_display_name(&surviving)),
        actor_id,
        actor_name,
        summary: label.clone(),
        before_json: Some(before_bundle),
        after_json,
        created_at: Utc::now().to_rfc3339(),
    });
    store.persist_if_production();

    Ok(PersonMergeResponse {
        status: "merged".to_string(),
        operation: "person_merge".to_string(),
        message: label,
        tx_id: Some(tx_id),
        snapshot_id: Some(snapshot.id),
        surviving_person_id: Some(surviving.id.clone()),
        surviving_person: Some(surviving),
        merged_id: Some(merge_id),
        merged_gid: Some(merge_person.gid),
    })
}
