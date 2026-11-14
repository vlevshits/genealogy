#[allow(unused_imports)]
use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DuplicateEventItemDto {
    pub id: String,
    pub gid: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub individual_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_sort: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
    pub media_count: usize,
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProposedMergedEventDto {
    pub keep_event_id: String,
    pub keep_event_gid: String,
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
    pub combined_media_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DuplicateEventGroupDto {
    pub id: String,
    pub event_type: String,
    pub scope: String,
    pub person: QualityWarningPersonRefDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner: Option<QualityWarningPersonRefDto>,
    pub family_ids: Vec<String>,
    pub events: Vec<DuplicateEventItemDto>,
    pub proposed: ProposedMergedEventDto,
    pub has_conflict: bool,
    pub conflict_fields: Vec<String>,
    pub confidence: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema, IntoParams)]
pub struct DuplicateEventsFilterQuery {
    #[serde(default)]
    pub include_dismissed: Option<bool>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DuplicateEventsListResponse {
    pub items: Vec<DuplicateEventGroupDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MergeDuplicateEventsRequest {
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default, alias = "target_event_id", alias = "surviving_event_id")]
    pub keep_event_id: Option<String>,
    #[serde(
        default,
        alias = "event_ids",
        alias = "duplicate_event_ids",
        alias = "source_event_ids"
    )]
    pub merge_event_ids: Option<Vec<String>>,
    #[serde(default)]
    pub date_text: Option<String>,
    #[serde(default)]
    pub place_id: Option<String>,
    #[serde(default)]
    pub place_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MergeDuplicateEventsResponse {
    pub status: String,
    pub tx_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub surviving_event: LifeEventDto,
    pub merged_event_ids: Vec<String>,
    pub merged_family_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BatchMergeDuplicateEventsRequest {
    #[serde(default)]
    pub group_ids: Option<Vec<String>>,
    #[serde(default)]
    pub only_high_confidence: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BatchMergeDuplicateEventsResponse {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<String>,
    pub merged_groups_count: usize,
    pub removed_events_count: usize,
    pub surviving_event_ids: Vec<String>,
}

pub(crate) fn canonical_event_type(ev: &LifeEventDto) -> String {
    normalize_event_type(&ev.event_type)
}

pub(crate) fn is_union_event_type(canon: &str) -> bool {
    matches!(canon, "marriage" | "engagement" | "divorce" | "annulment")
}

pub(crate) fn is_singleton_individual_vital(canon: &str) -> bool {
    matches!(
        canon,
        "birth" | "baptism" | "death" | "burial" | "cremation"
    )
}

pub(crate) fn is_union_context(canon: &str, has_family_id: bool) -> bool {
    is_union_event_type(canon) || (!is_singleton_individual_vital(canon) && has_family_id)
}

pub(crate) fn ev_date_text(ev: &LifeEventDto) -> Option<String> {
    ev.date
        .as_ref()
        .map(|d| d.text.trim().to_string())
        .or_else(|| ev.gedcom_date.as_deref().map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
}

pub(crate) fn ev_resolved_place(
    store: &GenealogyPrivacyStore,
    ev: &LifeEventDto,
) -> (Option<String>, Option<String>) {
    let place_id = ev
        .place_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|raw| {
            store
                .resolve_place_id(raw)
                .or_else(|| Some(raw.to_string()))
        });

    let place_full_name = ev
        .place_full_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            place_id.as_deref().and_then(|pid| {
                store.places.iter().find(|p| p.id == pid).map(|p| {
                    if p.full_name.trim().is_empty() {
                        p.name.trim().to_string()
                    } else {
                        p.full_name.trim().to_string()
                    }
                })
            })
        })
        .filter(|s| !s.is_empty());

    (place_id, place_full_name)
}

pub(crate) fn date_specificity(ev: &LifeEventDto) -> u8 {
    let Some(raw) = ev_date_text(ev) else {
        return if ev.date_sort.is_some() { 1 } else { 0 };
    };
    let gd = GenDate::parse(&raw);
    match gd.r#mod {
        GenDateMod::Exact => match gd.start {
            Some(s) if s.m.is_some() && s.d.is_some() => 5,
            Some(s) if s.m.is_some() => 4,
            Some(_) => 3,
            None => 1,
        },
        GenDateMod::About
        | GenDateMod::Estimated
        | GenDateMod::Calculated
        | GenDateMod::Between
        | GenDateMod::FromTo
        | GenDateMod::Before
        | GenDateMod::After => u8::from(gd.start.is_some()) + 1,
        GenDateMod::Phrase => 1,
    }
}

pub(crate) fn dates_compatible(ev_a: &LifeEventDto, ev_b: &LifeEventDto) -> bool {
    let da = ev_date_text(ev_a);
    let db = ev_date_text(ev_b);
    if (da.is_none() && ev_a.date_sort.is_none()) || (db.is_none() && ev_b.date_sort.is_none()) {
        return true;
    }
    if let (Some(a_txt), Some(b_txt)) = (da.as_deref(), db.as_deref()) {
        if a_txt.eq_ignore_ascii_case(b_txt) {
            return true;
        }
    }
    let wa = parse_plausible_window(da.as_deref(), ev_a.date_sort);
    let wb = parse_plausible_window(db.as_deref(), ev_b.date_sort);
    match (wa, wb) {
        (Some(a), Some(b)) => a.min_sort <= b.max_sort && b.min_sort <= a.max_sort,
        _ => false,
    }
}

pub(crate) fn normalize_place_segments(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| {
            s.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

pub(crate) fn places_compatible(
    store: &GenealogyPrivacyStore,
    ev_a: &LifeEventDto,
    ev_b: &LifeEventDto,
) -> bool {
    let (pid_a, pname_a) = ev_resolved_place(store, ev_a);
    let (pid_b, pname_b) = ev_resolved_place(store, ev_b);
    if (pid_a.is_none() && pname_a.is_none()) || (pid_b.is_none() && pname_b.is_none()) {
        return true;
    }
    if let (Some(ia), Some(ib)) = (pid_a.as_deref(), pid_b.as_deref()) {
        if ia == ib {
            return true;
        }
    }
    let (Some(na), Some(nb)) = (pname_a, pname_b) else {
        return pid_a == pid_b;
    };
    let sa = normalize_place_segments(&na);
    let sb = normalize_place_segments(&nb);
    if sa.is_empty() || sb.is_empty() {
        return true;
    }
    sa == sb
        || sa.starts_with(&sb)
        || sb.starts_with(&sa)
        || (sa.len() <= sb.len() && sb.windows(sa.len()).any(|w| w == sa.as_slice()))
        || (sb.len() <= sa.len() && sa.windows(sb.len()).any(|w| w == sb.as_slice()))
}

pub(crate) fn build_media_by_event_map(
    store: &GenealogyPrivacyStore,
) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for ev in &store.events {
        let entry = map.entry(ev.id.clone()).or_default();
        for mid in &ev.media_ids {
            if !entry.contains(mid) {
                entry.push(mid.clone());
            }
        }
    }
    for media in &store.media {
        for link in &media.links {
            if link.entity_type == "event" {
                let entry = map.entry(link.entity_id.clone()).or_default();
                if !entry.contains(&media.id) {
                    entry.push(media.id.clone());
                }
            }
        }
    }
    map
}

fn family_partner_count(store: &GenealogyPrivacyStore, ev: &LifeEventDto) -> usize {
    ev.family_id
        .as_deref()
        .and_then(|fid| store.families.iter().find(|f| f.id == fid))
        .map(|fam| {
            [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
                .into_iter()
                .flatten()
                .filter(|pid| store.persons.iter().any(|p| p.id == **pid))
                .count()
        })
        .unwrap_or(0)
}

pub(crate) fn event_completeness_score(
    store: &GenealogyPrivacyStore,
    ev: &LifeEventDto,
    media_by_ev: &HashMap<String, Vec<String>>,
) -> i64 {
    let fam_bonus = match family_partner_count(store, ev) {
        2 => 2_000,
        1 => 500,
        _ => 0,
    };
    let date_score = i64::from(date_specificity(ev)) * 200;
    let (pid, pname) = ev_resolved_place(store, ev);
    let place_score = (if pid.is_some() { 300 } else { 0 })
        + pname
            .as_deref()
            .map(|s| 150 + (s.len().min(60) as i64))
            .unwrap_or(0);
    let desc_score = if ev
        .description
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty())
    {
        100
    } else {
        0
    };
    let src_score = if ev
        .source_text
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty())
    {
        100
    } else {
        0
    };
    let media_score = media_by_ev
        .get(&ev.id)
        .map(|v| (v.len() as i64) * 50)
        .unwrap_or(0);

    fam_bonus + date_score + place_score + desc_score + src_score + media_score
}

pub(crate) fn sort_events_by_completeness(
    store: &GenealogyPrivacyStore,
    events: &mut [&LifeEventDto],
    media_by_ev: &HashMap<String, Vec<String>>,
) {
    events.sort_by(|a, b| {
        let sa = event_completeness_score(store, a, media_by_ev);
        let sb = event_completeness_score(store, b, media_by_ev);
        sb.cmp(&sa)
            .then_with(|| a.gid.cmp(&b.gid))
            .then_with(|| a.id.cmp(&b.id))
    });
}

fn join_unique_non_empty<'a, I: Iterator<Item = Option<&'a str>>>(iter: I) -> Option<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut parts: Vec<String> = Vec::new();
    for item in iter.flatten() {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_lowercase()) {
            parts.push(trimmed.to_string());
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

pub(crate) fn synthesize_group_proposal(
    store: &GenealogyPrivacyStore,
    ranked: &[&LifeEventDto],
    media_by_ev: &HashMap<String, Vec<String>>,
) -> (ProposedMergedEventDto, bool, Vec<String>) {
    let keep = ranked[0];
    let canon_type = canonical_event_type(keep);
    let mut date_conflict = false;
    let mut place_conflict = false;
    for i in 0..ranked.len() {
        for j in (i + 1)..ranked.len() {
            if !dates_compatible(ranked[i], ranked[j]) {
                date_conflict = true;
            }
            if !places_compatible(store, ranked[i], ranked[j]) {
                place_conflict = true;
            }
        }
    }
    let mut conflict_fields = Vec::new();
    if date_conflict {
        conflict_fields.push("date".to_string());
    }
    if place_conflict {
        conflict_fields.push("place".to_string());
    }

    let best_date = ranked
        .iter()
        .filter_map(|ev| {
            let txt = ev_date_text(ev)?;
            Some((date_specificity(ev), txt.len(), txt))
        })
        .max_by_key(|(spec, len, _)| (*spec, *len))
        .map(|(_, _, txt)| txt);

    let best_place = ranked
        .iter()
        .enumerate()
        .filter_map(|(idx, ev)| {
            let (pid, pname) = ev_resolved_place(store, ev);
            if pid.is_none() && pname.is_none() {
                return None;
            }
            let segs = pname
                .as_deref()
                .map(|s| normalize_place_segments(s).len())
                .unwrap_or(0);
            let len = pname.as_deref().map(str::len).unwrap_or(0);
            Some((
                (pid.is_some() as u8, segs, len, std::cmp::Reverse(idx)),
                pid,
                pname,
            ))
        })
        .max_by_key(|(score, _, _)| *score);

    let (place_id, place_full_name) = match best_place {
        Some((_, pid, pname)) => (pid, pname),
        None => (None, None),
    };

    let mut combined_media: HashSet<&str> = HashSet::new();
    for ev in ranked {
        if let Some(mids) = media_by_ev.get(&ev.id) {
            for m in mids {
                combined_media.insert(m.as_str());
            }
        }
    }

    let proposed = ProposedMergedEventDto {
        keep_event_id: keep.id.clone(),
        keep_event_gid: keep.gid.clone(),
        event_type: canon_type,
        date_text: best_date,
        place_id,
        place_full_name,
        description: join_unique_non_empty(ranked.iter().map(|ev| ev.description.as_deref())),
        source_text: join_unique_non_empty(ranked.iter().map(|ev| ev.source_text.as_deref())),
        combined_media_count: combined_media.len(),
    };
    (proposed, !conflict_fields.is_empty(), conflict_fields)
}

pub(crate) fn is_event_group_dismissed(store: &GenealogyPrivacyStore, group_id: &str) -> bool {
    let wid = format!("{RULE_DUPLICATE_EVENT}:{group_id}");
    is_warning_dismissed(
        &store.dismissed_warnings,
        &wid,
        RULE_DUPLICATE_EVENT,
        group_id,
        None,
    ) || store
        .dismissed_warnings
        .iter()
        .any(|d| d.entity_id == group_id)
}
