#[allow(unused_imports)]
use super::*;

pub(crate) fn analyze_single_place_issue(
    place: &PlaceSummaryDto,
    store: &GenealogyPrivacyStore,
    include_dismissed: bool,
) -> Option<PlaceHierarchyIssueDto> {
    if is_placeholder_place_name(&place.name) {
        return None;
    }

    let parent_is_placeholder = place
        .parent_id
        .as_deref()
        .and_then(|pid| store.places.iter().find(|p| p.id == pid))
        .is_some_and(|p| is_placeholder_place_name(&p.name));

    let parent_name = if parent_is_placeholder {
        None
    } else {
        place
            .parent_id
            .as_deref()
            .and_then(|pid| store.places.iter().find(|p| p.id == pid))
            .map(|p| p.full_name.clone())
    };
    let effective_current_parent_id = if parent_is_placeholder {
        None
    } else {
        place.parent_id.clone()
    };

    let has_delimiters_in_name =
        place.name.contains(',') || place.name.contains(';') || place.name.contains('/');
    let has_unlinked_full_name_path = (place.parent_id.is_none() || parent_is_placeholder)
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

                if seg_type == "country" {
                    curr_parent_id = None;
                }

                if is_leaf {
                    proposed_chain.push(ProposedHierarchyStepDto {
                        name: clean_seg,
                        place_type: seg_type,
                        existing_place_id: Some(place.id.clone()),
                        existing_place_gid: Some(place.gid.clone()),
                        is_new: false,
                        action: "clean_leaf".to_string(),
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
                            place_type: if ex.place_type == "other" || seg_type == "country" {
                                seg_type
                            } else {
                                ex.place_type.clone()
                            },
                            existing_place_id: Some(ex.id.clone()),
                            existing_place_gid: Some(ex.gid.clone()),
                            is_new: false,
                            action: "reuse".to_string(),
                        });
                    } else {
                        curr_parent_id = None;
                        proposed_chain.push(ProposedHierarchyStepDto {
                            name: clean_seg,
                            place_type: seg_type,
                            existing_place_id: None,
                            existing_place_gid: None,
                            is_new: true,
                            action: "create".to_string(),
                        });
                    }
                }
            }

            let leaf_step = proposed_chain.last().cloned().unwrap();
            let parent_step = if proposed_chain.len() >= 2 {
                Some(proposed_chain[proposed_chain.len() - 2].clone())
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
                    && !is_placeholder_place_name(&p.name)
                    && !p.name.contains(',')
                    && normalize_place_match_key(&p.name)
                        == normalize_place_match_key(&leaf_step.name)
                    && (parent_step
                        .as_ref()
                        .and_then(|ps| ps.existing_place_id.as_deref())
                        .is_some_and(|pid| p.parent_id.as_deref() == Some(pid))
                        || normalize_place_match_key(&p.full_name)
                            == normalize_place_match_key(&proposed_full_name))
            });

            if dup_target.is_some() {
                if let Some(last) = proposed_chain.last_mut() {
                    last.action = "consolidate".to_string();
                }
            }

            return Some(build_place_issue_dto(
                issue_id,
                place,
                "unparsed_path",
                RULE_PLACE_UNPARSED_PATH,
                "warning",
                95,
                effective_current_parent_id,
                parent_name,
                leaf_step.name,
                leaf_step.place_type,
                parent_step.as_ref().and_then(|s| s.existing_place_id.clone()),
                parent_step.map(|s| s.name),
                proposed_full_name,
                proposed_chain,
                vec![],
                dup_target.map(|d| d.id.clone()),
                dup_target.map(|d| d.full_name.clone()),
                None,
                "Right-to-left jurisdiction decomposition; original raw string archived to alternate names.".to_string(),
                Some(place.name.clone()),
            ));
        }
    }

    let single_seg_after_strip = if has_delimiters_in_name {
        let segs = split_jurisdiction_segments(&place.name);
        if segs.len() == 1 {
            segs.into_iter().next()
        } else {
            None
        }
    } else {
        None
    };
    let name_for_inference = single_seg_after_strip.as_deref().unwrap_or(&place.name);
    let (clean_name, inferred_type_opt) = infer_place_type_and_clean_name(name_for_inference);
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
                || is_placeholder_place_name(&other.name)
                || other.name.contains(',')
                || other.name.contains(';')
                || other.name.contains('/')
            {
                return false;
            }
            if normalize_place_match_key(&other.name) != place_norm_key {
                return false;
            }
            let other_parent_is_placeholder = other
                .parent_id
                .as_deref()
                .and_then(|pid| store.places.iter().find(|p| p.id == pid))
                .is_some_and(|p| is_placeholder_place_name(&p.name));

            if effective_type == "country" {
                if place.parent_id.is_none() && place.place_type == "country" {
                    other.parent_id.is_none()
                        && other.place_type == "country"
                        && (other.events_count > place.events_count
                            || (other.events_count == place.events_count
                                && other.children_count > place.children_count)
                            || (other.events_count == place.events_count
                                && other.children_count == place.children_count
                                && other.id < place.id))
                } else if other.parent_id.is_none() {
                    place.parent_id.is_some()
                        || other.place_type == "country"
                        || other.events_count > place.events_count
                        || (other.events_count == place.events_count && other.id < place.id)
                } else {
                    false
                }
            } else if place.parent_id == other.parent_id {
                // Prefer keeping the one with clean name (no abbreviation), or more events, or lower id
                let other_is_cleaner = other.name == infer_place_type_and_clean_name(&other.name).0
                    && place.name != clean_name;
                let other_has_more_events = other.events_count > place.events_count;
                other_is_cleaner
                    || other_has_more_events
                    || (other.events_count == place.events_count && other.id < place.id)
            } else {
                // If `place` is an orphan (`parent_id == None` or placeholder parent) and `other` already has a valid parent hierarchy
                (place.parent_id.is_none() || parent_is_placeholder)
                    && other.parent_id.is_some()
                    && !other_parent_is_placeholder
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
                let (_, dup_inferred_opt) = infer_place_type_and_clean_name(&dup.name);
                let dup_eff_type = if dup_inferred_opt.as_deref() == Some("country") {
                    "country".to_string()
                } else if dup.place_type == "other" {
                    dup_inferred_opt.unwrap_or_else(|| dup.place_type.clone())
                } else {
                    dup.place_type.clone()
                };
                let dup_parent_id = if dup_eff_type == "country" {
                    None
                } else {
                    dup.parent_id.clone()
                };
                let mut chain = if let Some(ref pid) = dup_parent_id {
                    build_ancestor_steps_for_parent(&store.places, pid)
                } else {
                    vec![]
                };
                chain.push(ProposedHierarchyStepDto {
                    name: dup.name.clone(),
                    place_type: dup_eff_type.clone(),
                    existing_place_id: Some(dup.id.clone()),
                    existing_place_gid: Some(dup.gid.clone()),
                    is_new: false,
                    action: "consolidate".to_string(),
                });
                let dup_parent_name = dup_parent_id
                    .as_deref()
                    .and_then(|pid| store.places.iter().find(|p| p.id == pid))
                    .filter(|p| !is_placeholder_place_name(&p.name))
                    .map(|p| p.full_name.clone());
                return Some(build_place_issue_dto(
                    issue_id,
                    place,
                    "duplicate_node",
                    RULE_PLACE_DUPLICATE_CANDIDATE,
                    "warning",
                    88,
                    effective_current_parent_id,
                    parent_name,
                    dup.name.clone(),
                    dup_eff_type,
                    dup_parent_id,
                    dup_parent_name,
                    dup.full_name.clone(),
                    chain,
                    vec![],
                    Some(dup.id.clone()),
                    Some(dup.full_name.clone()),
                    None,
                    format!(
                        "Duplicate place record matching structured node '{}' ({}).",
                        dup.full_name, dup.gid
                    ),
                    if place.name != dup.name {
                        Some(place.name.clone())
                    } else {
                        None
                    },
                ));
            }
        }
    }

    // --- Category 3: Orphan Hierarchy (`PLACE_ORPHAN_HIERARCHY`) ---
    let my_rank = place_hierarchy_rank(if effective_type == "other" {
        "city"
    } else {
        &effective_type
    });
    if (place.parent_id.is_none() || parent_is_placeholder) && effective_type != "country" {
        let my_stem = extract_jurisdiction_stem(&clean_name);
        let mut candidates: Vec<ParentPlaceCandidateDto> = Vec::new();

        for cand in &store.places {
            if cand.id == place.id
                || is_placeholder_place_name(&cand.name)
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
                candidates.push(make_parent_candidate(
                    cand,
                    cand_eff_type,
                    d_rounded,
                    92,
                    format!(
                        "Administrative stem and coordinate proximity match '{}'",
                        cand.full_name
                    ),
                ));
            } else if stem_matches {
                candidates.push(make_parent_candidate(
                    cand,
                    cand_eff_type,
                    min_dist_km.map(|d| (d * 10.0).round() / 10.0),
                    88,
                    format!(
                        "Administrative stem matches parent jurisdiction '{}'",
                        cand.full_name
                    ),
                ));
            } else if spatial_matches {
                let d_rounded = min_dist_km.map(|d| (d * 10.0).round() / 10.0);
                let conf = if cand_eff_type == "district" || inside_bounding_box {
                    88
                } else {
                    85
                };
                let match_reason = if let Some(d) = d_rounded {
                    format!(
                        "Coordinates within {:.1} km of parent {} '{}'",
                        d, cand.place_type, cand.full_name
                    )
                } else {
                    format!(
                        "Coordinates inside child cluster envelope of '{}'",
                        cand.full_name
                    )
                };
                candidates.push(make_parent_candidate(
                    cand,
                    cand_eff_type,
                    d_rounded,
                    conf,
                    match_reason,
                ));
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
            candidates[0].score = 75;
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
                    action: "clean_leaf".to_string(),
                });
                let proposed_full_name = format!("{}, {}", clean_name, best.full_name);

                return Some(build_place_issue_dto(
                    issue_id,
                    place,
                    "orphan_hierarchy",
                    RULE_PLACE_ORPHAN_HIERARCHY,
                    "warning",
                    best.confidence,
                    None,
                    None,
                    clean_name.clone(),
                    leaf_type,
                    Some(best.place_id),
                    Some(best.full_name),
                    proposed_full_name,
                    chain,
                    candidates,
                    None,
                    None,
                    best.distance_km,
                    best.match_reason,
                    if place.name != clean_name {
                        Some(place.name.clone())
                    } else {
                        None
                    },
                ));
            }
        }
    }

    // --- Category 4: Place Type Mismatch or Unstripped Locality Prefix (`PLACE_TYPE_MISMATCH`) ---
    let target_mismatch_type = inferred_type_opt
        .clone()
        .unwrap_or_else(|| effective_type.clone());
    let has_type_or_prefix_mismatch = inferred_type_opt
        .as_ref()
        .is_some_and(|inf| place.place_type != *inf || place.name != clean_name)
        || place.name != clean_name
        || parent_is_placeholder
        || (effective_type == "country" && place.parent_id.is_some());

    if has_type_or_prefix_mismatch {
        let issue_id = format!("{RULE_PLACE_TYPE_MISMATCH}:{}", place.id);
        let dismissed = is_warning_dismissed(
            &store.dismissed_warnings,
            &issue_id,
            RULE_PLACE_TYPE_MISMATCH,
            &place.id,
            None,
        );
        if include_dismissed || !dismissed {
            let detach_parent = target_mismatch_type == "country" || parent_is_placeholder;
            let prop_parent_id = if detach_parent {
                None
            } else {
                place.parent_id.clone()
            };
            let prop_parent_name = if detach_parent {
                None
            } else {
                parent_name.clone()
            };
            let mut chain = if let Some(ref pid) = prop_parent_id {
                build_ancestor_steps_for_parent(&store.places, pid)
            } else {
                vec![]
            };
            chain.push(ProposedHierarchyStepDto {
                name: clean_name.clone(),
                place_type: target_mismatch_type.clone(),
                existing_place_id: Some(place.id.clone()),
                existing_place_gid: Some(place.gid.clone()),
                is_new: false,
                action: "clean_leaf".to_string(),
            });
            let proposed_full_name = if let Some(ref pname) = prop_parent_name {
                format!("{clean_name}, {pname}")
            } else {
                clean_name.clone()
            };
            return Some(build_place_issue_dto(
                issue_id,
                place,
                "type_mismatch",
                RULE_PLACE_TYPE_MISMATCH,
                "info",
                90,
                effective_current_parent_id,
                parent_name,
                clean_name.clone(),
                target_mismatch_type.clone(),
                prop_parent_id,
                prop_parent_name,
                proposed_full_name,
                chain,
                vec![],
                None,
                None,
                None,
                format!(
                    "Administrative keyword indicates place_type '{}' (currently '{}').",
                    target_mismatch_type, place.place_type
                ),
                if place.name != clean_name {
                    Some(place.name.clone())
                } else {
                    None
                },
            ));
        }
    }

    // Fallback Low-Confidence Orphan Locality if parent_id is None and other higher-level places exist in the tree
    if (place.parent_id.is_none() || parent_is_placeholder)
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
            return Some(build_place_issue_dto(
                issue_id,
                place,
                "orphan_hierarchy",
                RULE_PLACE_ORPHAN_HIERARCHY,
                "warning",
                50,
                None,
                None,
                clean_name.clone(),
                leaf_type.clone(),
                None,
                None,
                clean_name.clone(),
                vec![ProposedHierarchyStepDto {
                    name: clean_name,
                    place_type: leaf_type,
                    existing_place_id: Some(place.id.clone()),
                    existing_place_gid: Some(place.gid.clone()),
                    is_new: false,
                    action: "clean_leaf".to_string(),
                }],
                vec![],
                None,
                None,
                None,
                "Locality has no parent jurisdiction assigned.".to_string(),
                None,
            ));
        }
    }

    None
}
