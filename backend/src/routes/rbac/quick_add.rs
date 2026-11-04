#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/persons/{id}/quick-add",
    params(
        ("id" = String, Path, description = "Anchor person UUID or GID")
    ),
    request_body = QuickAddRelativeRequest,
    responses(
        (status = 201, description = "Relative created/connected and linked via family record in a single operation", body = QuickAddRelativeResponse),
        (status = 400, description = "Validation error or ancestor cycle detected", body = ErrorResponse),
        (status = 404, description = "Anchor person not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn quick_add_relative(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<QuickAddRelativeRequest>,
) -> Result<(StatusCode, Json<QuickAddRelativeResponse>), AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;

    let anchor = store
        .persons
        .iter()
        .find(|p| p.id == id.trim() || p.gid.eq_ignore_ascii_case(id.trim()))
        .cloned()
        .ok_or(AppError::NotFound)?;

    let relation = req.relation.trim().to_ascii_lowercase();
    if !matches!(
        relation.as_str(),
        "father" | "mother" | "parent" | "spouse" | "partner" | "child" | "sibling"
    ) {
        let mut fields = HashMap::new();
        fields.insert("relation".to_string(), "invalid_relation".to_string());
        return Err(AppError::ValidationFailed(
            format!("Unsupported relation type: {}", req.relation),
            Some(fields),
        ));
    }

    // Helper closure to create a new person with optional birth/death events
    let create_new_relative = |store: &mut crate::services::privacy::GenealogyPrivacyStore,
                               given_name: String,
                               patronymic: Option<String>,
                               surname: String,
                               sex: String,
                               birth_date: Option<String>,
                               birth_place: Option<String>,
                               death_date: Option<String>,
                               death_place: Option<String>|
     -> Result<String, AppError> {
        if given_name.trim().is_empty() && surname.trim().is_empty() {
            let mut fields = HashMap::new();
            fields.insert("given_name".to_string(), "required".to_string());
            return Err(AppError::ValidationFailed(
                "At least a given name or surname is required".to_string(),
                Some(fields),
            ));
        }

        let existing_gids: HashSet<String> = store.persons.iter().map(|p| p.gid.clone()).collect();
        let gid = allocate_gid('I', &mut store.next_gid_seq, None, &existing_gids);
        let new_id = Uuid::new_v4().to_string();

        let parsed_birth = birth_date.as_deref().map(GenDate::parse);
        let birth_sort = parsed_birth.as_ref().and_then(|d| d.sort_key());
        let parsed_death = death_date.as_deref().map(GenDate::parse);
        let death_sort = parsed_death.as_ref().and_then(|d| d.sort_key());
        let has_death = death_date.is_some() || death_place.is_some();

        let person = TreePersonNodeDto {
            id: new_id.clone(),
            gid,
            sex: sex.trim().to_uppercase(),
            is_living: !has_death,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: given_name.trim().to_string(),
                patronymic: patronymic
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
                surname: surname.trim().to_string(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: None,
            birth: if birth_date.is_some() || birth_place.is_some() {
                Some(VitalEventSummaryDto {
                    date: birth_date.clone(),
                    place: birth_place.clone(),
                })
            } else {
                None
            },
            death: if death_date.is_some() || death_place.is_some() {
                Some(VitalEventSummaryDto {
                    date: death_date.clone(),
                    place: death_place.clone(),
                })
            } else {
                None
            },
            avatar_url: None,
            avatar_media_id: None,
            avatar_crop: None,
            notes: None,
            birth_sort,
            death_sort,
            has_death_event: has_death,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
            generation: None,
            is_direct_lineage: None,
        };
        store.persons.push(person);

        if birth_date.is_some() || birth_place.is_some() {
            let place_dto = if let Some(ref bp) = birth_place {
                if !bp.trim().is_empty() {
                    resolve_or_create_place_chain(store, bp, None, None, None, None).ok()
                } else {
                    None
                }
            } else {
                None
            };
            let existing_ev_gids: HashSet<String> =
                store.events.iter().map(|e| e.gid.clone()).collect();
            let ev_gid = allocate_gid('E', &mut store.next_event_gid_seq, None, &existing_ev_gids);
            store.events.push(LifeEventDto {
                id: Uuid::new_v4().to_string(),
                gid: ev_gid,
                individual_id: Some(new_id.clone()),
                family_id: None,
                event_type: "birth".to_string(),
                custom_type: None,
                date: parsed_birth.clone(),
                date_sort: birth_sort,
                gedcom_date: parsed_birth.as_ref().map(|d| d.to_gedcom()),
                place_id: place_dto.as_ref().map(|p| p.id.clone()),
                place_full_name: place_dto
                    .as_ref()
                    .map(|p| p.full_name.clone())
                    .or(birth_place),
                description: None,
                source_text: None,
                media_ids: vec![],
                is_private: false,
                version: 1,
            });
        }

        if death_date.is_some() || death_place.is_some() {
            let place_dto = if let Some(ref dp) = death_place {
                if !dp.trim().is_empty() {
                    resolve_or_create_place_chain(store, dp, None, None, None, None).ok()
                } else {
                    None
                }
            } else {
                None
            };
            let existing_ev_gids: HashSet<String> =
                store.events.iter().map(|e| e.gid.clone()).collect();
            let ev_gid = allocate_gid('E', &mut store.next_event_gid_seq, None, &existing_ev_gids);
            store.events.push(LifeEventDto {
                id: Uuid::new_v4().to_string(),
                gid: ev_gid,
                individual_id: Some(new_id.clone()),
                family_id: None,
                event_type: "death".to_string(),
                custom_type: None,
                date: parsed_death.clone(),
                date_sort: death_sort,
                gedcom_date: parsed_death.as_ref().map(|d| d.to_gedcom()),
                place_id: place_dto.as_ref().map(|p| p.id.clone()),
                place_full_name: place_dto
                    .as_ref()
                    .map(|p| p.full_name.clone())
                    .or(death_place),
                description: None,
                source_text: None,
                media_ids: vec![],
                is_private: false,
                version: 1,
            });
        }

        Ok(new_id)
    };

    let mut created_new_person = false;
    let rel_person_id: String;
    let family_id: String;
    let child_rel = req.child_relation.unwrap_or_else(|| "birth".to_string());

    match relation.as_str() {
        "father" | "mother" | "parent" => {
            let default_sex = if relation == "father" {
                "M".to_string()
            } else if relation == "mother" {
                "F".to_string()
            } else {
                req.sex.clone().unwrap_or_else(|| "U".to_string())
            };

            let default_surname = req
                .surname
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| anchor.primary_name.surname.clone());

            if let Some(ref existing_ref) = req.existing_person_id {
                rel_person_id = resolve_person_id_in_store(&store.persons, existing_ref)
                    .ok_or(AppError::NotFound)?;
                ensure_no_ancestor_cycle(&store.families, &rel_person_id, &anchor.id)?;
            } else {
                rel_person_id = create_new_relative(
                    &mut store,
                    req.given_name.clone().unwrap_or_default(),
                    req.patronymic.clone(),
                    default_surname,
                    req.sex.clone().unwrap_or(default_sex),
                    req.birth_date.clone(),
                    req.birth_place.clone(),
                    req.death_date.clone(),
                    req.death_place.clone(),
                )?;
                created_new_person = true;
            }

            // Check if anchor already belongs to a parent family with an open partner slot
            let existing_parent_fam_idx = store.families.iter().position(|f| {
                f.children.iter().any(|c| c.child_id == anchor.id)
                    && (f.partner1_id.is_none() || f.partner2_id.is_none())
                    && f.partner1_id.as_deref() != Some(rel_person_id.as_str())
                    && f.partner2_id.as_deref() != Some(rel_person_id.as_str())
            });

            if let Some(f_idx) = existing_parent_fam_idx {
                let child_ids: Vec<String> = store.families[f_idx]
                    .children
                    .iter()
                    .map(|c| c.child_id.clone())
                    .collect();
                for cid in &child_ids {
                    ensure_no_ancestor_cycle(&store.families, &rel_person_id, cid)?;
                }
                if store.families[f_idx].partner1_id.is_none() {
                    store.families[f_idx].partner1_id = Some(rel_person_id.clone());
                } else {
                    store.families[f_idx].partner2_id = Some(rel_person_id.clone());
                }
                store.families[f_idx].version += 1;
                family_id = store.families[f_idx].id.clone();
            } else {
                let existing_fam_gids: HashSet<String> =
                    store.families.iter().map(|f| f.gid.clone()).collect();
                let fam_gid = allocate_gid(
                    'F',
                    &mut store.next_family_gid_seq,
                    None,
                    &existing_fam_gids,
                );
                let new_fam_id = Uuid::new_v4().to_string();
                let fam = FamilyDto {
                    id: new_fam_id.clone(),
                    gid: fam_gid,
                    partner1_id: Some(rel_person_id.clone()),
                    partner2_id: None,
                    relationship_type: req
                        .relationship_type
                        .unwrap_or_else(|| "married".to_string()),
                    is_private: false,
                    children: vec![FamilyChildDto {
                        child_id: anchor.id.clone(),
                        sort_order: 0,
                        rel_to_partner1: child_rel,
                        rel_to_partner2: "birth".to_string(),
                    }],
                    external_ids: HashMap::new(),
                    version: 1,
                    marriage_date: None,
                    marriage_place: None,
                };
                store.families.push(fam);
                family_id = new_fam_id;
            }
        }
        "spouse" | "partner" => {
            let default_sex = match anchor.sex.as_str() {
                "M" => "F".to_string(),
                "F" => "M".to_string(),
                _ => "U".to_string(),
            };
            let default_surname = req
                .surname
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| anchor.primary_name.surname.clone());

            if let Some(ref existing_ref) = req.existing_person_id {
                rel_person_id = resolve_person_id_in_store(&store.persons, existing_ref)
                    .ok_or(AppError::NotFound)?;
                if rel_person_id == anchor.id {
                    let mut fields = HashMap::new();
                    fields.insert("existing_person_id".to_string(), "self_spouse".to_string());
                    return Err(AppError::ValidationFailed(
                        "A person cannot be their own spouse".to_string(),
                        Some(fields),
                    ));
                }
            } else {
                rel_person_id = create_new_relative(
                    &mut store,
                    req.given_name.clone().unwrap_or_default(),
                    req.patronymic.clone(),
                    default_surname,
                    req.sex.clone().unwrap_or(default_sex),
                    req.birth_date.clone(),
                    req.birth_place.clone(),
                    req.death_date.clone(),
                    req.death_place.clone(),
                )?;
                created_new_person = true;
            }

            let existing_fam_gids: HashSet<String> =
                store.families.iter().map(|f| f.gid.clone()).collect();
            let fam_gid = allocate_gid(
                'F',
                &mut store.next_family_gid_seq,
                None,
                &existing_fam_gids,
            );
            let new_fam_id = Uuid::new_v4().to_string();
            let fam = FamilyDto {
                id: new_fam_id.clone(),
                gid: fam_gid,
                partner1_id: Some(anchor.id.clone()),
                partner2_id: Some(rel_person_id.clone()),
                relationship_type: req
                    .relationship_type
                    .unwrap_or_else(|| "married".to_string()),
                is_private: false,
                children: vec![],
                external_ids: HashMap::new(),
                version: 1,
                marriage_date: None,
                marriage_place: None,
            };
            store.families.push(fam);
            family_id = new_fam_id;
        }
        "child" => {
            // Find target family where anchor is a partner, or create one
            let fam_idx = if let Some(ref tf_id) = req.target_family_id {
                let target = tf_id.trim();
                store
                    .families
                    .iter()
                    .position(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
            } else {
                store.families.iter().position(|f| {
                    f.partner1_id.as_deref() == Some(anchor.id.as_str())
                        || f.partner2_id.as_deref() == Some(anchor.id.as_str())
                })
            };

            let resolved_fam_idx = if let Some(idx) = fam_idx {
                idx
            } else {
                let existing_fam_gids: HashSet<String> =
                    store.families.iter().map(|f| f.gid.clone()).collect();
                let fam_gid = allocate_gid(
                    'F',
                    &mut store.next_family_gid_seq,
                    None,
                    &existing_fam_gids,
                );
                let new_fam_id = Uuid::new_v4().to_string();
                store.families.push(FamilyDto {
                    id: new_fam_id,
                    gid: fam_gid,
                    partner1_id: Some(anchor.id.clone()),
                    partner2_id: None,
                    relationship_type: req
                        .relationship_type
                        .unwrap_or_else(|| "married".to_string()),
                    is_private: false,
                    children: vec![],
                    external_ids: HashMap::new(),
                    version: 1,
                    marriage_date: None,
                    marriage_place: None,
                });
                store.families.len() - 1
            };

            let p1_id = store.families[resolved_fam_idx].partner1_id.clone();
            let p2_id = store.families[resolved_fam_idx].partner2_id.clone();

            // Determine father in this family for surname + Slavic patronymic smart defaults (`07 §3 #4`)
            let father_person = [p1_id.as_deref(), p2_id.as_deref()]
                .into_iter()
                .flatten()
                .filter_map(|pid| store.persons.iter().find(|p| p.id == pid))
                .find(|p| p.sex == "M")
                .cloned()
                .or_else(|| {
                    if anchor.sex == "M" {
                        Some(anchor.clone())
                    } else {
                        None
                    }
                });

            let child_sex = req.sex.clone().unwrap_or_else(|| "U".to_string());
            let default_surname = req
                .surname
                .clone()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    father_person
                        .as_ref()
                        .map(|f| f.primary_name.surname.clone())
                })
                .unwrap_or_else(|| anchor.primary_name.surname.clone());

            let default_patronymic = req
                .patronymic
                .clone()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    father_person.as_ref().and_then(|f| {
                        suggest_slavic_patronymic(&f.primary_name.given_name, &child_sex)
                    })
                });

            if let Some(ref existing_ref) = req.existing_person_id {
                rel_person_id = resolve_person_id_in_store(&store.persons, existing_ref)
                    .ok_or(AppError::NotFound)?;
                for parent_id in [p1_id.as_deref(), p2_id.as_deref()].into_iter().flatten() {
                    ensure_no_ancestor_cycle(&store.families, parent_id, &rel_person_id)?;
                }
            } else {
                rel_person_id = create_new_relative(
                    &mut store,
                    req.given_name.clone().unwrap_or_default(),
                    default_patronymic,
                    default_surname,
                    child_sex,
                    req.birth_date.clone(),
                    req.birth_place.clone(),
                    req.death_date.clone(),
                    req.death_place.clone(),
                )?;
                created_new_person = true;
            }

            let fam = &mut store.families[resolved_fam_idx];
            if !fam.children.iter().any(|c| c.child_id == rel_person_id) {
                let next_order = fam.children.len() as i32;
                fam.children.push(FamilyChildDto {
                    child_id: rel_person_id.clone(),
                    sort_order: next_order,
                    rel_to_partner1: child_rel.clone(),
                    rel_to_partner2: child_rel,
                });
                fam.version += 1;
            }
            family_id = fam.id.clone();
        }
        "sibling" => {
            let parent_fam_idx = store
                .families
                .iter()
                .position(|f| f.children.iter().any(|c| c.child_id == anchor.id));

            let resolved_fam_idx = if let Some(idx) = parent_fam_idx {
                idx
            } else {
                let existing_fam_gids: HashSet<String> =
                    store.families.iter().map(|f| f.gid.clone()).collect();
                let fam_gid = allocate_gid(
                    'F',
                    &mut store.next_family_gid_seq,
                    None,
                    &existing_fam_gids,
                );
                let new_fam_id = Uuid::new_v4().to_string();
                store.families.push(FamilyDto {
                    id: new_fam_id,
                    gid: fam_gid,
                    partner1_id: None,
                    partner2_id: None,
                    relationship_type: "unknown".to_string(),
                    is_private: false,
                    children: vec![FamilyChildDto {
                        child_id: anchor.id.clone(),
                        sort_order: 0,
                        rel_to_partner1: "birth".to_string(),
                        rel_to_partner2: "birth".to_string(),
                    }],
                    external_ids: HashMap::new(),
                    version: 1,
                    marriage_date: None,
                    marriage_place: None,
                });
                store.families.len() - 1
            };

            let p1_id = store.families[resolved_fam_idx].partner1_id.clone();
            let p2_id = store.families[resolved_fam_idx].partner2_id.clone();

            let father_person = [p1_id.as_deref(), p2_id.as_deref()]
                .into_iter()
                .flatten()
                .filter_map(|pid| store.persons.iter().find(|p| p.id == pid))
                .find(|p| p.sex == "M")
                .cloned();

            let sibling_sex = req.sex.clone().unwrap_or_else(|| "U".to_string());
            let default_surname = req
                .surname
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| anchor.primary_name.surname.clone());

            let default_patronymic = req
                .patronymic
                .clone()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    father_person.as_ref().and_then(|f| {
                        suggest_slavic_patronymic(&f.primary_name.given_name, &sibling_sex)
                    })
                });

            if let Some(ref existing_ref) = req.existing_person_id {
                rel_person_id = resolve_person_id_in_store(&store.persons, existing_ref)
                    .ok_or(AppError::NotFound)?;
                for parent_id in [p1_id.as_deref(), p2_id.as_deref()].into_iter().flatten() {
                    ensure_no_ancestor_cycle(&store.families, parent_id, &rel_person_id)?;
                }
            } else {
                rel_person_id = create_new_relative(
                    &mut store,
                    req.given_name.clone().unwrap_or_default(),
                    default_patronymic,
                    default_surname,
                    sibling_sex,
                    req.birth_date.clone(),
                    req.birth_place.clone(),
                    req.death_date.clone(),
                    req.death_place.clone(),
                )?;
                created_new_person = true;
            }

            let fam = &mut store.families[resolved_fam_idx];
            if !fam.children.iter().any(|c| c.child_id == rel_person_id) {
                let next_order = fam.children.len() as i32;
                fam.children.push(FamilyChildDto {
                    child_id: rel_person_id.clone(),
                    sort_order: next_order,
                    rel_to_partner1: child_rel.clone(),
                    rel_to_partner2: child_rel,
                });
                fam.version += 1;
            }
            family_id = fam.id.clone();
        }
        _ => unreachable!(),
    }

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let mut person = store
        .persons
        .iter()
        .find(|p| p.id == rel_person_id)
        .cloned()
        .ok_or(AppError::NotFound)?;
    let family = store
        .families
        .iter()
        .find(|f| f.id == family_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let tx_id = Uuid::new_v4().to_string();
    let tx_label = format!("person.quick_add_{relation}");
    let person_label = format_person_display_name(&person);
    if created_new_person {
        log_change(
            &mut store,
            Some(tx_id.clone()),
            &tx_label,
            "individual",
            &person.id,
            "create",
            Some(person_label.clone()),
            format!(
                "Created {} {} ({}) for {}",
                relation, person.gid, person_label, anchor.gid
            ),
            None,
            serde_json::to_value(&person).ok(),
            Some(editor.user.id.to_string()),
            Some(editor.user.display_name()),
        );
    }
    let entry = log_change(
        &mut store,
        Some(tx_id),
        &tx_label,
        "family",
        &family.id,
        "create",
        Some(family.gid.clone()),
        format!(
            "Linked {} {} in family {}",
            relation, person.gid, family.gid
        ),
        None,
        serde_json::to_value(&family).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );
    person.tx_id = Some(entry.tx_id.clone());

    Ok((
        StatusCode::CREATED,
        Json(QuickAddRelativeResponse {
            person,
            family,
            created_new_person,
            tx_id: Some(entry.tx_id),
        }),
    ))
}
