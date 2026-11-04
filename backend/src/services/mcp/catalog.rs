#[allow(unused_imports)]
use super::*;

pub fn all_mcp_tools() -> Vec<McpToolDefinitionDto> {
    vec![
        // Tier A — Read & Discovery (read:tree)
        McpToolDefinitionDto {
            name: "search_individuals".to_string(),
            description: "Full-text and fuzzy search across people by name, birth year, and place."
                .to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "get_individual".to_string(),
            description: "Fetch full profile, vitals, events, and family links for an individual."
                .to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "get_family_immediate".to_string(),
            description: "Fetch immediate parents, partners/spouses, siblings, and children."
                .to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "get_ancestors_descendants".to_string(),
            description: "Traverse ancestors and descendants up to N generations.".to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "calculate_kinship".to_string(),
            description: "Compute exact kinship path between two individuals.".to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["person_a_id", "person_b_id"],
                "properties": {
                    "person_a_id": { "type": "string" },
                    "person_b_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "search_places".to_string(),
            description: "Search geographical places and jurisdictions.".to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "get_place_hierarchy".to_string(),
            description: "Fetch a place's parent chain and child localities.".to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "list_events_timeline".to_string(),
            description: "Query chronological life events.".to_string(),
            tier: "A".to_string(),
            required_scope: "read:tree".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "person_id": { "type": "string" }
                }
            }),
        },
        // Tier B — Story Draft & Guided Intake (write:draft)
        McpToolDefinitionDto {
            name: "submit_story_draft".to_string(),
            description: "Parse a family story into a sandbox draft subtree.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:draft".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["raw_text"],
                "properties": {
                    "title": { "type": "string" },
                    "raw_text": { "type": "string" },
                    "speaker_person_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "get_story_draft".to_string(),
            description: "Fetch an existing story intake draft.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:draft".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["draft_id"],
                "properties": {
                    "draft_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "refine_story_draft".to_string(),
            description: "Apply a follow-up clarification to a story intake draft.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:draft".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["draft_id", "clarification_text"],
                "properties": {
                    "draft_id": { "type": "string" },
                    "clarification_text": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "reconcile_story_draft".to_string(),
            description: "Run relationship-centric reconciliation on a story draft.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:draft".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["draft_id"],
                "properties": {
                    "draft_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "commit_story_draft".to_string(),
            description: "Atomically commit a reconciled story draft into the live family tree."
                .to_string(),
            tier: "B".to_string(),
            required_scope: "write:draft".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["draft_id"],
                "properties": {
                    "draft_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "discard_story_draft".to_string(),
            description: "Discard a story intake draft without modifying the tree.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:draft".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["draft_id"],
                "properties": {
                    "draft_id": { "type": "string" }
                }
            }),
        },
        // Tier B — Direct Tree Mutations (write:direct)
        McpToolDefinitionDto {
            name: "create_individual".to_string(),
            description: "Create a new individual in the family tree.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["given_name"],
                "properties": {
                    "given_name": { "type": "string" },
                    "patronymic": { "type": "string" },
                    "surname": { "type": "string" },
                    "sex": { "type": "string" },
                    "birth_date": { "type": "string" },
                    "birth_place": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "update_individual".to_string(),
            description: "Update an existing individual's profile.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string" },
                    "given_name": { "type": "string" },
                    "patronymic": { "type": "string" },
                    "surname": { "type": "string" },
                    "biography": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "create_family".to_string(),
            description: "Create a family union between two partners.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "partner1_id": { "type": "string" },
                    "partner2_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "link_parent_child".to_string(),
            description: "Link a parent and child in the family tree.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["parent_id", "child_id"],
                "properties": {
                    "parent_id": { "type": "string" },
                    "child_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "link_spouses".to_string(),
            description: "Link two individuals as spouses/partners.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["partner1_id", "partner2_id"],
                "properties": {
                    "partner1_id": { "type": "string" },
                    "partner2_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "create_event".to_string(),
            description: "Create a life event for an individual.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["person_id", "event_type"],
                "properties": {
                    "person_id": { "type": "string" },
                    "event_type": { "type": "string" },
                    "date_text": { "type": "string" },
                    "place": { "type": "string" },
                    "description": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "create_or_resolve_place".to_string(),
            description: "Resolve or create a place in the gazetteer.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["name"],
                "properties": {
                    "name": { "type": "string" },
                    "place_type": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "add_research_note".to_string(),
            description: "Attach a research note to an individual.".to_string(),
            tier: "B".to_string(),
            required_scope: "write:direct".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["entity_id", "text"],
                "properties": {
                    "entity_id": { "type": "string" },
                    "text": { "type": "string" }
                }
            }),
        },
        // Tier C — Admin Quality & Maintenance (admin:maintenance)
        McpToolDefinitionDto {
            name: "list_duplicate_candidates".to_string(),
            description: "List relationship-centric duplicate individual candidates.".to_string(),
            tier: "C".to_string(),
            required_scope: "admin:maintenance".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "min_score": { "type": "integer" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "merge_duplicate_individuals".to_string(),
            description: "Atomically merge two duplicate individuals.".to_string(),
            tier: "C".to_string(),
            required_scope: "admin:maintenance".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["surviving_id", "duplicate_id"],
                "properties": {
                    "surviving_id": { "type": "string" },
                    "duplicate_id": { "type": "string" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "analyze_place_hierarchy_issues".to_string(),
            description: "Analyze unparsed or inconsistent place hierarchies.".to_string(),
            tier: "C".to_string(),
            required_scope: "admin:maintenance".to_string(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        McpToolDefinitionDto {
            name: "restructure_place_hierarchy".to_string(),
            description: "Apply automated place hierarchy restructuring and deduplication."
                .to_string(),
            tier: "C".to_string(),
            required_scope: "admin:maintenance".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "issue_ids": { "type": "array", "items": { "type": "string" } },
                    "dry_run": { "type": "boolean" }
                }
            }),
        },
        McpToolDefinitionDto {
            name: "undo_transaction".to_string(),
            description: "Undo a committed transaction by tx_id.".to_string(),
            tier: "C".to_string(),
            required_scope: "admin:maintenance".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["tx_id"],
                "properties": {
                    "tx_id": { "type": "string" }
                }
            }),
        },
    ]
}
