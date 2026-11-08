#[allow(unused_imports)]
use super::*;

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
    pub merged_individuals: usize,
    #[serde(default)]
    pub merged_families: usize,
    #[serde(default)]
    pub matched_by_uid: usize,
    #[serde(default)]
    pub matched_by_name_dates: usize,
    #[serde(default)]
    pub matched_by_kinship: usize,
    #[serde(default)]
    pub chronology_conflicts_prevented: usize,
    #[serde(default)]
    pub matched_keys_preview: Vec<String>,
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

pub type GedcomImportReportDto = GedcomImportResult;

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

/// Rejoins multi-byte UTF-8 characters that MyHeritage or other exporters split across a
/// line break and a `N CONC ` continuation tag (`...Белару\xD1\r\n4 CONC \x81ь...`).
pub(crate) fn rejoin_utf8_conc_splits(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        out.push(b);
        i += 1;

        if b >= 0x80 && i < bytes.len() && (bytes[i] == b'\r' || bytes[i] == b'\n') {
            let mut j = i;
            if j < bytes.len() && bytes[j] == b'\r' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'\n' {
                j += 1;
                while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                    j += 1;
                }
                let digits_start = j;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > digits_start {
                    let spaces_start = j;
                    while j < bytes.len() && bytes[j] == b' ' {
                        j += 1;
                    }
                    if j > spaces_start
                        && j + 4 <= bytes.len()
                        && bytes[j..j + 4].eq_ignore_ascii_case(b"CONC")
                    {
                        j += 4;
                        if j < bytes.len() && bytes[j] == b' ' {
                            j += 1;
                        }
                        if j < bytes.len() && (0x80..=0xBF).contains(&bytes[j]) {
                            i = j;
                        }
                    }
                }
            }
        }
    }
    out
}

/// Detects character encoding and decodes raw GEDCOM bytes (`03 §2.1 #2, §6 DoD #2`).
/// Supports UTF-8 (with or without BOM), UTF-16LE/BE, Windows-1251, Windows-1252, and ANSEL.
pub fn decode_gedcom_bytes(bytes: &[u8]) -> (String, String) {
    if bytes.is_empty() {
        return (String::new(), "UTF-8".to_string());
    }

    // 1. UTF-8 BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let rejoined = rejoin_utf8_conc_splits(&bytes[3..]);
        let text = String::from_utf8_lossy(&rejoined).into_owned();
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
    let explicit_char = ascii_head.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("1 CHAR ")
            .map(|v| v.trim().to_ascii_uppercase())
    });
    let declared_char = explicit_char.clone().unwrap_or_else(|| "UTF-8".to_string());

    if declared_char == "ANSEL" {
        return (decode_ansel(bytes), "ANSEL".to_string());
    }

    let rejoined = rejoin_utf8_conc_splits(bytes);
    if let Ok(utf8_str) = std::str::from_utf8(&rejoined) {
        return (utf8_str.to_string(), declared_char);
    }

    if explicit_char.as_deref().is_some_and(|c| c.contains("UTF")) {
        return (
            String::from_utf8_lossy(&rejoined).into_owned(),
            declared_char,
        );
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

pub(crate) fn decode_windows_1252(bytes: &[u8]) -> String {
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

pub(crate) fn decode_windows_1251(bytes: &[u8]) -> String {
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
pub(crate) fn decode_ansel(bytes: &[u8]) -> String {
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

pub(crate) fn compose_ansel_diacritic(diacritic: u8, base: char) -> String {
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

pub(crate) fn unescape_gedcom_value(raw: &str) -> String {
    if raw.contains("@@") {
        raw.replace("@@", "@")
    } else {
        raw.to_string()
    }
}
