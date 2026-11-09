#[allow(unused_imports)]
use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn layout_hourglass(
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

            let (union_x, parent_y_start) = match (p1_pos, p2_pos) {
                (Some((x1, y1)), Some((x2, y2))) => (
                    (x1.min(*x2) + CARD_WIDTH_MM + x1.max(*x2)) / 2.0,
                    // Both parents present: drop stem starts on the marital connector line
                    y1.min(*y2) + CARD_HEIGHT_MM / 2.0,
                ),
                (Some((x1, y1)), None) => (x1 + CARD_WIDTH_MM / 2.0, y1 + CARD_HEIGHT_MM),
                (None, Some((x2, y2))) => (x2 + CARD_WIDTH_MM / 2.0, y2 + CARD_HEIGHT_MM),
                (None, None) => continue,
            };

            let parent_y_bottom = p1_pos
                .or(p2_pos)
                .map(|(_, y)| *y + CARD_HEIGHT_MM)
                .unwrap_or(0.0);
            let bus_y = parent_y_bottom + (VERTICAL_GAP_MM * 1.5) / 2.0;

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
                    points: vec![(union_x, parent_y_start), (union_x, bus_y)],
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
