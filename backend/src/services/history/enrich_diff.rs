use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashSet};

use super::{
    enrich::{with_gid, HistoryLookupIndex},
    enrich_format::{
        extract_person_name_from_json, format_alt_names_array, format_children_array,
        format_event_snap_label, format_face_tags_array, format_fk_array, format_gendate_value,
        format_links_array, is_empty_val, is_meaningful_non_empty, resolve_opt_fk,
        safe_fallback_id, same_event_group, str_field, unwrap_deleted_person,
    },
    FieldDiffDto,
};
use crate::models::gendate::GenDate;

const NOISE_KEYS: &[&str] = &[
    "id",
    "gid",
    "version",
    "tx_id",
    "created_at",
    "updated_at",
    "birth_sort",
    "death_sort",
    "date_sort",
    "has_death_event",
    "kind",
    "keep_before",
    "merge_person",
    "families_before",
    "events_before",
    "notes_before",
    "media_before",
    "persons_before",
    "gid_aliases_before",
    "events_count",
    "children_count",
    "sha256",
    "file_url",
    "avatar_crop",
    "avatar_url",
    "generation",
    "is_direct_lineage",
];

pub(crate) fn compute_diffs_with_index(
    before: Option<&Value>,
    after: Option<&Value>,
    index: &HistoryLookupIndex,
) -> Vec<FieldDiffDto> {
    if let Some(b_val) = before {
        match b_val.get("kind").and_then(Value::as_str) {
            Some("person_merge_bundle") => return diff_person_merge(b_val, after, index),
            Some("event_merge_bundle") => return diff_event_merge(b_val, after, index),
            _ => {}
        }
    }

    match (before, after) {
        (Some(b), Some(a)) => {
            let b_norm = normalize_snapshot_object(unwrap_deleted_person(b).unwrap_or(b), index);
            let a_norm = normalize_snapshot_object(unwrap_deleted_person(a).unwrap_or(a), index);
            diff_normalized_maps(&b_norm, &a_norm)
        }
        (None, Some(a)) => normalize_snapshot_object(unwrap_deleted_person(a).unwrap_or(a), index)
            .into_iter()
            .filter(|(k, v)| is_meaningful_non_empty(k, v))
            .map(|(k, v)| FieldDiffDto {
                field: k,
                before: Value::Null,
                after: v,
            })
            .collect(),
        (Some(b), None) => normalize_snapshot_object(unwrap_deleted_person(b).unwrap_or(b), index)
            .into_iter()
            .filter(|(k, v)| is_meaningful_non_empty(k, v))
            .map(|(k, v)| FieldDiffDto {
                field: k,
                before: v,
                after: Value::Null,
            })
            .collect(),
        (None, None) => Vec::new(),
    }
}

fn diff_person_merge(
    bundle: &Value,
    after: Option<&Value>,
    index: &HistoryLookupIndex,
) -> Vec<FieldDiffDto> {
    let mut diffs = Vec::new();
    let keep_before = bundle.get("keep_before");
    if let (Some(mp), Some(kb)) = (bundle.get("merge_person"), keep_before) {
        let m_gid = str_field(mp, "gid").unwrap_or("");
        let k_gid = str_field(kb, "gid").unwrap_or("");
        diffs.push(FieldDiffDto {
            field: "merged_profile".to_string(),
            before: json!(with_gid(&extract_person_name_from_json(mp, m_gid), m_gid)),
            after: json!(with_gid(&extract_person_name_from_json(kb, k_gid), k_gid)),
        });
    }
    if let (Some(kb), Some(a)) = (keep_before, after) {
        let b_norm = normalize_snapshot_object(kb, index);
        let a_norm = normalize_snapshot_object(a, index);
        diffs.extend(diff_normalized_maps(&b_norm, &a_norm));
    }
    diffs
}

fn diff_event_merge(
    bundle: &Value,
    after: Option<&Value>,
    index: &HistoryLookupIndex,
) -> Vec<FieldDiffDto> {
    let mut diffs = Vec::new();
    let events_before = bundle
        .get("events_before")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let is_batch = after.is_some_and(|a| {
        a.get("surviving_event_ids").is_some() || a.get("merged_groups_count").is_some()
    });
    if is_batch {
        if let Some(a) = after {
            if let Some(cnt) = a.get("merged_groups_count").cloned() {
                diffs.push(FieldDiffDto {
                    field: "merged_groups_count".to_string(),
                    before: Value::Null,
                    after: cnt,
                });
            }
            let surviving_ids: Vec<&str> = a
                .get("surviving_event_ids")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let surviving_set: HashSet<&str> = surviving_ids.iter().copied().collect();
            let before_labels: Vec<String> = events_before
                .iter()
                .filter(|ev| str_field(ev, "id").is_some_and(|id| !surviving_set.contains(id)))
                .map(|ev| format_event_snap_label(ev, index))
                .collect();
            let after_labels: Vec<String> = surviving_ids
                .iter()
                .map(|id| index.resolve_fk_label("event", id))
                .collect();
            diffs.push(FieldDiffDto {
                field: "merged_events".to_string(),
                before: if before_labels.is_empty() {
                    Value::Null
                } else {
                    json!(before_labels)
                },
                after: json!(after_labels),
            });
        }
        return diffs;
    }

    if let Some(a) = after {
        let keep_id = str_field(a, "id").unwrap_or("");
        let keep_type = str_field(a, "event_type").unwrap_or("");
        let keep_before = events_before
            .iter()
            .find(|ev| str_field(ev, "id") == Some(keep_id));
        let source_labels: Vec<String> = events_before
            .iter()
            .filter(|ev| {
                str_field(ev, "id") != Some(keep_id)
                    && (keep_type.is_empty()
                        || str_field(ev, "event_type")
                            .is_some_and(|et| same_event_group(et, keep_type)))
            })
            .map(|ev| format_event_snap_label(ev, index))
            .collect();
        diffs.push(FieldDiffDto {
            field: "merged_events".to_string(),
            before: if source_labels.is_empty() {
                Value::Null
            } else {
                json!(source_labels)
            },
            after: json!(format_event_snap_label(a, index)),
        });
        if let Some(kb) = keep_before {
            let b_norm = normalize_snapshot_object(kb, index);
            let a_norm = normalize_snapshot_object(a, index);
            diffs.extend(diff_normalized_maps(&b_norm, &a_norm));
        }
    }
    diffs
}

fn diff_normalized_maps(
    b_map: &Map<String, Value>,
    a_map: &Map<String, Value>,
) -> Vec<FieldDiffDto> {
    let keys: BTreeSet<&String> = b_map.keys().chain(a_map.keys()).collect();
    let mut diffs = Vec::new();
    for key in keys {
        let b_val = b_map.get(key).cloned().unwrap_or(Value::Null);
        let a_val = a_map.get(key).cloned().unwrap_or(Value::Null);
        if b_val != a_val && !(is_empty_val(&b_val) && is_empty_val(&a_val)) {
            diffs.push(FieldDiffDto {
                field: key.clone(),
                before: b_val,
                after: a_val,
            });
        }
    }
    diffs
}

fn normalize_snapshot_object(val: &Value, index: &HistoryLookupIndex) -> Map<String, Value> {
    let mut out = Map::new();
    let Some(obj) = val.as_object() else {
        return out;
    };
    let has_place_name = obj
        .get("place_full_name")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.trim().is_empty());
    let record_entity_type = str_field(val, "entity_type");
    let date_formatted = obj.get("date").and_then(format_gendate_value);
    let date_gedcom_cmp = obj
        .get("date")
        .and_then(|d| serde_json::from_value::<GenDate>(d.clone()).ok())
        .map(|gd| gd.to_gedcom());

    for (key, raw_v) in obj {
        if NOISE_KEYS.contains(&key.as_str()) {
            continue;
        }
        match key.as_str() {
            "primary_name" => {
                if let Some(pn) = raw_v.as_object() {
                    for sub in ["given_name", "patronymic", "surname"] {
                        if !obj.contains_key(sub) {
                            if let Some(sv) = pn.get(sub) {
                                out.insert(sub.to_string(), sv.clone());
                            }
                        }
                    }
                }
            }
            "birth" | "death" => {
                if let Some(vo) = raw_v.as_object() {
                    if let Some(d) = vo.get("date") {
                        out.insert(
                            format!("{key}_date"),
                            format_gendate_value(d).unwrap_or_else(|| d.clone()),
                        );
                    }
                    if let Some(p) = vo.get("place") {
                        out.insert(format!("{key}_place"), p.clone());
                    }
                }
            }
            "date" => {
                let v = date_formatted
                    .clone()
                    .or_else(|| {
                        obj.get("gedcom_date")
                            .and_then(Value::as_str)
                            .map(|s| json!(s))
                    })
                    .unwrap_or(Value::Null);
                out.insert("date".to_string(), v);
            }
            "gedcom_date" => {
                if let Some(g_str) = raw_v.as_str() {
                    let same_as_text = date_formatted
                        .as_ref()
                        .and_then(Value::as_str)
                        .is_some_and(|d| d.trim().eq_ignore_ascii_case(g_str.trim()));
                    let same_as_gedcom = date_gedcom_cmp
                        .as_deref()
                        .is_some_and(|g| g.eq_ignore_ascii_case(g_str.trim()));
                    if !(same_as_text || same_as_gedcom || date_formatted.is_none()) {
                        out.insert("gedcom_date".to_string(), raw_v.clone());
                    }
                }
            }
            "place_id" => {
                if has_place_name || obj.contains_key("place_full_name") {
                    continue;
                }
                out.insert(key.clone(), resolve_opt_fk(raw_v, "place", index));
            }
            "place_full_name" => {
                let fallback = if raw_v.is_null() {
                    obj.get("place_id")
                        .and_then(Value::as_str)
                        .filter(|s| !s.trim().is_empty())
                        .map(|pid| json!(index.place_label_with_gid(pid)))
                } else {
                    None
                };
                out.insert(key.clone(), fallback.unwrap_or_else(|| raw_v.clone()));
            }
            "partner1_id" | "partner2_id" | "individual_id" => {
                out.insert(key.clone(), resolve_opt_fk(raw_v, "individual", index));
            }
            "family_id" => {
                out.insert(key.clone(), resolve_opt_fk(raw_v, "family", index));
            }
            "parent_id" => {
                out.insert(key.clone(), resolve_opt_fk(raw_v, "place", index));
            }
            "avatar_media_id" => {
                out.insert(key.clone(), resolve_opt_fk(raw_v, "media", index));
            }
            "entity_id" => {
                let et = record_entity_type.unwrap_or("any");
                out.insert(key.clone(), resolve_opt_fk(raw_v, et, index));
            }
            "children" => {
                out.insert(key.clone(), format_children_array(raw_v, index));
            }
            "alt_names" => {
                out.insert(key.clone(), format_alt_names_array(raw_v));
            }
            "face_tags" => {
                out.insert(key.clone(), format_face_tags_array(raw_v, index));
            }
            "links" => {
                out.insert(key.clone(), format_links_array(raw_v, index));
            }
            "linked_person_ids" => {
                out.insert(key.clone(), format_fk_array(raw_v, "individual", index));
            }
            "media_ids" => {
                out.insert(key.clone(), format_fk_array(raw_v, "media", index));
            }
            "restored_entity_ids" => {
                out.insert(key.clone(), format_fk_array(raw_v, "any", index));
            }
            "external_ids" => {
                if let Some(m) = raw_v.as_object() {
                    let items: Vec<String> = m
                        .iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| format!("{k}: {s}")))
                        .collect();
                    out.insert(key.clone(), json!(items));
                }
            }
            "undone_tx_id" => {
                let v = raw_v
                    .as_str()
                    .map(|tid| {
                        index
                            .resolve_tx_label(tid)
                            .unwrap_or_else(|| safe_fallback_id(tid, "Previous transaction"))
                    })
                    .map(|s| json!(s))
                    .unwrap_or_else(|| raw_v.clone());
                out.insert(key.clone(), v);
            }
            "restored_snapshot_id" | "safety_snapshot_id" => {
                let v = raw_v
                    .as_str()
                    .map(|sid| {
                        index
                            .snapshot_label(sid)
                            .map(ToString::to_string)
                            .unwrap_or_else(|| safe_fallback_id(sid, "Snapshot"))
                    })
                    .map(|s| json!(s))
                    .unwrap_or_else(|| raw_v.clone());
                out.insert(key.clone(), v);
            }
            "record_counts" => {
                if let Some(m) = raw_v.as_object() {
                    let mut counts = Map::new();
                    for k in [
                        "individuals",
                        "families",
                        "events",
                        "places",
                        "media",
                        "notes",
                    ] {
                        if let Some(n) = m.get(k).and_then(Value::as_u64) {
                            counts.insert(k.to_string(), json!(n));
                        }
                    }
                    if !counts.is_empty() {
                        out.insert(key.clone(), Value::Object(counts));
                    }
                }
            }
            k if k.ends_with("_date") => {
                let formatted = format_gendate_value(raw_v).unwrap_or_else(|| raw_v.clone());
                out.insert(key.clone(), formatted);
            }
            _ => {
                out.insert(key.clone(), raw_v.clone());
            }
        }
    }
    out
}
