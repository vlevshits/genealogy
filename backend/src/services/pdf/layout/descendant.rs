#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// 2. Descendant Layout (Top-Down with Couples & Midpoint Drops)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn layout_descendant(
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
            return (x + CARD_WIDTH_MM / 2.0, y);
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
            return (x + CARD_WIDTH_MM / 2.0, y);
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

        (x + CARD_WIDTH_MM / 2.0, y)
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
                let parent_bot_y = if p.spouse_id.is_some() {
                    p.y + CARD_HEIGHT_MM / 2.0
                } else {
                    p.y + CARD_HEIGHT_MM
                };
                let mid_y = p.y + CARD_HEIGHT_MM + (VERTICAL_GAP_MM * 1.5 / 2.0);

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
