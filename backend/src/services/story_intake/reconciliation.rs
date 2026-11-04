#[allow(unused_imports)]
use super::*;

pub(crate) fn apply_match_overrides(
    store: &GenealogyPrivacyStore,
    draft: &mut StoryIntakeDraftDto,
    overrides: &[DraftMatchOverrideDto],
) {
    for ov in overrides {
        if let Some(m) = draft
            .reconciliation_matches
            .iter_mut()
            .find(|m| m.draft_temp_id == ov.draft_temp_id)
        {
            m.selected_action = ov.selected_action.clone();
            if let Some(pid) = &ov.matched_person_id {
                let resolved = store
                    .resolve_person_id(pid)
                    .unwrap_or_else(|| pid.to_string());
                if let Some(p) = store.persons.iter().find(|p| p.id == resolved) {
                    m.matched_person_id = Some(p.id.clone());
                    m.matched_person_gid = Some(p.gid.clone());
                    m.matched_person_name = Some(format_person_display_name(p));
                } else {
                    m.matched_person_id = Some(resolved);
                }
            } else if ov.selected_action == "create_new" || ov.selected_action == "skip" {
                m.matched_person_id = None;
                m.matched_person_gid = None;
                m.matched_person_name = None;
            }
        }
    }
}

/// Runs relationship-centric candidate reconciliation against `store.persons` (`19-ai-family-story-intake-and-mcp.md §3.3`).
pub fn reconcile_story_draft_in_place(
    store: &GenealogyPrivacyStore,
    draft: &mut StoryIntakeDraftDto,
) {
    let mut resolved_temp_to_person_id: HashMap<String, String> = HashMap::new();
    let mut matches_by_temp_id: HashMap<String, DraftReconciliationMatchDto> = HashMap::new();

    // Pass 1: Anchor speaker if speaker_person_id is provided and !is_new_speaker
    if !draft.is_new_speaker {
        if let Some(speaker_pid) = &draft.speaker_person_id {
            if let Some(resolved_id) = store.resolve_person_id(speaker_pid) {
                if let Some(speaker_person) = store.persons.iter().find(|p| p.id == resolved_id) {
                    if let Some(speaker_draft) = draft
                        .extracted_data
                        .persons
                        .iter()
                        .find(|dp| dp.relation_to_speaker == "self")
                    {
                        let display_name = format_person_display_name(speaker_person);
                        let candidate = DraftMatchCandidateDto {
                            person_id: speaker_person.id.clone(),
                            person_gid: speaker_person.gid.clone(),
                            display_name: display_name.clone(),
                            birth_year: extract_person_birth_year(speaker_person),
                            birth_place: speaker_person
                                .birth
                                .as_ref()
                                .and_then(|b| b.place.clone()),
                            confidence: 100,
                            reasons: vec!["anchored_speaker".to_string()],
                        };
                        resolved_temp_to_person_id
                            .insert(speaker_draft.temp_id.clone(), speaker_person.id.clone());
                        matches_by_temp_id.insert(
                            speaker_draft.temp_id.clone(),
                            DraftReconciliationMatchDto {
                                draft_temp_id: speaker_draft.temp_id.clone(),
                                recommended_action: "link_enrich".to_string(),
                                selected_action: "link_enrich".to_string(),
                                matched_person_id: Some(speaker_person.id.clone()),
                                matched_person_gid: Some(speaker_person.gid.clone()),
                                matched_person_name: Some(display_name),
                                confidence: 100,
                                reasons: vec!["anchored_speaker".to_string()],
                                candidates: vec![candidate],
                            },
                        );
                    }
                }
            }
        }
    }

    // Pass 2 & 3: Iteratively match remaining draft persons (up to 3 hops from anchored nodes)
    for _hop in 0..3 {
        let mut any_newly_matched = false;
        for dp in &draft.extracted_data.persons {
            if matches_by_temp_id.contains_key(&dp.temp_id) {
                continue;
            }

            let mut candidates = score_candidates_for_draft_person(
                store,
                &draft.extracted_data,
                dp,
                &resolved_temp_to_person_id,
            );
            candidates.sort_by_key(|b| std::cmp::Reverse(b.confidence));
            candidates.truncate(5);

            if let Some(best) = candidates.first() {
                if best.confidence >= 80 {
                    resolved_temp_to_person_id.insert(dp.temp_id.clone(), best.person_id.clone());
                    matches_by_temp_id.insert(
                        dp.temp_id.clone(),
                        DraftReconciliationMatchDto {
                            draft_temp_id: dp.temp_id.clone(),
                            recommended_action: "link_enrich".to_string(),
                            selected_action: "link_enrich".to_string(),
                            matched_person_id: Some(best.person_id.clone()),
                            matched_person_gid: Some(best.person_gid.clone()),
                            matched_person_name: Some(best.display_name.clone()),
                            confidence: best.confidence,
                            reasons: best.reasons.clone(),
                            candidates,
                        },
                    );
                    any_newly_matched = true;
                }
            }
        }
        if !any_newly_matched {
            break;
        }
    }

    // Final Pass: Populate any remaining unmatched draft persons as "create_new"
    let mut final_matches = Vec::with_capacity(draft.extracted_data.persons.len());
    for dp in &draft.extracted_data.persons {
        if let Some(existing) = matches_by_temp_id.remove(&dp.temp_id) {
            final_matches.push(existing);
        } else {
            let mut candidates = score_candidates_for_draft_person(
                store,
                &draft.extracted_data,
                dp,
                &resolved_temp_to_person_id,
            );
            candidates.sort_by_key(|b| std::cmp::Reverse(b.confidence));
            candidates.truncate(5);

            let (top_conf, top_reasons) = candidates
                .first()
                .map(|c| (c.confidence, c.reasons.clone()))
                .unwrap_or((0, vec!["net_new_relative".to_string()]));

            final_matches.push(DraftReconciliationMatchDto {
                draft_temp_id: dp.temp_id.clone(),
                recommended_action: "create_new".to_string(),
                selected_action: "create_new".to_string(),
                matched_person_id: None,
                matched_person_gid: None,
                matched_person_name: None,
                confidence: top_conf,
                reasons: top_reasons,
                candidates,
            });
        }
    }

    draft.reconciliation_matches = final_matches;
}

pub(crate) fn score_candidates_for_draft_person(
    store: &GenealogyPrivacyStore,
    extracted: &DraftExtractedDataDto,
    dp: &DraftPersonDto,
    resolved_temp_to_person_id: &HashMap<String, String>,
) -> Vec<DraftMatchCandidateDto> {
    let mut structural_slot_bonuses: HashMap<String, (u8, String)> = HashMap::new();

    // 1. Check parent_child_links where dp is a parent of a matched child
    for link in &extracted.parent_child_links {
        if link.parent_temp_id == dp.temp_id {
            if let Some(matched_child_id) = resolved_temp_to_person_id.get(&link.child_temp_id) {
                for fam in &store.families {
                    if fam.children.iter().any(|c| &c.child_id == matched_child_id) {
                        for pid in fam.partner1_id.iter().chain(fam.partner2_id.iter()) {
                            structural_slot_bonuses
                                .insert(pid.clone(), (50, "linked_parent_slot".to_string()));
                        }
                    }
                }
            }
        }
        // Check where dp is a child of a matched parent
        if link.child_temp_id == dp.temp_id {
            if let Some(matched_parent_id) = resolved_temp_to_person_id.get(&link.parent_temp_id) {
                for fam in &store.families {
                    if fam.partner1_id.as_deref() == Some(matched_parent_id.as_str())
                        || fam.partner2_id.as_deref() == Some(matched_parent_id.as_str())
                    {
                        for c in &fam.children {
                            structural_slot_bonuses
                                .insert(c.child_id.clone(), (45, "linked_child_slot".to_string()));
                        }
                    }
                }
            }
        }
    }

    // 2. Check unions where dp is a partner of a matched person
    for u in &extracted.unions {
        let other_temp = if u.partner1_temp_id == dp.temp_id {
            Some(&u.partner2_temp_id)
        } else if u.partner2_temp_id == dp.temp_id {
            Some(&u.partner1_temp_id)
        } else {
            None
        };
        if let Some(other_tid) = other_temp {
            if let Some(matched_partner_id) = resolved_temp_to_person_id.get(other_tid) {
                for fam in &store.families {
                    if fam.partner1_id.as_deref() == Some(matched_partner_id.as_str()) {
                        if let Some(p2) = &fam.partner2_id {
                            structural_slot_bonuses
                                .insert(p2.clone(), (50, "linked_spouse_slot".to_string()));
                        }
                    } else if fam.partner2_id.as_deref() == Some(matched_partner_id.as_str()) {
                        if let Some(p1) = &fam.partner1_id {
                            structural_slot_bonuses
                                .insert(p1.clone(), (50, "linked_spouse_slot".to_string()));
                        }
                    }
                }
            }
        }
    }

    // 3. Check siblings if dp shares a parent with a matched sibling
    if matches!(dp.relation_to_speaker.as_str(), "brother" | "sister") {
        if let Some(speaker_temp) = &extracted.speaker_temp_id {
            if let Some(matched_speaker_id) = resolved_temp_to_person_id.get(speaker_temp) {
                for fam in &store.families {
                    if fam
                        .children
                        .iter()
                        .any(|c| &c.child_id == matched_speaker_id)
                    {
                        for c in &fam.children {
                            if &c.child_id != matched_speaker_id {
                                structural_slot_bonuses.insert(
                                    c.child_id.clone(),
                                    (45, "linked_sibling_slot".to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    let already_claimed_ids: std::collections::HashSet<&String> =
        resolved_temp_to_person_id.values().collect();

    let mut results = Vec::new();
    for person in &store.persons {
        if already_claimed_ids.contains(&person.id) {
            continue;
        }
        if matches!(dp.sex.as_str(), "M" | "F")
            && matches!(person.sex.as_str(), "M" | "F")
            && dp.sex != person.sex
        {
            continue;
        }

        let mut score: u32 = 0;
        let mut reasons: Vec<String> = Vec::new();

        if let Some((slot_bonus, slot_reason)) = structural_slot_bonuses.get(&person.id) {
            score += u32::from(*slot_bonus);
            reasons.push(slot_reason.clone());
        }

        if names_align(&dp.given_name, &person.primary_name.given_name) {
            score += 35;
            reasons.push("same_given_name".to_string());
        }

        let dp_surnames: Vec<&str> = [dp.surname.as_str(), dp.maiden_name.as_deref().unwrap_or("")]
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect();
        let person_surnames: Vec<&str> = [
            person.primary_name.surname.as_str(),
            person.birth_surname.as_deref().unwrap_or(""),
        ]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect();

        let surname_matched = dp_surnames
            .iter()
            .any(|ds| person_surnames.iter().any(|ps| surnames_align(ds, ps)));
        if surname_matched {
            score += 25;
            reasons.push("same_surname".to_string());
        }

        if let (Some(dp_pat), Some(p_pat)) = (&dp.patronymic, &person.primary_name.patronymic) {
            if names_align(dp_pat, p_pat) {
                score += 12;
                reasons.push("same_patronymic".to_string());
            }
        }

        let person_by = extract_person_birth_year(person);
        if let (Some(d_by), Some(p_by)) = (dp.birth_year, person_by) {
            let diff = (d_by - p_by).abs();
            if diff == 0 {
                score += 18;
                reasons.push("same_birth_year".to_string());
            } else if diff <= 2 {
                score += 12;
                reasons.push("close_birth_year".to_string());
            } else if diff > 15 {
                score = score.saturating_sub(40);
            }
        }

        if let (Some(d_bp), Some(p_bp)) = (
            dp.birth_place.as_deref(),
            person.birth.as_ref().and_then(|b| b.place.as_deref()),
        ) {
            if places_align(d_bp, p_bp) {
                score += 10;
                reasons.push("same_birth_place".to_string());
            }
        }

        let capped = score.min(99) as u8;
        if capped >= 40 {
            results.push(DraftMatchCandidateDto {
                person_id: person.id.clone(),
                person_gid: person.gid.clone(),
                display_name: format_person_display_name(person),
                birth_year: person_by,
                birth_place: person.birth.as_ref().and_then(|b| b.place.clone()),
                confidence: capped,
                reasons,
            });
        }
    }

    results
}
