#[allow(unused_imports)]
use super::*;

/// Atomically commits a reconciled `StoryIntakeDraftDto` into the live family tree (`19-ai-family-story-intake-and-mcp.md §3.3`).
pub fn commit_story_draft(
    store: &mut GenealogyPrivacyStore,
    draft_id: &str,
    req: CommitStoryDraftRequest,
    caller_id: Option<String>,
    caller_name: Option<String>,
    current_year: i32,
    age_threshold: i32,
) -> Result<CommitStoryDraftResponse, AppError> {
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

    if store.is_demo_seed {
        store.persons.clear();
        store.is_demo_seed = false;
    }

    if draft.reconciliation_matches.is_empty() {
        reconcile_story_draft_in_place(store, &mut draft);
    }
    if let Some(overrides) = &req.match_overrides {
        apply_match_overrides(store, &mut draft, overrides);
    }

    let shared_tx_id = format!("tx-{}", Uuid::new_v4());
    let mut temp_to_real_person_id: HashMap<String, String> = HashMap::new();
    let mut created_person_ids = Vec::new();
    let mut updated_person_ids = Vec::new();
    let mut created_family_ids = Vec::new();
    let mut updated_family_ids = Vec::new();
    let mut created_event_ids = Vec::new();
    let mut created_place_ids = Vec::new();

    // 1. Process each draft person according to its reconciliation match
    for dp in &draft.extracted_data.persons {
        let match_decision = draft
            .reconciliation_matches
            .iter()
            .find(|m| m.draft_temp_id == dp.temp_id);

        let selected_action = match_decision
            .map(|m| m.selected_action.as_str())
            .unwrap_or("create_new");

        if selected_action == "skip" {
            continue;
        }

        let matched_target_id = if selected_action == "link_enrich" {
            match_decision
                .and_then(|m| m.matched_person_id.as_deref())
                .and_then(|pid| store.resolve_person_id(pid))
        } else {
            None
        };

        let existing_person_idx = matched_target_id.as_ref().and_then(|existing_id| {
            store
                .persons
                .iter()
                .position(|p| &p.id == existing_id)
                .map(|p_idx| (existing_id.clone(), p_idx))
        });

        if let Some((existing_id, p_idx)) = existing_person_idx {
            temp_to_real_person_id.insert(dp.temp_id.clone(), existing_id.clone());

            let before_person = store.persons[p_idx].clone();
            let mut changed = false;

            if store.persons[p_idx].primary_name.patronymic.is_none() {
                if let Some(pat) = &dp.patronymic {
                    if !pat.trim().is_empty() {
                        store.persons[p_idx].primary_name.patronymic = Some(pat.trim().to_string());
                        changed = true;
                    }
                }
            }
            if store.persons[p_idx].birth_surname.is_none() {
                if let Some(maiden) = &dp.maiden_name {
                    if !maiden.trim().is_empty() {
                        store.persons[p_idx].birth_surname = Some(maiden.trim().to_string());
                        changed = true;
                    }
                }
            }
            if store.persons[p_idx].primary_name.surname.trim().is_empty()
                && !dp.surname.trim().is_empty()
            {
                store.persons[p_idx].primary_name.surname = dp.surname.trim().to_string();
                changed = true;
            }
            let norm_dp_sex = crate::services::genealogy::normalize_sex(&dp.sex);
            if store.persons[p_idx].sex == "U" && matches!(norm_dp_sex.as_str(), "M" | "F") {
                store.persons[p_idx].sex = norm_dp_sex;
                changed = true;
            }
            if store.persons[p_idx].biography.is_none() {
                if let Some(notes) = &dp.biography_notes {
                    if !notes.trim().is_empty() {
                        store.persons[p_idx].biography = Some(notes.trim().to_string());
                        changed = true;
                    }
                }
            }

            let has_birt_event = store.events.iter().any(|e| {
                e.individual_id.as_deref() == Some(existing_id.as_str())
                    && (e.event_type.eq_ignore_ascii_case("BIRT")
                        || e.event_type.eq_ignore_ascii_case("birth"))
            });
            let dp_birt_date = dp
                .birth_date_text
                .clone()
                .or_else(|| dp.birth_year.map(|y| y.to_string()));

            if (dp_birt_date.is_some() || dp.birth_place.is_some()) && !has_birt_event {
                let place_id = dp.birth_place.as_ref().map(|bp| {
                    resolve_or_create_place_in_tx(
                        store,
                        bp,
                        &shared_tx_id,
                        &mut created_place_ids,
                        caller_id.clone(),
                        caller_name.clone(),
                    )
                });

                let ev_id = create_event_in_tx(
                    store,
                    &existing_id,
                    "birth",
                    dp_birt_date.clone(),
                    dp.birth_place.clone(),
                    place_id,
                    dp.grounding_quote.clone(),
                    &shared_tx_id,
                    caller_id.clone(),
                    caller_name.clone(),
                );
                created_event_ids.push(ev_id);
                changed = true;
            } else if store.persons[p_idx].birth.is_none()
                && (dp_birt_date.is_some() || dp.birth_place.is_some())
            {
                store.persons[p_idx].birth = Some(VitalEventSummaryDto {
                    date: dp_birt_date,
                    place: dp.birth_place.clone(),
                });
                changed = true;
            }

            if changed {
                store.persons[p_idx].version += 1;
                store.persons[p_idx].tx_id = Some(shared_tx_id.clone());
                let after_person = store.persons[p_idx].clone();
                let label = format_person_display_name(&after_person);
                log_change(
                    store,
                    Some(shared_tx_id.clone()),
                    "ai.story_commit",
                    "individual",
                    &existing_id,
                    "update",
                    Some(label.clone()),
                    format!("Enriched individual {label} from story draft"),
                    Some(serde_json::to_value(&before_person).unwrap_or_else(|_| json!({}))),
                    Some(serde_json::to_value(&after_person).unwrap_or_else(|_| json!({}))),
                    caller_id.clone(),
                    caller_name.clone(),
                );
                updated_person_ids.push(existing_id);
            }
            continue;
        }

        // Create net-new person
        let new_id = Uuid::new_v4().to_string();
        let new_gid = format!("I{:04}", store.next_gid_seq);
        store.next_gid_seq += 1;

        let birt_date = dp
            .birth_date_text
            .clone()
            .or_else(|| dp.birth_year.map(|y| y.to_string()));
        let deat_date = dp
            .death_date_text
            .clone()
            .or_else(|| dp.death_year.map(|y| y.to_string()));

        let birth_summary = if birt_date.is_some() || dp.birth_place.is_some() {
            Some(VitalEventSummaryDto {
                date: birt_date.clone(),
                place: dp.birth_place.clone(),
            })
        } else {
            None
        };

        let death_summary = if deat_date.is_some() || dp.death_place.is_some() {
            Some(VitalEventSummaryDto {
                date: deat_date.clone(),
                place: dp.death_place.clone(),
            })
        } else {
            None
        };

        let birth_sort = birt_date
            .as_deref()
            .and_then(|d| GenDate::parse(d).compute_sort_key());
        let death_sort = deat_date
            .as_deref()
            .and_then(|d| GenDate::parse(d).compute_sort_key());

        let new_person = TreePersonNodeDto {
            id: new_id.clone(),
            gid: new_gid,
            sex: crate::services::genealogy::normalize_sex(&dp.sex),
            is_living: dp.is_living && death_summary.is_none(),
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: if dp.given_name.trim().is_empty() {
                    "Unknown".to_string()
                } else {
                    dp.given_name.trim().to_string()
                },
                patronymic: dp.patronymic.clone(),
                surname: dp.surname.trim().to_string(),
            },
            birth_surname: dp.maiden_name.clone(),
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: dp.biography_notes.clone(),
            birth: birth_summary,
            death: death_summary.clone(),
            avatar_url: None,
            avatar_media_id: None,
            avatar_crop: None,
            notes: None,
            birth_sort,
            death_sort,
            has_death_event: death_summary.is_some(),
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: Some(shared_tx_id.clone()),
            generation: None,
            is_direct_lineage: None,
        };

        let label = format_person_display_name(&new_person);
        store.persons.push(new_person.clone());
        log_change(
            store,
            Some(shared_tx_id.clone()),
            "ai.story_commit",
            "individual",
            &new_id,
            "create",
            Some(label.clone()),
            format!("Created individual {label} from story draft"),
            None,
            Some(serde_json::to_value(&new_person).unwrap_or_else(|_| json!({}))),
            caller_id.clone(),
            caller_name.clone(),
        );
        created_person_ids.push(new_id.clone());
        temp_to_real_person_id.insert(dp.temp_id.clone(), new_id.clone());

        if birt_date.is_some() || dp.birth_place.is_some() {
            let place_id = dp.birth_place.as_ref().map(|bp| {
                resolve_or_create_place_in_tx(
                    store,
                    bp,
                    &shared_tx_id,
                    &mut created_place_ids,
                    caller_id.clone(),
                    caller_name.clone(),
                )
            });
            let ev_id = create_event_in_tx(
                store,
                &new_id,
                "birth",
                birt_date,
                dp.birth_place.clone(),
                place_id,
                dp.grounding_quote.clone(),
                &shared_tx_id,
                caller_id.clone(),
                caller_name.clone(),
            );
            created_event_ids.push(ev_id);
        }

        if deat_date.is_some() || dp.death_place.is_some() {
            let place_id = dp.death_place.as_ref().map(|dp_place| {
                resolve_or_create_place_in_tx(
                    store,
                    dp_place,
                    &shared_tx_id,
                    &mut created_place_ids,
                    caller_id.clone(),
                    caller_name.clone(),
                )
            });
            let ev_id = create_event_in_tx(
                store,
                &new_id,
                "death",
                deat_date,
                dp.death_place.clone(),
                place_id,
                dp.grounding_quote.clone(),
                &shared_tx_id,
                caller_id.clone(),
                caller_name.clone(),
            );
            created_event_ids.push(ev_id);
        }
    }

    // 2. Process unions (partner/spouse families)
    for union in &draft.extracted_data.unions {
        let Some(p1_id) = temp_to_real_person_id.get(&union.partner1_temp_id).cloned() else {
            continue;
        };
        let Some(p2_id) = temp_to_real_person_id.get(&union.partner2_temp_id).cloned() else {
            continue;
        };
        if p1_id == p2_id {
            continue;
        }

        let already_exists = store.families.iter().any(|f| {
            (f.partner1_id.as_deref() == Some(p1_id.as_str())
                && f.partner2_id.as_deref() == Some(p2_id.as_str()))
                || (f.partner1_id.as_deref() == Some(p2_id.as_str())
                    && f.partner2_id.as_deref() == Some(p1_id.as_str()))
        });
        if already_exists {
            continue;
        }

        let fam_id = Uuid::new_v4().to_string();
        let fam_gid = format!("F{:04}", store.next_family_gid_seq);
        store.next_family_gid_seq += 1;
        let new_fam = FamilyDto {
            id: fam_id.clone(),
            gid: fam_gid.clone(),
            partner1_id: Some(p1_id),
            partner2_id: Some(p2_id),
            relationship_type: crate::services::genealogy::normalize_relationship_type(
                &union.union_type,
            ),
            is_private: false,
            children: vec![],
            external_ids: HashMap::new(),
            version: 1,
            marriage_date: union.marriage_date_text.clone(),
            marriage_place: union.marriage_place.clone(),
        };
        store.families.push(new_fam.clone());
        log_change(
            store,
            Some(shared_tx_id.clone()),
            "ai.story_commit",
            "family",
            &fam_id,
            "create",
            Some(fam_gid),
            "Created family union from story draft".to_string(),
            None,
            Some(serde_json::to_value(&new_fam).unwrap_or_else(|_| json!({}))),
            caller_id.clone(),
            caller_name.clone(),
        );
        created_family_ids.push(fam_id);
    }

    // 3. Process parent_child_links grouped by child
    let mut parents_by_child: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for link in &draft.extracted_data.parent_child_links {
        let Some(parent_id) = temp_to_real_person_id.get(&link.parent_temp_id).cloned() else {
            continue;
        };
        let Some(child_id) = temp_to_real_person_id.get(&link.child_temp_id).cloned() else {
            continue;
        };
        if parent_id == child_id {
            continue;
        }
        let entry = parents_by_child.entry(child_id).or_default();
        if !entry.iter().any(|(pid, _)| pid == &parent_id) {
            entry.push((parent_id, link.pedigree.clone()));
        }
    }

    for (child_id, parent_links) in parents_by_child {
        if parent_links.is_empty() {
            continue;
        }

        // Check if child already has a parent family in store.families
        if let Some(fam_idx) = store
            .families
            .iter()
            .position(|f| f.children.iter().any(|c| c.child_id == child_id))
        {
            let before_fam = store.families[fam_idx].clone();
            let mut fam_changed = false;
            for (parent_id, _pedigree) in &parent_links {
                let already_partner = store.families[fam_idx].partner1_id.as_deref()
                    == Some(parent_id.as_str())
                    || store.families[fam_idx].partner2_id.as_deref() == Some(parent_id.as_str());
                if !already_partner {
                    if store.families[fam_idx].partner1_id.is_none() {
                        store.families[fam_idx].partner1_id = Some(parent_id.clone());
                        fam_changed = true;
                    } else if store.families[fam_idx].partner2_id.is_none() {
                        store.families[fam_idx].partner2_id = Some(parent_id.clone());
                        fam_changed = true;
                    }
                }
            }
            if fam_changed {
                let fam_id = store.families[fam_idx].id.clone();
                if created_family_ids.contains(&fam_id) {
                    let updated_fam = store.families[fam_idx].clone();
                    update_created_family_log(store, &shared_tx_id, &updated_fam);
                } else {
                    store.families[fam_idx].version += 1;
                    let after_fam = store.families[fam_idx].clone();
                    log_change(
                        store,
                        Some(shared_tx_id.clone()),
                        "ai.story_commit",
                        "family",
                        &fam_id,
                        "update",
                        Some(after_fam.gid.clone()),
                        "Updated family parents from story draft".to_string(),
                        Some(serde_json::to_value(&before_fam).unwrap_or_else(|_| json!({}))),
                        Some(serde_json::to_value(&after_fam).unwrap_or_else(|_| json!({}))),
                        caller_id.clone(),
                        caller_name.clone(),
                    );
                    if !updated_family_ids.contains(&fam_id) {
                        updated_family_ids.push(fam_id);
                    }
                }
            }
            continue;
        }

        // Check if a family for these parents already exists
        let p1_target = parent_links.first().map(|(pid, _)| pid.clone());
        let p2_target = parent_links.get(1).map(|(pid, _)| pid.clone());

        let existing_parent_fam_idx =
            store
                .families
                .iter()
                .position(|f| match (&p1_target, &p2_target) {
                    (Some(p1), Some(p2)) => {
                        (f.partner1_id.as_deref() == Some(p1.as_str())
                            && f.partner2_id.as_deref() == Some(p2.as_str()))
                            || (f.partner1_id.as_deref() == Some(p2.as_str())
                                && f.partner2_id.as_deref() == Some(p1.as_str()))
                    }
                    (Some(p1), None) => {
                        f.partner1_id.as_deref() == Some(p1.as_str())
                            || f.partner2_id.as_deref() == Some(p1.as_str())
                    }
                    _ => false,
                });

        let ped = crate::services::genealogy::normalize_pedigree_type(
            parent_links
                .first()
                .map(|(_, p)| p.as_str())
                .unwrap_or("birth"),
        );

        if let Some(fam_idx) = existing_parent_fam_idx {
            let before_fam = store.families[fam_idx].clone();
            if !store.families[fam_idx]
                .children
                .iter()
                .any(|c| c.child_id == child_id)
            {
                let sort_order = store.families[fam_idx].children.len() as i32;
                store.families[fam_idx].children.push(FamilyChildDto {
                    child_id: child_id.clone(),
                    sort_order,
                    rel_to_partner1: ped.clone(),
                    rel_to_partner2: ped,
                });
            }
            let fam_id = store.families[fam_idx].id.clone();
            if created_family_ids.contains(&fam_id) {
                let updated_fam = store.families[fam_idx].clone();
                update_created_family_log(store, &shared_tx_id, &updated_fam);
            } else {
                store.families[fam_idx].version += 1;
                let after_fam = store.families[fam_idx].clone();
                log_change(
                    store,
                    Some(shared_tx_id.clone()),
                    "ai.story_commit",
                    "family",
                    &fam_id,
                    "update",
                    Some(after_fam.gid.clone()),
                    "Added child to family from story draft".to_string(),
                    Some(serde_json::to_value(&before_fam).unwrap_or_else(|_| json!({}))),
                    Some(serde_json::to_value(&after_fam).unwrap_or_else(|_| json!({}))),
                    caller_id.clone(),
                    caller_name.clone(),
                );
                if !updated_family_ids.contains(&fam_id) {
                    updated_family_ids.push(fam_id);
                }
            }
        } else {
            let fam_id = Uuid::new_v4().to_string();
            let fam_gid = format!("F{:04}", store.next_family_gid_seq);
            store.next_family_gid_seq += 1;
            let new_fam = FamilyDto {
                id: fam_id.clone(),
                gid: fam_gid.clone(),
                partner1_id: p1_target,
                partner2_id: p2_target,
                relationship_type: "married".to_string(),
                is_private: false,
                children: vec![FamilyChildDto {
                    child_id,
                    sort_order: 0,
                    rel_to_partner1: ped.clone(),
                    rel_to_partner2: ped,
                }],
                external_ids: HashMap::new(),
                version: 1,
                marriage_date: None,
                marriage_place: None,
            };
            store.families.push(new_fam.clone());
            log_change(
                store,
                Some(shared_tx_id.clone()),
                "ai.story_commit",
                "family",
                &fam_id,
                "create",
                Some(fam_gid),
                "Created parent-child family from story draft".to_string(),
                None,
                Some(serde_json::to_value(&new_fam).unwrap_or_else(|_| json!({}))),
                caller_id.clone(),
                caller_name.clone(),
            );
            created_family_ids.push(fam_id);
        }
    }

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    if let Some(d) = store.story_drafts.get_mut(draft_idx) {
        d.status = "committed".to_string();
        d.committed_tx_id = Some(shared_tx_id.clone());
        d.updated_at = Utc::now().to_rfc3339();
    }

    Ok(CommitStoryDraftResponse {
        status: "committed".to_string(),
        draft_id: draft_id.to_string(),
        tx_id: shared_tx_id,
        created_person_ids,
        updated_person_ids,
        created_family_ids,
        updated_family_ids,
        created_event_ids,
        created_place_ids,
    })
}

pub(crate) fn update_created_family_log(
    store: &mut GenealogyPrivacyStore,
    tx_id: &str,
    fam: &FamilyDto,
) {
    if let Some(entry) = store.change_log.iter_mut().rev().find(|e| {
        e.tx_id == tx_id && e.entity_type == "family" && e.entity_id == fam.id && e.op == "create"
    }) {
        entry.after_json = Some(serde_json::to_value(fam).unwrap_or_else(|_| json!({})));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_event_in_tx(
    store: &mut GenealogyPrivacyStore,
    person_id: &str,
    event_type: &str,
    date_text: Option<String>,
    place_text: Option<String>,
    place_id: Option<String>,
    description: Option<String>,
    tx_id: &str,
    caller_id: Option<String>,
    caller_name: Option<String>,
) -> String {
    let ev_id = Uuid::new_v4().to_string();
    let ev_gid = format!("E{:04}", store.next_event_gid_seq);
    store.next_event_gid_seq += 1;
    let parsed_date = date_text.as_deref().map(GenDate::parse);
    let date_sort = parsed_date.as_ref().and_then(GenDate::compute_sort_key);
    let normalized_type = crate::services::genealogy::normalize_event_type(event_type);

    let new_ev = LifeEventDto {
        id: ev_id.clone(),
        gid: ev_gid.clone(),
        individual_id: Some(person_id.to_string()),
        family_id: None,
        event_type: normalized_type.clone(),
        custom_type: None,
        date: parsed_date,
        date_sort,
        gedcom_date: date_text,
        place_id,
        place_full_name: place_text,
        description,
        source_text: None,
        media_ids: vec![],
        is_private: false,
        version: 1,
    };
    store.events.push(new_ev.clone());
    log_change(
        store,
        Some(tx_id.to_string()),
        "ai.story_commit",
        "event",
        &ev_id,
        "create",
        Some(ev_gid),
        format!("Created {normalized_type} event from story draft"),
        None,
        Some(serde_json::to_value(&new_ev).unwrap_or_else(|_| json!({}))),
        caller_id,
        caller_name,
    );
    ev_id
}

pub(crate) fn resolve_or_create_place_in_tx(
    store: &mut GenealogyPrivacyStore,
    place_name: &str,
    tx_id: &str,
    created_place_ids: &mut Vec<String>,
    caller_id: Option<String>,
    caller_name: Option<String>,
) -> String {
    let clean = place_name.trim();
    if let Some(existing) = store.places.iter().find(|p| {
        p.name.eq_ignore_ascii_case(clean)
            || p.full_name.eq_ignore_ascii_case(clean)
            || p.alt_names
                .iter()
                .any(|a| a.name.eq_ignore_ascii_case(clean))
    }) {
        return existing.id.clone();
    }

    let place_id = Uuid::new_v4().to_string();
    let place_gid = format!("P{:04}", store.next_place_gid_seq);
    store.next_place_gid_seq += 1;
    let new_place = PlaceSummaryDto {
        id: place_id.clone(),
        gid: place_gid,
        name: clean.to_string(),
        place_type: "city".to_string(),
        parent_id: None,
        full_name: clean.to_string(),
        country: None,
        latitude: None,
        longitude: None,
        events_count: 1,
        children_count: 0,
        alt_names: vec![],
        version: 1,
    };
    store.places.push(new_place.clone());
    log_change(
        store,
        Some(tx_id.to_string()),
        "ai.story_commit",
        "place",
        &place_id,
        "create",
        Some(clean.to_string()),
        format!("Created place {clean} from story draft"),
        None,
        Some(serde_json::to_value(&new_place).unwrap_or_else(|_| json!({}))),
        caller_id,
        caller_name,
    );
    created_place_ids.push(place_id.clone());
    place_id
}
