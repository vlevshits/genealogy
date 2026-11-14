use serde_json::{json, Value};
use uuid::Uuid;

use super::enrich::{with_gid, HistoryLookupIndex};
use crate::models::gendate::GenDate;

pub(crate) fn format_gendate_value(val: &Value) -> Option<Value> {
    match val {
        Value::Null => None,
        Value::String(s) => Some(json!(s)),
        Value::Object(m) => {
            if let Some(t) = m
                .get("text")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                return Some(json!(t.trim()));
            }
            if let Ok(gd) = serde_json::from_value::<GenDate>(val.clone()) {
                return Some(json!(gd.to_gedcom()));
            }
            m.get("year").and_then(Value::as_i64).map(|y| {
                let s = match (
                    m.get("month").and_then(Value::as_u64),
                    m.get("day").and_then(Value::as_u64),
                ) {
                    (Some(mo), Some(dy)) => format!("{y:04}-{mo:02}-{dy:02}"),
                    (Some(mo), None) => format!("{y:04}-{mo:02}"),
                    _ => y.to_string(),
                };
                json!(s)
            })
        }
        _ => Some(val.clone()),
    }
}

pub(crate) fn format_children_array(val: &Value, index: &HistoryLookupIndex) -> Value {
    let Some(arr) = val.as_array() else {
        return val.clone();
    };
    let out: Vec<String> = arr
        .iter()
        .map(|item| {
            let cid = str_field(item, "child_id").unwrap_or("");
            let name = index.person_label_with_gid(cid);
            let mut rels = Vec::new();
            for key in ["rel_to_partner1", "rel_to_partner2"] {
                if let Some(r) = str_field(item, key).filter(|r| !r.is_empty() && *r != "birth") {
                    if !rels.contains(&r) {
                        rels.push(r);
                    }
                }
            }
            if rels.is_empty() {
                name
            } else {
                format!("{name} [{}]", rels.join("/"))
            }
        })
        .collect();
    json!(out)
}

pub(crate) fn format_alt_names_array(val: &Value) -> Value {
    let Some(arr) = val.as_array() else {
        return val.clone();
    };
    let out: Vec<String> = arr
        .iter()
        .filter_map(|item| {
            if let Some(name) = str_field(item, "name").filter(|s| !s.trim().is_empty()) {
                return Some(
                    match str_field(item, "lang").filter(|l| !l.trim().is_empty()) {
                        Some(lang) => format!("{} [{lang}]", name.trim()),
                        None => name.trim().to_string(),
                    },
                );
            }
            let full = format_name_parts(
                str_field(item, "given_name").unwrap_or(""),
                str_field(item, "patronymic"),
                str_field(item, "surname").unwrap_or(""),
                "",
            );
            if full.is_empty() {
                return None;
            }
            Some(
                match str_field(item, "name_type").filter(|t| !t.is_empty()) {
                    Some(nt) => format!("{full} ({nt})"),
                    None => full,
                },
            )
        })
        .collect();
    json!(out)
}

pub(crate) fn format_face_tags_array(val: &Value, index: &HistoryLookupIndex) -> Value {
    let Some(arr) = val.as_array() else {
        return val.clone();
    };
    let out: Vec<String> = arr
        .iter()
        .map(|item| {
            let pid = str_field(item, "person_id").unwrap_or("");
            let label = if index.has_person(pid) {
                index.person_label_with_gid(pid)
            } else {
                str_field(item, "person_name")
                    .filter(|s| !s.is_empty())
                    .unwrap_or("Unknown")
                    .to_string()
            };
            if item.get("is_primary_avatar").and_then(Value::as_bool) == Some(true) {
                format!("{label} [primary avatar]")
            } else {
                label
            }
        })
        .collect();
    json!(out)
}

pub(crate) fn format_links_array(val: &Value, index: &HistoryLookupIndex) -> Value {
    let Some(arr) = val.as_array() else {
        return val.clone();
    };
    let out: Vec<String> = arr
        .iter()
        .map(|item| {
            let target = index.resolve_fk_label(
                str_field(item, "entity_type").unwrap_or("any"),
                str_field(item, "entity_id").unwrap_or(""),
            );
            match str_field(item, "role").filter(|r| !r.is_empty()) {
                Some(r) => format!("{target} ({r})"),
                None => target,
            }
        })
        .collect();
    json!(out)
}

pub(crate) fn format_fk_array(val: &Value, entity_type: &str, index: &HistoryLookupIndex) -> Value {
    let Some(arr) = val.as_array() else {
        return val.clone();
    };
    let out: Vec<String> = arr
        .iter()
        .filter_map(Value::as_str)
        .map(|id| index.resolve_fk_label(entity_type, id))
        .collect();
    json!(out)
}

pub(crate) fn resolve_opt_fk(val: &Value, entity_type: &str, index: &HistoryLookupIndex) -> Value {
    match val.as_str().map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => json!(index.resolve_fk_label(entity_type, id)),
        None => Value::Null,
    }
}

pub(crate) fn unwrap_deleted_person(val: &Value) -> Option<&Value> {
    val.get("person").filter(|v| v.is_object()).or(Some(val))
}

pub(crate) fn format_event_snap_label(ev: &Value, index: &HistoryLookupIndex) -> String {
    if let Some(lbl) = str_field(ev, "id").and_then(|id| index.event_label(id, true)) {
        return lbl;
    }
    let gid = str_field(ev, "gid").unwrap_or("");
    let et = str_field(ev, "event_type").unwrap_or("event");
    let owner = str_field(ev, "individual_id")
        .and_then(|pid| index.person_name(pid).map(ToString::to_string))
        .or_else(|| str_field(ev, "family_id").and_then(|fid| index.family_partners_label(fid)));
    let base = match owner {
        Some(o) => format!("{et} — {o}"),
        None => et.to_string(),
    };
    with_gid(&base, gid)
}

pub(crate) fn same_event_group(a: &str, b: &str) -> bool {
    fn norm(s: &str) -> String {
        match s.to_ascii_lowercase().as_str() {
            "birt" | "birth" => "birth".to_string(),
            "deat" | "death" => "death".to_string(),
            "buri" | "burial" => "burial".to_string(),
            "marr" | "marriage" => "marriage".to_string(),
            "div" | "divorce" => "divorce".to_string(),
            other => other.to_string(),
        }
    }
    norm(a) == norm(b)
}

pub(crate) fn extract_person_name_from_json(val: &Value, fallback: &str) -> String {
    let pn = val.get("primary_name").unwrap_or(val);
    format_name_parts(
        str_field(pn, "given_name").unwrap_or(""),
        str_field(pn, "patronymic"),
        str_field(pn, "surname").unwrap_or(""),
        fallback,
    )
}

pub(crate) fn format_name_parts(
    given: &str,
    patronymic: Option<&str>,
    surname: &str,
    fallback: &str,
) -> String {
    let mut parts = Vec::new();
    if !given.trim().is_empty() {
        parts.push(given.trim());
    }
    if let Some(pat) = patronymic.map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(pat);
    }
    if !surname.trim().is_empty() {
        parts.push(surname.trim());
    }
    if parts.is_empty() {
        fallback.to_string()
    } else {
        parts.join(" ")
    }
}

pub(crate) fn str_field<'a>(val: &'a Value, key: &str) -> Option<&'a str> {
    val.get(key).and_then(Value::as_str)
}

pub(crate) fn is_uuid(s: &str) -> bool {
    Uuid::parse_str(s.trim()).is_ok()
}

pub(crate) fn safe_fallback_id(id: &str, unknown_label: &str) -> String {
    if is_uuid(id) {
        unknown_label.to_string()
    } else {
        id.to_string()
    }
}

pub(crate) fn is_empty_val(val: &Value) -> bool {
    match val {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(m) => m.is_empty(),
        _ => false,
    }
}

pub(crate) fn is_meaningful_non_empty(key: &str, val: &Value) -> bool {
    if is_empty_val(val) {
        return false;
    }
    !(matches!(key, "is_private" | "is_missing" | "only_living_persons")
        && val == &Value::Bool(false))
}
