#[allow(unused_imports)]
use super::*;

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

pub(crate) fn format_coord_number(abs_val: f64) -> String {
    let s = format!("{abs_val:.6}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn looks_like_slavic_patronymic(token: &str) -> bool {
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
pub(crate) struct ParsedGedcomName {
    pub(crate) name_type: Option<String>,

    pub(crate) given_name: String,

    pub(crate) patronymic: Option<String>,

    pub(crate) surname: String,

    pub(crate) married_surname: Option<String>,

    pub(crate) prefix: Option<String>,

    pub(crate) suffix: Option<String>,

    pub(crate) nickname: Option<String>,
}

pub(crate) fn parse_gedcom_name_node(node: &GedcomNode) -> ParsedGedcomName {
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

    let mut married_surname = marnm_sub;
    if type_sub.is_none() && married_surname.is_none() {
        if let Some((outer, rest)) = surname.split_once('(') {
            if let Some(inner) = rest.strip_suffix(')') {
                let outer_clean = outer.trim();
                let inner_clean = inner.trim();
                let both_non_ascii = !outer_clean.is_empty()
                    && !inner_clean.is_empty()
                    && !outer_clean.is_ascii()
                    && !inner_clean.is_ascii()
                    && !surnames_match_normalized(outer_clean, inner_clean);
                if both_non_ascii {
                    married_surname = Some(outer_clean.to_string());
                    surname = inner_clean.to_string();
                }
            }
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
        } else if parts.len() >= 3
            && !parts.last().unwrap().starts_with('(')
            && looks_like_slavic_patronymic(parts.last().unwrap())
        {
            (
                parts[..parts.len() - 1].join(" "),
                Some((*parts.last().unwrap()).to_string()),
            )
        } else {
            (raw_given.trim().to_string(), None)
        }
    };

    ParsedGedcomName {
        name_type: type_sub,
        given_name,
        patronymic,
        surname,
        married_surname,
        prefix: npfx_sub,
        suffix: nsfx_sub,
        nickname: nick_sub,
    }
}

pub(crate) fn infer_mime_type_from_form_or_path(form: Option<&str>, path: &str) -> String {
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

pub(crate) fn strip_media_directory_path(raw_path: &str) -> (String, bool) {
    let trimmed = raw_path.trim();
    if let Some((_, file_part)) = trimmed.rsplit_once(['/', '\\']) {
        if !file_part.is_empty() {
            return (file_part.to_string(), true);
        }
    }
    (trimmed.to_string(), false)
}

#[derive(Debug, Clone)]
pub(crate) struct PendingAssociation {
    pub(crate) person_id: String,

    pub(crate) target_xref: String,

    pub(crate) rela: String,
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
