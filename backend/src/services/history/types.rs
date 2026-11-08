#[allow(unused_imports)]
use super::*;

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
pub struct StoreSnapshotPayload {
    pub persons: Vec<TreePersonNodeDto>,

    pub families: Vec<FamilyDto>,

    pub events: Vec<LifeEventDto>,

    pub notes: Vec<NoteDto>,

    pub media: Vec<MediaAssetSummaryDto>,

    pub places: Vec<PlaceSummaryDto>,

    pub next_gid_seq: u32,

    pub next_family_gid_seq: u32,

    pub next_event_gid_seq: u32,

    pub next_place_gid_seq: u32,

    pub next_media_gid_seq: u32,
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
    store.persist_if_production();
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

pub(crate) fn normalize_entity_type(raw: &str) -> String {
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

pub(crate) fn resolve_canonical_entity_id(
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
