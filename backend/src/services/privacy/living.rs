#[allow(unused_imports)]
use super::*;

pub(crate) fn default_version() -> i32 {
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
