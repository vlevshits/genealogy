#[allow(unused_imports)]
use super::*;

pub(crate) fn analyze_single_place_issue(
    place: &PlaceSummaryDto,
    store: &GenealogyPrivacyStore,
    include_dismissed: bool,
) -> Option<PlaceHierarchyIssueDto> {
    let parent_name = place
        .parent_id
        .as_deref()
        .and_then(|pid| store.places.iter().find(|p| p.id == pid))
        .map(|p| p.full_name.clone());

    let has_delimiters_in_name =
        place.name.contains(',') || place.name.contains(';') || place.name.contains('/');
    let has_unlinked_full_name_path = place.parent_id.is_none()
        && (place.full_name.contains(',')
            || place.full_name.contains(';')
            || place.full_name.contains('/'));

    // --- Category 1: Unparsed Multi-Segment Path (`PLACE_UNPARSED_PATH`) ---
    if has_delimiters_in_name || has_unlinked_full_name_path {
        let raw_source = if has_delimiters_in_name {
            &place.name
        } else {
            &place.full_name
        };
        let segments = split_jurisdiction_segments(raw_source);
        if segments.len() >= 2 {
            let issue_id = format!("{RULE_PLACE_UNPARSED_PATH}:{}", place.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &issue_id,
                RULE_PLACE_UNPARSED_PATH,
                &place.id,
                None,
            );
            if !include_dismissed && dismissed {
                return None;
            }

            let total_levels = segments.len();
            let mut proposed_chain = Vec::with_capacity(total_levels);
            let mut curr_parent_id: Option<String> = None;

            // Traverse right-to-left (top-most jurisdiction down to leaf)
            for (depth_from_top, seg) in segments.iter().rev().enumerate() {
                let is_leaf = depth_from_top + 1 == total_levels;
                let (clean_seg, inferred_opt) = infer_place_type_and_clean_name(seg);

                let seg_type = if let Some(inf) = inferred_opt {
                    inf
                } else if is_leaf {
                    if place.place_type != "other" {
                        place.place_type.clone()
                    } else if total_levels >= 4 || looks_like_rural_locality_name(&clean_seg) {
                        "village".to_string()
                    } else {
                        "city".to_string()
                    }
                } else if depth_from_top == 0 && total_levels >= 3 {
                    "country".to_string()
                } else if (depth_from_top == 0 && total_levels == 2) || depth_from_top == 1 {
                    "region".to_string()
                } else {
                    "district".to_string()
                };

                if is_leaf {
                    proposed_chain.push(ProposedHierarchyStepDto {
                        name: clean_seg,
                        place_type: seg_type,
                        existing_place_id: Some(place.id.clone()),
                        existing_place_gid: Some(place.gid.clone()),
                        is_new: false,
                    });
                } else {
                    let existing = find_matching_place_in_store(
                        &store.places,
                        &clean_seg,
                        curr_parent_id.as_deref(),
                        &place.id,
                    );
                    if let Some(ex) = existing {
                        curr_parent_id = Some(ex.id.clone());
                        proposed_chain.push(ProposedHierarchyStepDto {
                            name: ex.name.clone(),
                            place_type: if ex.place_type == "other" {
                                seg_type
                            } else {
                                ex.place_type.clone()
                            },
                            existing_place_id: Some(ex.id.clone()),
                            existing_place_gid: Some(ex.gid.clone()),
                            is_new: false,
                        });
                    } else {
                        curr_parent_id = None;
                        proposed_chain.push(ProposedHierarchyStepDto {
                            name: clean_seg,
                            place_type: seg_type,
                            existing_place_id: None,
                            existing_place_gid: None,
                            is_new: true,
                        });
                    }
                }
            }

            let leaf_step = proposed_chain.last().cloned().unwrap();
            let parent_step = if proposed_chain.len() >= 2 {
                Some(&proposed_chain[proposed_chain.len() - 2])
            } else {
                None
            };

            let mut full_parts: Vec<String> = proposed_chain
                .iter()
                .rev()
                .map(|s| s.name.clone())
                .collect();
            if full_parts.is_empty() {
                full_parts.push(leaf_step.name.clone());
            }
            let proposed_full_name = full_parts.join(", ");

            // Check if a clean place with this leaf name already exists under the resolved parent
            let dup_target = store.places.iter().find(|p| {
                p.id != place.id
                    && !p.name.contains(',')
                    && normalize_place_match_key(&p.name)
                        == normalize_place_match_key(&leaf_step.name)
                    && (parent_step
                        .and_then(|ps| ps.existing_place_id.as_deref())
                        .is_some_and(|pid| p.parent_id.as_deref() == Some(pid))
                        || normalize_place_match_key(&p.full_name)
                            == normalize_place_match_key(&proposed_full_name))
            });

            let confidence = 95;
            return Some(PlaceHierarchyIssueDto {
                id: issue_id,
                place_id: place.id.clone(),
                place_gid: place.gid.clone(),
                issue_type: "unparsed_path".to_string(),
                rule_code: RULE_PLACE_UNPARSED_PATH.to_string(),
                severity: "warning".to_string(),
                confidence,
                confidence_tier: confidence_tier(confidence),
                current_name: place.name.clone(),
                current_full_name: place.full_name.clone(),
                current_place_type: place.place_type.clone(),
                current_parent_id: place.parent_id.clone(),
                current_parent_name: parent_name,
                latitude: place.latitude,
                longitude: place.longitude,
                events_count: place.events_count,
                children_count: place.children_count,
                proposed_leaf_name: leaf_step.name.clone(),
                proposed_clean_name: leaf_step.name.clone(),
                proposed_place_type: leaf_step.place_type.clone(),
                proposed_parent_id: parent_step.and_then(|s| s.existing_place_id.clone()),
                proposed_parent_name: parent_step.map(|s| s.name.clone()),
                proposed_full_name,
                proposed_chain,
                parent_candidates: vec![],
                consolidate_with_place_id: dup_target.map(|d| d.id.clone()),
                consolidate_with_name: dup_target.map(|d| d.full_name.clone()),
                distance_km: None,
                reason: "Right-to-left jurisdiction decomposition; original raw string archived to alternate names.".to_string(),
                preserve_alt_name: Some(place.name.clone()),
            });
        }
    }

    let (clean_name, inferred_type_opt) = infer_place_type_and_clean_name(&place.name);
    let effective_type = if let Some(ref inf) = inferred_type_opt {
        inf.clone()
    } else if place.place_type != "other" {
        place.place_type.clone()
    } else if looks_like_rural_locality_name(&clean_name) {
        "village".to_string()
    } else {
        "other".to_string()
    };

    // --- Category 2: Duplicate Place Node under Same Parent or Structured Equivalent (`PLACE_DUPLICATE_CANDIDATE`) ---
    let place_norm_key = normalize_place_match_key(&clean_name);
    if !place_norm_key.is_empty() {
        let duplicate_target = store.places.iter().find(|other| {
            if other.id == place.id
                || other.name.contains(',')
                || other.name.contains(';')
                || other.name.contains('/')
            {
                return false;
            }
            if normalize_place_match_key(&other.name) != place_norm_key {
                return false;
            }
            if place.parent_id == other.parent_id {
                // Prefer keeping the one with clean name (no abbreviation), or more events, or lower id
                let other_is_cleaner = other.name == infer_place_type_and_clean_name(&other.name).0
                    && place.name != clean_name;
                let other_has_more_events = other.events_count > place.events_count;
                other_is_cleaner
                    || other_has_more_events
                    || (other.events_count == place.events_count && other.id < place.id)
            } else {
                // If `place` is an orphan (`parent_id == None`) and `other` already has a parent hierarchy
                place.parent_id.is_none()
                    && other.parent_id.is_some()
                    && (place.latitude.is_none()
                        || other.latitude.is_none()
                        || haversine_distance_km(
                            place.latitude.unwrap_or(0.0),
                            place.longitude.unwrap_or(0.0),
                            other.latitude.unwrap_or(0.0),
                            other.longitude.unwrap_or(0.0),
                        ) <= 25.0)
            }
        });

        if let Some(dup) = duplicate_target {
            let issue_id = format!("{RULE_PLACE_DUPLICATE_CANDIDATE}:{}", place.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &issue_id,
                RULE_PLACE_DUPLICATE_CANDIDATE,
                &place.id,
                Some(&dup.id),
            );
            if include_dismissed || !dismissed {
                let mut chain = if let Some(ref pid) = dup.parent_id {
                    build_ancestor_steps_for_parent(&store.places, pid)
                } else {
                    vec![]
                };
                chain.push(ProposedHierarchyStepDto {
                    name: dup.name.clone(),
                    place_type: dup.place_type.clone(),
                    existing_place_id: Some(dup.id.clone()),
                    existing_place_gid: Some(dup.gid.clone()),
                    is_new: false,
                });
                let confidence = 88;
                return Some(PlaceHierarchyIssueDto {
                    id: issue_id,
                    place_id: place.id.clone(),
                    place_gid: place.gid.clone(),
                    issue_type: "duplicate_node".to_string(),
                    rule_code: RULE_PLACE_DUPLICATE_CANDIDATE.to_string(),
                    severity: "warning".to_string(),
                    confidence,
                    confidence_tier: confidence_tier(confidence),
                    current_name: place.name.clone(),
                    current_full_name: place.full_name.clone(),
                    current_place_type: place.place_type.clone(),
                    current_parent_id: place.parent_id.clone(),
                    current_parent_name: parent_name,
                    latitude: place.latitude,
                    longitude: place.longitude,
                    events_count: place.events_count,
                    children_count: place.children_count,
                    proposed_leaf_name: dup.name.clone(),
                    proposed_clean_name: dup.name.clone(),
                    proposed_place_type: dup.place_type.clone(),
                    proposed_parent_id: dup.parent_id.clone(),
                    proposed_parent_name: dup
                        .parent_id
                        .as_deref()
                        .and_then(|pid| store.places.iter().find(|p| p.id == pid))
                        .map(|p| p.full_name.clone()),
                    proposed_full_name: dup.full_name.clone(),
                    proposed_chain: chain,
                    parent_candidates: vec![],
                    consolidate_with_place_id: Some(dup.id.clone()),
                    consolidate_with_name: Some(dup.full_name.clone()),
                    distance_km: None,
                    reason: format!(
                        "Duplicate place record matching structured node '{}' ({}).",
                        dup.full_name, dup.gid
                    ),
                    preserve_alt_name: if place.name != dup.name {
                        Some(place.name.clone())
                    } else {
                        None
                    },
                });
            }
        }
    }

    // --- Category 3: Orphan Hierarchy (`PLACE_ORPHAN_HIERARCHY`) ---
    let my_rank = place_hierarchy_rank(if effective_type == "other" {
        "city"
    } else {
        &effective_type
    });
    if place.parent_id.is_none() && effective_type != "country" {
        let my_stem = extract_jurisdiction_stem(&clean_name);
        let mut candidates: Vec<ParentPlaceCandidateDto> = Vec::new();

        for cand in &store.places {
            if cand.id == place.id
                || cand.name.contains(',')
                || cand.name.contains(';')
                || ensure_no_place_cycle(&store.places, &place.id, Some(&cand.id)).is_err()
            {
                continue;
            }

            let (_, cand_inferred) = infer_place_type_and_clean_name(&cand.name);
            let cand_eff_type = if cand.place_type != "other" {
                cand.place_type.clone()
            } else if let Some(inf) = cand_inferred {
                inf
            } else {
                continue;
            };
            let cand_rank = place_hierarchy_rank(&cand_eff_type);
            if cand_rank >= my_rank || cand_rank > 3 {
                continue;
            }

            let cand_stem = extract_jurisdiction_stem(&cand.name);
            let stem_matches = match (&my_stem, &cand_stem) {
                (Some(ms), Some(cs)) => ms == cs,
                _ => false,
            };

            // Spatial proximity & child cluster bounding box check
            let mut min_dist_km: Option<f64> = None;
            let mut inside_bounding_box = false;

            if let (Some(lat), Some(lon)) = (place.latitude, place.longitude) {
                if let (Some(clat), Some(clon)) = (cand.latitude, cand.longitude) {
                    min_dist_km = Some(haversine_distance_km(lat, lon, clat, clon));
                }
                let child_coords = collect_descendant_coordinates(&store.places, &cand.id);
                if !child_coords.is_empty() {
                    for &(ch_lat, ch_lon) in &child_coords {
                        let d = haversine_distance_km(lat, lon, ch_lat, ch_lon);
                        min_dist_km = Some(min_dist_km.map_or(d, |prev| prev.min(d)));
                    }
                    let min_lat = child_coords
                        .iter()
                        .map(|c| c.0)
                        .fold(f64::INFINITY, f64::min);
                    let max_lat = child_coords
                        .iter()
                        .map(|c| c.0)
                        .fold(f64::NEG_INFINITY, f64::max);
                    let min_lon = child_coords
                        .iter()
                        .map(|c| c.1)
                        .fold(f64::INFINITY, f64::min);
                    let max_lon = child_coords
                        .iter()
                        .map(|c| c.1)
                        .fold(f64::NEG_INFINITY, f64::max);
                    let lat_span = (max_lat - min_lat).max(0.20) * 0.15;
                    let lon_span = (max_lon - min_lon).max(0.20) * 0.15;
                    if lat >= min_lat - lat_span
                        && lat <= max_lat + lat_span
                        && lon >= min_lon - lon_span
                        && lon <= max_lon + lon_span
                    {
                        inside_bounding_box = true;
                    }
                }
            }

            let max_allowed_km = match cand_eff_type.as_str() {
                "district" => 40.0,
                "region" => 200.0,
                "country" => 1500.0,
                _ => 50.0,
            };
            let spatial_matches =
                inside_bounding_box || min_dist_km.is_some_and(|d| d <= max_allowed_km);

            if stem_matches && spatial_matches {
                let d_rounded = min_dist_km.map(|d| (d * 10.0).round() / 10.0);
                candidates.push(ParentPlaceCandidateDto {
                    place_id: cand.id.clone(),
                    gid: cand.gid.clone(),
                    name: cand.name.clone(),
                    full_name: cand.full_name.clone(),
                    place_type: cand_eff_type,
                    distance_km: d_rounded,
                    confidence: 92,
                    match_reason: format!(
                        "Administrative stem and coordinate proximity match '{}'",
                        cand.full_name
                    ),
                });
            } else if stem_matches {
                candidates.push(ParentPlaceCandidateDto {
                    place_id: cand.id.clone(),
                    gid: cand.gid.clone(),
                    name: cand.name.clone(),
                    full_name: cand.full_name.clone(),
                    place_type: cand_eff_type,
                    distance_km: min_dist_km.map(|d| (d * 10.0).round() / 10.0),
                    confidence: 88,
                    match_reason: format!(
                        "Administrative stem matches parent jurisdiction '{}'",
                        cand.full_name
                    ),
                });
            } else if spatial_matches {
                let d_rounded = min_dist_km.map(|d| (d * 10.0).round() / 10.0);
                let conf = if cand_eff_type == "district" || inside_bounding_box {
                    88
                } else {
                    85
                };
                candidates.push(ParentPlaceCandidateDto {
                    place_id: cand.id.clone(),
                    gid: cand.gid.clone(),
                    name: cand.name.clone(),
                    full_name: cand.full_name.clone(),
                    place_type: cand_eff_type,
                    distance_km: d_rounded,
                    confidence: conf,
                    match_reason: if let Some(d) = d_rounded {
                        format!(
                            "Coordinates within {:.1} km of parent {} '{}'",
                            d, cand.place_type, cand.full_name
                        )
                    } else {
                        format!(
                            "Coordinates inside child cluster envelope of '{}'",
                            cand.full_name
                        )
                    },
                });
            }
        }

        // Sort candidates: prefer highest confidence, then more specific rank (district > region > country), then shortest distance
        candidates.sort_by(|a, b| {
            b.confidence
                .cmp(&a.confidence)
                .then_with(|| {
                    place_hierarchy_rank(&b.place_type).cmp(&place_hierarchy_rank(&a.place_type))
                })
                .then_with(|| {
                    a.distance_km
                        .unwrap_or(9999.0)
                        .partial_cmp(&b.distance_km.unwrap_or(9999.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        // If multiple distinct candidates at the exact same rank matched by coordinates alone without stem, lower confidence to 75 ("medium")
        if candidates.len() >= 2
            && candidates[0].place_type == candidates[1].place_type
            && !candidates[0].match_reason.contains("stem")
        {
            candidates[0].confidence = 75;
        }

        if let Some(best) = candidates.first().cloned() {
            let issue_id = format!("{RULE_PLACE_ORPHAN_HIERARCHY}:{}", place.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &issue_id,
                RULE_PLACE_ORPHAN_HIERARCHY,
                &place.id,
                None,
            );
            if include_dismissed || !dismissed {
                let mut chain = build_ancestor_steps_for_parent(&store.places, &best.place_id);
                let leaf_type = if effective_type == "other" {
                    "village".to_string()
                } else {
                    effective_type.clone()
                };
                chain.push(ProposedHierarchyStepDto {
                    name: clean_name.clone(),
                    place_type: leaf_type.clone(),
                    existing_place_id: Some(place.id.clone()),
                    existing_place_gid: Some(place.gid.clone()),
                    is_new: false,
                });
                let proposed_full_name = format!("{}, {}", clean_name, best.full_name);

                return Some(PlaceHierarchyIssueDto {
                    id: issue_id,
                    place_id: place.id.clone(),
                    place_gid: place.gid.clone(),
                    issue_type: "orphan_hierarchy".to_string(),
                    rule_code: RULE_PLACE_ORPHAN_HIERARCHY.to_string(),
                    severity: "warning".to_string(),
                    confidence: best.confidence,
                    confidence_tier: confidence_tier(best.confidence),
                    current_name: place.name.clone(),
                    current_full_name: place.full_name.clone(),
                    current_place_type: place.place_type.clone(),
                    current_parent_id: None,
                    current_parent_name: None,
                    latitude: place.latitude,
                    longitude: place.longitude,
                    events_count: place.events_count,
                    children_count: place.children_count,
                    proposed_leaf_name: clean_name.clone(),
                    proposed_clean_name: clean_name.clone(),
                    proposed_place_type: leaf_type,
                    proposed_parent_id: Some(best.place_id.clone()),
                    proposed_parent_name: Some(best.full_name.clone()),
                    proposed_full_name,
                    proposed_chain: chain,
                    parent_candidates: candidates,
                    consolidate_with_place_id: None,
                    consolidate_with_name: None,
                    distance_km: best.distance_km,
                    reason: best.match_reason,
                    preserve_alt_name: if place.name != clean_name {
                        Some(place.name.clone())
                    } else {
                        None
                    },
                });
            }
        }
    }

    // --- Category 4: Place Type Mismatch or Unstripped Locality Prefix (`PLACE_TYPE_MISMATCH`) ---
    if let Some(inferred_type) = inferred_type_opt {
        if place.place_type != inferred_type || place.name != clean_name {
            let issue_id = format!("{RULE_PLACE_TYPE_MISMATCH}:{}", place.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &issue_id,
                RULE_PLACE_TYPE_MISMATCH,
                &place.id,
                None,
            );
            if include_dismissed || !dismissed {
                let mut chain = if let Some(ref pid) = place.parent_id {
                    build_ancestor_steps_for_parent(&store.places, pid)
                } else {
                    vec![]
                };
                chain.push(ProposedHierarchyStepDto {
                    name: clean_name.clone(),
                    place_type: inferred_type.clone(),
                    existing_place_id: Some(place.id.clone()),
                    existing_place_gid: Some(place.gid.clone()),
                    is_new: false,
                });
                let proposed_full_name = if let Some(ref pname) = parent_name {
                    format!("{clean_name}, {pname}")
                } else {
                    clean_name.clone()
                };
                let confidence = 90;
                return Some(PlaceHierarchyIssueDto {
                    id: issue_id,
                    place_id: place.id.clone(),
                    place_gid: place.gid.clone(),
                    issue_type: "type_mismatch".to_string(),
                    rule_code: RULE_PLACE_TYPE_MISMATCH.to_string(),
                    severity: "info".to_string(),
                    confidence,
                    confidence_tier: confidence_tier(confidence),
                    current_name: place.name.clone(),
                    current_full_name: place.full_name.clone(),
                    current_place_type: place.place_type.clone(),
                    current_parent_id: place.parent_id.clone(),
                    current_parent_name: parent_name.clone(),
                    latitude: place.latitude,
                    longitude: place.longitude,
                    events_count: place.events_count,
                    children_count: place.children_count,
                    proposed_leaf_name: clean_name.clone(),
                    proposed_clean_name: clean_name.clone(),
                    proposed_place_type: inferred_type.clone(),
                    proposed_parent_id: place.parent_id.clone(),
                    proposed_parent_name: parent_name,
                    proposed_full_name,
                    proposed_chain: chain,
                    parent_candidates: vec![],
                    consolidate_with_place_id: None,
                    consolidate_with_name: None,
                    distance_km: None,
                    reason: format!(
                        "Administrative keyword indicates place_type '{}' (currently '{}').",
                        inferred_type, place.place_type
                    ),
                    preserve_alt_name: if place.name != clean_name {
                        Some(place.name.clone())
                    } else {
                        None
                    },
                });
            }
        }
    }

    // Fallback Low-Confidence Orphan Locality if parent_id is None and other higher-level places exist in the tree
    if place.parent_id.is_none()
        && effective_type != "country"
        && effective_type != "region"
        && store.places.len() > 1
    {
        let issue_id = format!("{RULE_PLACE_ORPHAN_HIERARCHY}:{}", place.id);
        let dismissed = is_warning_dismissed(
            &store.dismissed_warnings,
            &issue_id,
            RULE_PLACE_ORPHAN_HIERARCHY,
            &place.id,
            None,
        );
        if include_dismissed || !dismissed {
            let leaf_type = if effective_type == "other" {
                "city".to_string()
            } else {
                effective_type
            };
            let confidence = 50;
            return Some(PlaceHierarchyIssueDto {
                id: issue_id,
                place_id: place.id.clone(),
                place_gid: place.gid.clone(),
                issue_type: "orphan_hierarchy".to_string(),
                rule_code: RULE_PLACE_ORPHAN_HIERARCHY.to_string(),
                severity: "warning".to_string(),
                confidence,
                confidence_tier: confidence_tier(confidence),
                current_name: place.name.clone(),
                current_full_name: place.full_name.clone(),
                current_place_type: place.place_type.clone(),
                current_parent_id: None,
                current_parent_name: None,
                latitude: place.latitude,
                longitude: place.longitude,
                events_count: place.events_count,
                children_count: place.children_count,
                proposed_leaf_name: clean_name.clone(),
                proposed_clean_name: clean_name.clone(),
                proposed_place_type: leaf_type.clone(),
                proposed_parent_id: None,
                proposed_parent_name: None,
                proposed_full_name: clean_name.clone(),
                proposed_chain: vec![ProposedHierarchyStepDto {
                    name: clean_name,
                    place_type: leaf_type,
                    existing_place_id: Some(place.id.clone()),
                    existing_place_gid: Some(place.gid.clone()),
                    is_new: false,
                }],
                parent_candidates: vec![],
                consolidate_with_place_id: None,
                consolidate_with_name: None,
                distance_km: None,
                reason: "Locality has no parent jurisdiction assigned.".to_string(),
                preserve_alt_name: None,
            });
        }
    }

    None
}
