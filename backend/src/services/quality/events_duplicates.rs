#[allow(unused_imports)]
use super::*;
use std::collections::{BTreeMap, HashMap};

pub const RULE_DUPLICATE_EVENT: &str = "DUPLICATE_EVENT";

pub use super::events_duplicates_batch::{
    batch_merge_duplicate_events_in_store, execute_mcp_list_duplicate_events,
    execute_mcp_merge_duplicate_events,
};
pub use super::events_duplicates_merge::merge_duplicate_events_in_store;
pub use super::events_duplicates_types::*;

fn cluster_compatible_events<'a>(
    store: &GenealogyPrivacyStore,
    events: &[&'a LifeEventDto],
    media_by_ev: &HashMap<String, Vec<String>>,
) -> Vec<Vec<&'a LifeEventDto>> {
    let mut ordered: Vec<&'a LifeEventDto> = events.to_vec();
    sort_events_by_completeness(store, &mut ordered, media_by_ev);

    let mut clusters: Vec<Vec<&'a LifeEventDto>> = Vec::new();
    for ev in ordered {
        let target_idx = clusters.iter().position(|cluster| {
            cluster
                .iter()
                .all(|m| dates_compatible(m, ev) && places_compatible(store, m, ev))
        });
        if let Some(idx) = target_idx {
            clusters[idx].push(ev);
        } else {
            clusters.push(vec![ev]);
        }
    }
    clusters
}

#[allow(clippy::too_many_arguments)]
fn build_group_dto(
    store: &GenealogyPrivacyStore,
    group_id: String,
    canon_type: String,
    scope: &str,
    person: QualityWarningPersonRefDto,
    partner: Option<QualityWarningPersonRefDto>,
    mut cluster: Vec<&LifeEventDto>,
    media_by_ev: &HashMap<String, Vec<String>>,
) -> DuplicateEventGroupDto {
    sort_events_by_completeness(store, &mut cluster, media_by_ev);
    let (proposed, has_conflict, conflict_fields) =
        synthesize_group_proposal(store, &cluster, media_by_ev);

    let mut family_ids: Vec<String> = Vec::new();
    for ev in &cluster {
        if let Some(fid) = ev.family_id.as_deref().filter(|s| !s.is_empty()) {
            if !family_ids.iter().any(|existing| existing == fid) {
                family_ids.push(fid.to_string());
            }
        }
    }

    let items: Vec<DuplicateEventItemDto> = cluster
        .iter()
        .map(|ev| {
            let (pid, pname) = ev_resolved_place(store, ev);
            DuplicateEventItemDto {
                id: ev.id.clone(),
                gid: ev.gid.clone(),
                event_type: canonical_event_type(ev),
                custom_type: ev.custom_type.clone(),
                individual_id: ev.individual_id.clone(),
                family_id: ev.family_id.clone(),
                date_text: ev_date_text(ev),
                date_sort: ev.date_sort,
                place_id: pid,
                place_full_name: pname,
                description: ev.description.clone(),
                source_text: ev.source_text.clone(),
                media_count: media_by_ev.get(&ev.id).map(Vec::len).unwrap_or(0),
                is_private: ev.is_private,
            }
        })
        .collect();

    let confidence = if has_conflict { "medium" } else { "high" }.to_string();
    let label = if let Some(ref p2) = partner {
        format!("{} & {}", person.display_name, p2.display_name)
    } else {
        person.display_name.clone()
    };
    let conflict_note = if has_conflict {
        format!(" ({} conflict)", conflict_fields.join("/"))
    } else {
        String::new()
    };
    let reason = format!(
        "{} duplicate {} events for {}{}",
        items.len(),
        canon_type,
        label,
        conflict_note
    );

    DuplicateEventGroupDto {
        id: group_id,
        event_type: canon_type,
        scope: scope.to_string(),
        person,
        partner,
        family_ids,
        events: items,
        proposed,
        has_conflict,
        conflict_fields,
        confidence,
        reason,
    }
}

/// Computes duplicate life event groups across individuals and couple/family unions.
pub fn compute_duplicate_event_groups(
    store: &GenealogyPrivacyStore,
    include_dismissed: bool,
) -> Vec<DuplicateEventGroupDto> {
    let person_map: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();
    let family_map: HashMap<&str, &FamilyDto> =
        store.families.iter().map(|f| (f.id.as_str(), f)).collect();
    let media_by_ev = build_media_by_event_map(store);

    let mut ind_vitals: BTreeMap<(String, String), Vec<&LifeEventDto>> = BTreeMap::new();
    let mut ind_repeatables: BTreeMap<(String, String), BTreeMap<String, Vec<&LifeEventDto>>> =
        BTreeMap::new();
    let mut couple_buckets: BTreeMap<(String, String, String), Vec<&LifeEventDto>> =
        BTreeMap::new();
    let mut single_unions: Vec<(String, String, &LifeEventDto)> = Vec::new();

    for ev in &store.events {
        let canon = canonical_event_type(ev);
        if is_union_context(&canon, ev.family_id.is_some()) {
            if let Some(fam) = ev.family_id.as_deref().and_then(|fid| family_map.get(fid)) {
                let mut partners: Vec<String> =
                    [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
                        .into_iter()
                        .flatten()
                        .filter(|pid| person_map.contains_key(pid.as_str()))
                        .cloned()
                        .collect();
                partners.sort();
                partners.dedup();
                if partners.len() == 2 {
                    couple_buckets
                        .entry((partners[0].clone(), partners[1].clone(), canon))
                        .or_default()
                        .push(ev);
                    continue;
                } else if partners.len() == 1 {
                    single_unions.push((partners[0].clone(), canon, ev));
                    continue;
                }
            }
            if let Some(pid) = ev.individual_id.as_deref() {
                if person_map.contains_key(pid) {
                    single_unions.push((pid.to_string(), canon, ev));
                }
            }
            continue;
        }

        let Some(pid) = ev.individual_id.as_deref() else {
            continue;
        };
        if !person_map.contains_key(pid) {
            continue;
        }
        if is_singleton_individual_vital(&canon) {
            ind_vitals
                .entry((pid.to_string(), canon))
                .or_default()
                .push(ev);
        } else {
            let custom_key = ev
                .custom_type
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .to_lowercase();
            ind_repeatables
                .entry((pid.to_string(), canon))
                .or_default()
                .entry(custom_key)
                .or_default()
                .push(ev);
        }
    }

    let base_couples = couple_buckets.clone();
    let mut remaining_singles: BTreeMap<(String, String), Vec<&LifeEventDto>> = BTreeMap::new();
    for (p1, canon, ev) in single_unions {
        let matched_key = base_couples
            .iter()
            .find(|((a, b, t), existing)| {
                t == &canon
                    && (a == &p1 || b == &p1)
                    && existing
                        .iter()
                        .any(|c| dates_compatible(c, ev) && places_compatible(store, c, ev))
            })
            .map(|(k, _)| k.clone());

        if let Some(key) = matched_key {
            couple_buckets.entry(key).or_default().push(ev);
        } else {
            remaining_singles.entry((p1, canon)).or_default().push(ev);
        }
    }

    let mut groups: Vec<DuplicateEventGroupDto> = Vec::new();

    for ((pid, canon), bucket) in ind_vitals {
        if bucket.len() >= 2 {
            let group_id = format!("dup_event:ind:{pid}:{canon}");
            if include_dismissed || !is_event_group_dismissed(store, &group_id) {
                let person = to_warning_person_ref(person_map[pid.as_str()]);
                groups.push(build_group_dto(
                    store,
                    group_id,
                    canon,
                    "individual",
                    person,
                    None,
                    bucket,
                    &media_by_ev,
                ));
            }
        }
    }

    for ((pid, canon), by_custom) in ind_repeatables {
        let mut cluster_idx = 0usize;
        for (_, bucket) in by_custom {
            for cluster in cluster_compatible_events(store, &bucket, &media_by_ev) {
                if cluster.len() < 2 {
                    continue;
                }
                let group_id = format!("dup_event:ind:{pid}:{canon}:{cluster_idx}");
                cluster_idx += 1;
                if include_dismissed || !is_event_group_dismissed(store, &group_id) {
                    let person = to_warning_person_ref(person_map[pid.as_str()]);
                    groups.push(build_group_dto(
                        store,
                        group_id,
                        canon.clone(),
                        "individual",
                        person,
                        None,
                        cluster,
                        &media_by_ev,
                    ));
                }
            }
        }
    }

    for ((p1, p2, canon), bucket) in couple_buckets {
        let is_union = is_union_event_type(&canon);
        let clusters = if is_union {
            vec![bucket]
        } else {
            cluster_compatible_events(store, &bucket, &media_by_ev)
        };
        let mut cluster_idx = 0usize;
        for cluster in clusters {
            if cluster.len() < 2 {
                continue;
            }
            let group_id = if is_union {
                format!("dup_event:fam:{p1}:{p2}:{canon}")
            } else {
                let id = format!("dup_event:fam:{p1}:{p2}:{canon}:{cluster_idx}");
                cluster_idx += 1;
                id
            };
            if include_dismissed || !is_event_group_dismissed(store, &group_id) {
                groups.push(build_group_dto(
                    store,
                    group_id,
                    canon.clone(),
                    "family",
                    to_warning_person_ref(person_map[p1.as_str()]),
                    Some(to_warning_person_ref(person_map[p2.as_str()])),
                    cluster,
                    &media_by_ev,
                ));
            }
        }
    }

    for ((p1, canon), bucket) in remaining_singles {
        let is_union = is_union_event_type(&canon);
        let clusters = if is_union {
            vec![bucket]
        } else {
            cluster_compatible_events(store, &bucket, &media_by_ev)
        };
        let mut cluster_idx = 0usize;
        for cluster in clusters {
            if cluster.len() < 2 {
                continue;
            }
            let group_id = if is_union {
                format!("dup_event:fam_single:{p1}:{canon}")
            } else {
                let id = format!("dup_event:fam_single:{p1}:{canon}:{cluster_idx}");
                cluster_idx += 1;
                id
            };
            if include_dismissed || !is_event_group_dismissed(store, &group_id) {
                groups.push(build_group_dto(
                    store,
                    group_id,
                    canon.clone(),
                    "family",
                    to_warning_person_ref(person_map[p1.as_str()]),
                    None,
                    cluster,
                    &media_by_ev,
                ));
            }
        }
    }

    groups.sort_by(|a, b| {
        let conf_a = i32::from(a.confidence != "high");
        let conf_b = i32::from(b.confidence != "high");
        conf_a.cmp(&conf_b).then_with(|| a.id.cmp(&b.id))
    });
    groups
}

pub fn filter_duplicate_event_groups(
    groups: Vec<DuplicateEventGroupDto>,
    query: &DuplicateEventsFilterQuery,
) -> DuplicateEventsListResponse {
    let type_filter = query
        .event_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_event_type);
    let scope_filter = query
        .scope
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase);
    let conf_filter = query
        .confidence
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase);

    let items: Vec<DuplicateEventGroupDto> = groups
        .into_iter()
        .filter(|g| {
            type_filter.as_ref().is_none_or(|tf| &g.event_type == tf)
                && scope_filter.as_ref().is_none_or(|sf| &g.scope == sf)
                && conf_filter.as_ref().is_none_or(|cf| &g.confidence == cf)
        })
        .collect();
    let total = items.len();
    DuplicateEventsListResponse { items, total }
}
