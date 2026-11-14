#[allow(unused_imports)]
use super::*;
use crate::services::genealogy::{
    recompute_all_individuals_vitals, recompute_all_places_metadata, resolve_or_create_place_chain,
};
use crate::services::privacy::{find_family_marriage_facts, MediaAssetSummaryDto, NoteDto};
use std::collections::{HashMap, HashSet};

fn resolve_event_id_in_store(store: &GenealogyPrivacyStore, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    store
        .events
        .iter()
        .find(|e| e.id == trimmed || e.gid.eq_ignore_ascii_case(trimmed))
        .map(|e| e.id.clone())
}

fn family_partner_ids(store: &GenealogyPrivacyStore, fam: &FamilyDto) -> Vec<String> {
    let mut out: Vec<String> = [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
        .into_iter()
        .flatten()
        .filter(|pid| store.persons.iter().any(|p| p.id == **pid))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

fn validate_explicit_events_compatible(
    store: &GenealogyPrivacyStore,
    keep_id: &str,
    merge_ids: &[String],
) -> Result<(), AppError> {
    let mut involved: Vec<&LifeEventDto> = Vec::new();
    for id in std::iter::once(keep_id).chain(merge_ids.iter().map(String::as_str)) {
        if let Some(ev) = store.events.iter().find(|e| e.id == id) {
            involved.push(ev);
        }
    }
    let first_type = canonical_event_type(involved[0]);
    if involved
        .iter()
        .any(|e| canonical_event_type(e) != first_type)
    {
        let mut fields = HashMap::new();
        fields.insert(
            "event_type".to_string(),
            "Cannot merge events of different types".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Events must share the same canonical event type".to_string(),
            Some(fields),
        ));
    }

    let has_fam = involved
        .iter()
        .any(|e| e.family_id.as_deref().is_some_and(|s| !s.trim().is_empty()));
    let is_union = is_union_context(&first_type, has_fam);
    if !is_union {
        let first_ind = involved[0].individual_id.as_deref();
        if first_ind.is_none()
            || involved
                .iter()
                .any(|e| e.individual_id.as_deref() != first_ind)
        {
            let mut fields = HashMap::new();
            fields.insert(
                "individual_id".to_string(),
                "Cannot merge individual events belonging to different individuals".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "Events must belong to the same individual".to_string(),
                Some(fields),
            ));
        }
        return Ok(());
    }

    let mut all_partners: HashSet<String> = HashSet::new();
    let mut shared_family_ids: HashSet<String> = HashSet::new();
    for ev in &involved {
        if let Some(fid) = ev.family_id.as_deref() {
            shared_family_ids.insert(fid.to_string());
            if let Some(fam) = store.families.iter().find(|f| f.id == fid) {
                for p in family_partner_ids(store, fam) {
                    all_partners.insert(p);
                }
            }
        } else if let Some(pid) = ev.individual_id.as_deref() {
            all_partners.insert(pid.to_string());
        }
    }
    if all_partners.len() > 2 || (all_partners.is_empty() && shared_family_ids.len() > 1) {
        let mut fields = HashMap::new();
        fields.insert(
            "family_id".to_string(),
            "Cannot merge union events belonging to different couples".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Events belong to incompatible unions or couples".to_string(),
            Some(fields),
        ));
    }
    Ok(())
}

fn resolve_explicit_events(
    store: &GenealogyPrivacyStore,
    req: &MergeDuplicateEventsRequest,
) -> Result<(Option<String>, String, Vec<String>), AppError> {
    if let Some(gid) = req
        .group_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let group = compute_duplicate_event_groups(store, true)
            .into_iter()
            .find(|g| g.id == gid)
            .ok_or(AppError::NotFound)?;
        let group_ids: HashSet<String> = group.events.iter().map(|e| e.id.clone()).collect();

        let keep_id = if let Some(raw_keep) = req
            .keep_event_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let resolved = resolve_event_id_in_store(store, raw_keep).ok_or(AppError::NotFound)?;
            if !group_ids.contains(&resolved) {
                return Err(AppError::ValidationFailed(
                    "Invalid surviving event for duplicate group".to_string(),
                    None,
                ));
            }
            resolved
        } else {
            group.proposed.keep_event_id.clone()
        };

        let merge_ids: Vec<String> = if let Some(ref explicit_merge) = req.merge_event_ids {
            let mut seen = HashSet::new();
            let mut ids = Vec::new();
            for raw in explicit_merge {
                let resolved = resolve_event_id_in_store(store, raw).ok_or(AppError::NotFound)?;
                if resolved == keep_id {
                    continue;
                }
                if !group_ids.contains(&resolved) {
                    return Err(AppError::ValidationFailed(
                        "Invalid merge event for duplicate group".to_string(),
                        None,
                    ));
                }
                if seen.insert(resolved.clone()) {
                    ids.push(resolved);
                }
            }
            ids
        } else {
            group
                .events
                .iter()
                .map(|e| e.id.clone())
                .filter(|id| id != &keep_id)
                .collect()
        };

        if merge_ids.is_empty() {
            return Err(AppError::ValidationFailed(
                "At least one duplicate event is required".to_string(),
                None,
            ));
        }
        return Ok((Some(group.id), keep_id, merge_ids));
    }

    let raw_keep = req
        .keep_event_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::ValidationFailed("keep_event_id is required".to_string(), None))?;
    let keep_id = resolve_event_id_in_store(store, raw_keep).ok_or(AppError::NotFound)?;
    let raw_merge = req.merge_event_ids.as_ref().ok_or_else(|| {
        AppError::ValidationFailed("merge_event_ids is required".to_string(), None)
    })?;

    let mut seen = HashSet::new();
    let mut merge_ids = Vec::new();
    for raw in raw_merge {
        if raw.trim().is_empty() {
            continue;
        }
        let resolved = resolve_event_id_in_store(store, raw).ok_or(AppError::NotFound)?;
        if resolved != keep_id && seen.insert(resolved.clone()) {
            merge_ids.push(resolved);
        }
    }
    if merge_ids.is_empty() {
        return Err(AppError::ValidationFailed(
            "No distinct duplicate events provided".to_string(),
            None,
        ));
    }
    validate_explicit_events_compatible(store, &keep_id, &merge_ids)?;
    Ok((None, keep_id, merge_ids))
}

fn families_are_consolidatable(
    store: &GenealogyPrivacyStore,
    target: &FamilyDto,
    other: &FamilyDto,
) -> bool {
    let tp: HashSet<String> = family_partner_ids(store, target).into_iter().collect();
    let op: HashSet<String> = family_partner_ids(store, other).into_iter().collect();
    !tp.is_empty()
        && !op.is_empty()
        && tp.intersection(&op).next().is_some()
        && tp.union(&op).count() <= 2
}

fn consolidate_union_families(
    store: &mut GenealogyPrivacyStore,
    all_fids: Vec<String>,
    prefer_fid: Option<&str>,
) -> (Option<String>, Vec<String>) {
    let target_fam_id = all_fids
        .iter()
        .max_by_key(|fid| {
            let p_cnt = store
                .families
                .iter()
                .find(|f| f.id == **fid)
                .map(|f| family_partner_ids(store, f).len())
                .unwrap_or(0);
            (p_cnt, prefer_fid == Some(fid.as_str()))
        })
        .cloned();

    let Some(ref target_fid) = target_fam_id else {
        return (None, vec![]);
    };
    let Some(target_snapshot) = store.families.iter().find(|f| f.id == *target_fid).cloned() else {
        return (target_fam_id, vec![]);
    };

    let mut merged_family_ids = Vec::new();
    for other_fid in all_fids.into_iter().filter(|f| f != target_fid) {
        let Some(other_fam) = store.families.iter().find(|f| f.id == other_fid).cloned() else {
            continue;
        };
        if !families_are_consolidatable(store, &target_snapshot, &other_fam) {
            continue;
        }
        if let Some(tfam) = store.families.iter_mut().find(|f| f.id == *target_fid) {
            if tfam.partner1_id.is_none() {
                tfam.partner1_id = other_fam
                    .partner1_id
                    .clone()
                    .filter(|p| tfam.partner2_id.as_ref() != Some(p))
                    .or_else(|| {
                        other_fam
                            .partner2_id
                            .clone()
                            .filter(|p| tfam.partner2_id.as_ref() != Some(p))
                    });
            }
            if tfam.partner2_id.is_none() {
                tfam.partner2_id = other_fam
                    .partner2_id
                    .clone()
                    .filter(|p| tfam.partner1_id.as_ref() != Some(p))
                    .or_else(|| {
                        other_fam
                            .partner1_id
                            .clone()
                            .filter(|p| tfam.partner1_id.as_ref() != Some(p))
                    });
            }
            let map_rel_for_target =
                |target_pid: Option<&str>, ch: &FamilyChildDto| -> (bool, String) {
                    match target_pid {
                        Some(tp) if other_fam.partner1_id.as_deref() == Some(tp) => {
                            (true, ch.rel_to_partner1.clone())
                        }
                        Some(tp) if other_fam.partner2_id.as_deref() == Some(tp) => {
                            (true, ch.rel_to_partner2.clone())
                        }
                        _ => (false, "birth".to_string()),
                    }
                };
            for ch in other_fam.children {
                let (has_p1, rel1) = map_rel_for_target(tfam.partner1_id.as_deref(), &ch);
                let (has_p2, rel2) = map_rel_for_target(tfam.partner2_id.as_deref(), &ch);
                if let Some(existing) = tfam.children.iter_mut().find(|c| c.child_id == ch.child_id)
                {
                    if has_p1 && existing.rel_to_partner1 == "birth" && rel1 != "birth" {
                        existing.rel_to_partner1 = rel1;
                    }
                    if has_p2 && existing.rel_to_partner2 == "birth" && rel2 != "birth" {
                        existing.rel_to_partner2 = rel2;
                    }
                } else {
                    tfam.children.push(FamilyChildDto {
                        child_id: ch.child_id,
                        sort_order: ch.sort_order,
                        rel_to_partner1: rel1,
                        rel_to_partner2: rel2,
                    });
                }
            }
            for (k, v) in other_fam.external_ids {
                tfam.external_ids.entry(k).or_insert(v);
            }
            tfam.version += 1;
        }

        for ev in &mut store.events {
            if ev.family_id.as_deref() == Some(other_fid.as_str()) {
                ev.family_id = Some(target_fid.clone());
            }
        }
        for note in &mut store.notes {
            if note.entity_type == "family" && note.entity_id == other_fid {
                note.entity_id = target_fid.clone();
            }
        }
        let mut seen_notes = HashSet::new();
        store.notes.retain(|n| {
            if n.entity_type == "family" && n.entity_id == *target_fid {
                seen_notes.insert(n.body.trim().to_string())
            } else {
                true
            }
        });
        for media in &mut store.media {
            for link in &mut media.links {
                if link.entity_type == "family" && link.entity_id == other_fid {
                    link.entity_id = target_fid.clone();
                }
            }
            let mut seen_links = HashSet::new();
            media
                .links
                .retain(|l| seen_links.insert((l.entity_type.clone(), l.entity_id.clone())));
        }
        store.families.retain(|f| f.id != other_fid);
        merged_family_ids.push(other_fid);
    }

    (target_fam_id, merged_family_ids)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_single_event_merge(
    store: &mut GenealogyPrivacyStore,
    keep_id: &str,
    merge_ids: &[String],
    date_override: Option<&str>,
    place_id_override: Option<&str>,
    place_name_override: Option<&str>,
    description_override: Option<&str>,
    source_text_override: Option<&str>,
) -> Result<(LifeEventDto, Vec<String>), AppError> {
    let media_by_ev = build_media_by_event_map(store);
    let keep_ev_orig = store
        .events
        .iter()
        .find(|e| e.id == keep_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let mut merge_evs = Vec::with_capacity(merge_ids.len());
    for mid in merge_ids {
        let ev = store
            .events
            .iter()
            .find(|e| e.id == *mid)
            .cloned()
            .ok_or(AppError::NotFound)?;
        merge_evs.push(ev);
    }

    let mut ranked_refs: Vec<&LifeEventDto> = vec![&keep_ev_orig];
    ranked_refs.extend(merge_evs.iter());
    let (proposed, _, _) = synthesize_group_proposal(store, &ranked_refs, &media_by_ev);

    let chosen_date_text = match date_override {
        Some(raw) => Some(raw.trim().to_string()).filter(|s| !s.is_empty()),
        None => proposed.date_text.clone(),
    };
    let (new_date, new_date_sort, new_gedcom_date) = match chosen_date_text {
        Some(ref txt) => {
            let gd = GenDate::parse(txt);
            (Some(gd.clone()), gd.sort_key(), Some(gd.to_gedcom()))
        }
        None => (None, None, None),
    };

    let (new_place_id, new_place_full_name) = match (place_id_override, place_name_override) {
        (Some(pid_raw), _) if !pid_raw.trim().is_empty() => {
            let pid = store
                .resolve_place_id(pid_raw.trim())
                .ok_or(AppError::NotFound)?;
            let fname = store.places.iter().find(|p| p.id == pid).map(|p| {
                if p.full_name.trim().is_empty() {
                    p.name.trim().to_string()
                } else {
                    p.full_name.trim().to_string()
                }
            });
            (Some(pid), fname)
        }
        (_, Some(pname_raw)) if !pname_raw.trim().is_empty() => {
            let trimmed = pname_raw.trim();
            if let Some(existing) = store.places.iter().find(|p| {
                p.full_name.eq_ignore_ascii_case(trimmed) || p.name.eq_ignore_ascii_case(trimmed)
            }) {
                let fname = if existing.full_name.trim().is_empty() {
                    existing.name.trim().to_string()
                } else {
                    existing.full_name.trim().to_string()
                };
                (Some(existing.id.clone()), Some(fname))
            } else if let Ok(created) =
                resolve_or_create_place_chain(store, trimmed, None, None, None, None)
            {
                let fname = if created.full_name.trim().is_empty() {
                    created.name.clone()
                } else {
                    created.full_name.clone()
                };
                (Some(created.id), Some(fname))
            } else {
                (None, Some(trimmed.to_string()))
            }
        }
        (Some(_), _) | (_, Some(_)) => (None, None),
        (None, None) => (proposed.place_id.clone(), proposed.place_full_name.clone()),
    };

    let new_description = match description_override {
        Some(raw) => Some(raw.trim().to_string()).filter(|s| !s.is_empty()),
        None => proposed.description.clone(),
    };
    let new_source_text = match source_text_override {
        Some(raw) => Some(raw.trim().to_string()).filter(|s| !s.is_empty()),
        None => proposed.source_text.clone(),
    };

    let merge_set: HashSet<String> = merge_ids.iter().cloned().collect();
    let mut union_media_ids: Vec<String> = Vec::new();
    for ev in std::iter::once(&keep_ev_orig).chain(merge_evs.iter()) {
        for mid in &ev.media_ids {
            if !union_media_ids.contains(mid) {
                union_media_ids.push(mid.clone());
            }
        }
    }

    for note in &mut store.notes {
        if note.entity_type == "event" && merge_set.contains(&note.entity_id) {
            note.entity_id = keep_id.to_string();
        }
    }
    let mut seen_notes = HashSet::new();
    store.notes.retain(|n| {
        if n.entity_type == "event" && n.entity_id == keep_id {
            seen_notes.insert(n.body.trim().to_string())
        } else {
            true
        }
    });

    for media in &mut store.media {
        let mut linked_here = false;
        for link in &mut media.links {
            if link.entity_type == "event"
                && (link.entity_id == keep_id || merge_set.contains(&link.entity_id))
            {
                link.entity_id = keep_id.to_string();
                linked_here = true;
            }
        }
        if linked_here && !union_media_ids.contains(&media.id) {
            union_media_ids.push(media.id.clone());
        }
        let mut seen_links = HashSet::new();
        media
            .links
            .retain(|l| seen_links.insert((l.entity_type.clone(), l.entity_id.clone())));
    }

    let surviving_canon = canonical_event_type(&keep_ev_orig);
    let any_private = keep_ev_orig.is_private || merge_evs.iter().any(|e| e.is_private);
    let has_fam = std::iter::once(&keep_ev_orig)
        .chain(merge_evs.iter())
        .any(|e| e.family_id.as_deref().is_some_and(|s| !s.trim().is_empty()));
    let is_union = is_union_context(&surviving_canon, has_fam);

    let (target_fam_id, merged_family_ids) = if is_union {
        let mut all_fids: Vec<String> = Vec::new();
        for ev in std::iter::once(&keep_ev_orig).chain(merge_evs.iter()) {
            if let Some(ref fid) = ev.family_id {
                if !fid.trim().is_empty() && !all_fids.contains(fid) {
                    all_fids.push(fid.clone());
                }
            }
        }
        consolidate_union_families(store, all_fids, keep_ev_orig.family_id.as_deref())
    } else {
        (None, Vec::new())
    };

    let mut surviving = keep_ev_orig;
    surviving.event_type = surviving_canon;
    surviving.date = new_date;
    surviving.date_sort = new_date_sort;
    surviving.gedcom_date = new_gedcom_date;
    surviving.place_id = new_place_id;
    surviving.place_full_name = new_place_full_name;
    surviving.description = new_description;
    surviving.source_text = new_source_text;
    surviving.media_ids = union_media_ids;
    surviving.is_private = any_private;
    if let Some(ref tfid) = target_fam_id {
        surviving.family_id = Some(tfid.clone());
        surviving.individual_id = None;
    } else if surviving.individual_id.is_none() {
        surviving.individual_id = merge_evs.iter().find_map(|e| e.individual_id.clone());
    }
    surviving.version += 1;

    store.events.retain(|e| !merge_set.contains(&e.id));
    if let Some(slot) = store.events.iter_mut().find(|e| e.id == keep_id) {
        *slot = surviving.clone();
    } else {
        store.events.push(surviving.clone());
    }

    if let Some(ref tfid) = target_fam_id {
        if surviving.event_type == "marriage" {
            let (m_date, m_place) = find_family_marriage_facts(&store.events, tfid);
            if let Some(tfam) = store.families.iter_mut().find(|f| f.id == *tfid) {
                tfam.marriage_date = m_date;
                tfam.marriage_place = m_place;
            }
        }
    }

    Ok((surviving, merged_family_ids))
}

/// Builds a scoped pre-merge snapshot containing only affected events, families, notes, media, and persons
/// so out-of-order Undo never overwrites unrelated later mutations.
pub(crate) fn build_event_merge_before_bundle(
    store: &GenealogyPrivacyStore,
    target_event_ids: &[String],
) -> serde_json::Value {
    let mut ev_ids: HashSet<String> = target_event_ids.iter().cloned().collect();
    let mut fam_ids: HashSet<String> = HashSet::new();
    for ev in &store.events {
        if ev_ids.contains(&ev.id) {
            if let Some(ref fid) = ev.family_id {
                if !fid.trim().is_empty() && is_union_context(&canonical_event_type(ev), true) {
                    fam_ids.insert(fid.clone());
                }
            }
        }
    }
    // Include any sibling events on involved families in case family consolidation re-points them
    for ev in &store.events {
        if ev
            .family_id
            .as_deref()
            .is_some_and(|fid| fam_ids.contains(fid))
        {
            ev_ids.insert(ev.id.clone());
        }
    }

    let events_before: Vec<LifeEventDto> = store
        .events
        .iter()
        .filter(|e| ev_ids.contains(&e.id))
        .cloned()
        .collect();
    let families_before: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| fam_ids.contains(&f.id))
        .cloned()
        .collect();
    let notes_before: Vec<NoteDto> = store
        .notes
        .iter()
        .filter(|n| {
            (n.entity_type == "event" && ev_ids.contains(&n.entity_id))
                || (n.entity_type == "family" && fam_ids.contains(&n.entity_id))
        })
        .cloned()
        .collect();
    let media_ids_in_events: HashSet<&str> = events_before
        .iter()
        .flat_map(|e| e.media_ids.iter().map(String::as_str))
        .collect();
    let media_before: Vec<crate::services::privacy::MediaAssetSummaryDto> = store
        .media
        .iter()
        .filter(|m| {
            media_ids_in_events.contains(m.id.as_str())
                || m.links.iter().any(|l| {
                    (l.entity_type == "event" && ev_ids.contains(&l.entity_id))
                        || (l.entity_type == "family" && fam_ids.contains(&l.entity_id))
                })
        })
        .cloned()
        .collect();

    let mut person_ids: HashSet<String> = HashSet::new();
    for ev in &events_before {
        if let Some(ref pid) = ev.individual_id {
            person_ids.insert(pid.clone());
        }
    }
    for fam in &families_before {
        if let Some(ref p1) = fam.partner1_id {
            person_ids.insert(p1.clone());
        }
        if let Some(ref p2) = fam.partner2_id {
            person_ids.insert(p2.clone());
        }
    }
    let persons_before: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter(|p| person_ids.contains(&p.id))
        .cloned()
        .collect();

    serde_json::json!({
        "kind": "event_merge_bundle",
        "events_before": events_before,
        "families_before": families_before,
        "notes_before": notes_before,
        "media_before": media_before,
        "persons_before": persons_before,
    })
}

/// Merges a duplicate event group (or explicit keep + duplicate event IDs) into a single surviving event with Undo snapshot.
pub fn merge_duplicate_events_in_store(
    store: &mut GenealogyPrivacyStore,
    req: MergeDuplicateEventsRequest,
    actor_id: Option<String>,
    actor_name: Option<String>,
    current_year: i32,
    age_threshold: i32,
) -> Result<MergeDuplicateEventsResponse, AppError> {
    let (resolved_group_id, keep_id, merge_ids) = resolve_explicit_events(store, &req)?;

    let mut target_event_ids = vec![keep_id.clone()];
    target_event_ids.extend(merge_ids.iter().cloned());
    let before_bundle = build_event_merge_before_bundle(store, &target_event_ids);

    let (surviving_event, merged_family_ids) = apply_single_event_merge(
        store,
        &keep_id,
        &merge_ids,
        req.date_text.as_deref(),
        req.place_id.as_deref(),
        req.place_name.as_deref(),
        req.description.as_deref(),
        req.source_text.as_deref(),
    )?;

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    let tx_id = Uuid::new_v4().to_string();
    let summary = format!(
        "Merged {} duplicate {} event(s) into {} ({})",
        merge_ids.len(),
        surviving_event.event_type,
        surviving_event.gid,
        surviving_event.id
    );
    log_change(
        store,
        Some(tx_id.clone()),
        "quality.events.merge",
        "event",
        &surviving_event.id,
        "merge",
        Some(format!(
            "{} ({})",
            surviving_event.event_type, surviving_event.gid
        )),
        summary,
        Some(before_bundle),
        serde_json::to_value(&surviving_event).ok(),
        actor_id,
        actor_name,
    );
    store.persist_if_production();

    Ok(MergeDuplicateEventsResponse {
        status: "merged".to_string(),
        tx_id,
        group_id: resolved_group_id,
        surviving_event,
        merged_event_ids: merge_ids,
        merged_family_ids,
    })
}
