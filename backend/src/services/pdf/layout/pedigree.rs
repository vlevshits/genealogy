#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// 1. Pedigree Layout (Left-to-Right Binary Ancestors)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn layout_pedigree(
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
