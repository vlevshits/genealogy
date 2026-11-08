#[allow(unused_imports)]
use super::*;

pub(crate) fn execute_mcp_mutation_or_admin_tool(
    store: &mut GenealogyPrivacyStore,
    caller: &McpCallerContext,
    tool_name: &str,
    args: &Value,
    current_year: i32,
    age_threshold: i32,
) -> Result<Value, AppError> {
    match tool_name {
        // ====================================================================
        // Tier B — Direct Tree Mutation Tools (write:direct)
        // ====================================================================
        "create_individual" => {
            let given_name = args
                .get("given_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if given_name.is_empty() {
                return Err(AppError::ValidationFailed(
                    "given_name is required".to_string(),
                    Some(HashMap::from([(
                        "given_name".to_string(),
                        "required".to_string(),
                    )])),
                ));
            }
            let surname = args
                .get("surname")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let patronymic = args
                .get("patronymic")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let sex = args
                .get("sex")
                .and_then(|v| v.as_str())
                .unwrap_or("U")
                .to_uppercase();
            let birth_date = args
                .get("birth_date")
                .or_else(|| args.get("birth_date_text"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let birth_place = args
                .get("birth_place")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let tx_id = format!("tx-{}", Uuid::new_v4());
            let new_id = Uuid::new_v4().to_string();
            let new_gid = format!("I{:04}", store.next_gid_seq);
            store.next_gid_seq += 1;

            let birth = if birth_date.is_some() || birth_place.is_some() {
                Some(VitalEventSummaryDto {
                    date: birth_date.clone(),
                    place: birth_place.clone(),
                })
            } else {
                None
            };

            let birth_sort = birth_date
                .as_deref()
                .and_then(|d| GenDate::parse(d).compute_sort_key());

            let new_person = TreePersonNodeDto {
                id: new_id.clone(),
                gid: new_gid,
                sex: if matches!(sex.as_str(), "M" | "F") {
                    sex
                } else {
                    "U".to_string()
                },
                is_living: args
                    .get("is_living")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                living_override: None,
                is_private: false,
                primary_name: PersonNameDto {
                    given_name: given_name.clone(),
                    patronymic,
                    surname: surname.clone(),
                },
                birth_surname: None,
                nickname: None,
                name_prefix: None,
                name_suffix: None,
                biography: args
                    .get("biography")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
                birth,
                death: None,
                avatar_url: None,
                avatar_media_id: None,
                avatar_crop: None,
                notes: None,
                birth_sort,
                death_sort: None,
                has_death_event: false,
                alt_names: vec![],
                external_ids: HashMap::new(),
                version: 1,
                tx_id: Some(tx_id.clone()),
                generation: None,
                is_direct_lineage: None,
            };

            store.persons.push(new_person.clone());
            log_change(
                store,
                Some(tx_id),
                "mcp.create_individual",
                "individual",
                &new_id,
                "create",
                Some(format!("{given_name} {surname}").trim().to_string()),
                format!("Created individual {given_name} {surname} via MCP"),
                None,
                Some(serde_json::to_value(&new_person).unwrap_or_else(|_| json!({}))),
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
            );
            recompute_all_individuals_vitals(store, current_year, age_threshold);
            let person_val = serde_json::to_value(&new_person).unwrap_or_else(|_| json!({}));
            let mut out = person_val.clone();
            if let Some(obj) = out.as_object_mut() {
                obj.insert("person".to_string(), person_val);
            }
            Ok(out)
        }
        "update_individual" => {
            let id_or_gid = args
                .get("id")
                .or_else(|| args.get("person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = store
                .resolve_person_id(id_or_gid)
                .ok_or(AppError::NotFound)?;
            let idx = store
                .persons
                .iter()
                .position(|p| p.id == resolved)
                .ok_or(AppError::NotFound)?;
            let before = store.persons[idx].clone();
            if let Some(gn) = args.get("given_name").and_then(|v| v.as_str()) {
                store.persons[idx].primary_name.given_name = gn.trim().to_string();
            }
            if let Some(sn) = args.get("surname").and_then(|v| v.as_str()) {
                store.persons[idx].primary_name.surname = sn.trim().to_string();
            }
            if let Some(pat) = args.get("patronymic").and_then(|v| v.as_str()) {
                store.persons[idx].primary_name.patronymic = Some(pat.trim().to_string());
            }
            if let Some(bio) = args.get("biography").and_then(|v| v.as_str()) {
                store.persons[idx].biography = Some(bio.trim().to_string());
            }
            let tx_id = format!("tx-{}", Uuid::new_v4());
            store.persons[idx].version += 1;
            store.persons[idx].tx_id = Some(tx_id.clone());
            let after = store.persons[idx].clone();
            log_change(
                store,
                Some(tx_id),
                "mcp.update_individual",
                "individual",
                &resolved,
                "update",
                Some(after.primary_name.given_name.clone()),
                format!("Updated individual {} via MCP", after.id),
                Some(serde_json::to_value(&before).unwrap_or_else(|_| json!({}))),
                Some(serde_json::to_value(&after).unwrap_or_else(|_| json!({}))),
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
            );
            let person_val = serde_json::to_value(&after).unwrap_or_else(|_| json!({}));
            let mut out = person_val.clone();
            if let Some(obj) = out.as_object_mut() {
                obj.insert("person".to_string(), person_val);
            }
            Ok(out)
        }
        "create_family" | "link_spouses" => {
            let p1 = args
                .get("partner1_id")
                .and_then(|v| v.as_str())
                .and_then(|id| store.resolve_person_id(id));
            let p2 = args
                .get("partner2_id")
                .and_then(|v| v.as_str())
                .and_then(|id| store.resolve_person_id(id));
            let fam_id = Uuid::new_v4().to_string();
            let fam_gid = format!("F{:04}", store.next_family_gid_seq);
            store.next_family_gid_seq += 1;
            let tx_id = format!("tx-{}", Uuid::new_v4());
            let fam = FamilyDto {
                id: fam_id.clone(),
                gid: fam_gid.clone(),
                partner1_id: p1,
                partner2_id: p2,
                relationship_type: "married".to_string(),
                is_private: false,
                children: vec![],
                external_ids: HashMap::new(),
                version: 1,
                marriage_date: None,
                marriage_place: None,
            };
            store.families.push(fam.clone());
            log_change(
                store,
                Some(tx_id),
                "mcp.create_family",
                "family",
                &fam_id,
                "create",
                Some(fam_gid),
                "Created family via MCP".to_string(),
                None,
                Some(serde_json::to_value(&fam).unwrap_or_else(|_| json!({}))),
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
            );
            Ok(serde_json::to_value(fam).unwrap_or_else(|_| json!({})))
        }
        "link_parent_child" => {
            let parent_id = args
                .get("parent_id")
                .and_then(|v| v.as_str())
                .and_then(|id| store.resolve_person_id(id))
                .ok_or(AppError::NotFound)?;
            let child_id = args
                .get("child_id")
                .and_then(|v| v.as_str())
                .and_then(|id| store.resolve_person_id(id))
                .ok_or(AppError::NotFound)?;
            let tx_id = format!("tx-{}", Uuid::new_v4());

            if let Some(fam_idx) = store.families.iter().position(|f| {
                f.partner1_id.as_deref() == Some(parent_id.as_str())
                    || f.partner2_id.as_deref() == Some(parent_id.as_str())
            }) {
                let before = store.families[fam_idx].clone();
                if !store.families[fam_idx]
                    .children
                    .iter()
                    .any(|c| c.child_id == child_id)
                {
                    let sort_order = store.families[fam_idx].children.len() as i32;
                    store.families[fam_idx].children.push(FamilyChildDto {
                        child_id,
                        sort_order,
                        rel_to_partner1: "birth".to_string(),
                        rel_to_partner2: "birth".to_string(),
                    });
                }
                store.families[fam_idx].version += 1;
                let after = store.families[fam_idx].clone();
                log_change(
                    store,
                    Some(tx_id),
                    "mcp.link_parent_child",
                    "family",
                    &after.id,
                    "update",
                    Some(after.gid.clone()),
                    "Linked parent and child via MCP".to_string(),
                    Some(serde_json::to_value(&before).unwrap_or_else(|_| json!({}))),
                    Some(serde_json::to_value(&after).unwrap_or_else(|_| json!({}))),
                    Some(caller.user_id.clone()),
                    Some(caller.username.clone()),
                );
                Ok(serde_json::to_value(after).unwrap_or_else(|_| json!({})))
            } else {
                let fam_id = Uuid::new_v4().to_string();
                let fam_gid = format!("F{:04}", store.next_family_gid_seq);
                store.next_family_gid_seq += 1;
                let fam = FamilyDto {
                    id: fam_id.clone(),
                    gid: fam_gid.clone(),
                    partner1_id: Some(parent_id),
                    partner2_id: None,
                    relationship_type: "married".to_string(),
                    is_private: false,
                    children: vec![FamilyChildDto {
                        child_id,
                        sort_order: 0,
                        rel_to_partner1: "birth".to_string(),
                        rel_to_partner2: "birth".to_string(),
                    }],
                    external_ids: HashMap::new(),
                    version: 1,
                    marriage_date: None,
                    marriage_place: None,
                };
                store.families.push(fam.clone());
                log_change(
                    store,
                    Some(tx_id),
                    "mcp.link_parent_child",
                    "family",
                    &fam_id,
                    "create",
                    Some(fam_gid),
                    "Created parent-child family via MCP".to_string(),
                    None,
                    Some(serde_json::to_value(&fam).unwrap_or_else(|_| json!({}))),
                    Some(caller.user_id.clone()),
                    Some(caller.username.clone()),
                );
                Ok(serde_json::to_value(fam).unwrap_or_else(|_| json!({})))
            }
        }
        "create_event" => {
            let person_id = args
                .get("person_id")
                .and_then(|v| v.as_str())
                .and_then(|id| store.resolve_person_id(id))
                .ok_or(AppError::NotFound)?;
            let event_type = args
                .get("event_type")
                .and_then(|v| v.as_str())
                .unwrap_or("EVEN")
                .to_uppercase();
            let date_text = args
                .get("date_text")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            let place = args
                .get("place")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);

            let ev_id = Uuid::new_v4().to_string();
            let ev_gid = format!("E{:04}", store.next_event_gid_seq);
            store.next_event_gid_seq += 1;
            let tx_id = format!("tx-{}", Uuid::new_v4());
            let parsed_date = date_text.as_deref().map(GenDate::parse);
            let date_sort = parsed_date.as_ref().and_then(GenDate::compute_sort_key);
            let new_ev = LifeEventDto {
                id: ev_id.clone(),
                gid: ev_gid.clone(),
                individual_id: Some(person_id),
                family_id: None,
                event_type,
                custom_type: None,
                date: parsed_date,
                date_sort,
                gedcom_date: date_text,
                place_id: None,
                place_full_name: place,
                description: args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
                source_text: None,
                media_ids: vec![],
                is_private: false,
                version: 1,
            };
            store.events.push(new_ev.clone());
            log_change(
                store,
                Some(tx_id),
                "mcp.create_event",
                "event",
                &ev_id,
                "create",
                Some(ev_gid),
                "Created event via MCP".to_string(),
                None,
                Some(serde_json::to_value(&new_ev).unwrap_or_else(|_| json!({}))),
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
            );
            recompute_all_individuals_vitals(store, current_year, age_threshold);
            Ok(serde_json::to_value(new_ev).unwrap_or_else(|_| json!({})))
        }
        "create_or_resolve_place" => {
            let name = args
                .get("name")
                .or_else(|| args.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if name.is_empty() {
                return Err(AppError::ValidationFailed(
                    "Place name is required".to_string(),
                    Some(HashMap::from([(
                        "name".to_string(),
                        "required".to_string(),
                    )])),
                ));
            }
            if let Some(existing) = store.places.iter().find(|p| {
                p.name.eq_ignore_ascii_case(name) || p.full_name.eq_ignore_ascii_case(name)
            }) {
                return Ok(serde_json::to_value(existing).unwrap_or_else(|_| json!({})));
            }
            let pid = Uuid::new_v4().to_string();
            let pgid = format!("P{:04}", store.next_place_gid_seq);
            store.next_place_gid_seq += 1;
            let tx_id = format!("tx-{}", Uuid::new_v4());
            let new_place = PlaceSummaryDto {
                id: pid.clone(),
                gid: pgid,
                name: name.to_string(),
                place_type: args
                    .get("place_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("city")
                    .to_string(),
                parent_id: None,
                full_name: name.to_string(),
                country: None,
                latitude: None,
                longitude: None,
                events_count: 0,
                children_count: 0,
                alt_names: vec![],
                version: 1,
            };
            store.places.push(new_place.clone());
            log_change(
                store,
                Some(tx_id),
                "mcp.create_place",
                "place",
                &pid,
                "create",
                Some(name.to_string()),
                format!("Created place {name} via MCP"),
                None,
                Some(serde_json::to_value(&new_place).unwrap_or_else(|_| json!({}))),
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
            );
            recompute_all_places_metadata(store);
            Ok(serde_json::to_value(new_place).unwrap_or_else(|_| json!({})))
        }
        "add_research_note" => {
            let entity_id = args
                .get("entity_id")
                .or_else(|| args.get("person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let body = args
                .get("text")
                .or_else(|| args.get("body"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let note_id = Uuid::new_v4().to_string();
            let note = NoteDto {
                id: note_id,
                entity_type: "individual".to_string(),
                entity_id,
                body,
                is_private: false,
                sort_order: 0,
                version: 1,
            };
            store.notes.push(note.clone());
            Ok(serde_json::to_value(note).unwrap_or_else(|_| json!({})))
        }

        // ====================================================================
        // Tier C — Admin Quality & Maintenance Tools (admin:maintenance)
        // ====================================================================
        "list_duplicate_candidates" => {
            let min_score = args.get("min_score").and_then(|v| v.as_i64()).unwrap_or(50) as i32;
            let items: Vec<_> = compute_duplicate_candidates(store, min_score)
                .into_iter()
                .filter(|c| c.score >= min_score)
                .collect();
            Ok(json!({ "items": items, "total": items.len() }))
        }
        "merge_duplicate_individuals" => {
            let primary_person_id = args
                .get("surviving_id")
                .or_else(|| args.get("primary_person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let secondary_person_id = args
                .get("duplicate_id")
                .or_else(|| args.get("secondary_person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let res = execute_person_merge(
                store,
                PersonMergeRequest {
                    keep_id: Some(primary_person_id.clone()),
                    merge_id: Some(secondary_person_id.clone()),
                    target_id: None,
                    source_id: None,
                    primary_person_id: Some(primary_person_id),
                    duplicate_person_id: Some(secondary_person_id),
                    field_choices: HashMap::new(),
                    field_overrides: HashMap::new(),
                },
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
                "./data/backups",
            )?;
            Ok(serde_json::to_value(res).unwrap_or_else(|_| json!({})))
        }
        "analyze_place_hierarchy_issues" => {
            let res = analyze_place_hierarchies(store, &PlaceHierarchyAnalysisQuery::default());
            Ok(serde_json::to_value(res).unwrap_or_else(|_| json!({})))
        }
        "restructure_place_hierarchy" => {
            let place_ids: Option<Vec<String>> = args
                .get("issue_ids")
                .or_else(|| args.get("place_ids"))
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            let min_confidence = args
                .get("min_confidence")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);
            let res = batch_fix_places(
                store,
                BatchFixPlacesRequest {
                    min_confidence,
                    place_ids,
                    issue_type: None,
                    category: None,
                },
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
                current_year,
                age_threshold,
            )?;
            Ok(serde_json::to_value(res).unwrap_or_else(|_| json!({})))
        }
        "undo_transaction" => {
            let tx_id = args.get("tx_id").and_then(|v| v.as_str()).unwrap_or("");
            let res = undo_transaction(
                store,
                tx_id,
                &caller.user_id,
                Some(caller.username.clone()),
                caller.access.is_admin(),
                age_threshold,
                current_year,
            )?;
            Ok(serde_json::to_value(res).unwrap_or_else(|_| json!({})))
        }
        _ => Err(AppError::NotFound),
    }
}
