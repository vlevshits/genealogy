#[allow(unused_imports)]
use super::*;

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
    pub is_missing: Option<bool>,
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

pub(crate) fn resolve_person_id_in_store(
    persons: &[TreePersonNodeDto],
    id_or_gid: &str,
) -> Option<String> {
    let target = id_or_gid.trim();
    persons
        .iter()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .map(|p| p.id.clone())
}

pub(crate) fn resolve_place_id_in_store(
    places: &[PlaceSummaryDto],
    id_or_gid: &str,
) -> Option<String> {
    let target = id_or_gid.trim();
    places
        .iter()
        .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
        .map(|p| p.id.clone())
}

pub(crate) fn is_family_only_event_type(event_type: &str) -> bool {
    matches!(
        event_type.trim().to_ascii_lowercase().as_str(),
        "engagement" | "marriage" | "marriage_license" | "divorce"
    )
}

pub(crate) async fn resolve_home_person_for_caller(
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
