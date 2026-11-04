#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ProfileKinshipDto {
    pub home_person_id: String,
    pub target_person_id: String,
    pub relation_key: String,
    pub label_en: String,
    pub label_ru: String,
    pub steps: usize,
    pub path_person_ids: Vec<String>,
}

/// Computes the shortest kinship relation between `home_id` and `target_id` across visible
/// individuals and families (`03-person-profile-page.md §2.1`, `16-dashboard-home.md §2`).
/// Uses cycle-safe BFS capped at 6 generations.
pub fn calculate_kinship_relation(
    home_id: &str,
    target_id: &str,
    persons: &[TreePersonNodeDto],
    families: &[FamilyDto],
) -> Option<ProfileKinshipDto> {
    let home_id = home_id.trim();
    let target_id = target_id.trim();
    if home_id.is_empty() || target_id.is_empty() {
        return None;
    }

    let person_by_id: HashMap<&str, &TreePersonNodeDto> =
        persons.iter().map(|p| (p.id.as_str(), p)).collect();
    let _home_person = *person_by_id.get(home_id)?;
    let target_person = *person_by_id.get(target_id)?;

    if home_id == target_id {
        return Some(ProfileKinshipDto {
            home_person_id: home_id.to_string(),
            target_person_id: target_id.to_string(),
            relation_key: "self".to_string(),
            label_en: "You (Home Person)".to_string(),
            label_ru: "Вы (исходная персона)".to_string(),
            steps: 0,
            path_person_ids: vec![home_id.to_string()],
        });
    }

    let target_sex = target_person.sex.trim().to_ascii_uppercase();
    let is_male = target_sex == "M";
    let is_female = target_sex == "F";

    // Build parent and spouse adjacency maps:
    // parents_of[person] -> Vec<parent_id>
    // spouses_of[person] -> Vec<spouse_id>
    let mut parents_of: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut spouses_of: HashMap<&str, Vec<&str>> = HashMap::new();

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
        }

        for ch in &fam.children {
            let cid = ch.child_id.as_str();
            if !person_by_id.contains_key(cid) {
                continue;
            }
            for parent in [p1, p2].into_iter().flatten() {
                parents_of.entry(cid).or_default().push(parent);
            }
        }
    }

    // Check direct spouse (steps = 1)
    if let Some(spouses) = spouses_of.get(home_id) {
        if spouses.contains(&target_id) {
            let (label_en, label_ru) = if is_male {
                ("Husband", "Муж")
            } else if is_female {
                ("Wife", "Жена")
            } else {
                ("Spouse", "Супруг(а)")
            };
            return Some(ProfileKinshipDto {
                home_person_id: home_id.to_string(),
                target_person_id: target_id.to_string(),
                relation_key: "spouse".to_string(),
                label_en: label_en.to_string(),
                label_ru: label_ru.to_string(),
                steps: 1,
                path_person_ids: vec![home_id.to_string(), target_id.to_string()],
            });
        }
    }

    // Compute upward ancestor paths from any person up to 6 generations:
    // returns HashMap<ancestor_id, (depth, path_from_start_to_ancestor)>
    let build_ancestors = |start_id: &str| -> HashMap<String, (usize, Vec<String>)> {
        let mut result: HashMap<String, (usize, Vec<String>)> = HashMap::new();
        let mut queue: VecDeque<(String, usize, Vec<String>)> = VecDeque::new();
        result.insert(start_id.to_string(), (0, vec![start_id.to_string()]));
        queue.push_back((start_id.to_string(), 0, vec![start_id.to_string()]));

        while let Some((curr, depth, path)) = queue.pop_front() {
            if depth >= 6 {
                continue;
            }
            if let Some(parents) = parents_of.get(curr.as_str()) {
                for &p_id in parents {
                    if !result.contains_key(p_id) {
                        let mut next_path = path.clone();
                        next_path.push(p_id.to_string());
                        result.insert(p_id.to_string(), (depth + 1, next_path.clone()));
                        queue.push_back((p_id.to_string(), depth + 1, next_path));
                    }
                }
            }
        }
        result
    };

    let home_ancestors = build_ancestors(home_id);
    let target_ancestors = build_ancestors(target_id);

    // Find lowest common ancestor (LCA) minimizing (up + down)
    let mut best_blood: Option<(usize, usize, Vec<String>)> = None;
    for (anc_id, (up_steps, path_home_to_anc)) in &home_ancestors {
        if let Some((down_steps, path_target_to_anc)) = target_ancestors.get(anc_id) {
            let total_steps = *up_steps + *down_steps;
            let is_better = match &best_blood {
                None => true,
                Some((best_u, best_d, _)) => total_steps < (*best_u + *best_d),
            };
            if is_better {
                let mut combined_path = path_home_to_anc.clone();
                // Append reversed path from ancestor down to target (excluding ancestor itself)
                for node in path_target_to_anc.iter().rev().skip(1) {
                    combined_path.push(node.clone());
                }
                best_blood = Some((*up_steps, *down_steps, combined_path));
            }
        }
    }

    if let Some((up, down, path)) = best_blood {
        let steps = up + down;
        let (relation_key, label_en, label_ru) = match (up, down) {
            (1, 0) => (
                "parent",
                if is_male {
                    "Father"
                } else if is_female {
                    "Mother"
                } else {
                    "Parent"
                },
                if is_male {
                    "Отец"
                } else if is_female {
                    "Мать"
                } else {
                    "Родитель"
                },
            ),
            (0, 1) => (
                "child",
                if is_male {
                    "Son"
                } else if is_female {
                    "Daughter"
                } else {
                    "Child"
                },
                if is_male {
                    "Сын"
                } else if is_female {
                    "Дочь"
                } else {
                    "Ребёнок"
                },
            ),
            (2, 0) => (
                "grandparent",
                if is_male {
                    "Grandfather"
                } else if is_female {
                    "Grandmother"
                } else {
                    "Grandparent"
                },
                if is_male {
                    "Дедушка"
                } else if is_female {
                    "Бабушка"
                } else {
                    "Дедушка / Бабушка"
                },
            ),
            (0, 2) => (
                "grandchild",
                if is_male {
                    "Grandson"
                } else if is_female {
                    "Granddaughter"
                } else {
                    "Grandchild"
                },
                if is_male {
                    "Внук"
                } else if is_female {
                    "Внучка"
                } else {
                    "Внук / Внучка"
                },
            ),
            (u, 0) if u >= 3 => (
                "great_grandparent",
                if is_male {
                    "Great-grandfather"
                } else if is_female {
                    "Great-grandmother"
                } else {
                    "Great-grandparent"
                },
                if is_male {
                    "Прадедушка"
                } else if is_female {
                    "Прабабушка"
                } else {
                    "Прародитель"
                },
            ),
            (0, d) if d >= 3 => (
                "great_grandchild",
                if is_male {
                    "Great-grandson"
                } else if is_female {
                    "Great-granddaughter"
                } else {
                    "Great-grandchild"
                },
                if is_male {
                    "Правнук"
                } else if is_female {
                    "Правнучка"
                } else {
                    "Правнук / Правнучка"
                },
            ),
            (1, 1) => (
                "sibling",
                if is_male {
                    "Brother"
                } else if is_female {
                    "Sister"
                } else {
                    "Sibling"
                },
                if is_male {
                    "Брат"
                } else if is_female {
                    "Сестра"
                } else {
                    "Брат / Сестра"
                },
            ),
            (2, 1) => (
                "pibling",
                if is_male {
                    "Uncle"
                } else if is_female {
                    "Aunt"
                } else {
                    "Uncle / Aunt"
                },
                if is_male {
                    "Дядя"
                } else if is_female {
                    "Тётя"
                } else {
                    "Дядя / Тётя"
                },
            ),
            (1, 2) => (
                "nibling",
                if is_male {
                    "Nephew"
                } else if is_female {
                    "Niece"
                } else {
                    "Nephew / Niece"
                },
                if is_male {
                    "Племянник"
                } else if is_female {
                    "Племянница"
                } else {
                    "Племянник / Племянница"
                },
            ),
            (2, 2) => (
                "first_cousin",
                "First cousin",
                if is_male {
                    "Двоюродный брат"
                } else if is_female {
                    "Двоюродная сестра"
                } else {
                    "Двоюродный брат / сестра"
                },
            ),
            _ => ("relative", "Blood relative", "Кровный родственник"),
        };

        return Some(ProfileKinshipDto {
            home_person_id: home_id.to_string(),
            target_person_id: target_id.to_string(),
            relation_key: relation_key.to_string(),
            label_en: label_en.to_string(),
            label_ru: label_ru.to_string(),
            steps,
            path_person_ids: path,
        });
    }

    // Check parent-in-law (parent of home person's spouse)
    if let Some(spouses) = spouses_of.get(home_id) {
        for &sp_id in spouses {
            if let Some(sp_parents) = parents_of.get(sp_id) {
                if sp_parents.contains(&target_id) {
                    let spouse_sex = person_by_id
                        .get(sp_id)
                        .map(|p| p.sex.trim().to_ascii_uppercase())
                        .unwrap_or_default();
                    let is_husband_side = spouse_sex == "M";
                    let relation_key = if is_husband_side {
                        "parent_in_law_husband"
                    } else {
                        "parent_in_law_wife"
                    };
                    let (label_en, label_ru) = if is_husband_side {
                        if is_male {
                            ("Father-in-law (husband's father)", "Свёкор")
                        } else if is_female {
                            ("Mother-in-law (husband's mother)", "Свекровь")
                        } else {
                            ("Parent-in-law", "Родитель мужа")
                        }
                    } else if is_male {
                        ("Father-in-law (wife's father)", "Тесть")
                    } else if is_female {
                        ("Mother-in-law (wife's mother)", "Тёща")
                    } else {
                        ("Parent-in-law", "Родитель жены")
                    };
                    return Some(ProfileKinshipDto {
                        home_person_id: home_id.to_string(),
                        target_person_id: target_id.to_string(),
                        relation_key: relation_key.to_string(),
                        label_en: label_en.to_string(),
                        label_ru: label_ru.to_string(),
                        steps: 2,
                        path_person_ids: vec![
                            home_id.to_string(),
                            sp_id.to_string(),
                            target_id.to_string(),
                        ],
                    });
                }
            }
        }
    }

    None
}
