use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::services::rbac::CallerAccess;

/// Input parameters for the canonical 5-branch living status algorithm (`10-core-data-model.md §5`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PersonLivingEvaluationInput {
    pub living_override: Option<bool>,
    pub has_death_or_burial_event: bool,
    pub birth_year: Option<i32>,
    pub earliest_child_birth_year: Option<i32>,
    pub partner_birth_year: Option<i32>,
    pub parent_birth_year: Option<i32>,
    pub marriage_year: Option<i32>,
}

/// Evaluates whether a person is considered living according to `docs/features/10-core-data-model.md §5`:
/// 1. If `living_override` is `Some(val)` -> return `val`.
/// 2. If `has_death_or_burial_event` is `true` -> `false` (deceased).
/// 3. If `birth_year` is known -> `(current_year - birth_year) < age_threshold`.
/// 4. Estimate birth year from closest relatives (first match wins):
///    - `earliest_child_birth_year - 25`
///    - `partner_birth_year`
///    - `parent_birth_year + 25`
///    - `marriage_year - 25`
///      If estimate exists -> `(current_year - estimated_birth_year) < age_threshold`.
/// 5. Otherwise -> `true` (unknown = living = protected).
pub fn evaluate_is_living(
    input: &PersonLivingEvaluationInput,
    current_year: i32,
    age_threshold: i32,
) -> bool {
    if let Some(override_val) = input.living_override {
        return override_val;
    }

    if input.has_death_or_burial_event {
        return false;
    }

    if let Some(by) = input.birth_year {
        return (current_year - by) < age_threshold;
    }

    let estimated_birth_year = input
        .earliest_child_birth_year
        .map(|y| y - 25)
        .or(input.partner_birth_year)
        .or_else(|| input.parent_birth_year.map(|y| y + 25))
        .or_else(|| input.marriage_year.map(|y| y - 25));

    if let Some(est_by) = estimated_birth_year {
        return (current_year - est_by) < age_threshold;
    }

    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PersonNameDto {
    pub given_name: String,
    pub patronymic: Option<String>,
    pub surname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VitalEventSummaryDto {
    pub date: Option<String>,
    pub place: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TreePersonNodeDto {
    pub id: String,
    pub gid: String,
    pub sex: String,
    pub is_living: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub living_override: Option<bool>,
    pub is_private: bool,
    pub primary_name: PersonNameDto,
    pub birth: Option<VitalEventSummaryDto>,
    pub death: Option<VitalEventSummaryDto>,
    pub avatar_url: Option<String>,
    pub notes: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct MediaAssetSummaryDto {
    pub id: String,
    pub gid: String,
    pub title: String,
    pub mime_type: String,
    pub file_url: String,
    pub is_private: bool,
    pub only_living_persons: bool,
    pub linked_person_ids: Vec<String>,
    pub face_tags: Vec<FaceTagDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PlaceSummaryDto {
    pub id: String,
    pub name: String,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub events_count: u32,
}

/// Applies the RBAC & Living-Person Redaction Contract (`02-rbac-access-control.md §3.2, §3.3, §4.2`).
/// - Returns `None` if `person.is_private` and caller does not have `Editor` or `Admin` access.
/// - For `AnonymousPublic`: masks living individuals (`given_name: "Living"` / `"Живущий"`,
///   `patronymic: None`, `birth: None`, `death: None`, `avatar_url: None`, `notes: None`).
/// - For `Viewer`, `Editor`, and `Admin`: returns living individuals unredacted.
pub fn redact_person_for_caller(
    person: &TreePersonNodeDto,
    access: &CallerAccess,
    locale: &str,
) -> Option<TreePersonNodeDto> {
    if person.is_private && !access.can_view_private() {
        return None;
    }

    if *access == CallerAccess::AnonymousPublic && person.is_living {
        let masked_given = if locale.trim().eq_ignore_ascii_case("ru") {
            "Живущий".to_string()
        } else {
            "Living".to_string()
        };
        return Some(TreePersonNodeDto {
            id: person.id.clone(),
            gid: person.gid.clone(),
            sex: person.sex.clone(),
            is_living: true,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: masked_given,
                patronymic: None,
                surname: person.primary_name.surname.clone(),
            },
            birth: None,
            death: None,
            avatar_url: None,
            notes: None,
        });
    }

    Some(person.clone())
}

/// Redacts a media asset for the given caller (`02-rbac-access-control.md §3.2, §7 DoD #3`).
/// - Private media assets are hidden from `AnonymousPublic` and `Viewer`.
/// - Media assets depicting only living persons are hidden (`404 Not Found`) from `AnonymousPublic`.
/// - Group photos containing both deceased and living persons remain viewable in `AnonymousPublic`,
///   but face-tag bounding boxes for living persons are stripped (`02 §3.2`).
pub fn redact_media_for_caller(
    media: &MediaAssetSummaryDto,
    access: &CallerAccess,
) -> Option<MediaAssetSummaryDto> {
    if media.is_private && !access.can_view_private() {
        return None;
    }

    if *access == CallerAccess::AnonymousPublic {
        if media.only_living_persons {
            return None;
        }
        let mut cloned = media.clone();
        cloned.face_tags.retain(|ft| !ft.is_living);
        return Some(cloned);
    }

    Some(media.clone())
}

#[derive(Debug, Clone)]
pub struct GenealogyPrivacyStore {
    pub persons: Vec<TreePersonNodeDto>,
    pub media: Vec<MediaAssetSummaryDto>,
    pub places: Vec<PlaceSummaryDto>,
    pub next_gid_seq: u32,
}

impl Default for GenealogyPrivacyStore {
    fn default() -> Self {
        let deceased_ancestor = TreePersonNodeDto {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            gid: "I0001".to_string(),
            sex: "M".to_string(),
            is_living: false,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: "Mikhail".to_string(),
                patronymic: Some("Isaakovich".to_string()),
                surname: "Levshits".to_string(),
            },
            birth: Some(VitalEventSummaryDto {
                date: Some("1912-03-19".to_string()),
                place: Some("Vitebsk, Belarus".to_string()),
            }),
            death: Some(VitalEventSummaryDto {
                date: Some("1988-11-04".to_string()),
                place: Some("Leningrad, USSR".to_string()),
            }),
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000101/file".to_string()),
            notes: Some("Archival engineer and family chronicler.".to_string()),
        };

        let living_relative = TreePersonNodeDto {
            id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
            gid: "I0042".to_string(),
            sex: "F".to_string(),
            is_living: true,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: "Elena".to_string(),
                patronymic: Some("Alexandrovna".to_string()),
                surname: "Levshits".to_string(),
            },
            birth: Some(VitalEventSummaryDto {
                date: Some("1994-06-14".to_string()),
                place: Some("Saint Petersburg, Russia".to_string()),
            }),
            death: None,
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000102/file".to_string()),
            notes: Some("Graduated from Saint Petersburg State University.".to_string()),
        };

        let private_record = TreePersonNodeDto {
            id: "00000000-0000-0000-0000-000000000099".to_string(),
            gid: "I0099".to_string(),
            sex: "M".to_string(),
            is_living: true,
            living_override: Some(true),
            is_private: true,
            primary_name: PersonNameDto {
                given_name: "Arkady".to_string(),
                patronymic: Some("Borisovich".to_string()),
                surname: "Levshits".to_string(),
            },
            birth: Some(VitalEventSummaryDto {
                date: Some("1989-11-02".to_string()),
                place: Some("Moscow, Russia".to_string()),
            }),
            death: None,
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000103/file".to_string()),
            notes: Some("Confidential adoption research note.".to_string()),
        };

        let deceased_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000101".to_string(),
            gid: "M0001".to_string(),
            title: "Mikhail Levshits Archival Portrait (1955)".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000101/file".to_string(),
            is_private: false,
            only_living_persons: false,
            linked_person_ids: vec![
                "00000000-0000-0000-0000-000000000001".to_string(),
                "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
            ],
            face_tags: vec![
                FaceTagDto {
                    id: "ft-01".to_string(),
                    person_id: "00000000-0000-0000-0000-000000000001".to_string(),
                    person_name: "Mikhail Levshits".to_string(),
                    is_living: false,
                    x: 0.25,
                    y: 0.30,
                    width: 0.20,
                    height: 0.25,
                },
                FaceTagDto {
                    id: "ft-02".to_string(),
                    person_id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
                    person_name: "Elena Levshits".to_string(),
                    is_living: true,
                    x: 0.60,
                    y: 0.35,
                    width: 0.18,
                    height: 0.22,
                },
            ],
        };

        let living_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000102".to_string(),
            gid: "M0002".to_string(),
            title: "Elena Levshits Portrait (2024)".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000102/file".to_string(),
            is_private: false,
            only_living_persons: true,
            linked_person_ids: vec!["c7a8b9d0-1234-5678-90ab-cdef12345678".to_string()],
            face_tags: vec![FaceTagDto {
                id: "ft-03".to_string(),
                person_id: "c7a8b9d0-1234-5678-90ab-cdef12345678".to_string(),
                person_name: "Elena Levshits".to_string(),
                is_living: true,
                x: 0.40,
                y: 0.25,
                width: 0.25,
                height: 0.30,
            }],
        };

        let private_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000103".to_string(),
            gid: "M0003".to_string(),
            title: "Confidential Archival Certificate".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000103/file".to_string(),
            is_private: true,
            only_living_persons: true,
            linked_person_ids: vec!["00000000-0000-0000-0000-000000000099".to_string()],
            face_tags: vec![],
        };

        let places = vec![
            PlaceSummaryDto {
                id: "00000000-0000-0000-0000-000000000201".to_string(),
                name: "Saint Petersburg, Russia".to_string(),
                country: Some("Russia".to_string()),
                latitude: Some(59.9343),
                longitude: Some(30.3351),
                events_count: 4,
            },
            PlaceSummaryDto {
                id: "00000000-0000-0000-0000-000000000202".to_string(),
                name: "Vitebsk, Belarus".to_string(),
                country: Some("Belarus".to_string()),
                latitude: Some(55.1904),
                longitude: Some(30.2049),
                events_count: 2,
            },
        ];

        Self {
            persons: vec![deceased_ancestor, living_relative, private_record],
            media: vec![deceased_media, living_media, private_media],
            places,
            next_gid_seq: 100,
        }
    }
}
