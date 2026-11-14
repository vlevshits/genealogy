#[allow(unused_imports)]
use super::*;

/// Executes an MCP JSON-RPC 2.0 request (`POST /api/v1/mcp`).
/// Returns `(http_status_code_u16, json_response_body)`.
pub fn handle_mcp_jsonrpc(
    store: &mut GenealogyPrivacyStore,
    caller: &McpCallerContext,
    req: McpJsonRpcRequest,
    current_year: i32,
    age_threshold: i32,
) -> (u16, Value) {
    let rpc_id = req.id.clone().unwrap_or(Value::Null);

    match req.method.as_str() {
        "initialize" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": rpc_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "genealogy-mcp-server",
                        "version": "1.0.0"
                    }
                }
            }),
        ),
        "notifications/initialized" | "ping" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": rpc_id,
                "result": {}
            }),
        ),
        "tools/list" => {
            let tools = list_authorized_mcp_tools(caller);
            (
                200,
                json!({
                    "jsonrpc": "2.0",
                    "id": rpc_id,
                    "result": {
                        "tools": tools
                    }
                }),
            )
        }
        "tools/call" => {
            let params = req.params.unwrap_or_else(|| json!({}));
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            let args = match params
                .get("arguments")
                .or_else(|| params.get("args"))
                .or_else(|| params.get("input"))
            {
                Some(Value::String(raw)) => {
                    let trimmed = raw.trim();
                    if trimmed.is_empty() {
                        json!({})
                    } else {
                        serde_json::from_str::<Value>(trimmed).unwrap_or_else(|_| json!({}))
                    }
                }
                Some(Value::Null) | None => params.clone(),
                Some(other) => other.clone(),
            };

            let all_tools = all_mcp_tools();
            let Some(tool_def) = all_tools.iter().find(|t| t.name == tool_name) else {
                return (
                    404,
                    json!({
                        "jsonrpc": "2.0",
                        "id": rpc_id,
                        "error": {
                            "code": -32601,
                            "error_code": "NOT_FOUND",
                            "message": format!("Unknown MCP tool: {tool_name}")
                        }
                    }),
                );
            };

            if !is_caller_authorized_for_tool(caller, &tool_def.tier, &tool_def.required_scope) {
                return (
                    403,
                    json!({
                        "jsonrpc": "2.0",
                        "id": rpc_id,
                        "code": "FORBIDDEN",
                        "error": {
                            "code": -32600,
                            "error_code": "FORBIDDEN",
                            "message": format!(
                                "Forbidden: access {:?} with scopes {:?} is not authorized to invoke tool '{}' (requires scope '{}')",
                                caller.access, caller.scopes, tool_name, tool_def.required_scope
                            )
                        }
                    }),
                );
            }

            match execute_mcp_tool(store, caller, tool_name, &args, current_year, age_threshold) {
                Ok(tool_output) => {
                    let text_repr = serde_json::to_string_pretty(&tool_output)
                        .unwrap_or_else(|_| "{}".to_string());
                    (
                        200,
                        json!({
                            "jsonrpc": "2.0",
                            "id": rpc_id,
                            "result": {
                                "content": [
                                    {
                                        "type": "text",
                                        "text": text_repr
                                    }
                                ],
                                "structuredContent": tool_output,
                                "isError": false
                            }
                        }),
                    )
                }
                Err(app_err) => {
                    let (rpc_code, msg) = match &app_err {
                        AppError::Forbidden(m) => (-32600, m.clone()),
                        AppError::NotFound | AppError::PersonNotFound => {
                            (-32004, "Record not found".to_string())
                        }
                        AppError::ValidationFailed(message, _) => (-32602, message.clone()),
                        AppError::VersionConflict(m) => (-32009, m.clone()),
                        _ => (-32603, "Tool execution error".to_string()),
                    };
                    (
                        200,
                        json!({
                            "jsonrpc": "2.0",
                            "id": rpc_id,
                            "error": {
                                "code": rpc_code,
                                "message": msg
                            }
                        }),
                    )
                }
            }
        }
        other => (
            400,
            json!({
                "jsonrpc": "2.0",
                "id": rpc_id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {other}")
                }
            }),
        ),
    }
}

pub(crate) fn mcp_privacy_access(caller: &McpCallerContext) -> CallerAccess {
    // Per spec 19 §4.2.3: Viewer MCP tokens have living individual data masked (initials only)
    // to prevent bulk PII scraping by external LLM providers.
    if caller.access == CallerAccess::Viewer {
        CallerAccess::AnonymousPublic
    } else {
        caller.access
    }
}

pub(crate) fn redact_person_for_mcp(
    person: &TreePersonNodeDto,
    privacy_access: &CallerAccess,
) -> Option<TreePersonNodeDto> {
    let mut redacted = redact_person_for_tree_view(person, privacy_access)?;
    if *privacy_access == CallerAccess::AnonymousPublic && person.is_living {
        redacted.primary_name.surname = person.primary_name.surname.clone();
        let birth_year_str = person
            .birth_sort
            .map(|bs| (bs / 10_000).to_string())
            .or_else(|| {
                person
                    .birth
                    .as_ref()
                    .and_then(|b| b.date.as_deref())
                    .and_then(|d| {
                        GenDate::parse(d)
                            .compute_sort_key()
                            .map(|bs| (bs / 10_000).to_string())
                    })
            });
        if let Some(yr) = birth_year_str {
            redacted.birth = Some(VitalEventSummaryDto {
                date: Some(yr),
                place: None,
            });
        }
    }
    Some(redacted)
}

pub(crate) fn redact_family_for_mcp(
    fam: &FamilyDto,
    persons: &[TreePersonNodeDto],
    privacy_access: &CallerAccess,
) -> Option<FamilyDto> {
    if fam.is_private && !privacy_access.can_view_private() {
        return None;
    }

    let is_person_visible = |pid: &str| -> bool {
        persons
            .iter()
            .find(|p| p.id == pid)
            .and_then(|p| redact_person_for_mcp(p, privacy_access))
            .is_some()
    };

    let partner1_id = fam
        .partner1_id
        .as_ref()
        .filter(|id| is_person_visible(id))
        .cloned();
    let partner2_id = fam
        .partner2_id
        .as_ref()
        .filter(|id| is_person_visible(id))
        .cloned();
    let children: Vec<FamilyChildDto> = fam
        .children
        .iter()
        .filter(|c| is_person_visible(&c.child_id))
        .cloned()
        .collect();

    if partner1_id.is_none() && partner2_id.is_none() && children.is_empty() {
        return None;
    }

    let is_masked_union = *privacy_access == CallerAccess::AnonymousPublic && {
        let p1_living = partner1_id
            .as_ref()
            .and_then(|pid| persons.iter().find(|p| &p.id == pid))
            .is_some_and(|p| p.is_living);
        let p2_living = partner2_id
            .as_ref()
            .and_then(|pid| persons.iter().find(|p| &p.id == pid))
            .is_some_and(|p| p.is_living);
        p1_living || p2_living
    };

    Some(FamilyDto {
        id: fam.id.clone(),
        gid: fam.gid.clone(),
        partner1_id,
        partner2_id,
        relationship_type: fam.relationship_type.clone(),
        is_private: fam.is_private,
        children,
        external_ids: fam.external_ids.clone(),
        version: fam.version,
        marriage_date: if is_masked_union {
            None
        } else {
            fam.marriage_date.clone()
        },
        marriage_place: if is_masked_union {
            None
        } else {
            fam.marriage_place.clone()
        },
    })
}

pub(crate) fn execute_mcp_tool(
    store: &mut GenealogyPrivacyStore,
    caller: &McpCallerContext,
    tool_name: &str,
    args: &Value,
    current_year: i32,
    age_threshold: i32,
) -> Result<Value, AppError> {
    let privacy_access = mcp_privacy_access(caller);

    match tool_name {
        // ====================================================================
        // Tier A — Read & Discovery Tools (read:tree)
        // ====================================================================
        "search_individuals" => {
            let query = args
                .get("query")
                .or_else(|| args.get("q"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_lowercase();
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(25)
                .min(100) as usize;

            let mut matches = Vec::new();
            for p in &store.persons {
                let Some(redacted) = redact_person_for_mcp(p, &privacy_access) else {
                    continue;
                };
                if query.is_empty()
                    || redacted
                        .primary_name
                        .given_name
                        .to_lowercase()
                        .contains(&query)
                    || redacted
                        .primary_name
                        .surname
                        .to_lowercase()
                        .contains(&query)
                    || redacted.gid.to_lowercase().contains(&query)
                {
                    matches.push(redacted);
                    if matches.len() >= limit {
                        break;
                    }
                }
            }
            Ok(json!({ "items": matches, "total": matches.len() }))
        }
        "get_individual" => {
            let id_or_gid = args
                .get("id")
                .or_else(|| args.get("person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = store
                .resolve_person_id(id_or_gid)
                .ok_or(AppError::NotFound)?;
            let person = store
                .persons
                .iter()
                .find(|p| p.id == resolved)
                .ok_or(AppError::NotFound)?;
            let redacted =
                redact_person_for_mcp(person, &privacy_access).ok_or(AppError::NotFound)?;
            Ok(serde_json::to_value(redacted).unwrap_or_else(|_| json!({})))
        }
        "get_family_immediate" => {
            let id_or_gid = args
                .get("id")
                .or_else(|| args.get("person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = store
                .resolve_person_id(id_or_gid)
                .ok_or(AppError::NotFound)?;
            let person = store
                .persons
                .iter()
                .find(|p| p.id == resolved)
                .ok_or(AppError::NotFound)?;
            redact_person_for_mcp(person, &privacy_access).ok_or(AppError::NotFound)?;

            let fams: Vec<FamilyDto> = store
                .families
                .iter()
                .filter(|f| {
                    f.partner1_id.as_deref() == Some(resolved.as_str())
                        || f.partner2_id.as_deref() == Some(resolved.as_str())
                        || f.children.iter().any(|c| c.child_id == resolved)
                })
                .filter_map(|f| redact_family_for_mcp(f, &store.persons, &privacy_access))
                .collect();
            Ok(json!({ "person_id": resolved, "families": fams }))
        }
        "get_ancestors_descendants" => {
            let id_or_gid = args
                .get("id")
                .or_else(|| args.get("person_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = store
                .resolve_person_id(id_or_gid)
                .ok_or(AppError::NotFound)?;
            let person = store
                .persons
                .iter()
                .find(|p| p.id == resolved)
                .ok_or(AppError::NotFound)?;
            redact_person_for_mcp(person, &privacy_access).ok_or(AppError::NotFound)?;

            let persons: Vec<TreePersonNodeDto> = store
                .persons
                .iter()
                .filter_map(|p| redact_person_for_mcp(p, &privacy_access))
                .collect();
            let families: Vec<FamilyDto> = store
                .families
                .iter()
                .filter_map(|f| redact_family_for_mcp(f, &store.persons, &privacy_access))
                .collect();
            Ok(json!({
                "root_person_id": resolved,
                "persons": persons,
                "families": families
            }))
        }
        "calculate_kinship" => {
            let from_raw = args
                .get("person_a_id")
                .or_else(|| args.get("from"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let to_raw = args
                .get("person_b_id")
                .or_else(|| args.get("to"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let from_id = store
                .resolve_person_id(from_raw)
                .ok_or(AppError::NotFound)?;
            let to_id = store.resolve_person_id(to_raw).ok_or(AppError::NotFound)?;
            let visible_persons: Vec<TreePersonNodeDto> = store
                .persons
                .iter()
                .filter_map(|p| redact_person_for_mcp(p, &privacy_access))
                .collect();
            let res =
                calculate_kinship_relation(&from_id, &to_id, &visible_persons, &store.families)
                    .ok_or(AppError::NotFound)?;
            Ok(serde_json::to_value(res).unwrap_or_else(|_| json!({})))
        }
        "search_places" => {
            let query = args
                .get("query")
                .or_else(|| args.get("q"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_lowercase();
            let items: Vec<PlaceSummaryDto> = store
                .places
                .iter()
                .filter(|p| {
                    query.is_empty()
                        || p.name.to_lowercase().contains(&query)
                        || p.full_name.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            Ok(json!({ "items": items, "total": items.len() }))
        }
        "get_place_hierarchy" => {
            let id_or_gid = args
                .get("id")
                .or_else(|| args.get("place_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = store
                .resolve_place_id(id_or_gid)
                .ok_or(AppError::NotFound)?;
            let place = store
                .places
                .iter()
                .find(|p| p.id == resolved)
                .cloned()
                .ok_or(AppError::NotFound)?;
            let children: Vec<PlaceSummaryDto> = store
                .places
                .iter()
                .filter(|p| p.parent_id.as_deref() == Some(resolved.as_str()))
                .cloned()
                .collect();
            Ok(json!({ "place": place, "children": children }))
        }
        "list_events_timeline" => {
            let raw_person_id = args
                .get("person_id")
                .or_else(|| args.get("id"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let person_filter = match raw_person_id {
                Some(id) => Some(store.resolve_person_id(id).ok_or(AppError::NotFound)?),
                None => None,
            };
            if let Some(ref pid) = person_filter {
                let person = store
                    .persons
                    .iter()
                    .find(|p| &p.id == pid)
                    .ok_or(AppError::NotFound)?;
                redact_person_for_mcp(person, &privacy_access).ok_or(AppError::NotFound)?;
            }
            let items: Vec<LifeEventDto> = store
                .events
                .iter()
                .filter(|e| {
                    person_filter
                        .as_ref()
                        .is_none_or(|pid| e.individual_id.as_ref() == Some(pid))
                })
                .filter_map(|e| {
                    redact_event_for_tree_view(e, &store.persons, &store.families, &privacy_access)
                })
                .collect();
            Ok(json!({ "items": items, "total": items.len() }))
        }

        // ====================================================================
        // Tier B — Story Draft & Guided Intake Tools (write:draft)
        // ====================================================================
        "submit_story_draft" => {
            let raw_text = args
                .get("raw_text")
                .or_else(|| args.get("story"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            let speaker_person_id = args
                .get("speaker_person_id")
                .or_else(|| args.get("speaker_id"))
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            let extracted_data: Option<DraftExtractedDataDto> = args
                .get("extracted_data")
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok());
            let draft = parse_story_into_draft(
                store,
                ParseStoryRequest {
                    title,
                    raw_text,
                    speaker_person_id,
                    speaker_id: None,
                    is_new_speaker: None,
                    language: None,
                    audio_media_id: None,
                    engine: None,
                    extracted_data,
                },
                &caller.user_id,
            )?;
            Ok(serde_json::to_value(draft).unwrap_or_else(|_| json!({})))
        }
        "get_story_draft" => {
            let draft_id = args
                .get("draft_id")
                .or_else(|| args.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let draft = store
                .story_drafts
                .iter()
                .find(|d| d.id == draft_id)
                .cloned()
                .ok_or(AppError::NotFound)?;
            Ok(serde_json::to_value(draft).unwrap_or_else(|_| json!({})))
        }
        "refine_story_draft" => {
            let draft_id = args
                .get("draft_id")
                .or_else(|| args.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let clarification_text = args
                .get("clarification_text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let draft = refine_story_draft(store, draft_id, clarification_text)?;
            Ok(serde_json::to_value(draft).unwrap_or_else(|_| json!({})))
        }
        "reconcile_story_draft" => {
            let draft_id = args
                .get("draft_id")
                .or_else(|| args.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let draft = reconcile_story_draft(store, draft_id)?;
            Ok(serde_json::to_value(draft).unwrap_or_else(|_| json!({})))
        }
        "commit_story_draft" => {
            let draft_id = args
                .get("draft_id")
                .or_else(|| args.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let res = commit_story_draft(
                store,
                draft_id,
                CommitStoryDraftRequest {
                    match_overrides: None,
                },
                Some(caller.user_id.clone()),
                Some(caller.username.clone()),
                current_year,
                age_threshold,
            )?;
            Ok(serde_json::to_value(res).unwrap_or_else(|_| json!({})))
        }
        "discard_story_draft" => {
            let draft_id = args
                .get("draft_id")
                .or_else(|| args.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let pos = store
                .story_drafts
                .iter()
                .position(|d| d.id == draft_id)
                .ok_or(AppError::NotFound)?;
            store.story_drafts.remove(pos);
            Ok(json!({ "status": "discarded", "draft_id": draft_id }))
        }
        _ => execute_mcp_mutation_or_admin_tool(
            store,
            caller,
            tool_name,
            args,
            current_year,
            age_threshold,
        ),
    }
}
