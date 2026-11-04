#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMcpTokenRequest {
    pub name: String,
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
    #[serde(default)]
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMcpTokenResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub token_prefix: String,
    pub token: String,
    pub raw_token: String,
    pub scopes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub created_at: String,
    pub token_info: PersonalAccessTokenDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpTokenListResponse {
    pub items: Vec<PersonalAccessTokenDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpToolDefinitionDto {
    pub name: String,
    pub description: String,
    pub tier: String,
    pub required_scope: String,
    #[serde(rename = "inputSchema", alias = "input_schema")]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct McpJsonRpcRequest {
    #[serde(default = "default_jsonrpc_version")]
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

pub(crate) fn default_jsonrpc_version() -> String {
    "2.0".to_string()
}

#[derive(Debug, Clone)]
pub struct McpCallerContext {
    pub user_id: String,
    pub username: String,
    pub access: CallerAccess,
    pub scopes: Vec<String>,
}

/// Computes the SHA-256 hex digest of a raw Personal Access Token.
pub fn hash_mcp_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.trim().as_bytes());
    hex::encode(hasher.finalize())
}

/// Returns the allowed scopes ceiling for a given `CallerAccess` (`19-ai-family-story-intake-and-mcp.md §4.1`).
pub fn allowed_scopes_for_access(access: CallerAccess) -> Vec<&'static str> {
    match access {
        CallerAccess::AnonymousPublic | CallerAccess::Viewer => vec!["read:tree"],
        CallerAccess::Editor => vec!["read:tree", "write:draft", "write:direct"],
        CallerAccess::Admin => vec![
            "read:tree",
            "write:draft",
            "write:direct",
            "admin:maintenance",
        ],
    }
}

/// Creates a new Personal Access Token (`POST /api/v1/mcp/tokens`).
pub fn create_mcp_token(
    store: &mut GenealogyPrivacyStore,
    user_id: &str,
    access: CallerAccess,
    req: CreateMcpTokenRequest,
) -> Result<CreateMcpTokenResponse, AppError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::ValidationFailed(
            "Token name must not be empty".to_string(),
            Some(HashMap::from([(
                "name".to_string(),
                "required".to_string(),
            )])),
        ));
    }

    let allowed = allowed_scopes_for_access(access);
    let scopes: Vec<String> = if let Some(requested) = req.scopes {
        let mut validated = Vec::new();
        for raw_scope in requested {
            let s = raw_scope.trim().to_string();
            if s.is_empty() {
                continue;
            }
            let normalized = match s.as_str() {
                "write:tree" => "write:direct",
                "write:ai_drafts" => "write:draft",
                other => other,
            };
            if !allowed.contains(&normalized) {
                return Err(AppError::Forbidden(format!(
                    "Requested token scope '{s}' exceeds caller's role ceiling"
                )));
            }
            if !validated.contains(&s) {
                validated.push(s);
            }
        }
        if validated.is_empty() {
            vec!["read:tree".to_string()]
        } else {
            validated
        }
    } else {
        allowed.into_iter().map(ToString::to_string).collect()
    };

    let raw_secret = format!(
        "mcp_pat_{}{}",
        Uuid::new_v4().simple(),
        &Uuid::new_v4().simple().to_string()[..12]
    );
    let token_prefix = raw_secret.chars().take(16).collect::<String>();
    let token_hash = hash_mcp_token(&raw_secret);
    let now = Utc::now();
    let expires_at = req
        .expires_in_days
        .filter(|&d| d > 0)
        .map(|d| (now + Duration::days(i64::from(d))).to_rfc3339());

    let dto = PersonalAccessTokenDto {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        name: name.clone(),
        token_prefix: token_prefix.clone(),
        token_hash,
        scopes: scopes.clone(),
        last_used_at: None,
        expires_at: expires_at.clone(),
        revoked_at: None,
        created_at: now.to_rfc3339(),
    };

    store.personal_access_tokens.push(dto.clone());

    Ok(CreateMcpTokenResponse {
        id: dto.id.clone(),
        user_id: dto.user_id.clone(),
        name,
        token_prefix,
        token: raw_secret.clone(),
        raw_token: raw_secret,
        scopes,
        expires_at,
        created_at: dto.created_at.clone(),
        token_info: dto,
    })
}

/// Lists active (non-revoked) Personal Access Tokens for the caller (`GET /api/v1/mcp/tokens`).
pub fn list_mcp_tokens(store: &GenealogyPrivacyStore, user_id: &str) -> McpTokenListResponse {
    let items = store
        .personal_access_tokens
        .iter()
        .filter(|t| t.user_id == user_id && t.revoked_at.is_none())
        .cloned()
        .collect();
    McpTokenListResponse { items }
}

/// Revokes a Personal Access Token (`DELETE /api/v1/mcp/tokens/{id}`).
pub fn revoke_mcp_token(
    store: &mut GenealogyPrivacyStore,
    token_id: &str,
    caller_user_id: &str,
    caller_access: CallerAccess,
) -> Result<(), AppError> {
    let token = store
        .personal_access_tokens
        .iter_mut()
        .find(|t| t.id == token_id && t.revoked_at.is_none())
        .ok_or(AppError::NotFound)?;

    if token.user_id != caller_user_id && !caller_access.is_admin() {
        return Err(AppError::Forbidden(
            "Cannot revoke another user's Personal Access Token".to_string(),
        ));
    }

    token.revoked_at = Some(Utc::now().to_rfc3339());
    Ok(())
}

/// Returns the complete MCP tool catalog filtered by caller's role and token scopes (`19-ai-family-story-intake-and-mcp.md §4.3`).
pub fn list_authorized_mcp_tools(caller: &McpCallerContext) -> Vec<McpToolDefinitionDto> {
    all_mcp_tools()
        .into_iter()
        .filter(|tool| is_caller_authorized_for_tool(caller, &tool.tier, &tool.required_scope))
        .collect()
}

pub fn is_caller_authorized_for_tool(
    caller: &McpCallerContext,
    tier: &str,
    required_scope: &str,
) -> bool {
    let role_ok = match tier {
        "A" => true,
        "B" => caller.access.can_edit(),
        "C" => caller.access.is_admin(),
        _ => false,
    };
    if !role_ok {
        return false;
    }

    if required_scope == "write:draft" {
        caller.scopes.iter().any(|s| {
            matches!(
                s.as_str(),
                "write:draft" | "write:ai_drafts" | "write:direct" | "write:tree"
            )
        })
    } else if required_scope == "write:direct" {
        caller
            .scopes
            .iter()
            .any(|s| matches!(s.as_str(), "write:direct" | "write:tree"))
    } else {
        caller.scopes.iter().any(|s| s == required_scope)
    }
}
