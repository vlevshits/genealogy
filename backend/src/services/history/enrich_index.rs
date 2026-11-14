use serde_json::{Map, Value};
use std::collections::HashMap;

use super::{
    enrich_format::{extract_person_name_from_json, format_name_parts, str_field},
    normalize_entity_type,
};
use crate::services::privacy::GenealogyPrivacyStore;

#[derive(Debug, Clone)]
pub(super) struct PersonMeta {
    pub(super) gid: String,
    pub(super) name: String,
}

#[derive(Debug, Clone)]
pub(super) struct FamilyMeta {
    pub(super) gid: String,
    pub(super) partner1_id: Option<String>,
    pub(super) partner2_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct EventMeta {
    pub(super) gid: String,
    pub(super) event_type: String,
    pub(super) custom_type: Option<String>,
    pub(super) individual_id: Option<String>,
    pub(super) family_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct PlaceMeta {
    pub(super) gid: String,
    pub(super) name: String,
    pub(super) full_name: String,
}

#[derive(Debug, Clone)]
pub(super) struct MediaMeta {
    pub(super) gid: String,
    pub(super) title: String,
}

#[derive(Debug, Clone)]
pub(super) struct NoteMeta {
    pub(super) entity_type: String,
    pub(super) entity_id: String,
}

#[derive(Debug, Clone)]
pub(super) struct TxMeta {
    pub(super) tx_label: String,
    pub(super) primary_type: String,
    pub(super) primary_id: String,
    pub(super) raw_label: Option<String>,
}

/// Lookup index across live store entities and historical `change_log` snapshots.
#[derive(Debug, Clone, Default)]
pub struct HistoryLookupIndex {
    pub(super) persons: HashMap<String, PersonMeta>,
    pub(super) families: HashMap<String, FamilyMeta>,
    pub(super) events: HashMap<String, EventMeta>,
    pub(super) places: HashMap<String, PlaceMeta>,
    pub(super) media: HashMap<String, MediaMeta>,
    pub(super) notes: HashMap<String, NoteMeta>,
    pub(super) snapshots: HashMap<String, String>,
    pub(super) transactions: HashMap<String, TxMeta>,
    pub(super) gid_to_id: HashMap<(String, String), String>,
}

impl HistoryLookupIndex {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Indexes historical `change_log` snapshots first, then overlays live store records.
    pub fn from_store(store: &GenealogyPrivacyStore) -> Self {
        let mut idx = Self::default();
        for entry in &store.change_log {
            let et = normalize_entity_type(&entry.entity_type);
            if et != "transaction" {
                let meta = TxMeta {
                    tx_label: entry.tx_label.clone(),
                    primary_type: et.clone(),
                    primary_id: entry.entity_id.clone(),
                    raw_label: entry.entity_label.clone(),
                };
                if !entry.tx_id.is_empty() {
                    idx.transactions
                        .entry(entry.tx_id.clone())
                        .or_insert_with(|| meta.clone());
                }
                if !entry.id.is_empty() {
                    idx.transactions.entry(entry.id.clone()).or_insert(meta);
                }
            }
            if let Some(ref b) = entry.before_json {
                idx.index_snapshot_value(&et, b);
            }
            if let Some(ref a) = entry.after_json {
                idx.index_snapshot_value(&et, a);
            }
        }
        for p in &store.persons {
            let name = format_name_parts(
                &p.primary_name.given_name,
                p.primary_name.patronymic.as_deref(),
                &p.primary_name.surname,
                &p.gid,
            );
            idx.insert_person(&p.id, &p.gid, name);
        }
        for f in &store.families {
            idx.insert_family(&f.id, &f.gid, f.partner1_id.clone(), f.partner2_id.clone());
        }
        for e in &store.events {
            idx.insert_event(
                &e.id,
                &e.gid,
                &e.event_type,
                e.custom_type.clone(),
                e.individual_id.clone(),
                e.family_id.clone(),
            );
        }
        for p in &store.places {
            idx.insert_place(&p.id, &p.gid, p.name.clone(), p.full_name.clone());
        }
        for m in &store.media {
            idx.insert_media(&m.id, &m.gid, m.title.clone());
        }
        for n in &store.notes {
            idx.notes.insert(
                n.id.clone(),
                NoteMeta {
                    entity_type: normalize_entity_type(&n.entity_type),
                    entity_id: n.entity_id.clone(),
                },
            );
        }
        for s in &store.snapshots {
            if !s.label.trim().is_empty() {
                idx.snapshots.insert(s.id.clone(), s.label.clone());
            }
        }
        idx
    }

    fn record_gid(&mut self, et: &str, gid: &str, id: &str) {
        if !gid.is_empty() {
            self.gid_to_id
                .insert((et.to_string(), gid.to_ascii_uppercase()), id.to_string());
        }
    }

    fn insert_person(&mut self, id: &str, gid: &str, name: String) {
        if !id.is_empty() {
            self.record_gid("individual", gid, id);
            self.persons.insert(
                id.to_string(),
                PersonMeta {
                    gid: gid.to_string(),
                    name,
                },
            );
        }
    }

    fn insert_family(&mut self, id: &str, gid: &str, p1: Option<String>, p2: Option<String>) {
        if !id.is_empty() {
            self.record_gid("family", gid, id);
            self.families.insert(
                id.to_string(),
                FamilyMeta {
                    gid: gid.to_string(),
                    partner1_id: p1,
                    partner2_id: p2,
                },
            );
        }
    }

    fn insert_event(
        &mut self,
        id: &str,
        gid: &str,
        event_type: &str,
        custom_type: Option<String>,
        individual_id: Option<String>,
        family_id: Option<String>,
    ) {
        if !id.is_empty() {
            self.record_gid("event", gid, id);
            self.events.insert(
                id.to_string(),
                EventMeta {
                    gid: gid.to_string(),
                    event_type: event_type.to_string(),
                    custom_type,
                    individual_id,
                    family_id,
                },
            );
        }
    }

    fn insert_place(&mut self, id: &str, gid: &str, name: String, full_name: String) {
        if !id.is_empty() {
            self.record_gid("place", gid, id);
            self.places.insert(
                id.to_string(),
                PlaceMeta {
                    gid: gid.to_string(),
                    name,
                    full_name,
                },
            );
        }
    }

    fn insert_media(&mut self, id: &str, gid: &str, title: String) {
        if !id.is_empty() {
            self.record_gid("media", gid, id);
            self.media.insert(
                id.to_string(),
                MediaMeta {
                    gid: gid.to_string(),
                    title,
                },
            );
        }
    }

    fn index_snapshot_value(&mut self, entity_type: &str, val: &Value) {
        let Some(obj) = val.as_object() else {
            return;
        };
        let kind = obj.get("kind").and_then(Value::as_str);
        if kind == Some("person_merge_bundle") {
            if let Some(kb) = obj.get("keep_before") {
                self.index_person_json(kb);
            }
            if let Some(mp) = obj.get("merge_person") {
                self.index_person_json(mp);
            }
            for (k, et) in [
                ("families_before", "family"),
                ("events_before", "event"),
                ("notes_before", "note"),
                ("media_before", "media"),
            ] {
                self.index_array_field(obj, k, et);
            }
            return;
        }
        if kind == Some("event_merge_bundle") {
            for (k, et) in [
                ("events_before", "event"),
                ("families_before", "family"),
                ("persons_before", "individual"),
                ("notes_before", "note"),
                ("media_before", "media"),
            ] {
                self.index_array_field(obj, k, et);
            }
            return;
        }
        if let Some(person_val) = obj.get("person").filter(|v| v.is_object()) {
            self.index_person_json(person_val);
            self.index_array_field(obj, "events", "event");
            self.index_array_field(obj, "families_before", "family");
            self.index_array_field(obj, "notes", "note");
            return;
        }
        match entity_type {
            "individual" => self.index_person_json(val),
            "family" => self.index_family_json(val),
            "event" => self.index_event_json(val),
            "place" => self.index_place_json(val),
            "media" => self.index_media_json(val),
            "note" => self.index_note_json(val),
            _ => {}
        }
    }

    fn index_array_field(&mut self, obj: &Map<String, Value>, key: &str, et: &str) {
        if let Some(arr) = obj.get(key).and_then(Value::as_array) {
            for item in arr {
                self.index_snapshot_value(et, item);
            }
        }
    }

    fn index_person_json(&mut self, val: &Value) {
        let Some(id) = str_field(val, "id") else {
            return;
        };
        let gid = str_field(val, "gid").unwrap_or("");
        self.insert_person(id, gid, extract_person_name_from_json(val, gid));
    }

    fn index_family_json(&mut self, val: &Value) {
        let Some(id) = str_field(val, "id") else {
            return;
        };
        let gid = str_field(val, "gid").unwrap_or("");
        self.insert_family(
            id,
            gid,
            str_field(val, "partner1_id").map(ToString::to_string),
            str_field(val, "partner2_id").map(ToString::to_string),
        );
    }

    fn index_event_json(&mut self, val: &Value) {
        let (Some(id), Some(ev_type)) = (str_field(val, "id"), str_field(val, "event_type")) else {
            return;
        };
        let gid = str_field(val, "gid").unwrap_or("");
        self.insert_event(
            id,
            gid,
            ev_type,
            str_field(val, "custom_type").map(ToString::to_string),
            str_field(val, "individual_id").map(ToString::to_string),
            str_field(val, "family_id").map(ToString::to_string),
        );
    }

    fn index_place_json(&mut self, val: &Value) {
        let Some(id) = str_field(val, "id") else {
            return;
        };
        let gid = str_field(val, "gid").unwrap_or("");
        let name = str_field(val, "name").unwrap_or(gid).to_string();
        let full = str_field(val, "full_name")
            .filter(|s| !s.is_empty())
            .unwrap_or(&name)
            .to_string();
        self.insert_place(id, gid, name, full);
    }

    fn index_media_json(&mut self, val: &Value) {
        let Some(id) = str_field(val, "id") else {
            return;
        };
        let gid = str_field(val, "gid").unwrap_or("");
        let title = str_field(val, "title").unwrap_or(gid).to_string();
        self.insert_media(id, gid, title);
    }

    fn index_note_json(&mut self, val: &Value) {
        let Some(id) = str_field(val, "id") else {
            return;
        };
        let et = str_field(val, "entity_type").unwrap_or("individual");
        let eid = str_field(val, "entity_id").unwrap_or("");
        self.notes.insert(
            id.to_string(),
            NoteMeta {
                entity_type: normalize_entity_type(et),
                entity_id: eid.to_string(),
            },
        );
    }
}
