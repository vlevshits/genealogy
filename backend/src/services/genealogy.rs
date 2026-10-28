use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::AppError,
    services::privacy::{
        evaluate_is_living, FamilyDto, GenealogyPrivacyStore, PersonLivingEvaluationInput,
        PlaceAltNameDto, PlaceBreadcrumbDto, PlaceSummaryDto, TreePersonNodeDto,
        VitalEventSummaryDto,
    },
};

/// Allocates a stable, monotonically increasing GID (`I0100`, `F0100`, `E0100`, `P0100`, `M0100`).
/// Preserves an explicit GEDCOM xref/GID if supplied and unused, advancing the sequence past it.
/// Never reuses a GID after deletion (`10-core-data-model.md §3.1, §8 DoD #3`).
pub fn allocate_gid(
    prefix: char,
    seq: &mut u32,
    requested_gid: Option<&str>,
    existing_gids: &HashSet<String>,
) -> String {
    if let Some(raw) = requested_gid {
        let trimmed = raw.trim().trim_matches('@').to_uppercase();
        let actual_prefix = trimmed.chars().next();
        let prefix_matches =
            actual_prefix == Some(prefix) || (prefix == 'M' && actual_prefix == Some('O'));
        if prefix_matches && trimmed.len() >= 2 {
            let eff_prefix = actual_prefix.unwrap_or(prefix);
            let digits_part = &trimmed[1..];
            if let Ok(num) = digits_part.parse::<u32>() {
                let canonical = format!("{eff_prefix}{num:04}");
                if !existing_gids.contains(&canonical) && !existing_gids.contains(&trimmed) {
                    if num >= *seq {
                        *seq = num + 1;
                    }
                    return canonical;
                }
            } else if !existing_gids.contains(&trimmed) {
                return trimmed;
            }
        }
    }

    loop {
        let candidate = format!("{prefix}{:04}", *seq);
        *seq += 1;
        if !existing_gids.contains(&candidate) {
            return candidate;
        }
    }
}

/// Verifies that linking `proposed_parent_id` as a parent of `proposed_child_id` does not
/// create a self-loop or ancestor-descendant cycle (`10-core-data-model.md §6.1, §8 DoD #4`).
pub fn ensure_no_ancestor_cycle(
    families: &[FamilyDto],
    proposed_parent_id: &str,
    proposed_child_id: &str,
) -> Result<(), AppError> {
    let parent = proposed_parent_id.trim();
    let child = proposed_child_id.trim();

    if parent.is_empty() || child.is_empty() {
        return Ok(());
    }

    if parent == child {
        return Err(AppError::CycleDetected(
            "A person cannot be their own parent or ancestor".to_string(),
        ));
    }

    // Walk upward through all ancestors of `proposed_parent_id`.
    // If `proposed_child_id` is found among `proposed_parent_id`'s ancestors,
    // then `proposed_child_id` is already an ancestor of `proposed_parent_id`,
    // so making `proposed_parent_id` a parent of `proposed_child_id` would form a cycle.
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(parent.to_string());
    visited.insert(parent.to_string());

    while let Some(curr_person_id) = queue.pop_front() {
        for fam in families {
            let is_child_in_fam = fam.children.iter().any(|c| c.child_id == curr_person_id);
            if is_child_in_fam {
                for ancestor_opt in [&fam.partner1_id, &fam.partner2_id] {
                    if let Some(anc_id) = ancestor_opt.as_deref() {
                        if anc_id == child {
                            return Err(AppError::CycleDetected(
                                "Cannot link a descendant as a parent/ancestor (cycle detected)"
                                    .to_string(),
                            ));
                        }
                        if visited.insert(anc_id.to_string()) {
                            queue.push_back(anc_id.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Verifies that setting `proposed_parent_id` on `place_id` does not create a cycle in `places`.
pub fn ensure_no_place_cycle(
    places: &[PlaceSummaryDto],
    place_id: &str,
    proposed_parent_id: Option<&str>,
) -> Result<(), AppError> {
    let Some(parent_id) = proposed_parent_id.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(());
    };

    if place_id.trim() == parent_id {
        return Err(AppError::CycleDetected(
            "A place cannot be its own parent".to_string(),
        ));
    }

    let mut curr = Some(parent_id.to_string());
    let mut hops = 0usize;
    while let Some(cid) = curr {
        if cid == place_id.trim() {
            return Err(AppError::CycleDetected(
                "Cannot move a place under one of its own descendants".to_string(),
            ));
        }
        hops += 1;
        if hops > places.len() + 2 {
            break;
        }
        curr = places
            .iter()
            .find(|p| p.id == cid)
            .and_then(|p| p.parent_id.clone());
    }

    Ok(())
}

/// Recomputes `full_name`, `country`, `children_count`, and `events_count` for all places (`12-places.md §2`).
pub fn recompute_all_places_metadata(store: &mut GenealogyPrivacyStore) {
    let parent_map: HashMap<String, (String, Option<String>)> = store
        .places
        .iter()
        .map(|p| (p.id.clone(), (p.name.clone(), p.parent_id.clone())))
        .collect();

    let mut children_counts: HashMap<String, u32> = HashMap::new();
    for p in &store.places {
        if let Some(ref pid) = p.parent_id {
            *children_counts.entry(pid.clone()).or_insert(0) += 1;
        }
    }

    let mut event_counts: HashMap<String, u32> = HashMap::new();
    for ev in &store.events {
        if let Some(ref pid) = ev.place_id {
            *event_counts.entry(pid.clone()).or_insert(0) += 1;
        }
    }

    for place in &mut store.places {
        let mut chain_names = vec![place.name.clone()];
        let mut curr_parent = place.parent_id.clone();
        let mut guard = 0usize;
        while let Some(pid) = curr_parent {
            guard += 1;
            if guard > 32 {
                break;
            }
            if let Some((pname, next_parent)) = parent_map.get(&pid) {
                chain_names.push(pname.clone());
                curr_parent = next_parent.clone();
            } else {
                break;
            }
        }

        if chain_names.len() > 1 {
            place.full_name = chain_names.join(", ");
            place.country = chain_names.last().cloned();
        } else if place.full_name.trim().is_empty() || place.parent_id.is_some() {
            place.full_name = place.name.clone();
        }

        place.children_count = children_counts.get(&place.id).copied().unwrap_or(0);
        if let Some(ev_cnt) = event_counts.get(&place.id) {
            place.events_count = *ev_cnt;
        }
    }
}

/// Builds root-to-leaf breadcrumbs for a place (`12-places.md §3.3`).
pub fn build_place_breadcrumbs(
    places: &[PlaceSummaryDto],
    place_id: &str,
) -> Vec<PlaceBreadcrumbDto> {
    let mut crumbs = Vec::new();
    let mut curr = places.iter().find(|p| p.id == place_id);
    let mut guard = 0usize;
    while let Some(p) = curr {
        crumbs.push(PlaceBreadcrumbDto {
            id: p.id.clone(),
            gid: p.gid.clone(),
            name: p.name.clone(),
            place_type: p.place_type.clone(),
        });
        guard += 1;
        if guard > 32 {
            break;
        }
        curr = p
            .parent_id
            .as_deref()
            .and_then(|pid| places.iter().find(|cand| cand.id == pid));
    }
    crumbs.reverse();
    crumbs
}

/// Resolves or creates a comma-separated place hierarchy right-to-left (top-down),
/// deduplicating existing nodes at each level (`12-places.md §3.1, §5, §7 DoD #1 & #5`).
pub fn resolve_or_create_place_chain(
    store: &mut GenealogyPrivacyStore,
    raw_path: &str,
    leaf_place_type: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    alt_names: Option<Vec<PlaceAltNameDto>>,
) -> Result<PlaceSummaryDto, AppError> {
    let segments: Vec<&str> = raw_path
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "Place name or path must not be empty".to_string(),
            Some(fields),
        ));
    }

    let total_levels = segments.len();
    let mut current_parent_id: Option<String> = None;
    let mut leaf_id = String::new();

    // Traverse from right (top-most jurisdiction, e.g. Country) to left (leaf locality)
    for (depth_from_top, seg_name) in segments.iter().rev().enumerate() {
        let is_leaf = depth_from_top + 1 == total_levels;

        // Check if a place with this name and parent already exists (or when single segment, matches name/full_name)
        let existing_idx = store.places.iter().position(|p| {
            let name_matches = p.name.eq_ignore_ascii_case(seg_name)
                || p.alt_names
                    .iter()
                    .any(|a| a.name.eq_ignore_ascii_case(seg_name));
            if total_levels == 1 {
                name_matches
            } else {
                name_matches && p.parent_id == current_parent_id
            }
        });

        if let Some(idx) = existing_idx {
            let found_id = store.places[idx].id.clone();
            if is_leaf {
                if let Some(lat) = latitude {
                    store.places[idx].latitude = Some(lat);
                }
                if let Some(lon) = longitude {
                    store.places[idx].longitude = Some(lon);
                }
                if let Some(ref extra_alts) = alt_names {
                    for alt in extra_alts {
                        if !store.places[idx]
                            .alt_names
                            .iter()
                            .any(|a| a.name.eq_ignore_ascii_case(&alt.name))
                        {
                            store.places[idx].alt_names.push(PlaceAltNameDto {
                                id: if alt.id.is_empty() {
                                    Uuid::new_v4().to_string()
                                } else {
                                    alt.id.clone()
                                },
                                name: alt.name.trim().to_string(),
                                lang: alt.lang.clone(),
                                note: alt.note.clone(),
                            });
                        }
                    }
                }
            }
            current_parent_id = Some(found_id.clone());
            leaf_id = found_id;
        } else {
            let existing_gids: HashSet<String> =
                store.places.iter().map(|p| p.gid.clone()).collect();
            let gid = allocate_gid('P', &mut store.next_place_gid_seq, None, &existing_gids);
            let new_id = Uuid::new_v4().to_string();

            let inferred_type = if is_leaf {
                leaf_place_type.unwrap_or(if total_levels >= 3 { "city" } else { "other" })
            } else if depth_from_top == 0 && total_levels >= 2 {
                "country"
            } else if depth_from_top == 1 {
                "region"
            } else {
                "district"
            };

            let new_place = PlaceSummaryDto {
                id: new_id.clone(),
                gid,
                name: (*seg_name).to_string(),
                place_type: inferred_type.to_string(),
                parent_id: current_parent_id.clone(),
                full_name: (*seg_name).to_string(),
                country: None,
                latitude: if is_leaf { latitude } else { None },
                longitude: if is_leaf { longitude } else { None },
                events_count: 0,
                children_count: 0,
                alt_names: if is_leaf {
                    alt_names
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .map(|mut a| {
                            if a.id.is_empty() {
                                a.id = Uuid::new_v4().to_string();
                            }
                            a
                        })
                        .collect()
                } else {
                    vec![]
                },
                version: 1,
            };

            store.places.push(new_place);
            current_parent_id = Some(new_id.clone());
            leaf_id = new_id;
        }
    }

    recompute_all_places_metadata(store);

    store
        .places
        .iter()
        .find(|p| p.id == leaf_id)
        .cloned()
        .ok_or(AppError::NotFound)
}

/// Recomputes `birth_sort`, `death_sort`, `has_death_event`, `birth`, `death`, and `is_living`
/// for all individuals in the store (`10-core-data-model.md §4.1, §4.3, §5`).
pub fn recompute_all_individuals_vitals(
    store: &mut GenealogyPrivacyStore,
    current_year: i32,
    age_threshold: i32,
) {
    let place_names: HashMap<String, String> = store
        .places
        .iter()
        .map(|p| (p.id.clone(), p.full_name.clone()))
        .collect();

    // First pass: update each person's birth/death summary & sort keys from their life_events
    for person in &mut store.persons {
        let mut person_events: Vec<_> = store
            .events
            .iter()
            .filter(|e| e.individual_id.as_deref() == Some(person.id.as_str()))
            .collect();

        if !person_events.is_empty() {
            person_events.sort_by_key(|e| e.date_sort.unwrap_or(i32::MAX));

            let primary_birth = person_events
                .iter()
                .find(|e| e.event_type == "birth")
                .or_else(|| person_events.iter().find(|e| e.event_type == "baptism"));

            if let Some(b_ev) = primary_birth {
                person.birth_sort = b_ev.date_sort;
                let date_str = b_ev
                    .gedcom_date
                    .clone()
                    .or_else(|| b_ev.date.as_ref().map(|d| d.text.clone()));
                let place_str = b_ev.place_full_name.clone().or_else(|| {
                    b_ev.place_id
                        .as_ref()
                        .and_then(|pid| place_names.get(pid).cloned())
                });
                person.birth = Some(VitalEventSummaryDto {
                    date: date_str,
                    place: place_str,
                });
            }

            let primary_death = person_events
                .iter()
                .find(|e| matches!(e.event_type.as_str(), "death" | "burial" | "cremation"));

            if let Some(d_ev) = primary_death {
                person.has_death_event = true;
                person.death_sort = d_ev.date_sort;
                let date_str = d_ev
                    .gedcom_date
                    .clone()
                    .or_else(|| d_ev.date.as_ref().map(|d| d.text.clone()));
                let place_str = d_ev.place_full_name.clone().or_else(|| {
                    d_ev.place_id
                        .as_ref()
                        .and_then(|pid| place_names.get(pid).cloned())
                });
                person.death = Some(VitalEventSummaryDto {
                    date: date_str,
                    place: place_str,
                });
            } else {
                person.has_death_event = false;
                person.death_sort = None;
                person.death = None;
            }
        }
    }

    let birth_years: HashMap<String, i32> = store
        .persons
        .iter()
        .filter_map(|p| p.birth_sort.map(|bs| (p.id.clone(), bs / 10_000)))
        .collect();

    let family_marriage_years: HashMap<String, i32> = store
        .events
        .iter()
        .filter(|e| e.event_type == "marriage" && e.family_id.is_some() && e.date_sort.is_some())
        .map(|e| (e.family_id.clone().unwrap(), e.date_sort.unwrap() / 10_000))
        .collect();

    let families_snapshot = store.families.clone();

    for person in &mut store.persons {
        let birth_year = person.birth_sort.map(|bs| bs / 10_000);

        let mut earliest_child_by: Option<i32> = None;
        let mut partner_by: Option<i32> = None;
        let mut parent_by: Option<i32> = None;
        let mut marriage_yr: Option<i32> = None;

        for fam in &families_snapshot {
            let is_partner1 = fam.partner1_id.as_deref() == Some(person.id.as_str());
            let is_partner2 = fam.partner2_id.as_deref() == Some(person.id.as_str());
            if is_partner1 || is_partner2 {
                let other_partner = if is_partner1 {
                    fam.partner2_id.as_deref()
                } else {
                    fam.partner1_id.as_deref()
                };
                if let Some(op_id) = other_partner {
                    if let Some(&opy) = birth_years.get(op_id) {
                        partner_by = Some(partner_by.map_or(opy, |cur: i32| cur.min(opy)));
                    }
                }
                for ch in &fam.children {
                    if let Some(&cy) = birth_years.get(&ch.child_id) {
                        earliest_child_by =
                            Some(earliest_child_by.map_or(cy, |cur: i32| cur.min(cy)));
                    }
                }
                if let Some(&my) = family_marriage_years.get(&fam.id) {
                    marriage_yr = Some(marriage_yr.map_or(my, |cur: i32| cur.min(my)));
                }
            }

            if fam.children.iter().any(|c| c.child_id == person.id) {
                for parent_opt in [&fam.partner1_id, &fam.partner2_id] {
                    if let Some(pid) = parent_opt.as_deref() {
                        if let Some(&py) = birth_years.get(pid) {
                            parent_by = Some(parent_by.map_or(py, |cur: i32| cur.min(py)));
                        }
                    }
                }
            }
        }

        let eval = PersonLivingEvaluationInput {
            living_override: person.living_override,
            has_death_or_burial_event: person.has_death_event || person.death.is_some(),
            birth_year,
            earliest_child_birth_year: earliest_child_by,
            partner_birth_year: partner_by,
            parent_birth_year: parent_by,
            marriage_year: marriage_yr,
        };

        person.is_living = evaluate_is_living(&eval, current_year, age_threshold);
    }

    let person_living_map: HashMap<String, bool> = store
        .persons
        .iter()
        .map(|p| (p.id.clone(), p.is_living))
        .collect();

    for media in &mut store.media {
        for tag in &mut media.face_tags {
            if let Some(&living) = person_living_map.get(&tag.person_id) {
                tag.is_living = living;
            }
        }
        let mut assoc_person_ids: Vec<&str> =
            media.linked_person_ids.iter().map(|s| s.as_str()).collect();
        for tag in &media.face_tags {
            if !assoc_person_ids.contains(&tag.person_id.as_str()) {
                assoc_person_ids.push(tag.person_id.as_str());
            }
        }
        for link in &media.links {
            if link.entity_type == "individual"
                && !assoc_person_ids.contains(&link.entity_id.as_str())
            {
                assoc_person_ids.push(link.entity_id.as_str());
            }
        }
        if !assoc_person_ids.is_empty() {
            media.only_living_persons = assoc_person_ids
                .iter()
                .all(|pid| person_living_map.get(*pid).copied().unwrap_or(false));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileKinshipDto {
    pub home_person_id: String,
    pub target_person_id: String,
    pub relation_key: String,
    pub label_en: String,
    pub label_ru: String,
    pub steps: usize,
    pub path_person_ids: Vec<String>,
}

/// Computes the shortest kinship relation between `home_id` and `target_id` across visible
/// individuals and families (`03-person-profile-page.md §2.1`, `16-dashboard-home.md §2`).
/// Uses cycle-safe BFS capped at 6 generations.
pub fn calculate_kinship_relation(
    home_id: &str,
    target_id: &str,
    persons: &[TreePersonNodeDto],
    families: &[FamilyDto],
) -> Option<ProfileKinshipDto> {
    let home_id = home_id.trim();
    let target_id = target_id.trim();
    if home_id.is_empty() || target_id.is_empty() {
        return None;
    }

    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        persons.iter().map(|p| (p.id.as_str(), p)).collect();
    let _home_person = *person_by_id.get(home_id)?;
    let target_person = *person_by_id.get(target_id)?;

    if home_id == target_id {
        return Some(ProfileKinshipDto {
            home_person_id: home_id.to_string(),
            target_person_id: target_id.to_string(),
            relation_key: "self".to_string(),
            label_en: "You (Home Person)".to_string(),
            label_ru: "Вы (исходная персона)".to_string(),
            steps: 0,
            path_person_ids: vec![home_id.to_string()],
        });
    }

    let target_sex = target_person.sex.trim().to_ascii_uppercase();
    let is_male = target_sex == "M";
    let is_female = target_sex == "F";

    // Build parent and spouse adjacency maps:
    // parents_of[person] -> Vec<parent_id>
    // spouses_of[person] -> Vec<spouse_id>
    let mut parents_of: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut spouses_of: HashMap<&str, Vec<&str>> = HashMap::new();

    for fam in families {
        let p1 = fam
            .partner1_id
            .as_deref()
            .filter(|id| person_by_id.contains_key(*id));
        let p2 = fam
            .partner2_id
            .as_deref()
            .filter(|id| person_by_id.contains_key(*id));

        if let (Some(a), Some(b)) = (p1, p2) {
            spouses_of.entry(a).or_default().push(b);
            spouses_of.entry(b).or_default().push(a);
        }

        for ch in &fam.children {
            let cid = ch.child_id.as_str();
            if !person_by_id.contains_key(cid) {
                continue;
            }
            for parent in [p1, p2].into_iter().flatten() {
                parents_of.entry(cid).or_default().push(parent);
            }
        }
    }

    // Check direct spouse (steps = 1)
    if let Some(spouses) = spouses_of.get(home_id) {
        if spouses.contains(&target_id) {
            let (label_en, label_ru) = if is_male {
                ("Husband", "Муж")
            } else if is_female {
                ("Wife", "Жена")
            } else {
                ("Spouse", "Супруг(а)")
            };
            return Some(ProfileKinshipDto {
                home_person_id: home_id.to_string(),
                target_person_id: target_id.to_string(),
                relation_key: "spouse".to_string(),
                label_en: label_en.to_string(),
                label_ru: label_ru.to_string(),
                steps: 1,
                path_person_ids: vec![home_id.to_string(), target_id.to_string()],
            });
        }
    }

    // Compute upward ancestor paths from any person up to 6 generations:
    // returns HashMap<ancestor_id, (depth, path_from_start_to_ancestor)>
    let build_ancestors = |start_id: &str| -> HashMap<String, (usize, Vec<String>)> {
        let mut result: HashMap<String, (usize, Vec<String>)> = HashMap::new();
        let mut queue: VecDeque<(String, usize, Vec<String>)> = VecDeque::new();
        result.insert(start_id.to_string(), (0, vec![start_id.to_string()]));
        queue.push_back((start_id.to_string(), 0, vec![start_id.to_string()]));

        while let Some((curr, depth, path)) = queue.pop_front() {
            if depth >= 6 {
                continue;
            }
            if let Some(parents) = parents_of.get(curr.as_str()) {
                for &p_id in parents {
                    if !result.contains_key(p_id) {
                        let mut next_path = path.clone();
                        next_path.push(p_id.to_string());
                        result.insert(p_id.to_string(), (depth + 1, next_path.clone()));
                        queue.push_back((p_id.to_string(), depth + 1, next_path));
                    }
                }
            }
        }
        result
    };

    let home_ancestors = build_ancestors(home_id);
    let target_ancestors = build_ancestors(target_id);

    // Find lowest common ancestor (LCA) minimizing (up + down)
    let mut best_blood: Option<(usize, usize, Vec<String>)> = None;
    for (anc_id, (up_steps, path_home_to_anc)) in &home_ancestors {
        if let Some((down_steps, path_target_to_anc)) = target_ancestors.get(anc_id) {
            let total_steps = *up_steps + *down_steps;
            let is_better = match &best_blood {
                None => true,
                Some((best_u, best_d, _)) => total_steps < (*best_u + *best_d),
            };
            if is_better {
                let mut combined_path = path_home_to_anc.clone();
                // Append reversed path from ancestor down to target (excluding ancestor itself)
                for node in path_target_to_anc.iter().rev().skip(1) {
                    combined_path.push(node.clone());
                }
                best_blood = Some((*up_steps, *down_steps, combined_path));
            }
        }
    }

    if let Some((up, down, path)) = best_blood {
        let steps = up + down;
        let (relation_key, label_en, label_ru) = match (up, down) {
            (1, 0) => (
                "parent",
                if is_male {
                    "Father"
                } else if is_female {
                    "Mother"
                } else {
                    "Parent"
                },
                if is_male {
                    "Отец"
                } else if is_female {
                    "Мать"
                } else {
                    "Родитель"
                },
            ),
            (0, 1) => (
                "child",
                if is_male {
                    "Son"
                } else if is_female {
                    "Daughter"
                } else {
                    "Child"
                },
                if is_male {
                    "Сын"
                } else if is_female {
                    "Дочь"
                } else {
                    "Ребёнок"
                },
            ),
            (2, 0) => (
                "grandparent",
                if is_male {
                    "Grandfather"
                } else if is_female {
                    "Grandmother"
                } else {
                    "Grandparent"
                },
                if is_male {
                    "Дедушка"
                } else if is_female {
                    "Бабушка"
                } else {
                    "Дедушка / Бабушка"
                },
            ),
            (0, 2) => (
                "grandchild",
                if is_male {
                    "Grandson"
                } else if is_female {
                    "Granddaughter"
                } else {
                    "Grandchild"
                },
                if is_male {
                    "Внук"
                } else if is_female {
                    "Внучка"
                } else {
                    "Внук / Внучка"
                },
            ),
            (u, 0) if u >= 3 => (
                "great_grandparent",
                if is_male {
                    "Great-grandfather"
                } else if is_female {
                    "Great-grandmother"
                } else {
                    "Great-grandparent"
                },
                if is_male {
                    "Прадедушка"
                } else if is_female {
                    "Прабабушка"
                } else {
                    "Прародитель"
                },
            ),
            (0, d) if d >= 3 => (
                "great_grandchild",
                if is_male {
                    "Great-grandson"
                } else if is_female {
                    "Great-granddaughter"
                } else {
                    "Great-grandchild"
                },
                if is_male {
                    "Правнук"
                } else if is_female {
                    "Правнучка"
                } else {
                    "Правнук / Правнучка"
                },
            ),
            (1, 1) => (
                "sibling",
                if is_male {
                    "Brother"
                } else if is_female {
                    "Sister"
                } else {
                    "Sibling"
                },
                if is_male {
                    "Брат"
                } else if is_female {
                    "Сестра"
                } else {
                    "Брат / Сестра"
                },
            ),
            (2, 1) => (
                "pibling",
                if is_male {
                    "Uncle"
                } else if is_female {
                    "Aunt"
                } else {
                    "Uncle / Aunt"
                },
                if is_male {
                    "Дядя"
                } else if is_female {
                    "Тётя"
                } else {
                    "Дядя / Тётя"
                },
            ),
            (1, 2) => (
                "nibling",
                if is_male {
                    "Nephew"
                } else if is_female {
                    "Niece"
                } else {
                    "Nephew / Niece"
                },
                if is_male {
                    "Племянник"
                } else if is_female {
                    "Племянница"
                } else {
                    "Племянник / Племянница"
                },
            ),
            (2, 2) => (
                "first_cousin",
                "First cousin",
                if is_male {
                    "Двоюродный брат"
                } else if is_female {
                    "Двоюродная сестра"
                } else {
                    "Двоюродный брат / сестра"
                },
            ),
            _ => ("relative", "Blood relative", "Кровный родственник"),
        };

        return Some(ProfileKinshipDto {
            home_person_id: home_id.to_string(),
            target_person_id: target_id.to_string(),
            relation_key: relation_key.to_string(),
            label_en: label_en.to_string(),
            label_ru: label_ru.to_string(),
            steps,
            path_person_ids: path,
        });
    }

    // Check parent-in-law (parent of home person's spouse)
    if let Some(spouses) = spouses_of.get(home_id) {
        for &sp_id in spouses {
            if let Some(sp_parents) = parents_of.get(sp_id) {
                if sp_parents.contains(&target_id) {
                    let spouse_sex = person_by_id
                        .get(sp_id)
                        .map(|p| p.sex.trim().to_ascii_uppercase())
                        .unwrap_or_default();
                    let is_husband_side = spouse_sex == "M";
                    let relation_key = if is_husband_side {
                        "parent_in_law_husband"
                    } else {
                        "parent_in_law_wife"
                    };
                    let (label_en, label_ru) = if is_husband_side {
                        if is_male {
                            ("Father-in-law (husband's father)", "Свёкор")
                        } else if is_female {
                            ("Mother-in-law (husband's mother)", "Свекровь")
                        } else {
                            ("Parent-in-law", "Родитель мужа")
                        }
                    } else if is_male {
                        ("Father-in-law (wife's father)", "Тесть")
                    } else if is_female {
                        ("Mother-in-law (wife's mother)", "Тёща")
                    } else {
                        ("Parent-in-law", "Родитель жены")
                    };
                    return Some(ProfileKinshipDto {
                        home_person_id: home_id.to_string(),
                        target_person_id: target_id.to_string(),
                        relation_key: relation_key.to_string(),
                        label_en: label_en.to_string(),
                        label_ru: label_ru.to_string(),
                        steps: 2,
                        path_person_ids: vec![
                            home_id.to_string(),
                            sp_id.to_string(),
                            target_id.to_string(),
                        ],
                    });
                }
            }
        }
    }

    None
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct KinshipQuery {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub person_a: Option<String>,
    #[serde(default)]
    pub person_b: Option<String>,
    #[serde(default)]
    pub id1: Option<String>,
    #[serde(default)]
    pub id2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct KinshipEdgeStepDto {
    pub from_id: String,
    pub to_id: String,
    pub edge_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct KinshipResponse {
    pub connected: bool,
    pub from_id: String,
    pub to_id: String,
    pub relationship_key: String,
    pub relation_key: String,
    pub label_en: String,
    pub label_ru: String,
    pub degree_of_kinship: usize,
    pub steps_count: usize,
    pub up_steps: usize,
    pub down_steps: usize,
    pub is_blood_relation: bool,
    pub common_ancestor_ids: Vec<String>,
    pub path_person_ids: Vec<String>,
    pub steps: Vec<KinshipEdgeStepDto>,
    pub path_nodes: Vec<TreePersonNodeDto>,
}

/// Full Phase 2 Kinship Finder (`GET /api/v1/kinship?from={id1}&to={id2}`, `04-tree-visualizations.md §7.5`).
/// Uses cycle-safe LCA + bidirectional BFS across parent, child, and spouse edges.
pub fn compute_kinship_path(
    from_id: &str,
    to_id: &str,
    persons: &[TreePersonNodeDto],
    families: &[FamilyDto],
) -> KinshipResponse {
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        persons.iter().map(|p| (p.id.as_str(), p)).collect();

    if from_id == to_id {
        let node_vec = person_by_id
            .get(from_id)
            .map(|p| vec![(*p).clone()])
            .unwrap_or_default();
        return KinshipResponse {
            connected: true,
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relationship_key: "self".to_string(),
            relation_key: "self".to_string(),
            label_en: "Self".to_string(),
            label_ru: "Сам(а) (исходная персона)".to_string(),
            degree_of_kinship: 0,
            steps_count: 0,
            up_steps: 0,
            down_steps: 0,
            is_blood_relation: true,
            common_ancestor_ids: vec![from_id.to_string()],
            path_person_ids: vec![from_id.to_string()],
            steps: vec![],
            path_nodes: node_vec,
        };
    }

    let mut parents_of: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut children_of: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut spouses_of: HashMap<&str, Vec<&str>> = HashMap::new();

    for fam in families {
        let p1 = fam
            .partner1_id
            .as_deref()
            .filter(|id| person_by_id.contains_key(*id));
        let p2 = fam
            .partner2_id
            .as_deref()
            .filter(|id| person_by_id.contains_key(*id));

        if let (Some(a), Some(b)) = (p1, p2) {
            spouses_of.entry(a).or_default().push(b);
            spouses_of.entry(b).or_default().push(a);
        }

        for ch in &fam.children {
            let cid = ch.child_id.as_str();
            if !person_by_id.contains_key(cid) {
                continue;
            }
            for parent in [p1, p2].into_iter().flatten() {
                parents_of.entry(cid).or_default().push(parent);
                children_of.entry(parent).or_default().push(cid);
            }
        }
    }

    let target_sex = person_by_id
        .get(to_id)
        .map(|p| p.sex.trim().to_ascii_uppercase())
        .unwrap_or_default();
    let is_male = target_sex == "M";
    let is_female = target_sex == "F";

    // 1. Check blood relationship via Lowest Common Ancestor (LCA) up to 10 generations
    let build_ancestors = |start_id: &str| -> HashMap<String, (usize, Vec<String>)> {
        let mut result: HashMap<String, (usize, Vec<String>)> = HashMap::new();
        let mut queue: VecDeque<(String, usize, Vec<String>)> = VecDeque::new();
        result.insert(start_id.to_string(), (0, vec![start_id.to_string()]));
        queue.push_back((start_id.to_string(), 0, vec![start_id.to_string()]));

        while let Some((curr, depth, path)) = queue.pop_front() {
            if depth >= 10 {
                continue;
            }
            if let Some(parents) = parents_of.get(curr.as_str()) {
                for &p_id in parents {
                    if !result.contains_key(p_id) {
                        let mut next_path = path.clone();
                        next_path.push(p_id.to_string());
                        result.insert(p_id.to_string(), (depth + 1, next_path.clone()));
                        queue.push_back((p_id.to_string(), depth + 1, next_path));
                    }
                }
            }
        }
        result
    };

    let from_anc = build_ancestors(from_id);
    let to_anc = build_ancestors(to_id);

    let mut best_lca: Option<(usize, usize, String, Vec<String>)> = None;
    for (anc_id, (u_steps, path_u)) in &from_anc {
        if let Some((d_steps, path_d)) = to_anc.get(anc_id) {
            let total = *u_steps + *d_steps;
            let better = match &best_lca {
                None => true,
                Some((bu, bd, _, _)) => total < (*bu + *bd),
            };
            if better {
                let mut combined = path_u.clone();
                for node in path_d.iter().rev().skip(1) {
                    combined.push(node.clone());
                }
                best_lca = Some((*u_steps, *d_steps, anc_id.clone(), combined));
            }
        }
    }

    if let Some((up, down, lca_id, path)) = best_lca {
        let mut common_ancestors = vec![lca_id.clone()];
        for (aid, (u, _)) in &from_anc {
            if let Some((d, _)) = to_anc.get(aid) {
                if *u == up && *d == down && !common_ancestors.contains(aid) {
                    common_ancestors.push(aid.clone());
                }
            }
        }

        // Determine paternal vs maternal side from the first step upward (`path[1]`)
        let first_up_sex = if up >= 1 && path.len() >= 2 {
            person_by_id
                .get(path[1].as_str())
                .map(|p| p.sex.trim().to_ascii_uppercase())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let is_paternal = first_up_sex == "M";
        let is_maternal = first_up_sex == "F";

        let (rel_key, label_en, label_ru) = match (up, down) {
            (1, 0) => (
                if is_male {
                    "father".to_string()
                } else if is_female {
                    "mother".to_string()
                } else {
                    "parent".to_string()
                },
                if is_male {
                    "Father".to_string()
                } else if is_female {
                    "Mother".to_string()
                } else {
                    "Parent".to_string()
                },
                if is_male {
                    "Отец".to_string()
                } else if is_female {
                    "Мать".to_string()
                } else {
                    "Родитель".to_string()
                },
            ),
            (0, 1) => (
                if is_male {
                    "son".to_string()
                } else if is_female {
                    "daughter".to_string()
                } else {
                    "child".to_string()
                },
                if is_male {
                    "Son".to_string()
                } else if is_female {
                    "Daughter".to_string()
                } else {
                    "Child".to_string()
                },
                if is_male {
                    "Сын".to_string()
                } else if is_female {
                    "Дочь".to_string()
                } else {
                    "Ребёнок".to_string()
                },
            ),
            (1, 1) => (
                if is_male {
                    "brother".to_string()
                } else if is_female {
                    "sister".to_string()
                } else {
                    "sibling".to_string()
                },
                if is_male {
                    "Brother".to_string()
                } else if is_female {
                    "Sister".to_string()
                } else {
                    "Sibling".to_string()
                },
                if is_male {
                    "Брат".to_string()
                } else if is_female {
                    "Сестра".to_string()
                } else {
                    "Брат / Сестра".to_string()
                },
            ),
            (2, 0) => {
                let side_prefix = if is_paternal {
                    "paternal_"
                } else if is_maternal {
                    "maternal_"
                } else {
                    ""
                };
                let base = if is_male {
                    "grandfather"
                } else if is_female {
                    "grandmother"
                } else {
                    "grandparent"
                };
                (
                    format!("{side_prefix}{base}"),
                    if is_male {
                        "Grandfather".to_string()
                    } else if is_female {
                        "Grandmother".to_string()
                    } else {
                        "Grandparent".to_string()
                    },
                    if is_male {
                        "Дедушка".to_string()
                    } else if is_female {
                        "Бабушка".to_string()
                    } else {
                        "Дедушка / Бабушка".to_string()
                    },
                )
            }
            (0, 2) => (
                if is_male {
                    "grandson".to_string()
                } else if is_female {
                    "granddaughter".to_string()
                } else {
                    "grandchild".to_string()
                },
                if is_male {
                    "Grandson".to_string()
                } else if is_female {
                    "Granddaughter".to_string()
                } else {
                    "Grandchild".to_string()
                },
                if is_male {
                    "Внук".to_string()
                } else if is_female {
                    "Внучка".to_string()
                } else {
                    "Внук / Внучка".to_string()
                },
            ),
            (2, 1) => {
                let side_prefix = if is_paternal {
                    "paternal_"
                } else if is_maternal {
                    "maternal_"
                } else {
                    ""
                };
                let base = if is_male {
                    "uncle"
                } else if is_female {
                    "aunt"
                } else {
                    "pibling"
                };
                (
                    format!("{side_prefix}{base}"),
                    if is_male {
                        "Uncle".to_string()
                    } else if is_female {
                        "Aunt".to_string()
                    } else {
                        "Uncle / Aunt".to_string()
                    },
                    if is_male {
                        "Дядя".to_string()
                    } else if is_female {
                        "Тётя".to_string()
                    } else {
                        "Дядя / Тётя".to_string()
                    },
                )
            }
            (1, 2) => (
                if is_male {
                    "nephew".to_string()
                } else if is_female {
                    "niece".to_string()
                } else {
                    "nibling".to_string()
                },
                if is_male {
                    "Nephew".to_string()
                } else if is_female {
                    "Niece".to_string()
                } else {
                    "Nephew / Niece".to_string()
                },
                if is_male {
                    "Племянник".to_string()
                } else if is_female {
                    "Племянница".to_string()
                } else {
                    "Племянник / Племянница".to_string()
                },
            ),
            (u, 0) if u >= 3 => (
                if is_male {
                    "great_grandfather".to_string()
                } else if is_female {
                    "great_grandmother".to_string()
                } else {
                    "great_grandparent".to_string()
                },
                if is_male {
                    "Great-grandfather".to_string()
                } else if is_female {
                    "Great-grandmother".to_string()
                } else {
                    "Great-grandparent".to_string()
                },
                if is_male {
                    "Прадедушка".to_string()
                } else if is_female {
                    "Прабабушка".to_string()
                } else {
                    "Прародитель".to_string()
                },
            ),
            (0, d) if d >= 3 => (
                if is_male {
                    "great_grandchild".to_string()
                } else if is_female {
                    "great_granddaughter".to_string()
                } else {
                    "great_grandchild".to_string()
                },
                "Great-grandchild".to_string(),
                "Правнук / Правнучка".to_string(),
            ),
            (u, d) if u >= 2 && d >= 2 => {
                let cousin_deg = u.min(d) - 1;
                let removed = u.abs_diff(d);
                let deg_word = match cousin_deg {
                    1 => "first_cousin",
                    2 => "second_cousin",
                    3 => "third_cousin",
                    _ => "cousin",
                };
                let key = match removed {
                    0 => deg_word.to_string(),
                    1 => format!("{deg_word}_once_removed"),
                    2 => format!("{deg_word}_twice_removed"),
                    r => format!("{deg_word}_{r}_times_removed"),
                };
                let label_en = if removed == 0 {
                    format!(
                        "{} cousin",
                        match cousin_deg {
                            1 => "First",
                            2 => "Second",
                            3 => "Third",
                            _ => "Distant",
                        }
                    )
                } else {
                    format!("Cousin ({cousin_deg} degree, {removed}x removed)")
                };
                let label_ru = if cousin_deg == 1 && removed == 0 {
                    if is_male {
                        "Двоюродный брат".to_string()
                    } else if is_female {
                        "Двоюродная сестра".to_string()
                    } else {
                        "Двоюродный брат / сестра".to_string()
                    }
                } else {
                    "Кровный родственник".to_string()
                };
                (key, label_en, label_ru)
            }
            _ => (
                "blood_relative".to_string(),
                "Blood relative".to_string(),
                "Кровный родственник".to_string(),
            ),
        };

        let mut steps_vec = Vec::new();
        for idx in 0..(path.len().saturating_sub(1)) {
            let edge_type = if idx < up { "parent" } else { "child" };
            steps_vec.push(KinshipEdgeStepDto {
                from_id: path[idx].clone(),
                to_id: path[idx + 1].clone(),
                edge_type: edge_type.to_string(),
            });
        }

        let path_nodes: Vec<TreePersonNodeDto> = path
            .iter()
            .filter_map(|id| person_by_id.get(id.as_str()).map(|p| (*p).clone()))
            .collect();

        return KinshipResponse {
            connected: true,
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relationship_key: rel_key.clone(),
            relation_key: rel_key,
            label_en,
            label_ru,
            degree_of_kinship: up + down,
            steps_count: steps_vec.len(),
            up_steps: up,
            down_steps: down,
            is_blood_relation: true,
            common_ancestor_ids: common_ancestors,
            path_person_ids: path,
            steps: steps_vec,
            path_nodes,
        };
    }

    // 2. Fallback: General shortest-path BFS across parent, child, and spouse edges (affinal / in-law relations)
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, Vec<String>, Vec<KinshipEdgeStepDto>)> = VecDeque::new();
    visited.insert(from_id.to_string());
    queue.push_back((from_id.to_string(), vec![from_id.to_string()], vec![]));

    while let Some((curr, path, edge_steps)) = queue.pop_front() {
        if edge_steps.len() >= 12 {
            continue;
        }
        if curr == to_id {
            let (rel_key, label_en, label_ru) =
                if edge_steps.len() == 1 && edge_steps[0].edge_type == "spouse" {
                    (
                        if is_male {
                            "husband".to_string()
                        } else if is_female {
                            "wife".to_string()
                        } else {
                            "spouse".to_string()
                        },
                        if is_male {
                            "Husband".to_string()
                        } else if is_female {
                            "Wife".to_string()
                        } else {
                            "Spouse".to_string()
                        },
                        if is_male {
                            "Муж".to_string()
                        } else if is_female {
                            "Жена".to_string()
                        } else {
                            "Супруг(а)".to_string()
                        },
                    )
                } else if edge_steps.len() == 2
                    && edge_steps[0].edge_type == "spouse"
                    && edge_steps[1].edge_type == "parent"
                {
                    (
                        if is_male {
                            "father_in_law".to_string()
                        } else if is_female {
                            "mother_in_law".to_string()
                        } else {
                            "parent_in_law".to_string()
                        },
                        if is_male {
                            "Father-in-law".to_string()
                        } else if is_female {
                            "Mother-in-law".to_string()
                        } else {
                            "Parent-in-law".to_string()
                        },
                        if is_male {
                            "Тесть / Свёкор".to_string()
                        } else if is_female {
                            "Тёща / Свекровь".to_string()
                        } else {
                            "Родитель супруга(и)".to_string()
                        },
                    )
                } else {
                    (
                        "relative_by_marriage".to_string(),
                        "Relative by marriage".to_string(),
                        "Родственник по браку".to_string(),
                    )
                };

            let path_nodes: Vec<TreePersonNodeDto> = path
                .iter()
                .filter_map(|id| person_by_id.get(id.as_str()).map(|p| (*p).clone()))
                .collect();

            return KinshipResponse {
                connected: true,
                from_id: from_id.to_string(),
                to_id: to_id.to_string(),
                relationship_key: rel_key.clone(),
                relation_key: rel_key,
                label_en,
                label_ru,
                degree_of_kinship: edge_steps.len(),
                steps_count: edge_steps.len(),
                up_steps: 0,
                down_steps: 0,
                is_blood_relation: false,
                common_ancestor_ids: vec![],
                path_person_ids: path,
                steps: edge_steps,
                path_nodes,
            };
        }

        let mut neighbors: Vec<(&str, &str)> = Vec::new();
        if let Some(ps) = parents_of.get(curr.as_str()) {
            for &p in ps {
                neighbors.push((p, "parent"));
            }
        }
        if let Some(cs) = children_of.get(curr.as_str()) {
            for &c in cs {
                neighbors.push((c, "child"));
            }
        }
        if let Some(ss) = spouses_of.get(curr.as_str()) {
            for &s in ss {
                neighbors.push((s, "spouse"));
            }
        }

        for (next_id, edge_kind) in neighbors {
            if visited.insert(next_id.to_string()) {
                let mut next_path = path.clone();
                next_path.push(next_id.to_string());
                let mut next_edges = edge_steps.clone();
                next_edges.push(KinshipEdgeStepDto {
                    from_id: curr.clone(),
                    to_id: next_id.to_string(),
                    edge_type: edge_kind.to_string(),
                });
                queue.push_back((next_id.to_string(), next_path, next_edges));
            }
        }
    }

    KinshipResponse {
        connected: false,
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        relationship_key: "unconnected".to_string(),
        relation_key: "unconnected".to_string(),
        label_en: "No known relationship".to_string(),
        label_ru: "Родственная связь не найдена".to_string(),
        degree_of_kinship: 0,
        steps_count: 0,
        up_steps: 0,
        down_steps: 0,
        is_blood_relation: false,
        common_ancestor_ids: vec![],
        path_person_ids: vec![],
        steps: vec![],
        path_nodes: vec![],
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct FanChartQuery {
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub focal_id: Option<String>,
    #[serde(default)]
    pub generations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct FanChartNodeDto {
    pub ahnentafel: u32,
    pub generation: u32,
    pub quadrant: String,
    pub person: TreePersonNodeDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub father_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mother_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct FanChartResponse {
    pub root_id: String,
    pub generations: u32,
    pub total_nodes: usize,
    pub nodes: Vec<FanChartNodeDto>,
}

fn ahnentafel_quadrant(n: u32) -> String {
    if n == 1 {
        return "root".to_string();
    }
    if n == 2 {
        return "paternal".to_string();
    }
    if n == 3 {
        return "maternal".to_string();
    }
    // Walk down to the Gen-3 ancestor (4..=7) by dividing by 2
    let mut curr = n;
    while curr > 7 {
        curr /= 2;
    }
    match curr {
        4 => "paternal_grandfather".to_string(),
        5 => "paternal_grandmother".to_string(),
        6 => "maternal_grandfather".to_string(),
        7 => "maternal_grandmother".to_string(),
        _ => "root".to_string(),
    }
}

/// Computes the Ahnentafel-indexed Fan Chart hierarchy up to `generations` (4..=8, default 5) (`04-tree-visualizations.md §4.3`).
pub fn compute_fan_chart(
    root_id: &str,
    generations: u32,
    persons: &[TreePersonNodeDto],
    families: &[FamilyDto],
) -> FanChartResponse {
    let generations = generations.clamp(1, 8);
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        persons.iter().map(|p| (p.id.as_str(), p)).collect();

    let mut nodes = Vec::new();
    let mut visited_in_branch: HashSet<(u32, String)> = HashSet::new();
    let mut queue: VecDeque<(u32, u32, String)> = VecDeque::new();

    if person_by_id.contains_key(root_id) {
        queue.push_back((1, 1, root_id.to_string()));
        visited_in_branch.insert((1, root_id.to_string()));
    }

    while let Some((ahnentafel, gen_num, pid)) = queue.pop_front() {
        let Some(&person) = person_by_id.get(pid.as_str()) else {
            continue;
        };

        // Find parents of `pid`
        let mut father_id: Option<String> = None;
        let mut mother_id: Option<String> = None;

        if let Some(parent_fam) = families
            .iter()
            .find(|f| f.children.iter().any(|c| c.child_id == pid))
        {
            let p1 = parent_fam
                .partner1_id
                .as_deref()
                .and_then(|id| person_by_id.get(id).copied());
            let p2 = parent_fam
                .partner2_id
                .as_deref()
                .and_then(|id| person_by_id.get(id).copied());

            match (p1, p2) {
                (Some(a), Some(b)) => {
                    if a.sex.eq_ignore_ascii_case("F") || b.sex.eq_ignore_ascii_case("M") {
                        father_id = Some(b.id.clone());
                        mother_id = Some(a.id.clone());
                    } else {
                        father_id = Some(a.id.clone());
                        mother_id = Some(b.id.clone());
                    }
                }
                (Some(only), None) | (None, Some(only)) => {
                    if only.sex.eq_ignore_ascii_case("F") {
                        mother_id = Some(only.id.clone());
                    } else {
                        father_id = Some(only.id.clone());
                    }
                }
                (None, None) => {}
            }
        }

        if gen_num < generations {
            if let Some(ref fid) = father_id {
                if fid != &pid {
                    queue.push_back((ahnentafel * 2, gen_num + 1, fid.clone()));
                }
            }
            if let Some(ref mid) = mother_id {
                if mid != &pid {
                    queue.push_back((ahnentafel * 2 + 1, gen_num + 1, mid.clone()));
                }
            }
        }

        nodes.push(FanChartNodeDto {
            ahnentafel,
            generation: gen_num,
            quadrant: ahnentafel_quadrant(ahnentafel),
            person: person.clone(),
            father_id,
            mother_id,
        });
    }

    nodes.sort_by_key(|n| n.ahnentafel);
    let total_nodes = nodes.len();

    FanChartResponse {
        root_id: root_id.to_string(),
        generations,
        total_nodes,
        nodes,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceMapMarkerPersonDto {
    pub id: String,
    pub gid: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceMapMarkerDto {
    pub id: String,
    pub gid: String,
    pub name: String,
    pub full_name: String,
    pub place_type: String,
    pub latitude: f64,
    pub longitude: f64,
    pub events_count: usize,
    pub person_count: usize,
    pub event_types: Vec<String>,
    pub persons: Vec<PlaceMapMarkerPersonDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlacesMapResponse {
    pub items: Vec<PlaceMapMarkerDto>,
    pub total: usize,
}

/// Builds the clustered Places Map markers response (`GET /api/v1/places/map`, `12-places.md`).
pub fn compute_places_map(
    store: &GenealogyPrivacyStore,
    visible_persons: &[TreePersonNodeDto],
) -> PlacesMapResponse {
    let visible_by_id: HashMap<&str, &TreePersonNodeDto> =
        visible_persons.iter().map(|p| (p.id.as_str(), p)).collect();

    let mut items = Vec::new();

    for place in &store.places {
        let (Some(lat), Some(lng)) = (place.latitude, place.longitude) else {
            continue;
        };

        let mut person_ids: HashSet<String> = HashSet::new();
        let mut event_types_set: HashSet<String> = HashSet::new();
        let mut matched_events_count = 0usize;

        for ev in &store.events {
            let matches_place = ev.place_id.as_deref() == Some(place.id.as_str())
                || ev
                    .place_full_name
                    .as_deref()
                    .map(|pn| {
                        pn.eq_ignore_ascii_case(&place.name)
                            || pn.eq_ignore_ascii_case(&place.full_name)
                    })
                    .unwrap_or(false);
            if matches_place {
                matched_events_count += 1;
                event_types_set.insert(ev.event_type.clone());
                if let Some(pid) = ev.individual_id.as_deref() {
                    if visible_by_id.contains_key(pid) {
                        person_ids.insert(pid.to_string());
                    }
                }
            }
        }

        // Also match person birth/death summary places by name/full_name
        for vp in visible_persons {
            if let Some(bp) = vp.birth.as_ref().and_then(|b| b.place.as_deref()) {
                if bp.eq_ignore_ascii_case(&place.name)
                    || bp.eq_ignore_ascii_case(&place.full_name)
                    || bp.contains(&place.name)
                {
                    person_ids.insert(vp.id.clone());
                    event_types_set.insert("birth".to_string());
                }
            }
            if let Some(dp) = vp.death.as_ref().and_then(|d| d.place.as_deref()) {
                if dp.eq_ignore_ascii_case(&place.name)
                    || dp.eq_ignore_ascii_case(&place.full_name)
                    || dp.contains(&place.name)
                {
                    person_ids.insert(vp.id.clone());
                    event_types_set.insert("death".to_string());
                }
            }
        }

        let events_count = matched_events_count.max(place.events_count as usize);
        let mut persons_list: Vec<PlaceMapMarkerPersonDto> = person_ids
            .iter()
            .filter_map(|pid| visible_by_id.get(pid.as_str()))
            .map(|p| {
                let display_name = [
                    p.primary_name.given_name.trim(),
                    p.primary_name.patronymic.as_deref().unwrap_or("").trim(),
                    p.primary_name.surname.trim(),
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
                PlaceMapMarkerPersonDto {
                    id: p.id.clone(),
                    gid: p.gid.clone(),
                    display_name,
                }
            })
            .collect();
        persons_list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        let person_count = persons_list.len();
        persons_list.truncate(10);

        let mut event_types: Vec<String> = event_types_set.into_iter().collect();
        event_types.sort();

        items.push(PlaceMapMarkerDto {
            id: place.id.clone(),
            gid: place.gid.clone(),
            name: place.name.clone(),
            full_name: place.full_name.clone(),
            place_type: place.place_type.clone(),
            latitude: lat,
            longitude: lng,
            events_count,
            person_count,
            event_types,
            persons: persons_list,
        });
    }

    let total = items.len();
    PlacesMapResponse { items, total }
}
