#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct KinshipQuery {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub person_a: Option<String>,
    #[serde(default)]
    pub person_b: Option<String>,
    #[serde(default)]
    pub id1: Option<String>,
    #[serde(default)]
    pub id2: Option<String>,
    #[serde(default)]
    pub max_traces: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct KinshipEdgeStepDto {
    pub from_id: String,
    pub to_id: String,
    pub edge_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct KinshipTraceStepDto {
    pub person_id: String,
    pub relation_to_next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation_to_next_ru: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct KinshipTraceDto {
    pub id: String,
    pub relationship_key: String,
    pub label_en: String,
    pub label_ru: String,
    pub degree_of_kinship: usize,
    pub is_blood_relation: bool,
    pub common_ancestor_ids: Vec<String>,
    pub path_person_ids: Vec<String>,
    pub steps: Vec<KinshipTraceStepDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct KinshipResponse {
    pub connected: bool,
    pub from_id: String,
    pub to_id: String,
    pub relationship_key: String,
    pub relation_key: String,
    pub label_en: String,
    pub label_ru: String,
    pub degree_of_kinship: usize,
    pub steps_count: usize,
    pub up_steps: usize,
    pub down_steps: usize,
    pub is_blood_relation: bool,
    pub common_ancestor_ids: Vec<String>,
    pub path_person_ids: Vec<String>,
    pub steps: Vec<KinshipEdgeStepDto>,
    pub path_nodes: Vec<TreePersonNodeDto>,
    #[serde(default)]
    pub traces_count: usize,
    #[serde(default)]
    pub traces: Vec<KinshipTraceDto>,
}

pub(crate) fn classify_blood_relation(
    u: usize,
    d: usize,
    path: &[String],
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
    parents_of: &HashMap<&str, Vec<&str>>,
) -> (String, String, String) {
    let to_id = path.last().map(|s| s.as_str()).unwrap_or_default();
    let target_sex = person_by_id
        .get(to_id)
        .map(|p| p.sex.trim().to_ascii_uppercase())
        .unwrap_or_default();
    let is_male = target_sex == "M";
    let is_female = target_sex == "F";

    let first_up_sex = if u >= 1 && path.len() >= 2 {
        person_by_id
            .get(path[1].as_str())
            .map(|p| p.sex.trim().to_ascii_uppercase())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let is_paternal = first_up_sex == "M";
    let is_maternal = first_up_sex == "F";

    match (u, d) {
        (1, 0) => (
            if is_male {
                "father".to_string()
            } else if is_female {
                "mother".to_string()
            } else {
                "parent".to_string()
            },
            if is_male {
                "Father".to_string()
            } else if is_female {
                "Mother".to_string()
            } else {
                "Parent".to_string()
            },
            if is_male {
                "Отец".to_string()
            } else if is_female {
                "Мать".to_string()
            } else {
                "Родитель".to_string()
            },
        ),
        (0, 1) => (
            if is_male {
                "son".to_string()
            } else if is_female {
                "daughter".to_string()
            } else {
                "child".to_string()
            },
            if is_male {
                "Son".to_string()
            } else if is_female {
                "Daughter".to_string()
            } else {
                "Child".to_string()
            },
            if is_male {
                "Сын".to_string()
            } else if is_female {
                "Дочь".to_string()
            } else {
                "Ребёнок".to_string()
            },
        ),
        (1, 1) => {
            let p0_parents = parents_of
                .get(path[0].as_str())
                .cloned()
                .unwrap_or_default();
            let p2_parents = parents_of
                .get(path[2].as_str())
                .cloned()
                .unwrap_or_default();
            let common_parents_count = p0_parents.iter().filter(|p| p2_parents.contains(p)).count();

            if common_parents_count >= 2 {
                (
                    if is_male {
                        "brother".to_string()
                    } else if is_female {
                        "sister".to_string()
                    } else {
                        "sibling".to_string()
                    },
                    if is_male {
                        "Brother".to_string()
                    } else if is_female {
                        "Sister".to_string()
                    } else {
                        "Sibling".to_string()
                    },
                    if is_male {
                        "Брат".to_string()
                    } else if is_female {
                        "Сестра".to_string()
                    } else {
                        "Брат / Сестра".to_string()
                    },
                )
            } else {
                let parent_sex = person_by_id
                    .get(path[1].as_str())
                    .map(|p| p.sex.trim().to_ascii_uppercase())
                    .unwrap_or_default();
                if parent_sex == "M" {
                    (
                        if is_male {
                            "half_brother_paternal".to_string()
                        } else if is_female {
                            "half_sister_paternal".to_string()
                        } else {
                            "half_sibling_paternal".to_string()
                        },
                        if is_male {
                            "Half-brother (paternal)".to_string()
                        } else if is_female {
                            "Half-sister (paternal)".to_string()
                        } else {
                            "Half-sibling (paternal)".to_string()
                        },
                        if is_male {
                            "Единокровный брат (по отцу)".to_string()
                        } else if is_female {
                            "Единокровная сестра (по отцу)".to_string()
                        } else {
                            "Единокровный брат / сестра (по отцу)".to_string()
                        },
                    )
                } else if parent_sex == "F" {
                    (
                        if is_male {
                            "half_brother_maternal".to_string()
                        } else if is_female {
                            "half_sister_maternal".to_string()
                        } else {
                            "half_sibling_maternal".to_string()
                        },
                        if is_male {
                            "Half-brother (maternal)".to_string()
                        } else if is_female {
                            "Half-sister (maternal)".to_string()
                        } else {
                            "Half-sibling (maternal)".to_string()
                        },
                        if is_male {
                            "Единоутробный брат (по матери)".to_string()
                        } else if is_female {
                            "Единоутробная сестра (по матери)".to_string()
                        } else {
                            "Единоутробный брат / сестра (по матери)".to_string()
                        },
                    )
                } else {
                    (
                        if is_male {
                            "half_brother".to_string()
                        } else if is_female {
                            "half_sister".to_string()
                        } else {
                            "half_sibling".to_string()
                        },
                        if is_male {
                            "Half-brother".to_string()
                        } else if is_female {
                            "Half-sister".to_string()
                        } else {
                            "Half-sibling".to_string()
                        },
                        if is_male {
                            "Неполнородный брат".to_string()
                        } else if is_female {
                            "Неполнородная сестра".to_string()
                        } else {
                            "Неполнородный брат / сестра".to_string()
                        },
                    )
                }
            }
        }
        (2, 0) => {
            let side_prefix = if is_paternal {
                "paternal_"
            } else if is_maternal {
                "maternal_"
            } else {
                ""
            };
            let base = if is_male {
                "grandfather"
            } else if is_female {
                "grandmother"
            } else {
                "grandparent"
            };
            (
                format!("{side_prefix}{base}"),
                if is_male {
                    "Grandfather".to_string()
                } else if is_female {
                    "Grandmother".to_string()
                } else {
                    "Grandparent".to_string()
                },
                if is_male {
                    "Дедушка".to_string()
                } else if is_female {
                    "Бабушка".to_string()
                } else {
                    "Дедушка / Бабушка".to_string()
                },
            )
        }
        (0, 2) => (
            if is_male {
                "grandson".to_string()
            } else if is_female {
                "granddaughter".to_string()
            } else {
                "grandchild".to_string()
            },
            if is_male {
                "Grandson".to_string()
            } else if is_female {
                "Granddaughter".to_string()
            } else {
                "Grandchild".to_string()
            },
            if is_male {
                "Внук".to_string()
            } else if is_female {
                "Внучка".to_string()
            } else {
                "Внук / Внучка".to_string()
            },
        ),
        (2, 1) => {
            let side_prefix = if is_paternal {
                "paternal_"
            } else if is_maternal {
                "maternal_"
            } else {
                ""
            };
            let base = if is_male {
                "uncle"
            } else if is_female {
                "aunt"
            } else {
                "pibling"
            };
            (
                format!("{side_prefix}{base}"),
                if is_male {
                    "Uncle".to_string()
                } else if is_female {
                    "Aunt".to_string()
                } else {
                    "Uncle / Aunt".to_string()
                },
                if is_male {
                    "Дядя".to_string()
                } else if is_female {
                    "Тётя".to_string()
                } else {
                    "Дядя / Тётя".to_string()
                },
            )
        }
        (1, 2) => (
            if is_male {
                "nephew".to_string()
            } else if is_female {
                "niece".to_string()
            } else {
                "nibling".to_string()
            },
            if is_male {
                "Nephew".to_string()
            } else if is_female {
                "Niece".to_string()
            } else {
                "Nephew / Niece".to_string()
            },
            if is_male {
                "Племянник".to_string()
            } else if is_female {
                "Племянница".to_string()
            } else {
                "Племянник / Племянница".to_string()
            },
        ),
        (u_val, 0) if u_val >= 3 => (
            if is_male {
                "great_grandfather".to_string()
            } else if is_female {
                "great_grandmother".to_string()
            } else {
                "great_grandparent".to_string()
            },
            if is_male {
                "Great-grandfather".to_string()
            } else if is_female {
                "Great-grandmother".to_string()
            } else {
                "Great-grandparent".to_string()
            },
            if is_male {
                "Прадедушка".to_string()
            } else if is_female {
                "Прабабушка".to_string()
            } else {
                "Прародитель".to_string()
            },
        ),
        (0, d_val) if d_val >= 3 => (
            if is_male {
                "great_grandson".to_string()
            } else if is_female {
                "great_granddaughter".to_string()
            } else {
                "great_grandchild".to_string()
            },
            "Great-grandchild".to_string(),
            "Правнук / Правнучка".to_string(),
        ),
        (u_val, d_val) if u_val >= 2 && d_val >= 2 => {
            let cousin_deg = u_val.min(d_val) - 1;
            let removed = u_val.abs_diff(d_val);
            let deg_word = match cousin_deg {
                1 => "first_cousin",
                2 => "second_cousin",
                3 => "third_cousin",
                _ => "cousin",
            };
            let key = match removed {
                0 => deg_word.to_string(),
                1 => format!("{deg_word}_once_removed"),
                2 => format!("{deg_word}_twice_removed"),
                r => format!("{deg_word}_{r}_times_removed"),
            };
            let label_en = if removed == 0 {
                format!(
                    "{} cousin",
                    match cousin_deg {
                        1 => "First",
                        2 => "Second",
                        3 => "Third",
                        _ => "Distant",
                    }
                )
            } else {
                format!("Cousin ({cousin_deg} degree, {removed}x removed)")
            };
            let label_ru = if cousin_deg == 1 && removed == 0 {
                if is_male {
                    "Двоюродный брат".to_string()
                } else if is_female {
                    "Двоюродная сестра".to_string()
                } else {
                    "Двоюродный брат / сестра".to_string()
                }
            } else {
                "Кровный родственник".to_string()
            };
            (key, label_en, label_ru)
        }
        _ => (
            "blood_relative".to_string(),
            "Blood relative".to_string(),
            "Кровный родственник".to_string(),
        ),
    }
}

pub(crate) fn classify_affinal_relation(
    edge_types: &[&str],
    path: &[String],
    person_by_id: &HashMap<&str, &TreePersonNodeDto>,
) -> (String, String, String) {
    let to_id = path.last().map(|s| s.as_str()).unwrap_or_default();
    let target_sex = person_by_id
        .get(to_id)
        .map(|p| p.sex.trim().to_ascii_uppercase())
        .unwrap_or_default();
    let is_male = target_sex == "M";
    let is_female = target_sex == "F";

    if edge_types == ["spouse"] {
        (
            if is_male {
                "husband".to_string()
            } else if is_female {
                "wife".to_string()
            } else {
                "spouse".to_string()
            },
            if is_male {
                "Husband".to_string()
            } else if is_female {
                "Wife".to_string()
            } else {
                "Spouse".to_string()
            },
            if is_male {
                "Муж".to_string()
            } else if is_female {
                "Жена".to_string()
            } else {
                "Супруг(а)".to_string()
            },
        )
    } else if edge_types == ["parent", "spouse"] {
        (
            if is_male {
                "stepfather".to_string()
            } else if is_female {
                "stepmother".to_string()
            } else {
                "stepparent".to_string()
            },
            if is_male {
                "Stepfather".to_string()
            } else if is_female {
                "Stepmother".to_string()
            } else {
                "Step-parent".to_string()
            },
            if is_male {
                "Отчим".to_string()
            } else if is_female {
                "Мачеха".to_string()
            } else {
                "Отчим / Мачеха".to_string()
            },
        )
    } else if edge_types == ["spouse", "child"] {
        (
            if is_male {
                "stepson".to_string()
            } else if is_female {
                "stepdaughter".to_string()
            } else {
                "stepchild".to_string()
            },
            if is_male {
                "Stepson".to_string()
            } else if is_female {
                "Stepdaughter".to_string()
            } else {
                "Stepchild".to_string()
            },
            if is_male {
                "Пасынок".to_string()
            } else if is_female {
                "Падчерица".to_string()
            } else {
                "Пасынок / Падчерица".to_string()
            },
        )
    } else if edge_types == ["parent", "spouse", "child"] && path.len() >= 4 {
        let parent_sex = person_by_id
            .get(path[1].as_str())
            .map(|p| p.sex.trim().to_ascii_uppercase())
            .unwrap_or_default();
        if parent_sex == "F" {
            (
                if is_male {
                    "step_brother_maternal_marriage".to_string()
                } else if is_female {
                    "step_sister_maternal_marriage".to_string()
                } else {
                    "step_sibling_maternal_marriage".to_string()
                },
                if is_male {
                    "Step-brother (via mother's marriage)".to_string()
                } else if is_female {
                    "Step-sister (via mother's marriage)".to_string()
                } else {
                    "Step-sibling (via mother's marriage)".to_string()
                },
                if is_male {
                    "Сводный брат (по браку матери)".to_string()
                } else if is_female {
                    "Сводная сестра (по браку матери)".to_string()
                } else {
                    "Сводный брат / сестра (по браку матери)".to_string()
                },
            )
        } else if parent_sex == "M" {
            (
                if is_male {
                    "step_brother_paternal_marriage".to_string()
                } else if is_female {
                    "step_sister_paternal_marriage".to_string()
                } else {
                    "step_sibling_paternal_marriage".to_string()
                },
                if is_male {
                    "Step-brother (via father's marriage)".to_string()
                } else if is_female {
                    "Step-sister (via father's marriage)".to_string()
                } else {
                    "Step-sibling (via father's marriage)".to_string()
                },
                if is_male {
                    "Сводный брат (по браку отца)".to_string()
                } else if is_female {
                    "Сводная сестра (по браку отца)".to_string()
                } else {
                    "Сводный брат / сестра (по браку отца)".to_string()
                },
            )
        } else {
            (
                if is_male {
                    "step_brother".to_string()
                } else if is_female {
                    "step_sister".to_string()
                } else {
                    "step_sibling".to_string()
                },
                if is_male {
                    "Step-brother".to_string()
                } else if is_female {
                    "Step-sister".to_string()
                } else {
                    "Step-sibling".to_string()
                },
                if is_male {
                    "Сводный брат".to_string()
                } else if is_female {
                    "Сводная сестра".to_string()
                } else {
                    "Сводный брат / сестра".to_string()
                },
            )
        }
    } else if edge_types == ["spouse", "parent"] {
        (
            if is_male {
                "father_in_law".to_string()
            } else if is_female {
                "mother_in_law".to_string()
            } else {
                "parent_in_law".to_string()
            },
            if is_male {
                "Father-in-law".to_string()
            } else if is_female {
                "Mother-in-law".to_string()
            } else {
                "Parent-in-law".to_string()
            },
            if is_male {
                "Тесть / Свёкор".to_string()
            } else if is_female {
                "Тёща / Свекровь".to_string()
            } else {
                "Родитель супруга(и)".to_string()
            },
        )
    } else if edge_types == ["child", "spouse"] {
        (
            if is_male {
                "son_in_law".to_string()
            } else if is_female {
                "daughter_in_law".to_string()
            } else {
                "child_in_law".to_string()
            },
            if is_male {
                "Son-in-law".to_string()
            } else if is_female {
                "Daughter-in-law".to_string()
            } else {
                "Child-in-law".to_string()
            },
            if is_male {
                "Зять".to_string()
            } else if is_female {
                "Невестка".to_string()
            } else {
                "Зять / Невестка".to_string()
            },
        )
    } else if edge_types == ["spouse", "parent", "child"]
        || edge_types == ["parent", "child", "spouse"]
    {
        (
            if is_male {
                "brother_in_law".to_string()
            } else if is_female {
                "sister_in_law".to_string()
            } else {
                "sibling_in_law".to_string()
            },
            if is_male {
                "Brother-in-law".to_string()
            } else if is_female {
                "Sister-in-law".to_string()
            } else {
                "Sibling-in-law".to_string()
            },
            if is_male {
                "Деверь / Зять / Шурин".to_string()
            } else if is_female {
                "Золовка / Невестка / Свояченица".to_string()
            } else {
                "Родственник(ца) по браку".to_string()
            },
        )
    } else {
        (
            "relative_by_marriage".to_string(),
            "Relative by marriage".to_string(),
            "Родственник по браку".to_string(),
        )
    }
}
