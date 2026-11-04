#[allow(unused_imports)]
use super::*;

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
