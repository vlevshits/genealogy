#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone)]
pub struct RenderPersonCard {
    pub id: String,
    pub gid: String,
    pub full_name: String,
    pub lifespan: String,
    pub is_living: bool,
    pub is_focal: bool,
    pub sex: String,
    pub x: f64, // mm from left
    pub y: f64, // mm from top
    pub width: f64,
    pub height: f64,
    pub avatar_bytes: Option<Vec<u8>>,
    pub generation: i32,
}

#[derive(Debug, Clone)]
pub struct RenderConnector {
    pub points: Vec<(f64, f64)>, // (x, y) coordinates in mm from top-left
    pub is_dashed: bool,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CanvasLayout {
    pub width_mm: f64,
    pub height_mm: f64,
    pub title: String,
    pub subtitle: String,
    pub cards: Vec<RenderPersonCard>,
    pub connectors: Vec<RenderConnector>,
    pub total_persons: usize,
}

pub const CARD_WIDTH_MM: f64 = 56.0;
pub const CARD_HEIGHT_MM: f64 = 26.0;
pub const HORIZONTAL_GAP_MM: f64 = 16.0;
pub const VERTICAL_GAP_MM: f64 = 12.0;
pub const BANNER_HEIGHT_MM: f64 = 32.0;
pub const MARGIN_X_MM: f64 = 20.0;
pub const MARGIN_Y_MM: f64 = 20.0;
pub const MARITAL_OCHRE: &str = "#d97706";

/// Formats a person's primary name into a display string.
pub(crate) fn format_person_name(person: &TreePersonNodeDto) -> String {
    let given = person.primary_name.given_name.trim();
    let patronymic = person
        .primary_name
        .patronymic
        .as_deref()
        .unwrap_or("")
        .trim();
    let surname = person.primary_name.surname.trim();

    if !patronymic.is_empty() {
        if !surname.is_empty() {
            format!("{given} {patronymic} {surname}")
        } else {
            format!("{given} {patronymic}")
        }
    } else if !surname.is_empty() {
        format!("{given} {surname}")
    } else if !given.is_empty() {
        given.to_string()
    } else {
        person.gid.clone()
    }
}

/// Formats a person's lifespan (e.g. "1920 – 1985" or "b. 1990").
/// If living and redacted, returns "Living" or masked placeholder.
pub(crate) fn format_person_lifespan(
    person: &TreePersonNodeDto,
    redact_living: bool,
    is_ru: bool,
) -> String {
    if person.is_living {
        if redact_living {
            return if is_ru {
                "Живущий".to_string()
            } else {
                "Living".to_string()
            };
        }
        if let Some(by) = extract_birth_year(person) {
            return if is_ru {
                format!("р. {by}")
            } else {
                format!("b. {by}")
            };
        }
        return if is_ru {
            "Живущий".to_string()
        } else {
            "Living".to_string()
        };
    }

    let birth = extract_birth_year(person);
    let death = extract_death_year(person);

    match (birth, death) {
        (Some(b), Some(d)) => format!("{b} – {d}"),
        (Some(b), None) => {
            if is_ru {
                format!("р. {b}")
            } else {
                format!("b. {b}")
            }
        }
        (None, Some(d)) => {
            if is_ru {
                format!("ум. {d}")
            } else {
                format!("d. {d}")
            }
        }
        (None, None) => String::new(),
    }
}

pub(crate) fn sanitize_year(val: i32) -> i32 {
    let abs = val.abs();
    if abs > 9999 {
        val / 10000
    } else {
        val
    }
}

pub(crate) fn extract_year_from_str(s: &str) -> Option<i32> {
    for word in s.split(|c: char| !c.is_numeric()) {
        if word.len() == 4 {
            if let Ok(y) = word.parse::<i32>() {
                return Some(y);
            }
        } else if word.len() == 8 {
            if let Ok(y) = word[..4].parse::<i32>() {
                return Some(y);
            }
        }
    }
    None
}

pub(crate) fn extract_birth_year(person: &TreePersonNodeDto) -> Option<i32> {
    person.birth_sort.map(sanitize_year).or_else(|| {
        person
            .birth
            .as_ref()
            .and_then(|b| b.date.as_deref())
            .and_then(extract_year_from_str)
    })
}

pub(crate) fn extract_death_year(person: &TreePersonNodeDto) -> Option<i32> {
    person.death_sort.map(sanitize_year).or_else(|| {
        person
            .death
            .as_ref()
            .and_then(|d| d.date.as_deref())
            .and_then(extract_year_from_str)
    })
}

/// Tries to resolve portrait avatar image bytes for a person from storage or data URL.
pub(crate) fn resolve_person_avatar(
    person: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
    media_storage_path: Option<&str>,
    redact_living: bool,
    show_photos: bool,
) -> Option<Vec<u8>> {
    if !show_photos || (person.is_living && redact_living) {
        return None;
    }

    // 1. Check data URL in avatar_url
    if let Some(ref url) = person.avatar_url {
        if let Some(comma_pos) = url.find(',') {
            if url.starts_with("data:image/") && url.contains(";base64,") {
                use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
                let b64_str = &url[comma_pos + 1..];
                if let Ok(bytes) = BASE64.decode(b64_str.trim()) {
                    return Some(bytes);
                }
            }
        }
    }

    // 2. Check avatar_media_id or extracted media id from avatar_url in storage
    if let Some(storage_dir) = media_storage_path {
        let media_id = person.avatar_media_id.as_deref().or_else(|| {
            person.avatar_url.as_deref().and_then(|url| {
                if let Some(rest) = url.strip_prefix("/api/v1/media/") {
                    rest.split('/').next()
                } else {
                    None
                }
            })
        });

        if let Some(mid) = media_id {
            if let Some(media) = store.media.iter().find(|m| m.id == mid) {
                if let Some(ref fn_str) = media.file_name {
                    let full_path = Path::new(storage_dir).join(fn_str);
                    if let Ok(bytes) = std::fs::read(full_path) {
                        return Some(bytes);
                    }
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Kinship Graph Helpers
// ---------------------------------------------------------------------------

pub(crate) fn find_parents(
    child_id: &str,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
) -> (Option<String>, Option<String>) {
    for fam in &store.families {
        if fam.children.iter().any(|c| c.child_id == child_id) {
            let p1 = fam
                .partner1_id
                .as_deref()
                .and_then(|id| person_by_id.get(id))
                .copied();
            let p2 = fam
                .partner2_id
                .as_deref()
                .and_then(|id| person_by_id.get(id))
                .copied();

            match (p1, p2) {
                (Some(a), Some(b)) => {
                    if a.sex.eq_ignore_ascii_case("M") || b.sex.eq_ignore_ascii_case("F") {
                        return (Some(a.id.clone()), Some(b.id.clone()));
                    } else {
                        return (Some(b.id.clone()), Some(a.id.clone()));
                    }
                }
                (Some(a), None) => return (Some(a.id.clone()), None),
                (None, Some(b)) => return (None, Some(b.id.clone())),
                (None, None) => {}
            }
        }
    }
    (None, None)
}

pub(crate) fn find_spouses(
    person_id: &str,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
) -> Vec<String> {
    let mut spouses = Vec::new();
    for fam in &store.families {
        let is_p1 = fam.partner1_id.as_deref() == Some(person_id);
        let is_p2 = fam.partner2_id.as_deref() == Some(person_id);
        if is_p1 {
            if let Some(ref sid) = fam.partner2_id {
                if person_by_id.contains_key(sid.as_str()) && !spouses.contains(sid) {
                    spouses.push(sid.clone());
                }
            }
        } else if is_p2 {
            if let Some(ref sid) = fam.partner1_id {
                if person_by_id.contains_key(sid.as_str()) && !spouses.contains(sid) {
                    spouses.push(sid.clone());
                }
            }
        }
    }
    spouses
}

pub(crate) fn find_children_of_person(
    parent_id: &str,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
) -> Vec<String> {
    let mut ch_ids = Vec::new();
    for fam in &store.families {
        let is_p1 = fam.partner1_id.as_deref() == Some(parent_id);
        let is_p2 = fam.partner2_id.as_deref() == Some(parent_id);
        if is_p1 || is_p2 {
            for ch in &fam.children {
                if person_by_id.contains_key(ch.child_id.as_str()) && !ch_ids.contains(&ch.child_id)
                {
                    ch_ids.push(ch.child_id.clone());
                }
            }
        }
    }
    ch_ids
}

pub(crate) fn find_siblings_of_person(
    person_id: &str,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
) -> Vec<String> {
    let mut sib_ids = Vec::new();
    for fam in &store.families {
        if fam.children.iter().any(|c| c.child_id == person_id) {
            for ch in &fam.children {
                if ch.child_id != person_id
                    && person_by_id.contains_key(ch.child_id.as_str())
                    && !sib_ids.contains(&ch.child_id)
                {
                    sib_ids.push(ch.child_id.clone());
                }
            }
        }
    }
    sib_ids
}

// ---------------------------------------------------------------------------
// Main Entry: compute_tree_layout
// ---------------------------------------------------------------------------

/// Computes the complete physical layout of the family tree canvas.
#[allow(clippy::too_many_arguments)]
pub fn compute_tree_layout(
    store: &GenealogyPrivacyStore,
    focal_id_or_gid: &str,
    view_mode: &str,
    generations: u32,
    redact_living: bool,
    show_photos: bool,
    custom_title: Option<&str>,
    custom_subtitle: Option<&str>,
    include_partner: bool,
    include_siblings: bool,
    collateral: bool,
    instance_tree_title: &str,
    media_storage_path: Option<&str>,
) -> Option<CanvasLayout> {
    // 1. Resolve focal person
    let focal = store
        .persons
        .iter()
        .find(|p| p.id == focal_id_or_gid || p.gid.eq_ignore_ascii_case(focal_id_or_gid))?;

    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();

    let focal_name = format_person_name(focal);
    let title = custom_title
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            if !instance_tree_title.trim().is_empty() {
                format!("{instance_tree_title}: {focal_name}")
            } else {
                format!("Family Tree of {focal_name}")
            }
        });

    let is_ru = focal_name
        .chars()
        .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));

    let mode = match view_mode.trim().to_ascii_lowercase().as_str() {
        "pedigree" | "fan" => "pedigree",
        "descendant" | "descendants" => "descendant",
        "all_nodes" | "all" => "all_nodes",
        _ => "hourglass",
    };

    let gens = generations.clamp(1, 8);

    let (cards, connectors) = match mode {
        "pedigree" => layout_pedigree(
            focal,
            store,
            &person_by_id,
            gens,
            redact_living,
            show_photos,
            is_ru,
            media_storage_path,
            include_partner,
            include_siblings,
        ),
        "descendant" => layout_descendant(
            focal,
            store,
            &person_by_id,
            gens,
            redact_living,
            show_photos,
            is_ru,
            media_storage_path,
            include_partner,
            include_siblings,
        ),
        "all_nodes" => layout_all_nodes(
            focal,
            store,
            &person_by_id,
            redact_living,
            show_photos,
            is_ru,
            media_storage_path,
            include_partner,
        ),
        _ => layout_hourglass(
            focal,
            store,
            &person_by_id,
            gens,
            redact_living,
            show_photos,
            is_ru,
            media_storage_path,
            include_partner,
            include_siblings,
            collateral,
        ),
    };

    if cards.is_empty() {
        return None;
    }

    // Determine canvas dimensions
    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;

    for card in &cards {
        let right = card.x + card.width;
        let bottom = card.y + card.height;
        if right > max_x {
            max_x = right;
        }
        if bottom > max_y {
            max_y = bottom;
        }
    }

    for conn in &connectors {
        for (cx, cy) in &conn.points {
            if *cx > max_x {
                max_x = *cx;
            }
            if *cy > max_y {
                max_y = *cy;
            }
        }
    }

    // Minimum canvas is at least A4 landscape: 297 x 210 mm
    let total_w = (max_x + MARGIN_X_MM).max(297.0);
    let total_h = (max_y + MARGIN_Y_MM).max(210.0);

    let now_str = Utc::now().format("%Y-%m-%d").to_string();
    let subtitle = if let Some(sub) = custom_subtitle.map(str::trim).filter(|s| !s.is_empty()) {
        sub.to_string()
    } else if mode == "all_nodes" {
        if is_ru {
            format!("Всего персон: {}", cards.len())
        } else {
            format!("Total Individuals: {}", cards.len())
        }
    } else if is_ru {
        format!(
            "Режим: {} • Поколений: {} • Персон: {} • {}",
            mode,
            gens,
            cards.len(),
            now_str
        )
    } else {
        format!(
            "Layout: {} • Generations: {} • Persons: {} • {}",
            mode,
            gens,
            cards.len(),
            now_str
        )
    };

    let total_persons = cards.len();
    Some(CanvasLayout {
        width_mm: total_w,
        height_mm: total_h,
        title,
        subtitle,
        cards,
        connectors,
        total_persons,
    })
}
