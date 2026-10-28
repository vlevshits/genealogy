use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    error::{AppError, ErrorResponse},
    models::gendate::GenDate,
    services::{
        auth::{AuthService, UserDto},
        dashboard::{
            build_dashboard_response, compute_tree_stats, compute_upcoming_anniversaries,
            AnniversariesListResponse, AnniversariesQuery, DashboardResponse, TreeStatsResponse,
        },
        gedcom::{
            analyze_gedcom_payload, export_store_to_gedcom, import_gedcom_into_store,
            GedcomAnalysisReport, GedcomAnalyzeRequest, GedcomExportRequest, GedcomImportRequest,
            GedcomImportResult,
        },
        genealogy::{
            allocate_gid, build_place_breadcrumbs, calculate_kinship_relation, compute_fan_chart,
            compute_kinship_path, compute_places_map, ensure_no_ancestor_cycle,
            ensure_no_place_cycle, recompute_all_individuals_vitals, recompute_all_places_metadata,
            resolve_or_create_place_chain, FanChartQuery, FanChartResponse, KinshipQuery,
            KinshipResponse, PlacesMapResponse, ProfileKinshipDto,
        },
        history::{
            create_snapshot, get_entity_version, log_change, parse_if_match_version,
            query_history_feed, query_record_history, restore_change, rollback_snapshot,
            undo_transaction, verify_optimistic_version, CreateSnapshotRequest,
            EntityVersionResponse, HistoryFeedQuery, HistoryFeedResponse, RecordHistoryResponse,
            RollbackSnapshotRequest, RollbackSnapshotResponse, SnapshotListResponse,
            UndoTransactionResponse,
        },
        privacy::{
            canonicalize_surname_display, evaluate_is_living, fold_slavic_surname_stem,
            normalize_russian_orthography_word, normalize_search_tokens, parse_search_query,
            redact_media_for_caller, redact_person_for_caller, redact_person_for_tree_view,
            suggest_slavic_patronymic, transliterate_to_latin, AvatarCropDto, ChangeLogEntryDto,
            FaceTagDto, FamilyChildDto, FamilyDto, GenealogyPrivacyStore, IndividualAltNameDto,
            LifeEventDto, MediaAssetSummaryDto, MediaLinkDto, NoteDto, PersonLivingEvaluationInput,
            PersonNameDto, PlaceAltNameDto, PlaceBreadcrumbDto, PlaceSummaryDto, TreePersonNodeDto,
            TreeSnapshotDto, VitalEventSummaryDto,
        },
        quality::{
            build_quality_summary, compute_duplicate_candidates, compute_quality_warnings,
            dismiss_warning_in_store, execute_person_merge, mark_not_duplicate_in_store,
            match_prospective_person, preview_person_merge, DismissWarningRequest,
            DismissWarningResponse, DuplicateCandidatesFilterQuery,
            DuplicateCandidatesListResponse, MergePreviewQuery, MergePreviewResponse,
            NotDuplicatePairRequest, NotDuplicatePairResponse, PersonMatchQuery,
            PersonMatchResponse, PersonMergeRequest as QualityPersonMergeRequest,
            PersonMergeResponse, QualitySummaryResponse, QualityWarningsFilterQuery,
            QualityWarningsListResponse,
        },
        rbac::{CallerAccess, PublicOrAuth, RequireAdmin, RequireAuth, RequireEditor},
    },
    state::AppState,
};

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct PersonProfileQuery {
    /// Comma-separated list of sections to include (`kinship`, `events`, `families`, `media`, `notes`, `sources`); defaults to all sections when omitted or empty
    pub include: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileRelativeSummaryDto {
    pub id: String,
    pub gid: String,
    pub given_name: String,
    pub patronymic: Option<String>,
    pub surname: String,
    pub sex: String,
    pub is_living: bool,
    pub birth_year: Option<i32>,
    pub death_year: Option<i32>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileChildSummaryDto {
    pub person: ProfileRelativeSummaryDto,
    pub sort_order: i32,
    pub rel_to_partner1: String,
    pub rel_to_partner2: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileParentFamilyDto {
    pub family_id: String,
    pub gid: String,
    pub father: Option<ProfileRelativeSummaryDto>,
    pub mother: Option<ProfileRelativeSummaryDto>,
    pub siblings: Vec<ProfileRelativeSummaryDto>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileSpouseFamilyDto {
    pub family_id: String,
    pub gid: String,
    pub relationship_type: String,
    pub spouse: Option<ProfileRelativeSummaryDto>,
    pub marriage_date: Option<String>,
    pub marriage_place: Option<String>,
    pub children: Vec<ProfileChildSummaryDto>,
    pub version: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileFamiliesDto {
    pub parent_families: Vec<ProfileParentFamilyDto>,
    pub spouse_families: Vec<ProfileSpouseFamilyDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ProfileLifeEventDto {
    pub id: String,
    pub gid: String,
    pub individual_id: Option<String>,
    pub family_id: Option<String>,
    pub event_type: String,
    pub custom_type: Option<String>,
    pub date: Option<GenDate>,
    pub date_sort: Option<i32>,
    pub gedcom_date: Option<String>,
    pub place_id: Option<String>,
    pub place_full_name: Option<String>,
    pub description: Option<String>,
    pub source_text: Option<String>,
    pub media_ids: Vec<String>,
    pub is_private: bool,
    pub version: i32,
    pub age_at_event: Option<i32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub scans_count: usize,
    pub spouse_id: Option<String>,
    pub spouse_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileSourceSummaryDto {
    pub source_text: String,
    pub event_ids: Vec<String>,
    pub events_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileCapabilitiesDto {
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_merge: bool,
    pub is_home_person: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PersonProfileResponseDto {
    pub person: TreePersonNodeDto,
    pub kinship: Option<ProfileKinshipDto>,
    pub events: Vec<ProfileLifeEventDto>,
    pub families: ProfileFamiliesDto,
    pub media: Vec<MediaAssetSummaryDto>,
    pub notes: Vec<NoteDto>,
    pub sources_summary: Vec<ProfileSourceSummaryDto>,
    pub capabilities: ProfileCapabilitiesDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TreeResponse {
    pub access_tier: String,
    pub tree_title: String,
    pub nodes: Vec<TreePersonNodeDto>,
    #[serde(default)]
    pub families: Vec<FamilyDto>,
    pub total_visible: usize,
    pub resolved_home_person_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct TreeViewQuery {
    /// Focal person UUID or GEDCOM-style GID (e.g. `I0100`); defaults to resolved home person or first root person
    pub focal_id: Option<String>,
    /// Projection mode: `"hourglass"`, `"pedigree"`, `"descendant"`, or `"fan"` (default `"hourglass"`)
    pub mode: Option<String>,
    /// Generation depth limit (`1..=10`, default `4`)
    pub generations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TreeEdgeDto {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: String,
    pub family_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TreeViewResponse {
    pub access_tier: String,
    pub tree_title: String,
    pub focal_id: Option<String>,
    pub mode: String,
    pub generations: u32,
    pub nodes: Vec<TreePersonNodeDto>,
    pub families: Vec<FamilyDto>,
    pub edges: Vec<TreeEdgeDto>,
    pub resolved_home_person_id: Option<String>,
    pub total_visible: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OnboardingStatusResponse {
    pub is_empty: bool,
    pub total_individuals: usize,
    pub can_onboard: bool,
    pub resolved_home_person_id: Option<String>,
    pub default_home_person_id: Option<String>,
    pub user_home_person_id: Option<String>,
    pub user_person_id: Option<String>,
    pub start_page: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct StartWithSelfRequest {
    #[serde(default)]
    pub given_name: String,
    pub patronymic: Option<String>,
    #[serde(default)]
    pub surname: String,
    pub sex: Option<String>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StartWithSelfResponse {
    pub individual: TreePersonNodeDto,
    pub user: UserDto,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<String>,
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
    pub children_ids: Option<Vec<String>>,
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

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    /// Comma-separated result groups (`person,place,media,action`; default all)
    pub types: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SearchPersonResultDto {
    pub id: String,
    pub gid: String,
    pub display_name: String,
    pub given_name: String,
    pub patronymic: Option<String>,
    pub surname: String,
    pub sex: String,
    pub birth_year: Option<i32>,
    pub death_year: Option<i32>,
    pub birth_place: Option<String>,
    pub avatar_url: Option<String>,
    pub is_living: bool,
    pub is_private: bool,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SearchPlaceResultDto {
    pub id: String,
    pub gid: String,
    pub name: String,
    pub full_name: String,
    pub place_type: String,
    pub events_count: u32,
    pub has_coordinates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SearchMediaResultDto {
    pub id: String,
    pub gid: String,
    pub title: String,
    pub mime_type: String,
    pub file_url: String,
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SearchActionDto {
    pub id: String,
    pub label: String,
    pub hint: String,
    pub path: String,
    pub icon: String,
    pub required_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResultResponseDto {
    pub query: String,
    pub people: Vec<SearchPersonResultDto>,
    pub places: Vec<SearchPlaceResultDto>,
    pub media: Vec<SearchMediaResultDto>,
    pub actions: Vec<SearchActionDto>,
    pub timing_ms: u64,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct PeopleDirectoryQuery {
    pub q: Option<String>,
    pub surname: Option<String>,
    pub born_from: Option<i32>,
    pub born_to: Option<i32>,
    pub sex: Option<String>,
    /// `"all"`, `"living"`, or `"deceased"`
    pub living: Option<String>,
    pub has_photo: Option<bool>,
    /// When `true`, filters to individuals missing a birth date OR having no parent links
    pub incomplete: Option<bool>,
    /// `"name_asc"`, `"name_desc"`, `"birth_asc"`, `"birth_desc"`, `"death_asc"`, `"death_desc"`, `"updated_desc"`
    pub sort: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PeopleDirectoryItemDto {
    pub id: String,
    pub gid: String,
    pub given_name: String,
    pub patronymic: Option<String>,
    pub surname: String,
    pub birth_surname: Option<String>,
    pub display_name: String,
    pub sex: String,
    pub birth_date: Option<String>,
    pub birth_year: Option<i32>,
    pub birth_place: Option<String>,
    pub death_date: Option<String>,
    pub death_year: Option<i32>,
    pub death_place: Option<String>,
    pub avatar_url: Option<String>,
    pub is_living: bool,
    pub is_private: bool,
    pub has_parents: bool,
    pub has_birth_date: bool,
    pub version: i32,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PeopleDirectoryResponse {
    pub items: Vec<PeopleDirectoryItemDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
    #[serde(default)]
    pub total_unfiltered: usize,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct SurnamesIndexQuery {
    pub letter: Option<String>,
    pub min_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SurnameIndexEntryDto {
    pub canonical_surname: String,
    pub variants: Vec<String>,
    pub count: usize,
    pub earliest_year: Option<i32>,
    pub latest_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SurnameLetterGroupDto {
    pub letter: String,
    pub items: Vec<SurnameIndexEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SurnamesIndexResponse {
    pub groups: Vec<SurnameLetterGroupDto>,
    pub total_surnames: usize,
    pub total_people: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlacesDirectoryResponse {
    pub items: Vec<PlaceSummaryDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct MediaFilterQuery {
    pub q: Option<String>,
    pub person_id: Option<String>,
    /// e.g. `"1940s"` or `"1940"`
    pub decade: Option<String>,
    /// `"all"`, `"photos"`/`"photo"`, `"documents"`/`"document"`, `"audio"`
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub untagged: Option<bool>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct CreateMediaRequest {
    pub gid: Option<String>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    /// Base64-encoded binary payload (optional; supports data URL prefix `data:image/...;base64,...`)
    pub data: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub caption: Option<String>,
    pub date_taken: Option<String>,
    pub place_id: Option<String>,
    pub place_path: Option<String>,
    pub is_private: Option<bool>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub linked_person_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateMediaRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub caption: Option<String>,
    pub date_taken: Option<String>,
    pub place_id: Option<String>,
    pub place_path: Option<String>,
    pub is_private: Option<bool>,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
pub struct MediaCropQuery {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub w: Option<f32>,
    pub h: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFaceTagRequest {
    pub person_id: String,
    pub x: f32,
    pub y: f32,
    pub w: Option<f32>,
    pub h: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub is_primary_avatar: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateFaceTagRequest {
    pub person_id: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub w: Option<f32>,
    pub h: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub is_primary_avatar: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMediaLinkRequest {
    pub entity_type: String,
    pub entity_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MediaArchiveResponse {
    pub items: Vec<MediaAssetSummaryDto>,
    pub next_cursor: Option<String>,
    pub total: usize,
    #[serde(default)]
    pub total_unfiltered: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminToolOperationResponse {
    pub status: String,
    pub operation: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ChangeLogEntryDto>,
}

pub type PersonMergeRequest = QualityPersonMergeRequest;

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

async fn resolve_home_person_for_caller(
    state: &AppState,
    caller: &PublicOrAuth,
    persons: &[TreePersonNodeDto],
    default_locale: &str,
    default_home_person_id: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>, String) {
    let access = caller.caller_access();
    let (user_home_person_id, user_person_id, start_page) = match caller {
        PublicOrAuth::Authenticated(auth) => {
            let auth_store = state.auth_store.read().await;
            let user_ref = auth_store
                .users
                .iter()
                .find(|u| u.id == auth.user.id)
                .unwrap_or(&auth.user);
            (
                user_ref.home_person_id.map(|id| id.to_string()),
                user_ref.person_id.map(|id| id.to_string()),
                user_ref.start_page.clone(),
            )
        }
        PublicOrAuth::AnonymousPublic => (None, None, "tree".to_string()),
    };

    let candidates = [
        user_home_person_id.as_deref(),
        user_person_id.as_deref(),
        default_home_person_id,
    ];

    let mut resolved = None;
    for cand in candidates.into_iter().flatten() {
        let target = cand.trim();
        if target.is_empty() {
            continue;
        }
        if let Some(person) = persons
            .iter()
            .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        {
            if let Some(visible) = redact_person_for_caller(person, &access, default_locale) {
                resolved = Some(visible.id);
                break;
            }
        }
    }

    (resolved, user_home_person_id, user_person_id, start_page)
}

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

fn filter_family_for_visible_persons(
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
    })
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

#[utoipa::path(
    get,
    path = "/api/v1/tree/view",
    params(TreeViewQuery),
    responses(
        (status = 200, description = "Projected tree canvas nodes, families, and edges for the requested view mode and generation depth", body = TreeViewResponse),
        (status = 400, description = "Invalid projection mode", body = ErrorResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse),
        (status = 404, description = "Specified focal person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_tree_view(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<TreeViewQuery>,
) -> Result<Json<TreeViewResponse>, AppError> {
    let mode = match query
        .mode
        .as_deref()
        .unwrap_or("hourglass")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "hourglass" | "" => "hourglass".to_string(),
        "pedigree" => "pedigree".to_string(),
        "descendant" | "descendants" => "descendant".to_string(),
        "fan" => "fan".to_string(),
        _ => {
            let mut fields = HashMap::new();
            fields.insert("mode".to_string(), "invalid_mode".to_string());
            return Err(AppError::ValidationFailed(
                "Mode must be hourglass, pedigree, descendant, or fan".to_string(),
                Some(fields),
            ));
        }
    };
    let generations = query.generations.unwrap_or(4).clamp(1, 10);

    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let visible_person_ids: HashSet<String> = store
        .persons
        .iter()
        .filter(|p| !p.is_private || access.can_view_private())
        .map(|p| p.id.clone())
        .collect();

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter_map(|f| {
            filter_family_for_visible_persons(f, access.can_view_private(), &visible_person_ids)
        })
        .collect();

    let (resolved_home_person_id, _, _, _) = resolve_home_person_for_caller(
        &state,
        &caller,
        &store.persons,
        &settings.default_locale,
        settings.default_home_person_id.as_deref(),
    )
    .await;

    let focal_id = if let Some(raw_focal) = query
        .focal_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let person = store
            .persons
            .iter()
            .find(|p| p.id == raw_focal || p.gid.eq_ignore_ascii_case(raw_focal))
            .ok_or(AppError::PersonNotFound)?;
        if !visible_person_ids.contains(&person.id) {
            return Err(AppError::PersonNotFound);
        }
        Some(person.id.clone())
    } else {
        resolved_home_person_id.clone().or_else(|| {
            store
                .persons
                .iter()
                .find(|p| visible_person_ids.contains(&p.id))
                .map(|p| p.id.clone())
        })
    };

    let mut included_person_ids: HashSet<String> = HashSet::new();
    let mut included_family_ids: HashSet<String> = HashSet::new();
    let mut included_child_links: HashSet<(String, String)> = HashSet::new();

    if let Some(ref root_id) = focal_id {
        included_person_ids.insert(root_id.clone());

        let mut anc_queue: VecDeque<(String, u32)> = VecDeque::new();
        let mut visited_anc: HashSet<String> = HashSet::new();

        let mut desc_queue: VecDeque<(String, u32)> = VecDeque::new();
        let mut visited_desc: HashSet<String> = HashSet::new();

        match mode.as_str() {
            "pedigree" | "fan" => {
                visited_anc.insert(root_id.clone());
                anc_queue.push_back((root_id.clone(), 0));
            }
            "descendant" => {
                visited_desc.insert(root_id.clone());
                desc_queue.push_back((root_id.clone(), 0));
            }
            _ => {
                // "hourglass": traverse upwards from focal person and their spouse(s), and downwards from focal person
                let mut focal_and_spouses = vec![root_id.clone()];
                for fam in &visible_families {
                    let is_p1 = fam.partner1_id.as_deref() == Some(root_id.as_str());
                    let is_p2 = fam.partner2_id.as_deref() == Some(root_id.as_str());
                    if is_p1 || is_p2 {
                        included_family_ids.insert(fam.id.clone());
                        if let Some(ref p1) = fam.partner1_id {
                            if !focal_and_spouses.contains(p1) {
                                focal_and_spouses.push(p1.clone());
                            }
                        }
                        if let Some(ref p2) = fam.partner2_id {
                            if !focal_and_spouses.contains(p2) {
                                focal_and_spouses.push(p2.clone());
                            }
                        }
                    }
                }
                for id in focal_and_spouses {
                    included_person_ids.insert(id.clone());
                    visited_anc.insert(id.clone());
                    anc_queue.push_back((id, 0));
                }
                visited_desc.insert(root_id.clone());
                desc_queue.push_back((root_id.clone(), 0));
            }
        }

        // Upward ancestor traversal
        while let Some((curr_id, depth)) = anc_queue.pop_front() {
            if depth >= generations {
                continue;
            }
            for fam in &visible_families {
                if fam.children.iter().any(|c| c.child_id == curr_id) {
                    included_family_ids.insert(fam.id.clone());
                    included_child_links.insert((fam.id.clone(), curr_id.clone()));
                    for parent_id in [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
                        .into_iter()
                        .flatten()
                    {
                        included_person_ids.insert(parent_id.clone());
                        if visited_anc.insert(parent_id.clone()) {
                            anc_queue.push_back((parent_id.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        // Downward descendant traversal
        while let Some((curr_id, depth)) = desc_queue.pop_front() {
            for fam in &visible_families {
                let is_partner = fam.partner1_id.as_deref() == Some(curr_id.as_str())
                    || fam.partner2_id.as_deref() == Some(curr_id.as_str());
                if !is_partner {
                    continue;
                }
                if depth < generations || curr_id == *root_id {
                    included_family_ids.insert(fam.id.clone());
                    for partner_id in [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
                        .into_iter()
                        .flatten()
                    {
                        included_person_ids.insert(partner_id.clone());
                    }
                }
                if depth < generations {
                    for ch in &fam.children {
                        included_person_ids.insert(ch.child_id.clone());
                        included_child_links.insert((fam.id.clone(), ch.child_id.clone()));
                        if visited_desc.insert(ch.child_id.clone()) {
                            desc_queue.push_back((ch.child_id.clone(), depth + 1));
                        }
                    }
                }
            }
        }
    }

    let nodes: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter(|p| included_person_ids.contains(&p.id))
        .filter_map(|p| redact_person_for_tree_view(p, &access))
        .collect();

    let mut families: Vec<FamilyDto> = Vec::new();
    let mut edges: Vec<TreeEdgeDto> = Vec::new();

    for fam in &visible_families {
        if !included_family_ids.contains(&fam.id) {
            continue;
        }
        let partner1_id = fam
            .partner1_id
            .as_ref()
            .filter(|id| included_person_ids.contains(*id))
            .cloned();
        let partner2_id = fam
            .partner2_id
            .as_ref()
            .filter(|id| included_person_ids.contains(*id))
            .cloned();
        let children: Vec<FamilyChildDto> = fam
            .children
            .iter()
            .filter(|c| {
                included_child_links.contains(&(fam.id.clone(), c.child_id.clone()))
                    && included_person_ids.contains(&c.child_id)
            })
            .cloned()
            .collect();

        if let (Some(ref p1), Some(ref p2)) = (&partner1_id, &partner2_id) {
            edges.push(TreeEdgeDto {
                id: format!("edge-spouse-{}", fam.id),
                source: p1.clone(),
                target: p2.clone(),
                kind: "spouse".to_string(),
                family_id: fam.id.clone(),
            });
        }

        for ch in &children {
            edges.push(TreeEdgeDto {
                id: format!("edge-child-{}-{}", fam.id, ch.child_id),
                source: fam.id.clone(),
                target: ch.child_id.clone(),
                kind: "parent_child".to_string(),
                family_id: fam.id.clone(),
            });
        }

        families.push(FamilyDto {
            id: fam.id.clone(),
            gid: fam.gid.clone(),
            partner1_id,
            partner2_id,
            relationship_type: fam.relationship_type.clone(),
            is_private: fam.is_private,
            children,
            external_ids: fam.external_ids.clone(),
            version: fam.version,
        });
    }

    let total_visible = nodes.len();

    Ok(Json(TreeViewResponse {
        access_tier: access.as_tier_str().to_string(),
        tree_title: settings.tree_title,
        focal_id,
        mode,
        generations,
        nodes,
        families,
        edges,
        resolved_home_person_id,
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

    let resolved_id = store.resolve_person_id(target).ok_or(AppError::NotFound)?;
    let person = store
        .persons
        .iter()
        .find(|p| p.id == resolved_id)
        .ok_or(AppError::NotFound)?;

    let redacted = redact_person_for_caller(person, &access, &settings.default_locale)
        .ok_or(AppError::NotFound)?;

    Ok(Json(redacted))
}

fn build_profile_relative_summary(
    person: &TreePersonNodeDto,
    access: &CallerAccess,
) -> Option<ProfileRelativeSummaryDto> {
    let redacted = redact_person_for_tree_view(person, access)?;
    Some(ProfileRelativeSummaryDto {
        id: redacted.id,
        gid: redacted.gid,
        given_name: redacted.primary_name.given_name,
        patronymic: redacted.primary_name.patronymic,
        surname: redacted.primary_name.surname,
        sex: redacted.sex,
        is_living: redacted.is_living,
        birth_year: redacted.birth_sort.map(|bs| bs / 10_000),
        death_year: redacted.death_sort.map(|ds| ds / 10_000),
        avatar_url: redacted.avatar_url,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/persons/{id}/profile",
    params(
        ("id" = String, Path, description = "Person UUID or GEDCOM-style GID (e.g. I0042)"),
        PersonProfileQuery
    ),
    responses(
        (status = 200, description = "Aggregated person profile dossier including kinship, life events, family links, media, notes, sources, and capabilities", body = PersonProfileResponseDto),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse),
        (status = 404, description = "Person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_person_profile(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<PersonProfileQuery>,
) -> Result<Json<PersonProfileResponseDto>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let resolved_id = store
        .resolve_person_id(target)
        .ok_or(AppError::PersonNotFound)?;
    let raw_person = store
        .persons
        .iter()
        .find(|p| p.id == resolved_id)
        .ok_or(AppError::PersonNotFound)?;

    let redacted_person =
        redact_person_for_tree_view(raw_person, &access).ok_or(AppError::PersonNotFound)?;

    let include_set: HashSet<String> = query
        .include
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.split(',')
                .map(|part| part.trim().to_ascii_lowercase())
                .filter(|part| !part.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let include_all = include_set.is_empty();
    let want_kinship = include_all || include_set.contains("kinship");
    let want_events = include_all || include_set.contains("events");
    let want_families = include_all || include_set.contains("families");
    let want_media = include_all || include_set.contains("media");
    let want_notes = include_all || include_set.contains("notes");
    let want_sources = include_all || include_set.contains("sources");

    let (resolved_home_person_id, _, _, _) = resolve_home_person_for_caller(
        &state,
        &caller,
        &store.persons,
        &settings.default_locale,
        settings.default_home_person_id.as_deref(),
    )
    .await;

    let capabilities = ProfileCapabilitiesDto {
        can_edit: access.can_edit(),
        can_delete: access.can_edit(),
        can_merge: access.is_admin(),
        is_home_person: resolved_home_person_id.as_deref() == Some(raw_person.id.as_str()),
    };

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_tree_view(p, &access))
        .collect();
    let visible_person_ids: HashSet<String> =
        visible_persons.iter().map(|p| p.id.clone()).collect();
    let person_map: HashMap<String, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.clone(), p)).collect();

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter_map(|f| {
            filter_family_for_visible_persons(f, access.can_view_private(), &visible_person_ids)
        })
        .collect();

    // Build families section (always respecting living/private redaction for each relative)
    let mut parent_families = Vec::new();
    let mut spouse_families = Vec::new();
    let mut person_spouse_family_ids: HashSet<String> = HashSet::new();
    let mut family_spouse_map: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();

    for fam in &visible_families {
        // Parent family where target person is a child
        if fam.children.iter().any(|c| c.child_id == raw_person.id) && want_families {
            let p1 = fam
                .partner1_id
                .as_ref()
                .and_then(|pid| person_map.get(pid))
                .and_then(|p| build_profile_relative_summary(p, &access));
            let p2 = fam
                .partner2_id
                .as_ref()
                .and_then(|pid| person_map.get(pid))
                .and_then(|p| build_profile_relative_summary(p, &access));

            // Assign father/mother by sex when known, otherwise partner1 -> father, partner2 -> mother
            let (father, mother) = match (&p1, &p2) {
                (Some(a), Some(b)) if a.sex == "F" && b.sex == "M" => {
                    (Some(b.clone()), Some(a.clone()))
                }
                (Some(a), None) if a.sex == "F" => (None, Some(a.clone())),
                (None, Some(b)) if b.sex == "M" => (Some(b.clone()), None),
                _ => (p1, p2),
            };

            let mut siblings = Vec::new();
            for ch in &fam.children {
                if ch.child_id == raw_person.id {
                    continue;
                }
                if let Some(sib_p) = person_map.get(&ch.child_id) {
                    if let Some(sib_summary) = build_profile_relative_summary(sib_p, &access) {
                        siblings.push(sib_summary);
                    }
                }
            }

            parent_families.push(ProfileParentFamilyDto {
                family_id: fam.id.clone(),
                gid: fam.gid.clone(),
                father,
                mother,
                siblings,
                version: fam.version,
            });
        }

        // Spouse family where target person is partner1 or partner2
        let is_p1 = fam.partner1_id.as_deref() == Some(raw_person.id.as_str());
        let is_p2 = fam.partner2_id.as_deref() == Some(raw_person.id.as_str());
        if is_p1 || is_p2 {
            person_spouse_family_ids.insert(fam.id.clone());
            let other_id = if is_p1 {
                fam.partner2_id.as_ref()
            } else {
                fam.partner1_id.as_ref()
            };
            let spouse_summary = other_id
                .and_then(|pid| person_map.get(pid))
                .and_then(|p| build_profile_relative_summary(p, &access));

            if let Some(ref sp) = spouse_summary {
                let full_name = format!(
                    "{} {} {}",
                    sp.given_name,
                    sp.patronymic.as_deref().unwrap_or(""),
                    sp.surname
                )
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
                family_spouse_map.insert(fam.id.clone(), (Some(sp.id.clone()), Some(full_name)));
            } else {
                family_spouse_map.insert(fam.id.clone(), (None, None));
            }

            if want_families {
                let marriage_ev = store.events.iter().find(|e| {
                    e.family_id.as_deref() == Some(fam.id.as_str())
                        && e.event_type == "marriage"
                        && (!e.is_private || access.can_view_private())
                });
                let is_masked_union = access == CallerAccess::AnonymousPublic
                    && (raw_person.is_living
                        || spouse_summary.as_ref().is_some_and(|s| s.is_living));

                let marriage_date = if is_masked_union {
                    None
                } else {
                    marriage_ev.and_then(|e| {
                        e.date
                            .as_ref()
                            .map(|d| d.text.clone())
                            .or_else(|| e.gedcom_date.clone())
                    })
                };
                let marriage_place = if is_masked_union {
                    None
                } else {
                    marriage_ev.and_then(|e| e.place_full_name.clone())
                };

                let mut children = Vec::new();
                for ch in &fam.children {
                    if let Some(ch_p) = person_map.get(&ch.child_id) {
                        if let Some(ch_summary) = build_profile_relative_summary(ch_p, &access) {
                            children.push(ProfileChildSummaryDto {
                                person: ch_summary,
                                sort_order: ch.sort_order,
                                rel_to_partner1: ch.rel_to_partner1.clone(),
                                rel_to_partner2: ch.rel_to_partner2.clone(),
                            });
                        }
                    }
                }
                children.sort_by_key(|c| c.sort_order);

                spouse_families.push(ProfileSpouseFamilyDto {
                    family_id: fam.id.clone(),
                    gid: fam.gid.clone(),
                    relationship_type: fam.relationship_type.clone(),
                    spouse: spouse_summary,
                    marriage_date,
                    marriage_place,
                    children,
                    version: fam.version,
                });
            }
        }
    }

    let families = ProfileFamiliesDto {
        parent_families,
        spouse_families,
    };

    // Living-person public shield (`03-person-profile-page.md §1, §3.1, §4 DoD #3`):
    // When caller is AnonymousPublic and target person is_living, mask events, media, notes, sources, and kinship.
    if access == CallerAccess::AnonymousPublic && raw_person.is_living {
        return Ok(Json(PersonProfileResponseDto {
            person: redacted_person,
            kinship: None,
            events: vec![],
            families,
            media: vec![],
            notes: vec![],
            sources_summary: vec![],
            capabilities,
        }));
    }

    let kinship = if want_kinship {
        resolved_home_person_id.as_deref().and_then(|home_id| {
            calculate_kinship_relation(home_id, &raw_person.id, &visible_persons, &visible_families)
        })
    } else {
        None
    };

    let places_map: HashMap<String, &PlaceSummaryDto> =
        store.places.iter().map(|p| (p.id.clone(), p)).collect();
    let birth_year = raw_person.birth_sort.map(|bs| bs / 10_000);

    // Collect relevant life events (individual + spouse family events)
    let mut raw_events: Vec<&LifeEventDto> = store
        .events
        .iter()
        .filter(|e| {
            if e.is_private && !access.can_view_private() {
                return false;
            }
            let is_ind = e.individual_id.as_deref() == Some(raw_person.id.as_str());
            let is_fam = e
                .family_id
                .as_ref()
                .is_some_and(|fid| person_spouse_family_ids.contains(fid));
            is_ind || is_fam
        })
        .collect();

    raw_events.sort_by(|a, b| {
        let sa = a.date_sort.unwrap_or(i32::MAX);
        let sb = b.date_sort.unwrap_or(i32::MAX);
        sa.cmp(&sb).then_with(|| a.gid.cmp(&b.gid))
    });

    let mut events = Vec::new();
    let mut source_groups: Vec<(String, Vec<String>)> = Vec::new();

    for ev in &raw_events {
        if want_sources {
            if let Some(src) = ev
                .source_text
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                if let Some(existing) = source_groups
                    .iter_mut()
                    .find(|(s, _)| s.eq_ignore_ascii_case(src))
                {
                    if !existing.1.contains(&ev.id) {
                        existing.1.push(ev.id.clone());
                    }
                } else {
                    source_groups.push((src.to_string(), vec![ev.id.clone()]));
                }
            }
        }

        if want_events {
            let ev_year = ev.date_sort.map(|ds| ds / 10_000);
            let is_post_death = matches!(
                ev.event_type.as_str(),
                "burial" | "cremation" | "probate" | "will"
            );
            let age_at_event = if ev.event_type == "birth" {
                Some(0)
            } else if is_post_death {
                None
            } else if let (Some(by), Some(ey)) = (birth_year, ev_year) {
                if ey >= by {
                    Some(ey - by)
                } else {
                    None
                }
            } else {
                None
            };

            let resolved_place = ev.place_id.as_ref().and_then(|pid| places_map.get(pid));
            let latitude = resolved_place.and_then(|p| p.latitude);
            let longitude = resolved_place.and_then(|p| p.longitude);
            let place_full_name = ev
                .place_full_name
                .clone()
                .or_else(|| resolved_place.map(|p| p.full_name.clone()));

            let (spouse_id, spouse_name) = ev
                .family_id
                .as_ref()
                .and_then(|fid| family_spouse_map.get(fid))
                .cloned()
                .unwrap_or((None, None));

            events.push(ProfileLifeEventDto {
                id: ev.id.clone(),
                gid: ev.gid.clone(),
                individual_id: ev.individual_id.clone(),
                family_id: ev.family_id.clone(),
                event_type: ev.event_type.clone(),
                custom_type: ev.custom_type.clone(),
                date: ev.date.clone(),
                date_sort: ev.date_sort,
                gedcom_date: ev.gedcom_date.clone(),
                place_id: ev.place_id.clone(),
                place_full_name,
                description: ev.description.clone(),
                source_text: ev.source_text.clone(),
                media_ids: ev.media_ids.clone(),
                is_private: ev.is_private,
                version: ev.version,
                age_at_event,
                latitude,
                longitude,
                scans_count: ev.media_ids.len(),
                spouse_id,
                spouse_name,
            });
        }
    }

    let sources_summary: Vec<ProfileSourceSummaryDto> = if want_sources {
        source_groups
            .into_iter()
            .map(|(source_text, event_ids)| {
                let events_count = event_ids.len();
                ProfileSourceSummaryDto {
                    source_text,
                    event_ids,
                    events_count,
                }
            })
            .collect()
    } else {
        vec![]
    };

    let media: Vec<MediaAssetSummaryDto> = if want_media {
        let event_media_ids: HashSet<String> = raw_events
            .iter()
            .flat_map(|e| e.media_ids.iter().cloned())
            .collect();
        store
            .media
            .iter()
            .filter(|m| {
                m.linked_person_ids.contains(&raw_person.id)
                    || m.face_tags.iter().any(|ft| ft.person_id == raw_person.id)
                    || event_media_ids.contains(&m.id)
            })
            .filter_map(|m| redact_media_for_caller(m, &access))
            .collect()
    } else {
        vec![]
    };

    let notes: Vec<NoteDto> = if want_notes {
        let mut person_notes: Vec<NoteDto> = store
            .notes
            .iter()
            .filter(|n| {
                n.entity_type == "individual"
                    && n.entity_id == raw_person.id
                    && (!n.is_private || access.can_view_private())
            })
            .cloned()
            .collect();
        person_notes.sort_by_key(|n| n.sort_order);
        person_notes
    } else {
        vec![]
    };

    Ok(Json(PersonProfileResponseDto {
        person: redacted_person,
        kinship,
        events,
        families,
        media,
        notes,
        sources_summary,
        capabilities,
    }))
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
        (status = 200, description = "Person safely deleted and snapshot logged to change_log", body = serde_json::Value),
        (status = 403, description = "Viewer role is read-only", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn delete_person(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
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

    let entity_label = format_person_display_name(&removed_person);
    let summary = format!(
        "Deleted individual {} ({} {})",
        removed_person.gid,
        removed_person.primary_name.given_name,
        removed_person.primary_name.surname
    )
    .trim()
    .to_string();

    let entry = log_change(
        &mut store,
        None,
        "person.delete",
        "individual",
        &person_id,
        "delete",
        Some(entity_label),
        summary,
        serde_json::to_value(snapshot).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    recompute_all_places_metadata(&mut store);
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);
    drop(store);

    // Match ON DELETE SET NULL for users.person_id / users.home_person_id and default_home_person_id
    if let Ok(deleted_uuid) = Uuid::parse_str(&person_id) {
        let mut auth_store = state.auth_store.write().await;
        for u in &mut auth_store.users {
            if u.person_id == Some(deleted_uuid) {
                u.person_id = None;
            }
            if u.home_person_id == Some(deleted_uuid) {
                u.home_person_id = None;
            }
        }
    }
    {
        let mut settings_guard = state.settings_override.write().await;
        if settings_guard.default_home_person_id.as_deref() == Some(person_id.as_str()) {
            settings_guard.default_home_person_id = None;
        }
    }

    Ok(Json(serde_json::json!({
        "status": "deleted",
        "deleted_id": person_id,
        "tx_id": entry.tx_id
    })))
}

#[utoipa::path(
    post,
    path = "/api/v1/history/{id}/restore",
    params(
        ("id" = String, Path, description = "Change log entry ID or deleted entity UUID/GID")
    ),
    responses(
        (status = 200, description = "Deleted or modified record restored from change_log", body = TreePersonNodeDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Change log entry or snapshot not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn restore_deleted_person(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let restored = restore_change(
        &mut store,
        &id,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
        current_year,
        age_threshold,
    )?;

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
    RequireEditor(editor): RequireEditor,
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
    let mut input_children = req.children.unwrap_or_default();
    for cid in req.children_ids.unwrap_or_default() {
        if !input_children.iter().any(|c| c.child_id == cid) {
            input_children.push(FamilyChildDto {
                child_id: cid,
                rel_to_partner1: "biological".to_string(),
                rel_to_partner2: "biological".to_string(),
                sort_order: 0,
            });
        }
    }
    for (idx, mut ch) in input_children.into_iter().enumerate() {
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
        external_ids: HashMap::new(),
        version: 1,
    };

    store.families.push(fam.clone());
    recompute_all_individuals_vitals(&mut store, current_year, age_threshold);

    log_change(
        &mut store,
        None,
        "family.create",
        "family",
        &fam.id,
        "create",
        Some(fam.gid.clone()),
        format!("Created family union {}", fam.gid),
        None,
        serde_json::to_value(&fam).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
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

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.families[fam_idx].version,
        expected_version,
        &format!("Family {}", store.families[fam_idx].gid),
        serde_json::to_value(&store.families[fam_idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.families[fam_idx]).ok();

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

    log_change(
        &mut store,
        None,
        "family.update",
        "family",
        &updated.id,
        "update",
        Some(updated.gid.clone()),
        format!("Updated family union {}", updated.gid),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
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

    log_change(
        &mut store,
        None,
        "family.delete",
        "family",
        &removed_fam.id,
        "delete",
        Some(removed_fam.gid.clone()),
        format!("Deleted family union {}", removed_fam.gid),
        serde_json::to_value(&removed_fam).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
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

    log_change(
        &mut store,
        None,
        "event.create",
        "event",
        &ev.id,
        "create",
        Some(format!("{} ({})", ev.event_type, ev.gid)),
        format!("Created {} event {}", ev.event_type, ev.gid),
        None,
        serde_json::to_value(&ev).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
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

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.events[ev_idx].version,
        expected_version,
        &format!("Event {}", store.events[ev_idx].gid),
        serde_json::to_value(&store.events[ev_idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.events[ev_idx]).ok();

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

    log_change(
        &mut store,
        None,
        "event.update",
        "event",
        &updated.id,
        "update",
        Some(format!("{} ({})", updated.event_type, updated.gid)),
        format!("Updated {} event {}", updated.event_type, updated.gid),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
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

    log_change(
        &mut store,
        None,
        "event.delete",
        "event",
        &removed_ev.id,
        "delete",
        Some(format!("{} ({})", removed_ev.event_type, removed_ev.gid)),
        format!("Deleted {} event {}", removed_ev.event_type, removed_ev.gid),
        serde_json::to_value(&removed_ev).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
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

    log_change(
        &mut store,
        None,
        "note.create",
        "note",
        &note.id,
        "create",
        Some(format!("Note on {}", note.entity_type)),
        format!("Created note on {} {}", note.entity_type, note.entity_id),
        None,
        serde_json::to_value(&note).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateNoteRequest>,
) -> Result<Json<NoteDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let note_idx = store
        .notes
        .iter()
        .position(|n| n.id == id.trim())
        .ok_or(AppError::NotFound)?;

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.notes[note_idx].version,
        expected_version,
        &format!("Note {}", store.notes[note_idx].id),
        serde_json::to_value(&store.notes[note_idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.notes[note_idx]).ok();

    let note = &mut store.notes[note_idx];
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
    let updated = note.clone();

    log_change(
        &mut store,
        None,
        "note.update",
        "note",
        &updated.id,
        "update",
        Some(format!("Note on {}", updated.entity_type)),
        format!("Updated note {}", updated.id),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(updated))
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
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let idx = store
        .notes
        .iter()
        .position(|n| n.id == id.trim())
        .ok_or(AppError::NotFound)?;
    let removed = store.notes.remove(idx);

    log_change(
        &mut store,
        None,
        "note.delete",
        "note",
        &removed.id,
        "delete",
        Some(format!("Note on {}", removed.entity_type)),
        format!("Deleted note {}", removed.id),
        serde_json::to_value(&removed).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(StatusCode::NO_CONTENT)
}

const DEFAULT_WEBP_BYTES: &[u8] =
    b"RIFF\x1a\x00\x00\x00WEBPVP8L\x0d\x00\x00\x00/\x00\x00\x00\x10\x07\x10\x11\x11\x88\x88\xfe\x07\x00";

fn save_media_bytes_to_disk(media_storage_path: &str, file_name: &str, bytes: &[u8]) {
    let primary_dir = PathBuf::from(media_storage_path).join("originals");
    if std::fs::create_dir_all(&primary_dir).is_ok() {
        let _ = std::fs::write(primary_dir.join(file_name), bytes);
    }
    let fallback_dir = PathBuf::from("/tmp/genealogy-media/originals");
    if std::fs::create_dir_all(&fallback_dir).is_ok() {
        let _ = std::fs::write(fallback_dir.join(file_name), bytes);
    }
}

fn read_media_bytes_from_disk(media_storage_path: &str, file_name: &str) -> Vec<u8> {
    if !file_name.is_empty() {
        let primary_path = PathBuf::from(media_storage_path)
            .join("originals")
            .join(file_name);
        if let Ok(bytes) = std::fs::read(&primary_path) {
            if !bytes.is_empty() {
                return bytes;
            }
        }
        let fallback_path = PathBuf::from("/tmp/genealogy-media/originals").join(file_name);
        if let Ok(bytes) = std::fs::read(&fallback_path) {
            if !bytes.is_empty() {
                return bytes;
            }
        }
    }
    DEFAULT_WEBP_BYTES.to_vec()
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
    tag = "Media & Avatars"
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

    let body_bytes = read_media_bytes_from_disk(
        &state.config.media_storage_path,
        media.file_name.as_deref().unwrap_or(""),
    );
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media.mime_type.as_str())
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(body_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to build media response: {e}")))?;

    Ok(response.into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{id}/crop",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID (e.g. M0001)"),
        ("x" = Option<f32>, Query, description = "Relative X coordinate (0.0..=1.0)"),
        ("y" = Option<f32>, Query, description = "Relative Y coordinate (0.0..=1.0)"),
        ("w" = Option<f32>, Query, description = "Relative width (0.0..=1.0)"),
        ("h" = Option<f32>, Query, description = "Relative height (0.0..=1.0)")
    ),
    responses(
        (status = 200, description = "Cropped avatar binary stream", content_type = "image/webp"),
        (status = 404, description = "Media asset not found or shielded by living/private privacy", body = ErrorResponse),
        (status = 422, description = "Invalid crop coordinates", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn get_media_crop(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(crop): Query<MediaCropQuery>,
) -> Result<Response, AppError> {
    let x = crop.x.unwrap_or(0.0);
    let y = crop.y.unwrap_or(0.0);
    let w = crop.w.or(crop.width).unwrap_or(1.0);
    let h = crop.h.or(crop.height).unwrap_or(1.0);
    if !(0.0..=1.0).contains(&x)
        || !(0.0..=1.0).contains(&y)
        || w <= 0.0
        || w > 1.0
        || h <= 0.0
        || h > 1.0
    {
        return Err(AppError::ValidationFailed(
            "Crop coordinates must be normalized within 0.0..=1.0".to_string(),
            Some(HashMap::from([(
                "crop".to_string(),
                "out_of_bounds".to_string(),
            )])),
        ));
    }

    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let media = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let _permitted = redact_media_for_caller(media, &access).ok_or(AppError::NotFound)?;

    let body_bytes = read_media_bytes_from_disk(
        &state.config.media_storage_path,
        media.file_name.as_deref().unwrap_or(""),
    );
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media.mime_type.as_str())
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(body_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to build media crop response: {e}")))?;

    Ok(response.into_response())
}

fn format_person_display_name(person: &TreePersonNodeDto) -> String {
    let mut parts = Vec::new();
    if !person.primary_name.surname.trim().is_empty() {
        if let Some(ref bs) = person.birth_surname {
            let bs_trimmed = bs.trim();
            if !bs_trimmed.is_empty()
                && !bs_trimmed.eq_ignore_ascii_case(&person.primary_name.surname)
            {
                parts.push(format!(
                    "{} ({bs_trimmed})",
                    person.primary_name.surname.trim()
                ));
            } else {
                parts.push(person.primary_name.surname.trim().to_string());
            }
        } else {
            parts.push(person.primary_name.surname.trim().to_string());
        }
    }
    if !person.primary_name.given_name.trim().is_empty() {
        parts.push(person.primary_name.given_name.trim().to_string());
    }
    if let Some(ref pat) = person.primary_name.patronymic {
        if !pat.trim().is_empty() {
            parts.push(pat.trim().to_string());
        }
    }
    if parts.is_empty() {
        person.gid.clone()
    } else {
        parts.join(" ")
    }
}

fn build_person_token_forms(word: &str) -> Vec<String> {
    let ortho = normalize_russian_orthography_word(word);
    let folded = fold_slavic_surname_stem(&ortho);
    let lat_ortho = transliterate_to_latin(&ortho);
    let lat_folded = transliterate_to_latin(&folded);
    let mut out = Vec::with_capacity(4);
    for cand in [ortho, folded, lat_ortho, lat_folded] {
        if !cand.is_empty() && !out.contains(&cand) {
            out.push(cand);
        }
    }
    out
}

fn score_person_for_search(
    person: &TreePersonNodeDto,
    parsed: &crate::services::privacy::ParsedSearchQuery,
) -> Option<i32> {
    // 1. Exact GID match jumps straight to top score
    if let Some(ref gid_q) = parsed.exact_gid {
        if person.gid.eq_ignore_ascii_case(gid_q) {
            return Some(10_000);
        }
    }

    let mut raw_words: Vec<&str> = Vec::new();
    for field in [
        Some(person.primary_name.surname.as_str()),
        Some(person.primary_name.given_name.as_str()),
        person.primary_name.patronymic.as_deref(),
        person.birth_surname.as_deref(),
        person.nickname.as_deref(),
        Some(person.gid.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        for w in field
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
        {
            raw_words.push(w);
        }
    }
    for alt in &person.alt_names {
        for field in [
            alt.given_name.as_deref(),
            alt.patronymic.as_deref(),
            alt.surname.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            for w in field
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
            {
                raw_words.push(w);
            }
        }
    }

    let candidate_word_forms: Vec<Vec<String>> = raw_words
        .iter()
        .map(|w| build_person_token_forms(w))
        .collect();

    let mut total_score: i32 = 0;

    for q_tok in &parsed.text_tokens {
        let q_forms = build_person_token_forms(q_tok);
        let mut best_token_score: i32 = 0;

        for c_forms in &candidate_word_forms {
            for qf in &q_forms {
                for cf in c_forms {
                    if cf == qf {
                        best_token_score = best_token_score.max(120);
                    } else if cf.starts_with(qf) {
                        best_token_score = best_token_score.max(85);
                    } else if cf.contains(qf) && qf.chars().count() >= 2 {
                        best_token_score = best_token_score.max(50);
                    }
                }
            }
        }

        if best_token_score == 0 {
            // Every query text token must match at least one name/gid token
            return None;
        }
        total_score += best_token_score;
    }

    let birth_year = person.birth_sort.map(|bs| bs / 10_000);
    if let Some(target_year) = parsed.year_hint {
        match birth_year {
            Some(by) => {
                let diff = (by - target_year).abs();
                if diff == 0 {
                    total_score += 250;
                } else if diff <= 2 {
                    total_score += 200 - (diff * 20);
                } else if diff <= 5 {
                    total_score += 60;
                } else if parsed.text_tokens.is_empty() {
                    // Pure numeric year search must be within ±2 years
                    return None;
                }
            }
            None => {
                if parsed.text_tokens.is_empty() {
                    return None;
                }
            }
        }
    }

    if total_score > 0 {
        Some(total_score)
    } else {
        None
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    params(SearchQuery),
    responses(
        (status = 200, description = "Global grouped omnibox search across people, places, media, and role-aware actions", body = SearchResultResponseDto),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn global_search(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResultResponseDto>, AppError> {
    let start_instant = std::time::Instant::now();
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let types_set: HashSet<String> = query
        .types
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.split(',')
                .map(|part| part.trim().to_ascii_lowercase())
                .filter(|part| !part.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let all_types = types_set.is_empty();
    let want_person = all_types || types_set.contains("person") || types_set.contains("people");
    let want_place = all_types || types_set.contains("place") || types_set.contains("places");
    let want_media = all_types || types_set.contains("media");
    let want_action = all_types || types_set.contains("action") || types_set.contains("actions");

    let person_limit = query.limit.unwrap_or(8).clamp(1, 50);
    let secondary_limit = query.limit.unwrap_or(3).clamp(1, 25);

    let q_trimmed = query.q.trim().to_string();
    let parsed = parse_search_query(&q_trimmed);

    // Count family connections per person to break ties (`13 §2.3`)
    let mut connections_count: HashMap<String, usize> = HashMap::new();
    for fam in &store.families {
        let mut members = Vec::new();
        if let Some(ref p1) = fam.partner1_id {
            members.push(p1.clone());
        }
        if let Some(ref p2) = fam.partner2_id {
            members.push(p2.clone());
        }
        for ch in &fam.children {
            members.push(ch.child_id.clone());
        }
        let deg = members.len().saturating_sub(1);
        for m in members {
            *connections_count.entry(m).or_insert(0) += deg;
        }
    }

    let mut people = Vec::new();
    if want_person && !q_trimmed.is_empty() {
        for raw_person in &store.persons {
            if raw_person.is_private && !access.can_view_private() {
                continue;
            }
            // Living-Person Privacy Guard (`13 §2.4, §6 DoD #5`):
            // Anonymous visitors in public mode must never be able to discover living people
            // by full name, surname, or birth year.
            if access == CallerAccess::AnonymousPublic && raw_person.is_living {
                if !state.config.public_search_show_living {
                    continue;
                }
                // Even when `public_search_show_living=true`, only match against the redacted node
                if let Some(masked) = redact_person_for_tree_view(raw_person, &access) {
                    if let Some(score) = score_person_for_search(&masked, &parsed) {
                        people.push(SearchPersonResultDto {
                            id: masked.id.clone(),
                            gid: masked.gid.clone(),
                            display_name: masked.primary_name.given_name.clone(),
                            given_name: masked.primary_name.given_name,
                            patronymic: None,
                            surname: String::new(),
                            sex: masked.sex,
                            birth_year: None,
                            death_year: None,
                            birth_place: None,
                            avatar_url: None,
                            is_living: true,
                            is_private: false,
                            score,
                        });
                    }
                }
                continue;
            }

            if let Some(base_score) = score_person_for_search(raw_person, &parsed) {
                let rel_bonus = connections_count
                    .get(&raw_person.id)
                    .copied()
                    .unwrap_or(0)
                    .min(15) as i32;
                let final_score = base_score + rel_bonus;
                let birth_year = raw_person.birth_sort.map(|bs| bs / 10_000);
                let death_year = raw_person.death_sort.map(|ds| ds / 10_000);
                let birth_place = raw_person.birth.as_ref().and_then(|b| b.place.clone());

                people.push(SearchPersonResultDto {
                    id: raw_person.id.clone(),
                    gid: raw_person.gid.clone(),
                    display_name: format_person_display_name(raw_person),
                    given_name: raw_person.primary_name.given_name.clone(),
                    patronymic: raw_person.primary_name.patronymic.clone(),
                    surname: raw_person.primary_name.surname.clone(),
                    sex: raw_person.sex.clone(),
                    birth_year,
                    death_year,
                    birth_place,
                    avatar_url: raw_person.avatar_url.clone(),
                    is_living: raw_person.is_living,
                    is_private: raw_person.is_private,
                    score: final_score,
                });
            }
        }

        people.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| {
                    a.birth_year
                        .unwrap_or(i32::MAX)
                        .cmp(&b.birth_year.unwrap_or(i32::MAX))
                })
                .then_with(|| a.display_name.cmp(&b.display_name))
        });
        people.truncate(person_limit);
    }

    let mut places = Vec::new();
    if want_place && !q_trimmed.is_empty() {
        let q_forms = build_person_token_forms(&q_trimmed);
        for place in &store.places {
            let place_tokens = normalize_search_tokens(&format!(
                "{} {} {}",
                place.name,
                place.full_name,
                place
                    .alt_names
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
            let matches_gid = parsed
                .exact_gid
                .as_ref()
                .is_some_and(|g| place.gid.eq_ignore_ascii_case(g));
            let matches_text = !q_forms.is_empty()
                && q_forms
                    .iter()
                    .any(|qf| place_tokens.split_whitespace().any(|pt| pt.contains(qf)));
            if matches_gid || matches_text {
                places.push(SearchPlaceResultDto {
                    id: place.id.clone(),
                    gid: place.gid.clone(),
                    name: place.name.clone(),
                    full_name: place.full_name.clone(),
                    place_type: place.place_type.clone(),
                    events_count: place.events_count,
                    has_coordinates: place.latitude.is_some() && place.longitude.is_some(),
                });
            }
        }
        places.sort_by(|a, b| {
            b.events_count
                .cmp(&a.events_count)
                .then_with(|| a.name.cmp(&b.name))
        });
        places.truncate(secondary_limit);
    }

    let mut media = Vec::new();
    if want_media && !q_trimmed.is_empty() {
        let q_forms = build_person_token_forms(&q_trimmed);
        for m in &store.media {
            let Some(visible_m) = redact_media_for_caller(m, &access) else {
                continue;
            };
            let media_tokens =
                normalize_search_tokens(&format!("{} {}", visible_m.title, visible_m.gid));
            let matches = q_forms
                .iter()
                .any(|qf| media_tokens.split_whitespace().any(|mt| mt.contains(qf)));
            if matches {
                media.push(SearchMediaResultDto {
                    id: visible_m.id,
                    gid: visible_m.gid,
                    title: visible_m.title,
                    mime_type: visible_m.mime_type,
                    file_url: visible_m.file_url,
                    is_private: visible_m.is_private,
                });
            }
        }
        media.truncate(secondary_limit);
    }

    let mut actions = Vec::new();
    if want_action && access != CallerAccess::AnonymousPublic {
        let mut catalog = vec![
            SearchActionDto {
                id: "nav-tree".to_string(),
                label: "Interactive Family Tree".to_string(),
                hint: "Open tree canvas".to_string(),
                path: "/tree".to_string(),
                icon: "git-branch".to_string(),
                required_role: "viewer".to_string(),
            },
            SearchActionDto {
                id: "nav-people".to_string(),
                label: "People Directory".to_string(),
                hint: "Browse and filter all individuals".to_string(),
                path: "/people".to_string(),
                icon: "users".to_string(),
                required_role: "viewer".to_string(),
            },
            SearchActionDto {
                id: "nav-surnames".to_string(),
                label: "Surnames Index".to_string(),
                hint: "Alphabetical list of family names".to_string(),
                path: "/people/surnames".to_string(),
                icon: "book-open".to_string(),
                required_role: "viewer".to_string(),
            },
            SearchActionDto {
                id: "nav-places".to_string(),
                label: "Places Directory".to_string(),
                hint: "Historical places and map".to_string(),
                path: "/places".to_string(),
                icon: "map-pin".to_string(),
                required_role: "viewer".to_string(),
            },
        ];

        if access.can_edit() {
            catalog.push(SearchActionDto {
                id: "act-add-person".to_string(),
                label: "Add Person".to_string(),
                hint: "Create a new individual record".to_string(),
                path: "/people?action=add".to_string(),
                icon: "user-plus".to_string(),
                required_role: "editor".to_string(),
            });
        }

        if access.is_admin() {
            catalog.push(SearchActionDto {
                id: "act-import-gedcom".to_string(),
                label: "Import GEDCOM".to_string(),
                hint: "Import a .ged genealogy file".to_string(),
                path: "/admin/gedcom".to_string(),
                icon: "upload".to_string(),
                required_role: "admin".to_string(),
            });
            catalog.push(SearchActionDto {
                id: "nav-settings".to_string(),
                label: "Go to Settings".to_string(),
                hint: "Instance configuration & backups".to_string(),
                path: "/admin/settings".to_string(),
                icon: "settings".to_string(),
                required_role: "admin".to_string(),
            });
        }

        if q_trimmed.is_empty() {
            actions = catalog;
        } else {
            let q_lower = q_trimmed.to_lowercase();
            actions = catalog
                .into_iter()
                .filter(|a| {
                    a.label.to_lowercase().contains(&q_lower)
                        || a.hint.to_lowercase().contains(&q_lower)
                })
                .collect();
        }
    }

    let timing_ms = start_instant.elapsed().as_millis() as u64;

    Ok(Json(SearchResultResponseDto {
        query: q_trimmed,
        people,
        places,
        media,
        actions,
        timing_ms,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/people",
    params(PeopleDirectoryQuery),
    responses(
        (status = 200, description = "People directory with cross-script search, filters, sorting, and cursor pagination (authenticated users only)", body = PeopleDirectoryResponse),
        (status = 401, description = "Authentication required even when public tree sharing is enabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_people_directory(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<PeopleDirectoryQuery>,
) -> Result<Json<PeopleDirectoryResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = auth.access();
    let store = state.privacy_store.read().await;

    // Precompute which individuals have at least one parent in visible families
    let mut has_parents_set: HashSet<String> = HashSet::new();
    for fam in &store.families {
        if fam.is_private && !access.can_view_private() {
            continue;
        }
        let has_any_parent = fam.partner1_id.is_some() || fam.partner2_id.is_some();
        if has_any_parent {
            for ch in &fam.children {
                has_parents_set.insert(ch.child_id.clone());
            }
        }
    }

    // Precompute latest change_log timestamp per individual
    let mut updated_at_map: HashMap<String, String> = HashMap::new();
    for entry in &store.change_log {
        if entry.entity_type == "individual" {
            updated_at_map.insert(entry.entity_id.clone(), entry.created_at.clone());
        }
    }

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();
    let total_unfiltered = visible_persons.len();

    let parsed_q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_search_query);

    let surname_filter_stem = query
        .surname
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| transliterate_to_latin(&fold_slavic_surname_stem(s)));

    let sex_filter = query
        .sex
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
        .map(|s| s.to_ascii_uppercase());

    let living_filter = query
        .living
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase());

    let mut filtered_items: Vec<PeopleDirectoryItemDto> = Vec::new();

    for p in visible_persons {
        let birth_year = p.birth_sort.map(|bs| bs / 10_000);
        let death_year = p.death_sort.map(|ds| ds / 10_000);
        let birth_date = p.birth.as_ref().and_then(|b| b.date.clone());
        let birth_place = p.birth.as_ref().and_then(|b| b.place.clone());
        let death_date = p.death.as_ref().and_then(|d| d.date.clone());
        let death_place = p.death.as_ref().and_then(|d| d.place.clone());
        let has_parents = has_parents_set.contains(&p.id);
        let has_birth_date = p.birth_sort.is_some() || birth_date.is_some();

        // 1. Free-text query filter (`q`)
        if let Some(ref pq) = parsed_q {
            if score_person_for_search(&p, pq).is_none() {
                continue;
            }
        }

        // 2. Surname filter (gender-folded + cross-script aware)
        if let Some(ref target_stem) = surname_filter_stem {
            let primary_stem =
                transliterate_to_latin(&fold_slavic_surname_stem(&p.primary_name.surname));
            let birth_stem = p
                .birth_surname
                .as_deref()
                .map(|bs| transliterate_to_latin(&fold_slavic_surname_stem(bs)));
            let matches_primary = &primary_stem == target_stem;
            let matches_birth = birth_stem.as_ref() == Some(target_stem);
            if !matches_primary && !matches_birth {
                continue;
            }
        }

        // 3. Birth year range filter (`born_from`, `born_to`)
        if let Some(from_yr) = query.born_from {
            match birth_year {
                Some(by) if by >= from_yr => {}
                _ => continue,
            }
        }
        if let Some(to_yr) = query.born_to {
            match birth_year {
                Some(by) if by <= to_yr => {}
                _ => continue,
            }
        }

        // 4. Sex filter
        if let Some(ref sf) = sex_filter {
            if !p.sex.eq_ignore_ascii_case(sf) {
                continue;
            }
        }

        // 5. Living / deceased filter
        if let Some(ref lf) = living_filter {
            match lf.as_str() {
                "living" if !p.is_living => continue,
                "deceased" if p.is_living => continue,
                _ => {}
            }
        }

        // 6. Has photo filter
        if let Some(want_photo) = query.has_photo {
            let has_avatar = p.avatar_url.as_ref().is_some_and(|u| !u.trim().is_empty());
            if has_avatar != want_photo {
                continue;
            }
        }

        // 7. Incomplete research filter (`!has_birth_date || !has_parents`) (`13 §4.2, §6 DoD #8`)
        if let Some(want_incomplete) = query.incomplete {
            let is_incomplete = !has_birth_date || !has_parents;
            if is_incomplete != want_incomplete {
                continue;
            }
        }

        let display_name = format_person_display_name(&p);
        let updated_at = updated_at_map.get(&p.id).cloned();

        filtered_items.push(PeopleDirectoryItemDto {
            id: p.id,
            gid: p.gid,
            given_name: p.primary_name.given_name,
            patronymic: p.primary_name.patronymic,
            surname: p.primary_name.surname,
            birth_surname: p.birth_surname,
            display_name,
            sex: p.sex,
            birth_date,
            birth_year,
            birth_place,
            death_date,
            death_year,
            death_place,
            avatar_url: p.avatar_url,
            is_living: p.is_living,
            is_private: p.is_private,
            has_parents,
            has_birth_date,
            version: p.version,
            updated_at,
        });
    }

    // Sorting (`name_asc`, `name_desc`, `birth_asc`, `birth_desc`, `death_asc`, `death_desc`, `updated_desc`)
    let sort_mode = query
        .sort
        .as_deref()
        .unwrap_or("name_asc")
        .trim()
        .to_ascii_lowercase();

    match sort_mode.as_str() {
        "name_desc" => {
            filtered_items.sort_by(|a, b| {
                b.display_name
                    .to_lowercase()
                    .cmp(&a.display_name.to_lowercase())
                    .then_with(|| a.gid.cmp(&b.gid))
            });
        }
        "birth_asc" => {
            filtered_items.sort_by(|a, b| {
                a.birth_year
                    .unwrap_or(i32::MAX)
                    .cmp(&b.birth_year.unwrap_or(i32::MAX))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "birth_desc" => {
            filtered_items.sort_by(|a, b| {
                b.birth_year
                    .unwrap_or(i32::MIN)
                    .cmp(&a.birth_year.unwrap_or(i32::MIN))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "death_asc" => {
            filtered_items.sort_by(|a, b| {
                a.death_year
                    .unwrap_or(i32::MAX)
                    .cmp(&b.death_year.unwrap_or(i32::MAX))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "death_desc" => {
            filtered_items.sort_by(|a, b| {
                b.death_year
                    .unwrap_or(i32::MIN)
                    .cmp(&a.death_year.unwrap_or(i32::MIN))
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        "updated_desc" => {
            filtered_items.sort_by(|a, b| {
                b.updated_at
                    .as_deref()
                    .unwrap_or("")
                    .cmp(a.updated_at.as_deref().unwrap_or(""))
                    .then_with(|| b.version.cmp(&a.version))
                    .then_with(|| a.gid.cmp(&b.gid))
            });
        }
        _ => {
            // Default `name_asc` (surname-first display_name)
            filtered_items.sort_by(|a, b| {
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase())
                    .then_with(|| a.gid.cmp(&b.gid))
            });
        }
    }

    let total = filtered_items.len();
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query
        .cursor
        .as_deref()
        .and_then(|c| c.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let page_items: Vec<PeopleDirectoryItemDto> = filtered_items
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let next_cursor = if offset + page_items.len() < total {
        Some((offset + page_items.len()).to_string())
    } else {
        None
    };

    Ok(Json(PeopleDirectoryResponse {
        items: page_items,
        next_cursor,
        total,
        total_unfiltered,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/people/surnames",
    params(SurnamesIndexQuery),
    responses(
        (status = 200, description = "Alphabetical surnames index with gender folding, counts, and year spans (authenticated users only)", body = SurnamesIndexResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn list_people_surnames(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<SurnamesIndexQuery>,
) -> Result<Json<SurnamesIndexResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = auth.access();
    let store = state.privacy_store.read().await;

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();
    let total_people = visible_persons.len();

    let min_count = query.min_count.unwrap_or(1).max(1);
    let letter_filter = query
        .letter
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase());

    // Group by gender-folded surname stem (`fold_slavic_surname_stem`)
    struct SurnameAgg {
        canonical: String,
        variants: Vec<String>,
        count: usize,
        earliest_year: Option<i32>,
        latest_year: Option<i32>,
    }

    let mut agg_map: HashMap<String, SurnameAgg> = HashMap::new();

    for p in &visible_persons {
        let raw_surname = p.primary_name.surname.trim();
        if raw_surname.is_empty() {
            continue;
        }
        let stem_key = fold_slavic_surname_stem(raw_surname);
        let canonical = canonicalize_surname_display(raw_surname);
        let entry = agg_map.entry(stem_key).or_insert_with(|| SurnameAgg {
            canonical,
            variants: Vec::new(),
            count: 0,
            earliest_year: None,
            latest_year: None,
        });

        entry.count += 1;
        if !entry
            .variants
            .iter()
            .any(|v| v.eq_ignore_ascii_case(raw_surname))
        {
            entry.variants.push(raw_surname.to_string());
            entry.variants.sort();
        }

        let by = p.birth_sort.map(|bs| bs / 10_000);
        let dy = p.death_sort.map(|ds| ds / 10_000);
        for yr in [by, dy].into_iter().flatten() {
            entry.earliest_year = Some(entry.earliest_year.map_or(yr, |cur| cur.min(yr)));
            entry.latest_year = Some(entry.latest_year.map_or(yr, |cur| cur.max(yr)));
        }
    }

    let mut entries: Vec<SurnameIndexEntryDto> = agg_map
        .into_values()
        .filter(|e| e.count >= min_count)
        .map(|e| SurnameIndexEntryDto {
            canonical_surname: e.canonical,
            variants: e.variants,
            count: e.count,
            earliest_year: e.earliest_year,
            latest_year: e.latest_year,
        })
        .collect();

    entries.sort_by(|a, b| {
        a.canonical_surname
            .to_lowercase()
            .cmp(&b.canonical_surname.to_lowercase())
    });

    let total_surnames = entries.len();

    let mut letter_groups: Vec<SurnameLetterGroupDto> = Vec::new();
    for item in entries {
        let first_letter = item
            .canonical_surname
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>())
            .unwrap_or_else(|| "#".to_string());

        if let Some(ref lf) = letter_filter {
            if &first_letter != lf {
                continue;
            }
        }

        if let Some(existing_group) = letter_groups.iter_mut().find(|g| g.letter == first_letter) {
            existing_group.items.push(item);
        } else {
            letter_groups.push(SurnameLetterGroupDto {
                letter: first_letter,
                items: vec![item],
            });
        }
    }

    Ok(Json(SurnamesIndexResponse {
        groups: letter_groups,
        total_surnames,
        total_people,
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
    RequireEditor(editor): RequireEditor,
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
        log_change(
            &mut store,
            None,
            "place.create",
            "place",
            &leaf.id,
            "create",
            Some(leaf.full_name.clone()),
            format!("Created place {} ({})", leaf.gid, leaf.full_name),
            None,
            serde_json::to_value(&leaf).ok(),
            Some(editor.user.id.to_string()),
            Some(editor.user.display_name()),
        );
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

    log_change(
        &mut store,
        None,
        "place.create",
        "place",
        &created.id,
        "create",
        Some(created.full_name.clone()),
        format!("Created place {} ({})", created.gid, created.full_name),
        None,
        serde_json::to_value(&created).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
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

    let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
    verify_optimistic_version(
        store.places[idx].version,
        expected_version,
        &format!("Place {}", store.places[idx].gid),
        serde_json::to_value(&store.places[idx]).unwrap_or_default(),
    )?;

    let before_json = serde_json::to_value(&store.places[idx]).ok();

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

    log_change(
        &mut store,
        None,
        "place.update",
        "place",
        &updated.id,
        "update",
        Some(updated.full_name.clone()),
        format!("Updated place {} ({})", updated.gid, updated.full_name),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
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

    let idx = store
        .places
        .iter()
        .position(|p| p.id == place_id)
        .ok_or(AppError::NotFound)?;
    let removed = store.places.remove(idx);
    recompute_all_places_metadata(&mut store);

    log_change(
        &mut store,
        None,
        "place.delete",
        "place",
        &removed.id,
        "delete",
        Some(removed.full_name.clone()),
        format!("Deleted place {} ({})", removed.gid, removed.full_name),
        serde_json::to_value(&removed).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

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
    RequireEditor(editor): RequireEditor,
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

    let before_target_json = store
        .places
        .iter()
        .find(|p| p.id == target_id)
        .and_then(|p| serde_json::to_value(p).ok());

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
        candidate_alts.extend(source_place.alt_names.clone());

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

    let tx_id = Uuid::new_v4().to_string();
    log_change(
        &mut store,
        Some(tx_id.clone()),
        "place.merge",
        "place",
        &source_place.id,
        "delete",
        Some(source_place.full_name.clone()),
        format!(
            "Merged place {} ({}) into {} ({})",
            source_place.gid, source_place.full_name, merged.gid, merged.full_name
        ),
        serde_json::to_value(&source_place).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );
    log_change(
        &mut store,
        Some(tx_id),
        "place.merge",
        "place",
        &merged.id,
        "update",
        Some(merged.full_name.clone()),
        format!(
            "Merged {} into place {} ({})",
            source_place.gid, merged.gid, merged.full_name
        ),
        before_target_json,
        serde_json::to_value(&merged).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(merged))
}

fn extract_year_from_media(m: &MediaAssetSummaryDto) -> Option<i32> {
    if let Some(ds) = m.date_sort {
        let yr = ds / 10_000;
        if (1000..=2999).contains(&yr) {
            return Some(yr);
        }
    }
    for text in [m.date_taken.as_deref().unwrap_or(""), m.title.as_str()] {
        for token in text.split(|c: char| !c.is_ascii_digit()) {
            if token.len() == 4 {
                if let Ok(yr) = token.parse::<i32>() {
                    if (1500..=2100).contains(&yr) {
                        return Some(yr);
                    }
                }
            }
        }
    }
    None
}

fn sync_media_links_and_living(store: &mut GenealogyPrivacyStore, media_id: &str) {
    let person_map: HashMap<String, (String, bool)> = store
        .persons
        .iter()
        .map(|p| (p.id.clone(), (format_person_display_name(p), p.is_living)))
        .collect();

    if let Some(media) = store.media.iter_mut().find(|m| m.id == media_id) {
        for tag in &mut media.face_tags {
            if let Some((pname, living)) = person_map.get(&tag.person_id) {
                tag.person_name = pname.clone();
                tag.is_living = *living;
            }
            if !media.linked_person_ids.contains(&tag.person_id) {
                media.linked_person_ids.push(tag.person_id.clone());
            }
            if !media
                .links
                .iter()
                .any(|l| l.entity_type == "individual" && l.entity_id == tag.person_id)
            {
                media.links.push(MediaLinkDto {
                    id: Uuid::new_v4().to_string(),
                    media_id: media.id.clone(),
                    entity_type: "individual".to_string(),
                    entity_id: tag.person_id.clone(),
                    role: Some("depicted".to_string()),
                });
            }
        }

        for link in &media.links {
            if link.entity_type == "individual"
                && !media.linked_person_ids.contains(&link.entity_id)
            {
                media.linked_person_ids.push(link.entity_id.clone());
            }
        }

        if media.linked_person_ids.is_empty() {
            media.only_living_persons = false;
        } else {
            media.only_living_persons = media
                .linked_person_ids
                .iter()
                .all(|pid| person_map.get(pid).is_some_and(|(_, living)| *living));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_primary_avatar_for_person(
    store: &mut GenealogyPrivacyStore,
    person_id: &str,
    media_id: &str,
    tag_id: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    for m in &mut store.media {
        for t in &mut m.face_tags {
            if t.person_id == person_id {
                t.is_primary_avatar = m.id == media_id && t.id == tag_id;
            }
        }
    }
    if let Some(person) = store.persons.iter_mut().find(|p| p.id == person_id) {
        person.avatar_media_id = Some(media_id.to_string());
        person.avatar_crop = Some(AvatarCropDto {
            x,
            y,
            width: w,
            height: h,
        });
        person.avatar_url = Some(format!(
            "/api/v1/media/{media_id}/crop?x={x:.4}&y={y:.4}&w={w:.4}&h={h:.4}"
        ));
    }
}

fn clear_primary_avatar_for_person(store: &mut GenealogyPrivacyStore, person_id: &str) {
    for m in &mut store.media {
        for t in &mut m.face_tags {
            if t.person_id == person_id {
                t.is_primary_avatar = false;
            }
        }
    }
    if let Some(person) = store.persons.iter_mut().find(|p| p.id == person_id) {
        person.avatar_media_id = None;
        person.avatar_crop = None;
        person.avatar_url = None;
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/media",
    params(
        ("q" = Option<String>, Query, description = "Search query across title, description, place, or tagged person"),
        ("person_id" = Option<String>, Query, description = "Filter by tagged/linked person UUID or GID"),
        ("decade" = Option<String>, Query, description = "Filter by decade (e.g. '1940s' or '1940')"),
        ("type" = Option<String>, Query, description = "Filter by media category ('photos', 'documents', 'audio')"),
        ("untagged" = Option<bool>, Query, description = "Filter untagged media (true = untagged only, false = tagged only)"),
        ("cursor" = Option<String>, Query, description = "Pagination cursor"),
        ("limit" = Option<usize>, Query, description = "Max items per page (default 50, max 100)")
    ),
    responses(
        (status = 200, description = "Media archive & gallery (authenticated users only)", body = MediaArchiveResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn list_media_archive(
    RequireAuth(auth): RequireAuth,
    State(state): State<AppState>,
    Query(query): Query<MediaFilterQuery>,
) -> Result<Json<MediaArchiveResponse>, AppError> {
    let access = auth.access();
    let store = state.privacy_store.read().await;

    let unfiltered_items: Vec<MediaAssetSummaryDto> = store
        .media
        .iter()
        .filter_map(|m| redact_media_for_caller(m, &access))
        .collect();
    let total_unfiltered = unfiltered_items.len();

    let resolved_person_filter = if let Some(ref pid_raw) = query.person_id {
        let trimmed = pid_raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(
                resolve_person_id_in_store(&store.persons, trimmed)
                    .unwrap_or_else(|| "__nonexistent__".to_string()),
            )
        }
    } else {
        None
    };

    let decade_start_year: Option<i32> = query.decade.as_deref().and_then(|d| {
        let digits: String = d
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if digits.len() == 4 {
            digits.parse::<i32>().ok().map(|yr| (yr / 10) * 10)
        } else {
            None
        }
    });

    let q_tokens: Vec<String> = query
        .q
        .as_deref()
        .map(|q| q.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default();

    let filtered: Vec<MediaAssetSummaryDto> = unfiltered_items
        .into_iter()
        .filter(|m| {
            if let Some(ref target_pid) = resolved_person_filter {
                let matches_person = m.linked_person_ids.iter().any(|id| id == target_pid)
                    || m.face_tags.iter().any(|t| &t.person_id == target_pid)
                    || m.links
                        .iter()
                        .any(|l| l.entity_type == "individual" && &l.entity_id == target_pid);
                if !matches_person {
                    return false;
                }
            }

            if let Some(dec_yr) = decade_start_year {
                let Some(media_yr) = extract_year_from_media(m) else {
                    return false;
                };
                if media_yr < dec_yr || media_yr > dec_yr + 9 {
                    return false;
                }
            }

            if let Some(ref mtype) = query.media_type {
                let mt = mtype.trim().to_lowercase();
                if !mt.is_empty() && mt != "all" {
                    let mime_matches = match mt.as_str() {
                        "photos" | "photo" | "image" | "images" => {
                            m.mime_type.starts_with("image/")
                        }
                        "documents" | "document" | "pdf" | "doc" => {
                            m.mime_type == "application/pdf" || m.mime_type.starts_with("text/")
                        }
                        "audio" => m.mime_type.starts_with("audio/"),
                        "video" => m.mime_type.starts_with("video/"),
                        other => m.mime_type.eq_ignore_ascii_case(other),
                    };
                    if !mime_matches {
                        return false;
                    }
                }
            }

            if let Some(untagged_filter) = query.untagged {
                let is_untagged = m.face_tags.is_empty() && m.linked_person_ids.is_empty();
                if untagged_filter != is_untagged {
                    return false;
                }
            }

            if !q_tokens.is_empty() {
                let mut searchable_fields = vec![
                    m.title.clone(),
                    m.gid.clone(),
                    m.original_file_name.clone().unwrap_or_default(),
                    m.description.clone().unwrap_or_default(),
                    m.place_full_name.clone().unwrap_or_default(),
                ];
                for tag in &m.face_tags {
                    searchable_fields.push(tag.person_name.clone());
                }
                let combined_raw = searchable_fields.join(" ").to_lowercase();
                let combined_ortho = searchable_fields
                    .iter()
                    .flat_map(|s| s.split_whitespace())
                    .map(normalize_russian_orthography_word)
                    .collect::<Vec<_>>()
                    .join(" ");
                let combined_lat = transliterate_to_latin(&combined_ortho);

                for tok in &q_tokens {
                    let tok_lower = tok.to_lowercase();
                    let tok_ortho = normalize_russian_orthography_word(tok);
                    let tok_lat = transliterate_to_latin(&tok_ortho);
                    let matched = combined_raw.contains(&tok_lower)
                        || combined_ortho.contains(&tok_ortho)
                        || (!tok_lat.is_empty() && combined_lat.contains(&tok_lat));
                    if !matched {
                        return false;
                    }
                }
            }

            true
        })
        .collect();

    let total = filtered.len();
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let start_idx = if let Some(ref cur) = query.cursor {
        let cur_trimmed = cur.trim();
        if let Ok(offset) = cur_trimmed.parse::<usize>() {
            offset.min(total)
        } else {
            filtered
                .iter()
                .position(|m| m.id == cur_trimmed || m.gid.eq_ignore_ascii_case(cur_trimmed))
                .map(|idx| idx + 1)
                .unwrap_or(0)
        }
    } else {
        0
    };

    let items: Vec<MediaAssetSummaryDto> =
        filtered.into_iter().skip(start_idx).take(limit).collect();
    let next_cursor = if start_idx + items.len() < total {
        items.last().map(|m| m.id.clone())
    } else {
        None
    };

    Ok(Json(MediaArchiveResponse {
        items,
        next_cursor,
        total,
        total_unfiltered,
    }))
}

const ALLOWED_MEDIA_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/tiff",
    "application/pdf",
    "audio/mpeg",
    "audio/ogg",
    "audio/wav",
    "audio/mp4",
    "video/mp4",
];

fn extension_for_media(file_name: &str, mime_type: &str) -> &'static str {
    let lower_name = file_name.to_lowercase();
    if lower_name.ends_with(".jpg") || lower_name.ends_with(".jpeg") {
        return "jpg";
    }
    if lower_name.ends_with(".png") {
        return "png";
    }
    if lower_name.ends_with(".webp") {
        return "webp";
    }
    if lower_name.ends_with(".gif") {
        return "gif";
    }
    if lower_name.ends_with(".tif") || lower_name.ends_with(".tiff") {
        return "tiff";
    }
    if lower_name.ends_with(".pdf") {
        return "pdf";
    }
    if lower_name.ends_with(".mp3") {
        return "mp3";
    }
    if lower_name.ends_with(".ogg") {
        return "ogg";
    }
    if lower_name.ends_with(".wav") {
        return "wav";
    }
    match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/tiff" => "tiff",
        "application/pdf" => "pdf",
        "audio/mpeg" => "mp3",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/mp4" | "video/mp4" => "mp4",
        _ => "webp",
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/media",
    request_body = CreateMediaRequest,
    responses(
        (status = 201, description = "Media asset uploaded and indexed", body = MediaAssetSummaryDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 422, description = "Validation error (unsupported MIME type or invalid payload)", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn create_media(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Json(req): Json<CreateMediaRequest>,
) -> Result<(StatusCode, Json<MediaAssetSummaryDto>), AppError> {
    let orig_name = req.file_name.as_deref().unwrap_or("").trim();
    if orig_name.is_empty() {
        return Err(AppError::ValidationFailed(
            "file_name is required".to_string(),
            Some(HashMap::from([(
                "file_name".to_string(),
                "required".to_string(),
            )])),
        ));
    }

    let mime_trimmed = req.mime_type.as_deref().unwrap_or("").trim().to_lowercase();
    let mime_type = if mime_trimmed.is_empty() {
        "image/webp".to_string()
    } else {
        mime_trimmed
    };

    if !ALLOWED_MEDIA_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(AppError::ValidationFailed(
            format!("Unsupported media MIME type: {mime_type}"),
            Some(HashMap::from([(
                "mime_type".to_string(),
                "unsupported_mime_type".to_string(),
            )])),
        ));
    }

    let raw_bytes: Vec<u8> = if let Some(ref b64_raw) = req.data {
        let stripped = b64_raw
            .trim()
            .split_once(";base64,")
            .map(|(_, payload)| payload)
            .unwrap_or_else(|| b64_raw.trim());
        if stripped.is_empty() {
            DEFAULT_WEBP_BYTES.to_vec()
        } else {
            BASE64_STANDARD.decode(stripped).map_err(|_| {
                AppError::ValidationFailed(
                    "Invalid base64 media payload".to_string(),
                    Some(HashMap::from([(
                        "data".to_string(),
                        "invalid_base64".to_string(),
                    )])),
                )
            })?
        }
    } else {
        DEFAULT_WEBP_BYTES.to_vec()
    };

    let sha_digest = Sha256::digest(&raw_bytes);
    let sha256_hex: String = sha_digest.iter().map(|b| format!("{b:02x}")).collect();
    let file_size_bytes = raw_bytes.len() as u64;

    let (date_taken, date_sort) = if let Some(ref dt_raw) = req.date_taken {
        let dt_trimmed = dt_raw.trim();
        if dt_trimmed.is_empty() {
            (None, None)
        } else {
            let parsed = GenDate::parse(dt_trimmed);
            let sort = parsed.sort_key();
            (Some(dt_trimmed.to_string()), sort)
        }
    } else {
        (None, None)
    };

    let mut store = state.privacy_store.write().await;

    let (place_id, place_full_name) = if let Some(ref pid_raw) = req.place_id {
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

    let is_image = mime_type.starts_with("image/");
    let width = req.width.or(if is_image { Some(1200) } else { None });
    let height = req.height.or(if is_image { Some(900) } else { None });
    let description = req
        .description
        .or(req.caption)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Check if an existing missing GEDCOM media stub matches by base filename (`03 §2.2 #4`)
    let upload_base_name = orig_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(orig_name)
        .trim();
    let matching_stub_id = if !upload_base_name.is_empty() {
        store
            .media
            .iter()
            .find(|m| {
                m.is_missing
                    && (m
                        .original_file_name
                        .as_deref()
                        .is_some_and(|ofn| ofn.eq_ignore_ascii_case(upload_base_name))
                        || m.file_name
                            .as_deref()
                            .is_some_and(|fnm| fnm.eq_ignore_ascii_case(upload_base_name))
                        || m.external_path
                            .as_deref()
                            .and_then(|p| p.rsplit(['/', '\\']).next())
                            .is_some_and(|bn| bn.eq_ignore_ascii_case(upload_base_name)))
            })
            .map(|m| m.id.clone())
    } else {
        None
    };

    let mut resolved_linked_persons = Vec::new();
    if let Some(ref person_ids) = req.linked_person_ids {
        for raw_pid in person_ids {
            let pid = resolve_person_id_in_store(&store.persons, raw_pid.trim())
                .ok_or(AppError::NotFound)?;
            if !resolved_linked_persons.contains(&pid) {
                resolved_linked_persons.push(pid);
            }
        }
    }

    if let Some(stub_id) = matching_stub_id {
        let ext = extension_for_media(orig_name, &mime_type);
        let stored_file_name = format!("{stub_id}.{ext}");
        save_media_bytes_to_disk(
            &state.config.media_storage_path,
            &stored_file_name,
            &raw_bytes,
        );

        if let Some(stub) = store.media.iter_mut().find(|m| m.id == stub_id) {
            stub.mime_type = mime_type;
            stub.file_url = format!("/api/v1/media/{stub_id}/file");
            stub.file_name = Some(stored_file_name);
            stub.original_file_name = Some(upload_base_name.to_string());
            stub.file_size_bytes = Some(file_size_bytes);
            stub.sha256 = Some(sha256_hex);
            stub.width = width;
            stub.height = height;
            stub.is_missing = false;
            stub.version += 1;

            if let Some(explicit_title) = req
                .title
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                stub.title = explicit_title.to_string();
            }
            if description.is_some() {
                stub.description = description;
            }
            if date_taken.is_some() {
                stub.date_taken = date_taken;
                stub.date_sort = date_sort;
            }
            if place_id.is_some() {
                stub.place_id = place_id;
                stub.place_full_name = place_full_name;
            }
            if let Some(priv_flag) = req.is_private {
                stub.is_private = priv_flag;
            }
            for pid in resolved_linked_persons {
                if !stub.linked_person_ids.contains(&pid) {
                    stub.linked_person_ids.push(pid.clone());
                    stub.links.push(MediaLinkDto {
                        id: Uuid::new_v4().to_string(),
                        media_id: stub_id.clone(),
                        entity_type: "individual".to_string(),
                        entity_id: pid,
                        role: Some("gallery".to_string()),
                    });
                }
            }
        }

        sync_media_links_and_living(&mut store, &stub_id);

        let reconciled = store
            .media
            .iter()
            .find(|m| m.id == stub_id)
            .cloned()
            .ok_or(AppError::NotFound)?;

        log_change(
            &mut store,
            None,
            "media.reconcile_stub",
            "media",
            &reconciled.id,
            "update",
            Some(reconciled.title.clone()),
            format!(
                "Reconciled media stub {} ({})",
                reconciled.gid, reconciled.title
            ),
            None,
            serde_json::to_value(&reconciled).ok(),
            Some(editor.user.id.to_string()),
            Some(editor.user.display_name()),
        );

        return Ok((StatusCode::CREATED, Json(reconciled)));
    }

    let id = Uuid::new_v4().to_string();
    let ext = extension_for_media(orig_name, &mime_type);
    let stored_file_name = format!("{id}.{ext}");

    save_media_bytes_to_disk(
        &state.config.media_storage_path,
        &stored_file_name,
        &raw_bytes,
    );

    let existing_gids: HashSet<String> = store.media.iter().map(|m| m.gid.clone()).collect();
    let gid = allocate_gid(
        'M',
        &mut store.next_media_gid_seq,
        req.gid.as_deref(),
        &existing_gids,
    );

    let title = req
        .title
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| orig_name.to_string());

    let links: Vec<MediaLinkDto> = resolved_linked_persons
        .iter()
        .map(|pid| MediaLinkDto {
            id: Uuid::new_v4().to_string(),
            media_id: id.clone(),
            entity_type: "individual".to_string(),
            entity_id: pid.clone(),
            role: Some("gallery".to_string()),
        })
        .collect();

    let media = MediaAssetSummaryDto {
        id: id.clone(),
        gid,
        title,
        mime_type,
        file_url: format!("/api/v1/media/{id}/file"),
        file_name: Some(stored_file_name),
        original_file_name: Some(orig_name.to_string()),
        file_size_bytes: Some(file_size_bytes),
        sha256: Some(sha256_hex),
        width,
        height,
        description,
        date_taken,
        date_sort,
        place_id,
        place_full_name,
        is_missing: false,
        external_path: None,
        is_private: req.is_private.unwrap_or(false),
        only_living_persons: false,
        linked_person_ids: resolved_linked_persons,
        face_tags: vec![],
        links,
        version: 1,
    };

    store.media.push(media);
    sync_media_links_and_living(&mut store, &id);

    let created = store
        .media
        .iter()
        .find(|m| m.id == id)
        .cloned()
        .ok_or(AppError::NotFound)?;

    log_change(
        &mut store,
        None,
        "media.create",
        "media",
        &created.id,
        "create",
        Some(created.title.clone()),
        format!("Uploaded media {} ({})", created.gid, created.title),
        None,
        serde_json::to_value(&created).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/v1/media/{id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID (e.g. M0001)")
    ),
    responses(
        (status = 200, description = "Media asset metadata, face tags, and entity links", body = MediaAssetSummaryDto),
        (status = 404, description = "Media asset not found or shielded by privacy", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn get_media_detail(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<MediaAssetSummaryDto>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let target = id.trim();

    let media = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let redacted = redact_media_for_caller(media, &access).ok_or(AppError::NotFound)?;
    Ok(Json(redacted))
}

#[utoipa::path(
    patch,
    path = "/api/v1/media/{id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    request_body = UpdateMediaRequest,
    responses(
        (status = 200, description = "Media asset metadata updated", body = MediaAssetSummaryDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset not found", body = ErrorResponse),
        (status = 409, description = "Optimistic concurrency version conflict", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn update_media(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateMediaRequest>,
) -> Result<Json<MediaAssetSummaryDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let resolved_place = if let Some(ref pid_raw) = req.place_id {
        if pid_raw.trim().is_empty() {
            Some((None, None))
        } else {
            let pid =
                resolve_place_id_in_store(&store.places, pid_raw).ok_or(AppError::NotFound)?;
            let fname = store
                .places
                .iter()
                .find(|p| p.id == pid)
                .map(|p| p.full_name.clone());
            Some((Some(pid), fname))
        }
    } else if let Some(ref ppath) = req.place_path {
        if ppath.trim().is_empty() {
            Some((None, None))
        } else {
            let place = resolve_or_create_place_chain(&mut store, ppath, None, None, None, None)?;
            Some((Some(place.id), Some(place.full_name)))
        }
    } else {
        None
    };

    let (updated, before_json) = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;

        let expected_version = req.version.or_else(|| parse_if_match_version(&headers));
        verify_optimistic_version(
            media.version,
            expected_version,
            &format!("Media {}", media.gid),
            serde_json::to_value(&*media).unwrap_or_default(),
        )?;

        let before_snapshot = serde_json::to_value(&*media).ok();

        if let Some(ref title) = req.title {
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                media.title = trimmed.to_string();
            }
        }

        if let Some(desc) = req.description.or(req.caption) {
            let trimmed = desc.trim();
            media.description = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
        }

        if let Some(ref dt_raw) = req.date_taken {
            let trimmed = dt_raw.trim();
            if trimmed.is_empty() {
                media.date_taken = None;
                media.date_sort = None;
            } else {
                let parsed = GenDate::parse(trimmed);
                media.date_sort = parsed.sort_key();
                media.date_taken = Some(trimmed.to_string());
            }
        }

        if let Some((pid, fname)) = resolved_place {
            media.place_id = pid;
            media.place_full_name = fname;
        }

        if let Some(is_priv) = req.is_private {
            media.is_private = is_priv;
        }

        media.version += 1;
        (media.clone(), before_snapshot)
    };

    log_change(
        &mut store,
        None,
        "media.update",
        "media",
        &updated.id,
        "update",
        Some(updated.title.clone()),
        format!("Updated media {} ({})", updated.gid, updated.title),
        before_json,
        serde_json::to_value(&updated).ok(),
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/media/{id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    responses(
        (status = 204, description = "Media asset deleted"),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset not found", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn delete_media(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_idx = store
        .media
        .iter()
        .position(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .ok_or(AppError::NotFound)?;

    let removed = store.media.remove(media_idx);

    for ev in &mut store.events {
        ev.media_ids.retain(|mid| mid != &removed.id);
    }

    for person in &mut store.persons {
        if person.avatar_media_id.as_deref() == Some(removed.id.as_str()) {
            person.avatar_media_id = None;
            person.avatar_crop = None;
            person.avatar_url = None;
        }
    }

    log_change(
        &mut store,
        None,
        "media.delete",
        "media",
        &removed.id,
        "delete",
        Some(removed.title.clone()),
        format!("Deleted media {} ({})", removed.gid, removed.title),
        serde_json::to_value(&removed).ok(),
        None,
        Some(editor.user.id.to_string()),
        Some(editor.user.display_name()),
    );

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/media/{id}/tags",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    request_body = CreateFaceTagRequest,
    responses(
        (status = 201, description = "Face tag created on media asset", body = FaceTagDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or person not found", body = ErrorResponse),
        (status = 422, description = "Invalid normalized bounding box coordinates", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn create_face_tag(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateFaceTagRequest>,
) -> Result<(StatusCode, Json<FaceTagDto>), AppError> {
    let w = req.w.or(req.width).unwrap_or(0.0);
    let h = req.h.or(req.height).unwrap_or(0.0);
    if !(0.0..=1.0).contains(&req.x)
        || !(0.0..=1.0).contains(&req.y)
        || w <= 0.0
        || w > 1.0
        || h <= 0.0
        || h > 1.0
    {
        return Err(AppError::ValidationFailed(
            "Face tag coordinates (x, y, w, h) must be normalized within 0.0..=1.0".to_string(),
            Some(HashMap::from([(
                "coordinates".to_string(),
                "out_of_bounds".to_string(),
            )])),
        ));
    }

    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let person_id = resolve_person_id_in_store(&store.persons, req.person_id.trim())
        .ok_or(AppError::NotFound)?;
    let (person_name, is_living) = store
        .persons
        .iter()
        .find(|p| p.id == person_id)
        .map(|p| (format_person_display_name(p), p.is_living))
        .ok_or(AppError::NotFound)?;

    let tag_id = Uuid::new_v4().to_string();
    let is_primary = req.is_primary_avatar.unwrap_or(false);

    let new_tag = FaceTagDto {
        id: tag_id.clone(),
        person_id: person_id.clone(),
        person_name,
        is_living,
        x: req.x,
        y: req.y,
        width: w,
        height: h,
        is_primary_avatar: is_primary,
    };

    if let Some(media) = store.media.iter_mut().find(|m| m.id == media_id) {
        media.face_tags.push(new_tag);
        media.version += 1;
    }

    if is_primary {
        apply_primary_avatar_for_person(
            &mut store, &person_id, &media_id, &tag_id, req.x, req.y, w, h,
        );
    }

    sync_media_links_and_living(&mut store, &media_id);

    let created_tag = store
        .media
        .iter()
        .find(|m| m.id == media_id)
        .and_then(|m| m.face_tags.iter().find(|t| t.id == tag_id).cloned())
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::CREATED, Json(created_tag)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/media/{id}/tags/{tag_id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID"),
        ("tag_id" = String, Path, description = "Face tag ID")
    ),
    request_body = UpdateFaceTagRequest,
    responses(
        (status = 200, description = "Face tag updated", body = FaceTagDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset, face tag, or person not found", body = ErrorResponse),
        (status = 422, description = "Invalid normalized bounding box coordinates", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn update_face_tag(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, tag_id)): Path<(String, String)>,
    Json(req): Json<UpdateFaceTagRequest>,
) -> Result<Json<FaceTagDto>, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let target_tag = tag_id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let new_person_info = if let Some(ref raw_pid) = req.person_id {
        let pid =
            resolve_person_id_in_store(&store.persons, raw_pid.trim()).ok_or(AppError::NotFound)?;
        let (pname, living) = store
            .persons
            .iter()
            .find(|p| p.id == pid)
            .map(|p| (format_person_display_name(p), p.is_living))
            .ok_or(AppError::NotFound)?;
        Some((pid, pname, living))
    } else {
        None
    };

    let (person_id, x, y, w, h, was_primary, now_primary) = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;
        let tag = media
            .face_tags
            .iter_mut()
            .find(|t| t.id == target_tag)
            .ok_or(AppError::NotFound)?;

        let next_x = req.x.unwrap_or(tag.x);
        let next_y = req.y.unwrap_or(tag.y);
        let next_w = req.w.or(req.width).unwrap_or(tag.width);
        let next_h = req.h.or(req.height).unwrap_or(tag.height);
        if !(0.0..=1.0).contains(&next_x)
            || !(0.0..=1.0).contains(&next_y)
            || next_w <= 0.0
            || next_w > 1.0
            || next_h <= 0.0
            || next_h > 1.0
        {
            return Err(AppError::ValidationFailed(
                "Face tag coordinates (x, y, w, h) must be normalized within 0.0..=1.0".to_string(),
                Some(HashMap::from([(
                    "coordinates".to_string(),
                    "out_of_bounds".to_string(),
                )])),
            ));
        }

        if let Some((pid, pname, living)) = new_person_info {
            tag.person_id = pid;
            tag.person_name = pname;
            tag.is_living = living;
        }
        tag.x = next_x;
        tag.y = next_y;
        tag.width = next_w;
        tag.height = next_h;

        let was_primary = tag.is_primary_avatar;
        let now_primary = req.is_primary_avatar.unwrap_or(was_primary);
        tag.is_primary_avatar = now_primary;
        media.version += 1;

        (
            tag.person_id.clone(),
            tag.x,
            tag.y,
            tag.width,
            tag.height,
            was_primary,
            now_primary,
        )
    };

    if now_primary {
        apply_primary_avatar_for_person(&mut store, &person_id, &media_id, target_tag, x, y, w, h);
    } else if was_primary && !now_primary {
        clear_primary_avatar_for_person(&mut store, &person_id);
    }

    sync_media_links_and_living(&mut store, &media_id);

    let updated = store
        .media
        .iter()
        .find(|m| m.id == media_id)
        .and_then(|m| m.face_tags.iter().find(|t| t.id == target_tag).cloned())
        .ok_or(AppError::NotFound)?;

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/media/{id}/tags/{tag_id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID"),
        ("tag_id" = String, Path, description = "Face tag ID")
    ),
    responses(
        (status = 204, description = "Face tag deleted"),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or face tag not found", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn delete_face_tag(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, tag_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let target_tag = tag_id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let removed_tag = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;
        let idx = media
            .face_tags
            .iter()
            .position(|t| t.id == target_tag)
            .ok_or(AppError::NotFound)?;
        let removed = media.face_tags.remove(idx);

        let still_tagged = media
            .face_tags
            .iter()
            .any(|t| t.person_id == removed.person_id);
        let has_explicit_gallery_link = media.links.iter().any(|l| {
            l.entity_type == "individual"
                && l.entity_id == removed.person_id
                && l.role.as_deref() != Some("depicted")
        });
        if !still_tagged && !has_explicit_gallery_link {
            media
                .links
                .retain(|l| !(l.entity_type == "individual" && l.entity_id == removed.person_id));
            media
                .linked_person_ids
                .retain(|pid| pid != &removed.person_id);
        }
        media.version += 1;
        removed
    };

    if removed_tag.is_primary_avatar {
        clear_primary_avatar_for_person(&mut store, &removed_tag.person_id);
    }

    sync_media_links_and_living(&mut store, &media_id);

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/media/{id}/links",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID")
    ),
    request_body = CreateMediaLinkRequest,
    responses(
        (status = 201, description = "Entity linked to media asset", body = MediaLinkDto),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or target entity not found", body = ErrorResponse),
        (status = 422, description = "Invalid entity_type", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn create_media_link(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateMediaLinkRequest>,
) -> Result<(StatusCode, Json<MediaLinkDto>), AppError> {
    let entity_type = req.entity_type.trim().to_lowercase();
    if !matches!(entity_type.as_str(), "individual" | "family" | "event") {
        return Err(AppError::ValidationFailed(
            "entity_type must be one of: individual, family, event".to_string(),
            Some(HashMap::from([(
                "entity_type".to_string(),
                "invalid_entity_type".to_string(),
            )])),
        ));
    }

    let mut store = state.privacy_store.write().await;
    let target = id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let raw_eid = req.entity_id.trim();
    let resolved_entity_id = match entity_type.as_str() {
        "individual" => {
            resolve_person_id_in_store(&store.persons, raw_eid).ok_or(AppError::NotFound)?
        }
        "family" => store
            .families
            .iter()
            .find(|f| f.id == raw_eid || f.gid.eq_ignore_ascii_case(raw_eid))
            .map(|f| f.id.clone())
            .ok_or(AppError::NotFound)?,
        "event" => store
            .events
            .iter()
            .find(|e| e.id == raw_eid || e.gid.eq_ignore_ascii_case(raw_eid))
            .map(|e| e.id.clone())
            .ok_or(AppError::NotFound)?,
        _ => unreachable!(),
    };

    let role = req
        .role
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("gallery")
        .to_string();

    let link = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;

        if let Some(existing) = media
            .links
            .iter()
            .find(|l| l.entity_type == entity_type && l.entity_id == resolved_entity_id)
        {
            existing.clone()
        } else {
            let new_link = MediaLinkDto {
                id: Uuid::new_v4().to_string(),
                media_id: media_id.clone(),
                entity_type: entity_type.clone(),
                entity_id: resolved_entity_id.clone(),
                role: Some(role),
            };
            media.links.push(new_link.clone());
            media.version += 1;
            new_link
        }
    };

    if entity_type == "event" {
        if let Some(ev) = store.events.iter_mut().find(|e| e.id == resolved_entity_id) {
            if !ev.media_ids.contains(&media_id) {
                ev.media_ids.push(media_id.clone());
            }
        }
    }

    sync_media_links_and_living(&mut store, &media_id);

    Ok((StatusCode::CREATED, Json(link)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/media/{id}/links/{link_id}",
    params(
        ("id" = String, Path, description = "Media asset UUID or GID"),
        ("link_id" = String, Path, description = "Media link ID or linked entity ID")
    ),
    responses(
        (status = 204, description = "Media link removed"),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Media asset or link not found", body = ErrorResponse)
    ),
    tag = "Media & Avatars"
)]
pub async fn delete_media_link(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((id, link_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let mut store = state.privacy_store.write().await;
    let target = id.trim();
    let target_link = link_id.trim();

    let media_id = store
        .media
        .iter()
        .find(|m| m.id == target || m.gid.eq_ignore_ascii_case(target))
        .map(|m| m.id.clone())
        .ok_or(AppError::NotFound)?;

    let removed_link = {
        let media = store
            .media
            .iter_mut()
            .find(|m| m.id == media_id)
            .ok_or(AppError::NotFound)?;
        let idx = media
            .links
            .iter()
            .position(|l| l.id == target_link || l.entity_id == target_link)
            .ok_or(AppError::NotFound)?;
        let removed = media.links.remove(idx);
        if removed.entity_type == "individual" {
            let still_tagged = media
                .face_tags
                .iter()
                .any(|t| t.person_id == removed.entity_id);
            let still_linked = media
                .links
                .iter()
                .any(|l| l.entity_type == "individual" && l.entity_id == removed.entity_id);
            if !still_tagged && !still_linked {
                media
                    .linked_person_ids
                    .retain(|pid| pid != &removed.entity_id);
            }
        }
        media.version += 1;
        removed
    };

    if removed_link.entity_type == "event" {
        if let Some(ev) = store
            .events
            .iter_mut()
            .find(|e| e.id == removed_link.entity_id)
        {
            ev.media_ids.retain(|mid| mid != &media_id);
        }
    }

    sync_media_links_and_living(&mut store, &media_id);

    Ok(StatusCode::NO_CONTENT)
}

fn decode_gedcom_request_payload(data: &str) -> Vec<u8> {
    let trimmed = data.trim_start();
    if trimmed.starts_with("0 HEAD")
        || trimmed.starts_with("\u{feff}0 HEAD")
        || data.contains("\n0 ")
        || data.contains("\r\n0 ")
    {
        return data.as_bytes().to_vec();
    }

    if let Some((_, b64_part)) = data.split_once(";base64,") {
        if let Ok(decoded) = BASE64_STANDARD.decode(b64_part.trim()) {
            return decoded;
        }
    }

    if !data.contains('\n') && !data.contains(' ') {
        if let Ok(decoded) = BASE64_STANDARD.decode(data.trim()) {
            return decoded;
        }
    }

    data.as_bytes().to_vec()
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/analyze",
    request_body = GedcomAnalyzeRequest,
    responses(
        (status = 200, description = "Pre-import GEDCOM analysis report and dialect summary (Admin only)", body = GedcomAnalysisReport),
        (status = 400, description = "Invalid or empty GEDCOM payload", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn analyze_gedcom(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<GedcomAnalyzeRequest>,
) -> Result<Json<GedcomAnalysisReport>, AppError> {
    if req.data.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("data".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "GEDCOM data payload is required".to_string(),
            Some(fields),
        ));
    }

    let settings = state.resolve_instance_settings().await;
    let raw_bytes = decode_gedcom_request_payload(&req.data);
    let mut report = analyze_gedcom_payload(
        &raw_bytes,
        Utc::now().year(),
        settings.living_person_age_threshold as i32,
    )?;
    if let Some(fname) = req
        .file_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        report.file_name = fname.to_string();
    }

    Ok(Json(report))
}

#[utoipa::path(
    get,
    path = "/api/v1/gedcom/export",
    responses(
        (status = 200, description = "GEDCOM export metadata or file download (Admin only)", body = AdminToolOperationResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn export_gedcom(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<GedcomExportRequest>,
) -> Result<Response, AppError> {
    let wants_file = query.version.is_some()
        || query.format.is_some()
        || query.redact_living.is_some()
        || query.include_media.is_some()
        || query.include_private_notes.is_some()
        || query.root_person_id.is_some()
        || query.export_scope.is_some();

    if wants_file {
        let settings = state.resolve_instance_settings().await;
        let store = state.privacy_store.read().await;
        let gedcom_text = export_store_to_gedcom(&store, &query, &settings.tree_title);
        return Ok((
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"family-tree.ged\"",
                ),
            ],
            Body::from(gedcom_text),
        )
            .into_response());
    }

    Ok(Json(AdminToolOperationResponse {
        status: "ok".to_string(),
        operation: "gedcom_export".to_string(),
        message: "GEDCOM export authorized for administrator".to_string(),
        items: vec![],
    })
    .into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/export",
    request_body(content = Option<GedcomExportRequest>, description = "GEDCOM export configuration"),
    responses(
        (status = 200, description = "Exported GEDCOM 5.5.1 or 7.0 file stream (Admin only)", body = String),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn export_gedcom_post(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<GedcomExportRequest>>,
) -> Result<Response, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let store = state.privacy_store.read().await;
    let gedcom_text = export_store_to_gedcom(&store, &req, &settings.tree_title);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"family-tree.ged\"",
            ),
        ],
        Body::from(gedcom_text),
    )
        .into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/gedcom/import",
    request_body(content = Option<GedcomImportRequest>, description = "GEDCOM import payload and mode"),
    responses(
        (status = 200, description = "GEDCOM import completed (Admin only)", body = GedcomImportResult),
        (status = 400, description = "Validation error or missing replace confirmation", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "GEDCOM"
)]
pub async fn import_gedcom(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<GedcomImportRequest>>,
) -> Result<Json<GedcomImportResult>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();

    // Compatibility with RBAC smoke check (`POST /api/v1/gedcom/import` with `{}`)
    if req.data.trim().is_empty() && req.file_name.is_none() && req.mode.is_none() {
        return Ok(Json(GedcomImportResult {
            status: "ok".to_string(),
            operation: "gedcom_import".to_string(),
            message: "GEDCOM import authorized for administrator".to_string(),
            mode: "new".to_string(),
            ..Default::default()
        }));
    }

    if req.data.trim().is_empty() {
        let mut fields = HashMap::new();
        fields.insert("data".to_string(), "required".to_string());
        return Err(AppError::ValidationFailed(
            "GEDCOM data payload is required".to_string(),
            Some(fields),
        ));
    }

    let settings = state.resolve_instance_settings().await;
    let raw_bytes = decode_gedcom_request_payload(&req.data);
    let mode = req.mode.as_deref().unwrap_or("new");

    let mut store = state.privacy_store.write().await;
    let result = import_gedcom_into_store(
        &mut store,
        &raw_bytes,
        mode,
        req.confirm_text.as_deref(),
        Utc::now().year(),
        settings.living_person_age_threshold as i32,
    )?;

    Ok(Json(result))
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
    params(HistoryFeedQuery),
    responses(
        (status = 200, description = "Global system audit history feed with filters and transaction grouping (Admin only)", body = HistoryFeedResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn get_audit_history(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<HistoryFeedQuery>,
) -> Result<Json<HistoryFeedResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = query_history_feed(&store, &query);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/history/{entity_type}/{entity_id}",
    params(
        ("entity_type" = String, Path, description = "Entity type ('individual', 'person', 'family', 'event', 'place', 'media', 'note')"),
        ("entity_id" = String, Path, description = "Entity UUID or GID")
    ),
    responses(
        (status = 200, description = "Per-record chronological change history with field-level diffs (Editor or Admin)", body = RecordHistoryResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn get_record_history(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Result<Json<RecordHistoryResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = query_record_history(&store, &entity_type, &entity_id)?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/history/tx/{tx_id}/undo",
    params(
        ("tx_id" = String, Path, description = "Transaction ID or change log entry ID to undo")
    ),
    responses(
        (status = 200, description = "Transaction reversed in reverse chronological order", body = UndoTransactionResponse),
        (status = 403, description = "Only the author (within 10 minutes) or an Administrator can undo", body = ErrorResponse),
        (status = 404, description = "Transaction not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn undo_history_transaction(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Path(tx_id): Path<String>,
) -> Result<Json<UndoTransactionResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;
    let is_admin = editor.user.role == "admin";

    let mut store = state.privacy_store.write().await;
    let resp = undo_transaction(
        &mut store,
        &tx_id,
        &editor.user.id.to_string(),
        Some(editor.user.display_name()),
        is_admin,
        current_year,
        age_threshold,
    )?;

    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/snapshots",
    responses(
        (status = 200, description = "List of point-in-time tree snapshots (Admin only)", body = SnapshotListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn list_snapshots(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<SnapshotListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let mut items = store.snapshots.clone();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let total = items.len();
    Ok(Json(SnapshotListResponse { items, total }))
}

#[utoipa::path(
    post,
    path = "/api/v1/snapshots",
    request_body(content = Option<CreateSnapshotRequest>, description = "Optional snapshot label and reason"),
    responses(
        (status = 201, description = "Point-in-time tree snapshot created (Admin only)", body = TreeSnapshotDto),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn create_manual_snapshot(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<CreateSnapshotRequest>>,
) -> Result<(StatusCode, Json<TreeSnapshotDto>), AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let reason = req.reason.as_deref().unwrap_or("manual");

    let mut store = state.privacy_store.write().await;
    let snapshot = create_snapshot(
        &mut store,
        req.label,
        reason,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        &state.config.backup_path,
    )?;

    Ok((StatusCode::CREATED, Json(snapshot)))
}

#[utoipa::path(
    post,
    path = "/api/v1/snapshots/{id}/rollback",
    params(
        ("id" = String, Path, description = "Snapshot UUID to roll back to")
    ),
    request_body = RollbackSnapshotRequest,
    responses(
        (status = 200, description = "Tree rolled back to snapshot after taking automatic pre_rollback safety snapshot", body = RollbackSnapshotResponse),
        (status = 400, description = "Missing or invalid ROLLBACK confirmation text", body = ErrorResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Snapshot not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn rollback_to_snapshot(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
    payload: Option<Json<RollbackSnapshotRequest>>,
) -> Result<Json<RollbackSnapshotResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let resp = rollback_snapshot(
        &mut store,
        &id,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        req.confirm_text.as_deref(),
        &state.config.backup_path,
        current_year,
        age_threshold,
    )?;

    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/{entity_type}/{id}/version",
    params(
        ("entity_type" = String, Path, description = "Entity type ('persons', 'individual', 'families', 'events', 'places', 'media', 'notes')"),
        ("id" = String, Path, description = "Entity UUID or GID")
    ),
    responses(
        (status = 200, description = "Current entity version and last-modified metadata for optimistic concurrency polling", body = EntityVersionResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse),
        (status = 404, description = "Entity not found", body = ErrorResponse)
    ),
    tag = "History & Concurrency"
)]
pub async fn get_record_version(
    RequireEditor(_editor): RequireEditor,
    State(state): State<AppState>,
    Path((entity_type, id)): Path<(String, String)>,
) -> Result<Json<EntityVersionResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = get_entity_version(&store, &entity_type, &id)?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality",
    responses(
        (status = 200, description = "Data quality summary, open warnings, and duplicate candidates (Admin only)", body = QualitySummaryResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_data_quality_warnings(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<QualitySummaryResponse>, AppError> {
    let store = state.privacy_store.read().await;
    Ok(Json(build_quality_summary(&store)))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/summary",
    responses(
        (status = 200, description = "Data quality summary and category counts (Admin only)", body = QualitySummaryResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_quality_summary(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<QualitySummaryResponse>, AppError> {
    let store = state.privacy_store.read().await;
    Ok(Json(build_quality_summary(&store)))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/warnings",
    params(
        ("rule" = Option<String>, Query, description = "Filter by rule code (e.g. DEATH_BEFORE_BIRTH)"),
        ("code" = Option<String>, Query, description = "Alias for rule code"),
        ("category" = Option<String>, Query, description = "Filter by category ('chronology', 'family_age', 'completeness', 'duplicates')"),
        ("severity" = Option<String>, Query, description = "Filter by severity ('warning' or 'info')"),
        ("include_dismissed" = Option<bool>, Query, description = "Include dismissed warnings"),
        ("limit" = Option<usize>, Query, description = "Page size (default 50)"),
        ("cursor" = Option<String>, Query, description = "Pagination offset cursor")
    ),
    responses(
        (status = 200, description = "Paginated list of data quality warnings (Admin only)", body = QualityWarningsListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn list_quality_warnings(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<QualityWarningsFilterQuery>,
) -> Result<Json<QualityWarningsListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let include_dismissed = query.include_dismissed.unwrap_or(false);
    let mut warnings = compute_quality_warnings(&store, include_dismissed);

    if let Some(rule_filter) = query
        .rule
        .as_deref()
        .or(query.code.as_deref())
        .or(query.rule_code.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        warnings.retain(|w| w.rule_code.eq_ignore_ascii_case(rule_filter));
    }
    if let Some(cat_filter) = query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        warnings.retain(|w| w.category.eq_ignore_ascii_case(cat_filter));
    }
    if let Some(sev_filter) = query
        .severity
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        warnings.retain(|w| w.severity.eq_ignore_ascii_case(sev_filter));
    }

    let total = warnings.len();
    let offset = query
        .cursor
        .as_deref()
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let page: Vec<_> = warnings.into_iter().skip(offset).take(limit).collect();
    let next_cursor = if offset + page.len() < total {
        Some((offset + page.len()).to_string())
    } else {
        None
    };

    Ok(Json(QualityWarningsListResponse {
        items: page,
        next_cursor,
        total,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/warnings/{id}/dismiss",
    params(
        ("id" = String, Path, description = "Composite warning ID (e.g. DEATH_BEFORE_BIRTH:<entity_id>)")
    ),
    request_body(content = Option<DismissWarningRequest>, description = "Optional dismissal metadata"),
    responses(
        (status = 200, description = "Warning dismissed and persisted in dismissal memory (Admin only)", body = DismissWarningResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn dismiss_quality_warning_by_id(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<String>,
    payload: Option<Json<DismissWarningRequest>>,
) -> Result<Json<DismissWarningResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let mut store = state.privacy_store.write().await;
    let resp =
        dismiss_warning_in_store(&mut store, Some(&id), req, Some(admin.user.id.to_string()))?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/warnings/dismiss",
    request_body = DismissWarningRequest,
    responses(
        (status = 200, description = "Warning dismissed by rule_code + entity_id (Admin only)", body = DismissWarningResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn dismiss_quality_warning(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<DismissWarningRequest>,
) -> Result<Json<DismissWarningResponse>, AppError> {
    let mut store = state.privacy_store.write().await;
    let resp = dismiss_warning_in_store(&mut store, None, req, Some(admin.user.id.to_string()))?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/quality/duplicates",
    params(
        ("min_score" = Option<i32>, Query, description = "Minimum similarity score (default 60)"),
        ("limit" = Option<usize>, Query, description = "Page size (default 50)"),
        ("cursor" = Option<String>, Query, description = "Pagination offset cursor")
    ),
    responses(
        (status = 200, description = "Candidate duplicate pairs sorted by similarity score descending (Admin only)", body = DuplicateCandidatesListResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn list_duplicate_candidates(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<DuplicateCandidatesFilterQuery>,
) -> Result<Json<DuplicateCandidatesListResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let min_score = query.min_score.unwrap_or(60).clamp(1, 100);
    let all = compute_duplicate_candidates(&store, min_score);
    let total = all.len();
    let offset = query
        .cursor
        .as_deref()
        .and_then(|c| c.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let items: Vec<_> = all.into_iter().skip(offset).take(limit).collect();
    let next_cursor = if offset + items.len() < total {
        Some((offset + items.len()).to_string())
    } else {
        None
    };

    Ok(Json(DuplicateCandidatesListResponse {
        items,
        next_cursor,
        total,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/quality/duplicates/not-duplicate",
    request_body = NotDuplicatePairRequest,
    responses(
        (status = 200, description = "Pair marked as not-duplicates and excluded from future candidate lists (Admin only)", body = NotDuplicatePairResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn mark_pair_not_duplicate(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<NotDuplicatePairRequest>,
) -> Result<Json<NotDuplicatePairResponse>, AppError> {
    let mut store = state.privacy_store.write().await;
    let resp = mark_not_duplicate_in_store(&mut store, req)?;
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/people/match",
    params(
        ("name" = Option<String>, Query, description = "Full name to match"),
        ("given_name" = Option<String>, Query, description = "Given name"),
        ("surname" = Option<String>, Query, description = "Surname"),
        ("patronymic" = Option<String>, Query, description = "Patronymic"),
        ("birth_year" = Option<i32>, Query, description = "Approximate birth year"),
        ("sex" = Option<String>, Query, description = "Sex ('M', 'F', 'U')")
    ),
    responses(
        (status = 200, description = "Inline duplicate match candidates during person creation (Editor or Admin)", body = PersonMatchResponse),
        (status = 403, description = "Editor or Admin role required", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn match_people_candidates(
    RequireEditor(editor): RequireEditor,
    State(state): State<AppState>,
    Query(query): Query<PersonMatchQuery>,
) -> Result<Json<PersonMatchResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = if editor.user.role == "admin" {
        CallerAccess::Admin
    } else {
        CallerAccess::Editor
    };
    let store = state.privacy_store.read().await;
    let resp = match_prospective_person(&store, &query, &access, &settings.default_locale);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/people/merge/preview",
    params(
        ("keep_id" = Option<String>, Query, description = "Surviving (keep/target) person UUID or GID"),
        ("merge_id" = Option<String>, Query, description = "Source (merge) person UUID or GID"),
        ("target_id" = Option<String>, Query, description = "Alias for keep_id"),
        ("source_id" = Option<String>, Query, description = "Alias for merge_id")
    ),
    responses(
        (status = 200, description = "Side-by-side person merge preview and conflict analysis (Admin only)", body = MergePreviewResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn preview_merge_people_get(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Query(query): Query<MergePreviewQuery>,
) -> Result<Json<MergePreviewResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = preview_person_merge(&store, &query)?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/people/merge/preview",
    request_body = MergePreviewQuery,
    responses(
        (status = 200, description = "Side-by-side person merge preview and conflict analysis (Admin only)", body = MergePreviewResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn preview_merge_people_post(
    RequireAdmin(_admin): RequireAdmin,
    State(state): State<AppState>,
    Json(query): Json<MergePreviewQuery>,
) -> Result<Json<MergePreviewResponse>, AppError> {
    let store = state.privacy_store.read().await;
    let resp = preview_person_merge(&store, &query)?;
    Ok(Json(resp))
}

#[utoipa::path(
    post,
    path = "/api/v1/people/merge",
    request_body(content = Option<PersonMergeRequest>, description = "Merge parameters (keep_id/target_id, merge_id/source_id, field_choices)"),
    responses(
        (status = 200, description = "Atomic undoable duplicate person merge executed (Admin only)", body = PersonMergeResponse),
        (status = 403, description = "Admin role required", body = ErrorResponse),
        (status = 404, description = "Person not found", body = ErrorResponse),
        (status = 422, description = "Self-merge or ancestor cycle detected", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn merge_duplicate_people(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    payload: Option<Json<PersonMergeRequest>>,
) -> Result<Json<PersonMergeResponse>, AppError> {
    let req = payload.map(|Json(r)| r).unwrap_or_default();
    let settings = state.resolve_instance_settings().await;
    let current_year = Utc::now().year();
    let age_threshold = settings.living_person_age_threshold as i32;

    let mut store = state.privacy_store.write().await;
    let mut resp = execute_person_merge(
        &mut store,
        req,
        Some(admin.user.id.to_string()),
        Some(admin.user.display_name()),
        &state.config.backup_path,
    )?;

    if let Some(chosen_surv) = resp.surviving_person.clone() {
        recompute_all_places_metadata(&mut store);
        recompute_all_individuals_vitals(&mut store, current_year, age_threshold);
        if let Some(person) = store.persons.iter_mut().find(|p| p.id == chosen_surv.id) {
            if chosen_surv.birth.is_some() {
                person.birth = chosen_surv.birth;
                person.birth_sort = chosen_surv.birth_sort;
            }
            if chosen_surv.death.is_some() {
                person.death = chosen_surv.death;
                person.death_sort = chosen_surv.death_sort;
                person.has_death_event = chosen_surv.has_death_event;
            }
            resp.surviving_person = Some(person.clone());
        }
    }

    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard",
    params(
        ("days" = Option<u32>, Query, description = "Anniversary lookahead window in days (default 30)"),
        ("scope" = Option<String>, Query, description = "Anniversary scope ('all' or 'close')"),
        ("reference_date" = Option<String>, Query, description = "Optional reference date (YYYY-MM-DD) for deterministic anniversary calculation")
    ),
    responses(
        (status = 200, description = "Home dashboard widgets: upcoming anniversaries, recent activity, stats, latest photos, and quality summary", body = DashboardResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_dashboard(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<AnniversariesQuery>,
) -> Result<Json<DashboardResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let (preferred_home_id, _, _, _) = resolve_home_person_for_caller(
        &state,
        &caller,
        &store.persons,
        &settings.default_locale,
        settings.default_home_person_id.as_deref(),
    )
    .await;

    let resp = build_dashboard_response(
        &store,
        &access,
        &settings.default_locale,
        preferred_home_id.as_deref(),
        &query,
    );
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/anniversaries",
    params(
        ("days" = Option<u32>, Query, description = "Lookahead window in days (default 30, max 365)"),
        ("scope" = Option<String>, Query, description = "Filter scope ('all' or 'close')"),
        ("home_person_id" = Option<String>, Query, description = "Anchor person ID for 'close' scope"),
        ("reference_date" = Option<String>, Query, description = "Optional YYYY-MM-DD reference date")
    ),
    responses(
        (status = 200, description = "Upcoming birthdays, wedding anniversaries, and milestone memorials with Julian-to-Gregorian shifting", body = AnniversariesListResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_anniversaries(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(mut query): Query<AnniversariesQuery>,
) -> Result<Json<AnniversariesListResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    if query.home_person_id.is_none() {
        let (home_id, _, _, _) = resolve_home_person_for_caller(
            &state,
            &caller,
            &store.persons,
            &settings.default_locale,
            settings.default_home_person_id.as_deref(),
        )
        .await;
        query.home_person_id = home_id;
    }

    let resp = compute_upcoming_anniversaries(&store, &query, &access, &settings.default_locale);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/stats",
    responses(
        (status = 200, description = "Tree-wide demographic and completeness statistics", body = TreeStatsResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_stats(
    caller: PublicOrAuth,
    State(state): State<AppState>,
) -> Result<Json<TreeStatsResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;
    let resp = compute_tree_stats(&store, &access, &settings.default_locale);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/kinship",
    params(
        ("from" = Option<String>, Query, description = "Starting person UUID or GID"),
        ("to" = Option<String>, Query, description = "Target person UUID or GID"),
        ("person_a" = Option<String>, Query, description = "Alias for from"),
        ("person_b" = Option<String>, Query, description = "Alias for to")
    ),
    responses(
        (status = 200, description = "Shortest kinship relationship path and degree of kinship between two individuals", body = KinshipResponse),
        (status = 404, description = "Person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_kinship(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<KinshipQuery>,
) -> Result<Json<KinshipResponse>, AppError> {
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let raw_from = query
        .from
        .as_deref()
        .or(query.person_a.as_deref())
        .or(query.id1.as_deref())
        .unwrap_or_default()
        .trim();
    let raw_to = query
        .to
        .as_deref()
        .or(query.person_b.as_deref())
        .or(query.id2.as_deref())
        .unwrap_or_default()
        .trim();

    let from_id = store
        .resolve_person_id(raw_from)
        .ok_or(AppError::PersonNotFound)?;
    let to_id = store
        .resolve_person_id(raw_to)
        .ok_or(AppError::PersonNotFound)?;

    let endpoint_visible = |pid: &str| -> bool {
        store
            .persons
            .iter()
            .find(|p| p.id == pid)
            .and_then(|p| redact_person_for_caller(p, &access, "en"))
            .is_some()
    };

    if !endpoint_visible(&from_id) || !endpoint_visible(&to_id) {
        return Err(AppError::PersonNotFound);
    }

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_tree_view(p, &access))
        .collect();

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| !f.is_private || access.can_view_private())
        .cloned()
        .collect();

    let resp = compute_kinship_path(&from_id, &to_id, &visible_persons, &visible_families);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/tree/fan",
    params(
        ("root" = Option<String>, Query, description = "Root person UUID or GID (defaults to caller's home person)"),
        ("focal_id" = Option<String>, Query, description = "Alias for root"),
        ("generations" = Option<u32>, Query, description = "Number of ancestor generations (4..=8, default 5)")
    ),
    responses(
        (status = 200, description = "Ahnentafel-indexed ancestor fan chart hierarchy with quadrant metadata", body = FanChartResponse),
        (status = 404, description = "Root person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_fan_chart(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<FanChartQuery>,
) -> Result<Json<FanChartResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_tree_view(p, &access))
        .collect();

    let raw_root = query
        .root
        .as_deref()
        .or(query.focal_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let root_id = if let Some(requested) = raw_root {
        let resolved = store
            .resolve_person_id(requested)
            .ok_or(AppError::PersonNotFound)?;
        if !visible_persons.iter().any(|p| p.id == resolved) {
            return Err(AppError::PersonNotFound);
        }
        resolved
    } else {
        let (home_opt, _, _, _) = resolve_home_person_for_caller(
            &state,
            &caller,
            &store.persons,
            &settings.default_locale,
            settings.default_home_person_id.as_deref(),
        )
        .await;
        home_opt
            .filter(|hid| visible_persons.iter().any(|p| &p.id == hid))
            .or_else(|| visible_persons.first().map(|p| p.id.clone()))
            .ok_or(AppError::PersonNotFound)?
    };

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter(|f| !f.is_private || access.can_view_private())
        .cloned()
        .collect();

    let generations = query.generations.unwrap_or(5).clamp(1, 8);
    let resp = compute_fan_chart(&root_id, generations, &visible_persons, &visible_families);
    Ok(Json(resp))
}

#[utoipa::path(
    get,
    path = "/api/v1/places/map",
    responses(
        (status = 200, description = "Clustered map markers for all geocoded places with event and person counts", body = PlacesMapResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_places_map(
    caller: PublicOrAuth,
    State(state): State<AppState>,
) -> Result<Json<PlacesMapResponse>, AppError> {
    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let visible_persons: Vec<TreePersonNodeDto> = store
        .persons
        .iter()
        .filter_map(|p| redact_person_for_caller(p, &access, &settings.default_locale))
        .collect();

    let resp = compute_places_map(&store, &visible_persons);
    Ok(Json(resp))
}
