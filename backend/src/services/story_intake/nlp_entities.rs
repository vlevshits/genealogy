#[allow(unused_imports)]
use super::*;

pub(crate) fn parse_person_name_tokens(
    fragment: &str,
    default_sex: &str,
) -> (String, Option<String>, String, String) {
    let tokens: Vec<&str> = fragment
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
        .filter(|t| !t.is_empty())
        .collect();

    if tokens.is_empty() {
        return (String::new(), None, String::new(), default_sex.to_string());
    }

    if tokens.len() == 1 {
        let given = tokens[0].to_string();
        let sex = if default_sex != "U" {
            default_sex.to_string()
        } else {
            infer_sex_from_names(&given, None, "")
        };
        return (given, None, String::new(), sex);
    }

    if tokens.len() == 2 {
        if is_russian_patronymic(tokens[1]) {
            let given = tokens[0].to_string();
            let pat = tokens[1].to_string();
            let sex = if default_sex != "U" {
                default_sex.to_string()
            } else {
                infer_sex_from_names(&given, Some(&pat), "")
            };
            return (given, Some(pat), String::new(), sex);
        }
        let given = tokens[0].to_string();
        let surname = tokens[1].to_string();
        let sex = if default_sex != "U" {
            default_sex.to_string()
        } else {
            infer_sex_from_names(&given, None, &surname)
        };
        return (given, None, surname, sex);
    }

    if is_russian_patronymic(tokens[1]) {
        let given = tokens[0].to_string();
        let pat = tokens[1].to_string();
        let surname = tokens[2..].join(" ");
        let sex = if default_sex != "U" {
            default_sex.to_string()
        } else {
            infer_sex_from_names(&given, Some(&pat), &surname)
        };
        return (given, Some(pat), surname, sex);
    }

    if tokens.len() >= 3 && is_russian_patronymic(tokens[2]) {
        let surname = tokens[0].to_string();
        let given = tokens[1].to_string();
        let pat = tokens[2].to_string();
        let sex = if default_sex != "U" {
            default_sex.to_string()
        } else {
            infer_sex_from_names(&given, Some(&pat), &surname)
        };
        return (given, Some(pat), surname, sex);
    }

    let given = tokens[0].to_string();
    let pat = Some(tokens[1].to_string());
    let surname = tokens[2..].join(" ");
    let sex = if default_sex != "U" {
        default_sex.to_string()
    } else {
        infer_sex_from_names(&given, pat.as_deref(), &surname)
    };
    (given, pat, surname, sex)
}

pub(crate) fn is_russian_patronymic(word: &str) -> bool {
    let lower = word.to_lowercase();
    let suffixes = [
        "ович",
        "евич",
        "ич",
        "овна",
        "евна",
        "ична",
        "инична",
        "ovich",
        "evich",
        "ovna",
        "evna",
    ];
    suffixes.iter().any(|s| lower.ends_with(s))
}

pub(crate) fn infer_sex_from_names(given: &str, patronymic: Option<&str>, surname: &str) -> String {
    if let Some(pat) = patronymic {
        let pl = pat.to_lowercase();
        if pl.ends_with("ович")
            || pl.ends_with("евич")
            || pl.ends_with("ич")
            || pl.ends_with("ovich")
            || pl.ends_with("evich")
        {
            return "M".to_string();
        }
        if pl.ends_with("овна")
            || pl.ends_with("евна")
            || pl.ends_with("ична")
            || pl.ends_with("ovna")
            || pl.ends_with("evna")
        {
            return "F".to_string();
        }
    }

    let sl = surname.to_lowercase();
    if sl.ends_with("ова")
        || sl.ends_with("ева")
        || sl.ends_with("ина")
        || sl.ends_with("ская")
        || sl.ends_with("ova")
        || sl.ends_with("eva")
        || sl.ends_with("ina")
    {
        return "F".to_string();
    }
    if sl.ends_with("ов")
        || sl.ends_with("ев")
        || sl.ends_with("ин")
        || sl.ends_with("ский")
        || sl.ends_with("ov")
        || sl.ends_with("ev")
    {
        return "M".to_string();
    }

    let gl = given.to_lowercase();
    if matches!(
        gl.as_str(),
        "анна"
            | "елена"
            | "мария"
            | "ольга"
            | "софия"
            | "наталья"
            | "татьяна"
            | "екатерина"
            | "anna"
            | "elena"
            | "maria"
            | "olga"
            | "sofia"
            | "sarah"
    ) {
        return "F".to_string();
    }
    if matches!(
        gl.as_str(),
        "иван"
            | "сергей"
            | "дмитрий"
            | "виктор"
            | "алексей"
            | "александр"
            | "михаил"
            | "николай"
            | "ivan"
            | "sergey"
            | "dmitry"
            | "viktor"
            | "alexey"
            | "mikhail"
            | "john"
    ) {
        return "M".to_string();
    }

    "U".to_string()
}

pub(crate) fn adjust_russian_surname_gender(base_surname: &str, target_sex: &str) -> String {
    let s = base_surname.trim();
    if target_sex == "F" {
        if s.ends_with("ов") || s.ends_with("ев") || s.ends_with("ин") {
            return format!("{s}а");
        }
        if let Some(stem) = s.strip_suffix("ский") {
            return format!("{stem}ская");
        }
    } else if target_sex == "M" {
        if let Some(stem) = s.strip_suffix("ова") {
            return format!("{stem}ов");
        }
        if let Some(stem) = s.strip_suffix("ева") {
            return format!("{stem}ев");
        }
        if let Some(stem) = s.strip_suffix("ина") {
            return format!("{stem}ин");
        }
        if let Some(stem) = s.strip_suffix("ская") {
            return format!("{stem}ский");
        }
    }
    s.to_string()
}

pub(crate) fn extract_birth_date_and_year(clause: &str) -> (Option<String>, Option<i32>) {
    let lower = clause.to_lowercase();
    if (lower.contains("died") || lower.contains("умер") || lower.contains("умерла"))
        && !lower.contains("born")
        && !lower.contains("родил")
    {
        return (None, None);
    }

    extract_date_and_year_common(clause)
}

pub(crate) fn extract_death_date_and_year(clause: &str) -> (Option<String>, Option<i32>) {
    let lower = clause.to_lowercase();
    for marker in ["died", "passed away", "умерла", "умер"] {
        if let Some(pos) = lower.find(marker) {
            let sub = &clause[pos..];
            return extract_date_and_year_common(sub);
        }
    }
    (None, None)
}

pub(crate) fn extract_date_and_year_common(text: &str) -> (Option<String>, Option<i32>) {
    for tok in text.split_whitespace() {
        let clean = tok.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-');
        let dot_parts: Vec<&str> = clean.split('.').collect();
        if dot_parts.len() == 3 {
            if let (Ok(d), Ok(m), Ok(y)) = (
                dot_parts[0].parse::<u32>(),
                dot_parts[1].parse::<u32>(),
                dot_parts[2].parse::<i32>(),
            ) {
                if (1..=31).contains(&d) && (1..=12).contains(&m) && (1600..=2100).contains(&y) {
                    let mon = month_num_to_gedcom(m);
                    return (Some(format!("{d} {mon} {y}")), Some(y));
                }
            }
        }

        let dash_parts: Vec<&str> = clean.split('-').collect();
        if dash_parts.len() == 3 {
            if let (Ok(y), Ok(m), Ok(d)) = (
                dash_parts[0].parse::<i32>(),
                dash_parts[1].parse::<u32>(),
                dash_parts[2].parse::<u32>(),
            ) {
                if (1..=31).contains(&d) && (1..=12).contains(&m) && (1600..=2100).contains(&y) {
                    let mon = month_num_to_gedcom(m);
                    return (Some(format!("{d} {mon} {y}")), Some(y));
                }
            }
        }
    }

    let mut year: Option<i32> = None;
    let mut month_gedcom: Option<&'static str> = None;
    let mut day: Option<u32> = None;

    for raw_tok in text.split(|c: char| c.is_whitespace() || c == ',') {
        let tok = raw_tok.trim_matches(|c: char| !c.is_alphanumeric());
        if tok.is_empty() {
            continue;
        }
        if let Ok(num) = tok.parse::<i32>() {
            if (1600..=2100).contains(&num) && year.is_none() {
                year = Some(num);
                continue;
            }
            if (1..=31).contains(&num) && day.is_none() {
                day = Some(num as u32);
                continue;
            }
        }
        if month_gedcom.is_none() {
            if let Some(m) = parse_month_token(tok) {
                month_gedcom = Some(m);
            }
        }
    }

    match (day, month_gedcom, year) {
        (Some(d), Some(m), Some(y)) => (Some(format!("{d} {m} {y}")), Some(y)),
        (None, Some(m), Some(y)) => (Some(format!("{m} {y}")), Some(y)),
        (_, _, Some(y)) => (Some(y.to_string()), Some(y)),
        _ => (None, None),
    }
}

pub(crate) fn parse_month_token(tok: &str) -> Option<&'static str> {
    match tok.to_lowercase().as_str() {
        "january" | "jan" | "января" | "январь" | "январе" => Some("JAN"),
        "february" | "feb" | "февраля" | "февраль" | "феврале" => Some("FEB"),
        "march" | "mar" | "марта" | "март" | "марте" => Some("MAR"),
        "april" | "apr" | "апреля" | "апрель" | "апреле" => Some("APR"),
        "may" | "мая" | "май" | "мае" => Some("MAY"),
        "june" | "jun" | "июня" | "июнь" | "июне" => Some("JUN"),
        "july" | "jul" | "июля" | "июль" | "июле" => Some("JUL"),
        "august" | "aug" | "августа" | "август" | "августе" => Some("AUG"),
        "september" | "sep" | "sept" | "сентября" | "сентябрь" | "сентябре" => {
            Some("SEP")
        }
        "october" | "oct" | "октября" | "октябрь" | "октябре" => Some("OCT"),
        "november" | "nov" | "ноября" | "ноябрь" | "ноябре" => Some("NOV"),
        "december" | "dec" | "декабря" | "декабрь" | "декабре" => Some("DEC"),
        _ => None,
    }
}

pub(crate) fn month_num_to_gedcom(m: u32) -> &'static str {
    match m {
        1 => "JAN",
        2 => "FEB",
        3 => "MAR",
        4 => "APR",
        5 => "MAY",
        6 => "JUN",
        7 => "JUL",
        8 => "AUG",
        9 => "SEP",
        10 => "OCT",
        11 => "NOV",
        12 => "DEC",
        _ => "JAN",
    }
}

pub(crate) fn extract_place_from_clause(clause: &str) -> Option<String> {
    let tokens: Vec<&str> = clause.split_whitespace().collect();
    for i in 0..tokens.len() {
        let lower = tokens[i]
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if (lower == "in" || lower == "at" || lower == "from" || lower == "в" || lower == "из")
            && i + 1 < tokens.len()
        {
            let mut next_idx = i + 1;
            let next_clean = tokens[next_idx]
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if matches!(
                next_clean.as_str(),
                "городе"
                    | "г"
                    | "селе"
                    | "деревне"
                    | "посёлке"
                    | "поселке"
                    | "city"
                    | "village"
            ) && next_idx + 1 < tokens.len()
            {
                next_idx += 1;
            }

            let candidate =
                tokens[next_idx].trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if candidate.is_empty() {
                continue;
            }
            if candidate.chars().all(|c| c.is_ascii_digit())
                || parse_month_token(candidate).is_some()
                || candidate.eq_ignore_ascii_case("девичестве")
            {
                continue;
            }
            if candidate.chars().next().is_some_and(char::is_uppercase) {
                let mut place_words = vec![normalize_russian_locative_place(candidate)];
                if next_idx + 1 < tokens.len() {
                    let second = tokens[next_idx + 1]
                        .trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                    if !second.is_empty()
                        && second.chars().next().is_some_and(char::is_uppercase)
                        && parse_month_token(second).is_none()
                        && !matches!(
                            second.to_lowercase().as_str(),
                            "my" | "мой" | "моя" | "у" | "her" | "his"
                        )
                    {
                        place_words.push(normalize_russian_locative_place(second));
                    }
                }
                return Some(place_words.join(" "));
            }
        }
    }
    None
}

pub(crate) fn normalize_russian_locative_place(word: &str) -> String {
    match word.to_lowercase().as_str() {
        "москве" => "Москва".to_string(),
        "твери" => "Тверь".to_string(),
        "самаре" => "Самара".to_string(),
        "казани" => "Казань".to_string(),
        "перми" => "Пермь".to_string(),
        "рязани" => "Рязань".to_string(),
        "туле" => "Тула".to_string(),
        "калуге" => "Калуга".to_string(),
        "вологде" => "Вологда".to_string(),
        "костроме" => "Кострома".to_string(),
        "ярославле" => "Ярославль".to_string(),
        "санкт-петербурге" | "петербурге" => "Санкт-Петербург".to_string(),
        "ленинграде" => "Ленинград".to_string(),
        "витебске" => "Витебск".to_string(),
        "минске" => "Минск".to_string(),
        "киеве" => "Киев".to_string(),
        "одессе" => "Одесса".to_string(),
        "саратове" => "Саратов".to_string(),
        "воронеже" => "Воронеж".to_string(),
        "новгороде" => "Новгород".to_string(),
        "нижнем" => "Нижний".to_string(),
        _ => word.to_string(),
    }
}

pub(crate) fn derive_story_title(raw_text: &str, extracted: &DraftExtractedDataDto) -> String {
    if let Some(speaker) = extracted.persons.first() {
        if !speaker.given_name.is_empty() && speaker.given_name != "Speaker" {
            let full = if speaker.surname.is_empty() {
                speaker.given_name.clone()
            } else {
                format!("{} {}", speaker.given_name, speaker.surname)
            };
            return if contains_cyrillic(raw_text) {
                format!("Семейная история: {full}")
            } else {
                format!("Family Story: {full}")
            };
        }
    }
    "Untitled Family Story".to_string()
}

pub fn contains_cyrillic(s: &str) -> bool {
    s.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
}

pub(crate) fn format_person_display_name(p: &TreePersonNodeDto) -> String {
    let mut parts = Vec::new();
    if !p.primary_name.given_name.trim().is_empty() {
        parts.push(p.primary_name.given_name.trim());
    }
    if let Some(pat) = &p.primary_name.patronymic {
        if !pat.trim().is_empty() {
            parts.push(pat.trim());
        }
    }
    if !p.primary_name.surname.trim().is_empty() {
        parts.push(p.primary_name.surname.trim());
    }
    parts.join(" ")
}

pub fn extract_person_birth_year(p: &TreePersonNodeDto) -> Option<i32> {
    if let Some(bs) = p.birth_sort {
        let y = bs / 10_000;
        if (1000..=2100).contains(&y) {
            return Some(y);
        }
    }
    if let Some(b) = &p.birth {
        if let Some(d_str) = &b.date {
            let (_, y) = extract_date_and_year_common(d_str);
            return y;
        }
    }
    None
}

pub(crate) fn names_align(a: &str, b: &str) -> bool {
    let na = transliterate_to_ascii_lower(a);
    let nb = transliterate_to_ascii_lower(b);
    if na.is_empty() || nb.is_empty() {
        return false;
    }
    na == nb || (na.len() >= 4 && nb.len() >= 4 && (na.starts_with(&nb) || nb.starts_with(&na)))
}

pub(crate) fn surnames_align(a: &str, b: &str) -> bool {
    let na = normalize_surname_stem(&transliterate_to_ascii_lower(a));
    let nb = normalize_surname_stem(&transliterate_to_ascii_lower(b));
    !na.is_empty() && na == nb
}

pub(crate) fn normalize_surname_stem(s: &str) -> String {
    let trimmed = s.trim();
    for suffix in ["ova", "eva", "ina", "skaya"] {
        if let Some(stem) = trimmed.strip_suffix(suffix) {
            if stem.len() >= 2 {
                return match suffix {
                    "ova" => format!("{stem}ov"),
                    "eva" => format!("{stem}ev"),
                    "ina" => format!("{stem}in"),
                    "skaya" => format!("{stem}sky"),
                    _ => trimmed.to_string(),
                };
            }
        }
    }
    trimmed.to_string()
}

pub(crate) fn places_align(a: &str, b: &str) -> bool {
    let na = canonical_place_key(a);
    let nb = canonical_place_key(b);
    !na.is_empty() && (na == nb || na.contains(&nb) || nb.contains(&na))
}

pub(crate) fn canonical_place_key(s: &str) -> String {
    let first = s.split(',').next().unwrap_or(s).trim();
    let norm = normalize_russian_locative_place(first);
    match norm.to_lowercase().as_str() {
        "москва" | "moscow" => "moscow".to_string(),
        "тверь" | "tver" => "tver".to_string(),
        "самара" | "samara" => "samara".to_string(),
        "санкт-петербург" | "петербург" | "saint petersburg" | "st petersburg" => {
            "saint petersburg".to_string()
        }
        "витебск" | "vitebsk" => "vitebsk".to_string(),
        _ => transliterate_to_ascii_lower(&norm),
    }
}

pub(crate) fn transliterate_to_ascii_lower(s: &str) -> String {
    let mut out = String::new();
    for ch in s.trim().to_lowercase().chars() {
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
            c if c.is_alphanumeric() => out.push(c),
            _ => {}
        }
    }
    out
}
