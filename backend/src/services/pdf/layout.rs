use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use chrono::Utc;

use crate::services::privacy::{GenealogyPrivacyStore, TreePersonNodeDto};

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
fn format_person_name(person: &TreePersonNodeDto) -> String {
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
fn format_person_lifespan(person: &TreePersonNodeDto, redact_living: bool, is_ru: bool) -> String {
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

fn sanitize_year(val: i32) -> i32 {
    let abs = val.abs();
    if abs > 9999 {
        val / 10000
    } else {
        val
    }
}

fn extract_year_from_str(s: &str) -> Option<i32> {
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

fn extract_birth_year(person: &TreePersonNodeDto) -> Option<i32> {
    person.birth_sort.map(sanitize_year).or_else(|| {
        person
            .birth
            .as_ref()
            .and_then(|b| b.date.as_deref())
            .and_then(extract_year_from_str)
    })
}

fn extract_death_year(person: &TreePersonNodeDto) -> Option<i32> {
    person.death_sort.map(sanitize_year).or_else(|| {
        person
            .death
            .as_ref()
            .and_then(|d| d.date.as_deref())
            .and_then(extract_year_from_str)
    })
}

/// Tries to resolve portrait avatar image bytes for a person from storage or data URL.
fn resolve_person_avatar(
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

fn find_parents(
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

fn find_spouses(
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

fn find_children_of_person(
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

fn find_siblings_of_person(
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

// ---------------------------------------------------------------------------
// 1. Pedigree Layout (Left-to-Right Binary Ancestors)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn layout_pedigree(
    focal: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    max_generations: u32,
    redact_living: bool,
    show_photos: bool,
    is_ru: bool,
    media_storage_path: Option<&str>,
    include_partner: bool,
    include_siblings: bool,
) -> (Vec<RenderPersonCard>, Vec<RenderConnector>) {
    let mut cards: Vec<RenderPersonCard> = Vec::new();
    let mut connectors: Vec<RenderConnector> = Vec::new();
    let mut leaf_y_counter: f64 = MARGIN_Y_MM + BANNER_HEIGHT_MM;

    struct NodePlacement {
        person_id: String,
        generation: u32,
        x: f64,
        y: f64,
        father_pos: Option<(f64, f64)>,
        mother_pos: Option<(f64, f64)>,
    }

    fn place_ancestor_node(
        person_id: &str,
        generation: u32,
        max_gen: u32,
        store: &GenealogyPrivacyStore,
        person_by_id: &HashMap<&str, &TreePersonNodeDto>,
        leaf_counter: &mut f64,
        placements: &mut Vec<NodePlacement>,
    ) -> (f64, f64) {
        let x = MARGIN_X_MM + (generation as f64) * (CARD_WIDTH_MM + HORIZONTAL_GAP_MM);

        if generation + 1 >= max_gen {
            let y = *leaf_counter;
            *leaf_counter += CARD_HEIGHT_MM + VERTICAL_GAP_MM;
            placements.push(NodePlacement {
                person_id: person_id.to_string(),
                generation,
                x,
                y,
                father_pos: None,
                mother_pos: None,
            });
            return (x, y);
        }

        let (father_id, mother_id) = find_parents(person_id, store, person_by_id);

        let father_res = father_id.as_deref().map(|fid| {
            place_ancestor_node(
                fid,
                generation + 1,
                max_gen,
                store,
                person_by_id,
                leaf_counter,
                placements,
            )
        });

        let mother_res = mother_id.as_deref().map(|mid| {
            place_ancestor_node(
                mid,
                generation + 1,
                max_gen,
                store,
                person_by_id,
                leaf_counter,
                placements,
            )
        });

        let y = match (father_res, mother_res) {
            (Some((_, fy)), Some((_, my))) => (fy + my) / 2.0,
            (Some((_, fy)), None) => fy,
            (None, Some((_, my))) => my,
            (None, None) => {
                let curr_y = *leaf_counter;
                *leaf_counter += CARD_HEIGHT_MM + VERTICAL_GAP_MM;
                curr_y
            }
        };

        placements.push(NodePlacement {
            person_id: person_id.to_string(),
            generation,
            x,
            y,
            father_pos: father_res,
            mother_pos: mother_res,
        });

        (x, y)
    }

    let mut placements: Vec<NodePlacement> = Vec::new();
    place_ancestor_node(
        &focal.id,
        0,
        max_generations,
        store,
        person_by_id,
        &mut leaf_y_counter,
        &mut placements,
    );

    let mut rendered_ids: HashSet<String> = HashSet::new();

    for p in &placements {
        if rendered_ids.contains(&p.person_id) {
            continue;
        }
        rendered_ids.insert(p.person_id.clone());

        if let Some(person) = person_by_id.get(p.person_id.as_str()) {
            let is_focal = person.id == focal.id;
            let full_name = format_person_name(person);
            let lifespan = format_person_lifespan(person, redact_living, is_ru);
            let avatar_bytes = resolve_person_avatar(
                person,
                store,
                media_storage_path,
                redact_living,
                show_photos,
            );

            cards.push(RenderPersonCard {
                id: person.id.clone(),
                gid: person.gid.clone(),
                full_name,
                lifespan,
                is_living: person.is_living,
                is_focal,
                sex: person.sex.clone(),
                x: p.x,
                y: p.y,
                width: CARD_WIDTH_MM,
                height: CARD_HEIGHT_MM,
                avatar_bytes,
                generation: p.generation as i32,
            });

            // Connectors from child to parents
            let child_right_x = p.x + CARD_WIDTH_MM;
            let child_mid_y = p.y + (CARD_HEIGHT_MM / 2.0);
            let conn_mid_x = child_right_x + (HORIZONTAL_GAP_MM / 2.0);

            if let Some((_, fy)) = p.father_pos {
                let father_mid_y = fy + (CARD_HEIGHT_MM / 2.0);
                connectors.push(RenderConnector {
                    points: vec![
                        (child_right_x, child_mid_y),
                        (conn_mid_x, child_mid_y),
                        (conn_mid_x, father_mid_y),
                        (child_right_x + HORIZONTAL_GAP_MM, father_mid_y),
                    ],
                    is_dashed: false,
                    color: None,
                });
            }

            if let Some((_, my)) = p.mother_pos {
                let mother_mid_y = my + (CARD_HEIGHT_MM / 2.0);
                connectors.push(RenderConnector {
                    points: vec![
                        (child_right_x, child_mid_y),
                        (conn_mid_x, child_mid_y),
                        (conn_mid_x, mother_mid_y),
                        (child_right_x + HORIZONTAL_GAP_MM, mother_mid_y),
                    ],
                    is_dashed: false,
                    color: None,
                });
            }
        }
    }

    // Partner placement for Pedigree (placed below focal at X = MARGIN_X_MM)
    if include_partner {
        if let Some(focal_card) = cards.iter().find(|c| c.id == focal.id).cloned() {
            let spouses = find_spouses(&focal.id, store, person_by_id);
            let mut prev_bottom_y = focal_card.y + CARD_HEIGHT_MM;

            for (idx, sid) in spouses.iter().enumerate() {
                if rendered_ids.contains(sid) {
                    continue;
                }
                if let Some(spouse) = person_by_id.get(sid.as_str()) {
                    rendered_ids.insert(spouse.id.clone());
                    let sy = focal_card.y + (idx as f64 + 1.0) * (CARD_HEIGHT_MM + 8.0);
                    let full_name = format_person_name(spouse);
                    let lifespan = format_person_lifespan(spouse, redact_living, is_ru);
                    let avatar_bytes = resolve_person_avatar(
                        spouse,
                        store,
                        media_storage_path,
                        redact_living,
                        show_photos,
                    );

                    cards.push(RenderPersonCard {
                        id: spouse.id.clone(),
                        gid: spouse.gid.clone(),
                        full_name,
                        lifespan,
                        is_living: spouse.is_living,
                        is_focal: false,
                        sex: spouse.sex.clone(),
                        x: focal_card.x,
                        y: sy,
                        width: CARD_WIDTH_MM,
                        height: CARD_HEIGHT_MM,
                        avatar_bytes,
                        generation: 0,
                    });

                    // Ochre marital connector line
                    connectors.push(RenderConnector {
                        points: vec![
                            (focal_card.x + CARD_WIDTH_MM / 2.0, prev_bottom_y),
                            (focal_card.x + CARD_WIDTH_MM / 2.0, sy),
                        ],
                        is_dashed: false,
                        color: Some(MARITAL_OCHRE.to_string()),
                    });
                    prev_bottom_y = sy + CARD_HEIGHT_MM;
                }
            }
        }
    }

    // Siblings placement for Pedigree (placed on Generation 0)
    if include_siblings {
        let siblings = find_siblings_of_person(&focal.id, store, person_by_id);
        let focal_pos = placements.iter().find(|p| p.person_id == focal.id);

        for sib_id in siblings {
            if rendered_ids.contains(&sib_id) {
                continue;
            }
            if let Some(sib) = person_by_id.get(sib_id.as_str()) {
                rendered_ids.insert(sib.id.clone());
                let sy = leaf_y_counter;
                leaf_y_counter += CARD_HEIGHT_MM + VERTICAL_GAP_MM;

                let full_name = format_person_name(sib);
                let lifespan = format_person_lifespan(sib, redact_living, is_ru);
                let avatar_bytes = resolve_person_avatar(
                    sib,
                    store,
                    media_storage_path,
                    redact_living,
                    show_photos,
                );

                cards.push(RenderPersonCard {
                    id: sib.id.clone(),
                    gid: sib.gid.clone(),
                    full_name,
                    lifespan,
                    is_living: sib.is_living,
                    is_focal: false,
                    sex: sib.sex.clone(),
                    x: MARGIN_X_MM,
                    y: sy,
                    width: CARD_WIDTH_MM,
                    height: CARD_HEIGHT_MM,
                    avatar_bytes,
                    generation: 0,
                });

                // Connect sibling to common parents if parents are on canvas
                let child_right_x = MARGIN_X_MM + CARD_WIDTH_MM;
                let child_mid_y = sy + (CARD_HEIGHT_MM / 2.0);
                let conn_mid_x = child_right_x + (HORIZONTAL_GAP_MM / 2.0);

                if let Some(fp) = focal_pos {
                    if let Some((_, fy)) = fp.father_pos {
                        let father_mid_y = fy + (CARD_HEIGHT_MM / 2.0);
                        connectors.push(RenderConnector {
                            points: vec![
                                (child_right_x, child_mid_y),
                                (conn_mid_x, child_mid_y),
                                (conn_mid_x, father_mid_y),
                                (child_right_x + HORIZONTAL_GAP_MM, father_mid_y),
                            ],
                            is_dashed: false,
                            color: None,
                        });
                    }
                    if let Some((_, my)) = fp.mother_pos {
                        let mother_mid_y = my + (CARD_HEIGHT_MM / 2.0);
                        connectors.push(RenderConnector {
                            points: vec![
                                (child_right_x, child_mid_y),
                                (conn_mid_x, child_mid_y),
                                (conn_mid_x, mother_mid_y),
                                (child_right_x + HORIZONTAL_GAP_MM, mother_mid_y),
                            ],
                            is_dashed: false,
                            color: None,
                        });
                    }
                    if fp.father_pos.is_none() && fp.mother_pos.is_none() {
                        // Unknown parents fallback: dashed fraternal bracket
                        connectors.push(RenderConnector {
                            points: vec![(fp.x + 8.0, fp.y), (MARGIN_X_MM + 8.0, sy)],
                            is_dashed: true,
                            color: None,
                        });
                    }
                }
            }
        }
    }

    (cards, connectors)
}

// ---------------------------------------------------------------------------
// 2. Descendant Layout (Top-Down with Couples & Midpoint Drops)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn layout_descendant(
    focal: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    max_generations: u32,
    redact_living: bool,
    show_photos: bool,
    is_ru: bool,
    media_storage_path: Option<&str>,
    include_partner: bool,
    include_siblings: bool,
) -> (Vec<RenderPersonCard>, Vec<RenderConnector>) {
    let mut cards: Vec<RenderPersonCard> = Vec::new();
    let mut connectors: Vec<RenderConnector> = Vec::new();
    let mut rendered_ids: HashSet<String> = HashSet::new();

    struct DescNodePlacement {
        person_id: String,
        generation: u32,
        x: f64,
        y: f64,
        union_mid_x: f64,
        children_pos: Vec<(f64, f64)>,
        spouse_id: Option<String>,
    }

    let mut placements: Vec<DescNodePlacement> = Vec::new();
    let mut leaf_x_counter = MARGIN_X_MM;

    fn place_desc_tree(
        person_id: &str,
        generation: u32,
        max_gen: u32,
        store: &GenealogyPrivacyStore,
        person_by_id: &HashMap<&str, &TreePersonNodeDto>,
        include_partner: bool,
        leaf_x_counter: &mut f64,
        placements: &mut Vec<DescNodePlacement>,
    ) -> (f64, f64) {
        let y = MARGIN_Y_MM
            + BANNER_HEIGHT_MM
            + (generation as f64) * (CARD_HEIGHT_MM + VERTICAL_GAP_MM * 1.5);

        let spouses = if include_partner {
            find_spouses(person_id, store, person_by_id)
        } else {
            Vec::new()
        };
        let primary_spouse = spouses.first().cloned();
        let couple_width = if primary_spouse.is_some() {
            CARD_WIDTH_MM * 2.0 + 12.0
        } else {
            CARD_WIDTH_MM
        };

        if generation + 1 >= max_gen {
            let x = *leaf_x_counter;
            *leaf_x_counter += couple_width + HORIZONTAL_GAP_MM;
            let union_mid = if primary_spouse.is_some() {
                x + CARD_WIDTH_MM + 6.0
            } else {
                x + CARD_WIDTH_MM / 2.0
            };

            placements.push(DescNodePlacement {
                person_id: person_id.to_string(),
                generation,
                x,
                y,
                union_mid_x: union_mid,
                children_pos: Vec::new(),
                spouse_id: primary_spouse,
            });
            return (union_mid, y);
        }

        let children_ids = find_children_of_person(person_id, store, person_by_id);
        if children_ids.is_empty() {
            let x = *leaf_x_counter;
            *leaf_x_counter += couple_width + HORIZONTAL_GAP_MM;
            let union_mid = if primary_spouse.is_some() {
                x + CARD_WIDTH_MM + 6.0
            } else {
                x + CARD_WIDTH_MM / 2.0
            };

            placements.push(DescNodePlacement {
                person_id: person_id.to_string(),
                generation,
                x,
                y,
                union_mid_x: union_mid,
                children_pos: Vec::new(),
                spouse_id: primary_spouse,
            });
            return (union_mid, y);
        }

        let mut child_positions = Vec::new();
        for ch_id in &children_ids {
            let pos = place_desc_tree(
                ch_id,
                generation + 1,
                max_gen,
                store,
                person_by_id,
                include_partner,
                leaf_x_counter,
                placements,
            );
            child_positions.push(pos);
        }

        let min_ch_x = child_positions
            .first()
            .map(|p| p.0)
            .unwrap_or(*leaf_x_counter);
        let max_ch_x = child_positions
            .last()
            .map(|p| p.0)
            .unwrap_or(*leaf_x_counter);
        let center_ch_x = (min_ch_x + max_ch_x) / 2.0;

        let x = (center_ch_x - couple_width / 2.0).max(MARGIN_X_MM);
        let union_mid = if primary_spouse.is_some() {
            x + CARD_WIDTH_MM + 6.0
        } else {
            x + CARD_WIDTH_MM / 2.0
        };

        placements.push(DescNodePlacement {
            person_id: person_id.to_string(),
            generation,
            x,
            y,
            union_mid_x: union_mid,
            children_pos: child_positions,
            spouse_id: primary_spouse,
        });

        (union_mid, y)
    }

    place_desc_tree(
        &focal.id,
        0,
        max_generations,
        store,
        person_by_id,
        include_partner,
        &mut leaf_x_counter,
        &mut placements,
    );

    // Sibling inclusion on Gen 0 flanking focal
    if include_siblings {
        let siblings = find_siblings_of_person(&focal.id, store, person_by_id);
        let y0 = MARGIN_Y_MM + BANNER_HEIGHT_MM;

        for sib_id in siblings {
            if !rendered_ids.contains(&sib_id) {
                if let Some(sib) = person_by_id.get(sib_id.as_str()) {
                    let sx = leaf_x_counter;
                    leaf_x_counter += CARD_WIDTH_MM + HORIZONTAL_GAP_MM;
                    rendered_ids.insert(sib.id.clone());

                    let full_name = format_person_name(sib);
                    let lifespan = format_person_lifespan(sib, redact_living, is_ru);
                    let avatar_bytes = resolve_person_avatar(
                        sib,
                        store,
                        media_storage_path,
                        redact_living,
                        show_photos,
                    );

                    cards.push(RenderPersonCard {
                        id: sib.id.clone(),
                        gid: sib.gid.clone(),
                        full_name,
                        lifespan,
                        is_living: sib.is_living,
                        is_focal: false,
                        sex: sib.sex.clone(),
                        x: sx,
                        y: y0,
                        width: CARD_WIDTH_MM,
                        height: CARD_HEIGHT_MM,
                        avatar_bytes,
                        generation: 0,
                    });

                    // Top fraternal bracket connecting focal and siblings
                    if let Some(fp) = placements.iter().find(|p| p.person_id == focal.id) {
                        let runner_y = y0 - 6.0;
                        connectors.push(RenderConnector {
                            points: vec![
                                (fp.union_mid_x, y0),
                                (fp.union_mid_x, runner_y),
                                (sx + CARD_WIDTH_MM / 2.0, runner_y),
                                (sx + CARD_WIDTH_MM / 2.0, y0),
                            ],
                            is_dashed: true,
                            color: None,
                        });
                    }
                }
            }
        }
    }

    // Build cards and connectors from placements
    for p in &placements {
        if rendered_ids.contains(&p.person_id) {
            continue;
        }
        rendered_ids.insert(p.person_id.clone());

        if let Some(person) = person_by_id.get(p.person_id.as_str()) {
            let is_focal = person.id == focal.id;
            let full_name = format_person_name(person);
            let lifespan = format_person_lifespan(person, redact_living, is_ru);
            let avatar_bytes = resolve_person_avatar(
                person,
                store,
                media_storage_path,
                redact_living,
                show_photos,
            );

            cards.push(RenderPersonCard {
                id: person.id.clone(),
                gid: person.gid.clone(),
                full_name,
                lifespan,
                is_living: person.is_living,
                is_focal,
                sex: person.sex.clone(),
                x: p.x,
                y: p.y,
                width: CARD_WIDTH_MM,
                height: CARD_HEIGHT_MM,
                avatar_bytes,
                generation: p.generation as i32,
            });

            // If spouse is paired
            if let Some(ref sid) = p.spouse_id {
                if let Some(spouse) = person_by_id.get(sid.as_str()) {
                    if !rendered_ids.contains(&spouse.id) {
                        rendered_ids.insert(spouse.id.clone());
                        let spouse_x = p.x + CARD_WIDTH_MM + 12.0;
                        let s_name = format_person_name(spouse);
                        let s_life = format_person_lifespan(spouse, redact_living, is_ru);
                        let s_avatar = resolve_person_avatar(
                            spouse,
                            store,
                            media_storage_path,
                            redact_living,
                            show_photos,
                        );

                        cards.push(RenderPersonCard {
                            id: spouse.id.clone(),
                            gid: spouse.gid.clone(),
                            full_name: s_name,
                            lifespan: s_life,
                            is_living: spouse.is_living,
                            is_focal: false,
                            sex: spouse.sex.clone(),
                            x: spouse_x,
                            y: p.y,
                            width: CARD_WIDTH_MM,
                            height: CARD_HEIGHT_MM,
                            avatar_bytes: s_avatar,
                            generation: p.generation as i32,
                        });

                        // Marital connector bar between person and spouse
                        let mid_y = p.y + CARD_HEIGHT_MM / 2.0;
                        connectors.push(RenderConnector {
                            points: vec![(p.x + CARD_WIDTH_MM, mid_y), (spouse_x, mid_y)],
                            is_dashed: false,
                            color: Some(MARITAL_OCHRE.to_string()),
                        });
                    }
                }
            }

            // Drop connectors to children from union midpoint
            if !p.children_pos.is_empty() {
                let parent_bot_x = p.union_mid_x;
                let parent_bot_y = p.y + CARD_HEIGHT_MM;
                let mid_y = parent_bot_y + (VERTICAL_GAP_MM * 1.5 / 2.0);

                // Vertical drop from parent couple
                connectors.push(RenderConnector {
                    points: vec![(parent_bot_x, parent_bot_y), (parent_bot_x, mid_y)],
                    is_dashed: false,
                    color: None,
                });

                // Horizontal distribution bar
                let min_x = p
                    .children_pos
                    .iter()
                    .map(|c| c.0)
                    .fold(f64::INFINITY, f64::min);
                let max_x = p
                    .children_pos
                    .iter()
                    .map(|c| c.0)
                    .fold(f64::NEG_INFINITY, f64::max);

                connectors.push(RenderConnector {
                    points: vec![
                        (min_x.min(parent_bot_x), mid_y),
                        (max_x.max(parent_bot_x), mid_y),
                    ],
                    is_dashed: false,
                    color: None,
                });

                // Vertical drops into each child
                for (cx, cy) in &p.children_pos {
                    connectors.push(RenderConnector {
                        points: vec![(*cx, mid_y), (*cx, *cy)],
                        is_dashed: false,
                        color: None,
                    });
                }
            }
        }
    }

    (cards, connectors)
}

// ---------------------------------------------------------------------------
// 3. Hierarchical Hourglass & Component Layout Helpers
// ---------------------------------------------------------------------------

struct DescBranch {
    person_id: String,
    spouses: Vec<String>,
    generation: i32,
    couple_width: f64,
    width: f64,
    center_x: f64,
    children_rel_x: Vec<f64>,
    children: Vec<DescBranch>,
}

fn build_desc_branch(
    pid: &str,
    gen: i32,
    max_gen: i32,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    include_partner: bool,
    assigned: &mut HashSet<String>,
) -> DescBranch {
    assigned.insert(pid.to_string());
    let mut spouses = Vec::new();
    if include_partner {
        for sp in find_spouses(pid, store, person_by_id) {
            if assigned.insert(sp.clone()) {
                spouses.push(sp);
            }
        }
    }
    let couple_count = 1 + spouses.len();
    let couple_width =
        couple_count as f64 * CARD_WIDTH_MM + (couple_count.saturating_sub(1) as f64) * 12.0;

    let mut children_branches = Vec::new();
    if gen < max_gen {
        let child_ids = find_children_of_person(pid, store, person_by_id);
        for cid in child_ids {
            if !assigned.contains(&cid) {
                children_branches.push(build_desc_branch(
                    &cid,
                    gen + 1,
                    max_gen,
                    store,
                    person_by_id,
                    include_partner,
                    assigned,
                ));
            }
        }
    }

    if children_branches.is_empty() {
        return DescBranch {
            person_id: pid.to_string(),
            spouses,
            generation: gen,
            couple_width,
            width: couple_width,
            center_x: couple_width / 2.0,
            children_rel_x: Vec::new(),
            children: Vec::new(),
        };
    }

    let mut ch_rel_x = Vec::new();
    let mut cur_x = 0.0;
    for (i, ch) in children_branches.iter().enumerate() {
        if i > 0 {
            cur_x += HORIZONTAL_GAP_MM;
        }
        ch_rel_x.push(cur_x);
        cur_x += ch.width;
    }
    let total_ch_width = cur_x;

    let first_ch_center = ch_rel_x[0] + children_branches[0].center_x;
    let last_ch_center = ch_rel_x.last().unwrap() + children_branches.last().unwrap().center_x;
    let avg_ch_center = (first_ch_center + last_ch_center) / 2.0;

    let couple_left = avg_ch_center - couple_width / 2.0;
    let couple_right = avg_ch_center + couple_width / 2.0;

    let min_extent = 0.0_f64.min(couple_left);
    let max_extent = total_ch_width.max(couple_right);
    let shift = -min_extent;

    let width = max_extent - min_extent;
    let center_x = avg_ch_center + shift;

    for x in &mut ch_rel_x {
        *x += shift;
    }

    DescBranch {
        person_id: pid.to_string(),
        spouses,
        generation: gen,
        couple_width,
        width,
        center_x,
        children_rel_x: ch_rel_x,
        children: children_branches,
    }
}

fn place_desc_branch(branch: &DescBranch, base_x: f64, coords: &mut HashMap<String, (f64, i32)>) {
    let couple_x = branch.center_x - branch.couple_width / 2.0;
    coords.insert(
        branch.person_id.clone(),
        (base_x + couple_x, branch.generation),
    );
    for (i, sp) in branch.spouses.iter().enumerate() {
        let sp_x = base_x + couple_x + ((i + 1) as f64) * (CARD_WIDTH_MM + 12.0);
        coords.insert(sp.clone(), (sp_x, branch.generation));
    }
    for (i, ch) in branch.children.iter().enumerate() {
        place_desc_branch(ch, base_x + branch.children_rel_x[i], coords);
    }
}

struct AncBranch {
    person_id: String,
    generation: i32,
    width: f64,
    center_x: f64,
    father: Option<(Box<AncBranch>, f64)>,
    mother: Option<(Box<AncBranch>, f64)>,
}

fn build_anc_branch(
    pid: &str,
    gen: i32,
    max_depth: i32,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    assigned: &mut HashSet<String>,
) -> AncBranch {
    assigned.insert(pid.to_string());

    let (f_id, m_id) = if gen.abs() < max_depth {
        find_parents(pid, store, person_by_id)
    } else {
        (None, None)
    };

    let f_branch = f_id
        .filter(|id| !assigned.contains(id))
        .map(|id| build_anc_branch(&id, gen - 1, max_depth, store, person_by_id, assigned));
    let m_branch = m_id
        .filter(|id| !assigned.contains(id))
        .map(|id| build_anc_branch(&id, gen - 1, max_depth, store, person_by_id, assigned));

    match (f_branch, m_branch) {
        (Some(f), Some(m)) => {
            let f_w = f.width;
            let m_rel_x = f_w + HORIZONTAL_GAP_MM;
            let total_parents_w = m_rel_x + m.width;

            let f_center = f.center_x;
            let m_center = m_rel_x + m.center_x;
            let avg_p_center = (f_center + m_center) / 2.0;

            let card_left = avg_p_center - CARD_WIDTH_MM / 2.0;
            let card_right = avg_p_center + CARD_WIDTH_MM / 2.0;

            let min_extent = 0.0_f64.min(card_left);
            let max_extent = total_parents_w.max(card_right);
            let shift = -min_extent;

            let width = max_extent - min_extent;
            let center_x = avg_p_center + shift;

            AncBranch {
                person_id: pid.to_string(),
                generation: gen,
                width,
                center_x,
                father: Some((Box::new(f), shift)),
                mother: Some((Box::new(m), m_rel_x + shift)),
            }
        }
        (Some(f), None) => {
            let f_center = f.center_x;
            let card_left = f_center - CARD_WIDTH_MM / 2.0;
            let card_right = f_center + CARD_WIDTH_MM / 2.0;

            let min_extent = 0.0_f64.min(card_left);
            let max_extent = f.width.max(card_right);
            let shift = -min_extent;

            let width = max_extent - min_extent;
            let center_x = f_center + shift;

            AncBranch {
                person_id: pid.to_string(),
                generation: gen,
                width,
                center_x,
                father: Some((Box::new(f), shift)),
                mother: None,
            }
        }
        (None, Some(m)) => {
            let m_center = m.center_x;
            let card_left = m_center - CARD_WIDTH_MM / 2.0;
            let card_right = m_center + CARD_WIDTH_MM / 2.0;

            let min_extent = 0.0_f64.min(card_left);
            let max_extent = m.width.max(card_right);
            let shift = -min_extent;

            let width = max_extent - min_extent;
            let center_x = m_center + shift;

            AncBranch {
                person_id: pid.to_string(),
                generation: gen,
                width,
                center_x,
                father: None,
                mother: Some((Box::new(m), shift)),
            }
        }
        (None, None) => AncBranch {
            person_id: pid.to_string(),
            generation: gen,
            width: CARD_WIDTH_MM,
            center_x: CARD_WIDTH_MM / 2.0,
            father: None,
            mother: None,
        },
    }
}

fn place_anc_branch(branch: &AncBranch, base_x: f64, coords: &mut HashMap<String, (f64, i32)>) {
    let card_x = base_x + branch.center_x - CARD_WIDTH_MM / 2.0;
    coords.insert(branch.person_id.clone(), (card_x, branch.generation));
    if let Some((ref f, rel_x)) = branch.father {
        place_anc_branch(f, base_x + rel_x, coords);
    }
    if let Some((ref m, rel_x)) = branch.mother {
        place_anc_branch(m, base_x + rel_x, coords);
    }
}

struct CollateralAuntUnit {
    aunt_id: String,
    spouses: Vec<String>,
    cousins: Vec<String>,
    cousin_spouses: HashMap<String, Vec<String>>,
    width: f64,
    center_x: f64,
    cousins_rel_x: Vec<f64>,
}

fn build_collateral_aunt_unit(
    aunt_id: &str,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    include_partner: bool,
    assigned: &mut HashSet<String>,
) -> CollateralAuntUnit {
    assigned.insert(aunt_id.to_string());

    let mut spouses = Vec::new();
    if include_partner {
        for sp in find_spouses(aunt_id, store, person_by_id) {
            if assigned.insert(sp.clone()) {
                spouses.push(sp);
            }
        }
    }
    let aunt_count = 1 + spouses.len();
    let aunt_w = aunt_count as f64 * CARD_WIDTH_MM + (aunt_count.saturating_sub(1) as f64) * 12.0;

    let mut cousins = Vec::new();
    let mut cousin_spouses = HashMap::new();
    for cid in find_children_of_person(aunt_id, store, person_by_id) {
        if assigned.insert(cid.clone()) {
            let mut c_sps = Vec::new();
            if include_partner {
                for sp in find_spouses(&cid, store, person_by_id) {
                    if assigned.insert(sp.clone()) {
                        c_sps.push(sp);
                    }
                }
            }
            cousin_spouses.insert(cid.clone(), c_sps);
            cousins.push(cid);
        }
    }

    if cousins.is_empty() {
        return CollateralAuntUnit {
            aunt_id: aunt_id.to_string(),
            spouses,
            cousins: Vec::new(),
            cousin_spouses: HashMap::new(),
            width: aunt_w,
            center_x: aunt_w / 2.0,
            cousins_rel_x: Vec::new(),
        };
    }

    let mut cousins_rel_x = Vec::new();
    let mut cur_x = 0.0;
    for (i, cid) in cousins.iter().enumerate() {
        if i > 0 {
            cur_x += HORIZONTAL_GAP_MM;
        }
        cousins_rel_x.push(cur_x);
        let sp_count = 1 + cousin_spouses.get(cid).map(|s| s.len()).unwrap_or(0);
        let c_w = sp_count as f64 * CARD_WIDTH_MM + (sp_count.saturating_sub(1) as f64) * 12.0;
        cur_x += c_w;
    }
    let total_cousins_w = cur_x;

    let width = aunt_w.max(total_cousins_w);
    let center_x = width / 2.0;

    let cousins_shift = (width - total_cousins_w) / 2.0;
    for x in &mut cousins_rel_x {
        *x += cousins_shift;
    }

    CollateralAuntUnit {
        aunt_id: aunt_id.to_string(),
        spouses,
        cousins,
        cousin_spouses,
        width,
        center_x,
        cousins_rel_x,
    }
}

fn place_collateral_aunt_unit(
    unit: &CollateralAuntUnit,
    base_x: f64,
    coords: &mut HashMap<String, (f64, i32)>,
) {
    let aunt_count = 1 + unit.spouses.len();
    let aunt_w = aunt_count as f64 * CARD_WIDTH_MM + (aunt_count.saturating_sub(1) as f64) * 12.0;
    let aunt_x = base_x + unit.center_x - aunt_w / 2.0;

    coords.insert(unit.aunt_id.clone(), (aunt_x, -1));
    for (i, sp) in unit.spouses.iter().enumerate() {
        let sp_x = aunt_x + ((i + 1) as f64) * (CARD_WIDTH_MM + 12.0);
        coords.insert(sp.clone(), (sp_x, -1));
    }

    for (i, cid) in unit.cousins.iter().enumerate() {
        let c_base_x = base_x + unit.cousins_rel_x[i];
        coords.insert(cid.clone(), (c_base_x, 0));
        if let Some(sps) = unit.cousin_spouses.get(cid) {
            for (j, sp) in sps.iter().enumerate() {
                let sp_x = c_base_x + ((j + 1) as f64) * (CARD_WIDTH_MM + 12.0);
                coords.insert(sp.clone(), (sp_x, 0));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_hourglass(
    focal: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    generations: u32,
    redact_living: bool,
    show_photos: bool,
    is_ru: bool,
    media_storage_path: Option<&str>,
    include_partner: bool,
    include_siblings: bool,
    collateral: bool,
) -> (Vec<RenderPersonCard>, Vec<RenderConnector>) {
    let mut cards: Vec<RenderPersonCard> = Vec::new();
    let mut connectors: Vec<RenderConnector> = Vec::new();
    let mut assigned: HashSet<String> = HashSet::new();

    // 1. Build descendant branch for focal person (Gen 0 downwards)
    let focal_branch = build_desc_branch(
        &focal.id,
        0,
        generations as i32,
        store,
        person_by_id,
        include_partner,
        &mut assigned,
    );

    // 2. Siblings of focal on Gen 0
    let mut sib_branches = Vec::new();
    if include_siblings || collateral {
        let sib_ids = find_siblings_of_person(&focal.id, store, person_by_id);
        let sib_max_gen = if collateral { generations as i32 } else { 0 };
        for sid in sib_ids {
            if !assigned.contains(&sid) {
                sib_branches.push(build_desc_branch(
                    &sid,
                    0,
                    sib_max_gen,
                    store,
                    person_by_id,
                    include_partner,
                    &mut assigned,
                ));
            }
        }
    }

    // 3. Ancestors of focal (Father side and Mother side)
    let (father_id, mother_id) = if generations > 0 {
        find_parents(&focal.id, store, person_by_id)
    } else {
        (None, None)
    };

    let father_branch = father_id
        .as_deref()
        .filter(|id| !assigned.contains(*id))
        .map(|fid| {
            build_anc_branch(
                fid,
                -1,
                generations as i32,
                store,
                person_by_id,
                &mut assigned,
            )
        });

    let mother_branch = mother_id
        .as_deref()
        .filter(|id| !assigned.contains(*id))
        .map(|mid| {
            build_anc_branch(
                mid,
                -1,
                generations as i32,
                store,
                person_by_id,
                &mut assigned,
            )
        });

    // Collaterals on generation -1 (Aunts/Uncles)
    let mut pat_units = Vec::new();
    let mut mat_units = Vec::new();
    if collateral {
        if let Some(ref fid) = father_id {
            for sid in find_siblings_of_person(fid, store, person_by_id) {
                if !assigned.contains(&sid) {
                    pat_units.push(build_collateral_aunt_unit(
                        &sid,
                        store,
                        person_by_id,
                        include_partner,
                        &mut assigned,
                    ));
                }
            }
        }
        if let Some(ref mid) = mother_id {
            for sid in find_siblings_of_person(mid, store, person_by_id) {
                if !assigned.contains(&sid) {
                    mat_units.push(build_collateral_aunt_unit(
                        &sid,
                        store,
                        person_by_id,
                        include_partner,
                        &mut assigned,
                    ));
                }
            }
        }
    }

    // 4. Calculate horizontal positioning relative to Focal's Center (X = 0)
    let mut sib_rel_x = Vec::new();
    let mut desc_cur_x = focal_branch.width;
    for sib in &sib_branches {
        desc_cur_x += HORIZONTAL_GAP_MM;
        sib_rel_x.push(desc_cur_x);
        desc_cur_x += sib.width;
    }
    let desc_total_w = desc_cur_x;
    let l_desc = focal_branch.center_x;
    let r_desc = desc_total_w - focal_branch.center_x;

    let mut pat_unit_rel_x = Vec::new();
    let mut pat_cur_x = 0.0;
    for (i, u) in pat_units.iter().enumerate() {
        if i > 0 {
            pat_cur_x += HORIZONTAL_GAP_MM;
        }
        pat_unit_rel_x.push(pat_cur_x);
        pat_cur_x += u.width;
    }
    let father_offset = if pat_units.is_empty() {
        0.0
    } else {
        pat_cur_x + HORIZONTAL_GAP_MM
    };
    let pat_total_w = father_offset + father_branch.as_ref().map(|f| f.width).unwrap_or(0.0);
    let father_center_in_pat =
        father_offset + father_branch.as_ref().map(|f| f.center_x).unwrap_or(0.0);
    let _l_pat = father_center_in_pat;
    let r_pat = pat_total_w - father_center_in_pat;

    let mother_w = mother_branch.as_ref().map(|m| m.width).unwrap_or(0.0);
    let mother_center_in_mat = mother_branch.as_ref().map(|m| m.center_x).unwrap_or(0.0);
    let mut mat_unit_rel_x = Vec::new();
    let mut mat_cur_x = if mat_units.is_empty() {
        mother_w
    } else {
        mother_w + HORIZONTAL_GAP_MM
    };
    for (i, u) in mat_units.iter().enumerate() {
        if i > 0 {
            mat_cur_x += HORIZONTAL_GAP_MM;
        }
        mat_unit_rel_x.push(mat_cur_x);
        mat_cur_x += u.width;
    }
    let mat_total_w = mat_cur_x;
    let l_mat = mother_center_in_mat;
    let _r_mat = mat_total_w - mother_center_in_mat;

    let has_father = father_branch.is_some();
    let has_mother = mother_branch.is_some();

    let mut delta = (CARD_WIDTH_MM + 12.0) / 2.0; // 34.0 mm
    if has_father && has_mother {
        delta = delta.max((r_pat + HORIZONTAL_GAP_MM + l_mat) / 2.0);
        delta = delta.max(r_desc + HORIZONTAL_GAP_MM + l_mat);
        delta = delta.max(l_desc + HORIZONTAL_GAP_MM + r_pat);
    }

    let mut coords: HashMap<String, (f64, i32)> = HashMap::new();

    // Place focal & siblings
    let focal_base_x = -focal_branch.center_x;
    place_desc_branch(&focal_branch, focal_base_x, &mut coords);

    for (i, sib) in sib_branches.iter().enumerate() {
        place_desc_branch(sib, focal_base_x + sib_rel_x[i], &mut coords);
    }

    // Place paternal block
    if has_father {
        let father_target_center = if has_mother { -delta } else { 0.0 };
        let pat_base_x = father_target_center - father_center_in_pat;
        for (i, u) in pat_units.iter().enumerate() {
            place_collateral_aunt_unit(u, pat_base_x + pat_unit_rel_x[i], &mut coords);
        }
        if let Some(ref fb) = father_branch {
            place_anc_branch(fb, pat_base_x + father_offset, &mut coords);
        }
    } else if !pat_units.is_empty() {
        let pat_base_x = -l_desc - HORIZONTAL_GAP_MM - pat_total_w;
        for (i, u) in pat_units.iter().enumerate() {
            place_collateral_aunt_unit(u, pat_base_x + pat_unit_rel_x[i], &mut coords);
        }
    }

    // Place maternal block
    if has_mother {
        let mother_target_center = if has_father { delta } else { 0.0 };
        let mat_base_x = mother_target_center - mother_center_in_mat;
        if let Some(ref mb) = mother_branch {
            place_anc_branch(mb, mat_base_x, &mut coords);
        }
        for (i, u) in mat_units.iter().enumerate() {
            place_collateral_aunt_unit(u, mat_base_x + mat_unit_rel_x[i], &mut coords);
        }
    } else if !mat_units.is_empty() {
        let mat_base_x = r_desc + HORIZONTAL_GAP_MM;
        for (i, u) in mat_units.iter().enumerate() {
            place_collateral_aunt_unit(u, mat_base_x + mat_unit_rel_x[i], &mut coords);
        }
    }

    let min_g = coords.values().map(|(_, g)| *g).min().unwrap_or(0);
    let max_g = coords.values().map(|(_, g)| *g).max().unwrap_or(0);
    let row_height_step = CARD_HEIGHT_MM + VERTICAL_GAP_MM * 1.5;

    let min_x = coords
        .values()
        .map(|(x, _)| *x)
        .fold(f64::INFINITY, f64::min);
    let x_shift = if min_x.is_finite() {
        MARGIN_X_MM - min_x
    } else {
        0.0
    };

    let mut person_coords: HashMap<String, (f64, f64)> = HashMap::new();
    for (pid, (x, g)) in &coords {
        let px = x + x_shift;
        let tier_idx = (g - min_g) as f64;
        let py = MARGIN_Y_MM + BANNER_HEIGHT_MM + tier_idx * row_height_step;
        person_coords.insert(pid.clone(), (px, py));
    }

    for (pid, (x, y)) in &person_coords {
        if let Some(person) = person_by_id.get(pid.as_str()) {
            let is_focal = person.id == focal.id;
            let full_name = format_person_name(person);
            let lifespan = format_person_lifespan(person, redact_living, is_ru);
            let avatar_bytes = resolve_person_avatar(
                person,
                store,
                media_storage_path,
                redact_living,
                show_photos,
            );
            let gen = coords.get(pid).map(|(_, g)| *g).unwrap_or(0);

            cards.push(RenderPersonCard {
                id: person.id.clone(),
                gid: person.gid.clone(),
                full_name,
                lifespan,
                is_living: person.is_living,
                is_focal,
                sex: person.sex.clone(),
                x: *x,
                y: *y,
                width: CARD_WIDTH_MM,
                height: CARD_HEIGHT_MM,
                avatar_bytes,
                generation: gen,
            });
        }
    }

    // Marital connectors
    let mut connected_couples: HashSet<(String, String)> = HashSet::new();
    for fam in &store.families {
        if let (Some(ref p1), Some(ref p2)) = (&fam.partner1_id, &fam.partner2_id) {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) =
                (person_coords.get(p1), person_coords.get(p2))
            {
                let g1 = coords.get(p1).map(|(_, g)| *g).unwrap_or(0);
                let g2 = coords.get(p2).map(|(_, g)| *g).unwrap_or(0);
                if g1 == g2 && (y1 - y2).abs() < 1.0 {
                    let key = if p1 < p2 {
                        (p1.clone(), p2.clone())
                    } else {
                        (p2.clone(), p1.clone())
                    };
                    if connected_couples.insert(key) {
                        let left_x = x1.min(x2);
                        let right_x = x1.max(x2);
                        let mid_y = y1 + CARD_HEIGHT_MM / 2.0;
                        connectors.push(RenderConnector {
                            points: vec![(left_x + CARD_WIDTH_MM, mid_y), (right_x, mid_y)],
                            is_dashed: false,
                            color: Some(MARITAL_OCHRE.to_string()),
                        });
                    }
                }
            }
        }
    }

    // Parent-child distribution connectors across consecutive tiers
    for g in min_g..max_g {
        for fam in &store.families {
            let p1_pos = fam.partner1_id.as_ref().and_then(|id| {
                if coords.get(id).map(|(_, cg)| *cg == g).unwrap_or(false) {
                    person_coords.get(id)
                } else {
                    None
                }
            });
            let p2_pos = fam.partner2_id.as_ref().and_then(|id| {
                if coords.get(id).map(|(_, cg)| *cg == g).unwrap_or(false) {
                    person_coords.get(id)
                } else {
                    None
                }
            });

            if p1_pos.is_none() && p2_pos.is_none() {
                continue;
            }

            let active_children: Vec<&String> = fam
                .children
                .iter()
                .map(|c| &c.child_id)
                .filter(|id| coords.get(*id).map(|(_, cg)| *cg == g + 1).unwrap_or(false))
                .collect();

            if active_children.is_empty() {
                continue;
            }

            let union_x = match (p1_pos, p2_pos) {
                (Some((x1, _)), Some((x2, _))) => (x1.min(*x2) + CARD_WIDTH_MM + x1.max(*x2)) / 2.0,
                (Some((x1, _)), None) => x1 + CARD_WIDTH_MM / 2.0,
                (None, Some((x2, _))) => x2 + CARD_WIDTH_MM / 2.0,
                (None, None) => continue,
            };

            let parent_y = p1_pos
                .or(p2_pos)
                .map(|(_, y)| *y + CARD_HEIGHT_MM)
                .unwrap_or(0.0);
            let bus_y = parent_y + (VERTICAL_GAP_MM * 1.5) / 2.0;

            let mut ch_mid_xs = Vec::new();
            for cid in &active_children {
                if let Some((cx, cy)) = person_coords.get(*cid) {
                    let cmx = cx + CARD_WIDTH_MM / 2.0;
                    ch_mid_xs.push(cmx);
                    connectors.push(RenderConnector {
                        points: vec![(cmx, bus_y), (cmx, *cy)],
                        is_dashed: false,
                        color: None,
                    });
                }
            }

            if !ch_mid_xs.is_empty() {
                connectors.push(RenderConnector {
                    points: vec![(union_x, parent_y), (union_x, bus_y)],
                    is_dashed: false,
                    color: None,
                });

                let min_ch_x = ch_mid_xs.iter().copied().fold(union_x, f64::min);
                let max_ch_x = ch_mid_xs.iter().copied().fold(union_x, f64::max);
                connectors.push(RenderConnector {
                    points: vec![(min_ch_x, bus_y), (max_ch_x, bus_y)],
                    is_dashed: false,
                    color: None,
                });
            }
        }
    }

    (cards, connectors)
}

// ---------------------------------------------------------------------------
// 4. All Nodes Layout (Forest Graph View with Disjoint Set Union)
// ---------------------------------------------------------------------------

struct Dsu {
    parent: HashMap<String, String>,
}

impl Dsu {
    fn new(ids: &[String]) -> Self {
        let mut parent = HashMap::new();
        for id in ids {
            parent.insert(id.clone(), id.clone());
        }
        Self { parent }
    }

    fn find(&mut self, i: &str) -> String {
        let p = self.parent.get(i).cloned().unwrap_or_else(|| i.to_string());
        if p != i {
            let root = self.find(&p);
            self.parent.insert(i.to_string(), root.clone());
            root
        } else {
            p
        }
    }

    fn union(&mut self, i: &str, j: &str) {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parent.insert(root_i, root_j);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_all_nodes(
    focal: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    redact_living: bool,
    show_photos: bool,
    is_ru: bool,
    media_storage_path: Option<&str>,
    include_partner: bool,
) -> (Vec<RenderPersonCard>, Vec<RenderConnector>) {
    let mut all_cards: Vec<RenderPersonCard> = Vec::new();
    let mut all_connectors: Vec<RenderConnector> = Vec::new();

    let all_person_ids: Vec<String> = store.persons.iter().map(|p| p.id.clone()).collect();
    let mut dsu = Dsu::new(&all_person_ids);

    // Union partners and children across all families
    for fam in &store.families {
        let p1 = fam.partner1_id.as_deref();
        let p2 = fam.partner2_id.as_deref();

        if let (Some(a), Some(b)) = (p1, p2) {
            dsu.union(a, b);
        }

        for ch in &fam.children {
            if let Some(a) = p1 {
                dsu.union(a, &ch.child_id);
            }
            if let Some(b) = p2 {
                dsu.union(b, &ch.child_id);
            }
        }
    }

    // Identify Singletons (0 parents, 0 spouses, 0 children)
    let mut singletons: Vec<&TreePersonNodeDto> = Vec::new();
    let mut component_nodes: HashMap<String, Vec<&TreePersonNodeDto>> = HashMap::new();

    for p in &store.persons {
        let has_parents = store
            .families
            .iter()
            .any(|f| f.children.iter().any(|c| c.child_id == p.id));
        let has_spouses = store.families.iter().any(|f| {
            f.partner1_id.as_deref() == Some(&p.id) || f.partner2_id.as_deref() == Some(&p.id)
        });
        let has_children = store.families.iter().any(|f| {
            (f.partner1_id.as_deref() == Some(&p.id) || f.partner2_id.as_deref() == Some(&p.id))
                && !f.children.is_empty()
        });

        if !has_parents && !has_spouses && !has_children {
            singletons.push(p);
        } else {
            let root = dsu.find(&p.id);
            component_nodes.entry(root).or_default().push(p);
        }
    }

    // Sort components: primary component (containing focal) first, then largest to smallest
    let focal_root = dsu.find(&focal.id);
    let mut sorted_roots: Vec<String> = component_nodes.keys().cloned().collect();
    sorted_roots.sort_by(|a, b| {
        if a == &focal_root {
            std::cmp::Ordering::Less
        } else if b == &focal_root {
            std::cmp::Ordering::Greater
        } else {
            component_nodes[b].len().cmp(&component_nodes[a].len())
        }
    });

    let mut current_offset_x = MARGIN_X_MM;
    let mut max_bottom_y = MARGIN_Y_MM + BANNER_HEIGHT_MM;

    // Layout each connected component
    for root in sorted_roots {
        let members = &component_nodes[&root];
        if members.is_empty() {
            continue;
        }

        // Generational leveling using BFS
        let mut gen_map: HashMap<String, i32> = HashMap::new();
        let seed_id = if members.iter().any(|m| m.id == focal.id) {
            focal.id.clone()
        } else {
            members[0].id.clone()
        };

        gen_map.insert(seed_id.clone(), 0);
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(seed_id);

        while let Some(curr) = queue.pop_front() {
            let curr_gen = gen_map[&curr];
            for fam in &store.families {
                let is_p1 = fam.partner1_id.as_deref() == Some(&curr);
                let is_p2 = fam.partner2_id.as_deref() == Some(&curr);

                if is_p1 {
                    if let Some(ref s) = fam.partner2_id {
                        if !gen_map.contains_key(s) {
                            gen_map.insert(s.clone(), curr_gen);
                            queue.push_back(s.clone());
                        }
                    }
                    for ch in &fam.children {
                        if !gen_map.contains_key(&ch.child_id) {
                            gen_map.insert(ch.child_id.clone(), curr_gen + 1);
                            queue.push_back(ch.child_id.clone());
                        }
                    }
                } else if is_p2 {
                    if let Some(ref s) = fam.partner1_id {
                        if !gen_map.contains_key(s) {
                            gen_map.insert(s.clone(), curr_gen);
                            queue.push_back(s.clone());
                        }
                    }
                    for ch in &fam.children {
                        if !gen_map.contains_key(&ch.child_id) {
                            gen_map.insert(ch.child_id.clone(), curr_gen + 1);
                            queue.push_back(ch.child_id.clone());
                        }
                    }
                } else if fam.children.iter().any(|c| c.child_id == curr) {
                    for p in [&fam.partner1_id, &fam.partner2_id].into_iter().flatten() {
                        if !gen_map.contains_key(p) {
                            gen_map.insert(p.clone(), curr_gen - 1);
                            queue.push_back(p.clone());
                        }
                    }
                }
            }
        }

        let min_gen = members
            .iter()
            .filter_map(|m| gen_map.get(&m.id))
            .copied()
            .min()
            .unwrap_or(0);
        let max_gen = members
            .iter()
            .filter_map(|m| gen_map.get(&m.id))
            .copied()
            .max()
            .unwrap_or(0);

        let row_height_step = CARD_HEIGHT_MM + VERTICAL_GAP_MM * 1.5;

        // Group parents and children hierarchically into trees
        let mut comp_assigned: HashSet<String> = HashSet::new();
        let mut root_trees: Vec<DescBranch> = Vec::new();

        for g in min_gen..=max_gen {
            for m in members {
                if gen_map.get(&m.id).copied().unwrap_or(0) == g && !comp_assigned.contains(&m.id) {
                    let has_parent_in_comp = store.families.iter().any(|f| {
                        f.children.iter().any(|c| c.child_id == m.id)
                            && (f
                                .partner1_id
                                .as_ref()
                                .is_some_and(|id| members.iter().any(|tm| tm.id == *id))
                                || f.partner2_id
                                    .as_ref()
                                    .is_some_and(|id| members.iter().any(|tm| tm.id == *id)))
                    });
                    if !has_parent_in_comp {
                        root_trees.push(build_desc_branch(
                            &m.id,
                            g,
                            max_gen,
                            store,
                            person_by_id,
                            include_partner,
                            &mut comp_assigned,
                        ));
                    }
                }
            }
        }

        // Any remaining members in component
        for m in members {
            if !comp_assigned.contains(&m.id) {
                let g = gen_map.get(&m.id).copied().unwrap_or(0);
                root_trees.push(build_desc_branch(
                    &m.id,
                    g,
                    max_gen,
                    store,
                    person_by_id,
                    include_partner,
                    &mut comp_assigned,
                ));
            }
        }

        let mut comp_coords: HashMap<String, (f64, f64)> = HashMap::new();
        let mut cur_x = current_offset_x;
        for tree in &root_trees {
            let mut tree_coords: HashMap<String, (f64, i32)> = HashMap::new();
            place_desc_branch(tree, cur_x, &mut tree_coords);
            for (pid, (x, g)) in tree_coords {
                let tier_idx = (g - min_gen) as f64;
                let y = MARGIN_Y_MM + BANNER_HEIGHT_MM + tier_idx * row_height_step;
                comp_coords.insert(pid, (x, y));
            }
            cur_x += tree.width + HORIZONTAL_GAP_MM;
        }

        let comp_max_x = comp_coords
            .values()
            .map(|(x, _)| *x + CARD_WIDTH_MM)
            .fold(current_offset_x, f64::max);

        // Add cards for component
        for m in members {
            if let Some((x, y)) = comp_coords.get(&m.id) {
                let is_focal = m.id == focal.id;
                let full_name = format_person_name(m);
                let lifespan = format_person_lifespan(m, redact_living, is_ru);
                let avatar_bytes =
                    resolve_person_avatar(m, store, media_storage_path, redact_living, show_photos);
                let gen = gen_map.get(&m.id).copied().unwrap_or(0) - min_gen;

                all_cards.push(RenderPersonCard {
                    id: m.id.clone(),
                    gid: m.gid.clone(),
                    full_name,
                    lifespan,
                    is_living: m.is_living,
                    is_focal,
                    sex: m.sex.clone(),
                    x: *x,
                    y: *y,
                    width: CARD_WIDTH_MM,
                    height: CARD_HEIGHT_MM,
                    avatar_bytes,
                    generation: gen,
                });

                if y + CARD_HEIGHT_MM > max_bottom_y {
                    max_bottom_y = y + CARD_HEIGHT_MM;
                }
            }
        }

        // Add ochre marital connectors
        let mut comp_connected_couples: HashSet<(String, String)> = HashSet::new();
        for fam in &store.families {
            if let (Some(ref p1), Some(ref p2)) = (&fam.partner1_id, &fam.partner2_id) {
                if let (Some(&(x1, y1)), Some(&(x2, y2))) =
                    (comp_coords.get(p1), comp_coords.get(p2))
                {
                    if (y1 - y2).abs() < 1.0 {
                        let key = if p1 < p2 {
                            (p1.clone(), p2.clone())
                        } else {
                            (p2.clone(), p1.clone())
                        };
                        if comp_connected_couples.insert(key) {
                            let left_x = x1.min(x2);
                            let right_x = x1.max(x2);
                            let mid_y = y1 + CARD_HEIGHT_MM / 2.0;
                            all_connectors.push(RenderConnector {
                                points: vec![(left_x + CARD_WIDTH_MM, mid_y), (right_x, mid_y)],
                                is_dashed: false,
                                color: Some(MARITAL_OCHRE.to_string()),
                            });
                        }
                    }
                }
            }
        }

        // Add family distribution connectors in component
        for fam in &store.families {
            let p1_pos = fam.partner1_id.as_ref().and_then(|id| comp_coords.get(id));
            let p2_pos = fam.partner2_id.as_ref().and_then(|id| comp_coords.get(id));

            if p1_pos.is_none() && p2_pos.is_none() {
                continue;
            }

            let union_x = match (p1_pos, p2_pos) {
                (Some((x1, _)), Some((x2, _))) => (x1.min(*x2) + CARD_WIDTH_MM + x1.max(*x2)) / 2.0,
                (Some((x1, _)), None) => x1 + CARD_WIDTH_MM / 2.0,
                (None, Some((x2, _))) => x2 + CARD_WIDTH_MM / 2.0,
                (None, None) => continue,
            };

            let parent_y = p1_pos
                .or(p2_pos)
                .map(|(_, y)| *y + CARD_HEIGHT_MM)
                .unwrap_or(0.0);
            let bus_y = parent_y + (VERTICAL_GAP_MM * 1.5) / 2.0;

            let mut ch_mid_xs = Vec::new();
            for ch in &fam.children {
                if let Some((cx, cy)) = comp_coords.get(&ch.child_id) {
                    let cmx = cx + CARD_WIDTH_MM / 2.0;
                    ch_mid_xs.push(cmx);
                    all_connectors.push(RenderConnector {
                        points: vec![(cmx, bus_y), (cmx, *cy)],
                        is_dashed: false,
                        color: None,
                    });
                }
            }

            if !ch_mid_xs.is_empty() {
                all_connectors.push(RenderConnector {
                    points: vec![(union_x, parent_y), (union_x, bus_y)],
                    is_dashed: false,
                    color: None,
                });

                let min_ch_x = ch_mid_xs.iter().copied().fold(union_x, f64::min);
                let max_ch_x = ch_mid_xs.iter().copied().fold(union_x, f64::max);
                all_connectors.push(RenderConnector {
                    points: vec![(min_ch_x, bus_y), (max_ch_x, bus_y)],
                    is_dashed: false,
                    color: None,
                });
            }
        }

        // Horizontal spacing between components >= 30.0 mm
        current_offset_x = comp_max_x + 30.0;
    }

    // Place singletons shelf at the bottom in a 4-column grid
    if !singletons.is_empty() {
        let shelf_y = max_bottom_y + 30.0;

        for (i, p) in singletons.iter().enumerate() {
            let col = (i % 4) as f64;
            let row = (i / 4) as f64;
            let x = MARGIN_X_MM + col * (CARD_WIDTH_MM + HORIZONTAL_GAP_MM);
            let y = shelf_y + row * (CARD_HEIGHT_MM + VERTICAL_GAP_MM);

            let is_focal = p.id == focal.id;
            let full_name = format_person_name(p);
            let lifespan = format_person_lifespan(p, redact_living, is_ru);
            let avatar_bytes =
                resolve_person_avatar(p, store, media_storage_path, redact_living, show_photos);

            all_cards.push(RenderPersonCard {
                id: p.id.clone(),
                gid: p.gid.clone(),
                full_name,
                lifespan,
                is_living: p.is_living,
                is_focal,
                sex: p.sex.clone(),
                x,
                y,
                width: CARD_WIDTH_MM,
                height: CARD_HEIGHT_MM,
                avatar_bytes,
                generation: 0,
            });
        }
    }

    (all_cards, all_connectors)
}
