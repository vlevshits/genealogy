#[allow(unused_imports)]
use super::*;

/// Connected-component and family-relationship graph for relationship-centric duplicate detection (`15 §3.1`).
#[derive(Debug, Clone)]
pub struct TreeRelationshipGraph {
    pub component_by_person: HashMap<String, usize>,
    pub component_sizes: HashMap<usize, usize>,
    pub is_singleton: HashMap<String, bool>,
    pub parents_by_person: HashMap<String, HashSet<String>>,
    pub spouses_by_person: HashMap<String, HashSet<String>>,
    pub children_by_person: HashMap<String, HashSet<String>>,
    pub siblings_by_person: HashMap<String, HashSet<String>>,
}

impl TreeRelationshipGraph {
    pub fn build(store: &GenealogyPrivacyStore) -> Self {
        let mut adj: HashMap<String, HashSet<String>> = HashMap::new();
        let mut parents_by_person: HashMap<String, HashSet<String>> = HashMap::new();
        let mut spouses_by_person: HashMap<String, HashSet<String>> = HashMap::new();
        let mut children_by_person: HashMap<String, HashSet<String>> = HashMap::new();
        let mut siblings_by_person: HashMap<String, HashSet<String>> = HashMap::new();

        for p in &store.persons {
            adj.entry(p.id.clone()).or_default();
        }

        for fam in &store.families {
            let p1 = fam.partner1_id.as_deref().filter(|s| !s.is_empty());
            let p2 = fam.partner2_id.as_deref().filter(|s| !s.is_empty());

            if let (Some(a), Some(b)) = (p1, p2) {
                if a != b {
                    adj.entry(a.to_string()).or_default().insert(b.to_string());
                    adj.entry(b.to_string()).or_default().insert(a.to_string());
                    spouses_by_person
                        .entry(a.to_string())
                        .or_default()
                        .insert(b.to_string());
                    spouses_by_person
                        .entry(b.to_string())
                        .or_default()
                        .insert(a.to_string());
                }
            }

            for ch in &fam.children {
                let cid = ch.child_id.as_str();
                if cid.is_empty() {
                    continue;
                }
                for parent in [p1, p2].into_iter().flatten() {
                    if parent != cid {
                        adj.entry(parent.to_string())
                            .or_default()
                            .insert(cid.to_string());
                        adj.entry(cid.to_string())
                            .or_default()
                            .insert(parent.to_string());
                        parents_by_person
                            .entry(cid.to_string())
                            .or_default()
                            .insert(parent.to_string());
                        children_by_person
                            .entry(parent.to_string())
                            .or_default()
                            .insert(cid.to_string());
                    }
                }
                for other_ch in &fam.children {
                    let ocid = other_ch.child_id.as_str();
                    if !ocid.is_empty() && ocid != cid {
                        adj.entry(cid.to_string())
                            .or_default()
                            .insert(ocid.to_string());
                        siblings_by_person
                            .entry(cid.to_string())
                            .or_default()
                            .insert(ocid.to_string());
                    }
                }
            }
        }

        let mut component_by_person = HashMap::new();
        let mut component_sizes = HashMap::new();
        let mut is_singleton = HashMap::new();
        let mut next_comp_id = 1usize;

        for p in &store.persons {
            let neighbors = adj.get(&p.id).map(|n| n.len()).unwrap_or(0);
            is_singleton.insert(p.id.clone(), neighbors == 0);
            if component_by_person.contains_key(&p.id) {
                continue;
            }
            let comp_id = next_comp_id;
            next_comp_id += 1;
            let mut queue = VecDeque::new();
            let mut comp_nodes = Vec::new();
            queue.push_back(p.id.clone());
            component_by_person.insert(p.id.clone(), comp_id);

            while let Some(curr) = queue.pop_front() {
                comp_nodes.push(curr.clone());
                if let Some(neighs) = adj.get(&curr) {
                    for nb in neighs {
                        if !component_by_person.contains_key(nb) {
                            component_by_person.insert(nb.clone(), comp_id);
                            queue.push_back(nb.clone());
                        }
                    }
                }
            }
            component_sizes.insert(comp_id, comp_nodes.len());
        }

        Self {
            component_by_person,
            component_sizes,
            is_singleton,
            parents_by_person,
            spouses_by_person,
            children_by_person,
            siblings_by_person,
        }
    }

    /// Checks if `candidate_ancestor_id` is a direct ancestor of `descendant_id` within `max_generations`.
    pub fn is_ancestor_within(
        &self,
        candidate_ancestor_id: &str,
        descendant_id: &str,
        max_generations: usize,
    ) -> bool {
        if candidate_ancestor_id == descendant_id {
            return false;
        }
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((descendant_id.to_string(), 0usize));
        visited.insert(descendant_id.to_string());

        while let Some((curr, depth)) = queue.pop_front() {
            if depth >= max_generations {
                continue;
            }
            if let Some(parents) = self.parents_by_person.get(&curr) {
                for p in parents {
                    if p == candidate_ancestor_id {
                        return true;
                    }
                    if visited.insert(p.clone()) {
                        queue.push_back((p.clone(), depth + 1));
                    }
                }
            }
        }
        false
    }
}

pub(crate) fn names_match_between_persons(a: &TreePersonNodeDto, b: &TreePersonNodeDto) -> bool {
    let givens_a = collect_person_given_names(a);
    let givens_b = collect_person_given_names(b);
    let mut given_ok = false;
    for ga in &givens_a {
        for gb in &givens_b {
            if dice_similarity(ga, gb) >= 0.72 {
                given_ok = true;
                break;
            }
        }
    }
    if !given_ok {
        return false;
    }

    let surnames_a = collect_person_surnames(a);
    let surnames_b = collect_person_surnames(b);
    if surnames_a.is_empty() || surnames_b.is_empty() {
        return true;
    }
    for sa in &surnames_a {
        for sb in &surnames_b {
            if dice_similarity(sa, sb) >= 0.72 {
                return true;
            }
        }
    }
    false
}

pub(crate) fn patronymic_aligns_with_father(
    patronymic: &str,
    father: &TreePersonNodeDto,
    child_sex: &str,
) -> bool {
    let pat_norm = normalize_cross_script_token(patronymic);
    if pat_norm.is_empty() {
        return false;
    }
    if let Some(suggested) = suggest_slavic_patronymic(&father.primary_name.given_name, child_sex) {
        let sug_norm = normalize_cross_script_token(&suggested);
        if !sug_norm.is_empty() && dice_similarity(&pat_norm, &sug_norm) >= 0.80 {
            return true;
        }
    }
    let father_given = normalize_cross_script_token(&father.primary_name.given_name);
    father_given.len() >= 3 && pat_norm.starts_with(&father_given[..father_given.len().min(4)])
}

/// Scores two person records for duplicate likelihood (`0..=100`) and returns human-readable reason codes (`15 §3.1`).
pub fn score_duplicate_pair(
    a: &TreePersonNodeDto,
    b: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
) -> (i32, Vec<String>) {
    let graph = TreeRelationshipGraph::build(store);
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();
    score_duplicate_pair_with_graph(a, b, store, &graph, &person_by_id)
}

pub fn score_duplicate_pair_with_graph(
    a: &TreePersonNodeDto,
    b: &TreePersonNodeDto,
    store: &GenealogyPrivacyStore,
    graph: &TreeRelationshipGraph,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
) -> (i32, Vec<String>) {
    if a.id == b.id {
        return (0, Vec::new());
    }

    // --- 0. Structural Kinship Disqualification (`15-data-quality-duplicates.md §3.1.1`) ---
    // (a) Direct Ancestor / Descendant (<= 8 generations): Father/son or Grandfather/grandson with identical name
    if graph.is_ancestor_within(&a.id, &b.id, 8) || graph.is_ancestor_within(&b.id, &a.id, 8) {
        return (0, Vec::new());
    }

    // (b) Spouses / Partners in the same union
    if graph
        .spouses_by_person
        .get(&a.id)
        .is_some_and(|s| s.contains(&b.id))
    {
        return (0, Vec::new());
    }

    let b_win_a = person_birth_window(a, &store.events);
    let b_win_b = person_birth_window(b, &store.events);
    let d_win_a = person_death_window(a, &store.events);
    let d_win_b = person_death_window(b, &store.events);

    // (c) Non-overlapping lifespans (`Death(A) < Birth(B) - 1` or `Death(B) < Birth(A) - 1`)
    if let (Some(da), Some(bb)) = (d_win_a, b_win_b) {
        if da.max_year < bb.min_year - 1 {
            return (0, Vec::new());
        }
    }
    if let (Some(db), Some(ba)) = (d_win_b, b_win_a) {
        if db.max_year < ba.min_year - 1 {
            return (0, Vec::new());
        }
    }

    // 1. Surname comparison (including birth_surname, gender-folded Slavic endings, cross-script)
    let surnames_a = collect_person_surnames(a);
    let surnames_b = collect_person_surnames(b);
    let mut best_surname_sim = 0.0_f64;
    for sa in &surnames_a {
        for sb in &surnames_b {
            let sim = dice_similarity(sa, sb);
            if sim > best_surname_sim {
                best_surname_sim = sim;
            }
        }
    }

    // 2. Given name comparison
    let givens_a = collect_person_given_names(a);
    let givens_b = collect_person_given_names(b);
    let mut best_given_sim = 0.0_f64;
    for ga in &givens_a {
        for gb in &givens_b {
            let sim = dice_similarity(ga, gb);
            if sim > best_given_sim {
                best_given_sim = sim;
            }
        }
    }

    let both_have_given = !givens_a.is_empty() && !givens_b.is_empty();
    let both_have_surnames = !surnames_a.is_empty() && !surnames_b.is_empty();

    if both_have_given && both_have_surnames && best_given_sim < 0.60 && best_surname_sim < 0.60 {
        return (0, Vec::new());
    }

    let sex_a = a.sex.trim().to_ascii_uppercase();
    let sex_b = b.sex.trim().to_ascii_uppercase();
    let conflicting_sex = (sex_a == "M" && sex_b == "F") || (sex_a == "F" && sex_b == "M");

    let exact_birth_a = person_exact_birth_date(a, &store.events);
    let exact_birth_b = person_exact_birth_date(b, &store.events);
    let distinct_exact_birth = match (exact_birth_a, exact_birth_b) {
        (Some(da), Some(db)) => (da - db).num_days().abs() > 30,
        _ => false,
    };
    let distinct_birth_years = match (b_win_a, b_win_b) {
        (Some(wa), Some(wb)) => wa.max_year < wb.min_year || wb.max_year < wa.min_year,
        _ => false,
    };

    let conflicting_patronymics = match (
        a.primary_name.patronymic.as_deref(),
        b.primary_name.patronymic.as_deref(),
    ) {
        (Some(pa), Some(pb)) => {
            let na = normalize_cross_script_token(pa);
            let nb = normalize_cross_script_token(pb);
            !na.is_empty() && !nb.is_empty() && dice_similarity(&na, &nb) < 0.55
        }
        _ => false,
    };

    // (d) Confirmed Distinct Siblings in Same Family
    let are_siblings_in_same_family = graph
        .siblings_by_person
        .get(&a.id)
        .is_some_and(|sibs| sibs.contains(&b.id));
    if are_siblings_in_same_family
        && (distinct_exact_birth
            || distinct_birth_years
            || conflicting_sex
            || (both_have_given && best_given_sim < 0.60)
            || conflicting_patronymics)
    {
        return (0, Vec::new());
    }

    // Connected component & singleton status
    let is_singleton_a = graph.is_singleton.get(&a.id).copied().unwrap_or(true);
    let is_singleton_b = graph.is_singleton.get(&b.id).copied().unwrap_or(true);
    let comp_a = graph.component_by_person.get(&a.id).copied();
    let comp_b = graph.component_by_person.get(&b.id).copied();
    let same_component = !is_singleton_a && !is_singleton_b && comp_a.is_some() && comp_a == comp_b;
    let is_detached = is_singleton_a || is_singleton_b || !same_component;

    // (e) Disjoint Confirmed Biological Parents in Same Connected Component
    let parents_a = graph.parents_by_person.get(&a.id);
    let parents_b = graph.parents_by_person.get(&b.id);
    let mut matching_parents_count = 0usize;
    if let (Some(pa_set), Some(pb_set)) = (parents_a, parents_b) {
        if !pa_set.is_empty() && !pb_set.is_empty() {
            for pa_id in pa_set {
                if pb_set.contains(pa_id) {
                    matching_parents_count += 1;
                } else if let Some(pa_node) = person_by_id.get(pa_id.as_str()) {
                    let matches_any_pb = pb_set.iter().any(|pb_id| {
                        person_by_id
                            .get(pb_id.as_str())
                            .is_some_and(|pb_node| names_match_between_persons(pa_node, pb_node))
                    });
                    if matches_any_pb {
                        matching_parents_count += 1;
                    }
                }
            }
            if matching_parents_count == 0 && same_component {
                return (0, Vec::new());
            }
        }
    }

    let mut score: i32 = 0;
    let mut reasons: Vec<String> = Vec::new();

    // --- 1. Name Similarity (Capped at max 40 points: 20 surname + 20 given name) ---
    if best_surname_sim >= 0.99 {
        score += 20;
        reasons.push("same_surname".to_string());
    } else if best_surname_sim >= 0.72 {
        score += 14;
        reasons.push("similar_surname".to_string());
    }

    if best_given_sim >= 0.99 {
        score += 20;
        reasons.push("same_given_name".to_string());
    } else if best_given_sim >= 0.70 {
        score += 14;
        reasons.push("similar_given_name".to_string());
    }

    if both_have_given && best_given_sim < 0.60 {
        score -= 40;
    }

    // --- 2. Relational Evidence (`15 §3.1.2`) ---
    let mut has_relational_corroboration = false;

    // 2a. Parents & Patronymic Alignment (+35 for 2 parents, +20 for 1 parent or patronymic alignment)
    if matching_parents_count >= 2 {
        score += 35;
        has_relational_corroboration = true;
        reasons.push("shared_parents".to_string());
        reasons.push("shared_relatives".to_string());
    } else if matching_parents_count == 1 {
        score += 20;
        has_relational_corroboration = true;
        reasons.push("shared_parents".to_string());
        reasons.push("shared_relatives".to_string());
    } else if let (Some(pa_set), Some(pb_set)) = (parents_a, parents_b) {
        if !pa_set.is_empty() && !pb_set.is_empty() {
            // Disjoint known parents across different components
            score -= 35;
        }
    }

    // Check patronymic comparison & father's given name alignment
    let mut patronymic_matched = false;
    if let (Some(pa), Some(pb)) = (
        a.primary_name.patronymic.as_deref(),
        b.primary_name.patronymic.as_deref(),
    ) {
        let na = normalize_cross_script_token(pa);
        let nb = normalize_cross_script_token(pb);
        if !na.is_empty() && !nb.is_empty() {
            let sim = dice_similarity(&na, &nb);
            if sim >= 0.85 {
                patronymic_matched = true;
                reasons.push("same_patronymic".to_string());
                reasons.push("patronymic_alignment".to_string());
                if matching_parents_count == 0 {
                    score += 20;
                } else {
                    score += 8;
                }
            } else if sim < 0.55 {
                score -= 25;
            }
        }
    }
    if !patronymic_matched {
        // Check if A's patronymic aligns with B's father or B's patronymic aligns with A's father
        let a_pat_matches_b_father = a.primary_name.patronymic.as_deref().is_some_and(|pat| {
            parents_b.is_some_and(|pbs| {
                pbs.iter().any(|pid| {
                    person_by_id.get(pid.as_str()).is_some_and(|f| {
                        f.sex.eq_ignore_ascii_case("M")
                            && patronymic_aligns_with_father(pat, f, &a.sex)
                    })
                })
            })
        });
        let b_pat_matches_a_father = b.primary_name.patronymic.as_deref().is_some_and(|pat| {
            parents_a.is_some_and(|pas| {
                pas.iter().any(|pid| {
                    person_by_id.get(pid.as_str()).is_some_and(|f| {
                        f.sex.eq_ignore_ascii_case("M")
                            && patronymic_aligns_with_father(pat, f, &b.sex)
                    })
                })
            })
        });
        if a_pat_matches_b_father || b_pat_matches_a_father {
            has_relational_corroboration = true;
            reasons.push("patronymic_alignment".to_string());
            if matching_parents_count == 0 {
                score += 20;
            }
        }
    }

    // 2b. Shared or Matching Spouse(s) (+25)
    let spouses_a = graph.spouses_by_person.get(&a.id);
    let spouses_b = graph.spouses_by_person.get(&b.id);
    if let (Some(sa_set), Some(sb_set)) = (spouses_a, spouses_b) {
        let spouse_match = sa_set.iter().any(|sa_id| {
            if sb_set.contains(sa_id) {
                return true;
            }
            person_by_id.get(sa_id.as_str()).is_some_and(|sa_node| {
                sb_set.iter().any(|sb_id| {
                    person_by_id
                        .get(sb_id.as_str())
                        .is_some_and(|sb_node| names_match_between_persons(sa_node, sb_node))
                })
            })
        });
        if spouse_match {
            score += 25;
            has_relational_corroboration = true;
            reasons.push("shared_spouse".to_string());
            if !reasons.contains(&"shared_relatives".to_string()) {
                reasons.push("shared_relatives".to_string());
            }
        }
    }

    // 2c. Shared or Matching Children (+20)
    let children_a = graph.children_by_person.get(&a.id);
    let children_b = graph.children_by_person.get(&b.id);
    if let (Some(ca_set), Some(cb_set)) = (children_a, children_b) {
        let child_match = ca_set.iter().any(|ca_id| {
            if cb_set.contains(ca_id) {
                return true;
            }
            person_by_id.get(ca_id.as_str()).is_some_and(|ca_node| {
                cb_set.iter().any(|cb_id| {
                    person_by_id
                        .get(cb_id.as_str())
                        .is_some_and(|cb_node| names_match_between_persons(ca_node, cb_node))
                })
            })
        });
        if child_match {
            score += 20;
            has_relational_corroboration = true;
            reasons.push("shared_children".to_string());
            if !reasons.contains(&"shared_relatives".to_string()) {
                reasons.push("shared_relatives".to_string());
            }
        }
    }

    // 2d. Shared or Matching Siblings across detached subtrees (+15)
    let sibs_a = graph.siblings_by_person.get(&a.id);
    let sibs_b = graph.siblings_by_person.get(&b.id);
    if is_detached {
        if let (Some(sa_set), Some(sb_set)) = (sibs_a, sibs_b) {
            let sib_match = sa_set.iter().any(|sa_id| {
                if sb_set.contains(sa_id) {
                    return true;
                }
                person_by_id.get(sa_id.as_str()).is_some_and(|sa_node| {
                    sb_set.iter().any(|sb_id| {
                        person_by_id
                            .get(sb_id.as_str())
                            .is_some_and(|sb_node| names_match_between_persons(sa_node, sb_node))
                    })
                })
            });
            if sib_match {
                score += 15;
                has_relational_corroboration = true;
                reasons.push("shared_siblings".to_string());
            }
        }
    }

    // --- 3. Vital Dates & Places ---
    if let (Some(wa), Some(wb)) = (b_win_a, b_win_b) {
        let year_diff = if wa.max_year < wb.min_year {
            wb.min_year - wa.max_year
        } else if wb.max_year < wa.min_year {
            wa.min_year - wb.max_year
        } else {
            0
        };
        if year_diff == 0 {
            score += 15;
            reasons.push("same_birth_year".to_string());
        } else if year_diff <= 2 {
            score += 12;
            reasons.push("close_birth_year".to_string());
            reasons.push("same_birth_year".to_string());
        } else if year_diff <= 5 {
            score += 6;
            reasons.push("approximate_birth_year".to_string());
        } else if year_diff > 15 {
            score -= 40;
        }
    }

    // Exact Birth Date Conflict Guard (> 30 days difference)
    if distinct_exact_birth {
        score -= 30;
    }

    // Birth place comparison (+8)
    if let (Some(pa), Some(pb)) = (
        a.birth.as_ref().and_then(|v| v.place.as_deref()),
        b.birth.as_ref().and_then(|v| v.place.as_deref()),
    ) {
        let na = normalize_cross_script_token(pa);
        let nb = normalize_cross_script_token(pb);
        if !na.is_empty() && !nb.is_empty() && (na == nb || na.contains(&nb) || nb.contains(&na)) {
            score += 8;
            reasons.push("same_birth_place".to_string());
        }
    }

    // Death year comparison (+8)
    if let (Some(da), Some(db)) = (d_win_a, d_win_b) {
        let death_diff = if da.max_year < db.min_year {
            db.min_year - da.max_year
        } else if db.max_year < da.min_year {
            da.min_year - db.max_year
        } else {
            0
        };
        if death_diff <= 2 {
            score += 8;
            reasons.push("same_death_year".to_string());
        } else if death_diff > 15 {
            score -= 30;
        }
    }

    // --- 4. Detached Component / Unconnected Singleton Context Bonus (+25) ---
    if is_detached && best_given_sim >= 0.70 && (best_surname_sim >= 0.72 || !both_have_surnames) {
        score += 25;
        if is_singleton_a || is_singleton_b {
            reasons.push("detached_singleton".to_string());
        } else {
            reasons.push("detached_subtree".to_string());
        }
    }

    // --- 5. Same-Component Name Cap Guard ---
    // Inside the same connected component without relational corroboration, name similarity alone
    // is capped at 40 points and total score cannot reach the duplicate threshold.
    if !is_detached && !has_relational_corroboration {
        score = score.min(55);
    }

    // --- 6. Sex compatibility penalty ---
    if conflicting_sex {
        score -= 30;
    }

    if score <= 0 {
        (0, Vec::new())
    } else {
        (score.min(100), reasons)
    }
}
