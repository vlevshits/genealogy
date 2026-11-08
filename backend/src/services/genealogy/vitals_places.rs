#[allow(unused_imports)]
use super::*;

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
                if !crate::services::quality::is_placeholder_place_name(pname) {
                    chain_names.push(pname.clone());
                }
                curr_parent = next_parent.clone();
            } else {
                break;
            }
        }

        if chain_names.len() > 1 {
            place.full_name = chain_names.join(", ");
            place.country = chain_names.last().cloned();
        } else if place.full_name.trim().is_empty()
            || (place.id != "00000000-0000-0000-0000-000000000201"
                && place.id != "00000000-0000-0000-0000-000000000202")
        {
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
        if p.id == place_id || !crate::services::quality::is_placeholder_place_name(&p.name) {
            crumbs.push(PlaceBreadcrumbDto {
                id: p.id.clone(),
                gid: p.gid.clone(),
                name: p.name.clone(),
                place_type: p.place_type.clone(),
            });
        }
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
        .filter(|s| !s.is_empty() && !crate::services::quality::is_placeholder_place_name(s))
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
        let (_, inferred_opt) = crate::services::quality::infer_place_type_and_clean_name(seg_name);
        let inferred_type = if let Some(ref inf_type) = inferred_opt {
            inf_type.as_str()
        } else if is_leaf {
            leaf_place_type.unwrap_or(if total_levels >= 3 { "city" } else { "other" })
        } else if depth_from_top == 0 && total_levels >= 2 {
            "country"
        } else if depth_from_top == 1 {
            "region"
        } else {
            "district"
        };

        if inferred_type == "country" {
            current_parent_id = None;
        }

        // Check if a place with this name and parent already exists (or when single segment, matches name/full_name)
        let existing_idx = store.places.iter().position(|p| {
            if crate::services::quality::is_placeholder_place_name(&p.name) {
                return false;
            }
            let name_matches = p.name.eq_ignore_ascii_case(seg_name)
                || p.alt_names
                    .iter()
                    .any(|a| a.name.eq_ignore_ascii_case(seg_name));
            if !name_matches {
                return false;
            }
            if total_levels == 1 {
                true
            } else if depth_from_top == 0 || inferred_type == "country" {
                let parent_is_placeholder = p
                    .parent_id
                    .as_deref()
                    .and_then(|pid| store.places.iter().find(|cand| cand.id == pid))
                    .is_some_and(|cand| {
                        crate::services::quality::is_placeholder_place_name(&cand.name)
                    });
                p.parent_id == current_parent_id || p.parent_id.is_none() || parent_is_placeholder
            } else {
                p.parent_id == current_parent_id
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
