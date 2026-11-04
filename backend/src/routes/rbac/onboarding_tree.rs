#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/onboarding/status",
    responses(
        (status = 200, description = "Tree onboarding state and resolved home person for the current caller", body = OnboardingStatusResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_onboarding_status(
    caller: PublicOrAuth,
    State(state): State<AppState>,
) -> Result<Json<OnboardingStatusResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    {
        let mut store = state.privacy_store.write().await;
        if store.is_demo_seed {
            store.persons.clear();
            store.is_demo_seed = false;
        }
    }
    let store = state.privacy_store.read().await;
    let total_individuals = store.persons.len();
    let is_empty = total_individuals == 0;
    let can_onboard = is_empty && access.can_edit();

    let (resolved_home_person_id, user_home_person_id, user_person_id, start_page) =
        resolve_home_person_for_caller(
            &state,
            &caller,
            &store.persons,
            &settings.default_locale,
            settings.default_home_person_id.as_deref(),
        )
        .await;

    Ok(Json(OnboardingStatusResponse {
        is_empty,
        total_individuals,
        can_onboard,
        resolved_home_person_id,
        default_home_person_id: settings.default_home_person_id,
        user_home_person_id,
        user_person_id,
        start_page,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/start-with-self",
    request_body = StartWithSelfRequest,
    responses(
        (status = 201, description = "Root self individual created and linked to caller's user profile and home person", body = StartWithSelfResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Viewer role is read-only", body = ErrorResponse),
        (status = 409, description = "Tree already onboarded (non-empty)", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn start_with_self(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<StartWithSelfRequest>,
) -> Result<(StatusCode, Json<StartWithSelfResponse>), AppError> {
    if req.given_name.trim().is_empty() && req.surname.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("given_name".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "At least a given name or surname is required".to_string(),
            Some(fields),
        ));
    }

    let sex = match req
        .sex
        .as_deref()
        .unwrap_or("U")
        .trim()
        .to_uppercase()
        .as_str()
    {
        "M" | "MALE" => "M".to_string(),
        "F" | "FEMALE" => "F".to_string(),
        "X" | "OTHER" => "X".to_string(),
        "U" | "UNKNOWN" | "" => "U".to_string(),
        _ => {
            let mut fields = HashMap::new();
            fields.insert("sex".to_string(), "invalid_sex".to_string());
            return Err(AppError::ValidationFailed(
                "Sex must be M, F, X, or U".to_string(),
                Some(fields),
            ));
        }
    };

    let birth_date_clean = req
        .birth_date
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let birth_place_clean = req
        .birth_place
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let (created_person, created_birth_event, newly_created_places, change_entry, person_uuid) = {
        let mut store = state.privacy_store.write().await;
        if store.is_demo_seed {
            store.persons.clear();
            store.is_demo_seed = false;
        }

        if !store.persons.is_empty() && !editor.access().is_admin() {
            return Err(AppError::AlreadyOnboarded);
        }

        let existing_gids: HashSet<String> = store.persons.iter().map(|p| p.gid.clone()).collect();
        let gid = allocate_gid('I', &mut store.next_gid_seq, None, &existing_gids);
        let person_uuid = Uuid::new_v4();
        let person_id = person_uuid.to_string();

        let places_before: HashSet<String> = store.places.iter().map(|p| p.id.clone()).collect();
        let place_dto = if let Some(ref bp) = birth_place_clean {
            resolve_or_create_place_chain(&mut store, bp, None, None, None, None).ok()
        } else {
            None
        };
        let newly_created_places: Vec<PlaceSummaryDto> = store
            .places
            .iter()
            .filter(|p| !places_before.contains(&p.id))
            .cloned()
            .collect();

        let parsed_birth_date = birth_date_clean.as_deref().map(GenDate::parse);
        let birth_sort = parsed_birth_date.as_ref().and_then(|d| d.sort_key());
        let gedcom_date = parsed_birth_date.as_ref().map(|d| d.to_gedcom());

        let birth_summary = if birth_date_clean.is_some() || birth_place_clean.is_some() {
            Some(VitalEventSummaryDto {
                date: birth_date_clean.clone(),
                place: place_dto
                    .as_ref()
                    .map(|p| p.full_name.clone())
                    .or_else(|| birth_place_clean.clone()),
            })
        } else {
            None
        };

        let patronymic_clean = req
            .patronymic
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let node = TreePersonNodeDto {
            id: person_id.clone(),
            gid: gid.clone(),
            sex: sex.clone(),
            is_living: true,
            living_override: Some(true),
            is_private: false,
            primary_name: PersonNameDto {
                given_name: req.given_name.trim().to_string(),
                patronymic: patronymic_clean,
                surname: req.surname.trim().to_string(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: None,
            birth: birth_summary,
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

        store.persons.push(node);

        let created_birth_event = if birth_date_clean.is_some() || birth_place_clean.is_some() {
            let existing_ev_gids: HashSet<String> =
                store.events.iter().map(|e| e.gid.clone()).collect();
            let ev_gid = allocate_gid('E', &mut store.next_event_gid_seq, None, &existing_ev_gids);
            let ev = LifeEventDto {
                id: Uuid::new_v4().to_string(),
                gid: ev_gid,
                individual_id: Some(person_id.clone()),
                family_id: None,
                event_type: "birth".to_string(),
                custom_type: None,
                date: parsed_birth_date,
                date_sort: birth_sort,
                gedcom_date,
                place_id: place_dto.as_ref().map(|p| p.id.clone()),
                place_full_name: place_dto
                    .as_ref()
                    .map(|p| p.full_name.clone())
                    .or(birth_place_clean),
                description: None,
                source_text: None,
                media_ids: vec![],
                is_private: false,
                version: 1,
            };
            store.events.push(ev.clone());
            Some(ev)
        } else {
            None
        };

        recompute_all_places_metadata(&mut store);
        recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

        let mut created_person = store
            .persons
            .iter()
            .find(|p| p.id == person_id)
            .cloned()
            .ok_or(AppError::NotFound)?;

        let summary = format!(
            "Created root self individual {} ({} {})",
            created_person.gid,
            created_person.primary_name.given_name,
            created_person.primary_name.surname
        )
        .trim()
        .to_string();

        let entity_label = format_person_display_name(&created_person);
        let change_entry = log_change(
            &mut store,
            None,
            "tree.start_with_self",
            "individual",
            &person_id,
            "create",
            Some(entity_label),
            summary,
            None,
            serde_json::to_value(&created_person).ok(),
            Some(editor.user.id.to_string()),
            Some(editor.user.display_name()),
        );
        created_person.tx_id = Some(change_entry.tx_id.clone());

        (
            created_person,
            created_birth_event,
            newly_created_places,
            change_entry,
            person_uuid,
        )
    };

    let updated_user = {
        let mut auth_store = state.auth_store.write().await;
        let u = auth_store
            .users
            .iter_mut()
            .find(|u| u.id == editor.user.id)
            .ok_or(AppError::NotAuthenticated)?;
        u.person_id = Some(person_uuid);
        u.home_person_id = Some(person_uuid);
        u.updated_at = Utc::now();
        u.clone()
    };

    state
        .set_default_home_person_if_unset(&created_person.id)
        .await;

    if let Some(pool) = &state.db {
        if let Ok(mut tx) = pool.begin().await {
            for place in &newly_created_places {
                let place_uuid = Uuid::parse_str(&place.id).unwrap_or_else(|_| Uuid::new_v4());
                let parent_uuid = place
                    .parent_id
                    .as_deref()
                    .and_then(|pid| Uuid::parse_str(pid).ok());
                let _ = sqlx::query(
                    "INSERT INTO places (id, gid, name, place_type, parent_id, latitude, longitude, full_name, version, created_by, updated_by)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
                     ON CONFLICT (id) DO NOTHING",
                )
                .bind(place_uuid)
                .bind(&place.gid)
                .bind(&place.name)
                .bind(&place.place_type)
                .bind(parent_uuid)
                .bind(place.latitude)
                .bind(place.longitude)
                .bind(&place.full_name)
                .bind(place.version)
                .bind(editor.user.id)
                .execute(&mut *tx)
                .await;
            }

            let db_sex = match created_person.sex.as_str() {
                "X" => "other",
                s => s,
            };
            let search_tokens = normalize_search_tokens(&format!(
                "{} {} {}",
                created_person.primary_name.given_name,
                created_person
                    .primary_name
                    .patronymic
                    .as_deref()
                    .unwrap_or(""),
                created_person.primary_name.surname
            ));
            let _ = sqlx::query(
                "INSERT INTO individuals (id, gid, given_name, patronymic, surname, sex, living_override, is_private, birth_sort, has_death_event, search_text, version, created_by, updated_by)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, FALSE, $10, 1, $11, $11)",
            )
            .bind(person_uuid)
            .bind(&created_person.gid)
            .bind(&created_person.primary_name.given_name)
            .bind(&created_person.primary_name.patronymic)
            .bind(&created_person.primary_name.surname)
            .bind(db_sex)
            .bind(created_person.living_override)
            .bind(created_person.is_private)
            .bind(created_person.birth_sort)
            .bind(search_tokens)
            .bind(editor.user.id)
            .execute(&mut *tx)
            .await;

            if let Some(ref ev) = created_birth_event {
                let ev_uuid = Uuid::parse_str(&ev.id).unwrap_or_else(|_| Uuid::new_v4());
                let ev_place_uuid = ev
                    .place_id
                    .as_deref()
                    .and_then(|pid| Uuid::parse_str(pid).ok());
                let ev_date_json = ev.date.as_ref().and_then(|d| serde_json::to_value(d).ok());
                let _ = sqlx::query(
                    "INSERT INTO life_events (id, gid, individual_id, event_type, date, date_sort, place_id, is_private, version, created_by, updated_by)
                     VALUES ($1, $2, $3, 'birth', $4, $5, $6, FALSE, 1, $7, $7)",
                )
                .bind(ev_uuid)
                .bind(&ev.gid)
                .bind(person_uuid)
                .bind(ev_date_json)
                .bind(ev.date_sort)
                .bind(ev_place_uuid)
                .bind(editor.user.id)
                .execute(&mut *tx)
                .await;
            }

            let _ = sqlx::query(
                "UPDATE users SET person_id = $2, home_person_id = $2, updated_at = NOW() WHERE id = $1",
            )
            .bind(editor.user.id)
            .bind(person_uuid)
            .execute(&mut *tx)
            .await;

            let log_uuid = Uuid::parse_str(&change_entry.id).unwrap_or_else(|_| Uuid::new_v4());
            let _ = sqlx::query(
                "INSERT INTO change_log (id, actor_id, op, entity_type, entity_id, summary, after_json)
                 VALUES ($1, $2, 'create', 'individual', $3, $4, $5)",
            )
            .bind(log_uuid)
            .bind(editor.user.id)
            .bind(person_uuid)
            .bind(&change_entry.summary)
            .bind(&change_entry.after_json)
            .execute(&mut *tx)
            .await;

            let _ = tx.commit().await;
        }
    }

    let user_dto = AuthService::build_user_dto(&state, &updated_user).await;

    Ok((
        StatusCode::CREATED,
        Json(StartWithSelfResponse {
            individual: created_person,
            user: user_dto,
        }),
    ))
}

pub(crate) fn filter_family_for_visible_persons(
    fam: &FamilyDto,
    can_view_private: bool,
    visible_person_ids: &HashSet<String>,
) -> Option<FamilyDto> {
    if fam.is_private && !can_view_private {
        return None;
    }
    let partner1_id = fam
        .partner1_id
        .as_ref()
        .filter(|id| visible_person_ids.contains(*id))
        .cloned();
    let partner2_id = fam
        .partner2_id
        .as_ref()
        .filter(|id| visible_person_ids.contains(*id))
        .cloned();
    let children: Vec<FamilyChildDto> = fam
        .children
        .iter()
        .filter(|c| visible_person_ids.contains(&c.child_id))
        .cloned()
        .collect();

    if partner1_id.is_none() && partner2_id.is_none() && children.is_empty() {
        return None;
    }

    Some(FamilyDto {
        id: fam.id.clone(),
        gid: fam.gid.clone(),
        partner1_id,
        partner2_id,
        relationship_type: fam.relationship_type.clone(),
        is_private: fam.is_private,
        children,
        external_ids: fam.external_ids.clone(),
        version: fam.version,
        marriage_date: fam.marriage_date.clone(),
        marriage_place: fam.marriage_place.clone(),
    })
}

pub(crate) fn extract_person_birth_year(p: &TreePersonNodeDto) -> Option<i32> {
    if let Some(bs) = p.birth_sort {
        let year = bs / 10_000;
        if year > 0 {
            return Some(year);
        }
    }
    if let Some(ref birth) = p.birth {
        if let Some(ref d) = birth.date {
            for token in d.split(|c: char| !c.is_ascii_digit()) {
                if token.len() == 4 {
                    if let Ok(y) = token.parse::<i32>() {
                        return Some(y);
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn compute_median_birth_year(persons: &[&TreePersonNodeDto]) -> Option<f64> {
    let mut years: Vec<i32> = persons
        .iter()
        .filter_map(|p| extract_person_birth_year(p))
        .collect();
    if years.is_empty() {
        return None;
    }
    years.sort_unstable();
    let n = years.len();
    if n % 2 == 1 {
        Some(f64::from(years[n / 2]))
    } else {
        Some(f64::from(years[n / 2 - 1] + years[n / 2]) / 2.0)
    }
}

pub(crate) struct TreeDsu {
    pub(crate) parent: HashMap<String, String>,
}

impl TreeDsu {
    pub(crate) fn new(ids: impl IntoIterator<Item = String>) -> Self {
        let mut parent = HashMap::new();
        for id in ids {
            parent.insert(id.clone(), id);
        }
        Self { parent }
    }

    pub(crate) fn find(&mut self, x: &str) -> String {
        let p = self.parent.get(x).cloned().unwrap_or_else(|| x.to_string());
        if p == x {
            p
        } else {
            let root = self.find(&p);
            self.parent.insert(x.to_string(), root.clone());
            root
        }
    }

    pub(crate) fn union(&mut self, x: &str, y: &str) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x != root_y {
            self.parent.insert(root_x, root_y);
        }
    }
}

#[utoipa::path(

    get,
    path = "/api/v1/tree",
    responses(
        (status = 200, description = "Interactive tree nodes and families filtered and redacted per caller RBAC tier", body = TreeResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_tree(
    caller: PublicOrAuth,
    State(state): State<AppState>,
) -> Result<Json<TreeResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let nodes: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();
    let visible_person_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter_map(|f| {
            filter_family_for_visible_persons(f, access.can_view_private(), &visible_person_ids)
        })
        .collect();
    let total_visible = nodes.len();

    let (resolved_home_person_id, _, _, _) = resolve_home_person_for_caller(
        &state,
        &caller,
        &store.persons,
        &settings.default_locale,
        settings.default_home_person_id.as_deref(),
    )
    .await;

    Ok(Json(TreeResponse {
        access_tier: access.as_tier_str().to_string(),
        tree_title: settings.tree_title,
        nodes,
        families,
        total_visible,
        resolved_home_person_id,
    }))
}
