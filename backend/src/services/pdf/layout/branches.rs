#[allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------
// 3. Hierarchical Hourglass & Component Layout Helpers
// ---------------------------------------------------------------------------

pub(crate) struct DescBranch {
    pub(crate) person_id: String,

    pub(crate) spouses: Vec<String>,

    pub(crate) generation: i32,

    pub(crate) couple_width: f64,

    pub(crate) width: f64,

    pub(crate) center_x: f64,

    pub(crate) children_rel_x: Vec<f64>,

    pub(crate) children: Vec<DescBranch>,
}

pub(crate) fn build_desc_branch(
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

pub(crate) fn place_desc_branch(
    branch: &DescBranch,
    base_x: f64,
    coords: &mut HashMap<String, (f64, i32)>,
) {
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

pub(crate) struct AncBranch {
    pub(crate) person_id: String,

    pub(crate) generation: i32,

    pub(crate) width: f64,

    pub(crate) center_x: f64,

    pub(crate) father: Option<(Box<AncBranch>, f64)>,

    pub(crate) mother: Option<(Box<AncBranch>, f64)>,
}

pub(crate) fn build_anc_branch(
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

pub(crate) fn place_anc_branch(
    branch: &AncBranch,
    base_x: f64,
    coords: &mut HashMap<String, (f64, i32)>,
) {
    let card_x = base_x + branch.center_x - CARD_WIDTH_MM / 2.0;
    coords.insert(branch.person_id.clone(), (card_x, branch.generation));
    if let Some((ref f, rel_x)) = branch.father {
        place_anc_branch(f, base_x + rel_x, coords);
    }
    if let Some((ref m, rel_x)) = branch.mother {
        place_anc_branch(m, base_x + rel_x, coords);
    }
}

pub(crate) struct CollateralAuntUnit {
    pub(crate) aunt_id: String,

    pub(crate) spouses: Vec<String>,

    pub(crate) cousins: Vec<String>,

    pub(crate) cousin_spouses: HashMap<String, Vec<String>>,

    pub(crate) width: f64,

    pub(crate) center_x: f64,

    pub(crate) cousins_rel_x: Vec<f64>,
}

pub(crate) fn build_collateral_aunt_unit(
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

pub(crate) fn place_collateral_aunt_unit(
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
