#[allow(unused_imports)]
use super::*;

/// Captures a point-in-time `TreeSnapshotDto` of the genealogy store (`14 §2, §3.3, §5, §6 DoD #6`).
pub fn create_snapshot(
    store: &mut GenealogyPrivacyStore,
    label: Option<String>,
    reason: &str,
    user_id: Option<String>,
    user_name: Option<String>,
    storage_dir: &str,
) -> Result<TreeSnapshotDto, AppError> {
    let snap_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = (now + Duration::days(14)).to_rfc3339();
    let created_at = now.to_rfc3339();

    let record_counts = TreeSnapshotRecordCountsDto {
        individuals: store.persons.len(),
        families: store.families.len(),
        events: store.events.len(),
        places: store.places.len(),
        media: store.media.len(),
        notes: store.notes.len(),
    };

    let payload = StoreSnapshotPayload {
        persons: store.persons.clone(),
        families: store.families.clone(),
        events: store.events.clone(),
        notes: store.notes.clone(),
        media: store.media.clone(),
        places: store.places.clone(),
        next_gid_seq: store.next_gid_seq,
        next_family_gid_seq: store.next_family_gid_seq,
        next_event_gid_seq: store.next_event_gid_seq,
        next_place_gid_seq: store.next_place_gid_seq,
        next_media_gid_seq: store.next_media_gid_seq,
    };

    let payload_json = serde_json::to_value(&payload)
        .map_err(|e| AppError::Internal(format!("Failed to serialize snapshot: {e}")))?;

    let snap_dir = std::path::Path::new(storage_dir).join("snapshots");
    let _ = std::fs::create_dir_all(&snap_dir);
    let file_path_buf = snap_dir.join(format!("snapshot-{snap_id}.json"));
    let file_path_str = file_path_buf.to_string_lossy().to_string();
    if let Ok(bytes) = serde_json::to_vec(&payload_json) {
        let _ = std::fs::write(&file_path_buf, bytes);
    }

    let resolved_label = label
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("Snapshot ({reason}) — {}", now.format("%Y-%m-%d %H:%M:%S")));

    let snapshot = TreeSnapshotDto {
        id: snap_id,
        label: resolved_label,
        reason: reason.to_string(),
        file_path: Some(file_path_str),
        created_by: user_id,
        created_by_name: user_name,
        created_at,
        expires_at,
        record_counts,
        payload_json: Some(payload_json),
    };

    store.snapshots.push(snapshot.clone());
    Ok(snapshot)
}

/// Safely rolls back the entire family tree to a target `TreeSnapshotDto` (`14 §3.3, §5, §6 DoD #6`).
/// Requires `confirm_text` matching `"ROLLBACK"` (or `"ОТКАТ"`) and automatically creates a
/// `pre_rollback` safety snapshot of the current state first.
#[allow(clippy::too_many_arguments)]
pub fn rollback_snapshot(
    store: &mut GenealogyPrivacyStore,
    snapshot_id: &str,
    user_id: Option<String>,
    user_name: Option<String>,
    confirm_text: Option<&str>,
    storage_dir: &str,
    current_year: i32,
    age_threshold: i32,
) -> Result<RollbackSnapshotResponse, AppError> {
    let confirmed = confirm_text
        .map(str::trim)
        .is_some_and(|c| c.eq_ignore_ascii_case("ROLLBACK") || c.eq_ignore_ascii_case("ОТКАТ"));
    if !confirmed {
        let mut fields = HashMap::new();
        fields.insert(
            "confirm_text".to_string(),
            "confirmation_required".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Type ROLLBACK to confirm restoring the family tree to this snapshot".to_string(),
            Some(fields),
        ));
    }

    let target_id = snapshot_id.trim();
    let target_snap = store
        .snapshots
        .iter()
        .find(|s| s.id == target_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let payload_val = target_snap.payload_json.ok_or(AppError::NotFound)?;
    let restored_payload: StoreSnapshotPayload = serde_json::from_value(payload_val)
        .map_err(|e| AppError::Internal(format!("Invalid snapshot payload: {e}")))?;

    // Take automatic pre-rollback safety snapshot first (`14 §3.3, §6 DoD #6`)
    let safety_snap = create_snapshot(
        store,
        Some(format!(
            "Pre-rollback safety snapshot before {}",
            target_snap.label
        )),
        "pre_rollback",
        user_id.clone(),
        user_name.clone(),
        storage_dir,
    )?;

    store.persons = restored_payload.persons;
    store.families = restored_payload.families;
    store.events = restored_payload.events;
    store.notes = restored_payload.notes;
    store.media = restored_payload.media;
    store.places = restored_payload.places;
    store.next_gid_seq = restored_payload.next_gid_seq;
    store.next_family_gid_seq = restored_payload.next_family_gid_seq;
    store.next_event_gid_seq = restored_payload.next_event_gid_seq;
    store.next_place_gid_seq = restored_payload.next_place_gid_seq;
    store.next_media_gid_seq = restored_payload.next_media_gid_seq;

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    let record_counts = TreeSnapshotRecordCountsDto {
        individuals: store.persons.len(),
        families: store.families.len(),
        events: store.events.len(),
        places: store.places.len(),
        media: store.media.len(),
        notes: store.notes.len(),
    };

    log_change(
        store,
        None,
        "snapshot.rollback",
        "tree",
        &target_snap.id,
        "restore",
        Some(target_snap.label.clone()),
        format!(
            "Rolled back family tree to snapshot '{}' (safety snapshot: {})",
            target_snap.label, safety_snap.id
        ),
        None,
        Some(json!({
            "restored_snapshot_id": target_snap.id,
            "safety_snapshot_id": safety_snap.id,
            "record_counts": record_counts,
        })),
        user_id,
        user_name,
    );

    Ok(RollbackSnapshotResponse {
        status: "ok".to_string(),
        restored_snapshot_id: target_snap.id,
        safety_snapshot_id: safety_snap.id,
        record_counts,
        message: "Family tree rolled back to snapshot".to_string(),
    })
}

/// Lightweight version polling helper (`GET /api/v1/:entity_type/:id/version`, `14 §4`).
pub fn get_entity_version(
    store: &GenealogyPrivacyStore,
    raw_entity_type: &str,
    raw_id: &str,
) -> Result<EntityVersionResponse, AppError> {
    let et = normalize_entity_type(raw_entity_type);
    let resolved_id = resolve_canonical_entity_id(store, &et, raw_id);

    let (id, gid, version) = match et.as_str() {
        "individual" => {
            let p = store
                .persons
                .iter()
                .find(|p| p.id == resolved_id)
                .ok_or(AppError::PersonNotFound)?;
            (p.id.clone(), p.gid.clone(), p.version)
        }
        "family" => {
            let f = store
                .families
                .iter()
                .find(|f| f.id == resolved_id)
                .ok_or(AppError::NotFound)?;
            (f.id.clone(), f.gid.clone(), f.version)
        }
        "event" => {
            let e = store
                .events
                .iter()
                .find(|e| e.id == resolved_id)
                .ok_or(AppError::NotFound)?;
            (e.id.clone(), e.gid.clone(), e.version)
        }
        "place" => {
            let p = store
                .places
                .iter()
                .find(|p| p.id == resolved_id)
                .ok_or(AppError::NotFound)?;
            (p.id.clone(), p.gid.clone(), p.version)
        }
        "media" => {
            let m = store
                .media
                .iter()
                .find(|m| m.id == resolved_id)
                .ok_or(AppError::NotFound)?;
            (m.id.clone(), m.gid.clone(), m.version)
        }
        "note" => {
            let n = store
                .notes
                .iter()
                .find(|n| n.id == resolved_id)
                .ok_or(AppError::NotFound)?;
            (n.id.clone(), n.id.clone(), n.version)
        }
        _ => return Err(AppError::NotFound),
    };

    let latest_log = store
        .change_log
        .iter()
        .rev()
        .find(|c| normalize_entity_type(&c.entity_type) == et && c.entity_id == id);

    let updated_at = latest_log
        .map(|c| c.created_at.clone())
        .unwrap_or_else(|| "2026-10-27T00:00:00Z".to_string());
    let updated_by = latest_log.and_then(|c| c.actor_id.clone());
    let updated_by_name = latest_log.and_then(|c| c.actor_name.clone());

    Ok(EntityVersionResponse {
        entity_type: et,
        id,
        gid,
        version,
        updated_at,
        updated_by,
        updated_by_name,
    })
}

/// Prunes change log entries older than `history_retention_days` (`0` = keep forever) and expired snapshots (`14 §2, §6 DoD #7`).
pub fn prune_expired_history_and_snapshots(
    store: &mut GenealogyPrivacyStore,
    history_retention_days: u32,
) -> (usize, usize) {
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let before_snaps = store.snapshots.len();
    store.snapshots.retain(|s| s.expires_at > now_str);
    let pruned_snapshots = before_snaps.saturating_sub(store.snapshots.len());

    let pruned_history = if history_retention_days > 0 {
        let cutoff = (now - Duration::days(i64::from(history_retention_days))).to_rfc3339();
        let before_hist = store.change_log.len();
        store.change_log.retain(|c| c.created_at >= cutoff);
        before_hist.saturating_sub(store.change_log.len())
    } else {
        0
    };

    (pruned_history, pruned_snapshots)
}
