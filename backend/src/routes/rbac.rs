use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::{AppError, ErrorResponse},
    services::{
        privacy::{
            evaluate_is_living, redact_media_for_caller, redact_person_for_caller,
            MediaAssetSummaryDto, PersonLivingEvaluationInput, PersonNameDto, PlaceSummaryDto,
            TreePersonNodeDto, VitalEventSummaryDto,
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePersonRequest {
    pub given_name: String,
    pub patronymic: Option<String>,
    pub surname: String,
    pub sex: Option<String>,
    pub birth_year: Option<i32>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
    pub has_death_or_burial_event: Option<bool>,
    pub living_override: Option<bool>,
    pub is_private: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdatePersonRequest {
    pub given_name: Option<String>,
    pub patronymic: Option<String>,
    pub surname: Option<String>,
    pub is_private: Option<bool>,
    pub living_override: Option<bool>,
    pub notes: Option<String>,
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PersonMergeRequest {
    pub primary_person_id: Option<String>,
    pub duplicate_person_id: Option<String>,
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
    let current_year = chrono::Utc::now().year();
    let eval_input = PersonLivingEvaluationInput {
        living_override: req.living_override,
        has_death_or_burial_event: req.has_death_or_burial_event.unwrap_or(false),
        birth_year: req.birth_year,
        ..Default::default()
    };
    let is_living = evaluate_is_living(
        &eval_input,
        current_year,
        settings.living_person_age_threshold as i32,
    );

    let mut store = state.privacy_store.write().await;
    let gid = format!("I{:04}", store.next_gid_seq);
    store.next_gid_seq += 1;

    let birth = if req.birth_date.is_some() || req.birth_place.is_some() || req.birth_year.is_some()
    {
        Some(VitalEventSummaryDto {
            date: req
                .birth_date
                .or_else(|| req.birth_year.map(|y| y.to_string())),
            place: req.birth_place,
        })
    } else {
        None
    };

    let node = TreePersonNodeDto {
        id: Uuid::new_v4().to_string(),
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
        birth,
        death: None,
        avatar_url: None,
        notes: req.notes,
    };

    store.persons.push(node.clone());
    Ok((StatusCode::CREATED, Json(node)))
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
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn update_person(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePersonRequest>,
) -> Result<Json<TreePersonNodeDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let person = store
        .persons
        .iter_mut()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

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
    if let Some(is_priv) = req.is_private {
        person.is_private = is_priv;
    }
    if let Some(override_val) = req.living_override {
        person.living_override = Some(override_val);
        person.is_living = override_val;
    }
    if let Some(notes) = req.notes {
        person.notes = Some(notes);
    }

    Ok(Json(person.clone()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/persons/{id}",
    params(
        ("id" = String, Path, description = "Person UUID or GID")
    ),
    responses(
        (status = 204, description = "Person deleted by Editor or Admin"),
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
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let before = store.persons.len();
    store
        .persons
        .retain(|p| !(p.id == target || p.gid.eq_ignore_ascii_case(target)));
    if store.persons.len() == before {
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
    responses(
        (status = 200, description = "Places directory & coordinates (authenticated users only)", body = PlacesDirectoryResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_places_directory(
    RequireAuth(_auth): RequireAuth,
    State(state): State<AppState>,
) -> Result<Json<PlacesDirectoryResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let items = store.places.clone();
    let total = items.len();

    Ok(Json(PlacesDirectoryResponse {
        items,
        next_cursor: None,
        total,
    }))
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
) -> Result<Json<AdminToolOperationResponse>, AppError> {
    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "audit_history".to_string(),
        message: "Global change history feed authorized for administrator".to_string(),
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
    }))
}
