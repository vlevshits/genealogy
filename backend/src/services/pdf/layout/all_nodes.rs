#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// 4. All Nodes Layout (Forest Graph View with Disjoint Set Union)
// ---------------------------------------------------------------------------

pub(crate) struct Dsu {
    pub(crate) parent: HashMap<String, String>,
}

impl Dsu {
    pub(crate) fn new(ids: &[String]) -> Self {
        let mut parent = HashMap::new();
        for id in ids {
            parent.insert(id.clone(), id.clone());
        }
        Self { parent }
    }

    pub(crate) fn find(&mut self, i: &str) -> String {
        let p = self.parent.get(i).cloned().unwrap_or_else(|| i.to_string());
        if p != i {
            let root = self.find(&p);
            self.parent.insert(i.to_string(), root.clone());
            root
        } else {
            p
        }
    }

    pub(crate) fn union(&mut self, i: &str, j: &str) {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parent.insert(root_i, root_j);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn layout_all_nodes(
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
