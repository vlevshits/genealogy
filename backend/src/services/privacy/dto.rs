#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PersonNameDto {
    pub given_name: String,
    pub patronymic: Option<String>,
    pub surname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct IndividualAltNameDto {
    #[serde(default)]
    pub id: String,
    pub name_type: String,
    pub given_name: Option<String>,
    pub patronymic: Option<String>,
    pub surname: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VitalEventSummaryDto {
    pub date: Option<String>,
    pub place: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AvatarCropDto {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct TreePersonNodeDto {
    pub id: String,
    pub gid: String,
    pub sex: String,
    pub is_living: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub living_override: Option<bool>,
    pub is_private: bool,
    pub primary_name: PersonNameDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_surname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_suffix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,
    pub birth: Option<VitalEventSummaryDto>,
    pub death: Option<VitalEventSummaryDto>,
    pub avatar_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_media_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_crop: Option<AvatarCropDto>,
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_sort: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_sort: Option<i32>,
    #[serde(default)]
    pub has_death_event: bool,
    #[serde(default)]
    pub alt_names: Vec<IndividualAltNameDto>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub external_ids: HashMap<String, String>,
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_direct_lineage: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct FamilyChildDto {
    pub child_id: String,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "default_child_rel")]
    pub rel_to_partner1: String,
    #[serde(default = "default_child_rel")]
    pub rel_to_partner2: String,
}

pub(crate) fn default_child_rel() -> String {
    "birth".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct FamilyDto {
    pub id: String,
    pub gid: String,
    pub partner1_id: Option<String>,
    pub partner2_id: Option<String>,
    pub relationship_type: String,
    pub is_private: bool,
    pub children: Vec<FamilyChildDto>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub external_ids: HashMap<String, String>,
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marriage_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marriage_place: Option<String>,
}

/// Extracts marriage date and place from family events (matching "marriage" or "marr").
pub fn find_family_marriage_facts(
    events: &[LifeEventDto],
    family_id: &str,
) -> (Option<String>, Option<String>) {
    let ev = events.iter().find(|e| {
        e.family_id.as_deref() == Some(family_id)
            && (e.event_type.eq_ignore_ascii_case("marriage")
                || e.event_type.eq_ignore_ascii_case("marr"))
    });
    if let Some(e) = ev {
        let date = e
            .date
            .as_ref()
            .map(|d| d.text.clone())
            .or_else(|| e.gedcom_date.clone());
        let place = e.place_full_name.clone();
        (date, place)
    } else {
        (None, None)
    }
}

/// Enriches a FamilyDto with marriage date and place if not already set.
pub fn enrich_family_dto_marriage(mut fam: FamilyDto, events: &[LifeEventDto]) -> FamilyDto {
    if fam.marriage_date.is_none() || fam.marriage_place.is_none() {
        let (d, p) = find_family_marriage_facts(events, &fam.id);
        if fam.marriage_date.is_none() {
            fam.marriage_date = d;
        }
        if fam.marriage_place.is_none() {
            fam.marriage_place = p;
        }
    }
    fam
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct LifeEventDto {
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
    #[serde(default)]
    pub media_ids: Vec<String>,
    pub is_private: bool,
    #[serde(default = "default_version")]
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct NoteDto {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub body: String,
    pub is_private: bool,
    pub sort_order: i32,
    #[serde(default = "default_version")]
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct FaceTagDto {
    pub id: String,
    pub person_id: String,
    pub person_name: String,
    pub is_living: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub is_primary_avatar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct MediaLinkDto {
    pub id: String,
    pub media_id: String,
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct MediaAssetSummaryDto {
    pub id: String,
    pub gid: String,
    pub title: String,
    pub mime_type: String,
    pub file_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_taken: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_sort: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place_full_name: Option<String>,
    #[serde(default)]
    pub is_missing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_path: Option<String>,
    pub is_private: bool,
    pub only_living_persons: bool,
    pub linked_person_ids: Vec<String>,
    pub face_tags: Vec<FaceTagDto>,
    #[serde(default)]
    pub links: Vec<MediaLinkDto>,
    #[serde(default = "default_version")]
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PlaceAltNameDto {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub lang: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PlaceBreadcrumbDto {
    pub id: String,
    pub gid: String,
    pub name: String,
    pub place_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceSummaryDto {
    pub id: String,
    #[serde(default)]
    pub gid: String,
    pub name: String,
    #[serde(default = "default_place_type")]
    pub place_type: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub full_name: String,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub events_count: u32,
    #[serde(default)]
    pub children_count: u32,
    #[serde(default)]
    pub alt_names: Vec<PlaceAltNameDto>,
    #[serde(default = "default_version")]
    pub version: i32,
}

pub(crate) fn default_place_type() -> String {
    "other".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ChangeLogEntryDto {
    pub id: String,
    #[serde(default)]
    pub tx_id: String,
    #[serde(default)]
    pub tx_label: String,
    pub op: String,
    #[serde(default)]
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_name: Option<String>,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_json: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_gid: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diffs: Vec<crate::services::history::FieldDiffDto>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TreeSnapshotRecordCountsDto {
    pub individuals: usize,
    pub families: usize,
    pub events: usize,
    pub places: usize,
    pub media: usize,
    pub notes: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TreeSnapshotDto {
    pub id: String,
    pub label: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_name: Option<String>,
    pub created_at: String,
    pub expires_at: String,
    pub record_counts: TreeSnapshotRecordCountsDto,
    #[serde(skip)]
    pub payload_json: Option<serde_json::Value>,
}
