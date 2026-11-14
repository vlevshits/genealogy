use super::*;
use chrono::DateTime;
use sqlx::{PgPool, Row};
use std::path::Path;

/// Disk-serialization representation of a Personal Access Token that preserves `token_hash`
/// in `mcp-tokens.json` and `live-store.json` without exposing `token_hash` on HTTP DTOs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedMcpTokenRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub token_prefix: String,
    #[serde(default)]
    pub token_hash: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
    pub created_at: String,
}

impl From<&PersonalAccessTokenDto> for PersistedMcpTokenRecord {
    fn from(dto: &PersonalAccessTokenDto) -> Self {
        Self {
            id: dto.id.clone(),
            user_id: dto.user_id.clone(),
            name: dto.name.clone(),
            token_prefix: dto.token_prefix.clone(),
            token_hash: dto.token_hash.clone(),
            scopes: dto.scopes.clone(),
            last_used_at: dto.last_used_at.clone(),
            expires_at: dto.expires_at.clone(),
            revoked_at: dto.revoked_at.clone(),
            created_at: dto.created_at.clone(),
        }
    }
}

impl From<PersistedMcpTokenRecord> for PersonalAccessTokenDto {
    fn from(rec: PersistedMcpTokenRecord) -> Self {
        Self {
            id: rec.id,
            user_id: rec.user_id,
            name: rec.name,
            token_prefix: rec.token_prefix,
            token_hash: rec.token_hash,
            scopes: rec.scopes,
            last_used_at: rec.last_used_at,
            expires_at: rec.expires_at,
            revoked_at: rec.revoked_at,
            created_at: rec.created_at,
        }
    }
}

/// Custom serde serializer for `GenealogyPrivacyStore.personal_access_tokens` so `live-store.json`
/// preserves `token_hash` while HTTP API responses using `PersonalAccessTokenDto` keep hiding it.
pub fn serialize_persisted_tokens<S>(
    tokens: &[PersonalAccessTokenDto],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let records: Vec<PersistedMcpTokenRecord> =
        tokens.iter().map(PersistedMcpTokenRecord::from).collect();
    records.serialize(serializer)
}

/// Atomically writes `mcp-tokens.json` (including `token_hash`) to `dir`.
pub fn persist_mcp_tokens_to_dir(dir: &Path, tokens: &[PersonalAccessTokenDto]) {
    if let Err(err) = std::fs::create_dir_all(dir) {
        tracing::warn!("Failed to create MCP backup directory: {err}");
        return;
    }
    let records: Vec<PersistedMcpTokenRecord> =
        tokens.iter().map(PersistedMcpTokenRecord::from).collect();
    let bytes = match serde_json::to_vec_pretty(&records) {
        Ok(b) => b,
        Err(err) => {
            tracing::warn!("Failed to serialize mcp-tokens.json: {err}");
            return;
        }
    };
    let tmp_path = dir.join("mcp-tokens.json.tmp");
    let final_path = dir.join("mcp-tokens.json");
    if let Err(err) = std::fs::write(&tmp_path, bytes) {
        tracing::warn!("Failed to write mcp-tokens.json.tmp: {err}");
        return;
    }
    if let Err(err) = std::fs::rename(&tmp_path, &final_path) {
        tracing::warn!("Failed to rename mcp-tokens.json.tmp -> mcp-tokens.json: {err}");
    }
}

/// Loads `mcp-tokens.json` from `dir` if present, preserving `token_hash`.
pub fn load_mcp_tokens_from_dir(dir: &Path) -> Vec<PersonalAccessTokenDto> {
    let tokens_path = dir.join("mcp-tokens.json");
    let Ok(bytes) = std::fs::read(&tokens_path) else {
        return Vec::new();
    };
    serde_json::from_slice::<Vec<PersistedMcpTokenRecord>>(&bytes)
        .map(|recs| recs.into_iter().map(PersonalAccessTokenDto::from).collect())
        .unwrap_or_default()
}

pub fn parse_optional_rfc3339(raw: Option<&str>) -> Option<DateTime<Utc>> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(trimmed)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Evaluates token expiration strictly using parsed RFC-3339 instants rather than string comparison.
pub fn is_mcp_token_expired(expires_at: Option<&str>, now: DateTime<Utc>) -> bool {
    let Some(raw) = expires_at else {
        return false;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return false;
    }
    match DateTime::parse_from_rfc3339(trimmed) {
        Ok(exp_dt) => exp_dt.with_timezone(&Utc) <= now,
        Err(err) => {
            tracing::warn!("Invalid expires_at timestamp on MCP PAT: {err}");
            true
        }
    }
}

/// Merges incoming MCP token records (from DB, `mcp-tokens.json`, or seed env) into the in-memory store,
/// repairing missing `token_hash` values, preserving revocations, and keeping the newest `last_used_at`.
pub fn merge_mcp_tokens_into_store(
    store: &mut GenealogyPrivacyStore,
    incoming: Vec<PersonalAccessTokenDto>,
) {
    for tok in incoming {
        let existing_idx = store.personal_access_tokens.iter().position(|ex| {
            (!tok.token_hash.is_empty() && ex.token_hash == tok.token_hash)
                || (!tok.id.is_empty() && ex.id == tok.id)
        });
        if let Some(idx) = existing_idx {
            let ex = &mut store.personal_access_tokens[idx];
            if ex.token_hash.is_empty() && !tok.token_hash.is_empty() {
                ex.token_hash = tok.token_hash;
            }
            if ex.revoked_at.is_none() && tok.revoked_at.is_some() {
                ex.revoked_at = tok.revoked_at;
            }
            match (
                parse_optional_rfc3339(ex.last_used_at.as_deref()),
                parse_optional_rfc3339(tok.last_used_at.as_deref()),
            ) {
                (None, Some(_)) => ex.last_used_at = tok.last_used_at,
                (Some(cur), Some(inc)) if inc > cur => ex.last_used_at = tok.last_used_at,
                _ => {}
            }
            if (ex.user_id.trim().is_empty() || ex.user_id == "seed-admin")
                && !tok.user_id.trim().is_empty()
                && tok.user_id != "seed-admin"
            {
                ex.user_id = tok.user_id;
            }
            if ex.scopes.is_empty() && !tok.scopes.is_empty() {
                ex.scopes = tok.scopes;
            }
            if ex.expires_at.is_none() && tok.expires_at.is_some() {
                ex.expires_at = tok.expires_at;
            }
        } else {
            store.personal_access_tokens.push(tok);
        }
    }
}

pub fn deterministic_seed_pat_id(token_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"seed-mcp-pat:");
    hasher.update(token_hash.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes).to_string()
}

fn read_seed_pat_raw(backup_dir: &Path) -> Option<String> {
    std::env::var("SEED_MCP_PAT")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::fs::read_to_string(backup_dir.join("seed-pat.txt"))
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
}

/// Builds a `PersonalAccessTokenDto` from `SEED_MCP_PAT` env var or `{backup_dir}/seed-pat.txt` if present.
pub fn load_seed_mcp_token(
    backup_dir: &Path,
    admin_user_id: Option<&str>,
) -> Option<PersonalAccessTokenDto> {
    let raw_spec = read_seed_pat_raw(backup_dir)?;
    let (uid, raw_token) = if let Some((u, t)) = raw_spec.split_once(':') {
        (u.trim().to_string(), t.trim().to_string())
    } else {
        (
            admin_user_id
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("seed-admin")
                .to_string(),
            raw_spec,
        )
    };
    if raw_token.is_empty() {
        return None;
    }
    let token_hash = hash_mcp_token(&raw_token);
    Some(PersonalAccessTokenDto {
        id: deterministic_seed_pat_id(&token_hash),
        user_id: uid,
        name: "Production MCP Agent".to_string(),
        token_prefix: raw_token.chars().take(16).collect(),
        token_hash: token_hash.clone(),
        scopes: vec![
            "read:tree".to_string(),
            "write:draft".to_string(),
            "write:direct".to_string(),
            "admin:maintenance".to_string(),
        ],
        last_used_at: None,
        expires_at: None,
        revoked_at: None,
        created_at: Utc::now().to_rfc3339(),
    })
}

/// Identifies whether a PAT record represents a seeded/legacy admin agent token rather than a regular user token.
pub fn is_seed_or_legacy_admin_token(backup_dir: &Path, pat: &PersonalAccessTokenDto) -> bool {
    if pat.user_id == "seed-admin" || pat.id == deterministic_seed_pat_id(&pat.token_hash) {
        return true;
    }
    if let Some(seed_pat) = load_seed_mcp_token(backup_dir, None) {
        if seed_pat.token_hash == pat.token_hash {
            return true;
        }
    }
    pat.name == "Production MCP Agent" && pat.scopes.iter().any(|s| s == "admin:maintenance")
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn row_to_pat_dto(row: &sqlx::postgres::PgRow) -> Result<PersonalAccessTokenDto, sqlx::Error> {
    let id: Uuid = row.try_get("id")?;
    let user_id: Uuid = row.try_get("user_id")?;
    let name: String = row.try_get("name")?;
    let token_prefix: String = row.try_get("token_prefix")?;
    let token_hash: String = row.try_get("token_hash")?;
    let scopes: Vec<String> = row.try_get("scopes")?;
    let last_used_at: Option<DateTime<Utc>> = row.try_get("last_used_at")?;
    let expires_at: Option<DateTime<Utc>> = row.try_get("expires_at")?;
    let revoked_at: Option<DateTime<Utc>> = row.try_get("revoked_at")?;
    let created_at: DateTime<Utc> = row.try_get("created_at")?;

    Ok(PersonalAccessTokenDto {
        id: id.to_string(),
        user_id: user_id.to_string(),
        name,
        token_prefix,
        token_hash,
        scopes,
        last_used_at: last_used_at.map(|d| d.to_rfc3339()),
        expires_at: expires_at.map(|d| d.to_rfc3339()),
        revoked_at: revoked_at.map(|d| d.to_rfc3339()),
        created_at: created_at.to_rfc3339(),
    })
}

/// Inserts or updates a Personal Access Token row in PostgreSQL `personal_access_tokens`.
pub async fn upsert_mcp_token_in_db(pool: &PgPool, record: &PersonalAccessTokenDto) {
    if record.token_hash.trim().is_empty() {
        return;
    }
    let Ok(token_id) = Uuid::parse_str(record.id.trim()) else {
        tracing::warn!(token_id = %record.id, "Skipping DB PAT upsert: invalid token UUID");
        return;
    };
    let Ok(user_id) = Uuid::parse_str(record.user_id.trim()) else {
        tracing::warn!(token_id = %record.id, "Skipping DB PAT upsert: invalid user UUID");
        return;
    };

    let name = truncate_chars(record.name.trim(), 120);
    let prefix = truncate_chars(record.token_prefix.trim(), 24);
    let last_used_at = parse_optional_rfc3339(record.last_used_at.as_deref());
    let expires_at = parse_optional_rfc3339(record.expires_at.as_deref());
    let revoked_at = parse_optional_rfc3339(record.revoked_at.as_deref());
    let created_at = parse_optional_rfc3339(Some(&record.created_at)).unwrap_or_else(Utc::now);

    let res = sqlx::query(
        "INSERT INTO personal_access_tokens (\
            id, user_id, name, token_prefix, token_hash, scopes, \
            last_used_at, expires_at, revoked_at, created_at\
        ) \
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
        ON CONFLICT (token_hash) DO UPDATE SET \
            name = EXCLUDED.name, \
            token_prefix = EXCLUDED.token_prefix, \
            scopes = EXCLUDED.scopes, \
            last_used_at = COALESCE(\
                GREATEST(personal_access_tokens.last_used_at, EXCLUDED.last_used_at), \
                EXCLUDED.last_used_at, \
                personal_access_tokens.last_used_at\
            ), \
            expires_at = EXCLUDED.expires_at, \
            revoked_at = COALESCE(personal_access_tokens.revoked_at, EXCLUDED.revoked_at)",
    )
    .bind(token_id)
    .bind(user_id)
    .bind(&name)
    .bind(&prefix)
    .bind(&record.token_hash)
    .bind(&record.scopes)
    .bind(last_used_at)
    .bind(expires_at)
    .bind(revoked_at)
    .bind(created_at)
    .execute(pool)
    .await;

    if let Err(err) = res {
        let is_unique_pk = matches!(
            &err,
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505")
        );
        if is_unique_pk {
            let update_res = sqlx::query(
                "UPDATE personal_access_tokens \
                 SET name = $2, token_prefix = $3, token_hash = $4, scopes = $5, \
                     last_used_at = COALESCE(GREATEST(last_used_at, $6), $6, last_used_at), \
                     expires_at = $7, \
                     revoked_at = COALESCE(revoked_at, $8) \
                 WHERE id = $1",
            )
            .bind(token_id)
            .bind(&name)
            .bind(&prefix)
            .bind(&record.token_hash)
            .bind(&record.scopes)
            .bind(last_used_at)
            .bind(expires_at)
            .bind(revoked_at)
            .execute(pool)
            .await;
            if let Err(update_err) = update_res {
                tracing::warn!(token_id = %record.id, "Failed fallback DB PAT update: {update_err}");
            }
        } else {
            tracing::warn!(token_id = %record.id, "Failed to upsert MCP PAT in PostgreSQL: {err}");
        }
    }
}

/// Marks a Personal Access Token as revoked in PostgreSQL.
pub async fn revoke_mcp_token_in_db(pool: &PgPool, token_id: &str, user_id: &str, is_admin: bool) {
    let Ok(token_uuid) = Uuid::parse_str(token_id.trim()) else {
        return;
    };
    let res = if is_admin {
        sqlx::query(
            "UPDATE personal_access_tokens \
             SET revoked_at = COALESCE(revoked_at, NOW()) \
             WHERE id = $1",
        )
        .bind(token_uuid)
        .execute(pool)
        .await
    } else {
        let Ok(user_uuid) = Uuid::parse_str(user_id.trim()) else {
            return;
        };
        sqlx::query(
            "UPDATE personal_access_tokens \
             SET revoked_at = COALESCE(revoked_at, NOW()) \
             WHERE id = $1 AND user_id = $2",
        )
        .bind(token_uuid)
        .bind(user_uuid)
        .execute(pool)
        .await
    };
    if let Err(err) = res {
        tracing::warn!(token_id = %token_id, "Failed to revoke MCP PAT in PostgreSQL: {err}");
    }
}

/// Loads all Personal Access Token records from PostgreSQL ordered newest first.
pub async fn load_mcp_tokens_from_db(pool: &PgPool) -> Vec<PersonalAccessTokenDto> {
    let rows = match sqlx::query(
        "SELECT id, user_id, name, token_prefix, token_hash, scopes, \
                last_used_at, expires_at, revoked_at, created_at \
         FROM personal_access_tokens \
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(err) => {
            tracing::warn!("Failed to load MCP PATs from PostgreSQL: {err}");
            return Vec::new();
        }
    };
    rows.iter()
        .filter_map(|row| row_to_pat_dto(row).ok())
        .collect()
}

/// Looks up a single Personal Access Token record in PostgreSQL by SHA-256 `token_hash`.
pub async fn find_mcp_token_by_hash_in_db(
    pool: &PgPool,
    token_hash: &str,
) -> Option<PersonalAccessTokenDto> {
    let row_opt = sqlx::query(
        "SELECT id, user_id, name, token_prefix, token_hash, scopes, \
                last_used_at, expires_at, revoked_at, created_at \
         FROM personal_access_tokens \
         WHERE token_hash = $1",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .ok()??;
    row_to_pat_dto(&row_opt).ok()
}

/// Best-effort update of `last_used_at` in PostgreSQL without rewriting disk backup files.
pub async fn touch_mcp_token_last_used_in_db(pool: &PgPool, token_hash: &str) {
    if let Err(err) = sqlx::query(
        "UPDATE personal_access_tokens \
         SET last_used_at = NOW() \
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(pool)
    .await
    {
        tracing::debug!("Failed to update MCP PAT last_used_at: {err}");
    }
}

/// Checks PostgreSQL to determine whether a `user_id` belongs to an explicitly deactivated user.
pub async fn is_user_deactivated_in_db(pool: &PgPool, user_id: Uuid) -> bool {
    match sqlx::query("SELECT is_active FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(row)) => !row.try_get::<bool, _>("is_active").unwrap_or(true),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::rbac::CallerAccess;

    #[test]
    fn test_is_mcp_token_expired_parses_rfc3339_accurately() {
        let now = DateTime::parse_from_rfc3339("2026-11-14T18:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        assert!(!is_mcp_token_expired(None, now));
        assert!(!is_mcp_token_expired(Some(""), now));
        assert!(!is_mcp_token_expired(Some("   "), now));

        assert!(!is_mcp_token_expired(Some("2026-11-14T18:00:01Z"), now));
        assert!(!is_mcp_token_expired(
            Some("2026-11-15T01:00:00+00:00"),
            now
        ));
        // 21:00:01+03:00 == 18:00:01Z -> future even across offset formats
        assert!(!is_mcp_token_expired(
            Some("2026-11-14T21:00:01+03:00"),
            now
        ));

        assert!(is_mcp_token_expired(Some("2026-11-14T18:00:00Z"), now));
        assert!(is_mcp_token_expired(Some("2026-11-14T17:59:59+00:00"), now));
        // 20:00:00+03:00 == 17:00:00Z -> expired even though "20" > "18" lexicographically
        assert!(is_mcp_token_expired(Some("2026-11-14T20:00:00+03:00"), now));
        assert!(is_mcp_token_expired(Some("not-a-timestamp"), now));
    }

    #[test]
    fn test_persist_and_reload_mcp_tokens_preserves_hash_while_hiding_in_http_dto() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = GenealogyPrivacyStore::empty();
        let created = create_mcp_token(
            &mut store,
            &Uuid::new_v4().to_string(),
            CallerAccess::Admin,
            CreateMcpTokenRequest {
                name: "Test Persist".to_string(),
                scopes: None,
                expires_in_days: Some(30),
            },
        )
        .unwrap();
        let expected_hash = hash_mcp_token(&created.token);

        // HTTP DTO serialization MUST NOT leak token_hash
        let http_json = serde_json::to_value(&created.token_info).unwrap();
        assert!(http_json.get("token_hash").is_none());

        // Persisting to disk MUST preserve token_hash in both mcp-tokens.json and live-store.json
        store.persist_to_dir(temp.path());

        let reloaded_file_tokens = load_mcp_tokens_from_dir(temp.path());
        assert_eq!(reloaded_file_tokens.len(), 1);
        assert_eq!(reloaded_file_tokens[0].id, created.token_info.id);
        assert_eq!(reloaded_file_tokens[0].token_hash, expected_hash);

        let live_bytes = std::fs::read(temp.path().join("live-store.json")).unwrap();
        let reloaded_store: GenealogyPrivacyStore = serde_json::from_slice(&live_bytes).unwrap();
        assert_eq!(reloaded_store.personal_access_tokens.len(), 1);
        assert_eq!(
            reloaded_store.personal_access_tokens[0].token_hash,
            expected_hash
        );
    }

    #[test]
    fn test_merge_mcp_tokens_repairs_missing_hash_and_preserves_revocation() {
        let mut store = GenealogyPrivacyStore::empty();
        let tok_id = Uuid::new_v4().to_string();
        let user_id = Uuid::new_v4().to_string();
        store.personal_access_tokens.push(PersonalAccessTokenDto {
            id: tok_id.clone(),
            user_id: user_id.clone(),
            name: "Agent".to_string(),
            token_prefix: "mcp_pat_abcd".to_string(),
            token_hash: String::new(),
            scopes: vec!["read:tree".to_string()],
            last_used_at: Some("2026-11-10T10:00:00Z".to_string()),
            expires_at: None,
            revoked_at: None,
            created_at: "2026-11-01T00:00:00Z".to_string(),
        });

        let incoming = vec![PersonalAccessTokenDto {
            id: tok_id.clone(),
            user_id,
            name: "Agent".to_string(),
            token_prefix: "mcp_pat_abcd".to_string(),
            token_hash: "deadbeefhash".to_string(),
            scopes: vec!["read:tree".to_string()],
            last_used_at: Some("2026-11-14T12:00:00Z".to_string()),
            expires_at: None,
            revoked_at: Some("2026-11-14T13:00:00Z".to_string()),
            created_at: "2026-11-01T00:00:00Z".to_string(),
        }];

        merge_mcp_tokens_into_store(&mut store, incoming);
        assert_eq!(store.personal_access_tokens.len(), 1);
        assert_eq!(store.personal_access_tokens[0].token_hash, "deadbeefhash");
        assert_eq!(
            store.personal_access_tokens[0].revoked_at.as_deref(),
            Some("2026-11-14T13:00:00Z")
        );
        assert_eq!(
            store.personal_access_tokens[0].last_used_at.as_deref(),
            Some("2026-11-14T12:00:00Z")
        );
    }
}
