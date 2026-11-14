#[allow(unused_imports)]
use super::*;
use crate::services::genealogy::{recompute_all_individuals_vitals, recompute_all_places_metadata};
use std::collections::HashSet;

/// Atomically batch-merges eligible duplicate event groups under one Undo `tx_id`.
pub fn batch_merge_duplicate_events_in_store(
    store: &mut GenealogyPrivacyStore,
    req: BatchMergeDuplicateEventsRequest,
    actor_id: Option<String>,
    actor_name: Option<String>,
    current_year: i32,
    age_threshold: i32,
) -> Result<BatchMergeDuplicateEventsResponse, AppError> {
    let only_high = req.only_high_confidence.unwrap_or(true);
    let requested_ids: Option<HashSet<String>> = req.group_ids.map(|v| {
        v.into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });

    let all_groups = compute_duplicate_event_groups(store, false);
    let selected: Vec<DuplicateEventGroupDto> = all_groups
        .into_iter()
        .filter(|g| {
            if only_high && g.confidence != "high" {
                return false;
            }
            if let Some(ref set) = requested_ids {
                return set.contains(&g.id);
            }
            true
        })
        .collect();

    if selected.is_empty() {
        return Ok(BatchMergeDuplicateEventsResponse {
            status: "ok".to_string(),
            tx_id: None,
            merged_groups_count: 0,
            removed_events_count: 0,
            surviving_event_ids: vec![],
        });
    }

    let target_event_ids: Vec<String> = selected
        .iter()
        .flat_map(|g| g.events.iter().map(|e| e.id.clone()))
        .collect();
    let before_bundle =
        super::events_duplicates_merge::build_event_merge_before_bundle(store, &target_event_ids);

    let mut merged_groups_count = 0usize;
    let mut removed_events_count = 0usize;
    let mut surviving_event_ids: Vec<String> = Vec::new();

    for group in &selected {
        let keep_id = &group.proposed.keep_event_id;
        if !store.events.iter().any(|e| e.id == *keep_id) {
            continue;
        }
        let present_merge_ids: Vec<String> = group
            .events
            .iter()
            .map(|e| e.id.clone())
            .filter(|id| id != keep_id && store.events.iter().any(|e| e.id == *id))
            .collect();
        if present_merge_ids.is_empty() {
            continue;
        }

        let (surviving, _) = super::events_duplicates_merge::apply_single_event_merge(
            store,
            keep_id,
            &present_merge_ids,
            None,
            None,
            None,
            None,
            None,
        )?;
        merged_groups_count += 1;
        removed_events_count += present_merge_ids.len();
        surviving_event_ids.push(surviving.id);
    }

    if merged_groups_count == 0 {
        return Ok(BatchMergeDuplicateEventsResponse {
            status: "ok".to_string(),
            tx_id: None,
            merged_groups_count: 0,
            removed_events_count: 0,
            surviving_event_ids: vec![],
        });
    }

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    let tx_id = Uuid::new_v4().to_string();
    let summary = format!(
        "Batch merged {merged_groups_count} duplicate event group(s) ({removed_events_count} duplicate event(s) removed)"
    );
    log_change(
        store,
        Some(tx_id.clone()),
        "quality.events.batch_merge",
        "event",
        &surviving_event_ids[0],
        "merge",
        Some(format!("Batch merge ({merged_groups_count} groups)")),
        summary,
        Some(before_bundle),
        Some(serde_json::json!({
            "merged_groups_count": merged_groups_count,
            "removed_events_count": removed_events_count,
            "surviving_event_ids": surviving_event_ids,
        })),
        actor_id,
        actor_name,
    );
    store.persist_if_production();

    Ok(BatchMergeDuplicateEventsResponse {
        status: "merged".to_string(),
        tx_id: Some(tx_id),
        merged_groups_count,
        removed_events_count,
        surviving_event_ids,
    })
}

pub fn execute_mcp_list_duplicate_events(
    store: &GenealogyPrivacyStore,
    args: &serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    let query: DuplicateEventsFilterQuery =
        serde_json::from_value(args.clone()).unwrap_or_default();
    let groups = compute_duplicate_event_groups(store, query.include_dismissed.unwrap_or(false));
    let resp = filter_duplicate_event_groups(groups, &query);
    serde_json::to_value(resp).map_err(|e| AppError::Internal(e.to_string()))
}

pub fn execute_mcp_merge_duplicate_events(
    store: &mut GenealogyPrivacyStore,
    args: &serde_json::Value,
    actor_id: Option<String>,
    actor_name: Option<String>,
    current_year: i32,
    age_threshold: i32,
) -> Result<serde_json::Value, AppError> {
    let batch_mode = args.get("batch").and_then(|v| v.as_bool()) == Some(true)
        || (args.get("group_ids").is_some()
            && args.get("group_id").is_none()
            && args.get("keep_event_id").is_none()
            && args.get("target_event_id").is_none()
            && args.get("surviving_event_id").is_none()
            && args.get("merge_event_ids").is_none()
            && args.get("event_ids").is_none()
            && args.get("duplicate_event_ids").is_none()
            && args.get("source_event_ids").is_none());

    if batch_mode {
        let req: BatchMergeDuplicateEventsRequest =
            serde_json::from_value(args.clone()).unwrap_or_default();
        let resp = batch_merge_duplicate_events_in_store(
            store,
            req,
            actor_id,
            actor_name,
            current_year,
            age_threshold,
        )?;
        return serde_json::to_value(resp).map_err(|e| AppError::Internal(e.to_string()));
    }

    let req: MergeDuplicateEventsRequest = serde_json::from_value(args.clone())
        .map_err(|e| AppError::ValidationFailed(format!("Invalid arguments: {e}"), None))?;
    let resp = merge_duplicate_events_in_store(
        store,
        req,
        actor_id,
        actor_name,
        current_year,
        age_threshold,
    )?;
    serde_json::to_value(resp).map_err(|e| AppError::Internal(e.to_string()))
}
