use serde_json::Value;

pub use super::enrich_index::HistoryLookupIndex;
use super::{
    enrich_diff::compute_diffs_with_index,
    enrich_format::{
        extract_person_name_from_json, is_uuid, safe_fallback_id, str_field, unwrap_deleted_person,
    },
    normalize_entity_type, FieldDiffDto,
};
use crate::services::privacy::{ChangeLogEntryDto, GenealogyPrivacyStore};

impl HistoryLookupIndex {
    pub(crate) fn resolve_id_by_gid(&self, entity_type: &str, raw_id: &str) -> Option<String> {
        let key = (
            normalize_entity_type(entity_type),
            raw_id.trim().to_ascii_uppercase(),
        );
        self.gid_to_id.get(&key).cloned()
    }

    pub(crate) fn has_person(&self, id: &str) -> bool {
        self.persons.contains_key(id)
    }

    pub(crate) fn snapshot_label(&self, id: &str) -> Option<&str> {
        self.snapshots.get(id).map(String::as_str)
    }

    pub(crate) fn person_name(&self, id: &str) -> Option<&str> {
        self.persons
            .get(id)
            .map(|p| p.name.as_str())
            .filter(|n| !n.is_empty())
    }

    pub(crate) fn person_label_with_gid(&self, id: &str) -> String {
        self.persons
            .get(id)
            .map(|p| with_gid(&p.name, &p.gid))
            .unwrap_or_else(|| safe_fallback_id(id, "Unknown individual"))
    }

    pub(crate) fn family_partners_label(&self, id: &str) -> Option<String> {
        let fam = self.families.get(id)?;
        let p1 = fam.partner1_id.as_deref().and_then(|i| self.person_name(i));
        let p2 = fam.partner2_id.as_deref().and_then(|i| self.person_name(i));
        join_partners(p1, p2)
    }

    pub(crate) fn family_label_with_gid(&self, id: &str) -> String {
        if let Some(fam) = self.families.get(id) {
            if let Some(partners) = self.family_partners_label(id) {
                return with_gid(&partners, &fam.gid);
            }
            if !fam.gid.is_empty() {
                return fam.gid.clone();
            }
        }
        safe_fallback_id(id, "Family")
    }

    pub(crate) fn event_label(&self, id: &str, include_gid: bool) -> Option<String> {
        let ev = self.events.get(id)?;
        let ev_type = ev
            .custom_type
            .as_deref()
            .filter(|c| ev.event_type == "custom" && !c.trim().is_empty())
            .unwrap_or(&ev.event_type);
        let owner = ev
            .individual_id
            .as_deref()
            .and_then(|pid| self.person_name(pid).map(ToString::to_string))
            .or_else(|| {
                ev.family_id
                    .as_deref()
                    .and_then(|fid| self.family_partners_label(fid))
            });
        let base = owner
            .map(|o| format!("{ev_type} — {o}"))
            .unwrap_or_else(|| ev_type.to_string());
        Some(if include_gid {
            with_gid(&base, &ev.gid)
        } else {
            base
        })
    }

    pub(crate) fn place_label_with_gid(&self, id: &str) -> String {
        self.places
            .get(id)
            .map(|p| {
                let lbl = if !p.full_name.is_empty() {
                    &p.full_name
                } else {
                    &p.name
                };
                with_gid(lbl, &p.gid)
            })
            .unwrap_or_else(|| safe_fallback_id(id, "Unknown place"))
    }

    fn media_label_with_gid(&self, id: &str) -> String {
        self.media
            .get(id)
            .map(|m| with_gid(&m.title, &m.gid))
            .unwrap_or_else(|| safe_fallback_id(id, "Unknown media"))
    }

    fn resolve_target_name(&self, entity_type: &str, entity_id: &str) -> Option<String> {
        match normalize_entity_type(entity_type).as_str() {
            "individual" => self.person_name(entity_id).map(ToString::to_string),
            "family" => self
                .family_partners_label(entity_id)
                .or_else(|| self.families.get(entity_id).map(|f| f.gid.clone())),
            "event" => self.event_label(entity_id, false),
            "place" => self.places.get(entity_id).map(|p| p.full_name.clone()),
            "media" => self.media.get(entity_id).map(|m| m.title.clone()),
            _ => None,
        }
    }

    pub(crate) fn resolve_fk_label(&self, entity_type: &str, id: &str) -> String {
        match normalize_entity_type(entity_type).as_str() {
            "individual" => self.person_label_with_gid(id),
            "family" => self.family_label_with_gid(id),
            "event" => self
                .event_label(id, true)
                .unwrap_or_else(|| safe_fallback_id(id, "Unknown event")),
            "place" => self.place_label_with_gid(id),
            "media" => self.media_label_with_gid(id),
            "note" => self
                .notes
                .get(id)
                .and_then(|n| self.resolve_target_name(&n.entity_type, &n.entity_id))
                .map(|t| format!("Note — {t}"))
                .unwrap_or_else(|| safe_fallback_id(id, "Note")),
            _ if self.persons.contains_key(id) => self.person_label_with_gid(id),
            _ if self.families.contains_key(id) => self.family_label_with_gid(id),
            _ if self.events.contains_key(id) => self
                .event_label(id, true)
                .unwrap_or_else(|| safe_fallback_id(id, "Record")),
            _ if self.places.contains_key(id) => self.place_label_with_gid(id),
            _ if self.media.contains_key(id) => self.media_label_with_gid(id),
            _ if self.notes.contains_key(id) => self.resolve_fk_label("note", id),
            _ => safe_fallback_id(id, "Record"),
        }
    }

    pub(crate) fn resolve_tx_label(&self, tx_id: &str) -> Option<String> {
        let meta = self.transactions.get(tx_id)?;
        let target = self
            .resolve_target_name(&meta.primary_type, &meta.primary_id)
            .or_else(|| {
                meta.raw_label
                    .clone()
                    .filter(|s| !s.trim().is_empty() && !is_uuid(s))
            });
        Some(match target {
            Some(t) if !meta.tx_label.is_empty() => format!("{} — {t}", meta.tx_label),
            Some(t) => t,
            None if !meta.tx_label.is_empty() => meta.tx_label.clone(),
            None => "Undo transaction".to_string(),
        })
    }

    pub(crate) fn resolve_entity_gid(
        &self,
        entity_type: &str,
        entity_id: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Option<String> {
        let et = normalize_entity_type(entity_type);
        if matches!(et.as_str(), "tree" | "transaction" | "note") {
            return None;
        }
        let from_snap = |v: Option<&Value>| -> Option<String> {
            let v = unwrap_deleted_person(v?)?;
            str_field(v, "gid")
                .filter(|g| !g.trim().is_empty())
                .map(ToString::to_string)
        };
        from_snap(after).or_else(|| from_snap(before)).or_else(|| {
            match et.as_str() {
                "individual" => self.persons.get(entity_id).map(|p| p.gid.clone()),
                "family" => self.families.get(entity_id).map(|f| f.gid.clone()),
                "event" => self.events.get(entity_id).map(|e| e.gid.clone()),
                "place" => self.places.get(entity_id).map(|p| p.gid.clone()),
                "media" => self.media.get(entity_id).map(|m| m.gid.clone()),
                _ => None,
            }
            .filter(|g| !g.trim().is_empty())
        })
    }

    pub(crate) fn resolve_entity_label(
        &self,
        entity_type: &str,
        entity_id: &str,
        existing_label: Option<&str>,
        tx_label: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Option<String> {
        let et = normalize_entity_type(entity_type);
        let gid = self.resolve_entity_gid(&et, entity_id, before, after);
        let cur = existing_label.map(str::trim).filter(|s| !s.is_empty());
        let is_bare = cur.is_none_or(|l| {
            l == entity_id
                || is_uuid(l)
                || gid.as_deref().is_some_and(|g| l.eq_ignore_ascii_case(g))
        });

        match et.as_str() {
            "individual" => {
                if !is_bare && tx_label != "mcp.update_individual" {
                    return cur.map(ToString::to_string);
                }
                self.person_name(entity_id)
                    .map(ToString::to_string)
                    .or_else(|| {
                        after
                            .or(before)
                            .and_then(unwrap_deleted_person)
                            .map(|v| extract_person_name_from_json(v, gid.as_deref().unwrap_or("")))
                    })
                    .or_else(|| cur.map(ToString::to_string))
            }
            "family" => {
                if !is_bare {
                    return cur.map(|l| strip_trailing_gid(l, gid.as_deref()));
                }
                let partners = after
                    .or(before)
                    .and_then(|v| {
                        let p1 = str_field(v, "partner1_id").and_then(|i| self.person_name(i));
                        let p2 = str_field(v, "partner2_id").and_then(|i| self.person_name(i));
                        join_partners(p1, p2)
                    })
                    .or_else(|| self.family_partners_label(entity_id));
                partners.or(gid).or_else(|| cur.map(ToString::to_string))
            }
            "event" => {
                if tx_label == "quality.events.batch_merge"
                    || cur.is_some_and(|l| l.starts_with("Batch merge"))
                {
                    return cur.map(ToString::to_string);
                }
                if let Some(l) = cur.filter(|l| l.contains(" — ")) {
                    return Some(strip_trailing_gid(l, gid.as_deref()));
                }
                let snap = after
                    .and_then(|v| v.get("event_type").map(|_| v))
                    .or(before);
                let from_snap = snap.and_then(|v| {
                    let ev_type = str_field(v, "custom_type")
                        .filter(|c| str_field(v, "event_type") == Some("custom") && !c.is_empty())
                        .or_else(|| str_field(v, "event_type"))?;
                    let owner = str_field(v, "individual_id")
                        .and_then(|pid| self.person_name(pid).map(ToString::to_string))
                        .or_else(|| {
                            str_field(v, "family_id")
                                .and_then(|fid| self.family_partners_label(fid))
                        });
                    Some(
                        owner
                            .map(|o| format!("{ev_type} — {o}"))
                            .unwrap_or_else(|| ev_type.to_string()),
                    )
                });
                let stripped_cur = cur.map(|l| strip_trailing_gid(l, gid.as_deref()));
                from_snap
                    .or_else(|| self.event_label(entity_id, false))
                    .or(gid)
                    .or(stripped_cur)
            }
            "note" => {
                if cur.is_some_and(|l| l.contains(" — ") && !is_uuid(l)) {
                    return cur.map(ToString::to_string);
                }
                let target = after
                    .or(before)
                    .and_then(|v| Some((str_field(v, "entity_type")?, str_field(v, "entity_id")?)))
                    .or_else(|| {
                        self.notes
                            .get(entity_id)
                            .map(|n| (n.entity_type.as_str(), n.entity_id.as_str()))
                    });
                target
                    .and_then(|(tt, tid)| self.resolve_target_name(tt, tid))
                    .map(|name| format!("Note — {name}"))
                    .or_else(|| cur.map(ToString::to_string))
            }
            "transaction" => self
                .resolve_tx_label(entity_id)
                .or_else(|| {
                    after
                        .and_then(|a| str_field(a, "undone_tx_id"))
                        .and_then(|tid| self.resolve_tx_label(tid))
                })
                .or_else(|| cur.filter(|l| !is_uuid(l)).map(ToString::to_string))
                .or_else(|| Some("Undo transaction".to_string())),
            "place" if is_bare => self
                .places
                .get(entity_id)
                .map(|p| p.full_name.clone())
                .or_else(|| cur.map(ToString::to_string)),
            "media" if is_bare => self
                .media
                .get(entity_id)
                .map(|m| m.title.clone())
                .or_else(|| cur.map(ToString::to_string)),
            "place" | "media" => cur.map(ToString::to_string),
            _ => cur
                .filter(|l| !is_uuid(l))
                .map(ToString::to_string)
                .or_else(|| Some("Family Tree".to_string())),
        }
    }

    pub(crate) fn enrich_entry(&self, entry: &mut ChangeLogEntryDto) {
        let before = entry.before_json.as_ref();
        let after = entry.after_json.as_ref();
        entry.entity_gid =
            self.resolve_entity_gid(&entry.entity_type, &entry.entity_id, before, after);
        entry.entity_label = self.resolve_entity_label(
            &entry.entity_type,
            &entry.entity_id,
            entry.entity_label.as_deref(),
            &entry.tx_label,
            before,
            after,
        );
        entry.diffs = compute_diffs_with_index(before, after, self);
    }
}

/// Computes enriched, human-readable field diffs using full store & change-log lookup context.
pub fn compute_field_diffs_with_store(
    store: &GenealogyPrivacyStore,
    before: Option<&Value>,
    after: Option<&Value>,
) -> Vec<FieldDiffDto> {
    let index = HistoryLookupIndex::from_store(store);
    compute_diffs_with_index(before, after, &index)
}

pub(crate) fn compute_field_diffs_with_index(
    before: Option<&Value>,
    after: Option<&Value>,
    index: &HistoryLookupIndex,
) -> Vec<FieldDiffDto> {
    compute_diffs_with_index(before, after, index)
}

fn join_partners(p1: Option<&str>, p2: Option<&str>) -> Option<String> {
    match (p1, p2) {
        (Some(a), Some(b)) => Some(format!("{a} & {b}")),
        (Some(a), None) | (None, Some(a)) => Some(a.to_string()),
        (None, None) => None,
    }
}

pub(crate) fn with_gid(label: &str, gid: &str) -> String {
    let l = label.trim();
    let g = gid.trim();
    if g.is_empty() {
        return l.to_string();
    }
    if l.is_empty() || l.eq_ignore_ascii_case(g) || l.ends_with(&format!("({g})")) {
        return if l.is_empty() {
            g.to_string()
        } else {
            l.to_string()
        };
    }
    format!("{l} ({g})")
}

fn strip_trailing_gid(label: &str, gid: Option<&str>) -> String {
    let trimmed = label.trim();
    if let Some(g) = gid.map(str::trim).filter(|g| !g.is_empty()) {
        let suffix = format!(" ({g})");
        if let Some(prefix) = trimmed.strip_suffix(&suffix) {
            let clean = prefix.trim();
            if !clean.is_empty() {
                return clean.to_string();
            }
        }
        let upper_suffix = format!(" ({})", g.to_ascii_uppercase());
        if trimmed.len() > upper_suffix.len()
            && trimmed.to_ascii_uppercase().ends_with(&upper_suffix)
        {
            let idx = trimmed.len() - upper_suffix.len();
            if trimmed.is_char_boundary(idx) {
                let clean = trimmed[..idx].trim();
                if !clean.is_empty() {
                    return clean.to_string();
                }
            }
        }
    }
    trimmed.to_string()
}
