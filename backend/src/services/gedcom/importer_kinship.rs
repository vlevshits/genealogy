#[allow(unused_imports)]
use super::*;

pub(crate) fn canonicalize_gid(prefix: char, raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('@').to_uppercase();
    if trimmed.is_empty() {
        return String::new();
    }
    let actual_prefix = trimmed.chars().next().unwrap_or(prefix);
    if actual_prefix == prefix && trimmed.len() >= 2 {
        if let Ok(num) = trimmed[1..].parse::<u32>() {
            return format!("{prefix}{num:04}");
        }
    }
    trimmed
}

pub(crate) fn looks_like_cp1251_mojibake(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 4 {
        return false;
    }
    let mut suspicious_pairs = 0usize;
    let mut i = 0;
    while i + 1 < chars.len() {
        if (chars[i] == 'Р' || chars[i] == 'С')
            && !chars[i + 1].is_ascii()
            && !('а'..='я').contains(&chars[i + 1])
            && chars[i + 1] != 'ё'
        {
            suspicious_pairs += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    suspicious_pairs >= 2
}

pub(crate) fn reserve_existing_person_uids(
    store: &GenealogyPrivacyStore,
    initial_persons_count: usize,
    roots: &[GedcomNode],
) -> (HashMap<String, usize>, HashSet<usize>) {
    let mut uid_to_idx: HashMap<String, usize> = HashMap::new();
    let mut reserved_indices: HashSet<usize> = HashSet::new();
    let bound = initial_persons_count.min(store.persons.len());

    for node in roots.iter().filter(|n| n.tag == "INDI") {
        let Some(uid_val) = node
            .child_value("_UID")
            .or_else(|| node.child_value("UID"))
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        if uid_to_idx.contains_key(&uid_val) {
            continue;
        }
        if let Some(idx) = store.persons[..bound].iter().position(|p| {
            p.external_ids
                .get("uid")
                .is_some_and(|pu| pu.eq_ignore_ascii_case(&uid_val))
        }) {
            if reserved_indices.insert(idx) {
                uid_to_idx.insert(uid_val, idx);
            }
        }
    }

    (uid_to_idx, reserved_indices)
}

pub(crate) fn reserve_existing_family_uids(
    store: &GenealogyPrivacyStore,
    initial_families_count: usize,
    roots: &[GedcomNode],
) -> (HashMap<String, usize>, HashSet<usize>) {
    let mut uid_to_idx: HashMap<String, usize> = HashMap::new();
    let mut reserved_indices: HashSet<usize> = HashSet::new();
    let bound = initial_families_count.min(store.families.len());

    for node in roots.iter().filter(|n| n.tag == "FAM") {
        let Some(uid_val) = node
            .child_value("_UID")
            .or_else(|| node.child_value("UID"))
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        if uid_to_idx.contains_key(&uid_val) {
            continue;
        }
        if let Some(idx) = store.families[..bound].iter().position(|f| {
            f.external_ids
                .get("uid")
                .is_some_and(|fu| fu.eq_ignore_ascii_case(&uid_val))
        }) {
            if reserved_indices.insert(idx) {
                uid_to_idx.insert(uid_val, idx);
            }
        }
    }

    (uid_to_idx, reserved_indices)
}

#[derive(Debug, Clone)]
pub(crate) struct PrecomputedPersonMatch {
    pub(crate) existing_idx: usize,
    pub(crate) kind: PersonMatchKind,
    pub(crate) preview_entry: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PrecomputedMergePlan {
    pub(crate) matches_by_xref: HashMap<String, PrecomputedPersonMatch>,
    pub(crate) matched_order_xrefs: Vec<String>,
    pub(crate) chronology_conflicts_prevented: usize,
}

#[derive(Debug, Clone)]
struct IncomingIndiSummary {
    xref_clean: String,
    given_name: String,
    patronymic: Option<String>,
    surname: String,
    birth_surname: Option<String>,
    sex: String,
    birth_year: Option<i32>,
    death_year: Option<i32>,
    min_child_birth_year: Option<i32>,
    max_parent_birth_year: Option<i32>,
    alt_names: Vec<IndividualAltNameDto>,
    external_ids: HashMap<String, String>,
}

fn summarize_incoming_indi(node: &GedcomNode) -> IncomingIndiSummary {
    let xref_clean = node
        .xref
        .as_deref()
        .map(|x| x.trim_matches('@').to_uppercase())
        .unwrap_or_default();

    let mut external_ids: HashMap<String, String> = HashMap::new();
    for child in &node.children {
        if matches!(child.tag.as_str(), "_UID" | "UID") {
            let uid_val = child.value.trim().to_string();
            if !uid_val.is_empty() {
                external_ids.insert("uid".to_string(), uid_val);
            }
        }
    }

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
                surname = marnm.clone();
                alt_names.push(IndividualAltNameDto {
                    id: String::new(),
                    name_type: "married".to_string(),
                    given_name: Some(given_name.clone()),
                    patronymic: patronymic.clone(),
                    surname: Some(marnm.clone()),
                    lang: None,
                });
            }
        }
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
            alt_names.push(IndividualAltNameDto {
                id: String::new(),
                name_type: "married".to_string(),
                given_name: Some(given_name.clone()),
                patronymic: patronymic.clone(),
                surname: Some(m_name.surname.clone()),
                lang: None,
            });
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
    let (birth_year, death_year) = extract_node_vital_years(node);

    IncomingIndiSummary {
        xref_clean,
        given_name,
        patronymic,
        surname,
        birth_surname,
        sex,
        birth_year,
        death_year,
        min_child_birth_year: None,
        max_parent_birth_year: None,
        alt_names,
        external_ids,
    }
}

pub(crate) fn precompute_person_matches(
    store: &GenealogyPrivacyStore,
    initial_persons_count: usize,
    initial_families_count: usize,
    roots: &[GedcomNode],
) -> PrecomputedMergePlan {
    let mut plan = PrecomputedMergePlan::default();
    let person_bound = initial_persons_count.min(store.persons.len());
    let fam_bound = initial_families_count.min(store.families.len());
    if person_bound == 0 {
        return plan;
    }

    let existing_vitals: Vec<(Option<i32>, Option<i32>)> = store.persons[..person_bound]
        .iter()
        .map(|p| existing_person_vital_years(p, &store.events))
        .collect();
    let person_id_to_idx: HashMap<&str, usize> = store.persons[..person_bound]
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.as_str(), i))
        .collect();

    let mut existing_family_bounds: Vec<(Option<i32>, Option<i32>)> =
        vec![(None, None); person_bound];
    for fam in &store.families[..fam_bound] {
        let p_indices: Vec<usize> = [fam.partner1_id.as_deref(), fam.partner2_id.as_deref()]
            .into_iter()
            .flatten()
            .filter_map(|pid| person_id_to_idx.get(pid).copied())
            .collect();
        let c_indices: Vec<usize> = fam
            .children
            .iter()
            .filter_map(|c| person_id_to_idx.get(c.child_id.as_str()).copied())
            .collect();
        let min_child_by = c_indices
            .iter()
            .filter_map(|&ci| existing_vitals[ci].0)
            .min();
        let max_parent_by = p_indices
            .iter()
            .filter_map(|&pi| existing_vitals[pi].0)
            .max();
        if let Some(mcy) = min_child_by {
            for &pi in &p_indices {
                let cur = &mut existing_family_bounds[pi].0;
                *cur = Some(cur.map_or(mcy, |v| v.min(mcy)));
            }
        }
        if let Some(mpy) = max_parent_by {
            for &ci in &c_indices {
                let cur = &mut existing_family_bounds[ci].1;
                *cur = Some(cur.map_or(mpy, |v| v.max(mpy)));
            }
        }
    }

    let mut indis: Vec<IncomingIndiSummary> = roots
        .iter()
        .filter(|n| n.tag == "INDI")
        .map(summarize_incoming_indi)
        .collect();
    let xref_to_indi_idx: HashMap<String, usize> = indis
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.xref_clean.is_empty())
        .map(|(i, s)| (s.xref_clean.clone(), i))
        .collect();

    for fam_node in roots.iter().filter(|n| n.tag == "FAM") {
        let p_xrefs: Vec<usize> = ["HUSB", "WIFE"]
            .into_iter()
            .filter_map(|t| fam_node.child_value(t))
            .map(|x| x.trim_matches('@').to_uppercase())
            .filter_map(|x| xref_to_indi_idx.get(&x).copied())
            .collect();
        let c_xrefs: Vec<usize> = fam_node
            .children
            .iter()
            .filter(|c| c.tag == "CHIL")
            .map(|c| c.value.trim().trim_matches('@').to_uppercase())
            .filter_map(|x| xref_to_indi_idx.get(&x).copied())
            .collect();
        let min_cy = c_xrefs.iter().filter_map(|&ci| indis[ci].birth_year).min();
        let max_py = p_xrefs.iter().filter_map(|&pi| indis[pi].birth_year).max();
        if let Some(mcy) = min_cy {
            for &pi in &p_xrefs {
                let cur = &mut indis[pi].min_child_birth_year;
                *cur = Some(cur.map_or(mcy, |v| v.min(mcy)));
            }
        }
        if let Some(mpy) = max_py {
            for &ci in &c_xrefs {
                let cur = &mut indis[ci].max_parent_birth_year;
                *cur = Some(cur.map_or(mpy, |v| v.max(mpy)));
            }
        }
    }

    let (reserved_uid_person_indices, reserved_person_indices) =
        reserve_existing_person_uids(store, initial_persons_count, roots);
    let mut matched_person_indices: HashSet<usize> = HashSet::new();

    for summary in &indis {
        if summary.xref_clean.is_empty() {
            continue;
        }
        let cand = IncomingPersonCandidate {
            xref_clean: &summary.xref_clean,
            given_name: &summary.given_name,
            patronymic: summary.patronymic.as_deref(),
            surname: &summary.surname,
            birth_surname: summary.birth_surname.as_deref(),
            nickname: None,
            name_prefix: None,
            name_suffix: None,
            sex: &summary.sex,
            birth_year: summary.birth_year,
            death_year: summary.death_year,
            min_child_birth_year: summary.min_child_birth_year,
            max_parent_birth_year: summary.max_parent_birth_year,
            alt_names: &summary.alt_names,
            external_ids: &summary.external_ids,
        };
        if let Some((idx, kind, reason)) = find_matching_person_index(
            store,
            initial_persons_count,
            &cand,
            &reserved_uid_person_indices,
            &reserved_person_indices,
            &matched_person_indices,
            &existing_family_bounds,
            &mut plan.chronology_conflicts_prevented,
        ) {
            matched_person_indices.insert(idx);
            let ext_gid = &store.persons[idx].gid;
            let disp = format!("{} {}", summary.given_name, summary.surname)
                .trim()
                .to_string();
            plan.matched_order_xrefs.push(summary.xref_clean.clone());
            plan.matches_by_xref.insert(
                summary.xref_clean.clone(),
                PrecomputedPersonMatch {
                    existing_idx: idx,
                    kind,
                    preview_entry: format!(
                        "{disp} ({} → {ext_gid}) [{reason}]",
                        summary.xref_clean
                    ),
                },
            );
        }
    }

    // Pass C: Iterative kinship propagation across FAM records
    let mut changed = true;
    while changed {
        changed = false;
        for fam_node in roots.iter().filter(|n| n.tag == "FAM") {
            let fam_uid = fam_node
                .child_value("_UID")
                .or_else(|| fam_node.child_value("UID"))
                .filter(|u| !u.is_empty());
            let husb_xref = fam_node
                .child_value("HUSB")
                .map(|x| x.trim_matches('@').to_uppercase())
                .filter(|x| !x.is_empty());
            let wife_xref = fam_node
                .child_value("WIFE")
                .map(|x| x.trim_matches('@').to_uppercase())
                .filter(|x| !x.is_empty());
            let chil_xrefs: Vec<String> = fam_node
                .children
                .iter()
                .filter(|c| c.tag == "CHIL")
                .map(|c| c.value.trim().trim_matches('@').to_uppercase())
                .filter(|x| !x.is_empty())
                .collect();

            let matched_husb_pid = husb_xref
                .as_ref()
                .and_then(|x| plan.matches_by_xref.get(x))
                .map(|m| store.persons[m.existing_idx].id.as_str());
            let matched_wife_pid = wife_xref
                .as_ref()
                .and_then(|x| plan.matches_by_xref.get(x))
                .map(|m| store.persons[m.existing_idx].id.as_str());
            let matched_child_pids: HashSet<&str> = chil_xrefs
                .iter()
                .filter_map(|x| plan.matches_by_xref.get(x))
                .map(|m| store.persons[m.existing_idx].id.as_str())
                .collect();

            let Some(ext_fam) = store.families[..fam_bound].iter().find(|ef| {
                let ef_uid = ef.external_ids.get("uid").filter(|u| !u.is_empty());
                if let (Some(fu), Some(iu)) = (ef_uid, fam_uid.as_deref()) {
                    if fu.eq_ignore_ascii_case(iu) {
                        return true;
                    }
                    return false;
                }
                let fp1 = ef.partner1_id.as_deref();
                let fp2 = ef.partner2_id.as_deref();
                if let (Some(mh), Some(mw), Some(p1), Some(p2)) =
                    (matched_husb_pid, matched_wife_pid, fp1, fp2)
                {
                    if (mh == p1 && mw == p2) || (mh == p2 && mw == p1) {
                        return true;
                    }
                }
                let shared_children = ef
                    .children
                    .iter()
                    .filter(|c| matched_child_pids.contains(c.child_id.as_str()))
                    .count();
                let one_partner_matches = (matched_husb_pid.is_some()
                    && (matched_husb_pid == fp1 || matched_husb_pid == fp2))
                    || (matched_wife_pid.is_some()
                        && (matched_wife_pid == fp1 || matched_wife_pid == fp2));
                if one_partner_matches && shared_children >= 1 {
                    return true;
                }
                let parents_given_compatible = {
                    let h_ok = match (&husb_xref, fp1) {
                        (Some(hx), Some(p1_id)) => {
                            let inc_g = xref_to_indi_idx
                                .get(hx)
                                .map(|&i| indis[i].given_name.as_str())
                                .unwrap_or("");
                            let ext_g = person_id_to_idx
                                .get(p1_id)
                                .map(|&i| store.persons[i].primary_name.given_name.as_str())
                                .unwrap_or("");
                            given_names_match(inc_g, ext_g)
                        }
                        _ => true,
                    };
                    let w_ok = match (&wife_xref, fp2) {
                        (Some(wx), Some(p2_id)) => {
                            let inc_g = xref_to_indi_idx
                                .get(wx)
                                .map(|&i| indis[i].given_name.as_str())
                                .unwrap_or("");
                            let ext_g = person_id_to_idx
                                .get(p2_id)
                                .map(|&i| store.persons[i].primary_name.given_name.as_str())
                                .unwrap_or("");
                            given_names_match(inc_g, ext_g)
                        }
                        _ => true,
                    };
                    h_ok && w_ok
                };
                if one_partner_matches
                    && parents_given_compatible
                    && husb_xref.is_some()
                    && wife_xref.is_some()
                    && fp1.is_some()
                    && fp2.is_some()
                {
                    return true;
                }
                shared_children >= 1 && parents_given_compatible
            }) else {
                continue;
            };

            let try_kinship_match = |inc_xref: &str,
                                     ext_idx: usize,
                                     role_label: &str,
                                     plan: &mut PrecomputedMergePlan,
                                     matched_person_indices: &mut HashSet<usize>|
             -> bool {
                if plan.matches_by_xref.contains_key(inc_xref)
                    || matched_person_indices.contains(&ext_idx)
                    || reserved_person_indices.contains(&ext_idx)
                {
                    return false;
                }
                let Some(&inc_idx) = xref_to_indi_idx.get(inc_xref) else {
                    return false;
                };
                let inc_s = &indis[inc_idx];
                let ext_p = &store.persons[ext_idx];
                if let (Some(pu), Some(cu)) = (
                    ext_p.external_ids.get("uid").filter(|v| !v.is_empty()),
                    inc_s.external_ids.get("uid").filter(|v| !v.is_empty()),
                ) {
                    if !pu.eq_ignore_ascii_case(cu) {
                        return false;
                    }
                }
                if !given_names_match(&ext_p.primary_name.given_name, &inc_s.given_name) {
                    return false;
                }
                let p_sex = ext_p.sex.trim().to_ascii_uppercase();
                let c_sex = inc_s.sex.trim().to_ascii_uppercase();
                if (p_sex == "M" || p_sex == "F")
                    && (c_sex == "M" || c_sex == "F")
                    && p_sex != c_sex
                {
                    return false;
                }
                let p_patr = ext_p
                    .primary_name
                    .patronymic
                    .as_deref()
                    .map(normalize_name_token)
                    .filter(|s| !s.is_empty());
                let c_patr = inc_s
                    .patronymic
                    .as_deref()
                    .map(normalize_name_token)
                    .filter(|s| !s.is_empty());
                if let (Some(ep), Some(cp)) = (p_patr, c_patr) {
                    if ep != cp {
                        return false;
                    }
                }
                let (p_by, _) = existing_vitals[ext_idx];
                let (p_min_cy, p_max_py) = existing_family_bounds[ext_idx];
                if has_chronology_conflict(
                    p_by,
                    p_min_cy,
                    p_max_py,
                    inc_s.birth_year,
                    inc_s.min_child_birth_year,
                    inc_s.max_parent_birth_year,
                ) {
                    return false;
                }
                matched_person_indices.insert(ext_idx);
                let disp = format!("{} {}", inc_s.given_name, inc_s.surname)
                    .trim()
                    .to_string();
                plan.matched_order_xrefs.push(inc_xref.to_string());
                plan.matches_by_xref.insert(
                    inc_xref.to_string(),
                    PrecomputedPersonMatch {
                        existing_idx: ext_idx,
                        kind: PersonMatchKind::Kinship,
                        preview_entry: format!(
                            "{disp} ({inc_xref} → {}) [Kinship: {role_label} in {}]",
                            ext_p.gid, ext_fam.gid
                        ),
                    },
                );
                true
            };

            if let (Some(hx), Some(p1_id)) = (&husb_xref, ext_fam.partner1_id.as_deref()) {
                if let Some(&ext_idx) = person_id_to_idx.get(p1_id) {
                    if try_kinship_match(
                        hx,
                        ext_idx,
                        "Partner",
                        &mut plan,
                        &mut matched_person_indices,
                    ) {
                        changed = true;
                    }
                }
            }
            if let (Some(wx), Some(p2_id)) = (&wife_xref, ext_fam.partner2_id.as_deref()) {
                if let Some(&ext_idx) = person_id_to_idx.get(p2_id) {
                    if try_kinship_match(
                        wx,
                        ext_idx,
                        "Partner",
                        &mut plan,
                        &mut matched_person_indices,
                    ) {
                        changed = true;
                    }
                }
            }
            for ch_xref in &chil_xrefs {
                if plan.matches_by_xref.contains_key(ch_xref) {
                    continue;
                }
                for ext_ch in &ext_fam.children {
                    if let Some(&ext_idx) = person_id_to_idx.get(ext_ch.child_id.as_str()) {
                        if try_kinship_match(
                            ch_xref,
                            ext_idx,
                            "Child",
                            &mut plan,
                            &mut matched_person_indices,
                        ) {
                            changed = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    plan
}
