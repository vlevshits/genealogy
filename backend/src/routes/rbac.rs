use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    error::{AppError, ErrorResponse},
    models::gendate::GenDate,
    services::{
        genealogy::{
            allocate_gid, build_place_breadcrumbs, ensure_no_ancestor_cycle, ensure_no_place_cycle,
            recompute_all_individuals_vitals, recompute_all_places_metadata,
            resolve_or_create_place_chain,
        },
        privacy::{
            evaluate_is_living, normalize_search_tokens, redact_media_for_caller,
            redact_person_for_caller, suggest_slavic_patronymic, ChangeLogEntryDto, FamilyChildDto,
            FamilyDto, IndividualAltNameDto, LifeEventDto, MediaAssetSummaryDto, NoteDto,
            PersonLivingEvaluationInput, PersonNameDto, PlaceAltNameDto, PlaceBreadcrumbDto,
            PlaceSummaryDto, TreePersonNodeDto, VitalEventSummaryDto,
        },
        rbac::{PublicOrAuth, RequireAdmin, RequireAuth, RequireEditor},
    },
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TreeResponse {
    pub access_tier: String,
    pub tree_title: String,
    pub nodes: Vec<TreePersonNodeDto>,
    pub total_visible: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreatePersonRequest {
    pub gid: Option<String>,
    #[serde(default)]
    pub given_name: String,
    pub patronymic: Option<String>,
    #[serde(default)]
    pub surname: String,
    pub birth_surname: Option<String>,
    pub nickname: Option<String>,
    pub name_prefix: Option<String>,
    pub name_suffix: Option<String>,
    pub sex: Option<String>,
    pub birth_year: Option<i32>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
    pub death_date: Option<String>,
    pub death_place: Option<String>,
    pub has_death_or_burial_event: Option<bool>,
    pub living_override: Option<bool>,
    pub biography: Option<String>,
    pub is_private: Option<bool>,
    pub notes: Option<String>,
    pub alt_names: Option<Vec<IndividualAltNameDto>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdatePersonRequest {
    pub given_name: Option<String>,
    pub patronymic: Option<String>,
    pub surname: Option<String>,
    pub birth_surname: Option<String>,
    pub nickname: Option<String>,
    pub name_prefix: Option<String>,
    pub name_suffix: Option<String>,
    pub sex: Option<String>,
    pub biography: Option<String>,
    pub is_private: Option<bool>,
    pub living_override: Option<bool>,
    pub clear_living_override: Option<bool>,
    pub notes: Option<String>,
    pub alt_names: Option<Vec<IndividualAltNameDto>>,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
pub struct PatronymicSuggestionQuery {
    pub father_given_name: String,
    pub sex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PatronymicSuggestionResponse {
    pub patronymic: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct QuickAddRelativeRequest {
    /// One of: `"father"`, `"mother"`, `"parent"`, `"spouse"`, `"partner"`, `"child"`, `"sibling"`
    pub relation: String,
    /// Optional ID or GID of an existing person to link instead of creating a new record
    pub existing_person_id: Option<String>,
    pub given_name: Option<String>,
    pub patronymic: Option<String>,
    pub surname: Option<String>,
    pub sex: Option<String>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
    pub death_date: Option<String>,
    pub death_place: Option<String>,
    pub relationship_type: Option<String>,
    pub child_relation: Option<String>,
    pub target_family_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuickAddRelativeResponse {
    pub person: TreePersonNodeDto,
    pub family: FamilyDto,
    pub created_new_person: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeleteImpactDto {
    pub person_id: String,
    pub gid: String,
    pub display_name: String,
    pub events_count: usize,
    pub families_unlinked_count: usize,
    pub empty_families_deleted_count: usize,
    pub notes_count: usize,
    pub media_links_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeletedPersonSnapshot {
    person: TreePersonNodeDto,
    events: Vec<LifeEventDto>,
    notes: Vec<NoteDto>,
    families_before: Vec<FamilyDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreateFamilyRequest {
    pub gid: Option<String>,
    pub partner1_id: Option<String>,
    pub partner2_id: Option<String>,
    pub relationship_type: Option<String>,
    pub is_private: Option<bool>,
    pub children: Option<Vec<FamilyChildDto>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateFamilyRequest {
    pub partner1_id: Option<String>,
    pub partner2_id: Option<String>,
    pub relationship_type: Option<String>,
    pub is_private: Option<bool>,
    pub children: Option<Vec<FamilyChildDto>>,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FamilyListResponse {
    pub items: Vec<FamilyDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct EventFilterQuery {
    pub individual_id: Option<String>,
    pub family_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreateEventRequest {
    pub gid: Option<String>,
    pub individual_id: Option<String>,
    pub family_id: Option<String>,
    pub event_type: String,
    pub custom_type: Option<String>,
    pub date_text: Option<String>,
    pub date: Option<GenDate>,
    pub place_id: Option<String>,
    pub place_path: Option<String>,
    pub description: Option<String>,
    pub source_text: Option<String>,
    pub media_ids: Option<Vec<String>>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateEventRequest {
    pub event_type: Option<String>,
    pub custom_type: Option<String>,
    pub date_text: Option<String>,
    pub date: Option<GenDate>,
    pub place_id: Option<String>,
    pub place_path: Option<String>,
    pub description: Option<String>,
    pub source_text: Option<String>,
    pub media_ids: Option<Vec<String>>,
    pub is_private: Option<bool>,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventListResponse {
    pub items: Vec<LifeEventDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct SourceSearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SourceAutocompleteResponse {
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct NoteFilterQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateNoteRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub body: String,
    pub is_private: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateNoteRequest {
    pub body: Option<String>,
    pub is_private: Option<bool>,
    pub sort_order: Option<i32>,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NoteListResponse {
    pub items: Vec<NoteDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct PlacesFilterQuery {
    pub q: Option<String>,
    pub parent_id: Option<String>,
    pub missing_coords: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreatePlaceRequest {
    pub gid: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub place_type: Option<String>,
    pub parent_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub alt_names: Option<Vec<PlaceAltNameDto>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdatePlaceRequest {
    pub name: Option<String>,
    pub place_type: Option<String>,
    pub parent_id: Option<String>,
    pub clear_parent: Option<bool>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub alt_names: Option<Vec<PlaceAltNameDto>>,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct MergePlaceRequest {
    pub other_id: Option<String>,
    pub source_place_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaceDetailDto {
    pub place: PlaceSummaryDto,
    pub breadcrumbs: Vec<PlaceBreadcrumbDto>,
    pub children: Vec<PlaceSummaryDto>,
    pub events: Vec<LifeEventDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PeopleDirectoryResponse {
    pub items: Vec<TreePersonNodeDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlacesDirectoryResponse {
    pub items: Vec<PlaceSummaryDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaArchiveResponse {
    pub items: Vec<MediaAssetSummaryDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminToolOperationResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ChangeLogEntryDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PersonMergeRequest {
    pub primary_person_id: Option<String>,
    pub duplicate_person_id: Option<String>,
}

fn resolve_person_id_in_store(persons: &[TreePersonNodeDto], id_or_gid: &str) -> Option<String> {
    let target = id_or_gid.trim();
    persons
        .iter()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .map(|p| p.id.clone())
}

fn resolve_place_id_in_store(places: &[PlaceSummaryDto], id_or_gid: &str) -> Option<String> {
    let target = id_or_gid.trim();
    places
        .iter()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .map(|p| p.id.clone())
}

fn is_family_only_event_type(event_type: &str) -> bool {
    matches!(
        event_type.trim().to_ascii_lowercase().as_str(),
        "engagement" | "marriage" | "marriage_license" | "divorce"
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/tree",
    responses(
        (status = 200, description = "Interactive tree nodes filtered and redacted per caller RBAC tier", body = TreeResponse),
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
    let total_visible = nodes.len();

    Ok(Json(TreeResponse {
        access_tier: access.as_tier_str().to_string(),
        tree_title: settings.tree_title,
        nodes,
        total_visible,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/persons/suggest-patronymic",
    params(PatronymicSuggestionQuery),
    responses(
        (status = 200, description = "Suggested Slavic patronymic for child", body = PatronymicSuggestionResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_patronymic_suggestion(
    RequireAuth(_auth): RequireAuth,
    Query(query): Query<PatronymicSuggestionQuery>,
) -> Result<Json<PatronymicSuggestionResponse>, AppError> {
    let patronymic = suggest_slavic_patronymic(&query.father_given_name, &query.sex);
    Ok(Json(PatronymicSuggestionResponse { patronymic }))
}

#[utoipa::path(
    get,
    path = "/api/v1/persons/{id}",
    params(
        ("id" = String, Path, description = "Person UUID or GEDCOM-style GID (e.g. I0042)")
    ),
    responses(
        (status = 200, description = "Person record redacted per caller RBAC tier", body = TreePersonNodeDto),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse),
        (status = 404, description = "Person not found or private", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_person(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TreePersonNodeDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let person = store
        .persons
        .iter()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let redacted = redact_person_for_caller(person, &access, &settings.default_locale)
        .ok_or(AppError::NotFound)?;

    Ok(Json(redacted))
}

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
    RequireEditor(_editor): RequireEditor,
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
        sex: req
            .sex
            .unwrap_or_else(|| "U".to_string())
            .trim()
            .to_uppercase(),
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
        notes: req.notes,
        birth_sort,
        death_sort,
        has_death_event: has_death,
        alt_names,
        version: 1,
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

    let created = store
        .persons
        .iter()
        .find(|p| p.id == person_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

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
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePersonRequest>,
) -> Result<Json<TreePersonNodeDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let person_id = {
        let person = store
            .persons
            .iter_mut()
            .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
            .ok_or(AppError::NotFound)?;

        if let Some(expected_version) = req.version {
            if expected_version != person.version {
                return Err(AppError::VersionConflict(
                    "Record was modified by another session".to_string(),
                ));
            }
        }

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
            person.sex = sex.trim().to_uppercase();
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
        person.id.clone()
    };

    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let updated = store
        .persons
        .iter()
        .find(|p| p.id == person_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    Ok(Json(updated))
}

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
    RequireEditor(_editor): RequireEditor,
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
            notes: None,
            birth_sort,
            death_sort,
            has_death_event: has_death,
            alt_names: vec![],
            version: 1,
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
                    version: 1,
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
                version: 1,
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
                    version: 1,
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
                    version: 1,
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

    let person = store
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

    Ok((
        StatusCode::CREATED,
        Json(QuickAddRelativeResponse {
            person,
            family,
            created_new_person,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/persons/{id}/delete-impact",
    params(
        ("id" = String, Path, description = "Person UUID or GID")
    ),
    responses(
        (status = 200, description = "Preview of records affected by deleting this person", body = DeleteImpactDto),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_person_delete_impact(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteImpactDto>, AppError> {
    let store = state.privacy_store.read().await;
    let target = id.trim();
    let person = store
        .persons
        .iter()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let events_count = store
        .events
        .iter()
        .filter(|e| e.individual_id.as_deref() == Some(person.id.as_str()))
        .count();

    let notes_count = store
        .notes
        .iter()
        .filter(|n| n.entity_type == "individual" && n.entity_id == person.id)
        .count();

    let media_links_count = store
        .media
        .iter()
        .filter(|m| m.linked_person_ids.contains(&person.id))
        .count();

    let mut families_unlinked_count = 0usize;
    let mut empty_families_deleted_count = 0usize;

    for fam in &store.families {
        let is_p1 = fam.partner1_id.as_deref() == Some(person.id.as_str());
        let is_p2 = fam.partner2_id.as_deref() == Some(person.id.as_str());
        let is_child = fam.children.iter().any(|c| c.child_id == person.id);
        if is_p1 || is_p2 || is_child {
            families_unlinked_count += 1;
            let rem_p1 = if is_p1 {
                None
            } else {
                fam.partner1_id.as_ref()
            };
            let rem_p2 = if is_p2 {
                None
            } else {
                fam.partner2_id.as_ref()
            };
            let rem_children = fam
                .children
                .iter()
                .filter(|c| c.child_id != person.id)
                .count();
            if rem_p1.is_none() && rem_p2.is_none() && rem_children == 0 {
                empty_families_deleted_count += 1;
            }
        }
    }

    let display_name = format!(
        "{} {}",
        person.primary_name.given_name, person.primary_name.surname
    )
    .trim()
    .to_string();

    Ok(Json(DeleteImpactDto {
        person_id: person.id.clone(),
        gid: person.gid.clone(),
        display_name,
        events_count,
        families_unlinked_count,
        empty_families_deleted_count,
        notes_count,
        media_links_count,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/persons/{id}",
    params(
        ("id" = String, Path, description = "Person UUID or GID")
    ),
    responses(
        (status = 204, description = "Person safely deleted and snapshot logged to change_log"),
        (status = 403, description = "Viewer role is read-only", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_person(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let person_idx = store
        .persons
        .iter()
        .position(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed_person = store.persons.remove(person_idx);
    let person_id = removed_person.id.clone();

    let mut removed_events = Vec::new();
    store.events.retain(|e| {
        if e.individual_id.as_deref() == Some(person_id.as_str()) {
            removed_events.push(e.clone());
            false
        } else {
            true
        }
    });

    let mut removed_notes = Vec::new();
    store.notes.retain(|n| {
        if n.entity_type == "individual" && n.entity_id == person_id {
            removed_notes.push(n.clone());
            false
        } else {
            true
        }
    });

    let families_before: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| {
            f.partner1_id.as_deref() == Some(person_id.as_str())
                || f.partner2_id.as_deref() == Some(person_id.as_str())
                || f.children.iter().any(|c| c.child_id == person_id)
        })
        .cloned()
        .collect();

    // Unlink from families and delete any family left with 0 partners and 0 children (`10 §6.2`)
    let mut empty_family_ids: HashSet<String> = HashSet::new();
    for fam in &mut store.families {
        let mut touched = false;
        if fam.partner1_id.as_deref() == Some(person_id.as_str()) {
            fam.partner1_id = None;
            touched = true;
        }
        if fam.partner2_id.as_deref() == Some(person_id.as_str()) {
            fam.partner2_id = None;
            touched = true;
        }
        let before_children = fam.children.len();
        fam.children.retain(|c| c.child_id != person_id);
        if fam.children.len() != before_children {
            touched = true;
        }
        if touched {
            fam.version += 1;
        }
        if fam.partner1_id.is_none() && fam.partner2_id.is_none() && fam.children.is_empty() {
            empty_family_ids.insert(fam.id.clone());
        }
    }

    if !empty_family_ids.is_empty() {
        store.families.retain(|f| !empty_family_ids.contains(&f.id));
        store.events.retain(|e| {
            e.family_id
                .as_ref()
                .is_none_or(|fid| !empty_family_ids.contains(fid))
        });
    }

    // Unlink from media assets
    for m in &mut store.media {
        m.linked_person_ids.retain(|pid| pid != &person_id);
        m.face_tags.retain(|ft| ft.person_id != person_id);
    }

    // Record full snapshot in change_log so deletion can be undone from History (`07 §4, §5 DoD #5`)
    let snapshot = DeletedPersonSnapshot {
        person: removed_person.clone(),
        events: removed_events,
        notes: removed_notes,
        families_before,
    };

    let summary = format!(
        "Deleted individual {} ({} {})",
        removed_person.gid,
        removed_person.primary_name.given_name,
        removed_person.primary_name.surname
    )
    .trim()
    .to_string();

    store.change_log.push(ChangeLogEntryDto {
        id: Uuid::new_v4().to_string(),
        op: "delete".to_string(),
        entity_type: "individual".to_string(),
        entity_id: person_id,
        summary,
        before_json: serde_json::to_value(snapshot).ok(),
        after_json: None,
        created_at: Utc::now().to_rfc3339(),
    });

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/history/{id}/restore",
    params(
        ("id" = String, Path, description = "Change log entry ID or deleted person UUID/GID")
    ),
    responses(
        (status = 200, description = "Deleted person and their links restored from change_log", body = TreePersonNodeDto),
        (status = 404, description = "Change log entry or snapshot not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn restore_deleted_person(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TreePersonNodeDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let log_entry = store
        .change_log
        .iter()
        .rev()
        .find(|entry| {
            entry.op == "delete"
                && entry.entity_type == "individual"
                && (entry.id == target
                    || entry.entity_id == target
                    || entry
                        .before_json
                        .as_ref()
                        .and_then(|v| v.get("person"))
                        .and_then(|p| p.get("gid"))
                        .and_then(|g| g.as_str())
                        .is_some_and(|g| g.eq_ignore_ascii_case(target)))
        })
        .cloned()
        .ok_or(AppError::NotFound)?;

    let before_val = log_entry.before_json.ok_or(AppError::NotFound)?;
    let snapshot: DeletedPersonSnapshot = serde_json::from_value(before_val)
        .map_err(|e| AppError::Internal(format!("Failed to deserialize snapshot: {e}")))?;

    let restored_person_id = snapshot.person.id.clone();

    if !store.persons.iter().any(|p| p.id == restored_person_id) {
        store.persons.push(snapshot.person);
    }

    for ev in snapshot.events {
        if !store.events.iter().any(|e| e.id == ev.id) {
            store.events.push(ev);
        }
    }

    for note in snapshot.notes {
        if !store.notes.iter().any(|n| n.id == note.id) {
            store.notes.push(note);
        }
    }

    // Restore family links
    for old_fam in snapshot.families_before {
        if let Some(existing_fam) = store.families.iter_mut().find(|f| f.id == old_fam.id) {
            if old_fam.partner1_id.as_deref() == Some(restored_person_id.as_str())
                && existing_fam.partner1_id.is_none()
            {
                existing_fam.partner1_id = Some(restored_person_id.clone());
            }
            if old_fam.partner2_id.as_deref() == Some(restored_person_id.as_str())
                && existing_fam.partner2_id.is_none()
            {
                existing_fam.partner2_id = Some(restored_person_id.clone());
            }
            for ch in &old_fam.children {
                if ch.child_id == restored_person_id
                    && !existing_fam
                        .children
                        .iter()
                        .any(|c| c.child_id == restored_person_id)
                {
                    existing_fam.children.push(ch.clone());
                }
            }
            existing_fam.version += 1;
        } else {
            store.families.push(old_fam);
        }
    }

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let restored = store
        .persons
        .iter()
        .find(|p| p.id == restored_person_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    Ok(Json(restored))
}

#[utoipa::path(
    get,
    path = "/api/v1/families",
    responses(
        (status = 200, description = "List of family unions", body = FamilyListResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_families(
    caller: PublicOrAuth,
    State(state): State<AppState>,
) -> Result<Json<FamilyListResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let items: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| !f.is_private || access.can_view_private())
        .cloned()
        .collect();
    let total = items.len();

    Ok(Json(FamilyListResponse { items, total }))
}

#[utoipa::path(
    get,
    path = "/api/v1/families/{id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID (e.g. F0100)")
    ),
    responses(
        (status = 200, description = "Family union record", body = FamilyDto),
        (status = 404, description = "Family not found or private", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_family(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<FamilyDto>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let fam = store
        .families
        .iter()
        .find(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    if fam.is_private && !access.can_view_private() {
        return Err(AppError::NotFound);
    }

    Ok(Json(fam.clone()))
}

#[utoipa::path(
    post,
    path = "/api/v1/families",
    request_body = CreateFamilyRequest,
    responses(
        (status = 201, description = "Family union created", body = FamilyDto),
        (status = 400, description = "Validation error or ancestor cycle detected", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_family(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateFamilyRequest>,
) -> Result<(StatusCode, Json<FamilyDto>), AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;

    let p1_id = match req
        .partner1_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
        None => None,
    };
    let p2_id = match req
        .partner2_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
        None => None,
    };

    if p1_id.is_some() && p1_id == p2_id {
        let mut fields = HashMap::new();
        fields.insert("partner2_id".to_string(), "duplicate_partner".to_string());
        return Err(AppError::ValidationFailed(
            "partner1_id and partner2_id must be distinct individuals".to_string(),
            Some(fields),
        ));
    }

    let mut resolved_children = Vec::new();
    for (idx, mut ch) in req.children.unwrap_or_default().into_iter().enumerate() {
        let resolved_cid =
            resolve_person_id_in_store(&store.persons, &ch.child_id).ok_or(AppError::NotFound)?;
        for parent_id in [p1_id.as_deref(), p2_id.as_deref()].into_iter().flatten() {
            ensure_no_ancestor_cycle(&store.families, parent_id, &resolved_cid)?;
        }
        ch.child_id = resolved_cid;
        if ch.sort_order == 0 {
            ch.sort_order = idx as i32;
        }
        resolved_children.push(ch);
    }

    let existing_gids: HashSet<String> = store.families.iter().map(|f| f.gid.clone()).collect();
    let gid = allocate_gid(
        'F',
        &mut store.next_family_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );

    let fam = FamilyDto {
        id: Uuid::new_v4().to_string(),
        gid,
        partner1_id: p1_id,
        partner2_id: p2_id,
        relationship_type: req
            .relationship_type
            .unwrap_or_else(|| "married".to_string()),
        is_private: req.is_private.unwrap_or(false),
        children: resolved_children,
        version: 1,
    };

    store.families.push(fam.clone());
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok((StatusCode::CREATED, Json(fam)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/families/{id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID")
    ),
    request_body = UpdateFamilyRequest,
    responses(
        (status = 200, description = "Family union updated", body = FamilyDto),
        (status = 400, description = "Validation error or ancestor cycle detected", body = ErrorResponse),
        (status = 404, description = "Family not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_family(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFamilyRequest>,
) -> Result<Json<FamilyDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let fam_idx = store
        .families
        .iter()
        .position(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    if let Some(expected_version) = req.version {
        if expected_version != store.families[fam_idx].version {
            return Err(AppError::VersionConflict(
                "Record was modified by another session".to_string(),
            ));
        }
    }

    let next_p1 = if let Some(ref raw) = req.partner1_id {
        if raw.trim().is_empty() {
            None
        } else {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
    } else {
        store.families[fam_idx].partner1_id.clone()
    };

    let next_p2 = if let Some(ref raw) = req.partner2_id {
        if raw.trim().is_empty() {
            None
        } else {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
    } else {
        store.families[fam_idx].partner2_id.clone()
    };

    if next_p1.is_some() && next_p1 == next_p2 {
        let mut fields = HashMap::new();
        fields.insert("partner2_id".to_string(), "duplicate_partner".to_string());
        return Err(AppError::ValidationFailed(
            "partner1_id and partner2_id must be distinct individuals".to_string(),
            Some(fields),
        ));
    }

    let next_children = if let Some(children) = req.children {
        let mut resolved = Vec::with_capacity(children.len());
        for mut ch in children {
            let cid = resolve_person_id_in_store(&store.persons, &ch.child_id)
                .ok_or(AppError::NotFound)?;
            ch.child_id = cid;
            resolved.push(ch);
        }
        resolved
    } else {
        store.families[fam_idx].children.clone()
    };

    // Validate cycle-free graph with other families + this updated family
    let other_families: Vec<FamilyDto> = store
        .families
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != fam_idx)
        .map(|(_, f)| f.clone())
        .collect();

    for ch in &next_children {
        for parent_id in [next_p1.as_deref(), next_p2.as_deref()]
            .into_iter()
            .flatten()
        {
            ensure_no_ancestor_cycle(&other_families, parent_id, &ch.child_id)?;
        }
    }

    let fam = &mut store.families[fam_idx];
    fam.partner1_id = next_p1;
    fam.partner2_id = next_p2;
    fam.children = next_children;
    if let Some(rel_type) = req.relationship_type {
        fam.relationship_type = rel_type;
    }
    if let Some(is_priv) = req.is_private {
        fam.is_private = is_priv;
    }
    fam.version += 1;

    let updated = fam.clone();
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/families/{id}",
    params(
        ("id" = String, Path, description = "Family UUID or GID")
    ),
    responses(
        (status = 204, description = "Family union deleted while keeping partner and child individuals intact"),
        (status = 404, description = "Family not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_family(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let fam_idx = store
        .families
        .iter()
        .position(|f| f.id == target || f.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed_fam = store.families.remove(fam_idx);
    store
        .events
        .retain(|e| e.family_id.as_deref() != Some(removed_fam.id.as_str()));

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/events",
    params(EventFilterQuery),
    responses(
        (status = 200, description = "Chronological life events list", body = EventListResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_events(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<EventFilterQuery>,
) -> Result<Json<EventListResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let filter_ind_id = query.individual_id.as_deref().and_then(|raw| {
        resolve_person_id_in_store(&store.persons, raw).or_else(|| Some(raw.to_string()))
    });

    let filter_fam_id = query.family_id.as_deref().map(|raw| {
        store
            .families
            .iter()
            .find(|f| f.id == raw.trim() || f.gid.eq_ignore_ascii_case(raw.trim()))
            .map(|f| f.id.clone())
            .unwrap_or_else(|| raw.trim().to_string())
    });

    let mut items: Vec<LifeEventDto> = store
        .events
        .iter()
        .filter(|e| !e.is_private || access.can_view_private())
        .filter(|e| {
            if let Some(ref ind_id) = filter_ind_id {
                if e.individual_id.as_deref() != Some(ind_id.as_str()) {
                    return false;
                }
            }
            if let Some(ref fam_id) = filter_fam_id {
                if e.family_id.as_deref() != Some(fam_id.as_str()) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    items.sort_by_key(|e| e.date_sort.unwrap_or(i32::MAX));
    let total = items.len();

    Ok(Json(EventListResponse { items, total }))
}

#[utoipa::path(
    post,
    path = "/api/v1/events",
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Life event created and vitals/living status recomputed", body = LifeEventDto),
        (status = 400, description = "Validation error (e.g. family-only event on individual)", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_event(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<LifeEventDto>), AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;

    let ind_id = match req
        .individual_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => {
            Some(resolve_person_id_in_store(&store.persons, raw).ok_or(AppError::NotFound)?)
        }
        None => None,
    };

    let fam_id = match req
        .family_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => Some(
            store
                .families
                .iter()
                .find(|f| f.id == raw || f.gid.eq_ignore_ascii_case(raw))
                .map(|f| f.id.clone())
                .ok_or(AppError::NotFound)?,
        ),
        None => None,
    };

    // Exclusive owner check (`10-core-data-model.md §4.3`)
    if ind_id.is_some() == fam_id.is_some() {
        let mut fields = HashMap::new();
        fields.insert(
            "owner".to_string(),
            "exactly_one_of_individual_id_or_family_id_required".to_string(),
        );
        return Err(AppError::ValidationFailed(
            "An event must belong to either an individual or a family, never both or neither"
                .to_string(),
            Some(fields),
        ));
    }

    let event_type = req.event_type.trim().to_ascii_lowercase();
    if ind_id.is_some() && is_family_only_event_type(&event_type) {
        let mut fields = HashMap::new();
        fields.insert(
            "event_type".to_string(),
            "family_only_event_on_individual".to_string(),
        );
        return Err(AppError::ValidationFailed(
            format!(
                "Event type '{event_type}' can only be attached to a family record, not an individual"
            ),
            Some(fields),
        ));
    }

    let parsed_date = if let Some(d) = req.date {
        Some(d)
    } else {
        req.date_text
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(GenDate::parse)
    };
    let date_sort = parsed_date.as_ref().and_then(|d| d.sort_key());
    let gedcom_date = parsed_date.as_ref().map(|d| d.to_gedcom());

    let (resolved_place_id, resolved_place_full_name) = if let Some(ref pid_raw) = req.place_id {
        if !pid_raw.trim().is_empty() {
            let pid =
                resolve_place_id_in_store(&store.places, pid_raw).ok_or(AppError::NotFound)?;
            let fname = store
                .places
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.full_name.clone());
            (Some(pid), fname)
        } else {
            (None, None)
        }
    } else if let Some(ref ppath) = req.place_path {
        if !ppath.trim().is_empty() {
            let place = resolve_or_create_place_chain(&mut store, ppath, None, None, None, None)?;
            (Some(place.id), Some(place.full_name))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let existing_gids: HashSet<String> = store.events.iter().map(|e| e.gid.clone()).collect();
    let gid = allocate_gid(
        'E',
        &mut store.next_event_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );

    let ev = LifeEventDto {
        id: Uuid::new_v4().to_string(),
        gid,
        individual_id: ind_id,
        family_id: fam_id,
        event_type,
        custom_type: req.custom_type,
        date: parsed_date,
        date_sort,
        gedcom_date,
        place_id: resolved_place_id,
        place_full_name: resolved_place_full_name,
        description: req.description,
        source_text: req
            .source_text
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        media_ids: req.media_ids.unwrap_or_default(),
        is_private: req.is_private.unwrap_or(false),
        version: 1,
    };

    store.events.push(ev.clone());
    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok((StatusCode::CREATED, Json(ev)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/events/{id}",
    params(
        ("id" = String, Path, description = "Event UUID or GID (e.g. E0100)")
    ),
    request_body = UpdateEventRequest,
    responses(
        (status = 200, description = "Life event updated", body = LifeEventDto),
        (status = 404, description = "Event not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_event(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateEventRequest>,
) -> Result<Json<LifeEventDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let ev_idx = store
        .events
        .iter()
        .position(|e| e.id == target || e.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    if let Some(expected_version) = req.version {
        if expected_version != store.events[ev_idx].version {
            return Err(AppError::VersionConflict(
                "Record was modified by another session".to_string(),
            ));
        }
    }

    if let Some(ref et) = req.event_type {
        let normalized_et = et.trim().to_ascii_lowercase();
        if store.events[ev_idx].individual_id.is_some() && is_family_only_event_type(&normalized_et)
        {
            let mut fields = HashMap::new();
            fields.insert(
                "event_type".to_string(),
                "family_only_event_on_individual".to_string(),
            );
            return Err(AppError::ValidationFailed(
                format!("Event type '{normalized_et}' can only be attached to a family record"),
                Some(fields),
            ));
        }
        store.events[ev_idx].event_type = normalized_et;
    }

    if let Some(ct) = req.custom_type {
        store.events[ev_idx].custom_type = Some(ct);
    }

    if let Some(d) = req.date {
        store.events[ev_idx].date_sort = d.sort_key();
        store.events[ev_idx].gedcom_date = Some(d.to_gedcom());
        store.events[ev_idx].date = Some(d);
    } else if let Some(ref raw_date) = req.date_text {
        if raw_date.trim().is_empty() {
            store.events[ev_idx].date = None;
            store.events[ev_idx].date_sort = None;
            store.events[ev_idx].gedcom_date = None;
        } else {
            let parsed = GenDate::parse(raw_date);
            store.events[ev_idx].date_sort = parsed.sort_key();
            store.events[ev_idx].gedcom_date = Some(parsed.to_gedcom());
            store.events[ev_idx].date = Some(parsed);
        }
    }

    if let Some(ref pid_raw) = req.place_id {
        if pid_raw.trim().is_empty() {
            store.events[ev_idx].place_id = None;
            store.events[ev_idx].place_full_name = None;
        } else {
            let pid =
                resolve_place_id_in_store(&store.places, pid_raw).ok_or(AppError::NotFound)?;
            let fname = store
                .places
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.full_name.clone());
            store.events[ev_idx].place_id = Some(pid);
            store.events[ev_idx].place_full_name = fname;
        }
    } else if let Some(ref ppath) = req.place_path {
        if !ppath.trim().is_empty() {
            let place = resolve_or_create_place_chain(&mut store, ppath, None, None, None, None)?;
            store.events[ev_idx].place_id = Some(place.id);
            store.events[ev_idx].place_full_name = Some(place.full_name);
        }
    }

    if let Some(desc) = req.description {
        store.events[ev_idx].description = Some(desc);
    }
    if let Some(src) = req.source_text {
        let trimmed = src.trim().to_string();
        store.events[ev_idx].source_text = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    }
    if let Some(mids) = req.media_ids {
        store.events[ev_idx].media_ids = mids;
    }
    if let Some(is_priv) = req.is_private {
        store.events[ev_idx].is_private = is_priv;
    }

    store.events[ev_idx].version += 1;
    let updated = store.events[ev_idx].clone();

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/events/{id}",
    params(
        ("id" = String, Path, description = "Event UUID or GID")
    ),
    responses(
        (status = 204, description = "Life event deleted and vitals recomputed"),
        (status = 404, description = "Event not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_event(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let ev_idx = store
        .events
        .iter()
        .position(|e| e.id == target || e.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed_ev = store.events.remove(ev_idx);
    if let Some(ref ind_id) = removed_ev.individual_id {
        let has_remaining_death = store.events.iter().any(|e| {
            e.individual_id.as_deref() == Some(ind_id.as_str())
                && matches!(e.event_type.as_str(), "death" | "burial" | "cremation")
        });
        let has_remaining_birth = store.events.iter().any(|e| {
            e.individual_id.as_deref() == Some(ind_id.as_str())
                && matches!(e.event_type.as_str(), "birth" | "baptism")
        });

        if let Some(person) = store.persons.iter_mut().find(|p| &p.id == ind_id) {
            if !has_remaining_death
                && matches!(
                    removed_ev.event_type.as_str(),
                    "death" | "burial" | "cremation"
                )
            {
                person.has_death_event = false;
                person.death_sort = None;
                person.death = None;
            }
            if !has_remaining_birth && matches!(removed_ev.event_type.as_str(), "birth" | "baptism")
            {
                person.birth_sort = None;
                person.birth = None;
            }
        }
    }

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/events/sources",
    params(SourceSearchQuery),
    responses(
        (status = 200, description = "Distinct source citation strings for autocomplete", body = SourceAutocompleteResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_event_sources(
    RequireAuth(_auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<SourceSearchQuery>,
) -> Result<Json<SourceAutocompleteResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let q_norm = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_search_tokens);

    let mut seen = HashSet::new();
    let mut items = Vec::new();

    for ev in &store.events {
        if let Some(ref src) = ev.source_text {
            let trimmed = src.trim();
            if !trimmed.is_empty() && seen.insert(trimmed.to_lowercase()) {
                if let Some(ref qn) = q_norm {
                    let src_norm = normalize_search_tokens(trimmed);
                    if !src_norm.contains(qn) {
                        continue;
                    }
                }
                items.push(trimmed.to_string());
            }
        }
    }

    Ok(Json(SourceAutocompleteResponse { items }))
}

#[utoipa::path(
    get,
    path = "/api/v1/notes",
    params(NoteFilterQuery),
    responses(
        (status = 200, description = "Notes attached to an entity", body = NoteListResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_notes(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<NoteFilterQuery>,
) -> Result<Json<NoteListResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let mut items: Vec<NoteDto> = store
        .notes
        .iter()
        .filter(|n| !n.is_private || access.can_view_private())
        .filter(|n| {
            if let Some(ref et) = query.entity_type {
                if !n.entity_type.eq_ignore_ascii_case(et.trim()) {
                    return false;
                }
            }
            if let Some(ref eid) = query.entity_id {
                if n.entity_id != eid.trim() {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    items.sort_by_key(|n| n.sort_order);
    let total = items.len();

    Ok(Json(NoteListResponse { items, total }))
}

#[utoipa::path(
    post,
    path = "/api/v1/notes",
    request_body = CreateNoteRequest,
    responses(
        (status = 201, description = "Note created", body = NoteDto),
        (status = 400, description = "Validation error", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_note(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<NoteDto>), AppError> {
    let entity_type = req.entity_type.trim().to_ascii_lowercase();
    if !matches!(
        entity_type.as_str(),
        "individual" | "family" | "event" | "place" | "media"
    ) {
        let mut fields = HashMap::new();
        fields.insert("entity_type".to_string(), "invalid_entity_type".to_string());
        return Err(AppError::ValidationFailed(
            "entity_type must be one of: individual, family, event, place, media".to_string(),
            Some(fields),
        ));
    }

    if req.body.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("body".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "Note body must not be empty".to_string(),
            Some(fields),
        ));
    }

    let mut store = state.privacy_store.write().await;
    let note = NoteDto {
        id: Uuid::new_v4().to_string(),
        entity_type,
        entity_id: req.entity_id.trim().to_string(),
        body: req.body,
        is_private: req.is_private.unwrap_or(false),
        sort_order: req.sort_order.unwrap_or(0),
        version: 1,
    };
    store.notes.push(note.clone());

    Ok((StatusCode::CREATED, Json(note)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/notes/{id}",
    params(
        ("id" = String, Path, description = "Note UUID")
    ),
    request_body = UpdateNoteRequest,
    responses(
        (status = 200, description = "Note updated", body = NoteDto),
        (status = 404, description = "Note not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_note(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<Json<NoteDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let note = store
        .notes
        .iter_mut()
        .find(|n| n.id == id.trim())
        .ok_or(AppError::NotFound)?;

    if let Some(expected_version) = req.version {
        if expected_version != note.version {
            return Err(AppError::VersionConflict(
                "Record was modified by another session".to_string(),
            ));
        }
    }

    if let Some(body) = req.body {
        note.body = body;
    }
    if let Some(is_priv) = req.is_private {
        note.is_private = is_priv;
    }
    if let Some(so) = req.sort_order {
        note.sort_order = so;
    }
    note.version += 1;

    Ok(Json(note.clone()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/notes/{id}",
    params(
        ("id" = String, Path, description = "Note UUID")
    ),
    responses(
        (status = 204, description = "Note deleted"),
        (status = 404, description = "Note not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_note(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let before = store.notes.len();
    store.notes.retain(|n| n.id != id.trim());
    if store.notes.len() == before {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{id}/file",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID (e.g. M0002)")
    ),
    responses(
        (status = 200, description = "Media file binary stream", content_type = "image/webp"),
        (status = 404, description = "Media asset not found or shielded by living/private privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_media_file(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let media = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    // Direct Media Shield (`02-rbac-access-control.md §7 DoD #3`):
    // Living-person photos and private assets return 404 Not Found (indistinguishable from missing)
    let _permitted = redact_media_for_caller(media, &access).ok_or(AppError::NotFound)?;

    // Minimal valid WebP RIFF container header payload
    let webp_bytes: &'static [u8] = b"RIFF\x1a\x00\x00\x00WEBPVP8L\x0d\x00\x00\x00/\x00\x00\x00\x10\x07\x10\x11\x11\x88\x88\xfe\x07\x00";
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media.mime_type.as_str())
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(webp_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to build media response: {e}")))?;

    Ok(response.into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/people",
    responses(
        (status = 200, description = "People & Surnames directory (authenticated users only)", body = PeopleDirectoryResponse),
        (status = 401, description = "Authentication required even when public tree sharing is enabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_people_directory(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<PeopleDirectoryResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = auth.access();
    let store = state.privacy_store.read().await;

    let items: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();
    let total = items.len();

    Ok(Json(PeopleDirectoryResponse {
        items,
        next_cursor: None,
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/places",
    params(PlacesFilterQuery),
    responses(
        (status = 200, description = "Places directory & coordinates (authenticated users only)", body = PlacesDirectoryResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_places_directory(
    RequireAuth(_auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<PlacesFilterQuery>,
) -> Result<Json<PlacesDirectoryResponse>, AppError> {
    let store = state.privacy_store.read().await;

    let q_raw = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());
    let q_norm = q_raw.as_deref().map(normalize_search_tokens);

    let resolved_parent_filter = query
        .parent_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|raw| {
            resolve_place_id_in_store(&store.places, raw).unwrap_or_else(|| raw.to_string())
        });

    let items: Vec<PlaceSummaryDto> = store
        .places
        .iter()
        .filter(|p| {
            if let Some(ref pid) = resolved_parent_filter {
                if p.parent_id.as_deref() != Some(pid.as_str()) {
                    return false;
                }
            }
            if query.missing_coords == Some(true) && p.latitude.is_some() && p.longitude.is_some() {
                return false;
            }
            if let (Some(ref qr), Some(ref qn)) = (&q_raw, &q_norm) {
                let name_matches = p.name.to_lowercase().contains(qr)
                    || normalize_search_tokens(&p.name).contains(qn);
                let full_matches = p.full_name.to_lowercase().contains(qr)
                    || normalize_search_tokens(&p.full_name).contains(qn);
                let alt_matches = p.alt_names.iter().any(|a| {
                    a.name.to_lowercase().contains(qr)
                        || normalize_search_tokens(&a.name).contains(qn)
                });
                if !name_matches && !full_matches && !alt_matches {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    let total = items.len();

    Ok(Json(PlacesDirectoryResponse {
        items,
        next_cursor: None,
        total,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/places",
    request_body = CreatePlaceRequest,
    responses(
        (status = 201, description = "Place or comma-separated place hierarchy created/resolved", body = PlaceSummaryDto),
        (status = 400, description = "Validation error", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn create_place(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreatePlaceRequest>,
) -> Result<(StatusCode, Json<PlaceSummaryDto>), AppError> {
    let mut store = state.privacy_store.write().await;

    let raw_path = req
        .path
        .as_deref()
        .or(req.name.as_deref())
        .map(str::trim)
        .unwrap_or("");

    if raw_path.is_empty() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "Place name or path is required".to_string(),
            Some(fields),
        ));
    }

    // If `path` is explicitly provided OR `name` contains comma-separated segments without an explicit `parent_id`,
    // resolve/create the hierarchy top-down (`12-places.md §3.1, §7 DoD #1 & #5`).
    if req.path.is_some() || (raw_path.contains(',') && req.parent_id.is_none()) {
        let leaf = resolve_or_create_place_chain(
            &mut store,
            raw_path,
            req.place_type.as_deref(),
            req.latitude,
            req.longitude,
            req.alt_names,
        )?;
        return Ok((StatusCode::CREATED, Json(leaf)));
    }

    let resolved_parent_id = match req
        .parent_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => Some(resolve_place_id_in_store(&store.places, raw).ok_or(AppError::NotFound)?),
        None => None,
    };

    let existing_gids: HashSet<String> = store.places.iter().map(|p| p.gid.clone()).collect();
    let gid = allocate_gid(
        'P',
        &mut store.next_place_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );
    let new_id = Uuid::new_v4().to_string();

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

    let new_place = PlaceSummaryDto {
        id: new_id.clone(),
        gid,
        name: raw_path.to_string(),
        place_type: req.place_type.unwrap_or_else(|| "other".to_string()),
        parent_id: resolved_parent_id,
        full_name: raw_path.to_string(),
        country: None,
        latitude: req.latitude,
        longitude: req.longitude,
        events_count: 0,
        children_count: 0,
        alt_names,
        version: 1,
    };

    store.places.push(new_place);
    recompute_all_places_metadata(&mut store);

    let created = store
        .places
        .iter()
        .find(|p| p.id == new_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/v1/places/{id}",
    params(
        ("id" = String, Path, description = "Place UUID or GID (e.g. P0001)")
    ),
    responses(
        (status = 200, description = "Place detail with breadcrumbs, child places, and events", body = PlaceDetailDto),
        (status = 404, description = "Place not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_place_detail(
    RequireAuth(_auth): RequireAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PlaceDetailDto>, AppError> {
    let store = state.privacy_store.read().await;
    let place_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;
    let place = store
        .places
        .iter()
        .find(|p| p.id == place_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    let breadcrumbs = build_place_breadcrumbs(&store.places, &place_id);
    let children: Vec<PlaceSummaryDto> = store
        .places
        .iter()
        .filter(|p| p.parent_id.as_deref() == Some(place_id.as_str()))
        .cloned()
        .collect();
    let events: Vec<LifeEventDto> = store
        .events
        .iter()
        .filter(|e| e.place_id.as_deref() == Some(place_id.as_str()))
        .cloned()
        .collect();

    Ok(Json(PlaceDetailDto {
        place,
        breadcrumbs,
        children,
        events,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/places/{id}",
    params(
        ("id" = String, Path, description = "Place UUID or GID")
    ),
    request_body = UpdatePlaceRequest,
    responses(
        (status = 200, description = "Place updated and descendant full_name values recomputed", body = PlaceSummaryDto),
        (status = 400, description = "Validation error or place hierarchy cycle detected", body = ErrorResponse),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 409, description = "Optimistic locking version conflict", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_place(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePlaceRequest>,
) -> Result<Json<PlaceSummaryDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let place_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;
    let idx = store
        .places
        .iter()
        .position(|p| p.id == place_id)
        .ok_or(AppError::NotFound)?;

    if let Some(expected_version) = req.version {
        if expected_version != store.places[idx].version {
            return Err(AppError::VersionConflict(
                "Record was modified by another session".to_string(),
            ));
        }
    }

    if req.clear_parent == Some(true) {
        store.places[idx].parent_id = None;
    } else if let Some(ref raw_parent) = req.parent_id {
        if raw_parent.trim().is_empty() {
            store.places[idx].parent_id = None;
        } else {
            let resolved_parent =
                resolve_place_id_in_store(&store.places, raw_parent).ok_or(AppError::NotFound)?;
            ensure_no_place_cycle(&store.places, &place_id, Some(&resolved_parent))?;
            store.places[idx].parent_id = Some(resolved_parent);
        }
    }

    if let Some(name) = req.name {
        let trimmed = name.trim().to_string();
        if !trimmed.is_empty() {
            store.places[idx].name = trimmed;
        }
    }
    if let Some(pt) = req.place_type {
        store.places[idx].place_type = pt;
    }
    if let Some(lat) = req.latitude {
        store.places[idx].latitude = Some(lat);
    }
    if let Some(lon) = req.longitude {
        store.places[idx].longitude = Some(lon);
    }
    if let Some(alts) = req.alt_names {
        store.places[idx].alt_names = alts
            .into_iter()
            .map(|mut a| {
                if a.id.is_empty() {
                    a.id = Uuid::new_v4().to_string();
                }
                a
            })
            .collect();
    }

    store.places[idx].version += 1;
    recompute_all_places_metadata(&mut store);

    // Refresh cached `place_full_name` on events
    let full_name_map: HashMap<String, String> = store
        .places
        .iter()
        .map(|p| (p.id.clone(), p.full_name.clone()))
        .collect();
    for ev in &mut store.events {
        if let Some(ref pid) = ev.place_id {
            if let Some(fname) = full_name_map.get(pid) {
                ev.place_full_name = Some(fname.clone());
            }
        }
    }
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let updated = store
        .places
        .iter()
        .find(|p| p.id == place_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/places/{id}",
    params(
        ("id" = String, Path, description = "Place UUID or GID")
    ),
    responses(
        (status = 204, description = "Unused place deleted"),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 409, description = "Place is referenced by events or child places (place_in_use)", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_place(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let place_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;

    let has_child_places = store
        .places
        .iter()
        .any(|p| p.parent_id.as_deref() == Some(place_id.as_str()));
    let has_referencing_events = store
        .events
        .iter()
        .any(|e| e.place_id.as_deref() == Some(place_id.as_str()));

    if has_child_places || has_referencing_events {
        return Err(AppError::PlaceInUse);
    }

    store.places.retain(|p| p.id != place_id);
    recompute_all_places_metadata(&mut store);

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/places/{id}/merge",
    params(
        ("id" = String, Path, description = "Target place UUID or GID to keep")
    ),
    request_body = MergePlaceRequest,
    responses(
        (status = 200, description = "Source place merged into target place", body = PlaceSummaryDto),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Target or source place not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn merge_place(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MergePlaceRequest>,
) -> Result<Json<PlaceSummaryDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let target_id = resolve_place_id_in_store(&store.places, &id).ok_or(AppError::NotFound)?;

    let source_raw = req
        .other_id
        .as_deref()
        .or(req.source_place_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            let mut fields = HashMap::new();
            fields.insert("other_id".to_string(), "required".to_string());
            AppError::ValidationFailed(
                "other_id (source place to merge) is required".to_string(),
                Some(fields),
            )
        })?;

    let source_id =
        resolve_place_id_in_store(&store.places, source_raw).ok_or(AppError::NotFound)?;

    if target_id == source_id {
        let mut fields = HashMap::new();
        fields.insert("other_id".to_string(), "same_as_target".to_string());
        return Err(AppError::ValidationFailed(
            "Cannot merge a place with itself".to_string(),
            Some(fields),
        ));
    }

    let source_idx = store
        .places
        .iter()
        .position(|p| p.id == source_id)
        .ok_or(AppError::NotFound)?;
    let source_place = store.places.remove(source_idx);

    // Re-point events referencing source_id to target_id
    for ev in &mut store.events {
        if ev.place_id.as_deref() == Some(source_id.as_str()) {
            ev.place_id = Some(target_id.clone());
        }
    }

    // Re-point child places referencing source_id to target_id
    for p in &mut store.places {
        if p.parent_id.as_deref() == Some(source_id.as_str()) && p.id != target_id {
            p.parent_id = Some(target_id.clone());
        }
    }

    // Re-point notes referencing source_id to target_id
    for n in &mut store.notes {
        if n.entity_type == "place" && n.entity_id == source_id {
            n.entity_id = target_id.clone();
        }
    }

    // Merge alternate names + old primary name into target place (`12-places.md §3.3, §7 DoD #4`)
    if let Some(target_place) = store.places.iter_mut().find(|p| p.id == target_id) {
        if target_place.latitude.is_none() {
            target_place.latitude = source_place.latitude;
        }
        if target_place.longitude.is_none() {
            target_place.longitude = source_place.longitude;
        }

        let mut candidate_alts = vec![PlaceAltNameDto {
            id: Uuid::new_v4().to_string(),
            name: source_place.name.clone(),
            lang: None,
            note: Some(format!("Merged from {}", source_place.gid)),
        }];
        candidate_alts.extend(source_place.alt_names);

        for cand in candidate_alts {
            let matches_primary = target_place.name.eq_ignore_ascii_case(&cand.name);
            let already_in_alts = target_place
                .alt_names
                .iter()
                .any(|a| a.name.eq_ignore_ascii_case(&cand.name));
            if !matches_primary && !already_in_alts {
                target_place.alt_names.push(cand);
            }
        }

        target_place.events_count += source_place.events_count;
        target_place.version += 1;
    }

    recompute_all_places_metadata(&mut store);

    let full_name_map: HashMap<String, String> = store
        .places
        .iter()
        .map(|p| (p.id.clone(), p.full_name.clone()))
        .collect();
    for ev in &mut store.events {
        if let Some(ref pid) = ev.place_id {
            if let Some(fname) = full_name_map.get(pid) {
                ev.place_full_name = Some(fname.clone());
            }
        }
    }
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    let merged = store
        .places
        .iter()
        .find(|p| p.id == target_id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    Ok(Json(merged))
}

#[utoipa::path(
    get,
    path = "/api/v1/media",
    responses(
        (status = 200, description = "Media archive & gallery (authenticated users only)", body = MediaArchiveResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_media_archive(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<MediaArchiveResponse>, AppError> {
    let access = auth.access();
    let store = state.privacy_store.read().await;

    let items: Vec<MediaAssetSummaryDto> = store
        .media
        .iter()
        .filter_map(|m| redact_media_for_caller(m, &access))
        .collect();
    let total = items.len();

    Ok(Json(MediaArchiveResponse {
        items,
        next_cursor: None,
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/gedcom/export",
    responses(
        (status = 200, description = "GEDCOM export metadata (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn export_gedcom(
    RequireAdmin(_admin): RequireAdmin,
) -> Result<Json<AdminToolOperationResponse>, AppError> {
    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "gedcom_export".to_string(),
        message: "GEDCOM export authorized for administrator".to_string(),
        items: vec![],
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/import",
    responses(
        (status = 200, description = "GEDCOM import job queued (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn import_gedcom(
    RequireAdmin(_admin): RequireAdmin,
) -> Result<Json<AdminToolOperationResponse>, AppError> {
    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "gedcom_import".to_string(),
        message: "GEDCOM import authorized for administrator".to_string(),
        items: vec![],
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/export/pdf",
    responses(
        (status = 200, description = "Printable PDF poster export queued (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn export_printable_pdf(
    RequireAdmin(_admin): RequireAdmin,
) -> Result<Json<AdminToolOperationResponse>, AppError> {
    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "pdf_export".to_string(),
        message: "Printable PDF export authorized for administrator".to_string(),
        items: vec![],
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/history",
    responses(
        (status = 200, description = "Global system audit history feed (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_audit_history(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<AdminToolOperationResponse>, AppError> {
    let store = state.privacy_store.read().await;
    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "audit_history".to_string(),
        message: "Global change history feed authorized for administrator".to_string(),
        items: store.change_log.clone(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality",
    responses(
        (status = 200, description = "Data quality warnings inspection (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_data_quality_warnings(
    RequireAdmin(_admin): RequireAdmin,
) -> Result<Json<AdminToolOperationResponse>, AppError> {
    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "data_quality".to_string(),
        message: "Data quality warnings authorized for administrator".to_string(),
        items: vec![],
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/people/merge",
    request_body(content = Option<PersonMergeRequest>, description = "Optional merge target IDs"),
    responses(
        (status = 200, description = "In-tree duplicate person merge (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn merge_duplicate_people(
    RequireAdmin(_admin): RequireAdmin,
    _payload: Option<Json<PersonMergeRequest>>,
) -> Result<Json<AdminToolOperationResponse>, AppError> {
    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "person_merge".to_string(),
        message: "Duplicate person merge authorized for administrator".to_string(),
        items: vec![],
    }))
}
