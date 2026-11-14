#[allow(unused_imports)]
use super::*;

/// Evaluates all 10 soft data-quality rules (`docs/features/15-data-quality-duplicates.md §2.1`).
pub fn compute_quality_warnings(
    store: &GenealogyPrivacyStore,
    include_dismissed: bool,
) -> Vec<QualityWarningDto> {
    let mut warnings = Vec::new();
    let current_year = Utc::now().year();
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();

    // Precompute connected persons (any parent, partner, or child link in `families`)
    let mut connected_ids: HashSet<&str> = HashSet::new();
    for fam in &store.families {
        let has_multiple_members = (fam.partner1_id.is_some() as usize
            + fam.partner2_id.is_some() as usize
            + fam.children.len())
            > 1;
        if has_multiple_members {
            if let Some(p1) = fam.partner1_id.as_deref() {
                connected_ids.insert(p1);
            }
            if let Some(p2) = fam.partner2_id.as_deref() {
                connected_ids.insert(p2);
            }
            for ch in &fam.children {
                connected_ids.insert(ch.child_id.as_str());
            }
        }
    }

    for person in &store.persons {
        let b_win = person_birth_window(person, &store.events);
        let d_win = person_death_window(person, &store.events);

        // Rule 1: DEATH_BEFORE_BIRTH (Death date < Birth date, using widest plausible window)
        if let (Some(b), Some(d)) = (b_win, d_win) {
            if d.max_sort < b.min_sort {
                let wid = format!("{RULE_DEATH_BEFORE_BIRTH}:{}", person.id);
                let dismissed = is_warning_dismissed(
                    &store.dismissed_warnings,
                    &wid,
                    RULE_DEATH_BEFORE_BIRTH,
                    &person.id,
                    None,
                );
                if include_dismissed || !dismissed {
                    warnings.push(QualityWarningDto {
                        id: wid,
                        code: RULE_DEATH_BEFORE_BIRTH.to_string(),
                        rule_code: RULE_DEATH_BEFORE_BIRTH.to_string(),
                        category: "chronology".to_string(),
                        severity: "critical".to_string(),
                        entity_type: "individual".to_string(),
                        entity_id: person.id.clone(),
                        person: to_warning_person_ref(person),
                        related_person: None,
                        message: format!(
                            "Death date precedes birth date for {}",
                            format_person_display_name(person)
                        ),
                        details: serde_json::json!({
                            "birth_sort": b.min_sort,
                            "death_sort": d.max_sort,
                        }),
                        is_dismissed: dismissed,
                    });
                }
            }
        }

        // Rule 2: TOO_OLD (Lifespan > 110 years, or living person with birth year > 110 years ago)
        if let Some(b) = b_win {
            let min_age_at_death_or_now = if let Some(d) = d_win {
                Some(d.min_year - b.max_year)
            } else if person.is_living {
                Some(current_year - b.max_year)
            } else {
                None
            };
            if let Some(age) = min_age_at_death_or_now {
                if age > 110 {
                    let wid = format!("{RULE_TOO_OLD}:{}", person.id);
                    let dismissed = is_warning_dismissed(
                        &store.dismissed_warnings,
                        &wid,
                        RULE_TOO_OLD,
                        &person.id,
                        None,
                    );
                    if include_dismissed || !dismissed {
                        warnings.push(QualityWarningDto {
                            id: wid,
                            code: RULE_TOO_OLD.to_string(),
                            rule_code: RULE_TOO_OLD.to_string(),
                            category: "chronology".to_string(),
                            severity: "warning".to_string(),
                            entity_type: "individual".to_string(),
                            entity_id: person.id.clone(),
                            person: to_warning_person_ref(person),
                            related_person: None,
                            message: format!(
                                "Recorded age ({age} years) exceeds 110 years for {}",
                                format_person_display_name(person)
                            ),
                            details: serde_json::json!({
                                "age": age,
                                "threshold": 110,
                            }),
                            is_dismissed: dismissed,
                        });
                    }
                }
            }
        }

        // Rule 7: EVENT_AFTER_DEATH (Personal life event > 30 days after death, except burial/cremation/probate/will)
        if let Some(d) = d_win {
            for ev in store
                .events
                .iter()
                .filter(|e| e.individual_id.as_deref() == Some(person.id.as_str()))
            {
                let ev_type = ev.event_type.trim().to_ascii_lowercase();
                if matches!(
                    ev_type.as_str(),
                    "death" | "burial" | "cremation" | "probate" | "will" | "memorial"
                ) {
                    continue;
                }
                if let Some(ev_win) = parse_plausible_window(event_date_text(ev), ev.date_sort) {
                    // > 30 days (~100 in YYYYMMDD sort key or > 0 years if year-only)
                    let threshold_sort = if d.max_sort % 10_000 == 0 {
                        (d.max_year + 1) * 10_000
                    } else {
                        d.max_sort + 100
                    };
                    if ev_win.min_sort > threshold_sort {
                        let wid = format!("{RULE_EVENT_AFTER_DEATH}:{}:{}", person.id, ev.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_EVENT_AFTER_DEATH,
                            &person.id,
                            Some(&ev.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_EVENT_AFTER_DEATH.to_string(),
                                rule_code: RULE_EVENT_AFTER_DEATH.to_string(),
                                category: "chronology".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: person.id.clone(),
                                person: to_warning_person_ref(person),
                                related_person: None,
                                message: format!(
                                    "Event '{}' occurs after death date for {}",
                                    ev.event_type,
                                    format_person_display_name(person)
                                ),
                                details: serde_json::json!({
                                    "event_id": ev.id,
                                    "event_type": ev.event_type,
                                    "event_date": event_date_text(ev),
                                    "death_sort": d.max_sort,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }
            }
        }

        // Rule 9: MISSING_BIRTH (No birth date on record)
        let has_birth_event = store.events.iter().any(|e| {
            e.individual_id.as_deref() == Some(person.id.as_str())
                && e.event_type.eq_ignore_ascii_case("birth")
                && (event_date_text(e)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
                    || e.date_sort.is_some())
        });
        let has_birth_summary = person
            .birth
            .as_ref()
            .and_then(|b| b.date.as_deref())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        if !has_birth_event && !has_birth_summary && person.birth_sort.is_none() {
            let wid = format!("{RULE_MISSING_BIRTH}:{}", person.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &wid,
                RULE_MISSING_BIRTH,
                &person.id,
                None,
            );
            if include_dismissed || !dismissed {
                warnings.push(QualityWarningDto {
                    id: wid,
                    code: RULE_MISSING_BIRTH.to_string(),
                    rule_code: RULE_MISSING_BIRTH.to_string(),
                    category: "completeness".to_string(),
                    severity: "info".to_string(),
                    entity_type: "individual".to_string(),
                    entity_id: person.id.clone(),
                    person: to_warning_person_ref(person),
                    related_person: None,
                    message: format!(
                        "{} is missing a birth date",
                        format_person_display_name(person)
                    ),
                    details: serde_json::json!({}),
                    is_dismissed: dismissed,
                });
            }
        }

        // Rule 10: UNCONNECTED (0 relatives in a tree with > 1 person)
        if store.persons.len() > 1 && !connected_ids.contains(person.id.as_str()) {
            let wid = format!("{RULE_UNCONNECTED}:{}", person.id);
            let dismissed = is_warning_dismissed(
                &store.dismissed_warnings,
                &wid,
                RULE_UNCONNECTED,
                &person.id,
                None,
            );
            if include_dismissed || !dismissed {
                warnings.push(QualityWarningDto {
                    id: wid,
                    code: RULE_UNCONNECTED.to_string(),
                    rule_code: RULE_UNCONNECTED.to_string(),
                    category: "completeness".to_string(),
                    severity: "info".to_string(),
                    entity_type: "individual".to_string(),
                    entity_id: person.id.clone(),
                    person: to_warning_person_ref(person),
                    related_person: None,
                    message: format!(
                        "{} has no connected family relationships",
                        format_person_display_name(person)
                    ),
                    details: serde_json::json!({}),
                    is_dismissed: dismissed,
                });
            }
        }
    }

    // Family-based rules: Rules 3, 4, 5, 6
    for fam in &store.families {
        let partners: Vec<&TreePersonNodeDto> =
            [fam.partner1_id.as_deref(), fam.partner2_id.as_deref()]
                .into_iter()
                .flatten()
                .filter_map(|pid| person_by_id.get(pid).copied())
                .collect();

        // Rule 6: MARRIED_TOO_YOUNG (< 14 years old at marriage)
        let marriage_ev = store.events.iter().find(|e| {
            e.family_id.as_deref() == Some(fam.id.as_str())
                && e.event_type.eq_ignore_ascii_case("marriage")
        });
        let marriage_win =
            marriage_ev.and_then(|ev| parse_plausible_window(event_date_text(ev), ev.date_sort));

        if let Some(m_win) = marriage_win {
            for spouse in &partners {
                if let Some(s_birth) = person_birth_window(spouse, &store.events) {
                    let max_possible_age = m_win.max_year - s_birth.min_year;
                    if max_possible_age < 14 {
                        let wid = format!("{RULE_MARRIED_TOO_YOUNG}:{}:{}", spouse.id, fam.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_MARRIED_TOO_YOUNG,
                            &spouse.id,
                            Some(&fam.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_MARRIED_TOO_YOUNG.to_string(),
                                rule_code: RULE_MARRIED_TOO_YOUNG.to_string(),
                                category: "family_age".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: spouse.id.clone(),
                                person: to_warning_person_ref(spouse),
                                related_person: None,
                                message: format!(
                                    "{} was {} years old at marriage (< 14)",
                                    format_person_display_name(spouse),
                                    max_possible_age
                                ),
                                details: serde_json::json!({
                                    "family_id": fam.id,
                                    "age_at_marriage": max_possible_age,
                                    "min_age_threshold": 14,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }
            }
        }

        // Parent-child checks: Rules 3, 4, 5
        for ch_link in &fam.children {
            let Some(child) = person_by_id.get(ch_link.child_id.as_str()).copied() else {
                continue;
            };
            let Some(c_birth) = person_birth_window(child, &store.events) else {
                continue;
            };

            for parent in &partners {
                if let Some(p_birth) = person_birth_window(parent, &store.events) {
                    let max_parent_age_at_birth = c_birth.max_year - p_birth.min_year;
                    let min_parent_age_at_birth = c_birth.min_year - p_birth.max_year;

                    // Rule 3: PARENT_TOO_YOUNG (Parent < 13 years old at child's birth)
                    if max_parent_age_at_birth < 13 {
                        let wid = format!("{RULE_PARENT_TOO_YOUNG}:{}:{}", parent.id, child.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_PARENT_TOO_YOUNG,
                            &parent.id,
                            Some(&child.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_PARENT_TOO_YOUNG.to_string(),
                                rule_code: RULE_PARENT_TOO_YOUNG.to_string(),
                                category: "family_age".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: parent.id.clone(),
                                person: to_warning_person_ref(parent),
                                related_person: Some(to_warning_person_ref(child)),
                                message: format!(
                                    "{} was {} years old when {} was born (< 13)",
                                    format_person_display_name(parent),
                                    max_parent_age_at_birth,
                                    format_person_display_name(child)
                                ),
                                details: serde_json::json!({
                                    "parent_age": max_parent_age_at_birth,
                                    "child_id": child.id,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }

                    // Rule 4: MOTHER_TOO_OLD (Mother > 55 years old at child's birth)
                    if parent.sex.eq_ignore_ascii_case("F") && min_parent_age_at_birth > 55 {
                        let wid = format!("{RULE_MOTHER_TOO_OLD}:{}:{}", parent.id, child.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_MOTHER_TOO_OLD,
                            &parent.id,
                            Some(&child.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_MOTHER_TOO_OLD.to_string(),
                                rule_code: RULE_MOTHER_TOO_OLD.to_string(),
                                category: "family_age".to_string(),
                                severity: "warning".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: parent.id.clone(),
                                person: to_warning_person_ref(parent),
                                related_person: Some(to_warning_person_ref(child)),
                                message: format!(
                                    "Mother {} was {} years old when {} was born (> 55)",
                                    format_person_display_name(parent),
                                    min_parent_age_at_birth,
                                    format_person_display_name(child)
                                ),
                                details: serde_json::json!({
                                    "mother_age": min_parent_age_at_birth,
                                    "child_id": child.id,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }

                // Rule 5: BORN_AFTER_PARENT_DEATH
                // Mother: child born after mother's death
                // Father: child born > 10 months (~1 year or +10000 sort key) after father's death
                if let Some(p_death) = person_death_window(parent, &store.events) {
                    let is_mother = parent.sex.eq_ignore_ascii_case("F");
                    let violated = if is_mother {
                        c_birth.min_sort > p_death.max_sort
                    } else {
                        let father_limit_sort = p_death.max_sort + 10_000;
                        c_birth.min_sort > father_limit_sort
                    };
                    if violated {
                        let wid =
                            format!("{RULE_BORN_AFTER_PARENT_DEATH}:{}:{}", child.id, parent.id);
                        let dismissed = is_warning_dismissed(
                            &store.dismissed_warnings,
                            &wid,
                            RULE_BORN_AFTER_PARENT_DEATH,
                            &child.id,
                            Some(&parent.id),
                        );
                        if include_dismissed || !dismissed {
                            warnings.push(QualityWarningDto {
                                id: wid,
                                code: RULE_BORN_AFTER_PARENT_DEATH.to_string(),
                                rule_code: RULE_BORN_AFTER_PARENT_DEATH.to_string(),
                                category: "chronology".to_string(),
                                severity: "critical".to_string(),
                                entity_type: "individual".to_string(),
                                entity_id: child.id.clone(),
                                person: to_warning_person_ref(child),
                                related_person: Some(to_warning_person_ref(parent)),
                                message: format!(
                                    "{} was born after the death of parent {}",
                                    format_person_display_name(child),
                                    format_person_display_name(parent)
                                ),
                                details: serde_json::json!({
                                    "parent_id": parent.id,
                                    "parent_death_sort": p_death.max_sort,
                                    "child_birth_sort": c_birth.min_sort,
                                }),
                                is_dismissed: dismissed,
                            });
                        }
                    }
                }
            }
        }
    }

    // Rule 8: POSSIBLE_DUPLICATE (Duplicate candidate pairs with score >= 75)
    let duplicates = compute_duplicate_candidates(store, 75);
    for dup in duplicates {
        let wid = format!(
            "{RULE_POSSIBLE_DUPLICATE}:{}:{}",
            dup.person_a.id, dup.person_b.id
        );
        let dismissed = is_warning_dismissed(
            &store.dismissed_warnings,
            &wid,
            RULE_POSSIBLE_DUPLICATE,
            &dup.person_a.id,
            Some(&dup.person_b.id),
        );
        if include_dismissed || !dismissed {
            warnings.push(QualityWarningDto {
                id: wid,
                code: RULE_POSSIBLE_DUPLICATE.to_string(),
                rule_code: RULE_POSSIBLE_DUPLICATE.to_string(),
                category: "duplicates".to_string(),
                severity: "warning".to_string(),
                entity_type: "individual".to_string(),
                entity_id: dup.person_a.id.clone(),
                person: dup.person_a.clone(),
                related_person: Some(dup.person_b.clone()),
                message: format!(
                    "Possible duplicate ({}% match): {} and {}",
                    dup.score, dup.person_a.display_name, dup.person_b.display_name
                ),
                details: serde_json::json!({
                    "score": dup.score,
                    "reasons": dup.reasons,
                    "person_b_id": dup.person_b.id,
                }),
                is_dismissed: dismissed,
            });
        }
    }

    // Rule 9: DUPLICATE_EVENT (Duplicate life events on individuals or couples)
    let dup_event_groups = compute_duplicate_event_groups(store, true);
    for group in dup_event_groups {
        let wid = format!("{RULE_DUPLICATE_EVENT}:{}", group.id);
        let dismissed = is_event_group_dismissed(store, &group.id);
        if include_dismissed || !dismissed {
            let event_ids: Vec<String> = group.events.iter().map(|e| e.id.clone()).collect();
            warnings.push(QualityWarningDto {
                id: wid,
                code: RULE_DUPLICATE_EVENT.to_string(),
                rule_code: RULE_DUPLICATE_EVENT.to_string(),
                category: "duplicates".to_string(),
                severity: "warning".to_string(),
                entity_type: "event".to_string(),
                entity_id: group.id.clone(),
                person: group.person.clone(),
                related_person: group.partner.clone(),
                message: group.reason.clone(),
                details: serde_json::json!({
                    "group_id": group.id,
                    "event_type": group.event_type,
                    "scope": group.scope,
                    "event_ids": event_ids,
                    "has_conflict": group.has_conflict,
                    "conflict_fields": group.conflict_fields,
                    "confidence": group.confidence,
                    "proposed": group.proposed,
                }),
                is_dismissed: dismissed,
            });
        }
    }

    warnings.sort_by_key(|w| match w.severity.as_str() {
        "critical" => 0,
        "warning" => 1,
        _ => 2,
    });

    warnings
}

/// Normalizes a name token across Russian Cyrillic and Latin scripts for fuzzy comparison (`13 §2.2`, `15 §3.1`).
pub(crate) fn normalize_cross_script_token(raw: &str) -> String {
    let ortho = normalize_russian_orthography_word(raw);
    let mut out = String::with_capacity(ortho.len() * 2);
    for ch in ortho.chars() {
        match ch {
            'а' => out.push('a'),
            'б' => out.push('b'),
            'в' => out.push('v'),
            'г' => out.push('g'),
            'д' => out.push('d'),
            'е' => out.push('e'),
            'ж' => out.push_str("zh"),
            'з' => out.push('z'),
            'и' | 'й' | 'ы' => out.push('i'),
            'к' => out.push('k'),
            'л' => out.push('l'),
            'м' => out.push('m'),
            'н' => out.push('n'),
            'о' => out.push('o'),
            'п' => out.push('p'),
            'р' => out.push('r'),
            'с' => out.push('s'),
            'т' => out.push('t'),
            'у' => out.push('u'),
            'ф' => out.push('f'),
            'х' => out.push_str("kh"),
            'ц' => out.push_str("ts"),
            'ч' => out.push_str("ch"),
            'ш' => out.push_str("sh"),
            'щ' => out.push_str("shch"),
            'ъ' | 'ь' => {}
            'ю' => out.push_str("yu"),
            'я' => out.push_str("ya"),
            c if c.is_alphanumeric() => out.push(c),
            _ => {}
        }
    }
    // Normalize Latin transliteration variations of Cyrillic 'ё'
    let out = out.replace("yo", "e").replace("jo", "e");
    for suffix in ["ii", "iy", "yi", "yy", "ey", "ei"] {
        if let Some(stem) = out.strip_suffix(suffix) {
            if stem.len() >= 2 {
                return format!("{stem}i");
            }
        }
    }
    if let Some(stem) = out.strip_suffix('y') {
        if stem.len() >= 3 {
            return format!("{stem}i");
        }
    }
    out
}

pub(crate) fn canonical_surname_key(raw: &str) -> String {
    let folded = fold_slavic_surname_stem(raw);
    normalize_cross_script_token(&folded)
}

pub(crate) fn bigrams(s: &str) -> HashSet<(char, char)> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 {
        return HashSet::new();
    }
    chars.windows(2).map(|w| (w[0], w[1])).collect()
}

pub(crate) fn dice_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }
    let bg_a = bigrams(a);
    let bg_b = bigrams(b);
    if bg_a.is_empty() || bg_b.is_empty() {
        return 0.0;
    }
    let inter = bg_a.intersection(&bg_b).count();
    (2.0 * inter as f64) / ((bg_a.len() + bg_b.len()) as f64)
}

pub(crate) fn collect_person_surnames(p: &TreePersonNodeDto) -> Vec<String> {
    let mut res = Vec::new();
    if !p.primary_name.surname.trim().is_empty() {
        res.push(canonical_surname_key(&p.primary_name.surname));
    }
    if let Some(bs) = p.birth_surname.as_deref() {
        if !bs.trim().is_empty() {
            res.push(canonical_surname_key(bs));
        }
    }
    for alt in &p.alt_names {
        if let Some(s) = alt.surname.as_deref() {
            if !s.trim().is_empty() {
                res.push(canonical_surname_key(s));
            }
        }
    }
    res.sort();
    res.dedup();
    res
}

pub(crate) fn collect_person_given_names(p: &TreePersonNodeDto) -> Vec<String> {
    let mut res = Vec::new();
    let add_token = |res: &mut Vec<String>, raw: &str| {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            res.push(normalize_cross_script_token(trimmed));
            for part in trimmed.split_whitespace() {
                let part_trimmed = part.trim();
                if !part_trimmed.is_empty() && part_trimmed != trimmed {
                    res.push(normalize_cross_script_token(part_trimmed));
                }
            }
        }
    };

    add_token(&mut res, &p.primary_name.given_name);
    if let Some(nick) = p.nickname.as_deref() {
        add_token(&mut res, nick);
    }
    for alt in &p.alt_names {
        if let Some(g) = alt.given_name.as_deref() {
            add_token(&mut res, g);
        }
    }
    res.sort();
    res.dedup();
    res
}
