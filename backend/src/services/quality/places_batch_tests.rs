#[allow(unused_imports)]
use super::*;

pub(crate) fn issue_type_to_category(issue_type: &str) -> String {
    match issue_type {
        "unparsed_path" | "unparsed_paths" => "unparsed_paths".to_string(),
        "orphan_hierarchy" | "orphans" => "orphans".to_string(),
        "type_mismatch" | "type_mismatches" => "type_mismatches".to_string(),
        "duplicate_node" | "duplicates" => "duplicates".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn matches_issue_type_or_category(issue: &PlaceHierarchyIssueDto, filter: &str) -> bool {
    let f = filter.trim().to_ascii_lowercase();
    issue.issue_type.eq_ignore_ascii_case(&f)
        || issue.category.eq_ignore_ascii_case(&f)
        || issue.rule_code.eq_ignore_ascii_case(&f)
        || issue_type_to_category(&f) == issue.category
}

pub(crate) fn make_parent_candidate(
    cand: &PlaceSummaryDto,
    place_type: String,
    distance_km: Option<f64>,
    confidence: i32,
    match_reason: String,
) -> ParentPlaceCandidateDto {
    ParentPlaceCandidateDto {
        place_id: cand.id.clone(),
        gid: cand.gid.clone(),
        place_gid: cand.gid.clone(),
        name: cand.name.clone(),
        full_name: cand.full_name.clone(),
        place_type,
        distance_km,
        confidence,
        score: confidence,
        match_reason,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_place_issue_dto(
    id: String,
    place: &PlaceSummaryDto,
    issue_type: &str,
    rule_code: &str,
    severity: &str,
    confidence: i32,
    current_parent_id: Option<String>,
    current_parent_name: Option<String>,
    proposed_clean_name: String,
    proposed_place_type: String,
    proposed_parent_id: Option<String>,
    proposed_parent_name: Option<String>,
    proposed_full_name: String,
    proposed_chain: Vec<ProposedHierarchyStepDto>,
    parent_candidates: Vec<ParentPlaceCandidateDto>,
    consolidate_with_place_id: Option<String>,
    consolidate_with_name: Option<String>,
    distance_km: Option<f64>,
    reason: String,
    preserve_alt_name: Option<String>,
) -> PlaceHierarchyIssueDto {
    PlaceHierarchyIssueDto {
        id,
        place_id: place.id.clone(),
        place_gid: place.gid.clone(),
        issue_type: issue_type.to_string(),
        category: issue_type_to_category(issue_type),
        rule_code: rule_code.to_string(),
        severity: severity.to_string(),
        confidence,
        confidence_score: confidence,
        confidence_tier: confidence_tier(confidence),
        current_name: place.name.clone(),
        current_full_name: place.full_name.clone(),
        current_place_type: place.place_type.clone(),
        current_parent_id,
        current_parent_name,
        latitude: place.latitude,
        longitude: place.longitude,
        events_count: place.events_count,
        children_count: place.children_count,
        proposed_leaf_name: proposed_clean_name.clone(),
        proposed_clean_name,
        proposed_place_type,
        proposed_parent_id,
        proposed_parent_name,
        proposed_full_name,
        proposed_chain,
        candidate_parents: parent_candidates.clone(),
        parent_candidates,
        consolidate_with_place_id,
        consolidate_with_name,
        distance_km,
        rationale: reason.clone(),
        reason,
        preserve_alt_name,
    }
}

/// Applies hierarchy restructuring for a single place (`POST /api/v1/quality/places/fix`).
pub fn fix_place_hierarchy(
    store: &mut GenealogyPrivacyStore,
    req: FixPlaceHierarchyRequest,
    actor_id: Option<String>,
    actor_name: Option<String>,
    current_year: i32,
    age_threshold: i32,
) -> Result<FixPlaceHierarchyResponse, AppError> {
    let tx_id = format!("tx-{}", Uuid::new_v4());
    let (place, created_parents, consolidated, merged_source_place_id) = fix_single_place_in_tx(
        store,
        &req,
        &tx_id,
        "quality.place_hierarchy_fix",
        actor_id,
        actor_name,
    )?;

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    let final_place = store
        .places
        .iter()
        .find(|p| p.id == place.id)
        .cloned()
        .unwrap_or(place);
    let breadcrumbs = build_place_breadcrumbs(&store.places, &final_place.id);

    Ok(FixPlaceHierarchyResponse {
        status: "ok".to_string(),
        operation: "place_hierarchy_fix".to_string(),
        message: format!("Place hierarchy updated for {}", final_place.full_name),
        tx_id,
        place: final_place,
        created_parents,
        consolidated,
        merged_source_place_id,
        breadcrumbs,
    })
}

/// Batch auto-fixes all high-confidence place hierarchy issues (`POST /api/v1/quality/places/batch-fix`).
pub fn batch_fix_places(
    store: &mut GenealogyPrivacyStore,
    req: BatchFixPlacesRequest,
    actor_id: Option<String>,
    actor_name: Option<String>,
    current_year: i32,
    age_threshold: i32,
) -> Result<BatchFixPlacesResponse, AppError> {
    let min_confidence = req.min_confidence.unwrap_or(85);
    let analysis = analyze_place_hierarchies(
        store,
        &PlaceHierarchyAnalysisQuery {
            issue_type: req.issue_type.clone().or(req.category.clone()),
            category: req.category.clone(),
            min_confidence: Some(min_confidence),
            include_dismissed: Some(false),
        },
    );

    let requested_ids: Option<HashSet<String>> = req.place_ids.as_ref().map(|ids| {
        ids.iter()
            .filter_map(|raw| {
                store
                    .resolve_place_id(raw)
                    .or_else(|| Some(raw.trim().to_string()))
            })
            .collect()
    });

    let target_issues: Vec<PlaceHierarchyIssueDto> = analysis
        .issues
        .into_iter()
        .filter(|iss| {
            if let Some(ref filter_set) = requested_ids {
                filter_set.contains(&iss.place_id)
            } else {
                true
            }
        })
        .collect();

    let tx_id = format!("tx-{}", Uuid::new_v4());
    let mut fixed_count = 0usize;
    let mut created_parents_count = 0usize;
    let mut consolidated_count = 0usize;
    let mut fixed_place_ids = Vec::new();
    let mut updated_places = Vec::new();

    for issue in target_issues {
        if !store.places.iter().any(|p| p.id == issue.place_id) {
            continue;
        }
        let single_req = FixPlaceHierarchyRequest {
            place_id: Some(issue.place_id.clone()),
            ..Default::default()
        };
        if let Ok((pl, created, consolidated, _)) = fix_single_place_in_tx(
            store,
            &single_req,
            &tx_id,
            "quality.place_hierarchy_batch_fix",
            actor_id.clone(),
            actor_name.clone(),
        ) {
            fixed_count += 1;
            created_parents_count += created.len();
            if consolidated {
                consolidated_count += 1;
            }
            fixed_place_ids.push(pl.id.clone());
            updated_places.push(pl);
        }
    }

    recompute_all_places_metadata(store);
    recompute_all_individuals_vitals(store, current_year, age_threshold);

    // Refresh updated_places after final metadata recomputation
    let refreshed_places: Vec<PlaceSummaryDto> = fixed_place_ids
        .iter()
        .filter_map(|pid| store.places.iter().find(|p| &p.id == pid).cloned())
        .collect();

    Ok(BatchFixPlacesResponse {
        status: "ok".to_string(),
        operation: "place_hierarchy_batch_fix".to_string(),
        message: format!("{fixed_count} place hierarchies updated"),
        tx_id,
        fixed_count,
        created_parents_count,
        consolidated_count,
        fixed_place_ids,
        places: refreshed_places,
    })
}

/// Dismisses a place hierarchy recommendation (`POST /api/v1/quality/places/dismiss`).
pub fn dismiss_place_issue_in_store(
    store: &mut GenealogyPrivacyStore,
    req: DismissPlaceIssueRequest,
    actor_id: Option<String>,
) -> Result<DismissWarningResponse, AppError> {
    let raw_place_id = req
        .place_id
        .or(req.id)
        .unwrap_or_default()
        .trim()
        .to_string();

    if raw_place_id.is_empty() {
        let mut fields = HashMap::new();
        fields.insert("place_id".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "place_id is required to dismiss a place hierarchy issue".to_string(),
            Some(fields),
        ));
    }

    // If composite ID like `PLACE_UNPARSED_PATH:<uuid>` was passed, delegate to `dismiss_warning_in_store`
    if raw_place_id.contains(':') {
        return dismiss_warning_in_store(
            store,
            Some(&raw_place_id),
            DismissWarningRequest {
                entity_type: Some("place".to_string()),
                ..Default::default()
            },
            actor_id,
        );
    }

    let resolved_id = store
        .resolve_place_id(&raw_place_id)
        .ok_or(AppError::NotFound)?;

    let rule_code = req
        .rule_code
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            store
                .places
                .iter()
                .find(|p| p.id == resolved_id)
                .and_then(|p| analyze_single_place_issue(p, store, true))
                .map(|iss| iss.rule_code)
        })
        .unwrap_or_else(|| RULE_PLACE_ORPHAN_HIERARCHY.to_string());

    dismiss_warning_in_store(
        store,
        None,
        DismissWarningRequest {
            rule_code: Some(rule_code),
            entity_type: Some("place".to_string()),
            entity_id: Some(resolved_id),
            ..Default::default()
        },
        actor_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::privacy::FamilyDto;

    fn make_test_person(
        id: &str,
        given: &str,
        surname: &str,
        patronymic: Option<&str>,
        sex: &str,
        birth_date: Option<&str>,
    ) -> TreePersonNodeDto {
        TreePersonNodeDto {
            id: id.to_string(),
            gid: format!("I{id}"),
            sex: sex.to_string(),
            is_living: false,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: given.to_string(),
                patronymic: patronymic.map(|s| s.to_string()),
                surname: surname.to_string(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: None,
            birth: birth_date.map(|d| VitalEventSummaryDto {
                date: Some(d.to_string()),
                place: None,
            }),
            death: None,
            avatar_url: None,
            avatar_media_id: None,
            avatar_crop: None,
            notes: None,
            birth_sort: None,
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
            generation: None,
            is_direct_lineage: None,
        }
    }

    #[test]
    fn test_different_given_names_false_positive_prevention() {
        let mut store = GenealogyPrivacyStore::empty();

        // Person A: Татьяна Шурубейко, female, born 20 NOV 1987
        let p_tatiana =
            make_test_person("p1", "Татьяна", "Шурубейко", None, "F", Some("20 NOV 1987"));

        // Person B: Валентина Левшиц, female, born 1 APR 1987
        let p_valentina =
            make_test_person("p2", "Валентина", "Левшиц", None, "F", Some("1 APR 1987"));

        // Person C: Shared relative (e.g. spouse/partner or relative)
        let p_relative = make_test_person("p3", "Иван", "Левшиц", None, "M", Some("1985"));

        store.persons.push(p_tatiana.clone());
        store.persons.push(p_valentina.clone());
        store.persons.push(p_relative.clone());

        // Connect both to shared relative in store families
        store.families.push(FamilyDto {
            id: "fam1".to_string(),
            gid: "F0001".to_string(),
            partner1_id: Some("p1".to_string()),
            partner2_id: Some("p3".to_string()),
            relationship_type: "married".to_string(),
            is_private: false,
            children: vec![],
            external_ids: HashMap::new(),
            version: 1,
            marriage_date: None,
            marriage_place: None,
        });
        store.families.push(FamilyDto {
            id: "fam2".to_string(),
            gid: "F0002".to_string(),
            partner1_id: Some("p2".to_string()),
            partner2_id: Some("p3".to_string()),
            relationship_type: "married".to_string(),
            is_private: false,
            children: vec![],
            external_ids: HashMap::new(),
            version: 1,
            marriage_date: None,
            marriage_place: None,
        });

        // 1. With completely different surnames ("Шурубейко" vs "Левшиц"):
        let (score1, _) = score_duplicate_pair(&p_tatiana, &p_valentina, &store);
        assert_eq!(
            score1, 0,
            "Different first and last names must return score 0"
        );

        // 2. Even if Татьяна took the married surname "Левшиц":
        let mut p_tatiana_married = p_tatiana.clone();
        p_tatiana_married.primary_name.surname = "Левшиц".to_string();
        p_tatiana_married.birth_surname = Some("Шурубейко".to_string());

        let (score2, _) = score_duplicate_pair(&p_tatiana_married, &p_valentina, &store);
        assert!(
            score2 < 40,
            "Татьяна vs Валентина with same married surname and birth year must have score < 40, got {score2}"
        );

        // 3. Even without exact dates (both just born 1987):
        let mut p_t_year = p_tatiana_married.clone();
        p_t_year.birth = Some(VitalEventSummaryDto {
            date: Some("1987".to_string()),
            place: None,
        });
        let mut p_v_year = p_valentina.clone();
        p_v_year.birth = Some(VitalEventSummaryDto {
            date: Some("1987".to_string()),
            place: None,
        });
        let (score3, _) = score_duplicate_pair(&p_t_year, &p_v_year, &store);
        assert!(
            score3 < 40,
            "Татьяна vs Валентина with year 1987 only must have score < 40, got {score3}"
        );

        // Compute candidates with threshold 75 — must NOT include this pair
        let candidates = compute_duplicate_candidates(&store, 75);
        assert!(
            candidates.is_empty(),
            "Expected 0 duplicate candidates, got {:?}",
            candidates
        );
    }

    #[test]
    fn test_actual_duplicate_candidates_flagged() {
        let store = GenealogyPrivacyStore::empty();

        // Person A: Petr Shcheglov, born 1910
        let p_petr = make_test_person("p1", "Petr", "Shcheglov", None, "M", Some("1910"));

        // Person B: Pyotr Fyodorovich Shcheglov, born 1910
        let p_pyotr = make_test_person(
            "p2",
            "Pyotr",
            "Shcheglov",
            Some("Fyodorovich"),
            "M",
            Some("1910"),
        );

        let (score, reasons) = score_duplicate_pair(&p_petr, &p_pyotr, &store);
        assert!(
            score >= 80,
            "Petr Shcheglov vs Pyotr Fyodorovich Shcheglov must score >= 80, got {score}"
        );
        assert!(reasons.contains(&"same_surname".to_string()));
        assert!(reasons.contains(&"same_given_name".to_string()));
        assert!(reasons.contains(&"same_birth_year".to_string()));
        assert!(reasons.contains(&"detached_singleton".to_string()));

        let mut store_with_dups = store;
        store_with_dups.persons.push(p_petr);
        store_with_dups.persons.push(p_pyotr);

        let candidates = compute_duplicate_candidates(&store_with_dups, 75);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].score >= 80);
    }

    #[test]
    fn test_exact_birth_date_conflict_penalty() {
        let store = GenealogyPrivacyStore::empty();

        // Person A: Ivan Ivanov, born 20 NOV 1987
        let p_a = make_test_person("p1", "Ivan", "Ivanov", None, "M", Some("20 NOV 1987"));

        // Person B: Ivan Ivanov, born 1 APR 1987 (7 months apart, > 30 days)
        let p_b = make_test_person("p2", "Ivan", "Ivanov", None, "M", Some("1 APR 1987"));

        let (score, _) = score_duplicate_pair(&p_a, &p_b, &store);
        // Same surname (20) + same given (20) + same birth year (15) + detached_singleton (25) - date conflict penalty (30) = 50
        assert_eq!(score, 50);

        // Person C: Ivan Ivanov, born 25 NOV 1987 (5 days apart, <= 30 days)
        let p_c = make_test_person("p3", "Ivan", "Ivanov", None, "M", Some("25 NOV 1987"));
        let (score_close, _) = score_duplicate_pair(&p_a, &p_c, &store);
        // No date conflict penalty: 20 + 20 + 15 + 25 = 80
        assert_eq!(score_close, 80);
    }

    #[test]
    fn test_nickname_in_given_names() {
        let mut p = make_test_person("p1", "Alexander", "Ivanov", None, "M", None);
        p.nickname = Some("Sasha".to_string());
        let names = collect_person_given_names(&p);
        assert!(names.contains(&"sasha".to_string()));
        assert!(names.contains(&"alexander".to_string()));
    }

    #[test]
    fn test_root_country_belarus_not_demoted_under_unknown_placeholder() {
        use crate::services::genealogy::resolve_or_create_place_chain;

        let mut store = GenealogyPrivacyStore::empty();

        // 1. Canonical root country "Беларусь" (P0106, country, parent: None)
        store.places.push(PlaceSummaryDto {
            id: "pl-belarus-root".to_string(),
            gid: "P0106".to_string(),
            name: "Беларусь".to_string(),
            place_type: "country".to_string(),
            parent_id: None,
            full_name: "Беларусь".to_string(),
            country: Some("Беларусь".to_string()),
            latitude: None,
            longitude: None,
            events_count: 1,
            children_count: 0,
            alt_names: vec![],
            version: 1,
        });

        // 2. Bogus "Unknown" root country and duplicate "Беларусь" (region) under "Unknown"
        store.places.push(PlaceSummaryDto {
            id: "pl-unknown".to_string(),
            gid: "P0101".to_string(),
            name: "Unknown".to_string(),
            place_type: "country".to_string(),
            parent_id: None,
            full_name: "Unknown".to_string(),
            country: Some("Unknown".to_string()),
            latitude: None,
            longitude: None,
            events_count: 0,
            children_count: 1,
            alt_names: vec![],
            version: 1,
        });
        store.places.push(PlaceSummaryDto {
            id: "pl-belarus-bogus".to_string(),
            gid: "P0102".to_string(),
            name: "Беларусь".to_string(),
            place_type: "region".to_string(),
            parent_id: Some("pl-unknown".to_string()),
            full_name: "Беларусь, Unknown".to_string(),
            country: Some("Unknown".to_string()),
            latitude: None,
            longitude: None,
            events_count: 2,
            children_count: 0,
            alt_names: vec![],
            version: 1,
        });

        // 3. Also resolve a GEDCOM-style path with ", Unknown": "Кобрин, Беларусь, Unknown"
        let kobrin = resolve_or_create_place_chain(
            &mut store,
            "Кобрин, Беларусь, Unknown",
            None,
            None,
            None,
            None,
        )
        .expect("should resolve Кобрин under Беларусь without creating Unknown");
        assert_eq!(kobrin.name, "Кобрин");
        assert!(
            !kobrin.full_name.contains("Unknown"),
            "full_name must never include Unknown: {}",
            kobrin.full_name
        );

        let analysis = analyze_place_hierarchies(&store, &PlaceHierarchyAnalysisQuery::default());

        // Canonical root country "Беларусь" (P0106) must NEVER be flagged as an issue!
        assert!(
            !analysis
                .issues
                .iter()
                .any(|i| i.place_id == "pl-belarus-root"),
            "Canonical root country Беларусь (P0106) must not be flagged: {:?}",
            analysis.issues
        );
        // Placeholder "Unknown" itself must not be flagged
        assert!(
            !analysis.issues.iter().any(|i| i.place_id == "pl-unknown"),
            "Placeholder Unknown must not generate a hierarchy proposal"
        );

        // Bogus "Беларусь" (P0102, region under Unknown) MUST be flagged to consolidate into root "Беларусь" (P0106)
        let bogus_issue = analysis
            .issues
            .iter()
            .find(|i| i.place_id == "pl-belarus-bogus")
            .expect("Bogus Беларусь under Unknown should be flagged as duplicate");
        assert_eq!(bogus_issue.category, "duplicates");
        assert_eq!(
            bogus_issue.consolidate_with_place_id.as_deref(),
            Some("pl-belarus-root")
        );
        assert_eq!(bogus_issue.proposed_chain.len(), 1);
        assert_eq!(bogus_issue.proposed_chain[0].name, "Беларусь");
        assert_eq!(bogus_issue.proposed_chain[0].place_type, "country");
        assert_eq!(bogus_issue.proposed_chain[0].action, "consolidate");

        // Applying batch-fix consolidates P0102 into P0106 and prunes orphan "Unknown"
        let fix_res = batch_fix_places(
            &mut store,
            BatchFixPlacesRequest::default(),
            None,
            None,
            2026,
            110,
        )
        .unwrap();
        assert!(fix_res.consolidated_count >= 1);
        assert!(
            !store.places.iter().any(|p| p.id == "pl-unknown"),
            "Orphan Unknown placeholder place should be pruned after fix"
        );
        assert!(
            !store.places.iter().any(|p| p.id == "pl-belarus-bogus"),
            "Bogus Беларусь under Unknown should be consolidated into P0106"
        );
        let root_belarus = store
            .places
            .iter()
            .find(|p| p.id == "pl-belarus-root")
            .unwrap();
        assert_eq!(root_belarus.place_type, "country");
        assert_eq!(root_belarus.parent_id, None);
    }
}
