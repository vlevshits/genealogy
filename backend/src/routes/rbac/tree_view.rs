#[allow(unused_imports)]
use super::*;

#[utoipa::path(
    get,
    path = "/api/v1/tree/view",
    params(TreeViewQuery),
    responses(
        (status = 200, description = "Projected tree canvas nodes, families, and edges for the requested view mode and generation depth", body = TreeViewResponse),
        (status = 400, description = "Invalid projection mode", body = ErrorResponse),
        (status = 401, description = "Unauthenticated when public tree sharing is disabled", body = ErrorResponse),
        (status = 404, description = "Specified focal person not found or hidden by privacy", body = ErrorResponse)
    ),
    tag = "Tree & Privacy"
)]
pub async fn get_tree_view(
    caller: PublicOrAuth,
    State(state): State<AppState>,
    Query(query): Query<TreeViewQuery>,
) -> Result<Json<TreeViewResponse>, AppError> {
    let mode = match query
        .mode
        .as_deref()
        .unwrap_or("hourglass")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "hourglass" | "" => "hourglass".to_string(),
        "pedigree" => "pedigree".to_string(),
        "descendant" | "descendants" => "descendant".to_string(),
        "fan" => "fan".to_string(),
        "all" => "all".to_string(),
        _ => {
            let mut fields = HashMap::new();
            fields.insert("mode".to_string(), "invalid_mode".to_string());
            return Err(AppError::ValidationFailed(
                "Mode must be hourglass, pedigree, descendant, fan, or all".to_string(),
                Some(fields),
            ));
        }
    };
    let generations = query.generations.unwrap_or(4).clamp(1, 10);
    let is_hourglass = mode == "hourglass";
    let collateral = query.collateral.unwrap_or(is_hourglass);

    let settings = state.resolve_instance_settings().await;
    let access = caller.caller_access();
    let store = state.privacy_store.read().await;

    let visible_person_ids: HashSet<String> = store
        .persons
        .iter()
        .filter(|p| !p.is_private || access.can_view_private())
        .map(|p| p.id.clone())
        .collect();

    let visible_families: Vec<FamilyDto> = store
        .families
        .iter()
        .filter_map(|f| {
            filter_family_for_visible_persons(f, access.can_view_private(), &visible_person_ids)
        })
        .collect();

    let (resolved_home_person_id, _, _, _) = resolve_home_person_for_caller(
        &state,
        &caller,
        &store.persons,
        &settings.default_locale,
        settings.default_home_person_id.as_deref(),
    )
    .await;

    let focal_id = if let Some(raw_focal) = query
        .focal_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let person = store
            .persons
            .iter()
            .find(|p| p.id == raw_focal || p.gid.eq_ignore_ascii_case(raw_focal))
            .ok_or(AppError::PersonNotFound)?;
        if !visible_person_ids.contains(&person.id) {
            return Err(AppError::PersonNotFound);
        }
        Some(person.id.clone())
    } else {
        resolved_home_person_id.clone().or_else(|| {
            store
                .persons
                .iter()
                .find(|p| visible_person_ids.contains(&p.id))
                .map(|p| p.id.clone())
        })
    };

    let mut included_person_ids: HashSet<String> = HashSet::new();
    let mut included_family_ids: HashSet<String> = HashSet::new();
    let mut included_child_links: HashSet<(String, String)> = HashSet::new();
    let mut direct_person_ids: HashSet<String> = HashSet::new();
    let mut direct_family_ids: HashSet<String> = HashSet::new();
    let mut person_generation: HashMap<String, i32> = HashMap::new();
    let mut ordered_person_ids: Vec<String> = Vec::new();

    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        store.persons.iter().map(|p| (p.id.as_str(), p)).collect();

    if mode == "all" {
        // 1. Include ALL visible persons and ALL visible families
        included_person_ids = visible_person_ids.clone();
        included_family_ids = visible_families.iter().map(|f| f.id.clone()).collect();
        for fam in &visible_families {
            for ch in &fam.children {
                if visible_person_ids.contains(&ch.child_id) {
                    included_child_links.insert((fam.id.clone(), ch.child_id.clone()));
                }
            }
        }

        // 2. Direct Lineage relative to focal_id (if present)
        if let Some(ref root_id) = focal_id {
            direct_person_ids.insert(root_id.clone());

            // Direct ancestors upward
            let mut anc_queue: VecDeque<String> = VecDeque::new();
            let mut visited_anc: HashSet<String> = HashSet::new();
            visited_anc.insert(root_id.clone());
            anc_queue.push_back(root_id.clone());

            while let Some(curr_id) = anc_queue.pop_front() {
                for fam in &visible_families {
                    if fam.children.iter().any(|c| c.child_id == curr_id) {
                        direct_family_ids.insert(fam.id.clone());
                        for parent_id in [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
                            .into_iter()
                            .flatten()
                        {
                            if visible_person_ids.contains(parent_id) {
                                direct_person_ids.insert(parent_id.clone());
                                if visited_anc.insert(parent_id.clone()) {
                                    anc_queue.push_back(parent_id.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Direct descendants downward
            let mut desc_queue: VecDeque<String> = VecDeque::new();
            let mut visited_desc: HashSet<String> = HashSet::new();
            visited_desc.insert(root_id.clone());
            desc_queue.push_back(root_id.clone());

            while let Some(curr_id) = desc_queue.pop_front() {
                for fam in &visible_families {
                    let is_partner = fam.partner1_id.as_deref() == Some(curr_id.as_str())
                        || fam.partner2_id.as_deref() == Some(curr_id.as_str());
                    if is_partner {
                        direct_family_ids.insert(fam.id.clone());
                        for ch in &fam.children {
                            if visible_person_ids.contains(&ch.child_id) {
                                direct_person_ids.insert(ch.child_id.clone());
                                if visited_desc.insert(ch.child_id.clone()) {
                                    desc_queue.push_back(ch.child_id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Adjacency graph for BFS generational leveling
        let mut adj: HashMap<String, Vec<(String, i32)>> = HashMap::new();
        let mut has_parents: HashSet<String> = HashSet::new();

        for fam in &visible_families {
            let p1 = fam
                .partner1_id
                .as_ref()
                .filter(|id| visible_person_ids.contains(*id));
            let p2 = fam
                .partner2_id
                .as_ref()
                .filter(|id| visible_person_ids.contains(*id));

            if let (Some(a), Some(b)) = (p1, p2) {
                adj.entry(a.clone()).or_default().push((b.clone(), 0));
                adj.entry(b.clone()).or_default().push((a.clone(), 0));
            }

            for p in [p1, p2].into_iter().flatten() {
                for ch in &fam.children {
                    if visible_person_ids.contains(&ch.child_id) {
                        has_parents.insert(ch.child_id.clone());
                        adj.entry(p.clone())
                            .or_default()
                            .push((ch.child_id.clone(), 1));
                        adj.entry(ch.child_id.clone())
                            .or_default()
                            .push((p.clone(), -1));
                    }
                }
            }

            // Sibling edges
            let vis_children: Vec<&String> = fam
                .children
                .iter()
                .map(|c| &c.child_id)
                .filter(|id| visible_person_ids.contains(*id))
                .collect();
            for i in 0..vis_children.len() {
                for j in (i + 1)..vis_children.len() {
                    let c1 = vis_children[i];
                    let c2 = vis_children[j];
                    adj.entry(c1.clone()).or_default().push((c2.clone(), 0));
                    adj.entry(c2.clone()).or_default().push((c1.clone(), 0));
                }
            }
        }

        // 4. Connected Component Partitioning via DSU
        let mut dsu = TreeDsu::new(visible_person_ids.iter().cloned());
        for fam in &visible_families {
            let mut members: Vec<&String> = Vec::new();
            if let Some(ref p1) = fam.partner1_id {
                if visible_person_ids.contains(p1) {
                    members.push(p1);
                }
            }
            if let Some(ref p2) = fam.partner2_id {
                if visible_person_ids.contains(p2) {
                    members.push(p2);
                }
            }
            for ch in &fam.children {
                if visible_person_ids.contains(&ch.child_id) {
                    members.push(&ch.child_id);
                }
            }
            if members.len() >= 2 {
                let first = members[0];
                for m in &members[1..] {
                    dsu.union(first, m);
                }
            }
        }

        let mut comp_map: HashMap<String, Vec<String>> = HashMap::new();
        for pid in &visible_person_ids {
            let root = dsu.find(pid);
            comp_map.entry(root).or_default().push(pid.clone());
        }

        // 5. Identify C_primary
        let primary_root = focal_id.as_ref().map(|fid| dsu.find(fid));

        let mut primary_component: Vec<String> = Vec::new();
        let mut secondary_components: Vec<Vec<String>> = Vec::new();
        let mut singletons: Vec<String> = Vec::new();

        for (c_root, members) in comp_map {
            if Some(&c_root) == primary_root.as_ref() {
                primary_component = members;
            } else if members.len() == 1 {
                singletons.push(members[0].clone());
            } else {
                secondary_components.push(members);
            }
        }

        // Sort secondary components by size descending, then earliest birth year ascending
        secondary_components.sort_by(|a, b| {
            let size_cmp = b.len().cmp(&a.len());
            if size_cmp != std::cmp::Ordering::Equal {
                return size_cmp;
            }
            let min_by_a = a
                .iter()
                .filter_map(|id| {
                    person_by_id
                        .get(id.as_str())
                        .and_then(|p| extract_person_birth_year(p))
                })
                .min()
                .unwrap_or(i32::MAX);
            let min_by_b = b
                .iter()
                .filter_map(|id| {
                    person_by_id
                        .get(id.as_str())
                        .and_then(|p| extract_person_birth_year(p))
                })
                .min()
                .unwrap_or(i32::MAX);
            min_by_a.cmp(&min_by_b)
        });

        // 6. Generational Leveling
        // C_primary: focal_id anchored at generation = 0
        if let Some(ref fid) = focal_id {
            person_generation.insert(fid.clone(), 0);
            let mut queue: VecDeque<String> = VecDeque::new();
            queue.push_back(fid.clone());

            while let Some(curr) = queue.pop_front() {
                let curr_gen = person_generation.get(&curr).copied().unwrap_or(0);
                if let Some(neighbors) = adj.get(&curr) {
                    for (nbr, delta) in neighbors {
                        if !person_generation.contains_key(nbr) {
                            person_generation.insert(nbr.clone(), curr_gen + delta);
                            queue.push_back(nbr.clone());
                        }
                    }
                }
            }

            for id in &primary_component {
                person_generation.entry(id.clone()).or_insert(0);
            }
        }

        // Secondary Components C_j
        let focal_birth_year = focal_id
            .as_deref()
            .and_then(|fid| person_by_id.get(fid))
            .and_then(|p| extract_person_birth_year(p));

        for comp in &secondary_components {
            let comp_persons: Vec<&TreePersonNodeDto> = comp
                .iter()
                .filter_map(|id| person_by_id.get(id.as_str()).copied())
                .collect();
            let median_by = compute_median_birth_year(&comp_persons);

            let base_gen = match (median_by, focal_birth_year) {
                (Some(mby), Some(fby)) => ((mby - f64::from(fby)) / 30.0).round() as i32,
                _ => 0,
            };

            let comp_set: HashSet<&str> = comp.iter().map(String::as_str).collect();

            // Find topological roots in this component (no visible parents in comp)
            let roots: Vec<String> = comp
                .iter()
                .filter(|id| !has_parents.contains(*id))
                .cloned()
                .collect();

            let top_roots: Vec<String> = roots
                .iter()
                .filter(|u| {
                    let married_to_child = visible_families.iter().any(|fam| {
                        let is_p1 = fam.partner1_id.as_deref() == Some(u.as_str());
                        let is_p2 = fam.partner2_id.as_deref() == Some(u.as_str());
                        if is_p1 {
                            fam.partner2_id
                                .as_ref()
                                .is_some_and(|p2| has_parents.contains(p2))
                        } else if is_p2 {
                            fam.partner1_id
                                .as_ref()
                                .is_some_and(|p1| has_parents.contains(p1))
                        } else {
                            false
                        }
                    });
                    !married_to_child
                })
                .cloned()
                .collect();

            let seed_roots = if !top_roots.is_empty() {
                top_roots
            } else if !roots.is_empty() {
                roots
            } else {
                comp.clone()
            };

            let mut queue: VecDeque<String> = VecDeque::new();
            for r in seed_roots {
                if !person_generation.contains_key(&r) {
                    person_generation.insert(r.clone(), base_gen);
                    queue.push_back(r);
                }
            }

            while let Some(curr) = queue.pop_front() {
                let curr_gen = person_generation.get(&curr).copied().unwrap_or(0);
                if let Some(neighbors) = adj.get(&curr) {
                    for (nbr, delta) in neighbors {
                        if comp_set.contains(nbr.as_str()) && !person_generation.contains_key(nbr) {
                            person_generation.insert(nbr.clone(), curr_gen + delta);
                            queue.push_back(nbr.clone());
                        }
                    }
                }
            }

            // Ensure any unvisited member in comp has a generation
            for id in comp {
                person_generation.entry(id.clone()).or_insert(base_gen);
            }
        }

        // Singletons: generation = 0
        for s in &singletons {
            person_generation.insert(s.clone(), 0);
        }

        // 7. Order nodes: Primary component, then Secondary components in sorted order, then Singletons
        let primary_set: HashSet<&str> = primary_component.iter().map(String::as_str).collect();
        for p in &store.persons {
            if primary_set.contains(p.id.as_str()) {
                ordered_person_ids.push(p.id.clone());
            }
        }
        for comp in &secondary_components {
            let comp_set: HashSet<&str> = comp.iter().map(String::as_str).collect();
            for p in &store.persons {
                if comp_set.contains(p.id.as_str()) {
                    ordered_person_ids.push(p.id.clone());
                }
            }
        }
        let singleton_set: HashSet<&str> = singletons.iter().map(String::as_str).collect();
        for p in &store.persons {
            if singleton_set.contains(p.id.as_str()) {
                ordered_person_ids.push(p.id.clone());
            }
        }
    } else if let Some(ref root_id) = focal_id {
        included_person_ids.insert(root_id.clone());
        direct_person_ids.insert(root_id.clone());
        person_generation.insert(root_id.clone(), 0);

        let mut anc_queue: VecDeque<(String, u32)> = VecDeque::new();
        let mut visited_anc: HashSet<String> = HashSet::new();

        let mut desc_queue: VecDeque<(String, u32)> = VecDeque::new();
        let mut visited_desc: HashSet<String> = HashSet::new();

        let mut direct_ancestor_families: Vec<(String, i32)> = Vec::new();

        match mode.as_str() {
            "pedigree" | "fan" => {
                visited_anc.insert(root_id.clone());
                anc_queue.push_back((root_id.clone(), 0));
            }
            "descendant" => {
                visited_desc.insert(root_id.clone());
                desc_queue.push_back((root_id.clone(), 0));
            }
            _ => {
                // "hourglass": focal couple, upward ancestor traversal, downward descendant traversal
                for fam in &visible_families {
                    let is_p1 = fam.partner1_id.as_deref() == Some(root_id.as_str());
                    let is_p2 = fam.partner2_id.as_deref() == Some(root_id.as_str());
                    if is_p1 || is_p2 {
                        included_family_ids.insert(fam.id.clone());
                        direct_family_ids.insert(fam.id.clone());
                        if let Some(ref p1) = fam.partner1_id {
                            included_person_ids.insert(p1.clone());
                            person_generation.entry(p1.clone()).or_insert(0);
                        }
                        if let Some(ref p2) = fam.partner2_id {
                            included_person_ids.insert(p2.clone());
                            person_generation.entry(p2.clone()).or_insert(0);
                        }
                    }
                }
                visited_anc.insert(root_id.clone());
                anc_queue.push_back((root_id.clone(), 0));

                visited_desc.insert(root_id.clone());
                desc_queue.push_back((root_id.clone(), 0));
            }
        }

        // Upward ancestor traversal
        while let Some((curr_id, depth)) = anc_queue.pop_front() {
            if depth >= generations {
                continue;
            }
            for fam in &visible_families {
                if fam.children.iter().any(|c| c.child_id == curr_id) {
                    let parent_gen = -((depth as i32) + 1);
                    included_family_ids.insert(fam.id.clone());
                    direct_family_ids.insert(fam.id.clone());
                    included_child_links.insert((fam.id.clone(), curr_id.clone()));

                    if !direct_ancestor_families
                        .iter()
                        .any(|(fid, _)| fid == &fam.id)
                    {
                        direct_ancestor_families.push((fam.id.clone(), parent_gen));
                    }

                    for parent_id in [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
                        .into_iter()
                        .flatten()
                    {
                        included_person_ids.insert(parent_id.clone());
                        direct_person_ids.insert(parent_id.clone());
                        person_generation
                            .entry(parent_id.clone())
                            .or_insert(parent_gen);
                        if visited_anc.insert(parent_id.clone()) {
                            anc_queue.push_back((parent_id.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        // When collateral == true in hourglass mode:
        if is_hourglass && collateral {
            let mut collateral_queue: VecDeque<(String, i32)> = VecDeque::new();
            let mut visited_collateral: HashSet<String> = HashSet::new();

            for (fam_id, parent_gen) in &direct_ancestor_families {
                if let Some(fam) = visible_families.iter().find(|f| &f.id == fam_id) {
                    for ch in &fam.children {
                        if !direct_person_ids.contains(&ch.child_id) {
                            let child_gen = parent_gen + 1;
                            included_person_ids.insert(ch.child_id.clone());
                            included_child_links.insert((fam.id.clone(), ch.child_id.clone()));
                            person_generation
                                .entry(ch.child_id.clone())
                                .or_insert(child_gen);
                            if visited_collateral.insert(ch.child_id.clone()) {
                                collateral_queue.push_back((ch.child_id.clone(), child_gen));
                            }
                        }
                    }
                }
            }

            // Wide Hourglass: include secondary spouse marriages for ancestors in direct_person_ids with tier < 0
            let ancestors_with_gen: Vec<(String, i32)> = direct_person_ids
                .iter()
                .filter_map(|pid| {
                    person_generation
                        .get(pid)
                        .copied()
                        .filter(|&gen| gen < 0)
                        .map(|gen| (pid.clone(), gen))
                })
                .collect();

            for (anc_id, parent_gen) in ancestors_with_gen {
                for fam in &visible_families {
                    let is_p1 = fam.partner1_id.as_deref() == Some(anc_id.as_str());
                    let is_p2 = fam.partner2_id.as_deref() == Some(anc_id.as_str());
                    if !(is_p1 || is_p2) {
                        continue;
                    }
                    included_family_ids.insert(fam.id.clone());

                    if let Some(ref p1) = fam.partner1_id {
                        included_person_ids.insert(p1.clone());
                        person_generation.entry(p1.clone()).or_insert(parent_gen);
                    }
                    if let Some(ref p2) = fam.partner2_id {
                        included_person_ids.insert(p2.clone());
                        person_generation.entry(p2.clone()).or_insert(parent_gen);
                    }

                    for ch in &fam.children {
                        if !direct_person_ids.contains(&ch.child_id) {
                            let child_gen = parent_gen + 1;
                            included_person_ids.insert(ch.child_id.clone());
                            included_child_links.insert((fam.id.clone(), ch.child_id.clone()));
                            person_generation
                                .entry(ch.child_id.clone())
                                .or_insert(child_gen);
                            if visited_collateral.insert(ch.child_id.clone()) {
                                collateral_queue.push_back((ch.child_id.clone(), child_gen));
                            }
                        }
                    }
                }
            }

            while let Some((collat_id, curr_gen)) = collateral_queue.pop_front() {
                for fam in &visible_families {
                    let is_p1 = fam.partner1_id.as_deref() == Some(collat_id.as_str());
                    let is_p2 = fam.partner2_id.as_deref() == Some(collat_id.as_str());
                    if !(is_p1 || is_p2) {
                        continue;
                    }

                    included_family_ids.insert(fam.id.clone());

                    if let Some(ref p1) = fam.partner1_id {
                        included_person_ids.insert(p1.clone());
                        person_generation.entry(p1.clone()).or_insert(curr_gen);
                    }
                    if let Some(ref p2) = fam.partner2_id {
                        included_person_ids.insert(p2.clone());
                        person_generation.entry(p2.clone()).or_insert(curr_gen);
                    }

                    if curr_gen + 1 < generations as i32 {
                        for ch in &fam.children {
                            if !direct_person_ids.contains(&ch.child_id) {
                                let child_gen = curr_gen + 1;
                                included_person_ids.insert(ch.child_id.clone());
                                included_child_links.insert((fam.id.clone(), ch.child_id.clone()));
                                person_generation
                                    .entry(ch.child_id.clone())
                                    .or_insert(child_gen);
                                if visited_collateral.insert(ch.child_id.clone()) {
                                    collateral_queue.push_back((ch.child_id.clone(), child_gen));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Downward direct descendant traversal
        while let Some((curr_id, depth)) = desc_queue.pop_front() {
            for fam in &visible_families {
                let is_partner = fam.partner1_id.as_deref() == Some(curr_id.as_str())
                    || fam.partner2_id.as_deref() == Some(curr_id.as_str());
                if !is_partner {
                    continue;
                }
                if depth < generations || curr_id == *root_id {
                    included_family_ids.insert(fam.id.clone());
                    direct_family_ids.insert(fam.id.clone());
                    for partner_id in [fam.partner1_id.as_ref(), fam.partner2_id.as_ref()]
                        .into_iter()
                        .flatten()
                    {
                        included_person_ids.insert(partner_id.clone());
                        person_generation
                            .entry(partner_id.clone())
                            .or_insert(depth as i32);
                    }
                }
                if depth < generations {
                    for ch in &fam.children {
                        let child_gen = (depth + 1) as i32;
                        included_person_ids.insert(ch.child_id.clone());
                        direct_person_ids.insert(ch.child_id.clone());
                        included_child_links.insert((fam.id.clone(), ch.child_id.clone()));
                        person_generation
                            .entry(ch.child_id.clone())
                            .or_insert(child_gen);
                        if visited_desc.insert(ch.child_id.clone()) {
                            desc_queue.push_back((ch.child_id.clone(), depth + 1));
                        }
                    }
                }
            }
        }
    }

    let nodes: Vec<TreePersonNodeDto> = if mode == "all" {
        ordered_person_ids
            .iter()
            .filter_map(|id| person_by_id.get(id.as_str()).copied())
            .filter_map(|p| {
                let mut p_clone = p.clone();
                p_clone.generation = person_generation.get(&p.id).copied();
                p_clone.is_direct_lineage = Some(direct_person_ids.contains(&p.id));
                redact_person_for_tree_view(&p_clone, &access)
            })
            .collect()
    } else {
        store
            .persons
            .iter()
            .filter(|p| included_person_ids.contains(&p.id))
            .filter_map(|p| {
                let mut p_clone = p.clone();
                p_clone.generation = person_generation.get(&p.id).copied();
                p_clone.is_direct_lineage = Some(direct_person_ids.contains(&p.id));
                redact_person_for_tree_view(&p_clone, &access)
            })
            .collect()
    };

    let mut families: Vec<FamilyDto> = Vec::new();
    let mut edges: Vec<TreeEdgeDto> = Vec::new();

    for fam in &visible_families {
        if !included_family_ids.contains(&fam.id) {
            continue;
        }
        let partner1_id = fam
            .partner1_id
            .as_ref()
            .filter(|id| included_person_ids.contains(*id))
            .cloned();
        let partner2_id = fam
            .partner2_id
            .as_ref()
            .filter(|id| included_person_ids.contains(*id))
            .cloned();
        let children: Vec<FamilyChildDto> = fam
            .children
            .iter()
            .filter(|c| {
                included_child_links.contains(&(fam.id.clone(), c.child_id.clone()))
                    && included_person_ids.contains(&c.child_id)
            })
            .cloned()
            .collect();

        if let (Some(ref p1), Some(ref p2)) = (&partner1_id, &partner2_id) {
            edges.push(TreeEdgeDto {
                id: format!("edge-spouse-{}", fam.id),
                source: p1.clone(),
                target: p2.clone(),
                kind: "spouse".to_string(),
                family_id: fam.id.clone(),
                is_direct_lineage: Some(direct_family_ids.contains(&fam.id)),
            });
        }

        for ch in &children {
            let is_direct_child =
                direct_family_ids.contains(&fam.id) && direct_person_ids.contains(&ch.child_id);
            edges.push(TreeEdgeDto {
                id: format!("edge-child-{}-{}", fam.id, ch.child_id),
                source: fam.id.clone(),
                target: ch.child_id.clone(),
                kind: "parent_child".to_string(),
                family_id: fam.id.clone(),
                is_direct_lineage: Some(is_direct_child),
            });
        }

        let (marriage_date, marriage_place) =
            if fam.marriage_date.is_some() || fam.marriage_place.is_some() {
                (fam.marriage_date.clone(), fam.marriage_place.clone())
            } else {
                find_family_marriage_facts(&store.events, &fam.id)
            };

        families.push(FamilyDto {
            id: fam.id.clone(),
            gid: fam.gid.clone(),
            partner1_id,
            partner2_id,
            relationship_type: fam.relationship_type.clone(),
            is_private: fam.is_private,
            children,
            external_ids: fam.external_ids.clone(),
            version: fam.version,
            marriage_date,
            marriage_place,
        });
    }

    let total_visible = nodes.len();

    Ok(Json(TreeViewResponse {
        access_tier: access.as_tier_str().to_string(),
        tree_title: settings.tree_title,
        focal_id,
        mode,
        generations,
        collateral: Some(collateral),
        nodes,
        families,
        edges,
        resolved_home_person_id,
        total_visible,
    }))
}
