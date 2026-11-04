#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone)]
pub(crate) struct ParsedClausePerson {
    pub(crate) relation: String,

    pub(crate) sex: String,

    pub(crate) given_name: String,

    pub(crate) patronymic: Option<String>,

    pub(crate) surname: String,

    pub(crate) maiden_name: Option<String>,

    pub(crate) birth_date_text: Option<String>,

    pub(crate) birth_year: Option<i32>,

    pub(crate) birth_place: Option<String>,

    pub(crate) death_date_text: Option<String>,

    pub(crate) death_year: Option<i32>,

    pub(crate) death_place: Option<String>,
}

pub(crate) fn split_narrative_into_clauses(raw_text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = raw_text.chars().collect();
    let mut idx = 0;

    while idx < chars.len() {
        let ch = chars[idx];
        if ch == '!' || ch == '?' || ch == ';' || ch == '\n' {
            if !current.trim().is_empty() {
                sentences.push(current.trim().to_string());
            }
            current.clear();
            idx += 1;
            continue;
        }
        if ch == '.' {
            let prev_is_digit = idx > 0 && chars[idx - 1].is_ascii_digit();
            let next_is_digit = idx + 1 < chars.len() && chars[idx + 1].is_ascii_digit();
            if !prev_is_digit || !next_is_digit {
                if !current.trim().is_empty() {
                    sentences.push(current.trim().to_string());
                }
                current.clear();
                idx += 1;
                continue;
            }
        }
        current.push(ch);
        idx += 1;
    }
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    let mut clauses = Vec::new();
    for sentence in sentences {
        let sub_parts: Vec<&str> = sentence.split(',').collect();
        let mut accum = String::new();
        for part in sub_parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !accum.is_empty() && starts_new_person_clause(trimmed) {
                clauses.push(accum.trim().to_string());
                accum = trimmed.to_string();
            } else {
                if !accum.is_empty() {
                    accum.push_str(", ");
                }
                accum.push_str(trimmed);
            }
        }
        if !accum.trim().is_empty() {
            clauses.push(accum.trim().to_string());
        }
    }

    clauses
}

pub(crate) fn starts_new_person_clause(segment: &str) -> bool {
    let lower = segment.to_lowercase();
    let stripped = lower
        .trim_start_matches("and ")
        .trim_start_matches("а ")
        .trim_start_matches("и ")
        .trim_start();

    let intro_prefixes = [
        "my ",
        "i am ",
        "i'm ",
        "my name is ",
        "we have ",
        "our ",
        "his ",
        "her ",
        "мой ",
        "моя ",
        "мои ",
        "моего ",
        "моей ",
        "у меня ",
        "у нас ",
        "наш ",
        "наша ",
        "я ",
        "меня зовут ",
        "его ",
        "её ",
        "ее ",
    ];
    intro_prefixes.iter().any(|p| stripped.starts_with(p))
}

pub(crate) fn classify_and_extract_clause(clause: &str) -> Option<ParsedClausePerson> {
    let lower = clause.to_lowercase();

    let (birth_date_text, birth_year) = extract_birth_date_and_year(clause);
    let birth_place = extract_place_from_clause(clause);
    let (death_date_text, death_year) = extract_death_date_and_year(clause);
    let maiden_name = extract_maiden_name(clause);

    if let Some((relation, default_sex)) = detect_relationship_role(&lower) {
        let name_fragment = extract_name_fragment_after_role(clause);
        let (given_name, patronymic, surname, inferred_sex) =
            parse_person_name_tokens(&name_fragment, &default_sex);

        return Some(ParsedClausePerson {
            relation,
            sex: inferred_sex,
            given_name,
            patronymic,
            surname,
            maiden_name,
            birth_date_text,
            birth_year,
            birth_place,
            death_date_text,
            death_year,
            death_place: None,
        });
    }

    if let Some(name_fragment) = extract_speaker_intro_fragment(clause, &lower) {
        let (given_name, patronymic, surname, inferred_sex) =
            parse_person_name_tokens(&name_fragment, "U");
        let sex = if inferred_sex != "U" {
            inferred_sex
        } else if lower.contains("родилась") {
            "F".to_string()
        } else if lower.contains("родился") {
            "M".to_string()
        } else {
            "U".to_string()
        };

        return Some(ParsedClausePerson {
            relation: "self".to_string(),
            sex,
            given_name,
            patronymic,
            surname,
            maiden_name,
            birth_date_text,
            birth_year,
            birth_place,
            death_date_text,
            death_year,
            death_place: None,
        });
    }

    if birth_date_text.is_some()
        || birth_year.is_some()
        || birth_place.is_some()
        || death_date_text.is_some()
        || death_year.is_some()
    {
        return Some(ParsedClausePerson {
            relation: "continuation_vitals".to_string(),
            sex: "U".to_string(),
            given_name: String::new(),
            patronymic: None,
            surname: String::new(),
            maiden_name,
            birth_date_text,
            birth_year,
            birth_place,
            death_date_text,
            death_year,
            death_place: None,
        });
    }

    None
}

pub(crate) fn detect_relationship_role(lower: &str) -> Option<(String, String)> {
    let s = lower
        .trim_start_matches("actually ")
        .trim_start_matches("also ")
        .trim_start_matches("and ")
        .trim_start_matches("а ")
        .trim_start_matches("и ")
        .trim_start();

    if s.starts_with("her father")
        || s.starts_with("её отец")
        || s.starts_with("ее отец")
        || s.starts_with("её папа")
        || s.starts_with("ее папа")
        || s.contains("maternal grandfather")
    {
        return Some(("maternal_grandfather".to_string(), "M".to_string()));
    }
    if s.starts_with("her mother")
        || s.starts_with("её мать")
        || s.starts_with("ее мать")
        || s.starts_with("её мама")
        || s.starts_with("ее мама")
        || s.contains("maternal grandmother")
    {
        return Some(("maternal_grandmother".to_string(), "F".to_string()));
    }
    if s.starts_with("his father")
        || s.starts_with("его отец")
        || s.starts_with("его папа")
        || s.contains("paternal grandfather")
    {
        return Some(("paternal_grandfather".to_string(), "M".to_string()));
    }
    if s.starts_with("his mother")
        || s.starts_with("его мать")
        || s.starts_with("его мама")
        || s.contains("paternal grandmother")
    {
        return Some(("paternal_grandmother".to_string(), "F".to_string()));
    }

    if s.contains("my grandfather")
        || s.contains("my grandpa")
        || s.contains("мой дедушка")
        || s.contains("мой дед")
    {
        return Some(("grandfather".to_string(), "M".to_string()));
    }
    if s.contains("my grandmother") || s.contains("my grandma") || s.contains("моя бабушка")
    {
        return Some(("grandmother".to_string(), "F".to_string()));
    }

    if s.contains("my father")
        || s.contains("my dad")
        || s.contains("мой отец")
        || s.contains("мой папа")
        || s.contains("моего папу")
        || s.contains("моего отца")
    {
        return Some(("father".to_string(), "M".to_string()));
    }
    if s.contains("my mother")
        || s.contains("my mom")
        || s.contains("my mum")
        || s.contains("моя мама")
        || s.contains("моя мать")
        || s.contains("мою маму")
    {
        return Some(("mother".to_string(), "F".to_string()));
    }

    if s.contains("my bf ")
        || s.ends_with("my bf")
        || s.contains("my boyfriend")
        || s.contains("my husband")
        || s.contains("мой парень")
        || s.contains("мой муж")
        || s.contains("мой супруг")
    {
        return Some(("partner".to_string(), "M".to_string()));
    }
    if s.contains("my gf ")
        || s.ends_with("my gf")
        || s.contains("my girlfriend")
        || s.contains("my wife")
        || s.contains("моя девушка")
        || s.contains("моя жена")
        || s.contains("моя супруга")
    {
        return Some(("partner".to_string(), "F".to_string()));
    }
    if s.contains("my partner") || s.contains("my spouse") {
        return Some(("partner".to_string(), "U".to_string()));
    }

    if s.contains("my brother")
        || s.contains("have a brother")
        || s.contains("мой брат")
        || s.contains("есть брат")
    {
        return Some(("brother".to_string(), "M".to_string()));
    }
    if s.contains("my sister")
        || s.contains("have a sister")
        || s.contains("моя сестра")
        || s.contains("есть сестра")
    {
        return Some(("sister".to_string(), "F".to_string()));
    }

    if s.contains("my son")
        || s.contains("our son")
        || s.contains("have a son")
        || s.contains("мой сын")
        || s.contains("наш сын")
        || s.contains("есть сын")
    {
        return Some(("son".to_string(), "M".to_string()));
    }
    if s.contains("my daughter")
        || s.contains("our daughter")
        || s.contains("have a daughter")
        || s.contains("моя дочь")
        || s.contains("моя дочка")
        || s.contains("наша дочь")
        || s.contains("наша дочка")
        || s.contains("есть дочка")
        || s.contains("есть дочь")
    {
        return Some(("daughter".to_string(), "F".to_string()));
    }
    if s.contains("our child")
        || s.contains("my child")
        || s.contains("наш ребёнок")
        || s.contains("наш ребенок")
    {
        return Some(("son".to_string(), "U".to_string()));
    }

    None
}

pub(crate) fn extract_speaker_intro_fragment(clause: &str, lower: &str) -> Option<String> {
    let prefixes = [
        "my name is ",
        "i am ",
        "i'm ",
        "меня зовут ",
        "я — ",
        "я - ",
        "я ",
    ];
    for prefix in prefixes {
        if let Some(pos) = lower.find(prefix) {
            if pos <= 4 {
                let rest = &clause[pos + prefix.len()..];
                return Some(truncate_at_vitals_or_parens(rest));
            }
        }
    }
    None
}

pub(crate) fn extract_name_fragment_after_role(clause: &str) -> String {
    let without_parens = strip_parentheticals(clause);
    let clean = without_parens.trim();

    for sep in [" — ", " – ", " - "] {
        if let Some(pos) = clean.find(sep) {
            let after = &clean[pos + sep.len()..];
            return truncate_at_vitals_or_parens(after);
        }
    }

    let lower = clean.to_lowercase();
    let markers = [
        " is named ",
        " was named ",
        " named ",
        " is ",
        " was ",
        " зовут ",
        " есть дочка ",
        " есть дочь ",
        " есть сын ",
        " есть брат ",
        " есть сестра ",
        " my sister ",
        " my brother ",
        " my father ",
        " my dad ",
        " my mother ",
        " my mom ",
        " my husband ",
        " my wife ",
        " my boyfriend ",
        " my girlfriend ",
        " my partner ",
        " my son ",
        " my daughter ",
        " my grandfather ",
        " my grandmother ",
        " a sister ",
        " a brother ",
        " a son ",
        " a daughter ",
        " папа ",
        " мама ",
        " отец ",
        " мать ",
        " муж ",
        " жена ",
        " парень ",
        " девушка ",
        " брат ",
        " сестра ",
        " сын ",
        " дочка ",
        " дочь ",
        " дедушка ",
        " бабушка ",
    ];

    for marker in markers {
        if let Some(pos) = lower.find(marker) {
            let after = &clean[pos + marker.len()..];
            let candidate = truncate_at_vitals_or_parens(after);
            if !candidate.is_empty() {
                return candidate;
            }
        }
    }

    truncate_at_vitals_or_parens(clean)
}

pub(crate) fn truncate_at_vitals_or_parens(s: &str) -> String {
    let without_parens = strip_parentheticals(s);
    let first_segment = without_parens.split(',').next().unwrap_or(&without_parens);

    let stop_words = [
        "born",
        "who",
        "was",
        "in",
        "on",
        "родился",
        "родилась",
        "умер",
        "умерла",
        "в",
        "из",
    ];
    let mut tokens = Vec::new();
    for tok in first_segment.split_whitespace() {
        let cleaned = tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if cleaned.is_empty() {
            continue;
        }
        if stop_words.contains(&cleaned.to_lowercase().as_str()) {
            break;
        }
        if cleaned.chars().all(|c| c.is_ascii_digit()) {
            break;
        }
        tokens.push(cleaned.to_string());
    }
    tokens.join(" ")
}

pub(crate) fn strip_parentheticals(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0usize;
    for ch in s.chars() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 {
            out.push(ch);
        }
    }
    out
}

pub(crate) fn extract_maiden_name(clause: &str) -> Option<String> {
    let lower = clause.to_lowercase();
    for marker in [
        "в девичестве ",
        "урождённая ",
        "урожденная ",
        "née ",
        "nee ",
    ] {
        if let Some(pos) = lower.find(marker) {
            let after = &clause[pos + marker.len()..];
            let word = after
                .split(|c: char| !c.is_alphanumeric() && c != '-')
                .find(|w| !w.is_empty())?;
            return Some(word.to_string());
        }
    }
    None
}
