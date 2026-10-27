use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{models::gendate::GenDate, services::rbac::CallerAccess};

fn default_version() -> i32 {
    1
}

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

/// Suggests a gendered Slavic patronymic from a father's given name and child's sex (`07 §3 #4, §5 DoD #8`).
/// Supports both Russian Cyrillic (`Иван` -> `Иванович`/`Ивановна`, `Сергей` -> `Сергеевич`/`Сергеевна`,
/// `Илья` -> `Ильич`/`Ильинична`, `Никита` -> `Никитич`/`Никитична`, `Павел` -> `Павлович`/`Павловна`,
/// `Лев` -> `Львович`/`Львовна`, `Игорь` -> `Игоревич`/`Игоревна`, `Михаил` -> `Михайлович`/`Михайловна`)
/// and Latin transliterated forms (`Ivan` -> `Ivanovich`/`Ivanovna`, `Mikhail` -> `Mikhailovich`/`Mikhailovna`).
pub fn suggest_slavic_patronymic(father_given_name: &str, child_sex: &str) -> Option<String> {
    let name = father_given_name.trim();
    if name.is_empty() {
        return None;
    }

    let is_female = matches!(
        child_sex.trim().to_ascii_lowercase().as_str(),
        "f" | "female"
    );
    let is_male = matches!(child_sex.trim().to_ascii_lowercase().as_str(), "m" | "male");
    if !is_female && !is_male {
        return None;
    }

    let lower = name.to_lowercase();

    // Irregular & special Russian Cyrillic names
    let special_ru: Option<(&str, &str)> = match lower.as_str() {
        "илья" => Some(("Ильич", "Ильинична")),
        "никита" => Some(("Никитич", "Никитична")),
        "савва" => Some(("Саввич", "Саввична")),
        "фома" => Some(("Фомич", "Фоминична")),
        "лука" => Some(("Лукич", "Лукинична")),
        "кузьма" => Some(("Кузьмич", "Кузьминична")),
        "яков" => Some(("Яковлевич", "Яковлевна")),
        "павел" => Some(("Павлович", "Павловна")),
        "лев" => Some(("Львович", "Львовна")),
        "пётр" | "петр" => Some(("Петрович", "Петровна")),
        "михаил" => Some(("Михайлович", "Михайловна")),
        "игорь" => Some(("Игоревич", "Игоревна")),
        "василий" => Some(("Васильевич", "Васильевна")),
        "георгий" => Some(("Георгиевич", "Георгиевна")),
        "дмитрий" => Some(("Дмитриевич", "Дмитриевна")),
        "анатолий" => Some(("Анатольевич", "Анатольевна")),
        "валерий" => Some(("Валерьевич", "Валерьевна")),
        "евгений" => Some(("Евгеньевич", "Евгеньевна")),
        "геннадий" => Some(("Геннадьевич", "Геннадьевна")),
        "виталий" => Some(("Витальевич", "Витальевна")),
        "аркадий" => Some(("Аркадьевич", "Аркадьевна")),
        "григорий" => Some(("Григорьевич", "Григорьевна")),
        "юрий" => Some(("Юрьевич", "Юрьевна")),
        "сергей" => Some(("Сергеевич", "Сергеевна")),
        "андрей" => Some(("Андреевич", "Андреевна")),
        "алексей" => Some(("Алексеевич", "Алексеевна")),
        "матвей" => Some(("Матвеевич", "Матвеевна")),
        "тимофей" => Some(("Тимофеевич", "Тимофеевна")),
        "николай" => Some(("Николаевич", "Николаевна")),
        _ => None,
    };

    if let Some((male_form, female_form)) = special_ru {
        return Some(if is_female {
            female_form.to_string()
        } else {
            male_form.to_string()
        });
    }

    // Irregular & common Latin transliterated Slavic names
    let special_en: Option<(&str, &str)> = match lower.as_str() {
        "ilya" => Some(("Ilyich", "Ilyinichna")),
        "nikita" => Some(("Nikitich", "Nikitichna")),
        "pavel" => Some(("Pavlovich", "Pavlovna")),
        "lev" => Some(("Lvovich", "Lvovna")),
        "pyotr" | "petr" => Some(("Petrovich", "Petrovna")),
        "mikhail" => Some(("Mikhailovich", "Mikhailovna")),
        "igor" => Some(("Igorevich", "Igorevna")),
        "sergey" | "sergei" => Some(("Sergeevich", "Sergeevna")),
        "andrey" | "andrei" => Some(("Andreevich", "Andreevna")),
        "aleksey" | "aleksei" | "alexey" => Some(("Alekseevich", "Alekseevna")),
        "nikolay" | "nikolai" => Some(("Nikolaevich", "Nikolaevna")),
        "dmitry" | "dmitri" => Some(("Dmitrievich", "Dmitrievna")),
        "yuri" | "yury" => Some(("Yuryevich", "Yuryevna")),
        "vasily" | "vasili" => Some(("Vasilyevich", "Vasilyevna")),
        _ => None,
    };

    if let Some((male_form, female_form)) = special_en {
        return Some(if is_female {
            female_form.to_string()
        } else {
            male_form.to_string()
        });
    }

    let is_cyrillic = name.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
    if is_cyrillic {
        if let Some(stem) = name.strip_suffix('й').or_else(|| name.strip_suffix('ь')) {
            return Some(if is_female {
                format!("{stem}евна")
            } else {
                format!("{stem}евич")
            });
        }
        if let Some(stem) = name.strip_suffix('а').or_else(|| name.strip_suffix('я')) {
            return Some(if is_female {
                format!("{stem}инична")
            } else {
                format!("{stem}ич")
            });
        }
        return Some(if is_female {
            format!("{name}овна")
        } else {
            format!("{name}ович")
        });
    }

    // Standard Latin stem
    Some(if is_female {
        format!("{name}ovna")
    } else {
        format!("{name}ovich")
    })
}

/// Normalizes Cyrillic and Latin text into a common lowercase Latin representation for
/// cross-script search (`"Тверь"` <-> `"tver"`, `"Калинин"` <-> `"kalinin"`).
pub fn normalize_search_tokens(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for ch in input.trim().to_lowercase().chars() {
        match ch {
            'а' => out.push('a'),
            'б' => out.push('b'),
            'в' => out.push('v'),
            'г' => out.push('g'),
            'д' => out.push('d'),
            'е' | 'ё' | 'э' => out.push('e'),
            'ж' => out.push_str("zh"),
            'з' => out.push('z'),
            'и' | 'й' | 'ы' => out.push('i'),
            'к' => out.push('k'),
            'л' => out.push('l'),
            'м' => out.push('m'),
            'н' => out.push('n'),
            'о' => out.push('o'),
            'п' => out.push('p'),
            'р' => out.push('r'),
            'с' => out.push('s'),
            'т' => out.push('t'),
            'у' => out.push('u'),
            'ф' => out.push('f'),
            'х' => out.push_str("kh"),
            'ц' => out.push_str("ts"),
            'ч' => out.push_str("ch"),
            'ш' => out.push_str("sh"),
            'щ' => out.push_str("shch"),
            'ъ' | 'ь' => {}
            'ю' => out.push_str("yu"),
            'я' => out.push_str("ya"),
            'w' => out.push('v'),
            other => out.push(other),
        }
    }
    out
}

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
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_sort: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_sort: Option<i32>,
    #[serde(default)]
    pub has_death_event: bool,
    #[serde(default)]
    pub alt_names: Vec<IndividualAltNameDto>,
    #[serde(default = "default_version")]
    pub version: i32,
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

fn default_child_rel() -> String {
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
    #[serde(default = "default_version")]
    pub version: i32,
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

fn default_place_type() -> String {
    "other".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ChangeLogEntryDto {
    pub id: String,
    pub op: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_json: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_json: Option<serde_json::Value>,
    pub created_at: String,
}

/// Applies the RBAC & Living-Person Redaction Contract (`02-rbac-access-control.md §3.2, §3.3, §4.2`).
/// - Returns `None` if `person.is_private` and caller does not have `Editor` or `Admin` access.
/// - For `AnonymousPublic`: masks living individuals (`given_name: "Living"` / `"Живущий"`,
///   `patronymic: None`, `birth: None`, `death: None`, `avatar_url: None`, `notes: None`, `biography: None`).
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
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: None,
            birth: None,
            death: None,
            avatar_url: None,
            notes: None,
            birth_sort: None,
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            version: person.version,
        });
    }

    Some(person.clone())
}

/// Redacts a media asset for the given caller (`02-rbac-access-control.md §3.2, §7 DoD #3`).
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
    pub families: Vec<FamilyDto>,
    pub events: Vec<LifeEventDto>,
    pub notes: Vec<NoteDto>,
    pub media: Vec<MediaAssetSummaryDto>,
    pub places: Vec<PlaceSummaryDto>,
    pub change_log: Vec<ChangeLogEntryDto>,
    pub next_gid_seq: u32,
    pub next_family_gid_seq: u32,
    pub next_event_gid_seq: u32,
    pub next_place_gid_seq: u32,
    pub next_media_gid_seq: u32,
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
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: Some("Archival engineer and family chronicler.".to_string()),
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
            birth_sort: Some(19120319),
            death_sort: Some(19881104),
            has_death_event: true,
            alt_names: vec![],
            version: 1,
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
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: Some("Graduated from Saint Petersburg State University.".to_string()),
            birth: Some(VitalEventSummaryDto {
                date: Some("1994-06-14".to_string()),
                place: Some("Saint Petersburg, Russia".to_string()),
            }),
            death: None,
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000102/file".to_string()),
            notes: Some("Graduated from Saint Petersburg State University.".to_string()),
            birth_sort: Some(19940614),
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            version: 1,
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
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: None,
            birth: Some(VitalEventSummaryDto {
                date: Some("1989-11-02".to_string()),
                place: Some("Moscow, Russia".to_string()),
            }),
            death: None,
            avatar_url: Some("/api/v1/media/00000000-0000-0000-0000-000000000103/file".to_string()),
            notes: Some("Confidential adoption research note.".to_string()),
            birth_sort: Some(19891102),
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            version: 1,
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
                gid: "P0001".to_string(),
                name: "Saint Petersburg".to_string(),
                place_type: "city".to_string(),
                parent_id: None,
                full_name: "Saint Petersburg, Russia".to_string(),
                country: Some("Russia".to_string()),
                latitude: Some(59.9343),
                longitude: Some(30.3351),
                events_count: 4,
                children_count: 0,
                alt_names: vec![
                    PlaceAltNameDto {
                        id: "pan-01".to_string(),
                        name: "Leningrad".to_string(),
                        lang: Some("en".to_string()),
                        note: Some("1924–1991 Soviet name".to_string()),
                    },
                    PlaceAltNameDto {
                        id: "pan-02".to_string(),
                        name: "Санкт-Петербург".to_string(),
                        lang: Some("ru".to_string()),
                        note: None,
                    },
                ],
                version: 1,
            },
            PlaceSummaryDto {
                id: "00000000-0000-0000-0000-000000000202".to_string(),
                gid: "P0002".to_string(),
                name: "Vitebsk".to_string(),
                place_type: "city".to_string(),
                parent_id: None,
                full_name: "Vitebsk, Belarus".to_string(),
                country: Some("Belarus".to_string()),
                latitude: Some(55.1904),
                longitude: Some(30.2049),
                events_count: 2,
                children_count: 0,
                alt_names: vec![PlaceAltNameDto {
                    id: "pan-03".to_string(),
                    name: "Витебск".to_string(),
                    lang: Some("ru".to_string()),
                    note: None,
                }],
                version: 1,
            },
        ];

        Self {
            persons: vec![deceased_ancestor, living_relative, private_record],
            families: vec![],
            events: vec![],
            notes: vec![],
            media: vec![deceased_media, living_media, private_media],
            places,
            change_log: vec![],
            next_gid_seq: 100,
            next_family_gid_seq: 100,
            next_event_gid_seq: 100,
            next_place_gid_seq: 100,
            next_media_gid_seq: 100,
        }
    }
}
