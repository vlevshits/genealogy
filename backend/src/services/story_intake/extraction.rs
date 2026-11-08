#[allow(unused_imports)]
use super::*;

// ============================================================================
// Bilingual (EN + RU) Deterministic Narrative Extraction Engine
// ============================================================================

pub(crate) fn extract_narrative_graph(
    raw_text: &str,
    anchored_speaker: Option<&TreePersonNodeDto>,
) -> DraftExtractedDataDto {
    let mut persons: Vec<DraftPersonDto> = Vec::new();
    let mut unions: Vec<DraftUnionDto> = Vec::new();
    let mut parent_child_links: Vec<DraftParentChildDto> = Vec::new();
    let mut events: Vec<DraftEventDto> = Vec::new();

    let speaker_temp_id = "draft-p-1".to_string();
    let mut speaker_node = if let Some(sp) = anchored_speaker {
        DraftPersonDto {
            temp_id: speaker_temp_id.clone(),
            given_name: sp.primary_name.given_name.clone(),
            patronymic: sp.primary_name.patronymic.clone(),
            surname: sp.primary_name.surname.clone(),
            maiden_name: sp.birth_surname.clone(),
            sex: sp.sex.clone(),
            relation_to_speaker: "self".to_string(),
            birth_date_text: sp.birth.as_ref().and_then(|b| b.date.clone()),
            birth_year: extract_person_birth_year(sp),
            birth_place: sp.birth.as_ref().and_then(|b| b.place.clone()),
            death_date_text: sp.death.as_ref().and_then(|d| d.date.clone()),
            death_year: None,
            death_place: sp.death.as_ref().and_then(|d| d.place.clone()),
            is_living: sp.is_living,
            occupation: None,
            biography_notes: None,
            confidence: 98,
            grounding_quote: None,
        }
    } else {
        DraftPersonDto {
            temp_id: speaker_temp_id.clone(),
            given_name: "Speaker".to_string(),
            patronymic: None,
            surname: String::new(),
            maiden_name: None,
            sex: "U".to_string(),
            relation_to_speaker: "self".to_string(),
            birth_date_text: None,
            birth_year: None,
            birth_place: None,
            death_date_text: None,
            death_year: None,
            death_place: None,
            is_living: true,
            occupation: None,
            biography_notes: None,
            confidence: 90,
            grounding_quote: None,
        }
    };
    persons.push(speaker_node.clone());

    let clauses = split_narrative_into_clauses(raw_text);
    let mut last_subject_idx: usize = 0;

    for clause in clauses {
        let trimmed = clause.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(parsed_rel) = classify_and_extract_clause(trimmed) {
            if parsed_rel.relation == "self" {
                if !parsed_rel.given_name.is_empty() {
                    speaker_node.given_name = parsed_rel.given_name;
                }
                if parsed_rel.patronymic.is_some() {
                    speaker_node.patronymic = parsed_rel.patronymic;
                }
                if !parsed_rel.surname.is_empty() {
                    speaker_node.surname = parsed_rel.surname;
                }
                if parsed_rel.maiden_name.is_some() {
                    speaker_node.maiden_name = parsed_rel.maiden_name;
                }
                if parsed_rel.sex != "U" {
                    speaker_node.sex = parsed_rel.sex;
                }
                if parsed_rel.birth_date_text.is_some() {
                    speaker_node.birth_date_text = parsed_rel.birth_date_text;
                }
                if parsed_rel.birth_year.is_some() {
                    speaker_node.birth_year = parsed_rel.birth_year;
                }
                if parsed_rel.birth_place.is_some() {
                    speaker_node.birth_place = parsed_rel.birth_place;
                }
                speaker_node.grounding_quote = Some(trimmed.to_string());
                persons[0] = speaker_node.clone();
                last_subject_idx = 0;
            } else if parsed_rel.relation == "continuation_vitals" {
                if let Some(target) = persons.get_mut(last_subject_idx) {
                    if parsed_rel.birth_date_text.is_some() {
                        target.birth_date_text = parsed_rel.birth_date_text;
                    }
                    if parsed_rel.birth_year.is_some() {
                        target.birth_year = parsed_rel.birth_year;
                    }
                    if parsed_rel.birth_place.is_some() {
                        target.birth_place = parsed_rel.birth_place;
                    }
                    if parsed_rel.death_date_text.is_some() {
                        target.death_date_text = parsed_rel.death_date_text;
                        target.is_living = false;
                    }
                    if parsed_rel.death_year.is_some() {
                        target.death_year = parsed_rel.death_year;
                        target.is_living = false;
                    }
                }
            } else {
                let next_num = persons.len() + 1;
                let temp_id = format!("draft-p-{next_num}");
                let mut rel_surname = parsed_rel.surname;
                if rel_surname.is_empty()
                    && matches!(
                        parsed_rel.relation.as_str(),
                        "father" | "mother" | "brother" | "sister" | "son" | "daughter"
                    )
                    && !persons[0].surname.is_empty()
                {
                    rel_surname =
                        adjust_russian_surname_gender(&persons[0].surname, &parsed_rel.sex);
                }

                let is_living =
                    parsed_rel.death_year.is_none() && parsed_rel.death_date_text.is_none();
                let new_person = DraftPersonDto {
                    temp_id: temp_id.clone(),
                    given_name: if parsed_rel.given_name.is_empty() {
                        "Unknown".to_string()
                    } else {
                        parsed_rel.given_name
                    },
                    patronymic: parsed_rel.patronymic,
                    surname: rel_surname,
                    maiden_name: parsed_rel.maiden_name,
                    sex: parsed_rel.sex.clone(),
                    relation_to_speaker: parsed_rel.relation.clone(),
                    birth_date_text: parsed_rel.birth_date_text,
                    birth_year: parsed_rel.birth_year,
                    birth_place: parsed_rel.birth_place,
                    death_date_text: parsed_rel.death_date_text,
                    death_year: parsed_rel.death_year,
                    death_place: parsed_rel.death_place,
                    is_living,
                    occupation: None,
                    biography_notes: None,
                    confidence: 92,
                    grounding_quote: Some(trimmed.to_string()),
                };
                persons.push(new_person);
                last_subject_idx = persons.len() - 1;
            }
        }
    }

    if persons[0].sex == "U" {
        let inferred = infer_sex_from_names(
            &persons[0].given_name,
            persons[0].patronymic.as_deref(),
            &persons[0].surname,
        );
        if inferred != "U" {
            persons[0].sex = inferred;
        } else if let Some(partner) = persons.iter().find(|p| p.relation_to_speaker == "partner") {
            if partner.sex == "M" {
                persons[0].sex = "F".to_string();
            } else if partner.sex == "F" {
                persons[0].sex = "M".to_string();
            }
        }
    }

    rebuild_relationships_and_events(&persons, &mut unions, &mut parent_child_links, &mut events);

    DraftExtractedDataDto {
        speaker_temp_id: Some(speaker_temp_id),
        persons,
        unions,
        parent_child_links,
        events,
    }
}

pub(crate) fn rebuild_relationships_and_events(
    persons: &[DraftPersonDto],
    unions: &mut Vec<DraftUnionDto>,
    parent_child_links: &mut Vec<DraftParentChildDto>,
    events: &mut Vec<DraftEventDto>,
) {
    unions.clear();
    parent_child_links.clear();
    events.clear();

    let Some(speaker) = persons.first() else {
        return;
    };
    let speaker_id = speaker.temp_id.clone();

    let father_id = persons
        .iter()
        .find(|p| p.relation_to_speaker == "father")
        .map(|p| p.temp_id.clone());
    let mother_id = persons
        .iter()
        .find(|p| p.relation_to_speaker == "mother")
        .map(|p| p.temp_id.clone());
    let partner_id = persons
        .iter()
        .find(|p| p.relation_to_speaker == "partner")
        .map(|p| p.temp_id.clone());

    let mut speaker_union_id = None;
    if let Some(pid) = &partner_id {
        let uid = format!("draft-u-{}", unions.len() + 1);
        unions.push(DraftUnionDto {
            temp_id: uid.clone(),
            partner1_temp_id: speaker_id.clone(),
            partner2_temp_id: pid.clone(),
            union_type: "partner".to_string(),
            marriage_date_text: None,
            marriage_place: None,
            confidence: 92,
        });
        speaker_union_id = Some(uid);
    }

    let mut parents_union_id = None;
    if let (Some(fid), Some(mid)) = (&father_id, &mother_id) {
        let uid = format!("draft-u-{}", unions.len() + 1);
        unions.push(DraftUnionDto {
            temp_id: uid.clone(),
            partner1_temp_id: fid.clone(),
            partner2_temp_id: mid.clone(),
            union_type: "married".to_string(),
            marriage_date_text: None,
            marriage_place: None,
            confidence: 95,
        });
        parents_union_id = Some(uid);
    }

    for p in persons {
        match p.relation_to_speaker.as_str() {
            "father" => {
                parent_child_links.push(DraftParentChildDto {
                    union_temp_id: parents_union_id.clone(),
                    parent_temp_id: p.temp_id.clone(),
                    child_temp_id: speaker_id.clone(),
                    pedigree: "birth".to_string(),
                });
            }
            "mother" => {
                parent_child_links.push(DraftParentChildDto {
                    union_temp_id: parents_union_id.clone(),
                    parent_temp_id: p.temp_id.clone(),
                    child_temp_id: speaker_id.clone(),
                    pedigree: "birth".to_string(),
                });
            }
            "brother" | "sister" => {
                if let Some(fid) = &father_id {
                    parent_child_links.push(DraftParentChildDto {
                        union_temp_id: parents_union_id.clone(),
                        parent_temp_id: fid.clone(),
                        child_temp_id: p.temp_id.clone(),
                        pedigree: "birth".to_string(),
                    });
                }
                if let Some(mid) = &mother_id {
                    parent_child_links.push(DraftParentChildDto {
                        union_temp_id: parents_union_id.clone(),
                        parent_temp_id: mid.clone(),
                        child_temp_id: p.temp_id.clone(),
                        pedigree: "birth".to_string(),
                    });
                }
            }
            "son" | "daughter" => {
                parent_child_links.push(DraftParentChildDto {
                    union_temp_id: speaker_union_id.clone(),
                    parent_temp_id: speaker_id.clone(),
                    child_temp_id: p.temp_id.clone(),
                    pedigree: "birth".to_string(),
                });
                if let Some(pid) = &partner_id {
                    parent_child_links.push(DraftParentChildDto {
                        union_temp_id: speaker_union_id.clone(),
                        parent_temp_id: pid.clone(),
                        child_temp_id: p.temp_id.clone(),
                        pedigree: "birth".to_string(),
                    });
                }
            }
            "maternal_grandfather" | "maternal_grandmother" => {
                if let Some(mid) = &mother_id {
                    parent_child_links.push(DraftParentChildDto {
                        union_temp_id: None,
                        parent_temp_id: p.temp_id.clone(),
                        child_temp_id: mid.clone(),
                        pedigree: "birth".to_string(),
                    });
                }
            }
            "paternal_grandfather" | "paternal_grandmother" | "grandfather" | "grandmother" => {
                if let Some(target_parent) = father_id.as_ref().or(mother_id.as_ref()) {
                    parent_child_links.push(DraftParentChildDto {
                        union_temp_id: None,
                        parent_temp_id: p.temp_id.clone(),
                        child_temp_id: target_parent.clone(),
                        pedigree: "birth".to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    for p in persons {
        let b_date = p
            .birth_date_text
            .clone()
            .or_else(|| p.birth_year.map(|y| y.to_string()));
        if b_date.is_some() || p.birth_place.is_some() {
            events.push(DraftEventDto {
                temp_id: format!("draft-e-{}", events.len() + 1),
                person_temp_id: p.temp_id.clone(),
                event_type: "birth".to_string(),
                date_text: b_date,
                place_text: p.birth_place.clone(),
                description: None,
                grounding_quote: p.grounding_quote.clone(),
            });
        }
        let d_date = p
            .death_date_text
            .clone()
            .or_else(|| p.death_year.map(|y| y.to_string()));
        if d_date.is_some() || p.death_place.is_some() {
            events.push(DraftEventDto {
                temp_id: format!("draft-e-{}", events.len() + 1),
                person_temp_id: p.temp_id.clone(),
                event_type: "death".to_string(),
                date_text: d_date,
                place_text: p.death_place.clone(),
                description: None,
                grounding_quote: p.grounding_quote.clone(),
            });
        }
    }
}

pub(crate) fn apply_clarification_to_extracted_data(
    extracted: &mut DraftExtractedDataDto,
    clarification: &str,
) {
    let sub_clauses = split_clarification_clauses(clarification);
    for raw_sub in sub_clauses {
        let sub = raw_sub.trim();
        if sub.is_empty() {
            continue;
        }
        let lower = sub.to_lowercase();

        let surname_corr = extract_surname_correction(sub, &lower);
        let role_target = detect_relationship_role(&lower);
        let (b_date, b_year) = extract_birth_date_and_year(sub);
        let b_place = extract_place_from_clause(sub);
        let (d_date, d_year) = extract_death_date_and_year(sub);

        let existing_idx = if let Some((role, _)) = &role_target {
            extracted
                .persons
                .iter()
                .position(|p| &p.relation_to_speaker == role)
        } else if let Some((target_rel, _)) = &surname_corr {
            extracted
                .persons
                .iter()
                .position(|p| &p.relation_to_speaker == target_rel)
        } else {
            extracted.persons.iter().position(|p| {
                let gn = p.given_name.to_lowercase();
                gn.len() >= 3 && lower.contains(&gn)
            })
        };

        if let Some(idx) = existing_idx {
            let person = &mut extracted.persons[idx];
            if b_date.is_some() {
                person.birth_date_text = b_date;
            }
            if b_year.is_some() {
                person.birth_year = b_year;
            }
            if b_place.is_some() {
                person.birth_place = b_place;
            }
            if d_date.is_some() {
                person.death_date_text = d_date;
                person.is_living = false;
            }
            if d_year.is_some() {
                person.death_year = d_year;
                person.is_living = false;
            }
            if let Some((_, new_surname)) = extract_surname_correction(sub, &lower) {
                person.surname = new_surname;
            }
            continue;
        }

        if let Some(parsed) = classify_and_extract_clause(sub) {
            if parsed.relation != "self" && parsed.relation != "continuation_vitals" {
                let next_num = extracted.persons.len() + 1;
                let temp_id = format!("draft-p-{next_num}");
                let is_living = parsed.death_year.is_none() && parsed.death_date_text.is_none();
                extracted.persons.push(DraftPersonDto {
                    temp_id,
                    given_name: if parsed.given_name.is_empty() {
                        "Unknown".to_string()
                    } else {
                        parsed.given_name
                    },
                    patronymic: parsed.patronymic,
                    surname: parsed.surname,
                    maiden_name: parsed.maiden_name,
                    sex: parsed.sex,
                    relation_to_speaker: parsed.relation,
                    birth_date_text: parsed.birth_date_text,
                    birth_year: parsed.birth_year,
                    birth_place: parsed.birth_place,
                    death_date_text: parsed.death_date_text,
                    death_year: parsed.death_year,
                    death_place: parsed.death_place,
                    is_living,
                    occupation: None,
                    biography_notes: None,
                    confidence: 90,
                    grounding_quote: Some(sub.to_string()),
                });
            }
        }
    }

    let persons_snapshot = extracted.persons.clone();
    rebuild_relationships_and_events(
        &persons_snapshot,
        &mut extracted.unions,
        &mut extracted.parent_child_links,
        &mut extracted.events,
    );
}

pub(crate) fn split_clarification_clauses(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for sentence in text.split(['.', '!', '?', ';', '\n']) {
        let s = sentence.trim();
        if s.is_empty() {
            continue;
        }
        for part in split_on_conjunctions(s) {
            let p = part.trim();
            if !p.is_empty() {
                out.push(p.to_string());
            }
        }
    }
    out
}

pub(crate) fn split_on_conjunctions(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let tok_lower = tokens[i].to_lowercase();
        let next_lower = tokens
            .get(i + 1)
            .map(|t| t.to_lowercase())
            .unwrap_or_default();
        if (tok_lower == "and" || tok_lower == "а" || tok_lower == "и")
            && matches!(
                next_lower.as_str(),
                "my" | "мой" | "моя" | "у" | "her" | "his" | "её" | "ее" | "его"
            )
            && !current.trim().is_empty()
        {
            parts.push(current.trim().trim_end_matches(',').to_string());
            current.clear();
            i += 1;
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(tokens[i]);
        i += 1;
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().trim_end_matches(',').to_string());
    }
    parts
}

pub(crate) fn extract_surname_correction(clause: &str, lower: &str) -> Option<(String, String)> {
    let rel = if lower.contains("boyfriend")
        || lower.contains("bf")
        || lower.contains("partner")
        || lower.contains("husband")
        || lower.contains("wife")
        || lower.contains("парня")
        || lower.contains("мужа")
        || lower.contains("жены")
        || lower.contains("девушки")
    {
        "partner"
    } else if lower.contains("father")
        || lower.contains("dad")
        || lower.contains("папы")
        || lower.contains("отца")
    {
        "father"
    } else if lower.contains("mother")
        || lower.contains("mom")
        || lower.contains("мамы")
        || lower.contains("матери")
    {
        "mother"
    } else {
        return None;
    };

    if let Some(pos) = lower.find("surname is ") {
        let after = clause[pos + "surname is ".len()..].trim();
        let word = after
            .split(|c: char| c.is_whitespace() || c == ',' || c == '.')
            .next()
            .unwrap_or("")
            .trim();
        if !word.is_empty() {
            return Some((rel.to_string(), word.to_string()));
        }
    }
    if let Some(pos) = lower.find("last name is ") {
        let after = clause[pos + "last name is ".len()..].trim();
        let word = after
            .split(|c: char| c.is_whitespace() || c == ',' || c == '.')
            .next()
            .unwrap_or("")
            .trim();
        if !word.is_empty() {
            return Some((rel.to_string(), word.to_string()));
        }
    }
    if lower.contains("фамилия") {
        let last_word = clause
            .split(|c: char| c.is_whitespace() || c == ',' || c == '.')
            .rfind(|w| !w.is_empty())
            .unwrap_or("");
        if !last_word.is_empty() && last_word.chars().next().is_some_and(char::is_uppercase) {
            return Some((rel.to_string(), last_word.to_string()));
        }
    }

    None
}
