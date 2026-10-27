use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::gendate::GenDate,
    services::{
        genealogy::{
            allocate_gid, recompute_all_individuals_vitals, recompute_all_places_metadata,
            resolve_or_create_place_chain,
        },
        privacy::{
            redact_person_for_tree_view, ChangeLogEntryDto, FamilyChildDto, FamilyDto,
            GenealogyPrivacyStore, IndividualAltNameDto, LifeEventDto, MediaAssetSummaryDto,
            MediaLinkDto, NoteDto, PersonNameDto, TreePersonNodeDto,
        },
        rbac::CallerAccess,
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GedcomDialectSummary {
    #[serde(default)]
    pub source_system: String,
    #[serde(default)]
    pub source_version: Option<String>,
    #[serde(default)]
    pub gedcom_version: String,
    #[serde(default)]
    pub charset: String,
    #[serde(default)]
    pub normalized_coordinates_count: usize,
    #[serde(default)]
    pub preserved_uids_count: usize,
    #[serde(default)]
    pub preserved_rins_count: usize,
    #[serde(default)]
    pub filtered_upd_timestamps_count: usize,
    #[serde(default)]
    pub deduplicated_parental_associations_count: usize,
    #[serde(default)]
    pub synthesized_single_parent_families_count: usize,
    #[serde(default)]
    pub social_associations_notes_count: usize,
    #[serde(default)]
    pub stripped_media_paths_count: usize,
    #[serde(default)]
    pub marriage_licenses_count: usize,
    #[serde(default)]
    pub deceased_flag_only_count: usize,
    // Architect / Frontend fields (`03-gedcom-import-export.md`):
    #[serde(default)]
    pub coordinates_normalized: usize,
    #[serde(default)]
    pub uids_preserved: usize,
    #[serde(default)]
    pub vendor_upd_filtered: usize,
    #[serde(default)]
    pub married_names_resolved: usize,
    #[serde(default)]
    pub parental_asso_deduplicated: usize,
    #[serde(default)]
    pub missing_media_stubs: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GedcomAnalysisReport {
    #[serde(default)]
    pub total_individuals: usize,
    #[serde(default)]
    pub total_families: usize,
    #[serde(default)]
    pub total_events: usize,
    #[serde(default)]
    pub total_places: usize,
    #[serde(default)]
    pub total_media: usize,
    #[serde(default)]
    pub missing_media_count: usize,
    #[serde(default)]
    pub detected_software: String,
    #[serde(default)]
    pub gedcom_version: String,
    #[serde(default)]
    pub character_encoding: String,
    #[serde(default)]
    pub dialect_summary: GedcomDialectSummary,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub file_size_bytes: usize,
    #[serde(default)]
    pub individuals_count: usize,
    #[serde(default)]
    pub families_count: usize,
    #[serde(default)]
    pub media_count: usize,
    #[serde(default)]
    pub media_missing_count: usize,
    #[serde(default)]
    pub events_count: usize,
    #[serde(default)]
    pub places_count: usize,
    #[serde(default)]
    pub notes_count: usize,
    #[serde(default)]
    pub sources_count: usize,
    #[serde(default)]
    pub dialect: GedcomDialectSummary,
    #[serde(default)]
    pub can_import: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct GedcomAnalyzeRequest {
    #[serde(default)]
    pub data: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct GedcomImportRequest {
    #[serde(default)]
    pub data: String,
    pub file_name: Option<String>,
    /// `"new"`, `"replace"`, `"clean_slate"`, or `"merge"`
    pub mode: Option<String>,
    /// Required when `mode == "replace"` or `"clean_slate"` on a non-empty tree (`"CLEAN_SLATE"`, `"ЗАМЕНИТЬ"`, or `"REPLACE"`)
    pub confirm_text: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct GedcomImportResult {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub operation: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub snapshot_id: Option<String>,
    #[serde(default)]
    pub imported_individuals: usize,
    #[serde(default)]
    pub imported_families: usize,
    #[serde(default)]
    pub imported_events: usize,
    #[serde(default)]
    pub imported_places: usize,
    #[serde(default)]
    pub imported_media: usize,
    #[serde(default)]
    pub missing_media_count: usize,
    #[serde(default)]
    pub prompt_home_person: bool,
    #[serde(default)]
    pub individuals_imported: usize,
    #[serde(default)]
    pub families_imported: usize,
    #[serde(default)]
    pub events_imported: usize,
    #[serde(default)]
    pub places_imported: usize,
    #[serde(default)]
    pub media_imported: usize,
    #[serde(default)]
    pub media_missing_count: usize,
    #[serde(default)]
    pub notes_imported: usize,
    #[serde(default)]
    pub dialect: GedcomDialectSummary,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct GedcomExportRequest {
    /// `"7.0"` (default) or `"5.5.1"`
    pub version: Option<String>,
    /// `"gedcom70"` (default), `"gedcom551"`, or `"gdz"`
    pub format: Option<String>,
    pub include_media: Option<bool>,
    pub redact_living: Option<bool>,
    pub include_private_notes: Option<bool>,
    pub root_person_id: Option<String>,
    /// `"all"` (default), `"ancestors"`, `"descendants"`, or `"hourglass"`
    pub export_scope: Option<String>,
    pub generations: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct GedcomNode {
    pub level: u8,
    pub xref: Option<String>,
    pub tag: String,
    pub value: String,
    pub children: Vec<GedcomNode>,
}

impl GedcomNode {
    pub fn find_child(&self, tag: &str) -> Option<&GedcomNode> {
        self.children
            .iter()
            .find(|c| c.tag.eq_ignore_ascii_case(tag))
    }

    pub fn child_value(&self, tag: &str) -> Option<String> {
        self.find_child(tag)
            .map(|c| c.value.trim().to_string())
            .filter(|v| !v.is_empty())
    }
}

/// Detects character encoding and decodes raw GEDCOM bytes (`03 §2.1 #2, §6 DoD #2`).
/// Supports UTF-8 (with or without BOM), UTF-16LE/BE, Windows-1251, Windows-1252, and ANSEL.
pub fn decode_gedcom_bytes(bytes: &[u8]) -> (String, String) {
    if bytes.is_empty() {
        return (String::new(), "UTF-8".to_string());
    }

    // 1. UTF-8 BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let text = String::from_utf8_lossy(&bytes[3..]).into_owned();
        return (text, "UTF-8".to_string());
    }

    // 2. UTF-16LE BOM or alternating null bytes
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return (String::from_utf16_lossy(&u16s), "UNICODE".to_string());
    }

    // 3. UTF-16BE BOM
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return (String::from_utf16_lossy(&u16s), "UNICODE".to_string());
    }

    // Check header `1 CHAR` in ASCII prefix
    let ascii_head: String = bytes
        .iter()
        .take(1024)
        .map(|&b| if b.is_ascii() { b as char } else { ' ' })
        .collect();
    let declared_char = ascii_head
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("1 CHAR ")
                .map(|v| v.trim().to_ascii_uppercase())
        })
        .unwrap_or_else(|| "UTF-8".to_string());

    if declared_char == "ANSEL" {
        return (decode_ansel(bytes), "ANSEL".to_string());
    }

    if let Ok(utf8_str) = std::str::from_utf8(bytes) {
        return (utf8_str.to_string(), declared_char);
    }

    // Non-UTF-8 8-bit legacy encoding: Windows-1251 vs Windows-1252
    if declared_char.contains("1251") || declared_char.contains("CYR") {
        return (decode_windows_1251(bytes), "WINDOWS-1251".to_string());
    }

    // Heuristic: if many bytes in 0xC0..=0xFF exist, check if Windows-1252 vs Windows-1251
    let high_bytes_count = bytes.iter().filter(|&&b| b >= 0xC0).count();
    if high_bytes_count > 8 && declared_char != "ANSI" && declared_char != "WINDOWS-1252" {
        return (decode_windows_1251(bytes), "WINDOWS-1251".to_string());
    }

    (decode_windows_1252(bytes), "ANSI".to_string())
}

fn decode_windows_1252(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        let ch = match b {
            0x80 => '€',
            0x82 => '‚',
            0x83 => 'ƒ',
            0x84 => '„',
            0x85 => '…',
            0x86 => '†',
            0x87 => '‡',
            0x88 => 'ˆ',
            0x89 => '‰',
            0x8A => 'Š',
            0x8B => '‹',
            0x8C => 'Œ',
            0x8E => 'Ž',
            0x91 => '‘',
            0x92 => '’',
            0x93 => '“',
            0x94 => '”',
            0x95 => '•',
            0x96 => '–',
            0x97 => '—',
            0x98 => '˜',
            0x99 => '™',
            0x9A => 'š',
            0x9B => '›',
            0x9C => 'œ',
            0x9E => 'ž',
            0x9F => 'Ÿ',
            other => other as char,
        };
        out.push(ch);
    }
    out
}

fn decode_windows_1251(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        let ch = match b {
            0xA8 => 'Ё',
            0xB8 => 'ё',
            0xC0..=0xFF => char::from_u32(0x0410 + (b as u32 - 0xC0)).unwrap_or('?'),
            other => other as char,
        };
        out.push(ch);
    }
    out
}

/// Decodes ANSEL (ANSI/NISO Z39.47) bytes where combining diacritics precede the base character.
fn decode_ansel(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if (0xE0..=0xFE).contains(&b) && i + 1 < bytes.len() {
            let base = bytes[i + 1] as char;
            let composed = compose_ansel_diacritic(b, base);
            out.push_str(&composed);
            i += 2;
            continue;
        }
        let ch = match b {
            0xA1 => 'Ł',
            0xA2 => 'Ø',
            0xA3 => 'Đ',
            0xA4 => 'Þ',
            0xA5 => 'Æ',
            0xA6 => 'Œ',
            0xB1 => 'ł',
            0xB2 => 'ø',
            0xB3 => 'đ',
            0xB4 => 'þ',
            0xB5 => 'æ',
            0xB6 => 'œ',
            0xCF => 'ß',
            other => other as char,
        };
        out.push(ch);
        i += 1;
    }
    out
}

fn compose_ansel_diacritic(diacritic: u8, base: char) -> String {
    match (diacritic, base) {
        // Grave (0xE1)
        (0xE1, 'a') => "à".to_string(),
        (0xE1, 'A') => "À".to_string(),
        (0xE1, 'e') => "è".to_string(),
        (0xE1, 'E') => "È".to_string(),
        (0xE1, 'i') => "ì".to_string(),
        (0xE1, 'o') => "ò".to_string(),
        (0xE1, 'u') => "ù".to_string(),
        // Acute (0xE2)
        (0xE2, 'a') => "á".to_string(),
        (0xE2, 'A') => "Á".to_string(),
        (0xE2, 'e') => "é".to_string(),
        (0xE2, 'E') => "É".to_string(),
        (0xE2, 'i') => "í".to_string(),
        (0xE2, 'I') => "Í".to_string(),
        (0xE2, 'o') => "ó".to_string(),
        (0xE2, 'O') => "Ó".to_string(),
        (0xE2, 'u') => "ú".to_string(),
        (0xE2, 'U') => "Ú".to_string(),
        (0xE2, 'y') => "ý".to_string(),
        (0xE2, 'c') => "ć".to_string(),
        (0xE2, 'n') => "ń".to_string(),
        (0xE2, 's') => "ś".to_string(),
        (0xE2, 'z') => "ź".to_string(),
        // Circumflex (0xE3)
        (0xE3, 'a') => "â".to_string(),
        (0xE3, 'e') => "ê".to_string(),
        (0xE3, 'i') => "î".to_string(),
        (0xE3, 'o') => "ô".to_string(),
        (0xE3, 'u') => "û".to_string(),
        // Tilde (0xE4)
        (0xE4, 'n') => "ñ".to_string(),
        (0xE4, 'N') => "Ñ".to_string(),
        (0xE4, 'a') => "ã".to_string(),
        (0xE4, 'o') => "õ".to_string(),
        // Umlaut / Diaeresis (0xE8)
        (0xE8, 'a') => "ä".to_string(),
        (0xE8, 'A') => "Ä".to_string(),
        (0xE8, 'e') => "ë".to_string(),
        (0xE8, 'E') => "Ë".to_string(),
        (0xE8, 'i') => "ï".to_string(),
        (0xE8, 'o') => "ö".to_string(),
        (0xE8, 'O') => "Ö".to_string(),
        (0xE8, 'u') => "ü".to_string(),
        (0xE8, 'U') => "Ü".to_string(),
        (0xE8, 'y') => "ÿ".to_string(),
        // Caron / Háček (0xE9)
        (0xE9, 'c') => "č".to_string(),
        (0xE9, 'C') => "Č".to_string(),
        (0xE9, 's') => "š".to_string(),
        (0xE9, 'S') => "Š".to_string(),
        (0xE9, 'z') => "ž".to_string(),
        (0xE9, 'Z') => "Ž".to_string(),
        (0xE9, 'r') => "ř".to_string(),
        // Ring above (0xEA)
        (0xEA, 'a') => "å".to_string(),
        (0xEA, 'A') => "Å".to_string(),
        // Cedilla (0xF0)
        (0xF0, 'c') => "ç".to_string(),
        (0xF0, 'C') => "Ç".to_string(),
        _ => base.to_string(),
    }
}

fn unescape_gedcom_value(raw: &str) -> String {
    if raw.contains("@@") {
        raw.replace("@@", "@")
    } else {
        raw.to_string()
    }
}

/// Parses raw GEDCOM text into a forest of top-level (`level == 0`) `GedcomNode` records,
/// handling `CONC` and `CONT` continuations at any nesting depth (`03 §2.1 #1`).
pub fn parse_gedcom_tree(text: &str) -> Result<Vec<GedcomNode>, AppError> {
    let cleaned = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut stack: Vec<GedcomNode> = Vec::new();
    let mut roots: Vec<GedcomNode> = Vec::new();

    for raw_line in cleaned.lines() {
        let line = raw_line.trim_end_matches('\r');
        let trimmed_start = line.trim_start();
        if trimmed_start.is_empty() {
            continue;
        }

        let Some((level_str, rest)) = trimmed_start.split_once(' ') else {
            // Line with just level and tag (no spaces)? No, level and tag are always separated by space.
            continue;
        };
        let Ok(level) = level_str.trim().parse::<u8>() else {
            continue;
        };

        let rest_trimmed = rest.trim_start();
        let (xref, tag_and_val) = if let Some(after_start_at) = rest_trimmed.strip_prefix('@') {
            if let Some(end_at) = after_start_at.find('@') {
                let xref_end = end_at + 2;
                let xref_token = rest_trimmed[..xref_end].to_string();
                let after_xref = rest_trimmed[xref_end..].trim_start();
                (Some(xref_token), after_xref)
            } else {
                (None, rest_trimmed)
            }
        } else {
            (None, rest_trimmed)
        };

        let (tag_raw, val_raw) = match tag_and_val.split_once(' ') {
            Some((t, v)) => (t.trim(), v),
            None => (tag_and_val.trim(), ""),
        };

        if tag_raw.is_empty() {
            continue;
        }

        let tag_upper = tag_raw.to_ascii_uppercase();
        let value_unescaped = unescape_gedcom_value(val_raw);

        // Handle CONC and CONT by appending directly to the parent node on the stack
        if (tag_upper == "CONC" || tag_upper == "CONT") && !stack.is_empty() {
            while stack.len() > 1 && stack.last().is_some_and(|n| n.level >= level) {
                let completed = stack.pop().unwrap();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(completed);
                }
            }
            if let Some(target) = stack.last_mut() {
                if tag_upper == "CONT" {
                    target.value.push('\n');
                }
                target.value.push_str(&value_unescaped);
                continue;
            }
        }

        // Pop completed nodes of level >= current level
        while stack.last().is_some_and(|n| n.level >= level) {
            let completed = stack.pop().unwrap();
            if let Some(parent) = stack.last_mut() {
                parent.children.push(completed);
            } else {
                roots.push(completed);
            }
        }

        stack.push(GedcomNode {
            level,
            xref,
            tag: tag_upper,
            value: value_unescaped.trim_end().to_string(),
            children: Vec::new(),
        });
    }

    while let Some(completed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(completed);
        } else {
            roots.push(completed);
        }
    }

    if roots.is_empty() {
        let mut fields = HashMap::new();
        fields.insert("data".to_string(), "invalid_gedcom".to_string());
        return Err(AppError::ValidationFailed(
            "GEDCOM payload does not contain any valid records".to_string(),
            Some(fields),
        ));
    }

    Ok(roots)
}

/// Parses directional (`N56.858721`, `S12.34`, `E35.917596`, `W74.006`) or signed decimal coordinates (`03 §2.2 #2`).
/// Returns `(coordinate, had_directional_prefix)`.
pub fn parse_gedcom_coordinate(raw: &str) -> Option<(f64, bool)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let first = trimmed.chars().next()?;
    let (sign, rest, had_dir) = match first {
        'N' | 'n' | 'E' | 'e' => (1.0, trimmed[first.len_utf8()..].trim(), true),
        'S' | 's' | 'W' | 'w' => (-1.0, trimmed[first.len_utf8()..].trim(), true),
        _ => (1.0, trimmed, false),
    };

    let numeric_clean: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
        .collect();
    let val = numeric_clean.parse::<f64>().ok()?;
    Some((val * sign, had_dir))
}

/// Formats a decimal latitude (`N`/`S`) or longitude (`E`/`W`) for GEDCOM `MAP` -> `LATI`/`LONG`.
pub fn format_gedcom_latitude(lat: f64) -> String {
    let prefix = if lat >= 0.0 { 'N' } else { 'S' };
    format!("{prefix}{}", format_coord_number(lat.abs()))
}

pub fn format_gedcom_longitude(lon: f64) -> String {
    let prefix = if lon >= 0.0 { 'E' } else { 'W' };
    format!("{prefix}{}", format_coord_number(lon.abs()))
}

fn format_coord_number(abs_val: f64) -> String {
    let s = format!("{abs_val:.6}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn looks_like_slavic_patronymic(token: &str) -> bool {
    let lower = token.trim().to_lowercase();
    lower.ends_with("ович")
        || lower.ends_with("евич")
        || lower.ends_with("ич")
        || lower.ends_with("овна")
        || lower.ends_with("евна")
        || lower.ends_with("ична")
        || lower.ends_with("инична")
        || lower.ends_with("ovich")
        || lower.ends_with("evich")
        || lower.ends_with("ovna")
        || lower.ends_with("evna")
        || lower.ends_with("ichna")
}

#[derive(Debug, Clone, Default)]
struct ParsedGedcomName {
    name_type: Option<String>,
    given_name: String,
    patronymic: Option<String>,
    surname: String,
    married_surname: Option<String>,
    prefix: Option<String>,
    suffix: Option<String>,
    nickname: Option<String>,
}

fn parse_gedcom_name_node(node: &GedcomNode) -> ParsedGedcomName {
    let raw_val = node.value.trim();
    let val_given;
    let mut val_surname = String::new();
    let mut val_suffix = String::new();

    if let Some(first_slash) = raw_val.find('/') {
        val_given = raw_val[..first_slash].trim().to_string();
        let after_first = &raw_val[first_slash + 1..];
        if let Some(second_slash) = after_first.find('/') {
            val_surname = after_first[..second_slash].trim().to_string();
            val_suffix = after_first[second_slash + 1..].trim().to_string();
        } else {
            val_surname = after_first.trim().to_string();
        }
    } else {
        val_given = raw_val.to_string();
    }

    let givn_sub = node.child_value("GIVN");
    let surn_sub = node.child_value("SURN");
    let spfx_sub = node.child_value("SPFX");
    let patr_sub = node.child_value("_PATR");
    let marnm_sub = node.child_value("_MARNM");
    let npfx_sub = node.child_value("NPFX");
    let nsfx_sub = node
        .child_value("NSFX")
        .or_else(|| (!val_suffix.is_empty()).then_some(val_suffix));
    let nick_sub = node.child_value("NICK");
    let type_sub = node
        .child_value("TYPE")
        .or_else(|| node.child_value("_TYPE"))
        .map(|t| t.to_ascii_lowercase());

    let raw_given = givn_sub.unwrap_or(val_given);
    let mut surname = surn_sub.unwrap_or(val_surname);
    if let Some(spfx) = spfx_sub {
        if !surname.is_empty() && !surname.starts_with(&spfx) {
            surname = format!("{spfx} {surname}");
        }
    }

    let (given_name, patronymic) = if let Some(explicit_patr) = patr_sub {
        let first_given = raw_given
            .split_whitespace()
            .next()
            .unwrap_or(&raw_given)
            .to_string();
        (first_given, Some(explicit_patr))
    } else {
        let parts: Vec<&str> = raw_given.split_whitespace().collect();
        if parts.len() >= 2 && looks_like_slavic_patronymic(parts[1]) {
            (parts[0].to_string(), Some(parts[1..].join(" ")))
        } else {
            (raw_given.trim().to_string(), None)
        }
    };

    ParsedGedcomName {
        name_type: type_sub,
        given_name,
        patronymic,
        surname,
        married_surname: marnm_sub,
        prefix: npfx_sub,
        suffix: nsfx_sub,
        nickname: nick_sub,
    }
}

fn infer_mime_type_from_form_or_path(form: Option<&str>, path: &str) -> String {
    let ext = form
        .map(|f| f.trim().trim_start_matches('.').to_ascii_lowercase())
        .or_else(|| {
            path.rsplit_once('.')
                .map(|(_, e)| e.trim().to_ascii_lowercase())
        })
        .unwrap_or_default();

    match ext.as_str() {
        "jpg" | "jpeg" | " image/jpeg" => "image/jpeg".to_string(),
        "png" | "image/png" => "image/png".to_string(),
        "webp" | "image/webp" => "image/webp".to_string(),
        "gif" | "image/gif" => "image/gif".to_string(),
        "pdf" | "application/pdf" => "application/pdf".to_string(),
        _ => "image/jpeg".to_string(),
    }
}

fn strip_media_directory_path(raw_path: &str) -> (String, bool) {
    let trimmed = raw_path.trim();
    if let Some((_, file_part)) = trimmed.rsplit_once(['/', '\\']) {
        if !file_part.is_empty() {
            return (file_part.to_string(), true);
        }
    }
    (trimmed.to_string(), false)
}

#[derive(Debug, Clone)]
struct PendingAssociation {
    person_id: String,
    target_xref: String,
    rela: String,
}

/// Analyzes a GEDCOM payload without modifying the active database/store (`03 §3.1 Step 2, §6 DoD #6`).
pub fn analyze_gedcom_payload(
    raw_bytes: &[u8],
    current_year: i32,
    age_threshold: i32,
) -> Result<GedcomAnalysisReport, AppError> {
    let mut temp_store = GenealogyPrivacyStore::empty();
    let result = import_gedcom_into_store(
        &mut temp_store,
        raw_bytes,
        "new",
        None,
        current_year,
        age_threshold,
    )?;

    let (_, charset_detected) = decode_gedcom_bytes(raw_bytes);
    let roots = parse_gedcom_tree(&decode_gedcom_bytes(raw_bytes).0)?;
    let sources_count = roots
        .iter()
        .filter(|r| r.tag == "SOUR" && r.xref.is_some())
        .count();

    let mut dialect = result.dialect;
    if dialect.charset.is_empty() {
        dialect.charset = charset_detected;
    }

    let detected_software = match &dialect.source_version {
        Some(ver) if !ver.trim().is_empty() => {
            format!("{} {}", dialect.source_system.trim(), ver.trim())
        }
        _ => dialect.source_system.clone(),
    };

    Ok(GedcomAnalysisReport {
        total_individuals: result.individuals_imported,
        total_families: result.families_imported,
        total_events: result.events_imported,
        total_places: result.places_imported,
        total_media: result.media_imported,
        missing_media_count: result.media_missing_count,
        detected_software,
        gedcom_version: dialect.gedcom_version.clone(),
        character_encoding: dialect.charset.clone(),
        dialect_summary: dialect.clone(),
        warnings: result.warnings,
        file_name: "gedcom-import.ged".to_string(),
        file_size_bytes: raw_bytes.len(),
        individuals_count: result.individuals_imported,
        families_count: result.families_imported,
        media_count: result.media_imported,
        media_missing_count: result.media_missing_count,
        events_count: result.events_imported,
        places_count: result.places_imported,
        notes_count: result.notes_imported,
        sources_count,
        dialect,
        can_import: result.individuals_imported > 0 || result.families_imported > 0,
    })
}

/// Imports a GEDCOM 5.5.1 or 7.0 payload into `GenealogyPrivacyStore` (`03 §2.1, §2.2, §3.1, §6 DoD #1–#6`).
pub fn import_gedcom_into_store(
    store: &mut GenealogyPrivacyStore,
    raw_bytes: &[u8],
    mode_raw: &str,
    confirm_text: Option<&str>,
    current_year: i32,
    age_threshold: i32,
) -> Result<GedcomImportResult, AppError> {
    let mode = match mode_raw.trim().to_ascii_lowercase().as_str() {
        "" | "new" => "new".to_string(),
        "replace" | "clean_slate" => "replace".to_string(),
        "merge" => "merge".to_string(),
        _ => {
            let mut fields = HashMap::new();
            fields.insert("mode".to_string(), "invalid_mode".to_string());
            return Err(AppError::ValidationFailed(
                "Import mode must be 'new', 'replace', 'clean_slate', or 'merge'".to_string(),
                Some(fields),
            ));
        }
    };

    if mode == "replace" && !store.persons.is_empty() && !store.is_demo_seed {
        let confirmed = confirm_text.map(str::trim).is_some_and(|c| {
            c.eq_ignore_ascii_case("ЗАМЕНИТЬ")
                || c.eq_ignore_ascii_case("REPLACE")
                || c.eq_ignore_ascii_case("CLEAN_SLATE")
        });
        if !confirmed {
            let mut fields = HashMap::new();
            fields.insert(
                "confirm_text".to_string(),
                "confirmation_required".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "Type CLEAN_SLATE, ЗАМЕНИТЬ, or REPLACE to confirm replacing the existing family tree"
                    .to_string(),
                Some(fields),
            ));
        }
    }

    let (decoded_text, detected_charset) = decode_gedcom_bytes(raw_bytes);
    let roots = parse_gedcom_tree(&decoded_text)?;

    let snap_reason = if mode == "merge" {
        "merge_import"
    } else {
        "clean_slate_import"
    };
    let created_snap = crate::services::history::create_snapshot(
        store,
        Some(format!("Pre-GEDCOM import ({mode})")),
        snap_reason,
        None,
        None,
        "/tmp/genealogy-backups",
    )?;
    let snapshot_id = Some(created_snap.id);

    if mode == "new" || mode == "replace" {
        let preserved_snapshots = store.snapshots.clone();
        *store = GenealogyPrivacyStore::empty();
        store.snapshots = preserved_snapshots;
    } else if store.is_demo_seed {
        store.is_demo_seed = false;
    }

    let initial_places_count = store.places.len();
    let initial_persons_count = store.persons.len();
    let initial_families_count = store.families.len();
    let initial_events_count = store.events.len();
    let initial_media_count = store.media.len();
    let initial_notes_count = store.notes.len();

    let mut dialect = GedcomDialectSummary {
        source_system: "Unknown".to_string(),
        source_version: None,
        gedcom_version: "5.5.1".to_string(),
        charset: detected_charset,
        ..Default::default()
    };
    let mut warnings: Vec<String> = Vec::new();

    // Parse HEAD metadata
    if let Some(head) = roots.iter().find(|r| r.tag == "HEAD") {
        if let Some(sour) = head.find_child("SOUR") {
            if !sour.value.trim().is_empty() {
                dialect.source_system = sour.value.trim().to_string();
            } else if let Some(name) = sour.child_value("NAME") {
                dialect.source_system = name;
            }
            dialect.source_version = sour.child_value("VERS");
        }
        if let Some(gedc) = head.find_child("GEDC") {
            if let Some(vers) = gedc.child_value("VERS") {
                dialect.gedcom_version = vers;
            }
        }
        if let Some(ch) = head.child_value("CHAR") {
            dialect.charset = ch;
        }
    }

    // Pass 1: Collect top-level NOTE (`0 @Nxxxx@ NOTE`), SOUR (`0 @Sxxxx@ SOUR`), and OBJE (`0 @Oxxxx@ OBJE`)
    let mut top_level_notes: HashMap<String, String> = HashMap::new();
    let mut attached_top_level_notes: HashSet<String> = HashSet::new();
    let mut top_level_sources: HashMap<String, String> = HashMap::new();
    let mut obje_xref_to_media_id: HashMap<String, String> = HashMap::new();

    for node in &roots {
        match node.tag.as_str() {
            "NOTE" => {
                if let Some(ref xref) = node.xref {
                    let key = xref.trim_matches('@').to_uppercase();
                    top_level_notes.insert(key, node.value.trim().to_string());
                }
            }
            "SOUR" => {
                if let Some(ref xref) = node.xref {
                    let key = xref.trim_matches('@').to_uppercase();
                    let title = node
                        .child_value("TITL")
                        .or_else(|| node.child_value("ABBR"))
                        .unwrap_or_else(|| node.value.trim().to_string());
                    top_level_sources.insert(key, title);
                }
            }
            "OBJE" => {
                let raw_xref = node.xref.as_deref();
                let existing_gids: HashSet<String> =
                    store.media.iter().map(|m| m.gid.clone()).collect();
                let gid =
                    allocate_gid('M', &mut store.next_media_gid_seq, raw_xref, &existing_gids);

                let file_node = node.find_child("FILE");
                let raw_path = file_node
                    .map(|f| f.value.trim().to_string())
                    .unwrap_or_default();
                let (clean_file_name, was_stripped) = strip_media_directory_path(&raw_path);
                if was_stripped {
                    dialect.stripped_media_paths_count += 1;
                }

                let form_val = file_node
                    .and_then(|f| f.child_value("FORM"))
                    .or_else(|| node.child_value("FORM"));
                let title_val = file_node
                    .and_then(|f| f.child_value("TITL"))
                    .or_else(|| node.child_value("TITL"))
                    .unwrap_or_else(|| {
                        if !clean_file_name.is_empty() {
                            clean_file_name.clone()
                        } else {
                            gid.clone()
                        }
                    });

                let mime_type = infer_mime_type_from_form_or_path(form_val.as_deref(), &raw_path);
                let media_id = Uuid::new_v4().to_string();

                if let Some(xref) = raw_xref {
                    obje_xref_to_media_id
                        .insert(xref.trim_matches('@').to_uppercase(), media_id.clone());
                }
                obje_xref_to_media_id.insert(gid.to_uppercase(), media_id.clone());

                store.media.push(MediaAssetSummaryDto {
                    id: media_id.clone(),
                    gid,
                    title: title_val,
                    mime_type,
                    file_url: format!("/api/v1/media/{media_id}/file"),
                    file_name: if clean_file_name.is_empty() {
                        None
                    } else {
                        Some(clean_file_name.clone())
                    },
                    original_file_name: if clean_file_name.is_empty() {
                        None
                    } else {
                        Some(clean_file_name)
                    },
                    file_size_bytes: None,
                    sha256: None,
                    width: None,
                    height: None,
                    description: None,
                    date_taken: None,
                    date_sort: None,
                    place_id: None,
                    place_full_name: None,
                    is_missing: true,
                    external_path: if raw_path.is_empty() {
                        None
                    } else {
                        Some(raw_path)
                    },
                    is_private: false,
                    only_living_persons: false,
                    linked_person_ids: vec![],
                    face_tags: vec![],
                    links: vec![],
                    version: 1,
                });
            }
            _ => {}
        }
    }

    // Pass 2: Parse all `0 @Ixxxx@ INDI` records
    let mut indi_xref_to_person_id: HashMap<String, String> = HashMap::new();
    let mut person_famc_pedi: HashMap<(String, String), String> = HashMap::new();
    let mut pending_associations: Vec<PendingAssociation> = Vec::new();

    for node in &roots {
        if node.tag != "INDI" {
            continue;
        }

        let raw_xref = node.xref.as_deref();
        let xref_clean = raw_xref
            .map(|x| x.trim_matches('@').to_uppercase())
            .unwrap_or_default();

        // Extract external IDs (`_UID` / `UID` and `FACT ...` + `2 TYPE RIN` / `RIN`)
        let mut external_ids: HashMap<String, String> = HashMap::new();
        for child in &node.children {
            match child.tag.as_str() {
                "_UID" | "UID" => {
                    let uid_val = child.value.trim().to_string();
                    if !uid_val.is_empty() {
                        external_ids.insert("uid".to_string(), uid_val);
                        dialect.preserved_uids_count += 1;
                    }
                }
                "RIN" => {
                    let rin_val = child.value.trim().to_string();
                    if !rin_val.is_empty() {
                        external_ids.insert("mh_rin".to_string(), rin_val);
                        dialect.preserved_rins_count += 1;
                    }
                }
                "FACT"
                    if child
                        .child_value("TYPE")
                        .is_some_and(|t| t.eq_ignore_ascii_case("RIN")) =>
                {
                    let rin_val = child.value.trim().to_string();
                    if !rin_val.is_empty() {
                        external_ids.insert("mh_rin".to_string(), rin_val);
                        dialect.preserved_rins_count += 1;
                    }
                }
                _ => {}
            }
        }

        // Parse all `1 NAME` blocks (`03 §2.2 #1`)
        let parsed_names: Vec<ParsedGedcomName> = node
            .children
            .iter()
            .filter(|c| c.tag == "NAME")
            .map(parse_gedcom_name_node)
            .collect();

        let mut given_name = String::new();
        let mut patronymic: Option<String> = None;
        let mut surname = String::new();
        let mut birth_surname: Option<String> = None;
        let mut nickname: Option<String> = None;
        let mut name_prefix: Option<String> = None;
        let mut name_suffix: Option<String> = None;
        let mut alt_names: Vec<IndividualAltNameDto> = Vec::new();

        let birth_entry = parsed_names
            .iter()
            .find(|n| matches!(n.name_type.as_deref(), Some("birth" | "maiden")))
            .or_else(|| parsed_names.first());
        let married_entry = parsed_names
            .iter()
            .find(|n| matches!(n.name_type.as_deref(), Some("married")));

        if let Some(b_name) = birth_entry {
            given_name = b_name.given_name.clone();
            patronymic = b_name.patronymic.clone();
            surname = b_name.surname.clone();
            if !b_name.surname.is_empty() {
                birth_surname = Some(b_name.surname.clone());
            }
            if let Some(ref marnm) = b_name.married_surname {
                if !marnm.is_empty() {
                    birth_surname = if !b_name.surname.is_empty() {
                        Some(b_name.surname.clone())
                    } else {
                        None
                    };
                    surname = marnm.clone();
                    dialect.married_names_resolved += 1;
                    alt_names.push(IndividualAltNameDto {
                        id: Uuid::new_v4().to_string(),
                        name_type: "married".to_string(),
                        given_name: Some(given_name.clone()),
                        patronymic: patronymic.clone(),
                        surname: Some(marnm.clone()),
                        lang: None,
                    });
                }
            }
            nickname = b_name.nickname.clone();
            name_prefix = b_name.prefix.clone();
            name_suffix = b_name.suffix.clone();
        }

        if let Some(m_name) = married_entry {
            if given_name.is_empty() && !m_name.given_name.is_empty() {
                given_name = m_name.given_name.clone();
            }
            if patronymic.is_none() && m_name.patronymic.is_some() {
                patronymic = m_name.patronymic.clone();
            }
            if !m_name.surname.is_empty() {
                surname = m_name.surname.clone();
                dialect.married_names_resolved += 1;
                alt_names.push(IndividualAltNameDto {
                    id: Uuid::new_v4().to_string(),
                    name_type: "married".to_string(),
                    given_name: if m_name.given_name.is_empty() {
                        Some(given_name.clone())
                    } else {
                        Some(m_name.given_name.clone())
                    },
                    patronymic: m_name.patronymic.clone().or_else(|| patronymic.clone()),
                    surname: Some(m_name.surname.clone()),
                    lang: None,
                });
            }
        }

        for extra in &parsed_names {
            let t = extra.name_type.as_deref().unwrap_or("");
            if !matches!(t, "" | "birth" | "maiden" | "married") {
                alt_names.push(IndividualAltNameDto {
                    id: Uuid::new_v4().to_string(),
                    name_type: t.to_string(),
                    given_name: if extra.given_name.is_empty() {
                        None
                    } else {
                        Some(extra.given_name.clone())
                    },
                    patronymic: extra.patronymic.clone(),
                    surname: if extra.surname.is_empty() {
                        None
                    } else {
                        Some(extra.surname.clone())
                    },
                    lang: None,
                });
            }
        }

        if let Some(top_nick) = node.child_value("NICK") {
            if nickname.is_none() {
                nickname = Some(top_nick);
            }
        }

        let sex = match node
            .child_value("SEX")
            .unwrap_or_else(|| "U".to_string())
            .to_ascii_uppercase()
            .as_str()
        {
            "M" | "MALE" => "M".to_string(),
            "F" | "FEMALE" => "F".to_string(),
            _ => "U".to_string(),
        };

        // In merge mode, check if person already exists by external UID or GID
        let existing_person_idx = if mode == "merge" {
            let uid_opt = external_ids.get("uid");
            store.persons.iter().position(|p| {
                (uid_opt.is_some() && p.external_ids.get("uid") == uid_opt)
                    || (!xref_clean.is_empty() && p.gid.eq_ignore_ascii_case(&xref_clean))
            })
        } else {
            None
        };

        let person_id = if let Some(idx) = existing_person_idx {
            let pid = store.persons[idx].id.clone();
            for (k, v) in &external_ids {
                store.persons[idx].external_ids.insert(k.clone(), v.clone());
            }
            if store.persons[idx].birth_surname.is_none() && birth_surname.is_some() {
                store.persons[idx].birth_surname = birth_surname;
            }
            pid
        } else {
            let existing_gids: HashSet<String> =
                store.persons.iter().map(|p| p.gid.clone()).collect();
            let gid = allocate_gid('I', &mut store.next_gid_seq, raw_xref, &existing_gids);
            let new_pid = Uuid::new_v4().to_string();

            store.persons.push(TreePersonNodeDto {
                id: new_pid.clone(),
                gid: gid.clone(),
                sex,
                is_living: true,
                living_override: None,
                is_private: node.child_value("RESN").is_some_and(|r| {
                    r.to_ascii_lowercase().contains("privacy")
                        || r.to_ascii_lowercase().contains("confidential")
                }),
                primary_name: PersonNameDto {
                    given_name,
                    patronymic,
                    surname,
                },
                birth_surname,
                nickname,
                name_prefix,
                name_suffix,
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
                alt_names,
                external_ids,
                version: 1,
                tx_id: None,
            });
            new_pid
        };

        if !xref_clean.is_empty() {
            indi_xref_to_person_id.insert(xref_clean.clone(), person_id.clone());
        }

        // Parse child tags on INDI: events, FAMC, ASSO, OBJE, NOTE
        for child in &node.children {
            match child.tag.as_str() {
                "FAMC" => {
                    let fam_xref = child.value.trim().trim_matches('@').to_uppercase();
                    if !fam_xref.is_empty() {
                        let pedi = child
                            .child_value("PEDI")
                            .map(|p| p.to_ascii_lowercase())
                            .unwrap_or_else(|| "birth".to_string());
                        let mapped_pedi = match pedi.as_str() {
                            "adopted" | "adoption" => "adopted",
                            "foster" => "foster",
                            "step" => "step",
                            "unknown" => "unknown",
                            _ => "birth",
                        };
                        person_famc_pedi
                            .insert((person_id.clone(), fam_xref), mapped_pedi.to_string());
                    }
                }
                "ASSO" => {
                    let target_xref = child.value.trim().trim_matches('@').to_uppercase();
                    let rela = child
                        .child_value("RELA")
                        .or_else(|| child.child_value("TYPE"))
                        .unwrap_or_else(|| "Association".to_string());
                    if !target_xref.is_empty() {
                        pending_associations.push(PendingAssociation {
                            person_id: person_id.clone(),
                            target_xref,
                            rela,
                        });
                    }
                }
                "OBJE" => {
                    let obje_xref = child.value.trim().trim_matches('@').to_uppercase();
                    if let Some(media_id) = obje_xref_to_media_id.get(&obje_xref).cloned() {
                        link_media_to_person(store, &media_id, &person_id);
                    }
                }
                "NOTE" => {
                    attach_gedcom_note(
                        store,
                        &child.value,
                        "individual",
                        &person_id,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                    );
                }
                "_UPD" => {
                    dialect.filtered_upd_timestamps_count += 1;
                }
                "EVEN" => {
                    if child
                        .child_value("TYPE")
                        .is_some_and(|t| t.eq_ignore_ascii_case("_UPD"))
                    {
                        dialect.filtered_upd_timestamps_count += 1;
                        continue;
                    }
                    parse_and_insert_event(
                        store,
                        child,
                        Some(&person_id),
                        None,
                        "custom",
                        &mut dialect,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                        &top_level_sources,
                        &obje_xref_to_media_id,
                    );
                }
                "BIRT" | "CHR" | "BAPM" | "DEAT" | "BURI" | "CREM" | "ADOP" | "OCCU" | "EDUC"
                | "GRAD" | "RESI" | "IMMI" | "EMIG" | "NATU" | "CENS" | "MILI" | "RETI" => {
                    let ev_type = match child.tag.as_str() {
                        "BIRT" => "birth",
                        "CHR" | "BAPM" => "baptism",
                        "DEAT" => "death",
                        "BURI" => "burial",
                        "CREM" => "cremation",
                        "ADOP" => "adoption",
                        "OCCU" => "occupation",
                        "EDUC" => "education",
                        "GRAD" => "graduation",
                        "RESI" => "residence",
                        "IMMI" => "immigration",
                        "EMIG" => "emigration",
                        "NATU" => "naturalization",
                        "CENS" => "census",
                        "MILI" => "military_service",
                        "RETI" => "retirement",
                        _ => "custom",
                    };

                    if child.tag == "DEAT" {
                        if let Some(p) = store.persons.iter_mut().find(|p| p.id == person_id) {
                            p.has_death_event = true;
                            p.living_override = Some(false);
                        }
                        let has_date_or_place = child.find_child("DATE").is_some()
                            || child.find_child("PLAC").is_some();
                        if !has_date_or_place {
                            dialect.deceased_flag_only_count += 1;
                        }
                    }

                    parse_and_insert_event(
                        store,
                        child,
                        Some(&person_id),
                        None,
                        ev_type,
                        &mut dialect,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                        &top_level_sources,
                        &obje_xref_to_media_id,
                    );
                }
                _ => {}
            }
        }
    }

    // Pass 3: Parse all `0 @Fxxxx@ FAM` records
    for node in &roots {
        if node.tag != "FAM" {
            continue;
        }

        let raw_xref = node.xref.as_deref();
        let fam_xref_clean = raw_xref
            .map(|x| x.trim_matches('@').to_uppercase())
            .unwrap_or_default();

        let mut external_ids: HashMap<String, String> = HashMap::new();
        for child in &node.children {
            match child.tag.as_str() {
                "_UID" | "UID" => {
                    let uid_val = child.value.trim().to_string();
                    if !uid_val.is_empty() {
                        external_ids.insert("uid".to_string(), uid_val);
                        dialect.preserved_uids_count += 1;
                    }
                }
                "RIN" => {
                    let rin_val = child.value.trim().to_string();
                    if !rin_val.is_empty() {
                        external_ids.insert("mh_rin".to_string(), rin_val);
                        dialect.preserved_rins_count += 1;
                    }
                }
                "FACT"
                    if child
                        .child_value("TYPE")
                        .is_some_and(|t| t.eq_ignore_ascii_case("RIN")) =>
                {
                    let rin_val = child.value.trim().to_string();
                    if !rin_val.is_empty() {
                        external_ids.insert("mh_rin".to_string(), rin_val);
                        dialect.preserved_rins_count += 1;
                    }
                }
                _ => {}
            }
        }

        let partner1_id = node
            .child_value("HUSB")
            .map(|x| x.trim_matches('@').to_uppercase())
            .and_then(|x| indi_xref_to_person_id.get(&x).cloned());
        let partner2_id = node
            .child_value("WIFE")
            .map(|x| x.trim_matches('@').to_uppercase())
            .and_then(|x| indi_xref_to_person_id.get(&x).cloned());

        let mut children: Vec<FamilyChildDto> = Vec::new();
        for (idx, ch_node) in node.children.iter().filter(|c| c.tag == "CHIL").enumerate() {
            let ch_xref = ch_node.value.trim().trim_matches('@').to_uppercase();
            if let Some(child_id) = indi_xref_to_person_id.get(&ch_xref).cloned() {
                let pedi = person_famc_pedi
                    .get(&(child_id.clone(), fam_xref_clean.clone()))
                    .cloned()
                    .unwrap_or_else(|| "birth".to_string());
                children.push(FamilyChildDto {
                    child_id,
                    sort_order: idx as i32,
                    rel_to_partner1: pedi.clone(),
                    rel_to_partner2: pedi,
                });
            }
        }

        let has_div = node.find_child("DIV").is_some();
        let has_marr = node.find_child("MARR").is_some() || node.find_child("MARL").is_some();
        let relationship_type = if has_div {
            "divorced".to_string()
        } else if has_marr || (partner1_id.is_some() && partner2_id.is_some()) {
            "married".to_string()
        } else {
            "unknown".to_string()
        };

        let existing_gids: HashSet<String> = store.families.iter().map(|f| f.gid.clone()).collect();
        let fam_gid = allocate_gid(
            'F',
            &mut store.next_family_gid_seq,
            raw_xref,
            &existing_gids,
        );
        let fam_id = Uuid::new_v4().to_string();

        store.families.push(FamilyDto {
            id: fam_id.clone(),
            gid: fam_gid,
            partner1_id,
            partner2_id,
            relationship_type,
            is_private: false,
            children,
            external_ids,
            version: 1,
        });

        for child in &node.children {
            match child.tag.as_str() {
                "_UPD" => {
                    dialect.filtered_upd_timestamps_count += 1;
                }
                "EVEN" => {
                    if child
                        .child_value("TYPE")
                        .is_some_and(|t| t.eq_ignore_ascii_case("_UPD"))
                    {
                        dialect.filtered_upd_timestamps_count += 1;
                        continue;
                    }
                    parse_and_insert_event(
                        store,
                        child,
                        None,
                        Some(&fam_id),
                        "custom",
                        &mut dialect,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                        &top_level_sources,
                        &obje_xref_to_media_id,
                    );
                }
                "MARL" => {
                    dialect.marriage_licenses_count += 1;
                    parse_and_insert_event(
                        store,
                        child,
                        None,
                        Some(&fam_id),
                        "marriage_license",
                        &mut dialect,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                        &top_level_sources,
                        &obje_xref_to_media_id,
                    );
                }
                "MARR" => {
                    let is_flag_only =
                        child.value.trim().eq_ignore_ascii_case("Y") && child.children.is_empty();
                    if !is_flag_only {
                        parse_and_insert_event(
                            store,
                            child,
                            None,
                            Some(&fam_id),
                            "marriage",
                            &mut dialect,
                            &top_level_notes,
                            &mut attached_top_level_notes,
                            &top_level_sources,
                            &obje_xref_to_media_id,
                        );
                    }
                }
                "DIV" | "ENGA" | "ANUL" => {
                    let ev_type = match child.tag.as_str() {
                        "DIV" => "divorce",
                        "ENGA" => "engagement",
                        _ => "custom",
                    };
                    parse_and_insert_event(
                        store,
                        child,
                        None,
                        Some(&fam_id),
                        ev_type,
                        &mut dialect,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                        &top_level_sources,
                        &obje_xref_to_media_id,
                    );
                }
                "NOTE" => {
                    attach_gedcom_note(
                        store,
                        &child.value,
                        "family",
                        &fam_id,
                        &top_level_notes,
                        &mut attached_top_level_notes,
                    );
                }
                _ => {}
            }
        }
    }

    // Pass 4: Resolve `ASSO` associations (`03 §2.2 #7`)
    for assoc in pending_associations {
        let Some(target_person_id) = indi_xref_to_person_id.get(&assoc.target_xref).cloned() else {
            warnings.push(format!("Unresolved ASSO target @{}@", assoc.target_xref));
            continue;
        };

        let rela_lower = assoc.rela.trim().to_lowercase();
        let is_parental = matches!(
            rela_lower.as_str(),
            "отец" | "мать" | "родитель" | "father" | "mother" | "parent"
        );

        if is_parental {
            let already_linked = store.families.iter().any(|f| {
                let is_child = f.children.iter().any(|c| c.child_id == assoc.person_id);
                let is_parent = f.partner1_id.as_deref() == Some(target_person_id.as_str())
                    || f.partner2_id.as_deref() == Some(target_person_id.as_str());
                is_child && is_parent
            });

            if already_linked {
                dialect.deduplicated_parental_associations_count += 1;
            } else {
                // Synthesize or attach to a single-parent family
                let target_sex = store
                    .persons
                    .iter()
                    .find(|p| p.id == target_person_id)
                    .map(|p| p.sex.clone())
                    .unwrap_or_else(|| "U".to_string());
                let existing_gids: HashSet<String> =
                    store.families.iter().map(|f| f.gid.clone()).collect();
                let fam_gid =
                    allocate_gid('F', &mut store.next_family_gid_seq, None, &existing_gids);
                let (p1, p2) =
                    if target_sex == "F" || rela_lower == "мать" || rela_lower == "mother" {
                        (None, Some(target_person_id))
                    } else {
                        (Some(target_person_id), None)
                    };
                store.families.push(FamilyDto {
                    id: Uuid::new_v4().to_string(),
                    gid: fam_gid,
                    partner1_id: p1,
                    partner2_id: p2,
                    relationship_type: "unknown".to_string(),
                    is_private: false,
                    children: vec![FamilyChildDto {
                        child_id: assoc.person_id,
                        sort_order: 0,
                        rel_to_partner1: "birth".to_string(),
                        rel_to_partner2: "birth".to_string(),
                    }],
                    external_ids: HashMap::new(),
                    version: 1,
                });
                dialect.synthesized_single_parent_families_count += 1;
            }
        } else {
            let target_info = store
                .persons
                .iter()
                .find(|p| p.id == target_person_id)
                .map(|p| {
                    format!(
                        "@{}@ {} {}",
                        p.gid, p.primary_name.given_name, p.primary_name.surname
                    )
                    .trim()
                    .to_string()
                })
                .unwrap_or_else(|| format!("@{}@", assoc.target_xref));

            store.notes.push(NoteDto {
                id: Uuid::new_v4().to_string(),
                entity_type: "individual".to_string(),
                entity_id: assoc.person_id,
                body: format!("Связь ({}): {}", assoc.rela.trim(), target_info),
                is_private: false,
                sort_order: store.notes.len() as i32,
                version: 1,
            });
            dialect.social_associations_notes_count += 1;
        }
    }

    // Pass 5: Recompute vitals, living status, place counts, and media `only_living_persons`
    recompute_all_individuals_vitals(store, current_year, age_threshold);
    recompute_all_places_metadata(store);

    let living_map: HashMap<String, bool> = store
        .persons
        .iter()
        .map(|p| (p.id.clone(), p.is_living))
        .collect();
    for media in &mut store.media {
        if !media.linked_person_ids.is_empty() {
            media.only_living_persons = media
                .linked_person_ids
                .iter()
                .all(|pid| living_map.get(pid).copied().unwrap_or(true));
        }
    }

    let individuals_imported = store.persons.len().saturating_sub(initial_persons_count);
    let families_imported = store.families.len().saturating_sub(initial_families_count);
    let events_imported = store.events.len().saturating_sub(initial_events_count);
    let places_imported = store.places.len().saturating_sub(initial_places_count);
    let media_imported = store.media.len().saturating_sub(initial_media_count);
    let media_missing_count = store.media.iter().filter(|m| m.is_missing).count();
    let notes_imported = store.notes.len().saturating_sub(initial_notes_count);

    dialect.coordinates_normalized = dialect.normalized_coordinates_count;
    dialect.uids_preserved = dialect.preserved_uids_count;
    dialect.vendor_upd_filtered = dialect.filtered_upd_timestamps_count;
    dialect.parental_asso_deduplicated = dialect.deduplicated_parental_associations_count;
    dialect.missing_media_stubs = media_missing_count;

    if media_missing_count > 0 && warnings.is_empty() {
        warnings.push(format!(
            "{media_missing_count} external media paths have no bundled binary; media asset stubs will be created for post-import reconciliation."
        ));
    }

    let import_tx_id = Uuid::new_v4().to_string();
    store.change_log.push(ChangeLogEntryDto {
        id: Uuid::new_v4().to_string(),
        tx_id: import_tx_id,
        tx_label: "gedcom.import".to_string(),
        op: "gedcom_import".to_string(),
        action: "gedcom_import".to_string(),
        entity_type: "tree".to_string(),
        entity_id: "root".to_string(),
        entity_label: Some("GEDCOM Import".to_string()),
        actor_id: None,
        actor_name: None,
        summary: format!(
            "Imported GEDCOM ({} individuals, {} families, {} media)",
            individuals_imported, families_imported, media_imported
        ),
        before_json: None,
        after_json: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(GedcomImportResult {
        status: "ok".to_string(),
        operation: "gedcom_import".to_string(),
        message: format!(
            "Imported {} individuals, {} families, and {} media items",
            individuals_imported, families_imported, media_imported
        ),
        mode,
        snapshot_id,
        imported_individuals: individuals_imported,
        imported_families: families_imported,
        imported_events: events_imported,
        imported_places: places_imported,
        imported_media: media_imported,
        missing_media_count: media_missing_count,
        prompt_home_person: individuals_imported > 0,
        individuals_imported,
        families_imported,
        events_imported,
        places_imported,
        media_imported,
        media_missing_count,
        notes_imported,
        dialect,
        warnings,
    })
}

fn link_media_to_person(store: &mut GenealogyPrivacyStore, media_id: &str, person_id: &str) {
    if let Some(media) = store.media.iter_mut().find(|m| m.id == media_id) {
        if !media.linked_person_ids.iter().any(|id| id == person_id) {
            media.linked_person_ids.push(person_id.to_string());
        }
        if !media
            .links
            .iter()
            .any(|l| l.entity_type == "individual" && l.entity_id == person_id)
        {
            media.links.push(MediaLinkDto {
                id: Uuid::new_v4().to_string(),
                media_id: media_id.to_string(),
                entity_type: "individual".to_string(),
                entity_id: person_id.to_string(),
                role: Some("gallery".to_string()),
            });
        }
    }

    if let Some(person) = store.persons.iter_mut().find(|p| p.id == person_id) {
        if person.avatar_media_id.is_none() {
            person.avatar_media_id = Some(media_id.to_string());
            person.avatar_url = Some(format!("/api/v1/media/{media_id}/file"));
        }
    }
}

fn attach_gedcom_note(
    store: &mut GenealogyPrivacyStore,
    raw_value: &str,
    entity_type: &str,
    entity_id: &str,
    top_level_notes: &HashMap<String, String>,
    attached_top_level_notes: &mut HashSet<String>,
) {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return;
    }

    let body = if trimmed.starts_with('@') && trimmed.ends_with('@') && trimmed.len() > 2 {
        let key = trimmed.trim_matches('@').to_uppercase();
        if !attached_top_level_notes.insert(key.clone()) {
            // Avoid duplicating shared top-level boilerplate notes across dozens of records
            return;
        }
        let Some(resolved) = top_level_notes.get(&key) else {
            return;
        };
        resolved.clone()
    } else {
        trimmed.to_string()
    };

    if body.is_empty() {
        return;
    }

    store.notes.push(NoteDto {
        id: Uuid::new_v4().to_string(),
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        body,
        is_private: false,
        sort_order: store.notes.len() as i32,
        version: 1,
    });
}

#[allow(clippy::too_many_arguments)]
fn parse_and_insert_event(
    store: &mut GenealogyPrivacyStore,
    ev_node: &GedcomNode,
    individual_id: Option<&str>,
    family_id: Option<&str>,
    event_type: &str,
    dialect: &mut GedcomDialectSummary,
    top_level_notes: &HashMap<String, String>,
    attached_top_level_notes: &mut HashSet<String>,
    top_level_sources: &HashMap<String, String>,
    obje_xref_to_media_id: &HashMap<String, String>,
) {
    let raw_date = ev_node.child_value("DATE");
    let parsed_date = raw_date.as_deref().map(GenDate::parse);
    let date_sort = parsed_date.as_ref().and_then(|d| d.sort_key());
    let gedcom_date = raw_date
        .clone()
        .or_else(|| parsed_date.as_ref().map(|d| d.to_gedcom()));

    // Place & directional coordinates (`03 §2.2 #2, #3`)
    let mut place_id: Option<String> = None;
    let mut place_full_name: Option<String> = None;
    if let Some(plac_node) = ev_node.find_child("PLAC") {
        let raw_plac = plac_node.value.trim();
        if !raw_plac.is_empty() {
            let mut lat: Option<f64> = None;
            let mut lon: Option<f64> = None;

            if let Some(map_node) = plac_node.find_child("MAP") {
                if let Some(lati_str) = map_node.child_value("LATI") {
                    if let Some((v, had_dir)) = parse_gedcom_coordinate(&lati_str) {
                        lat = Some(v);
                        if had_dir {
                            dialect.normalized_coordinates_count += 1;
                        }
                    }
                }
                if let Some(long_str) = map_node.child_value("LONG") {
                    if let Some((v, had_dir)) = parse_gedcom_coordinate(&long_str) {
                        lon = Some(v);
                        if had_dir {
                            dialect.normalized_coordinates_count += 1;
                        }
                    }
                }
            }

            if let Ok(resolved_place) =
                resolve_or_create_place_chain(store, raw_plac, None, lat, lon, None)
            {
                place_id = Some(resolved_place.id);
                place_full_name = Some(raw_plac.to_string());
            } else {
                place_full_name = Some(raw_plac.to_string());
            }
        }
    }

    // Description, `CAUS` + `AGE` (`03 §2.2 #9`), and `EMAIL` (`03 §2.2 #10`)
    let mut desc_parts: Vec<String> = Vec::new();
    let inline_val = ev_node.value.trim();
    if !inline_val.is_empty() && !inline_val.eq_ignore_ascii_case("Y") {
        desc_parts.push(inline_val.to_string());
    }

    let caus_opt = ev_node.child_value("CAUS");
    let age_opt = ev_node.child_value("AGE");
    match (caus_opt, age_opt) {
        (Some(c), Some(a)) => desc_parts.push(format!("{c} (возраст: {a})")),
        (Some(c), None) => desc_parts.push(c),
        (None, Some(a)) => desc_parts.push(format!("возраст: {a}")),
        (None, None) => {}
    }

    if let Some(email) = ev_node.child_value("EMAIL") {
        desc_parts.push(email);
    }

    let description = if desc_parts.is_empty() {
        None
    } else {
        Some(desc_parts.join("; "))
    };

    let custom_type = if event_type == "custom" {
        ev_node.child_value("TYPE")
    } else {
        None
    };

    // Source citation text
    let source_text = ev_node.find_child("SOUR").and_then(|s_node| {
        let s_val = s_node.value.trim();
        let page = s_node.child_value("PAGE");
        let base = if s_val.starts_with('@') && s_val.ends_with('@') {
            let key = s_val.trim_matches('@').to_uppercase();
            top_level_sources
                .get(&key)
                .cloned()
                .unwrap_or_else(|| s_val.to_string())
        } else {
            s_val.to_string()
        };
        match (base.is_empty(), page) {
            (false, Some(p)) => Some(format!("{base}, {p}")),
            (false, None) => Some(base),
            (true, Some(p)) => Some(p),
            (true, None) => None,
        }
    });

    // Event-level NOTE
    if let Some(note_node) = ev_node.find_child("NOTE") {
        if let Some(pid) = individual_id {
            attach_gedcom_note(
                store,
                &note_node.value,
                "individual",
                pid,
                top_level_notes,
                attached_top_level_notes,
            );
        } else if let Some(fid) = family_id {
            attach_gedcom_note(
                store,
                &note_node.value,
                "family",
                fid,
                top_level_notes,
                attached_top_level_notes,
            );
        }
    }

    // Event-level OBJE
    let mut media_ids: Vec<String> = Vec::new();
    for ch in ev_node.children.iter().filter(|c| c.tag == "OBJE") {
        let obje_xref = ch.value.trim().trim_matches('@').to_uppercase();
        if let Some(mid) = obje_xref_to_media_id.get(&obje_xref).cloned() {
            media_ids.push(mid);
        }
    }

    let existing_gids: HashSet<String> = store.events.iter().map(|e| e.gid.clone()).collect();
    let ev_gid = allocate_gid('E', &mut store.next_event_gid_seq, None, &existing_gids);

    store.events.push(LifeEventDto {
        id: Uuid::new_v4().to_string(),
        gid: ev_gid,
        individual_id: individual_id.map(str::to_string),
        family_id: family_id.map(str::to_string),
        event_type: event_type.to_string(),
        custom_type,
        date: parsed_date,
        date_sort,
        gedcom_date,
        place_id,
        place_full_name,
        description,
        source_text,
        media_ids,
        is_private: false,
        version: 1,
    });
}

/// Exports `GenealogyPrivacyStore` to GEDCOM 7.0 or 5.5.1 text (`03 §2.3, §4, §6 DoD #3–#5`).
pub fn export_store_to_gedcom(
    store: &GenealogyPrivacyStore,
    req: &GedcomExportRequest,
    tree_title: &str,
) -> String {
    let format_str = req
        .version
        .as_deref()
        .or(req.format.as_deref())
        .unwrap_or("gedcom70")
        .trim()
        .to_ascii_lowercase();
    let is_gedcom_70 = !format_str.contains("551") && !format_str.contains("5.5");
    let include_media = req.include_media.unwrap_or(true);
    let redact_living = req.redact_living.unwrap_or(false);
    let include_private_notes = req.include_private_notes.unwrap_or(!redact_living);

    let mut out = String::with_capacity(64 * 1024);

    // 0 HEAD
    out.push_str("0 HEAD\n");
    out.push_str("1 SOUR Genealogy\n");
    out.push_str("2 VERS 1.0.0\n");
    out.push_str(&format!("2 NAME {}\n", tree_title.replace('\n', " ")));
    out.push_str("1 GEDC\n");
    if is_gedcom_70 {
        out.push_str("2 VERS 7.0\n");
    } else {
        out.push_str("2 VERS 5.5.1\n");
        out.push_str("2 FORM LINEAGE-LINKED\n");
        out.push_str("1 CHAR UTF-8\n");
    }

    let place_by_id: HashMap<String, _> = store.places.iter().map(|p| (p.id.clone(), p)).collect();
    let media_by_id: HashMap<String, _> = store.media.iter().map(|m| (m.id.clone(), m)).collect();

    // Filter & optionally redact individuals
    let mut exported_persons: Vec<TreePersonNodeDto> = Vec::new();
    for person in &store.persons {
        if redact_living {
            if let Some(redacted) =
                redact_person_for_tree_view(person, &CallerAccess::AnonymousPublic)
            {
                exported_persons.push(redacted);
            }
        } else {
            exported_persons.push(person.clone());
        }
    }

    let person_id_to_gid: HashMap<String, String> = exported_persons
        .iter()
        .map(|p| (p.id.clone(), p.gid.clone()))
        .collect();
    let person_is_redacted_living: HashSet<String> = if redact_living {
        exported_persons
            .iter()
            .filter(|p| p.is_living)
            .map(|p| p.id.clone())
            .collect()
    } else {
        HashSet::new()
    };

    // Map person_id -> FAMC and FAMS GIDs
    let mut person_famc: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut person_fams: HashMap<String, Vec<String>> = HashMap::new();

    for fam in &store.families {
        if redact_living && fam.is_private {
            continue;
        }
        if let Some(ref p1) = fam.partner1_id {
            if person_id_to_gid.contains_key(p1) {
                person_fams
                    .entry(p1.clone())
                    .or_default()
                    .push(fam.gid.clone());
            }
        }
        if let Some(ref p2) = fam.partner2_id {
            if person_id_to_gid.contains_key(p2) {
                person_fams
                    .entry(p2.clone())
                    .or_default()
                    .push(fam.gid.clone());
            }
        }
        for ch in &fam.children {
            if person_id_to_gid.contains_key(&ch.child_id) {
                person_famc
                    .entry(ch.child_id.clone())
                    .or_default()
                    .push((fam.gid.clone(), ch.rel_to_partner1.clone()));
            }
        }
    }

    // Emit 0 @Ixxxx@ INDI records
    for person in &exported_persons {
        let is_masked = person_is_redacted_living.contains(&person.id);
        out.push_str(&format!("0 @{}@ INDI\n", person.gid));

        let given_with_patr = match &person.primary_name.patronymic {
            Some(patr) if !patr.trim().is_empty() => {
                format!("{} {}", person.primary_name.given_name, patr.trim())
            }
            _ => person.primary_name.given_name.clone(),
        };

        let has_distinct_birth_surname = person
            .birth_surname
            .as_ref()
            .is_some_and(|bs| !bs.is_empty() && *bs != person.primary_name.surname);
        let has_married_alt = person
            .alt_names
            .iter()
            .any(|a| a.name_type.eq_ignore_ascii_case("married"));

        if !is_masked && (has_distinct_birth_surname || has_married_alt) {
            let b_surn = person
                .birth_surname
                .clone()
                .unwrap_or_else(|| person.primary_name.surname.clone());
            out.push_str(&format!("1 NAME {given_with_patr} /{b_surn}/\n"));
            out.push_str("2 TYPE birth\n");
            if !given_with_patr.is_empty() {
                out.push_str(&format!("2 GIVN {given_with_patr}\n"));
            }
            if !b_surn.is_empty() {
                out.push_str(&format!("2 SURN {b_surn}\n"));
            }

            if !person.primary_name.surname.is_empty() && person.primary_name.surname != b_surn {
                out.push_str(&format!(
                    "1 NAME {given_with_patr} /{}/\n",
                    person.primary_name.surname
                ));
                out.push_str("2 TYPE married\n");
                if !given_with_patr.is_empty() {
                    out.push_str(&format!("2 GIVN {given_with_patr}\n"));
                }
                out.push_str(&format!("2 SURN {}\n", person.primary_name.surname));
            }
        } else {
            out.push_str(&format!(
                "1 NAME {given_with_patr} /{}/\n",
                person.primary_name.surname
            ));
            if !is_masked && person.birth_surname.is_some() {
                out.push_str("2 TYPE birth\n");
            }
            if !given_with_patr.is_empty() {
                out.push_str(&format!("2 GIVN {given_with_patr}\n"));
            }
            if !person.primary_name.surname.is_empty() {
                out.push_str(&format!("2 SURN {}\n", person.primary_name.surname));
            }
        }

        if !is_masked {
            for alt in &person.alt_names {
                if alt.name_type.eq_ignore_ascii_case("married") {
                    continue;
                }
                let ag = alt
                    .given_name
                    .clone()
                    .unwrap_or_else(|| given_with_patr.clone());
                let asurn = alt.surname.clone().unwrap_or_default();
                out.push_str(&format!("1 NAME {ag} /{asurn}/\n"));
                out.push_str(&format!("2 TYPE {}\n", alt.name_type));
                if !ag.is_empty() {
                    out.push_str(&format!("2 GIVN {ag}\n"));
                }
                if !asurn.is_empty() {
                    out.push_str(&format!("2 SURN {asurn}\n"));
                }
            }
        }

        out.push_str(&format!("1 SEX {}\n", person.sex));

        if !is_masked {
            // Emit individual events
            for ev in store
                .events
                .iter()
                .filter(|e| e.individual_id.as_deref() == Some(person.id.as_str()))
            {
                if ev.is_private && !include_private_notes {
                    continue;
                }
                emit_gedcom_event(&mut out, ev, &place_by_id, is_gedcom_70);
            }

            // External IDs
            if let Some(rin) = person.external_ids.get("mh_rin") {
                out.push_str(&format!("1 FACT {rin}\n2 TYPE RIN\n"));
            }
            if let Some(uid) = person.external_ids.get("uid") {
                let uid_tag = if is_gedcom_70 { "UID" } else { "_UID" };
                out.push_str(&format!("1 {uid_tag} {uid}\n"));
            }
        }

        if let Some(famc_list) = person_famc.get(&person.id) {
            for (fam_gid, pedi) in famc_list {
                out.push_str(&format!("1 FAMC @{fam_gid}@\n"));
                out.push_str(&format!("2 PEDI {pedi}\n"));
            }
        }

        if let Some(fams_list) = person_fams.get(&person.id) {
            for fam_gid in fams_list {
                out.push_str(&format!("1 FAMS @{fam_gid}@\n"));
            }
        }

        if include_media && !is_masked {
            for media in &store.media {
                if media.is_private && !include_private_notes {
                    continue;
                }
                if media.linked_person_ids.iter().any(|pid| pid == &person.id) {
                    out.push_str(&format!("1 OBJE @{}@\n", media.gid));
                }
            }
        }

        if !is_masked {
            for note in store
                .notes
                .iter()
                .filter(|n| n.entity_type == "individual" && n.entity_id == person.id)
            {
                if note.is_private && !include_private_notes {
                    continue;
                }
                emit_multiline_tag(&mut out, 1, "NOTE", &note.body);
            }
        }
    }

    // Emit 0 @Fxxxx@ FAM records
    for fam in &store.families {
        if redact_living && fam.is_private {
            continue;
        }
        let p1_gid = fam
            .partner1_id
            .as_ref()
            .and_then(|id| person_id_to_gid.get(id));
        let p2_gid = fam
            .partner2_id
            .as_ref()
            .and_then(|id| person_id_to_gid.get(id));
        let visible_children: Vec<&FamilyChildDto> = fam
            .children
            .iter()
            .filter(|c| person_id_to_gid.contains_key(&c.child_id))
            .collect();

        if p1_gid.is_none() && p2_gid.is_none() && visible_children.is_empty() {
            continue;
        }

        out.push_str(&format!("0 @{}@ FAM\n", fam.gid));
        if let Some(g1) = p1_gid {
            out.push_str(&format!("1 HUSB @{g1}@\n"));
        }
        if let Some(g2) = p2_gid {
            out.push_str(&format!("1 WIFE @{g2}@\n"));
        }

        let fam_events: Vec<&LifeEventDto> = store
            .events
            .iter()
            .filter(|e| e.family_id.as_deref() == Some(fam.id.as_str()))
            .filter(|e| !e.is_private || include_private_notes)
            .collect();

        let has_marr_event = fam_events.iter().any(|e| e.event_type == "marriage");
        for ev in &fam_events {
            emit_gedcom_event(&mut out, ev, &place_by_id, is_gedcom_70);
        }
        if fam.relationship_type == "married" && !has_marr_event {
            out.push_str("1 MARR Y\n");
        }

        if let Some(rin) = fam.external_ids.get("mh_rin") {
            out.push_str(&format!("1 FACT {rin}\n2 TYPE RIN\n"));
        }
        if let Some(uid) = fam.external_ids.get("uid") {
            let uid_tag = if is_gedcom_70 { "UID" } else { "_UID" };
            out.push_str(&format!("1 {uid_tag} {uid}\n"));
        }

        for ch in visible_children {
            if let Some(ch_gid) = person_id_to_gid.get(&ch.child_id) {
                out.push_str(&format!("1 CHIL @{ch_gid}@\n"));
            }
        }

        for note in store
            .notes
            .iter()
            .filter(|n| n.entity_type == "family" && n.entity_id == fam.id)
        {
            if note.is_private && !include_private_notes {
                continue;
            }
            emit_multiline_tag(&mut out, 1, "NOTE", &note.body);
        }
    }

    // Emit 0 @Oxxxx@ OBJE records
    if include_media {
        for media in &store.media {
            if media.is_private && !include_private_notes {
                continue;
            }
            if redact_living && media.only_living_persons {
                continue;
            }
            let _ = media_by_id.get(&media.id);
            out.push_str(&format!("0 @{}@ OBJE\n", media.gid));
            let file_path = media
                .external_path
                .as_deref()
                .or(media.original_file_name.as_deref())
                .or(media.file_name.as_deref())
                .unwrap_or(&media.file_url);
            out.push_str(&format!("1 FILE {file_path}\n"));
            let form_ext = match media.mime_type.as_str() {
                "image/png" => "png",
                "image/webp" => "webp",
                "image/gif" => "gif",
                "application/pdf" => "pdf",
                _ => "jpg",
            };
            out.push_str(&format!("2 FORM {form_ext}\n"));
            if !media.title.is_empty() {
                out.push_str(&format!("2 TITL {}\n", media.title.replace('\n', " ")));
            }
        }
    }

    out.push_str("0 TRLR\n");
    out
}

fn emit_gedcom_event(
    out: &mut String,
    ev: &LifeEventDto,
    place_by_id: &HashMap<String, &crate::services::privacy::PlaceSummaryDto>,
    is_gedcom_70: bool,
) {
    let tag = match ev.event_type.as_str() {
        "birth" => "BIRT",
        "baptism" => "CHR",
        "death" => "DEAT",
        "burial" => "BURI",
        "cremation" => "CREM",
        "adoption" => "ADOP",
        "occupation" => "OCCU",
        "education" => "EDUC",
        "graduation" => "GRAD",
        "residence" => "RESI",
        "immigration" => "IMMI",
        "emigration" => "EMIG",
        "naturalization" => "NATU",
        "census" => "CENS",
        "military_service" => "MILI",
        "retirement" => "RETI",
        "marriage" => "MARR",
        "marriage_license" => "MARL",
        "divorce" => "DIV",
        "engagement" => "ENGA",
        _ => "EVEN",
    };

    let date_str = ev
        .gedcom_date
        .clone()
        .or_else(|| ev.date.as_ref().map(|d| d.to_gedcom()))
        .filter(|s| !s.trim().is_empty());

    let has_details = date_str.is_some()
        || ev.place_full_name.is_some()
        || ev.description.is_some()
        || ev.custom_type.is_some();

    if tag == "DEAT" && !has_details {
        out.push_str("1 DEAT Y\n");
        return;
    }

    let inline_desc = ev
        .description
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty() && tag != "DEAT" && !(tag == "RESI" && t.contains('@')));

    if let Some(inline_text) = inline_desc {
        out.push_str(&format!("1 {tag} {}\n", inline_text.replace('\n', " ")));
    } else {
        out.push_str(&format!("1 {tag}\n"));
    }
    if let Some(ref ct) = ev.custom_type {
        if !ct.trim().is_empty() {
            out.push_str(&format!("2 TYPE {}\n", ct.trim()));
        }
    }
    if let Some(ref ds) = date_str {
        out.push_str(&format!("2 DATE {ds}\n"));
    }
    if let Some(ref plac) = ev.place_full_name {
        if !plac.trim().is_empty() {
            out.push_str(&format!("2 PLAC {}\n", plac.trim()));
            if let Some(ref pid) = ev.place_id {
                if let Some(place) = place_by_id.get(pid) {
                    if let (Some(lat), Some(lon)) = (place.latitude, place.longitude) {
                        out.push_str("3 MAP\n");
                        out.push_str(&format!("4 LATI {}\n", format_gedcom_latitude(lat)));
                        out.push_str(&format!("4 LONG {}\n", format_gedcom_longitude(lon)));
                    }
                }
            }
        }
    }

    if let Some(ref desc) = ev.description {
        let trimmed = desc.trim();
        if tag == "RESI" && trimmed.contains('@') {
            let escaped = if is_gedcom_70 {
                trimmed.to_string()
            } else {
                trimmed.replace('@', "@@")
            };
            out.push_str(&format!("2 EMAIL {escaped}\n"));
        } else if tag == "DEAT" {
            if let Some((caus_part, age_part)) = trimmed.rsplit_once(" (возраст: ") {
                let age_clean = age_part.trim_end_matches(')');
                if !caus_part.trim().is_empty() {
                    out.push_str(&format!("2 CAUS {}\n", caus_part.trim()));
                }
                if !age_clean.trim().is_empty() {
                    out.push_str(&format!("2 AGE {}\n", age_clean.trim()));
                }
            } else if let Some(age_only) = trimmed.strip_prefix("возраст: ") {
                out.push_str(&format!("2 AGE {}\n", age_only.trim()));
            } else {
                out.push_str(&format!("2 CAUS {trimmed}\n"));
            }
        }
    }
}

fn emit_multiline_tag(out: &mut String, level: u8, tag: &str, text: &str) {
    let mut lines = text.lines();
    if let Some(first) = lines.next() {
        out.push_str(&format!("{level} {tag} {first}\n"));
        let cont_level = level + 1;
        for line in lines {
            out.push_str(&format!("{cont_level} CONT {line}\n"));
        }
    }
}
