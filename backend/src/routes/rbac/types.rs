#[allow(unused_imports)]
use super::*;

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
    /// Projection mode: `"hourglass"`, `"pedigree"`, `"descendant"`, `"fan"`, or `"all"` (default `"hourglass"`)
    pub mode: Option<String>,
    /// Generation depth limit (`1..=10`, default `4`)
    pub generations: Option<u32>,
    /// When `mode=hourglass`, whether to include collateral branches (siblings, aunts/uncles, cousins, nieces/nephews)
    #[serde(default)]
    pub collateral: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TreeEdgeDto {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: String,
    pub family_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_direct_lineage: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TreeViewResponse {
    pub access_tier: String,
    pub tree_title: String,
    pub focal_id: Option<String>,
    pub mode: String,
    pub generations: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collateral: Option<bool>,
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
pub(crate) struct DeletedPersonSnapshot {
    pub(crate) person: TreePersonNodeDto,

    pub(crate) events: Vec<LifeEventDto>,

    pub(crate) notes: Vec<NoteDto>,

    pub(crate) families_before: Vec<FamilyDto>,
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
    #[serde(alias = "gedcom_date")]
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
    #[serde(alias = "gedcom_date")]
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
