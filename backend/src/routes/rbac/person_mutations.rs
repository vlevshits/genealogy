#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    post,
    path = "/api/v1/persons",
    request_body = CreatePersonRequest,
    responses(
        (status = 201, description = "Person created by Editor or Admin", body = TreePersonNodeDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Viewer role is read-only", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_person(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreatePersonRequest>,
) -> Result<(StatusCode, Json<TreePersonNodeDto>), AppError> {
    if req.given_name.trim().is_empty() && req.surname.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("given_name".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "At least a given name or surname is required".to_string(),
            Some(fields),
        ));
    }

    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let was_empty = store.persons.is_empty();
    store.is_demo_seed = false;

    let existing_gids: HashSet<String> = store.persons.iter().map(|p| p.gid.clone()).collect();
    let gid = allocate_gid(
        'I',
        &mut store.next_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );
    let person_id = Uuid::new_v4().to_string();

    let birth_date_str = req
        .birth_date
        .clone()
        .or_else(|| req.birth_year.map(|y| y.to_string()));

    let mut birth_sort = req.birth_year.map(|y| y * 10_000 + 101);
    if let Some(ref b_raw) = birth_date_str {
        let parsed = GenDate::parse(b_raw);
        if let Some(sk) = parsed.sort_key() {
            birth_sort = Some(sk);
        }
    }

    let mut death_sort = None;
    if let Some(ref d_raw) = req.death_date {
        let parsed = GenDate::parse(d_raw);
        death_sort = parsed.sort_key();
    }

    let has_death = req.has_death_or_burial_event.unwrap_or(false)
        || req.death_date.is_some()
        || req.death_place.is_some();

    let eval_input = PersonLivingEvaluationInput {
        living_override: req.living_override,
        has_death_or_burial_event: has_death,
        birth_year: birth_sort.map(|bs| bs / 10_000).or(req.birth_year),
        ..Default::default()
    };
    let is_living = evaluate_is_living(&eval_input, current_year, age_threshold);

    let birth = if birth_date_str.is_some() || req.birth_place.is_some() {
        Some(VitalEventSummaryDto {
            date: birth_date_str.clone(),
            place: req.birth_place.clone(),
        })
    } else {
        None
    };

    let death = if req.death_date.is_some() || req.death_place.is_some() {
        Some(VitalEventSummaryDto {
            date: req.death_date.clone(),
            place: req.death_place.clone(),
        })
    } else {
        None
    };

    let alt_names = req
        .alt_names
        .unwrap_or_default()
        .into_iter()
        .map(|mut a| {
            if a.id.is_empty() {
                a.id = Uuid::new_v4().to_string();
            }
            a
        })
        .collect();

    let node = TreePersonNodeDto {
        id: person_id.clone(),
        gid,
        sex: normalize_sex(req.sex.as_deref().unwrap_or("U")),
        is_living,
        living_override: req.living_override,
        is_private: req.is_private.unwrap_or(false),
        primary_name: PersonNameDto {
            given_name: req.given_name.trim().to_string(),
            patronymic: req
                .patronymic
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            surname: req.surname.trim().to_string(),
        },
        birth_surname: req
            .birth_surname
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        nickname: req
            .nickname
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        name_prefix: req
            .name_prefix
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        name_suffix: req
            .name_suffix
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        biography: req.biography,
        birth,
        death,
        avatar_url: None,
        avatar_media_id: None,
        avatar_crop: None,
        notes: req.notes,
        birth_sort,
        death_sort,
        has_death_event: has_death,
        alt_names,
        external_ids: HashMap::new(),
        version: 1,
        tx_id: None,
        generation: None,
        is_direct_lineage: None,
    };

    store.persons.push(node);

    // Create inline birth event if birth_date/birth_place provided
    if birth_date_str.is_some() || req.birth_place.is_some() {
        let place_dto = if let Some(ref bp) = req.birth_place {
            if !bp.trim().is_empty() {
                resolve_or_create_place_chain(&mut store, bp, None, None, None, None).ok()
            } else {
                None
            }
        } else {
            None
        };
        let parsed_date = birth_date_str.as_deref().map(GenDate::parse);
        let date_sort_val = parsed_date.as_ref().and_then(|d| d.sort_key());
        let gedcom_str = parsed_date.as_ref().map(|d| d.to_gedcom());
        let existing_ev_gids: HashSet<String> =
            store.events.iter().map(|e| e.gid.clone()).collect();
        let ev_gid = allocate_gid('E', &mut store.next_event_gid_seq, None, &existing_ev_gids);
        store.events.push(LifeEventDto {
            id: Uuid::new_v4().to_string(),
            gid: ev_gid,
            individual_id: Some(person_id.clone()),
            family_id: None,
            event_type: "birth".to_string(),
            custom_type: None,
            date: parsed_date,
            date_sort: date_sort_val,
            gedcom_date: gedcom_str,
            place_id: place_dto.as_ref().map(|p| p.id.clone()),
            place_full_name: place_dto
                .as_ref()
                .map(|p| p.full_name.clone())
                .or(req.birth_place),
            description: None,
            source_text: None,
            media_ids: vec![],
            is_private: false,
            version: 1,
        });
    }

    if req.death_date.is_some() || req.death_place.is_some() {
        let place_dto = if let Some(ref dp) = req.death_place {
            if !dp.trim().is_empty() {
                resolve_or_create_place_chain(&mut store, dp, None, None, None, None).ok()
            } else {
                None
            }
        } else {
            None
        };
        let parsed_date = req.death_date.as_deref().map(GenDate::parse);
        let date_sort_val = parsed_date.as_ref().and_then(|d| d.sort_key());
        let gedcom_str = parsed_date.as_ref().map(|d| d.to_gedcom());
        let existing_ev_gids: HashSet<String> =
            store.events.iter().map(|e| e.gid.clone()).collect();
        let ev_gid = allocate_gid('E', &mut store.next_event_gid_seq, None, &existing_ev_gids);
        store.events.push(LifeEventDto {
            id: Uuid::new_v4().to_string(),
            gid: ev_gid,
            individual_id: Some(person_id.clone()),
            family_id: None,
            event_type: "death".to_string(),
            custom_type: None,
            date: parsed_date,
            date_sort: date_sort_val,
            gedcom_date: gedcom_str,
            place_id: place_dto.as_ref().map(|p| p.id.clone()),
            place_full_name: place_dto
                .as_ref()
                .map(|p| p.full_name.clone())
                .or(req.death_place),
            description: None,
            source_text: None,
            media_ids: vec![],
            is_private: false,
            version: 1,
        });
    }

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let mut created = store
        .persons
        .iter()
        .find(|p| p.id == person_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let entity_label = format_person_display_name(&created);
    let entry = log_change(
        &mut store,
        None,
        "person.create",
        "individual",
        &created.id,
        "create",
        Some(entity_label.clone()),
        format!("Created individual {} ({})", created.gid, entity_label),
        None,
        serde_json::to_value(&created).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );
    created.tx_id = Some(entry.tx_id);
    drop(store);

    if was_empty {
        state.set_default_home_person_if_unset(&person_id).await;
    }

    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/persons/{id}",
    params(
        ("id" = String, Path, description = "Person UUID or GID")
    ),
    request_body = UpdatePersonRequest,
    responses(
        (status = 200, description = "Person updated by Editor or Admin", body = TreePersonNodeDto),
        (status = 403, description = "Viewer role is read-only", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_person(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdatePersonRequest>,
) -> Result<Json<TreePersonNodeDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let (person_id, before_json) = {
        let person = store
            .persons
            .iter_mut()
            .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
            .ok_or(AppError::NotFound)?;

        let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
        verify_optimistic_version(
            person.version,
            expected_version,
            &format!("Individual {}", person.gid),
            serde_json::to_value(&*person).unwrap_or_default(),
        )?;

        let before_snapshot = serde_json::to_value(&*person).ok();

        if let Some(given) = req.given_name {
            person.primary_name.given_name = given.trim().to_string();
        }
        if let Some(patronymic) = req.patronymic {
            let trimmed = patronymic.trim().to_string();
            person.primary_name.patronymic = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
        if let Some(surname) = req.surname {
            person.primary_name.surname = surname.trim().to_string();
        }
        if let Some(bs) = req.birth_surname {
            let trimmed = bs.trim().to_string();
            person.birth_surname = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
        if let Some(nick) = req.nickname {
            let trimmed = nick.trim().to_string();
            person.nickname = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
        if let Some(pref) = req.name_prefix {
            let trimmed = pref.trim().to_string();
            person.name_prefix = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
        if let Some(suff) = req.name_suffix {
            let trimmed = suff.trim().to_string();
            person.name_suffix = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
        if let Some(sex) = req.sex {
            person.sex = normalize_sex(&sex);
        }
        if let Some(bio) = req.biography {
            person.biography = Some(bio);
        }
        if let Some(is_priv) = req.is_private {
            person.is_private = is_priv;
        }
        if req.clear_living_override == Some(true) {
            person.living_override = None;
        } else if let Some(override_val) = req.living_override {
            person.living_override = Some(override_val);
            person.is_living = override_val;
        }
        if let Some(notes) = req.notes {
            person.notes = Some(notes);
        }
        if let Some(alts) = req.alt_names {
            person.alt_names = alts
                .into_iter()
                .map(|mut a| {
                    if a.id.is_empty() {
                        a.id = Uuid::new_v4().to_string();
                    }
                    a
                })
                .collect();
        }

        person.version += 1;
        (person.id.clone(), before_snapshot)
    };

    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let mut updated = store
        .persons
        .iter()
        .find(|p| p.id == person_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let entity_label = format_person_display_name(&updated);
    let entry = log_change(
        &mut store,
        None,
        "person.update",
        "individual",
        &updated.id,
        "update",
        Some(entity_label.clone()),
        format!("Updated individual {} ({})", updated.gid, entity_label),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );
    updated.tx_id = Some(entry.tx_id);

    Ok(Json(updated))
}
