#[allow(unused_imports)]
use super::*;

pub(crate) fn confidence_tier(score: i32) -> String {
    if score >= 85 {
        "high".to_string()
    } else if score >= 70 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

pub fn is_not_duplicate_pair(store: &GenealogyPrivacyStore, id_a: &str, id_b: &str) -> bool {
    let (p1, p2) = if id_a < id_b {
        (id_a, id_b)
    } else {
        (id_b, id_a)
    };
    store
        .not_duplicate_pairs
        .iter()
        .any(|(a, b)| a == p1 && b == p2)
}

/// Computes all candidate duplicate pairs in `store` with `score >= min_score`, excluding `not_duplicate_pairs`.
pub fn compute_duplicate_candidates(
    store: &GenealogyPrivacyStore,
    min_score: i32,
) -> Vec<DuplicateCandidatePairDto> {
    let graph = TreeRelationshipGraph::build(store);
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();
    let mut out = Vec::new();

    for i in 0..store.persons.len() {
        for j in (i + 1)..store.persons.len() {
            let p_a = &store.persons[i];
            let p_b = &store.persons[j];
            if is_not_duplicate_pair(store, &p_a.id, &p_b.id) {
                continue;
            }
            let (score, reasons) =
                score_duplicate_pair_with_graph(p_a, p_b, store, &graph, &person_by_id);
            if score >= min_score {
                let (first, second) = if p_a.id < p_b.id {
                    (p_a, p_b)
                } else {
                    (p_b, p_a)
                };
                let is_singleton_a = graph.is_singleton.get(&p_a.id).copied().unwrap_or(true);
                let is_singleton_b = graph.is_singleton.get(&p_b.id).copied().unwrap_or(true);
                let comp_a = graph.component_by_person.get(&p_a.id).copied();
                let comp_b = graph.component_by_person.get(&p_b.id).copied();
                let is_detached =
                    is_singleton_a || is_singleton_b || comp_a.is_none() || comp_a != comp_b;

                out.push(DuplicateCandidatePairDto {
                    id: format!("dup:{}:{}", first.id, second.id),
                    person_a: to_warning_person_ref(first),
                    person_b: to_warning_person_ref(second),
                    score,
                    confidence: confidence_tier(score),
                    reasons,
                    is_detached: Some(is_detached),
                });
            }
        }
    }
    out.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.id.cmp(&b.id)));
    out
}

/// Matches a prospective person input against existing records in the tree (`GET /api/v1/people/match`).
pub fn match_prospective_person(
    store: &GenealogyPrivacyStore,
    query: &PersonMatchQuery,
    access: &CallerAccess,
    locale: &str,
) -> PersonMatchResponse {
    let mut given_name = query
        .given_name
        .clone()
        .or_else(|| query.given.clone())
        .unwrap_or_default();
    let mut surname = query.surname.clone().unwrap_or_default();
    let mut patronymic = query.patronymic.clone();

    if given_name.trim().is_empty() && surname.trim().is_empty() {
        if let Some(full) = query.name.as_deref() {
            let parts: Vec<&str> = full.split_whitespace().collect();
            match parts.len() {
                1 => {
                    given_name = parts[0].to_string();
                }
                2 => {
                    given_name = parts[0].to_string();
                    surname = parts[1].to_string();
                }
                n if n >= 3 => {
                    given_name = parts[0].to_string();
                    patronymic = Some(parts[1].to_string());
                    surname = parts[2..].join(" ");
                }
                _ => {}
            }
        }
    }

    let birth_date_str = query
        .birth
        .clone()
        .or_else(|| query.birth_year.map(|y| y.to_string()));
    let birth_sort = query.birth_year.map(|y| y * 10_000);
    let sex = query
        .sex
        .clone()
        .or_else(|| query.gender.clone())
        .unwrap_or_else(|| "U".to_string());

    let probe = TreePersonNodeDto {
        id: "__probe__".to_string(),
        gid: "I0000".to_string(),
        sex,
        is_living: true,
        living_override: None,
        is_private: false,
        primary_name: PersonNameDto {
            given_name,
            patronymic,
            surname,
        },
        birth_surname: None,
        nickname: None,
        name_prefix: None,
        name_suffix: None,
        biography: None,
        birth: if birth_date_str.is_some() || query.birth_place.is_some() {
            Some(VitalEventSummaryDto {
                date: birth_date_str,
                place: query.birth_place.clone(),
            })
        } else {
            None
        },
        death: None,
        avatar_url: None,
        avatar_media_id: None,
        avatar_crop: None,
        notes: None,
        birth_sort,
        death_sort: None,
        has_death_event: false,
        alt_names: vec![],
        external_ids: HashMap::new(),
        version: 1,
        tx_id: None,
        generation: None,
        is_direct_lineage: None,
    };

    let graph = TreeRelationshipGraph::build(store);
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();

    let mut items = Vec::new();
    for existing in &store.persons {
        if Some(existing.id.as_str()) == query.exclude_id.as_deref()
            || Some(existing.gid.as_str()) == query.exclude_id.as_deref()
        {
            continue;
        }
        let Some(visible) = redact_person_for_caller(existing, access, locale) else {
            continue;
        };
        let (score, reasons) =
            score_duplicate_pair_with_graph(&probe, &visible, store, &graph, &person_by_id);
        if score >= 50 {
            items.push(PersonMatchCandidateDto {
                person: to_warning_person_ref(&visible),
                score,
                confidence: confidence_tier(score),
                reasons,
            });
        }
    }

    items.sort_by_key(|b| std::cmp::Reverse(b.score));
    items.truncate(5);
    let total = items.len();
    PersonMatchResponse { items, total }
}

/// Builds the quality summary response (`GET /api/v1/quality` and `GET /api/v1/quality/summary`).
pub fn build_quality_summary(store: &GenealogyPrivacyStore) -> QualitySummaryResponse {
    let warnings = compute_quality_warnings(store, false);
    let duplicates = compute_duplicate_candidates(store, 75);
    let place_analysis = analyze_place_hierarchies(store, &PlaceHierarchyAnalysisQuery::default());

    let mut chronology = 0;
    let mut family_age = 0;
    let mut completeness = 0;
    let mut duplicates_cat = 0;
    let mut warning_count = 0;
    let mut info_count = 0;
    let mut by_rule: HashMap<String, usize> = HashMap::new();

    for w in &warnings {
        *by_rule.entry(w.rule_code.clone()).or_insert(0) += 1;
        match w.category.as_str() {
            "chronology" => chronology += 1,
            "family_age" => family_age += 1,
            "completeness" => completeness += 1,
            "duplicates" => duplicates_cat += 1,
            _ => {}
        }
        if w.severity == "info" {
            info_count += 1;
        } else {
            warning_count += 1;
        }
    }

    for issue in &place_analysis.issues {
        *by_rule.entry(issue.rule_code.clone()).or_insert(0) += 1;
    }

    QualitySummaryResponse {
        status: "ok".to_string(),
        operation: "data_quality".to_string(),
        message: "Data quality warnings authorized for administrator".to_string(),
        total_open_warnings: warnings.len(),
        warning_count,
        info_count,
        duplicate_candidates_count: duplicates.len(),
        place_hierarchy_issues: place_analysis.counts.total_issues,
        place_hierarchy_high_confidence: place_analysis.counts.high_confidence_count,
        dismissed_count: store.dismissed_warnings.len(),
        by_category: QualityCategoryCountsDto {
            chronology,
            family_age,
            completeness,
            duplicates: duplicates_cat,
            places: place_analysis.counts.total_issues,
        },
        by_severity: QualitySeverityCountsDto {
            warning: warning_count,
            info: info_count,
        },
        by_rule,
        warnings,
        duplicates,
    }
}

/// Records a dismissed warning (`POST /api/v1/quality/warnings/dismiss` or `POST /api/v1/quality/warnings/{id}/dismiss`).
pub fn dismiss_warning_in_store(
    store: &mut GenealogyPrivacyStore,
    path_id: Option<&str>,
    req: DismissWarningRequest,
    actor_id: Option<String>,
) -> Result<DismissWarningResponse, AppError> {
    let raw_id = path_id
        .map(|s| s.to_string())
        .or(req.id.clone())
        .unwrap_or_default();

    let mut rule_code = req
        .rule_code
        .or(req.code)
        .unwrap_or_default()
        .trim()
        .to_uppercase();
    let mut entity_id = req.entity_id.unwrap_or_default().trim().to_string();
    let mut secondary_id = req.secondary_id;

    if (rule_code.is_empty() || entity_id.is_empty()) && !raw_id.trim().is_empty() {
        let parts: Vec<&str> = raw_id.trim().split(':').collect();
        if parts.len() >= 2 {
            if rule_code.is_empty() {
                rule_code = parts[0].to_uppercase();
            }
            if entity_id.is_empty() {
                entity_id = parts[1].to_string();
            }
            if parts.len() >= 3 && secondary_id.is_none() {
                secondary_id = Some(parts[2].to_string());
            }
        }
    }

    let mut inferred_entity_type = req.entity_type.clone();
    if let Some(resolved) = store.resolve_person_id(&entity_id) {
        entity_id = resolved;
        if inferred_entity_type.is_none() {
            inferred_entity_type = Some("individual".to_string());
        }
    } else if let Some(resolved) = store.resolve_place_id(&entity_id) {
        entity_id = resolved;
        if inferred_entity_type.is_none() {
            inferred_entity_type = Some("place".to_string());
        }
    } else if rule_code.starts_with("PLACE_") && inferred_entity_type.is_none() {
        inferred_entity_type = Some("place".to_string());
    }

    if rule_code.is_empty() || entity_id.is_empty() {
        let mut fields = HashMap::new();
        fields.insert(
            "warning".to_string(),
            "Both rule_code and entity_id (or composite warning id) are required".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Invalid warning dismissal parameters".to_string(),
            Some(fields),
        ));
    }

    let id = if !raw_id.trim().is_empty() {
        raw_id.trim().to_string()
    } else if let Some(sec) = secondary_id.as_deref() {
        format!("{rule_code}:{entity_id}:{sec}")
    } else {
        format!("{rule_code}:{entity_id}")
    };

    let dto = DismissedWarningDto {
        id: id.clone(),
        rule_code: rule_code.clone(),
        entity_type: inferred_entity_type.unwrap_or_else(|| "individual".to_string()),
        entity_id: entity_id.clone(),
        secondary_id: secondary_id.clone(),
        dismissed_by: actor_id,
        dismissed_at: Utc::now().to_rfc3339(),
    };

    if !store.dismissed_warnings.iter().any(|d| {
        d.id == dto.id
            || (d.rule_code == dto.rule_code
                && d.entity_id == dto.entity_id
                && d.secondary_id == dto.secondary_id)
    }) {
        store.dismissed_warnings.push(dto.clone());
    }

    Ok(DismissWarningResponse {
        status: "dismissed".to_string(),
        dismissed: dto,
    })
}

/// Marks a pair of individuals as not-duplicates (`POST /api/v1/quality/duplicates/not-duplicate` / `POST /api/v1/people/merge/not-duplicate`).
pub fn mark_not_duplicate_in_store(
    store: &mut GenealogyPrivacyStore,
    req: NotDuplicatePairRequest,
) -> Result<NotDuplicatePairResponse, AppError> {
    let raw_a = req
        .person_a_id
        .or(req.person_a)
        .or(req.a)
        .unwrap_or_default();
    let raw_b = req
        .person_b_id
        .or(req.person_b)
        .or(req.b)
        .unwrap_or_default();

    let id_a = store.resolve_person_id(&raw_a).ok_or(AppError::NotFound)?;
    let id_b = store.resolve_person_id(&raw_b).ok_or(AppError::NotFound)?;

    if id_a == id_b {
        let mut fields = HashMap::new();
        fields.insert(
            "person_b_id".to_string(),
            "Cannot mark the same person as not-duplicate of themselves".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "Distinct individuals required".to_string(),
            Some(fields),
        ));
    }

    let (p1, p2) = if id_a < id_b {
        (id_a, id_b)
    } else {
        (id_b, id_a)
    };

    if !store
        .not_duplicate_pairs
        .iter()
        .any(|(a, b)| a == &p1 && b == &p2)
    {
        store.not_duplicate_pairs.push((p1.clone(), p2.clone()));
    }

    Ok(NotDuplicatePairResponse {
        status: "marked_not_duplicate".to_string(),
        person_a_id: p1,
        person_b_id: p2,
    })
}
