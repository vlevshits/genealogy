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

/// Normalizes pre-1918 Russian orthography (`ѣ -> е`, `і -> и`, `ѳ -> ф`, `ѵ -> и`, trailing `ъ`)
/// and `ё -> е` on a single word token before script conversion or gender folding (`13 §2.2`).
pub fn normalize_russian_orthography_word(word: &str) -> String {
    let mut mapped = String::with_capacity(word.len());
    for ch in word.trim().to_lowercase().chars() {
        match ch {
            'ё' | 'ѣ' | 'э' => mapped.push('е'),
            'і' | 'ѵ' => mapped.push('и'),
            'ѳ' => mapped.push('ф'),
            other => mapped.push(other),
        }
    }
    // Strip trailing hard sign `ъ` at word boundary (e.g. "Левшицъ" -> "левшиц", "Петровъ" -> "петров")
    while mapped.ends_with('ъ') && mapped.chars().count() > 1 {
        mapped.pop();
    }
    mapped
}

/// Folds Slavic feminine surname endings to their masculine/stem form in both Cyrillic and Latin
/// (`-ова -> -ов`, `-ева -> -ев`, `-ина -> -ин`, `-ская -> -ский`, `-цкая -> -цкий`,
/// `-ova -> -ov`, `-eva -> -ev`, `-ina -> -in`, `-skaya -> -sky`, `-tskaya -> -tsky`) (`13 §2.2, §4.3`).
pub fn fold_slavic_surname_stem(word: &str) -> String {
    let ortho = normalize_russian_orthography_word(word);
    let char_count = ortho.chars().count();
    if char_count <= 3 {
        return ortho;
    }

    // Cyrillic feminine endings
    for (fem, masc) in [
        ("цкая", "цкий"),
        ("ская", "ский"),
        ("ова", "ов"),
        ("ева", "ев"),
        ("ина", "ин"),
    ] {
        if let Some(stem) = ortho.strip_suffix(fem) {
            if stem.chars().count() >= 2 {
                return format!("{stem}{masc}");
            }
        }
    }

    // Latin feminine endings
    for (fem, masc) in [
        ("tskaya", "tsky"),
        ("skaya", "sky"),
        ("ova", "ov"),
        ("eva", "ev"),
        ("ina", "in"),
    ] {
        if let Some(stem) = ortho.strip_suffix(fem) {
            if stem.chars().count() >= 2 {
                return format!("{stem}{masc}");
            }
        }
    }

    ortho
}

/// Produces a display-cased canonical surname (e.g. `"Петрова"` -> `"Петров"`, `"Левшицъ"` -> `"Левшиц"`).
pub fn canonicalize_surname_display(surname: &str) -> String {
    let trimmed = surname.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let folded = fold_slavic_surname_stem(trimmed);
    let mut chars = folded.chars();
    match chars.next() {
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            format!("{upper}{}", chars.as_str())
        }
        None => String::new(),
    }
}

/// Transliterates a single normalized token into a canonical Latin representation (`13 §2.2`).
pub fn transliterate_to_latin(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        match ch {
            'а' => out.push('a'),
            'б' => out.push('b'),
            'в' => out.push('v'),
            'г' => out.push('g'),
            'д' => out.push('d'),
            'е' | 'ё' | 'ѣ' | 'э' => out.push('e'),
            'ж' => out.push_str("zh"),
            'з' => out.push('z'),
            'и' | 'й' | 'ы' | 'і' | 'ѵ' => out.push('i'),
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
            'ф' | 'ѳ' => out.push('f'),
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
    // Canonicalize Latin y/i variations at endings (e.g. "sky" / "skiy" / "skii" -> "ski")
    if out.len() > 4 {
        if let Some(stem) = out
            .strip_suffix("skiy")
            .or_else(|| out.strip_suffix("skii"))
            .or_else(|| out.strip_suffix("sky"))
        {
            return format!("{stem}ski");
        }
    }
    out
}

/// Normalizes Cyrillic and Latin text into a space-separated token stream containing:
/// 1. Pre-1918 Russian orthography normalized tokens (`ѣ->е`, `і->и`, `ѳ->ф`, `ѵ->и`, trailing `ъ` stripped, `ё->е`),
/// 2. Slavic surname gender-folded stem tokens (`Петрова -> петров`, `Petrova -> petrov`), and
/// 3. Bidirectional cross-script Latin transliteration tokens (`Щеглов <-> shcheglov`, `Тверь <-> tver`).
pub fn normalize_search_tokens(input: &str) -> String {
    let mut tokens: Vec<String> = Vec::new();
    for raw_word in input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
    {
        let ortho = normalize_russian_orthography_word(raw_word);
        let folded = fold_slavic_surname_stem(&ortho);
        let latin_ortho = transliterate_to_latin(&ortho);
        let latin_folded = transliterate_to_latin(&folded);

        for candidate in [ortho, folded, latin_ortho, latin_folded] {
            if !candidate.is_empty() && !tokens.contains(&candidate) {
                tokens.push(candidate);
            }
        }
    }
    tokens.join(" ")
}

/// Parsed search query separating free-text name tokens from an optional 4-digit birth year hint (`13 §2.3`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSearchQuery {
    pub raw: String,
    pub text_tokens: Vec<String>,
    pub year_hint: Option<i32>,
    pub exact_gid: Option<String>,
}

pub fn parse_search_query(q: &str) -> ParsedSearchQuery {
    let trimmed = q.trim();
    let raw_parts: Vec<&str> = trimmed.split_whitespace().collect();

    let mut year_hint: Option<i32> = None;
    let mut text_tokens: Vec<String> = Vec::new();
    let mut exact_gid: Option<String> = None;

    if raw_parts.len() == 1 {
        let single = raw_parts[0].trim_matches('@').to_ascii_uppercase();
        if single.len() >= 2
            && matches!(single.chars().next(), Some('I' | 'F' | 'E' | 'P' | 'M'))
            && single[1..].chars().all(|c| c.is_ascii_digit())
        {
            exact_gid = Some(single);
        }
    }

    for part in &raw_parts {
        if year_hint.is_none() && part.len() == 4 && part.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(yr) = part.parse::<i32>() {
                if (1000..=2100).contains(&yr) {
                    year_hint = Some(yr);
                    continue;
                }
            }
        }
        let cleaned = normalize_russian_orthography_word(part);
        if !cleaned.is_empty() {
            text_tokens.push(cleaned);
        }
    }

    ParsedSearchQuery {
        raw: trimmed.to_string(),
        text_tokens,
        year_hint,
        exact_gid,
    }
}

use std::collections::HashMap;

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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub external_ids: HashMap<String, String>,
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

fn default_place_type() -> String {
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
            avatar_media_id: None,
            avatar_crop: None,
            notes: None,
            birth_sort: None,
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: person.version,
            tx_id: None,
        });
    }

    Some(person.clone())
}

/// Applies the Tree Canvas Projection Living-Person Redaction Contract (`04-tree-visualizations.md §3.1`, `02-rbac-access-control.md §3.2, §4.2`).
/// - Returns `None` if `person.is_private` and caller does not have `Editor` or `Admin` access.
/// - For `AnonymousPublic`: masks living individuals to a single given-name initial character with a dot
///   (e.g. `"И."` / `"J."`), strips surnames, patronymics, dates, places, biographies, notes, and avatars.
/// - For `Viewer`, `Editor`, and `Admin`: returns living individuals unredacted.
pub fn redact_person_for_tree_view(
    person: &TreePersonNodeDto,
    access: &CallerAccess,
) -> Option<TreePersonNodeDto> {
    if person.is_private && !access.can_view_private() {
        return None;
    }

    if *access == CallerAccess::AnonymousPublic && person.is_living {
        let initial = person
            .primary_name
            .given_name
            .trim()
            .chars()
            .next()
            .unwrap_or('?');
        return Some(TreePersonNodeDto {
            id: person.id.clone(),
            gid: person.gid.clone(),
            sex: person.sex.clone(),
            is_living: true,
            living_override: None,
            is_private: false,
            primary_name: PersonNameDto {
                given_name: format!("{initial}."),
                patronymic: None,
                surname: String::new(),
            },
            birth_surname: None,
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            biography: None,
            birth: None,
            death: None,
            avatar_url: None,
            avatar_media_id: None,
            avatar_crop: None,
            notes: None,
            birth_sort: None,
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: person.version,
            tx_id: None,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DismissedWarningDto {
    pub id: String,
    pub rule_code: String,
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dismissed_by: Option<String>,
    pub dismissed_at: String,
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
    pub snapshots: Vec<TreeSnapshotDto>,
    pub dismissed_warnings: Vec<DismissedWarningDto>,
    pub not_duplicate_pairs: Vec<(String, String)>,
    pub gid_aliases: HashMap<String, String>,
    pub next_gid_seq: u32,
    pub next_family_gid_seq: u32,
    pub next_event_gid_seq: u32,
    pub next_place_gid_seq: u32,
    pub next_media_gid_seq: u32,
    pub is_demo_seed: bool,
}

impl GenealogyPrivacyStore {
    pub fn empty() -> Self {
        Self {
            persons: vec![],
            families: vec![],
            events: vec![],
            notes: vec![],
            media: vec![],
            places: vec![],
            change_log: vec![],
            snapshots: vec![],
            dismissed_warnings: vec![],
            not_duplicate_pairs: vec![],
            gid_aliases: HashMap::new(),
            next_gid_seq: 100,
            next_family_gid_seq: 100,
            next_event_gid_seq: 100,
            next_place_gid_seq: 100,
            next_media_gid_seq: 100,
            is_demo_seed: false,
        }
    }

    /// Resolves a person UUID or GID (including merged losing GIDs via `gid_aliases`) to the canonical person UUID.
    pub fn resolve_person_id(&self, id_or_gid: &str) -> Option<String> {
        let trimmed = id_or_gid.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(p) = self
            .persons
            .iter()
            .find(|p| p.id == trimmed || p.gid.eq_ignore_ascii_case(trimmed))
        {
            return Some(p.id.clone());
        }
        for (alias_gid, target_id) in &self.gid_aliases {
            if alias_gid.eq_ignore_ascii_case(trimmed)
                && self.persons.iter().any(|p| p.id == *target_id)
            {
                return Some(target_id.clone());
            }
        }
        None
    }

    pub fn demo() -> Self {
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
            avatar_media_id: Some("00000000-0000-0000-0000-000000000101".to_string()),
            avatar_crop: None,
            notes: Some("Archival engineer and family chronicler.".to_string()),
            birth_sort: Some(19120319),
            death_sort: Some(19881104),
            has_death_event: true,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
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
            avatar_media_id: Some("00000000-0000-0000-0000-000000000102".to_string()),
            avatar_crop: None,
            notes: Some("Graduated from Saint Petersburg State University.".to_string()),
            birth_sort: Some(19940614),
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
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
            avatar_media_id: Some("00000000-0000-0000-0000-000000000103".to_string()),
            avatar_crop: None,
            notes: Some("Confidential adoption research note.".to_string()),
            birth_sort: Some(19891102),
            death_sort: None,
            has_death_event: false,
            alt_names: vec![],
            external_ids: HashMap::new(),
            version: 1,
            tx_id: None,
        };

        let deceased_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000101".to_string(),
            gid: "M0001".to_string(),
            title: "Mikhail Levshits Archival Portrait (1955)".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000101/file".to_string(),
            file_name: Some("m0001.webp".to_string()),
            original_file_name: Some("mikhail_1955.webp".to_string()),
            file_size_bytes: Some(34),
            sha256: None,
            width: Some(800),
            height: Some(1000),
            description: Some("Archival portrait taken in Leningrad, 1955.".to_string()),
            date_taken: Some("1955".to_string()),
            date_sort: Some(19550000),
            place_id: Some("00000000-0000-0000-0000-000000000201".to_string()),
            place_full_name: Some("Saint Petersburg, Russia".to_string()),
            is_missing: false,
            external_path: None,
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
                    is_primary_avatar: true,
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
                    is_primary_avatar: false,
                },
            ],
            links: vec![],
            version: 1,
        };

        let living_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000102".to_string(),
            gid: "M0002".to_string(),
            title: "Elena Levshits Portrait (2024)".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000102/file".to_string(),
            file_name: Some("m0002.webp".to_string()),
            original_file_name: Some("elena_2024.webp".to_string()),
            file_size_bytes: Some(34),
            sha256: None,
            width: Some(800),
            height: Some(1000),
            description: None,
            date_taken: Some("2024".to_string()),
            date_sort: Some(20240000),
            place_id: None,
            place_full_name: None,
            is_missing: false,
            external_path: None,
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
                is_primary_avatar: true,
            }],
            links: vec![],
            version: 1,
        };

        let private_media = MediaAssetSummaryDto {
            id: "00000000-0000-0000-0000-000000000103".to_string(),
            gid: "M0003".to_string(),
            title: "Confidential Archival Certificate".to_string(),
            mime_type: "image/webp".to_string(),
            file_url: "/api/v1/media/00000000-0000-0000-0000-000000000103/file".to_string(),
            file_name: Some("m0003.webp".to_string()),
            original_file_name: Some("certificate.webp".to_string()),
            file_size_bytes: Some(34),
            sha256: None,
            width: None,
            height: None,
            description: None,
            date_taken: None,
            date_sort: None,
            place_id: None,
            place_full_name: None,
            is_missing: false,
            external_path: None,
            is_private: true,
            only_living_persons: true,
            linked_person_ids: vec!["00000000-0000-0000-0000-000000000099".to_string()],
            face_tags: vec![],
            links: vec![],
            version: 1,
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
            snapshots: vec![],
            dismissed_warnings: vec![],
            not_duplicate_pairs: vec![],
            gid_aliases: HashMap::new(),
            next_gid_seq: 100,
            next_family_gid_seq: 100,
            next_event_gid_seq: 100,
            next_place_gid_seq: 100,
            next_media_gid_seq: 100,
            is_demo_seed: true,
        }
    }
}

impl Default for GenealogyPrivacyStore {
    fn default() -> Self {
        Self::empty()
    }
}
