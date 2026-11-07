#[allow(unused_imports)]
use super::*;

pub(crate) fn build_trace_from_path(
    path: &[String],
    trace_id: String,
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    parents_of: &HashMap<&str, Vec<&str>>,
    spouses_of: &HashMap<&str, Vec<&str>>,
) -> KinshipTraceDto {
    let mut steps = Vec::new();
    for i in 0..path.len() {
        let curr = &path[i];
        let (relation_to_next, relation_to_next_ru) = if i + 1 < path.len() {
            let next = &path[i + 1];
            let next_sex = person_by_id
                .get(next.as_str())
                .map(|p| p.sex.trim().to_ascii_uppercase())
                .unwrap_or_default();
            let next_is_male = next_sex == "M";
            let next_is_female = next_sex == "F";

            if parents_of
                .get(curr.as_str())
                .is_some_and(|ps| ps.contains(&next.as_str()))
            {
                if next_is_male {
                    (Some("Father".to_string()), Some("Отец".to_string()))
                } else if next_is_female {
                    (Some("Mother".to_string()), Some("Мать".to_string()))
                } else {
                    (Some("Parent".to_string()), Some("Родитель".to_string()))
                }
            } else if parents_of
                .get(next.as_str())
                .is_some_and(|ps| ps.contains(&curr.as_str()))
            {
                if next_is_male {
                    (Some("Son".to_string()), Some("Сын".to_string()))
                } else if next_is_female {
                    (Some("Daughter".to_string()), Some("Дочь".to_string()))
                } else {
                    (Some("Child".to_string()), Some("Ребёнок".to_string()))
                }
            } else if spouses_of
                .get(curr.as_str())
                .is_some_and(|ss| ss.contains(&next.as_str()))
            {
                if next_is_male {
                    (Some("Husband".to_string()), Some("Муж".to_string()))
                } else if next_is_female {
                    (Some("Wife".to_string()), Some("Жена".to_string()))
                } else {
                    (Some("Spouse".to_string()), Some("Супруг(а)".to_string()))
                }
            } else {
                (
                    Some("Relative".to_string()),
                    Some("Родственник".to_string()),
                )
            }
        } else {
            (None, None)
        };

        steps.push(KinshipTraceStepDto {
            person_id: curr.clone(),
            relation_to_next,
            relation_to_next_ru,
        });
    }

    let mut edge_types: Vec<&'static str> = Vec::new();
    for i in 0..path.len().saturating_sub(1) {
        let p_curr = path[i].as_str();
        let p_next = path[i + 1].as_str();
        if parents_of
            .get(p_curr)
            .is_some_and(|ps| ps.contains(&p_next))
        {
            edge_types.push("parent");
        } else if parents_of
            .get(p_next)
            .is_some_and(|ps| ps.contains(&p_curr))
        {
            edge_types.push("child");
        } else if spouses_of
            .get(p_curr)
            .is_some_and(|ss| ss.contains(&p_next))
        {
            edge_types.push("spouse");
        } else {
            edge_types.push("other");
        }
    }

    let mut is_blood = false;
    let mut u_steps = 0usize;
    let mut d_steps = 0usize;

    if !edge_types.contains(&"spouse") && !edge_types.contains(&"other") {
        let first_child_idx = edge_types.iter().position(|e| *e == "child");
        let all_parents_before = match first_child_idx {
            Some(idx) => edge_types[..idx].iter().all(|e| *e == "parent"),
            None => edge_types.iter().all(|e| *e == "parent"),
        };
        let all_children_after = match first_child_idx {
            Some(idx) => edge_types[idx..].iter().all(|e| *e == "child"),
            None => true,
        };
        if all_parents_before && all_children_after {
            is_blood = true;
            u_steps = match first_child_idx {
                Some(idx) => idx,
                None => edge_types.len(),
            };
            d_steps = edge_types.len() - u_steps;
        }
    }

    if is_blood {
        let common_ancestor_id = path[u_steps].clone();
        let (rel_key, label_en, label_ru) =
            classify_blood_relation(u_steps, d_steps, path, person_by_id, parents_of);
        KinshipTraceDto {
            id: trace_id,
            relationship_key: rel_key,
            label_en,
            label_ru,
            degree_of_kinship: u_steps + d_steps,
            is_blood_relation: true,
            common_ancestor_ids: vec![common_ancestor_id],
            path_person_ids: path.to_vec(),
            steps,
        }
    } else {
        let (rel_key, label_en, label_ru) =
            classify_affinal_relation(&edge_types, path, person_by_id);
        KinshipTraceDto {
            id: trace_id,
            relationship_key: rel_key,
            label_en,
            label_ru,
            degree_of_kinship: edge_types.len(),
            is_blood_relation: false,
            common_ancestor_ids: vec![],
            path_person_ids: path.to_vec(),
            steps,
        }
    }
}

/// Validates that a genealogical candidate path does not contain artificial detours,
/// spurious inversions (e.g. descending to a child and ascending back up to a spouse),
/// or fake step-family classifications when biological relationships exist.
pub(crate) fn is_valid_kinship_path(
    path: &[String],
    parents_of: &HashMap<&str, Vec<&str>>,
    spouses_of: &HashMap<&str, Vec<&str>>,
) -> bool {
    if path.len() <= 2 {
        return true;
    }

    let mut edge_types: Vec<&'static str> = Vec::with_capacity(path.len() - 1);
    for i in 0..path.len() - 1 {
        let curr = path[i].as_str();
        let next = path[i + 1].as_str();
        if parents_of.get(curr).is_some_and(|ps| ps.contains(&next)) {
            edge_types.push("parent");
        } else if parents_of.get(next).is_some_and(|ps| ps.contains(&curr)) {
            edge_types.push("child");
        } else if spouses_of.get(curr).is_some_and(|ss| ss.contains(&next)) {
            edge_types.push("spouse");
        } else {
            edge_types.push("other");
        }
    }

    // Rule 1: No Ascent after Descent (Never go child -> parent)
    // In genealogy, a path can ascend to ancestors or in-laws, and then descend to descendants.
    // It can never descend to a child and then ascend to the other parent.
    let mut seen_child = false;
    for &edge in &edge_types {
        if edge == "child" {
            seen_child = true;
        } else if edge == "parent" && seen_child {
            return false;
        }
    }

    // Rule 2: No fake step-child via spouse when source is already a biological parent
    for i in 0..edge_types.len().saturating_sub(1) {
        if edge_types[i] == "spouse" && edge_types[i + 1] == "child" {
            let p_start = path[i].as_str();
            let p_child = path[i + 2].as_str();
            if parents_of
                .get(p_child)
                .is_some_and(|ps| ps.contains(&p_start))
            {
                return false;
            }
        }
    }

    // Rule 3: No fake step-parent via spouse when destination is already a biological parent
    for i in 0..edge_types.len().saturating_sub(1) {
        if edge_types[i] == "parent" && edge_types[i + 1] == "spouse" {
            let p_child = path[i].as_str();
            let p_end = path[i + 2].as_str();
            if parents_of
                .get(p_child)
                .is_some_and(|ps| ps.contains(&p_end))
            {
                return false;
            }
        }
    }

    // Rule 4: No internal shortcuts/chords (skipping a direct connection to take an unnecessary detour)
    for i in 0..path.len() {
        for j in (i + 2)..path.len() {
            if i == 0 && j == path.len() - 1 {
                // An edge between endpoints is permitted if one path is blood and one is spouse/affinal
                continue;
            }
            let u = path[i].as_str();
            let v = path[j].as_str();
            let is_neighbor = parents_of.get(u).is_some_and(|ps| ps.contains(&v))
                || parents_of.get(v).is_some_and(|ps| ps.contains(&u))
                || spouses_of.get(u).is_some_and(|ss| ss.contains(&v));
            if is_neighbor {
                return false;
            }
        }
    }

    // Rule 5: If there is a direct biological parent-child or spouse edge between endpoints,
    // any longer path containing a spouse/affinal edge is an invalid detour.
    let (Some(from_id), Some(to_id)) = (path.first(), path.last()) else {
        return false;
    };
    let from_id = from_id.as_str();
    let to_id = to_id.as_str();
    let is_direct_parent_child = parents_of
        .get(from_id)
        .is_some_and(|ps| ps.contains(&to_id))
        || parents_of
            .get(to_id)
            .is_some_and(|ps| ps.contains(&from_id));
    if is_direct_parent_child && edge_types.contains(&"spouse") {
        return false;
    }

    let is_direct_spouse = spouses_of
        .get(from_id)
        .is_some_and(|ss| ss.contains(&to_id));
    if is_direct_spouse && path.len() > 2 && edge_types.contains(&"spouse") {
        return false;
    }

    true
}

/// Full Phase 2 Kinship Finder (`GET /api/v1/kinship?from={id1}&to={id2}&max_traces={n}`, `04-tree-visualizations.md §2.5, §7.5`).
/// Discovers up to `max_traces` distinct minimal genealogical paths across multiple marriages, remarriages,
/// and collateral branches, ranking biological blood relationships first, followed by affinal/step paths.
pub fn compute_kinship_path(
    from_id: &str,
    to_id: &str,
    persons: &[TreePersonNodeDto],
    families: &[FamilyDto],
    max_traces: usize,
) -> KinshipResponse {
    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        persons.iter().map(|p| (p.id.as_str(), p)).collect();

    if from_id == to_id {
        let node_vec = person_by_id
            .get(from_id)
            .map(|p| vec![(*p).clone()])
            .unwrap_or_default();
        let trace = KinshipTraceDto {
            id: "trace-1".to_string(),
            relationship_key: "self".to_string(),
            label_en: "Self".to_string(),
            label_ru: "Сам(а) (исходная персона)".to_string(),
            degree_of_kinship: 0,
            is_blood_relation: true,
            common_ancestor_ids: vec![from_id.to_string()],
            path_person_ids: vec![from_id.to_string()],
            steps: vec![KinshipTraceStepDto {
                person_id: from_id.to_string(),
                relation_to_next: None,
                relation_to_next_ru: None,
            }],
        };
        return KinshipResponse {
            connected: true,
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relationship_key: "self".to_string(),
            relation_key: "self".to_string(),
            label_en: "Self".to_string(),
            label_ru: "Сам(а) (исходная персона)".to_string(),
            degree_of_kinship: 0,
            steps_count: 0,
            up_steps: 0,
            down_steps: 0,
            is_blood_relation: true,
            common_ancestor_ids: vec![from_id.to_string()],
            path_person_ids: vec![from_id.to_string()],
            steps: vec![],
            path_nodes: node_vec,
            traces_count: 1,
            traces: vec![trace],
        };
    }

    let mut parents_of: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut children_of: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut spouses_of: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut all_neighbors: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();

    for fam in families {
        let p1 = fam
            .partner1_id
            .as_deref()
            .filter(|id| person_by_id.contains_key(*id));
        let p2 = fam
            .partner2_id
            .as_deref()
            .filter(|id| person_by_id.contains_key(*id));

        if let (Some(a), Some(b)) = (p1, p2) {
            spouses_of.entry(a).or_default().push(b);
            spouses_of.entry(b).or_default().push(a);
            all_neighbors.entry(a).or_default().push((b, "spouse"));
            all_neighbors.entry(b).or_default().push((a, "spouse"));
        }

        for ch in &fam.children {
            let cid = ch.child_id.as_str();
            if !person_by_id.contains_key(cid) {
                continue;
            }
            for parent in [p1, p2].into_iter().flatten() {
                parents_of.entry(cid).or_default().push(parent);
                children_of.entry(parent).or_default().push(cid);
                all_neighbors
                    .entry(cid)
                    .or_default()
                    .push((parent, "parent"));
                all_neighbors
                    .entry(parent)
                    .or_default()
                    .push((cid, "child"));
            }
        }
    }

    let mut candidate_paths: Vec<Vec<String>> = Vec::new();

    // 1. Discover all minimal biological paths via common ancestors (up to 10 generations)
    let build_ancestor_paths = |start_id: &str| -> HashMap<String, Vec<Vec<String>>> {
        let mut result: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        let mut queue: VecDeque<(String, usize, Vec<String>)> = VecDeque::new();
        result.insert(start_id.to_string(), vec![vec![start_id.to_string()]]);
        queue.push_back((start_id.to_string(), 0, vec![start_id.to_string()]));

        while let Some((curr, depth, path)) = queue.pop_front() {
            if depth >= 10 {
                continue;
            }
            if let Some(parents) = parents_of.get(curr.as_str()) {
                for &p_id in parents {
                    if !path.iter().any(|n| n == p_id) {
                        let mut next_path = path.clone();
                        next_path.push(p_id.to_string());
                        let list = result.entry(p_id.to_string()).or_default();
                        if !list.contains(&next_path) {
                            list.push(next_path.clone());
                        }
                        queue.push_back((p_id.to_string(), depth + 1, next_path));
                    }
                }
            }
        }
        result
    };

    let from_anc = build_ancestor_paths(from_id);
    let to_anc = build_ancestor_paths(to_id);

    for (anc_id, paths_from) in &from_anc {
        if let Some(paths_to) = to_anc.get(anc_id) {
            for pu in paths_from {
                for pd in paths_to {
                    let pu_sub = &pu[0..pu.len().saturating_sub(1)];
                    let pd_sub = &pd[0..pd.len().saturating_sub(1)];
                    let has_common_lower = pu_sub.iter().any(|node| pd_sub.contains(node));
                    if !has_common_lower {
                        let mut combined = pu.clone();
                        for node in pd.iter().rev().skip(1) {
                            combined.push(node.clone());
                        }
                        if !candidate_paths.contains(&combined) {
                            candidate_paths.push(combined);
                        }
                    }
                }
            }
        }
    }

    // 2. Discover shortest paths across all edges (including marital/affinal) using Yen's loopless K-shortest paths
    let find_shortest = |start: &str,
                         dest: &str,
                         excluded_edges: &HashSet<(&str, &str)>,
                         excluded_nodes: &HashSet<&str>|
     -> Option<Vec<String>> {
        let mut q: VecDeque<Vec<String>> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(start.to_string());
        q.push_back(vec![start.to_string()]);

        while let Some(path) = q.pop_front() {
            let curr = path.last()?.as_str();
            if curr == dest {
                return Some(path);
            }
            if path.len() >= 12 {
                continue;
            }
            if let Some(neighbors) = all_neighbors.get(curr) {
                for &(next, kind) in neighbors {
                    if excluded_nodes.contains(next) {
                        continue;
                    }
                    if excluded_edges.contains(&(curr, next))
                        || excluded_edges.contains(&(next, curr))
                    {
                        continue;
                    }

                    // Prune "parent" step after "child" step (cannot descend to a child and then ascend to a parent)
                    let is_parent_step = kind == "parent";
                    let has_child_step = path.windows(2).any(|w| {
                        parents_of
                            .get(w[1].as_str())
                            .is_some_and(|ps| ps.contains(&w[0].as_str()))
                    });
                    if has_child_step && is_parent_step {
                        continue;
                    }

                    if !visited.contains(next) {
                        visited.insert(next.to_string());
                        let mut next_path = path.clone();
                        next_path.push(next.to_string());
                        q.push_back(next_path);
                    }
                }
            }
        }
        None
    };

    let mut yen_result: Vec<Vec<String>> = Vec::new();
    let mut yen_candidates: Vec<Vec<String>> = Vec::new();

    if let Some(p0) = find_shortest(from_id, to_id, &HashSet::new(), &HashSet::new()) {
        if is_valid_kinship_path(&p0, &parents_of, &spouses_of) {
            yen_result.push(p0);
        }
    }

    let target_yen_count = max_traces + 3;
    for k in 1..=target_yen_count {
        if k > yen_result.len() {
            break;
        }
        let prev_path = &yen_result[k - 1];
        for i in 0..(prev_path.len().saturating_sub(1)) {
            let spur_node = &prev_path[i];
            let root_path = &prev_path[0..=i];

            let mut excluded_edges: HashSet<(&str, &str)> = HashSet::new();
            for p in &yen_result {
                if p.len() > i + 1 && &p[0..=i] == root_path {
                    excluded_edges.insert((p[i].as_str(), p[i + 1].as_str()));
                    excluded_edges.insert((p[i + 1].as_str(), p[i].as_str()));
                }
            }

            let mut excluded_nodes: HashSet<&str> = HashSet::new();
            for node in &root_path[0..i] {
                excluded_nodes.insert(node.as_str());
            }

            if let Some(spur_path) =
                find_shortest(spur_node, to_id, &excluded_edges, &excluded_nodes)
            {
                let mut total_path = root_path.to_vec();
                for node in &spur_path[1..] {
                    total_path.push(node.clone());
                }
                if is_valid_kinship_path(&total_path, &parents_of, &spouses_of)
                    && !yen_candidates.contains(&total_path)
                    && !yen_result.contains(&total_path)
                {
                    yen_candidates.push(total_path);
                }
            }
        }

        if yen_candidates.is_empty() {
            break;
        }
        yen_candidates.sort_by_key(|p| p.len());
        let best_next = yen_candidates.remove(0);
        yen_result.push(best_next);
    }

    for p in yen_result {
        if is_valid_kinship_path(&p, &parents_of, &spouses_of) && !candidate_paths.contains(&p) {
            candidate_paths.push(p);
        }
    }

    candidate_paths.retain(|p| is_valid_kinship_path(p, &parents_of, &spouses_of));

    if candidate_paths.is_empty() {
        return KinshipResponse {
            connected: false,
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            relationship_key: "unconnected".to_string(),
            relation_key: "unconnected".to_string(),
            label_en: "No known relationship".to_string(),
            label_ru: "Родственная связь не найдена".to_string(),
            degree_of_kinship: 0,
            steps_count: 0,
            up_steps: 0,
            down_steps: 0,
            is_blood_relation: false,
            common_ancestor_ids: vec![],
            path_person_ids: vec![],
            steps: vec![],
            path_nodes: vec![],
            traces_count: 0,
            traces: vec![],
        };
    }

    // Convert candidate paths to traces
    let mut traces: Vec<KinshipTraceDto> = candidate_paths
        .into_iter()
        .map(|path| {
            build_trace_from_path(
                &path,
                "temp".to_string(),
                &person_by_id,
                &parents_of,
                &spouses_of,
            )
        })
        .collect();

    // Rank traces: biological blood relationships first, then sorted by degree/steps count
    traces.sort_by(|a, b| {
        b.is_blood_relation
            .cmp(&a.is_blood_relation)
            .then_with(|| a.degree_of_kinship.cmp(&b.degree_of_kinship))
            .then_with(|| a.steps.len().cmp(&b.steps.len()))
            .then_with(|| a.relationship_key.cmp(&b.relationship_key))
    });

    traces.truncate(max_traces);

    for (idx, trace) in traces.iter_mut().enumerate() {
        trace.id = format!("trace-{}", idx + 1);
    }

    let primary = &traces[0];
    let path_nodes: Vec<TreePersonNodeDto> = primary
        .path_person_ids
        .iter()
        .filter_map(|id| person_by_id.get(id.as_str()).map(|p| (*p).clone()))
        .collect();

    let mut steps_vec = Vec::new();
    for idx in 0..primary.path_person_ids.len().saturating_sub(1) {
        let p_curr = &primary.path_person_ids[idx];
        let p_next = &primary.path_person_ids[idx + 1];
        let edge_type = if parents_of
            .get(p_curr.as_str())
            .is_some_and(|ps| ps.contains(&p_next.as_str()))
        {
            "parent"
        } else if parents_of
            .get(p_next.as_str())
            .is_some_and(|ps| ps.contains(&p_curr.as_str()))
        {
            "child"
        } else if spouses_of
            .get(p_curr.as_str())
            .is_some_and(|ss| ss.contains(&p_next.as_str()))
        {
            "spouse"
        } else {
            "relative"
        };
        steps_vec.push(KinshipEdgeStepDto {
            from_id: p_curr.clone(),
            to_id: p_next.clone(),
            edge_type: edge_type.to_string(),
        });
    }

    let (up_steps, down_steps) = if primary.is_blood_relation {
        let u = steps_vec
            .iter()
            .take_while(|s| s.edge_type == "parent")
            .count();
        let d = steps_vec.len().saturating_sub(u);
        (u, d)
    } else {
        (0, 0)
    };

    KinshipResponse {
        connected: true,
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        relationship_key: primary.relationship_key.clone(),
        relation_key: primary.relationship_key.clone(),
        label_en: primary.label_en.clone(),
        label_ru: primary.label_ru.clone(),
        degree_of_kinship: primary.degree_of_kinship,
        steps_count: steps_vec.len(),
        up_steps,
        down_steps,
        is_blood_relation: primary.is_blood_relation,
        common_ancestor_ids: primary.common_ancestor_ids.clone(),
        path_person_ids: primary.path_person_ids.clone(),
        steps: steps_vec,
        path_nodes,
        traces_count: traces.len(),
        traces,
    }
}
