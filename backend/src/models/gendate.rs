use chrono::{Datelike, Days, NaiveDate};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GenDateMod {
    Exact,
    About,
    Estimated,
    Calculated,
    Before,
    After,
    Between,
    FromTo,
    Phrase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalendarType {
    #[default]
    Gregorian,
    Julian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PartialDate {
    pub y: i32,
    pub m: Option<u8>,
    pub d: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GenDate {
    #[serde(rename = "mod")]
    pub r#mod: GenDateMod,
    pub start: Option<PartialDate>,
    pub end: Option<PartialDate>,
    #[serde(default)]
    pub cal: CalendarType,
    pub text: String,
}

impl GenDate {
    /// Computes the integer sort key (`YYYYMMDD`) used for chronological ordering
    /// and the living-status calculation (`docs/features/10-core-data-model.md §3.2`).
    pub fn compute_sort_key(&self) -> Option<i32> {
        if self.r#mod == GenDateMod::Phrase {
            return None;
        }
        let start = self.start.as_ref()?;

        let (y, m, d) = match (self.cal, start.m, start.d) {
            (CalendarType::Julian, Some(m), Some(d)) => {
                let offset = julian_to_gregorian_offset_days(start.y);
                if let Some(naive) = NaiveDate::from_ymd_opt(start.y, u32::from(m), u32::from(d))
                    .and_then(|dt| dt.checked_add_days(Days::new(offset)))
                {
                    (naive.year(), naive.month() as i32, naive.day() as i32)
                } else {
                    (start.y, i32::from(m), i32::from(d))
                }
            }
            _ => (
                start.y,
                i32::from(start.m.unwrap_or(0)),
                i32::from(start.d.unwrap_or(0)),
            ),
        };

        let base = y * 10_000 + m * 100 + d;

        Some(match self.r#mod {
            GenDateMod::Before => base - 1,
            GenDateMod::After => base + 1,
            _ => base,
        })
    }

    /// Bilingual (EN / RU) flexible genealogy date parser (`00 §2.4`, `07 §2.2`, `10 §3.2`).
    /// Unparseable input never errors; it produces a `GenDateMod::Phrase` preserving `text`.
    pub fn parse(input: &str) -> Self {
        let trimmed = input.trim();
        let (working, cal) = extract_calendar(trimmed);
        let working = working.trim();

        if let Some((r#mod, start_str, end_str)) = split_range(working) {
            if let (Some(start), Some(end)) =
                (parse_partial_date(start_str), parse_partial_date(end_str))
            {
                return Self {
                    r#mod,
                    start: Some(start),
                    end: Some(end),
                    cal,
                    text: trimmed.to_string(),
                };
            }
        }

        let (r#mod, rest) = extract_modifier(working);
        if let Some(start) = parse_partial_date(rest) {
            return Self {
                r#mod,
                start: Some(start),
                end: None,
                cal,
                text: trimmed.to_string(),
            };
        }

        Self {
            r#mod: GenDateMod::Phrase,
            start: None,
            end: None,
            cal: CalendarType::Gregorian,
            text: trimmed.to_string(),
        }
    }
}

fn julian_to_gregorian_offset_days(year: i32) -> u64 {
    match year {
        1900..=2099 => 13,
        1800..=1899 => 12,
        1700..=1799 => 11,
        1582..=1699 => 10,
        _ => {
            let century = year.div_euclid(100);
            (century - century.div_euclid(4) - 2).max(0) as u64
        }
    }
}

fn extract_calendar(s: &str) -> (&str, CalendarType) {
    let lower = s.to_lowercase();
    let julian_markers = [
        "(ст. ст.)",
        "(ст.ст.)",
        "ст. ст.",
        "ст.ст.",
        "(julian)",
        "(o.s.)",
    ];
    for marker in julian_markers {
        if lower.ends_with(marker) {
            let new_len = s.len() - marker.len();
            return (&s[..new_len], CalendarType::Julian);
        }
    }

    let gregorian_markers = [
        "(нов. ст.)",
        "(нов.ст.)",
        "нов. ст.",
        "нов.ст.",
        "(gregorian)",
        "(n.s.)",
    ];
    for marker in gregorian_markers {
        if lower.ends_with(marker) {
            let new_len = s.len() - marker.len();
            return (&s[..new_len], CalendarType::Gregorian);
        }
    }

    (s, CalendarType::Gregorian)
}

fn split_range(s: &str) -> Option<(GenDateMod, &str, &str)> {
    let lower = s.to_lowercase();

    for prefix in ["between ", "bet ", "bet. "] {
        if let Some(rest_lower) = lower.strip_prefix(prefix) {
            let rest = &s[prefix.len()..];
            if let Some(idx) = rest_lower.find(" and ") {
                return Some((GenDateMod::Between, &rest[..idx], &rest[idx + 5..]));
            }
        }
    }

    let ru_between = "между ";
    if let Some(rest_lower) = lower.strip_prefix(ru_between) {
        let rest = &s[ru_between.len()..];
        if let Some(idx) = rest_lower.find(" и ") {
            return Some((
                GenDateMod::Between,
                &rest[..idx],
                &rest[idx + " и ".len()..],
            ));
        }
    }

    if let Some(rest_lower) = lower.strip_prefix("from ") {
        let rest = &s["from ".len()..];
        if let Some(idx) = rest_lower.find(" to ") {
            return Some((GenDateMod::FromTo, &rest[..idx], &rest[idx + 4..]));
        }
    }

    if let Some(rest_lower) = lower.strip_prefix("с ") {
        let rest = &s["с ".len()..];
        for sep in [" по ", " до "] {
            if let Some(idx) = rest_lower.find(sep) {
                return Some((GenDateMod::FromTo, &rest[..idx], &rest[idx + sep.len()..]));
            }
        }
    }

    None
}

fn extract_modifier(s: &str) -> (GenDateMod, &str) {
    let lower = s.to_lowercase();
    let prefixes: &[(&str, GenDateMod)] = &[
        ("about ", GenDateMod::About),
        ("abt. ", GenDateMod::About),
        ("abt ", GenDateMod::About),
        ("circa ", GenDateMod::About),
        ("ca. ", GenDateMod::About),
        ("около ", GenDateMod::About),
        ("ок. ", GenDateMod::About),
        ("ок ", GenDateMod::About),
        ("estimated ", GenDateMod::Estimated),
        ("est. ", GenDateMod::Estimated),
        ("est ", GenDateMod::Estimated),
        ("приблизительно ", GenDateMod::Estimated),
        ("прибл. ", GenDateMod::Estimated),
        ("оценочно ", GenDateMod::Estimated),
        ("calculated ", GenDateMod::Calculated),
        ("cal. ", GenDateMod::Calculated),
        ("cal ", GenDateMod::Calculated),
        ("вычислено ", GenDateMod::Calculated),
        ("выч. ", GenDateMod::Calculated),
        ("before ", GenDateMod::Before),
        ("bef. ", GenDateMod::Before),
        ("bef ", GenDateMod::Before),
        ("до ", GenDateMod::Before),
        ("after ", GenDateMod::After),
        ("aft. ", GenDateMod::After),
        ("aft ", GenDateMod::After),
        ("после ", GenDateMod::After),
    ];

    for (prefix, modifier) in prefixes {
        if lower.starts_with(prefix) {
            return (*modifier, s[prefix.len()..].trim());
        }
    }

    (GenDateMod::Exact, s)
}

fn strip_year_suffix(s: &str) -> &str {
    let mut trimmed = s.trim();
    let lower = trimmed.to_lowercase();
    for suffix in [" гг.", " гг", " г.", " г", " года", " год"] {
        if lower.ends_with(suffix) {
            trimmed = trimmed[..trimmed.len() - suffix.len()].trim();
            break;
        }
    }
    trimmed
}

fn parse_partial_date(raw: &str) -> Option<PartialDate> {
    let cleaned = strip_year_suffix(raw);
    if cleaned.is_empty() {
        return None;
    }

    // 1. DD.MM.YYYY or MM.YYYY
    if cleaned.contains('.') && cleaned.chars().all(|c| c.is_ascii_digit() || c == '.') {
        let parts: Vec<&str> = cleaned.split('.').collect();
        match parts.as_slice() {
            [d_str, m_str, y_str] => {
                let d: u8 = d_str.parse().ok()?;
                let m: u8 = m_str.parse().ok()?;
                let y: i32 = y_str.parse().ok()?;
                if (1..=12).contains(&m) && (1..=31).contains(&d) && (100..=9999).contains(&y) {
                    return Some(PartialDate {
                        y,
                        m: Some(m),
                        d: Some(d),
                    });
                }
                return None;
            }
            [m_str, y_str] => {
                let m: u8 = m_str.parse().ok()?;
                let y: i32 = y_str.parse().ok()?;
                if (1..=12).contains(&m) && (100..=9999).contains(&y) {
                    return Some(PartialDate {
                        y,
                        m: Some(m),
                        d: None,
                    });
                }
                return None;
            }
            _ => return None,
        }
    }

    // 2. ISO YYYY-MM-DD or YYYY-MM
    if cleaned.contains('-') && cleaned.chars().all(|c| c.is_ascii_digit() || c == '-') {
        let parts: Vec<&str> = cleaned.split('-').collect();
        match parts.as_slice() {
            [y_str, m_str, d_str] => {
                let y: i32 = y_str.parse().ok()?;
                let m: u8 = m_str.parse().ok()?;
                let d: u8 = d_str.parse().ok()?;
                if (1..=12).contains(&m) && (1..=31).contains(&d) && (100..=9999).contains(&y) {
                    return Some(PartialDate {
                        y,
                        m: Some(m),
                        d: Some(d),
                    });
                }
                return None;
            }
            [y_str, m_str] => {
                let y: i32 = y_str.parse().ok()?;
                let m: u8 = m_str.parse().ok()?;
                if (1..=12).contains(&m) && (100..=9999).contains(&y) {
                    return Some(PartialDate {
                        y,
                        m: Some(m),
                        d: None,
                    });
                }
                return None;
            }
            _ => return None,
        }
    }

    // 3. Tokenized forms: `YYYY`, `<Month> <YYYY>`, `<D> <Month> <YYYY>`, `<Month> <D>, <YYYY>`
    let tokens: Vec<&str> = cleaned
        .split_whitespace()
        .map(|t| t.trim_matches(','))
        .filter(|t| !t.is_empty())
        .collect();

    match tokens.as_slice() {
        [y_str] => {
            let y: i32 = y_str.parse().ok()?;
            if (100..=9999).contains(&y) {
                Some(PartialDate {
                    y,
                    m: None,
                    d: None,
                })
            } else {
                None
            }
        }
        [m_str, y_str] => {
            let m = parse_month_name(m_str)?;
            let y: i32 = y_str.parse().ok()?;
            if (100..=9999).contains(&y) {
                Some(PartialDate {
                    y,
                    m: Some(m),
                    d: None,
                })
            } else {
                None
            }
        }
        [first, second, y_str] => {
            let y: i32 = y_str.parse().ok()?;
            if !(100..=9999).contains(&y) {
                return None;
            }
            if let (Ok(d), Some(m)) = (first.parse::<u8>(), parse_month_name(second)) {
                if (1..=31).contains(&d) {
                    return Some(PartialDate {
                        y,
                        m: Some(m),
                        d: Some(d),
                    });
                }
            }
            if let (Some(m), Ok(d)) = (parse_month_name(first), second.parse::<u8>()) {
                if (1..=31).contains(&d) {
                    return Some(PartialDate {
                        y,
                        m: Some(m),
                        d: Some(d),
                    });
                }
            }
            None
        }
        _ => None,
    }
}

fn parse_month_name(token: &str) -> Option<u8> {
    let lower = token.trim_end_matches('.').to_lowercase();
    match lower.as_str() {
        "jan" | "january" | "янв" | "январь" | "января" | "январе" | "январем" | "январём" => {
            Some(1)
        }
        "feb" | "february" | "фев" | "февраль" | "февраля" | "феврале" | "февралем"
        | "февралём" => Some(2),
        "mar" | "march" | "мар" | "март" | "марта" | "марте" | "мартом" => {
            Some(3)
        }
        "apr" | "april" | "апр" | "апрель" | "апреля" | "апреле" | "апрелем" => {
            Some(4)
        }
        "may" | "май" | "мая" | "мае" | "маем" => Some(5),
        "jun" | "june" | "июн" | "июнь" | "июня" | "июне" | "июнем" => Some(6),
        "jul" | "july" | "июл" | "июль" | "июля" | "июле" | "июлем" => Some(7),
        "aug" | "august" | "авг" | "август" | "августа" | "августе" | "августом" => {
            Some(8)
        }
        "sep" | "sept" | "september" | "сен" | "сент" | "сентябрь" | "сентября" | "сентябре"
        | "сентябрем" | "сентябрём" => Some(9),
        "oct" | "october" | "окт" | "октябрь" | "октября" | "октябре" | "октябрем" | "октябрём" => {
            Some(10)
        }
        "nov" | "november" | "ноя" | "нояб" | "ноябрь" | "ноября" | "ноябре" | "ноябрем"
        | "ноябрём" => Some(11),
        "dec" | "december" | "дек" | "декабрь" | "декабря" | "декабре" | "декабрем"
        | "декабрём" => Some(12),
        _ => None,
    }
}
