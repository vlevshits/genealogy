use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    error::AppError,
    services::{
        genealogy::{recompute_all_individuals_vitals, recompute_all_places_metadata},
        privacy::{
            ChangeLogEntryDto, FamilyDto, GenealogyPrivacyStore, LifeEventDto,
            MediaAssetSummaryDto, NoteDto, PlaceSummaryDto, TreePersonNodeDto, TreeSnapshotDto,
            TreeSnapshotRecordCountsDto,
        },
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct FieldDiffDto {
    pub field: String,
    pub before: Value,
    pub after: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RecordHistoryItemDto {
    pub id: String,
    pub tx_id: String,
    pub tx_label: String,
    pub op: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_name: Option<String>,
    pub summary: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before_json: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after_json: Option<Value>,
    pub diffs: Vec<FieldDiffDto>,
    pub can_revert: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RecordHistoryResponse {
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_label: Option<String>,
    pub items: Vec<RecordHistoryItemDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ChangeLogTransactionGroupDto {
    pub tx_id: String,
    pub tx_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_name: Option<String>,
    pub created_at: String,
    pub summary: String,
    pub primary_op: String,
    #[serde(default)]
    pub entries: Vec<ChangeLogEntryDto>,
    pub changes: Vec<ChangeLogEntryDto>,
    pub can_undo: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct HistoryFeedQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
    pub entity_type: Option<String>,
    pub action: Option<String>,
    pub op: Option<String>,
    pub actor_id: Option<String>,
    pub user_id: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub before: Option<String>,
    pub tx_id: Option<String>,
    pub only_deletions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct HistoryFeedResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    pub items: Vec<ChangeLogEntryDto>,
    pub transactions: Vec<ChangeLogTransactionGroupDto>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct UndoTransactionResponse {
    pub status: String,
    pub tx_id: String,
    pub undone_count: usize,
    pub restored_entity_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreateSnapshotRequest {
    pub label: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SnapshotListResponse {
    pub items: Vec<TreeSnapshotDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct RollbackSnapshotRequest {
    pub confirm_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RollbackSnapshotResponse {
    pub status: String,
    pub restored_snapshot_id: String,
    pub safety_snapshot_id: String,
    pub record_counts: TreeSnapshotRecordCountsDto,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct EntityVersionResponse {
    pub entity_type: String,
    pub id: String,
    pub gid: String,
    pub version: i32,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedPersonSnapshotPayload {
    pub person: TreePersonNodeDto,
    #[serde(default)]
    pub events: Vec<LifeEventDto>,
    #[serde(default)]
    pub notes: Vec<NoteDto>,
    #[serde(default)]
    pub families_before: Vec<FamilyDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreSnapshotPayload {
    persons: Vec<TreePersonNodeDto>,
    families: Vec<FamilyDto>,
    events: Vec<LifeEventDto>,
    notes: Vec<NoteDto>,
    media: Vec<MediaAssetSummaryDto>,
    places: Vec<PlaceSummaryDto>,
    next_gid_seq: u32,
    next_family_gid_seq: u32,
    next_event_gid_seq: u32,
    next_place_gid_seq: u32,
    next_media_gid_seq: u32,
}

/// Logs an atomic mutation to `store.change_log` (`14-change-history-concurrency.md §2, §6 DoD #1`).
#[allow(clippy::too_many_arguments)]
pub fn log_change(
    store: &mut GenealogyPrivacyStore,
    tx_id: Option<String>,
    tx_label: &str,
    entity_type: &str,
    entity_id: &str,
    op: &str,
    entity_label: Option<String>,
    summary: String,
    before_json: Option<Value>,
    after_json: Option<Value>,
    actor_id: Option<String>,
    actor_name: Option<String>,
) -> ChangeLogEntryDto {
    let resolved_tx_id = tx_id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let entry = ChangeLogEntryDto {
        id: Uuid::new_v4().to_string(),
        tx_id: resolved_tx_id,
        tx_label: tx_label.to_string(),
        op: op.to_string(),
        action: op.to_string(),
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        entity_label,
        actor_id,
        actor_name,
        summary,
        before_json,
        after_json,
        created_at: Utc::now().to_rfc3339(),
    };
    store.change_log.push(entry.clone());
    entry
}

/// Verifies optimistic concurrency version (`14-change-history-concurrency.md §4, §6 DoD #5`).
/// When `expected_version` differs from `current_version`, returns `409 Conflict` (`version_conflict`)
/// with `their_data` (the current server record) and `current_version` in `error.details`.
pub fn verify_optimistic_version(
    current_version: i32,
    expected_version: Option<i32>,
    entity_label: &str,
    current_record: Value,
) -> Result<(), AppError> {
    if let Some(req_ver) = expected_version {
        if req_ver != current_version {
            return Err(AppError::VersionConflictWithRecord(
                format!(
                    "{entity_label} has been modified by another user (expected version {req_ver}, current version {current_version})"
                ),
                json!({
                    "expected_version": req_ver,
                    "current_version": current_version,
                    "their_data": current_record,
                }),
            ));
        }
    }
    Ok(())
}

/// Extracts an optional `If-Match` version integer from HTTP headers (`14 §4`).
pub fn parse_if_match_version(headers: &axum::http::HeaderMap) -> Option<i32> {
    headers
        .get(axum::http::header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().trim_start_matches("W/").trim_matches('"').trim())
        .and_then(|s| s.parse::<i32>().ok())
}

/// Computes field-level diffs between `before_json` and `after_json` (`14 §3.2`).
pub fn compute_field_diffs(before: Option<&Value>, after: Option<&Value>) -> Vec<FieldDiffDto> {
    let mut diffs = Vec::new();
    if let (Some(Value::Object(b_map)), Some(Value::Object(a_map))) = (before, after) {
        let mut all_keys: Vec<&String> = b_map.keys().chain(a_map.keys()).collect();
        all_keys.sort();
        all_keys.dedup();
        for key in all_keys {
            if matches!(key.as_str(), "version" | "id" | "birth_sort" | "death_sort") {
                continue;
            }
            let b_val = b_map.get(key).cloned().unwrap_or(Value::Null);
            let a_val = a_map.get(key).cloned().unwrap_or(Value::Null);
            if b_val != a_val {
                diffs.push(FieldDiffDto {
                    field: key.clone(),
                    before: b_val,
                    after: a_val,
                });
            }
        }
    }
    diffs
}

fn normalize_entity_type(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "person" | "persons" | "individual" | "individuals" => "individual".to_string(),
        "family" | "families" => "family".to_string(),
        "event" | "events" | "life_event" | "life_events" => "event".to_string(),
        "place" | "places" => "place".to_string(),
        "media" | "media_asset" | "media_assets" => "media".to_string(),
        "note" | "notes" => "note".to_string(),
        other => other.to_string(),
    }
}

fn resolve_canonical_entity_id(
    store: &GenealogyPrivacyStore,
    entity_type: &str,
    raw_id: &str,
) -> String {
    let trimmed = raw_id.trim();
    match entity_type {
        "individual" => store
            .persons
            .iter()
            .find(|p| p.id == trimmed || p.gid.eq_ignore_ascii_case(trimmed))
            .map(|p| p.id.clone())
            .unwrap_or_else(|| trimmed.to_string()),
        "family" => store
            .families
            .iter()
            .find(|f| f.id == trimmed || f.gid.eq_ignore_ascii_case(trimmed))
            .map(|f| f.id.clone())
            .unwrap_or_else(|| trimmed.to_string()),
        "event" => store
            .events
            .iter()
            .find(|e| e.id == trimmed || e.gid.eq_ignore_ascii_case(trimmed))
            .map(|e| e.id.clone())
            .unwrap_or_else(|| trimmed.to_string()),
        "place" => store
            .places
            .iter()
            .find(|p| p.id == trimmed || p.gid.eq_ignore_ascii_case(trimmed))
            .map(|p| p.id.clone())
            .unwrap_or_else(|| trimmed.to_string()),
        "media" => store
            .media
            .iter()
            .find(|m| m.id == trimmed || m.gid.eq_ignore_ascii_case(trimmed))
            .map(|m| m.id.clone())
            .unwrap_or_else(|| trimmed.to_string()),
        _ => trimmed.to_string(),
    }
}

/// Queries the global change log feed with filtering, pagination, and transaction grouping (`14 §3.1, §5`).
pub fn query_history_feed(
    store: &GenealogyPrivacyStore,
    query: &HistoryFeedQuery,
) -> HistoryFeedResponse {
    let entity_type_filter = query
        .entity_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_entity_type);
    let op_filter = query
        .action
        .as_deref()
        .or(query.op.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase());
    let actor_filter = query
        .actor_id
        .as_deref()
        .or(query.user_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let tx_filter = query
        .tx_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let from_filter = query
        .from_date
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let to_filter = query
        .to_date
        .as_deref()
        .or(query.before.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let only_deletions = query.only_deletions.unwrap_or(false);

    // Filter in reverse chronological order (newest first)
    let mut filtered: Vec<ChangeLogEntryDto> = store
        .change_log
        .iter()
        .rev()
        .filter(|entry| {
            if let Some(ref et) = entity_type_filter {
                if normalize_entity_type(&entry.entity_type) != *et {
                    return false;
                }
            }
            if only_deletions && entry.op != "delete" {
                return false;
            }
            if let Some(ref op) = op_filter {
                if !entry.op.eq_ignore_ascii_case(op) && !entry.action.eq_ignore_ascii_case(op) {
                    return false;
                }
            }
            if let Some(actor) = actor_filter {
                if entry.actor_id.as_deref() != Some(actor) {
                    return false;
                }
            }
            if let Some(tx) = tx_filter {
                if entry.tx_id != tx && entry.id != tx {
                    return false;
                }
            }
            if let Some(from_dt) = from_filter {
                if entry.created_at.as_str() < from_dt {
                    return false;
                }
            }
            if let Some(to_dt) = to_filter {
                if entry.created_at.as_str() > to_dt {
                    return false;
                }
            }
            true
        })
        .cloned()
        .map(|mut e| {
            if e.tx_id.is_empty() {
                e.tx_id = e.id.clone();
            }
            if e.action.is_empty() {
                e.action = e.op.clone();
            }
            e
        })
        .collect();

    let total = filtered.len();
    let page_size = query.page_size.or(query.limit).unwrap_or(50).clamp(1, 200);
    let page = query.page.unwrap_or(1).max(1);
    let offset = if let Some(ref c) = query.cursor {
        c.trim().parse::<usize>().unwrap_or(0)
    } else {
        (page - 1) * page_size
    };

    let page_items: Vec<ChangeLogEntryDto> =
        filtered.drain(..).skip(offset).take(page_size).collect();

    let next_cursor = if offset + page_items.len() < total {
        Some((offset + page_items.len()).to_string())
    } else {
        None
    };

    // Group page_items into transactions (`tx_id`) preserving chronological order of first appearance
    let mut tx_groups: Vec<ChangeLogTransactionGroupDto> = Vec::new();
    let mut tx_index: HashMap<String, usize> = HashMap::new();

    for item in &page_items {
        let tx_key = if item.tx_id.is_empty() {
            item.id.clone()
        } else {
            item.tx_id.clone()
        };
        if let Some(&idx) = tx_index.get(&tx_key) {
            tx_groups[idx].entries.push(item.clone());
            tx_groups[idx].changes.push(item.clone());
        } else {
            let idx = tx_groups.len();
            tx_index.insert(tx_key.clone(), idx);
            let changes = vec![item.clone()];
            tx_groups.push(ChangeLogTransactionGroupDto {
                tx_id: tx_key,
                tx_label: if item.tx_label.is_empty() {
                    format!("{}.{}", item.entity_type, item.op)
                } else {
                    item.tx_label.clone()
                },
                actor_id: item.actor_id.clone(),
                actor_name: item.actor_name.clone(),
                created_at: item.created_at.clone(),
                summary: item.summary.clone(),
                primary_op: item.op.clone(),
                entries: changes.clone(),
                changes,
                can_undo: matches!(item.op.as_str(), "create" | "update" | "delete"),
            });
        }
    }

    HistoryFeedResponse {
        status: "ok".to_string(),
        operation: "audit_history".to_string(),
        message: "Global change history feed authorized for administrator".to_string(),
        items: page_items,
        transactions: tx_groups,
        total,
        page,
        page_size,
        next_cursor,
    }
}

/// Queries the chronological history of a single record with field-level diffs (`14 §3.2, §5`).
pub fn query_record_history(
    store: &GenealogyPrivacyStore,
    raw_entity_type: &str,
    raw_entity_id: &str,
) -> Result<RecordHistoryResponse, AppError> {
    let entity_type = normalize_entity_type(raw_entity_type);
    let resolved_id = resolve_canonical_entity_id(store, &entity_type, raw_entity_id);

    let items: Vec<RecordHistoryItemDto> = store
        .change_log
        .iter()
        .rev()
        .filter(|e| {
            normalize_entity_type(&e.entity_type) == entity_type
                && (e.entity_id == resolved_id
                    || e.entity_id.eq_ignore_ascii_case(raw_entity_id.trim()))
        })
        .map(|e| {
            let diffs = compute_field_diffs(e.before_json.as_ref(), e.after_json.as_ref());
            RecordHistoryItemDto {
                id: e.id.clone(),
                tx_id: if e.tx_id.is_empty() {
                    e.id.clone()
                } else {
                    e.tx_id.clone()
                },
                tx_label: e.tx_label.clone(),
                op: e.op.clone(),
                action: if e.action.is_empty() {
                    e.op.clone()
                } else {
                    e.action.clone()
                },
                entity_type: entity_type.clone(),
                entity_id: e.entity_id.clone(),
                entity_label: e.entity_label.clone(),
                actor_id: e.actor_id.clone(),
                actor_name: e.actor_name.clone(),
                summary: e.summary.clone(),
                created_at: e.created_at.clone(),
                before_json: e.before_json.clone(),
                after_json: e.after_json.clone(),
                diffs,
                can_revert: matches!(e.op.as_str(), "update" | "delete") && e.before_json.is_some(),
            }
        })
        .collect();

    let entity_label = items.iter().find_map(|i| i.entity_label.clone());
    let total = items.len();

    Ok(RecordHistoryResponse {
        entity_type,
        entity_id: resolved_id,
        entity_label,
        items,
        total,
    })
}

/// Reverts a single change log entry in reverse (`create` -> delete, `update` -> restore `before_json`, `delete` -> recreate from `before_json`).
fn revert_single_change_entry(
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
