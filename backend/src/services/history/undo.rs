#[allow(unused_imports)]
use super::*;

/// Reverts a single change log entry in reverse (`create` -> delete, `update` -> restore `before_json`, `delete` -> recreate from `before_json`).
pub(crate) fn revert_single_change_entry(
    store: &mut GenealogyPrivacyStore,
    entry: &ChangeLogEntryDto,
) -> Result<Option<String>, AppError> {
    let et = normalize_entity_type(&entry.entity_type);
    match entry.op.as_str() {
        "create" => {
            match et.as_str() {
                "individual" => {
                    store.persons.retain(|p| p.id != entry.entity_id);
                    store
                        .events
                        .retain(|e| e.individual_id.as_deref() != Some(entry.entity_id.as_str()));
                    store.notes.retain(|n| {
                        !(n.entity_type == "individual" && n.entity_id == entry.entity_id)
                    });
                    for fam in &mut store.families {
                        if fam.partner1_id.as_deref() == Some(entry.entity_id.as_str()) {
                            fam.partner1_id = None;
                        }
                        if fam.partner2_id.as_deref() == Some(entry.entity_id.as_str()) {
                            fam.partner2_id = None;
                        }
                        fam.children.retain(|c| c.child_id != entry.entity_id);
                    }
                    store.families.retain(|f| {
                        f.partner1_id.is_some() || f.partner2_id.is_some() || !f.children.is_empty()
                    });
                }
                "family" => {
                    store.families.retain(|f| f.id != entry.entity_id);
                }
                "event" => {
                    store.events.retain(|e| e.id != entry.entity_id);
                }
                "place" => {
                    store.places.retain(|p| p.id != entry.entity_id);
                }
                "media" => {
                    store.media.retain(|m| m.id != entry.entity_id);
                }
                "note" => {
                    store.notes.retain(|n| n.id != entry.entity_id);
                }
                _ => {}
            }
            Ok(Some(entry.entity_id.clone()))
        }
        "update" => {
            let Some(ref before_val) = entry.before_json else {
                return Ok(None);
            };
            match et.as_str() {
                "individual" => {
                    if let Ok(mut prev_person) =
                        serde_json::from_value::<TreePersonNodeDto>(before_val.clone())
                    {
                        if let Some(cur) =
                            store.persons.iter_mut().find(|p| p.id == entry.entity_id)
                        {
                            prev_person.version = cur.version + 1;
                            *cur = prev_person;
                            return Ok(Some(entry.entity_id.clone()));
                        }
                    }
                }
                "family" => {
                    if let Ok(mut prev_fam) =
                        serde_json::from_value::<FamilyDto>(before_val.clone())
                    {
                        if let Some(cur) =
                            store.families.iter_mut().find(|f| f.id == entry.entity_id)
                        {
                            prev_fam.version = cur.version + 1;
                            *cur = prev_fam;
                            return Ok(Some(entry.entity_id.clone()));
                        }
                    }
                }
                "event" => {
                    if let Ok(mut prev_ev) =
                        serde_json::from_value::<LifeEventDto>(before_val.clone())
                    {
                        if let Some(cur) = store.events.iter_mut().find(|e| e.id == entry.entity_id)
                        {
                            prev_ev.version = cur.version + 1;
                            *cur = prev_ev;
                            return Ok(Some(entry.entity_id.clone()));
                        }
                    }
                }
                "place" => {
                    if let Ok(mut prev_pl) =
                        serde_json::from_value::<PlaceSummaryDto>(before_val.clone())
                    {
                        if let Some(cur) = store.places.iter_mut().find(|p| p.id == entry.entity_id)
                        {
                            prev_pl.version = cur.version + 1;
                            *cur = prev_pl;
                            return Ok(Some(entry.entity_id.clone()));
                        }
                    }
                }
                "media" => {
                    if let Ok(mut prev_m) =
                        serde_json::from_value::<MediaAssetSummaryDto>(before_val.clone())
                    {
                        if let Some(cur) = store.media.iter_mut().find(|m| m.id == entry.entity_id)
                        {
                            prev_m.version = cur.version + 1;
                            *cur = prev_m;
                            return Ok(Some(entry.entity_id.clone()));
                        }
                    }
                }
                "note" => {
                    if let Ok(mut prev_n) = serde_json::from_value::<NoteDto>(before_val.clone()) {
                        if let Some(cur) = store.notes.iter_mut().find(|n| n.id == entry.entity_id)
                        {
                            prev_n.version = cur.version + 1;
                            *cur = prev_n;
                            return Ok(Some(entry.entity_id.clone()));
                        }
                    }
                }
                _ => {}
            }
            Ok(None)
        }
        "delete" => {
            let Some(ref before_val) = entry.before_json else {
                return Ok(None);
            };
            match et.as_str() {
                "individual" => {
                    if let Ok(snap) =
                        serde_json::from_value::<DeletedPersonSnapshotPayload>(before_val.clone())
                    {
                        let pid = snap.person.id.clone();
                        if !store.persons.iter().any(|p| p.id == pid) {
                            store.persons.push(snap.person);
                        }
                        for ev in snap.events {
                            if !store.events.iter().any(|e| e.id == ev.id) {
                                store.events.push(ev);
                            }
                        }
                        for note in snap.notes {
                            if !store.notes.iter().any(|n| n.id == note.id) {
                                store.notes.push(note);
                            }
                        }
                        for old_fam in snap.families_before {
                            if let Some(existing_fam) =
                                store.families.iter_mut().find(|f| f.id == old_fam.id)
                            {
                                if old_fam.partner1_id.as_deref() == Some(pid.as_str())
                                    && existing_fam.partner1_id.is_none()
                                {
                                    existing_fam.partner1_id = Some(pid.clone());
                                }
                                if old_fam.partner2_id.as_deref() == Some(pid.as_str())
                                    && existing_fam.partner2_id.is_none()
                                {
                                    existing_fam.partner2_id = Some(pid.clone());
                                }
                                for ch in &old_fam.children {
                                    if ch.child_id == pid
                                        && !existing_fam.children.iter().any(|c| c.child_id == pid)
                                    {
                                        existing_fam.children.push(ch.clone());
                                    }
                                }
                                existing_fam.version += 1;
                            } else {
                                store.families.push(old_fam);
                            }
                        }
                        return Ok(Some(pid));
                    } else if let Ok(person) =
                        serde_json::from_value::<TreePersonNodeDto>(before_val.clone())
                    {
                        let pid = person.id.clone();
                        if !store.persons.iter().any(|p| p.id == pid) {
                            store.persons.push(person);
                        }
                        return Ok(Some(pid));
                    }
                }
                "family" => {
                    if let Ok(fam) = serde_json::from_value::<FamilyDto>(before_val.clone()) {
                        let fid = fam.id.clone();
                        if !store.families.iter().any(|f| f.id == fid) {
                            store.families.push(fam);
                        }
                        return Ok(Some(fid));
                    }
                }
                "event" => {
                    if let Ok(ev) = serde_json::from_value::<LifeEventDto>(before_val.clone()) {
                        let eid = ev.id.clone();
                        if !store.events.iter().any(|e| e.id == eid) {
                            store.events.push(ev);
                        }
                        return Ok(Some(eid));
                    }
                }
                "place" => {
                    if let Ok(pl) = serde_json::from_value::<PlaceSummaryDto>(before_val.clone()) {
                        let plid = pl.id.clone();
                        store.gid_aliases.remove(&pl.gid);
                        if !store.places.iter().any(|p| p.id == plid) {
                            store.places.push(pl);
                        }
                        return Ok(Some(plid));
                    }
                }
                "media" => {
                    if let Ok(m) =
                        serde_json::from_value::<MediaAssetSummaryDto>(before_val.clone())
                    {
                        let mid = m.id.clone();
                        if !store.media.iter().any(|x| x.id == mid) {
                            store.media.push(m);
                        }
                        return Ok(Some(mid));
                    }
                }
                "note" => {
                    if let Ok(n) = serde_json::from_value::<NoteDto>(before_val.clone()) {
                        let nid = n.id.clone();
                        if !store.notes.iter().any(|x| x.id == nid) {
                            store.notes.push(n);
                        }
                        return Ok(Some(nid));
                    }
                }
                _ => {}
            }
            Ok(None)
        }
        "merge" => {
            let Some(ref before_val) = entry.before_json else {
                return Ok(None);
            };
            if before_val.get("kind").and_then(|v| v.as_str()) == Some("person_merge_bundle") {
                if let Some(keep_val) = before_val.get("keep_before") {
                    if let Ok(mut keep_before) =
                        serde_json::from_value::<TreePersonNodeDto>(keep_val.clone())
                    {
                        if let Some(cur) = store.persons.iter_mut().find(|p| p.id == keep_before.id)
                        {
                            keep_before.version = cur.version + 1;
                            *cur = keep_before;
                        } else {
                            store.persons.push(keep_before);
                        }
                    }
                }
                if let Some(merge_val) = before_val.get("merge_person") {
                    if let Ok(merge_person) =
                        serde_json::from_value::<TreePersonNodeDto>(merge_val.clone())
                    {
                        if !store.persons.iter().any(|p| p.id == merge_person.id) {
                            store.persons.push(merge_person);
                        }
                    }
                }
                if let Some(fams_val) = before_val.get("families_before") {
                    if let Ok(fams) = serde_json::from_value::<Vec<FamilyDto>>(fams_val.clone()) {
                        store.families = fams;
                    }
                }
                if let Some(evs_val) = before_val.get("events_before") {
                    if let Ok(evs) = serde_json::from_value::<Vec<LifeEventDto>>(evs_val.clone()) {
                        store.events = evs;
                    }
                }
                if let Some(notes_val) = before_val.get("notes_before") {
                    if let Ok(notes) = serde_json::from_value::<Vec<NoteDto>>(notes_val.clone()) {
                        store.notes = notes;
                    }
                }
                if let Some(media_val) = before_val.get("media_before") {
                    if let Ok(media) =
                        serde_json::from_value::<Vec<MediaAssetSummaryDto>>(media_val.clone())
                    {
                        store.media = media;
                    }
                }
                if let Some(aliases_val) = before_val.get("gid_aliases_before") {
                    if let Ok(aliases) =
                        serde_json::from_value::<HashMap<String, String>>(aliases_val.clone())
                    {
                        store.gid_aliases = aliases;
                    }
                }
                return Ok(Some(entry.entity_id.clone()));
            }
            if before_val.get("kind").and_then(|v| v.as_str()) == Some("event_merge_bundle") {
                let mut restored_event_ids = std::collections::HashSet::new();
                let mut restored_family_ids = std::collections::HashSet::new();
                if let Some(evs_val) = before_val.get("events_before") {
                    if let Ok(evs) = serde_json::from_value::<Vec<LifeEventDto>>(evs_val.clone()) {
                        for ev in evs {
                            restored_event_ids.insert(ev.id.clone());
                            if let Some(cur) = store.events.iter_mut().find(|e| e.id == ev.id) {
                                *cur = ev;
                            } else {
                                store.events.push(ev);
                            }
                        }
                    }
                }
                if let Some(fams_val) = before_val.get("families_before") {
                    if let Ok(fams) = serde_json::from_value::<Vec<FamilyDto>>(fams_val.clone()) {
                        for fam in fams {
                            restored_family_ids.insert(fam.id.clone());
                            if let Some(cur) = store.families.iter_mut().find(|f| f.id == fam.id) {
                                *cur = fam;
                            } else {
                                store.families.push(fam);
                            }
                        }
                    }
                }
                if let Some(notes_val) = before_val.get("notes_before") {
                    if let Ok(notes) = serde_json::from_value::<Vec<NoteDto>>(notes_val.clone()) {
                        store.notes.retain(|n| {
                            !((n.entity_type == "event"
                                && restored_event_ids.contains(&n.entity_id))
                                || (n.entity_type == "family"
                                    && restored_family_ids.contains(&n.entity_id)))
                        });
                        for note in notes {
                            if !store.notes.iter().any(|n| n.id == note.id) {
                                store.notes.push(note);
                            }
                        }
                    }
                }
                if let Some(media_val) = before_val.get("media_before") {
                    if let Ok(media) =
                        serde_json::from_value::<Vec<MediaAssetSummaryDto>>(media_val.clone())
                    {
                        for m in media {
                            if let Some(cur) = store.media.iter_mut().find(|x| x.id == m.id) {
                                *cur = m;
                            } else {
                                store.media.push(m);
                            }
                        }
                    }
                }
                if let Some(persons_val) = before_val.get("persons_before") {
                    if let Ok(persons) =
                        serde_json::from_value::<Vec<TreePersonNodeDto>>(persons_val.clone())
                    {
                        for pb in persons {
                            if let Some(cur) = store.persons.iter_mut().find(|p| p.id == pb.id) {
                                cur.birth = pb.birth;
                                cur.birth_sort = pb.birth_sort;
                                cur.death = pb.death;
                                cur.death_sort = pb.death_sort;
                                cur.has_death_event = pb.has_death_event;
                                cur.version += 1;
                            } else {
                                store.persons.push(pb);
                            }
                        }
                    }
                }
                return Ok(Some(entry.entity_id.clone()));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// Undoes all mutations in a transaction (`POST /api/v1/history/tx/:tx_id/undo`, `14 §3.3, §5, §6 DoD #4`).
/// Allowed if `caller_id == actor_id` within 10 minutes of creation OR if `is_admin`.
pub fn undo_transaction(
    store: &mut GenealogyPrivacyStore,
    tx_id: &str,
    caller_id: &str,
    caller_name: Option<String>,
    is_admin: bool,
    current_year: i32,
    age_threshold: i32,
) -> Result<UndoTransactionResponse, AppError> {
    let target_tx = tx_id.trim();
    let matching_entries: Vec<ChangeLogEntryDto> = store
        .change_log
        .iter()
        .filter(|e| e.tx_id == target_tx || e.id == target_tx)
        .cloned()
        .collect();

    if matching_entries.is_empty() {
        return Err(AppError::NotFound);
    }

    // Check authorization: author within 10 minutes OR Admin
    if !is_admin {
        let first = &matching_entries[0];
        let is_author = first.actor_id.as_deref() == Some(caller_id);
        let within_10_min = DateTime::parse_from_rfc3339(&first.created_at)
            .map(|dt| (Utc::now() - dt.with_timezone(&Utc)) <= Duration::minutes(10))
            .unwrap_or(true);
        if !is_author || !within_10_min {
            return Err(AppError::Forbidden(
                "Only the author (within 10 minutes) or an Administrator can undo this transaction"
                    .to_string(),
            ));
        }
    }

    let mut restored_entity_ids = Vec::new();
    let mut undone_count = 0usize;

    // Revert in reverse chronological order
    for entry in matching_entries.iter().rev() {
        if let Some(eid) = revert_single_change_entry(store, entry)? {
            undone_count += 1;
            if !restored_entity_ids.contains(&eid) {
                restored_entity_ids.push(eid);
            }
        }
    }

    for draft in &mut store.story_drafts {
        if draft.committed_tx_id.as_deref() == Some(target_tx) {
            draft.status = "draft".to_string();
            draft.committed_tx_id = None;
            draft.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    log_change(
        store,
        None,
        "history.undo_transaction",
        "transaction",
        target_tx,
        "restore",
        None,
        format!("Undid transaction {target_tx} ({undone_count} changes)"),
        None,
        Some(json!({
            "undone_tx_id": target_tx,
            "undone_count": undone_count,
            "restored_entity_ids": restored_entity_ids,
        })),
        Some(caller_id.to_string()),
        caller_name,
    );

    Ok(UndoTransactionResponse {
        status: "ok".to_string(),
        tx_id: target_tx.to_string(),
        undone_count,
        restored_entity_ids,
    })
}

/// Restores a deleted record or reverts an updated record from `change_log` (`POST /api/v1/history/:change_id/restore`, `14 §3.3, §5, §6 DoD #3`).
pub fn restore_change(
    store: &mut GenealogyPrivacyStore,
    change_or_entity_id: &str,
    caller_id: Option<String>,
    caller_name: Option<String>,
    current_year: i32,
    age_threshold: i32,
) -> Result<Value, AppError> {
    let target = change_or_entity_id.trim();

    let log_entry = store
        .change_log
        .iter()
        .rev()
        .find(|entry| {
            matches!(entry.op.as_str(), "delete" | "update")
                && (entry.id == target
                    || entry.entity_id == target
                    || entry
                        .before_json
                        .as_ref()
                        .and_then(|v| v.get("person"))
                        .and_then(|p| p.get("gid"))
                        .and_then(|g| g.as_str())
                        .is_some_and(|g| g.eq_ignore_ascii_case(target))
                    || entry
                        .before_json
                        .as_ref()
                        .and_then(|v| v.get("gid"))
                        .and_then(|g| g.as_str())
                        .is_some_and(|g| g.eq_ignore_ascii_case(target)))
        })
        .cloned()
        .ok_or(AppError::NotFound)?;

    let restored_id = revert_single_change_entry(store, &log_entry)?.ok_or(AppError::NotFound)?;

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    let et = normalize_entity_type(&log_entry.entity_type);

    log_change(
        store,
        None,
        "history.restore_record",
        &et,
        &restored_id,
        "restore",
        log_entry.entity_label.clone(),
        format!("Restored {} {}", et, restored_id),
        None,
        log_entry.before_json.clone(),
        caller_id,
        caller_name,
    );

    if et == "individual" {
        let person = store
            .persons
            .iter()
            .find(|p| p.id == restored_id)
            .cloned()
            .ok_or(AppError::NotFound)?;
        let mut val = serde_json::to_value(&person).unwrap_or_else(|_| json!({}));
        if let Value::Object(ref mut map) = val {
            map.insert("status".to_string(), json!("ok"));
            map.insert("restored_entity_id".to_string(), json!(restored_id));
            map.insert("entity_type".to_string(), json!("individual"));
        }
        return Ok(val);
    }

    Ok(json!({
        "status": "ok",
        "change_id": log_entry.id,
        "entity_type": et,
        "entity_id": restored_id,
        "restored_entity_id": restored_id,
    }))
}
