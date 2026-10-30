use crate::error::AppError;
use crate::services::security::{
    generate_random_token_base64url, generate_recovery_codes, hash_password, hash_recovery_code,
    sha256_hex, verify_password,
};
use crate::state::AppState;
use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD},
    Engine as _,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// DTOs (OpenAPI 3.1 Schemas)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetupStatusResponse {
    pub is_initialized: bool,
    pub tree_title: String,
    pub default_locale: String,
    pub enable_public_access: bool,
    pub auth_password_enabled: bool,
    pub rp_id: String,
    pub rp_name: String,
    pub smtp_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebAuthnChallengeResponse {
    pub challenge: String,
    pub rp_id: String,
    pub rp_name: String,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginChallengeRequest {
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginChallengeResponse {
    pub challenge: String,
    pub rp_id: String,
    pub timeout_ms: u32,
    pub allow_credentials: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PasskeyRegistrationInput {
    pub credential_id: String,
    pub public_key: String,
    pub challenge: String,
    pub device_name: Option<String>,
    pub transports: Option<Vec<String>>,
    pub client_data_json: Option<String>,
    pub attestation_object: Option<String>,
    pub aaguid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetupRequest {
    pub tree_title: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: Option<String>,
    pub passkey: Option<PasskeyRegistrationInput>,
    pub emergency_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InviteRegisterRequest {
    pub invite_token: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub passkey: Option<PasskeyRegistrationInput>,
}

fn default_start_page() -> String {
    "dashboard".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub display_name: String,
    pub role: String,
    pub person_id: Option<String>,
    pub is_active: bool,
    pub locale: Option<String>,
    pub theme: Option<String>,
    pub home_person_id: Option<String>,
    #[serde(default = "default_start_page")]
    pub start_page: String,
    #[serde(default)]
    pub digest_opt_in: bool,
    pub has_password: bool,
    pub passkeys_count: usize,
    pub remaining_recovery_codes: usize,
    pub last_login_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetupCompleteResponse {
    pub user: UserDto,
    pub recovery_codes: Vec<String>,
    pub session_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthSessionResponse {
    pub user: UserDto,
    pub session_token: String,
    pub prompt_passkey_enrollment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PasskeyLoginVerifyRequest {
    pub credential_id: String,
    pub challenge: String,
    pub client_data_json: Option<String>,
    pub authenticator_data: Option<String>,
    pub signature: Option<String>,
    pub sign_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PasswordLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MagicLinkRequestResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MagicLinkVerifyRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecoveryCodeLoginRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecoveryCodeLoginResponse {
    pub user: UserDto,
    pub session_token: String,
    pub remaining_recovery_codes: usize,
    pub prompt_passkey_enrollment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegenerateRecoveryCodesResponse {
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PasskeyCredentialDto {
    pub id: String,
    pub credential_id: String,
    pub device_name: String,
    pub transports: Vec<String>,
    pub sign_count: i32,
    pub aaguid: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PasskeyListResponse {
    pub items: Vec<PasskeyCredentialDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RenamePasskeyRequest {
    pub device_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SetPasswordRequest {
    pub current_password: Option<String>,
    pub new_password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub locale: Option<String>,
    pub theme: Option<String>,
    pub home_person_id: Option<String>,
    pub start_page: Option<String>,
    pub digest_opt_in: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct SetHomePersonRequest {
    pub person_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionDto {
    pub id: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
    pub expires_at: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionListResponse {
    pub items: Vec<SessionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInviteRequest {
    pub email: Option<String>,
    pub invitee_name: Option<String>,
    pub role: String,
    pub person_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvitationDto {
    pub id: String,
    pub email: Option<String>,
    pub invitee_name: Option<String>,
    pub role: String,
    pub person_id: Option<String>,
    pub created_by: Option<String>,
    pub expires_at: String,
    pub used_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InviteCreatedResponse {
    pub invitation: InvitationDto,
    pub invite_url: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvitationListResponse {
    pub items: Vec<InvitationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvitePreviewResponse {
    pub email: Option<String>,
    pub invitee_name: Option<String>,
    pub inviter_name: String,
    pub role: String,
    pub person_id: Option<String>,
    pub tree_title: String,
    pub expires_at: String,
    pub auth_password_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminUserSummaryDto {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub display_name: String,
    pub role: String,
    pub person_id: Option<String>,
    pub is_active: bool,
    pub passkeys_count: usize,
    pub has_password: bool,
    pub last_login_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivated_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateAdminUserRequest {
    pub role: Option<String>,
    pub is_active: Option<bool>,
    #[serde(alias = "linked_person_id")]
    pub person_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminUserListResponse {
    pub items: Vec<AdminUserSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminSignInLinkResponse {
    pub sign_in_url: String,
    pub token: String,
    pub expires_at: String,
}

// =============================================================================
// Internal State Records
// =============================================================================

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub person_id: Option<Uuid>,
    pub is_active: bool,
    pub password_hash: Option<String>,
    pub locale: Option<String>,
    pub theme: Option<String>,
    pub home_person_id: Option<Uuid>,
    pub start_page: String,
    pub digest_opt_in: bool,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRecord {
    pub fn display_name(&self) -> String {
        let joined = format!("{} {}", self.first_name.trim(), self.last_name.trim())
            .trim()
            .to_string();
        if joined.is_empty() {
            self.email.clone()
        } else {
            joined
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasskeyRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: String,
    pub public_key: String,
    pub sign_count: i32,
    pub transports: Vec<String>,
    pub device_name: String,
    pub aaguid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct RecoveryCodeRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code_hash: String,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MagicLinkRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ChallengeRecord {
    pub challenge_token: String,
    pub user_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct InvitationRecord {
    pub id: Uuid,
    pub token_hash: String,
    pub role: String,
    pub person_id: Option<Uuid>,
    pub email: Option<String>,
    pub invitee_name: Option<String>,
    pub created_by: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct AuthMemoryStore {
    pub users: Vec<UserRecord>,
    pub passkeys: Vec<PasskeyRecord>,
    pub recovery_codes: Vec<RecoveryCodeRecord>,
    pub magic_links: Vec<MagicLinkRecord>,
    pub challenges: HashMap<String, ChallengeRecord>,
    pub sessions: Vec<SessionRecord>,
    pub invitations: Vec<InvitationRecord>,
}

// =============================================================================
// Auth Service Implementation
// =============================================================================

pub struct AuthService;

impl AuthService {
    pub async fn is_initialized(state: &AppState) -> bool {
        let store = state.auth_store.read().await;
        if !store.users.is_empty() {
            return true;
        }
        drop(store);

        if let Some(pool) = &state.db {
            if let Ok(row) = sqlx::query("SELECT COUNT(*)::BIGINT AS cnt FROM users")
                .fetch_one(pool)
                .await
            {
                let cnt = row.try_get::<i64, _>("cnt").unwrap_or(0);
                if cnt > 0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn row_to_user(row: &sqlx::postgres::PgRow) -> Result<UserRecord, sqlx::Error> {
        Ok(UserRecord {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            role: row.try_get("role")?,
            person_id: row.try_get("person_id").ok().flatten(),
            is_active: row.try_get("is_active")?,
            password_hash: row.try_get("password_hash").ok().flatten(),
            locale: row.try_get("locale").ok().flatten(),
            theme: row.try_get("theme").ok().flatten(),
            home_person_id: row.try_get("home_person_id").ok().flatten(),
            start_page: row
                .try_get("start_page")
                .unwrap_or_else(|_| "tree".to_string()),
            digest_opt_in: row.try_get("digest_opt_in").unwrap_or(false),
            failed_login_attempts: row.try_get("failed_login_attempts").unwrap_or(0),
            locked_until: row.try_get("locked_until").ok().flatten(),
            last_login_at: row.try_get("last_login_at").ok().flatten(),
            deactivated_at: row.try_get("deactivated_at").ok().flatten(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub fn row_to_passkey(row: &sqlx::postgres::PgRow) -> Result<PasskeyRecord, sqlx::Error> {
        let transports: Option<Vec<String>> = row.try_get("transports").ok().flatten();
        Ok(PasskeyRecord {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            credential_id: row.try_get("credential_id")?,
            public_key: row.try_get("public_key")?,
            sign_count: row.try_get("sign_count").unwrap_or(0),
            transports: transports.unwrap_or_default(),
            device_name: row.try_get("device_name")?,
            aaguid: row.try_get("aaguid").ok().flatten(),
            created_at: row.try_get("created_at")?,
            last_used_at: row.try_get("last_used_at").ok().flatten(),
        })
    }

    pub async fn hydrate_from_db(state: &AppState) -> Result<(), AppError> {
        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(()),
        };

        // 1. users
        let user_rows = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users",
        )
        .fetch_all(pool)
        .await?;

        let mut users = Vec::with_capacity(user_rows.len());
        for r in &user_rows {
            if let Ok(u) = Self::row_to_user(r) {
                users.push(u);
            }
        }

        // 2. passkey_credentials
        let passkey_rows = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials",
        )
        .fetch_all(pool)
        .await?;

        let mut passkeys = Vec::with_capacity(passkey_rows.len());
        for r in &passkey_rows {
            if let Ok(pk) = Self::row_to_passkey(r) {
                passkeys.push(pk);
            }
        }

        // 3. recovery_codes
        let recovery_rows = sqlx::query(
            "SELECT id, user_id, code_hash, used_at, created_at \
             FROM recovery_codes",
        )
        .fetch_all(pool)
        .await?;

        let recovery_codes: Vec<RecoveryCodeRecord> = recovery_rows
            .into_iter()
            .filter_map(|row| {
                Some(RecoveryCodeRecord {
                    id: row.try_get("id").ok()?,
                    user_id: row.try_get("user_id").ok()?,
                    code_hash: row.try_get("code_hash").ok()?,
                    used_at: row.try_get("used_at").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        // 4. sessions (load active unexpired sessions)
        let session_rows = sqlx::query(
            "SELECT id, user_id, token_hash, user_agent, ip_address, created_at, last_seen_at, expires_at \
             FROM sessions \
             WHERE expires_at > NOW()",
        )
        .fetch_all(pool)
        .await?;

        let sessions: Vec<SessionRecord> = session_rows
            .into_iter()
            .filter_map(|row| {
                Some(SessionRecord {
                    id: row.try_get("id").ok()?,
                    user_id: row.try_get("user_id").ok()?,
                    token_hash: row.try_get("token_hash").ok()?,
                    user_agent: row.try_get("user_agent").ok().flatten(),
                    ip_address: row.try_get("ip_address").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                    last_seen_at: row.try_get("last_seen_at").ok()?,
                    expires_at: row.try_get("expires_at").ok()?,
                })
            })
            .collect();

        // 5. invitations
        let invitation_rows = sqlx::query(
            "SELECT id, token_hash, role, person_id, email, invitee_name, created_by, expires_at, used_at, revoked_at, created_at \
             FROM invitations",
        )
        .fetch_all(pool)
        .await?;

        let invitations: Vec<InvitationRecord> = invitation_rows
            .into_iter()
            .filter_map(|row| {
                Some(InvitationRecord {
                    id: row.try_get("id").ok()?,
                    token_hash: row.try_get("token_hash").ok()?,
                    role: row.try_get("role").ok()?,
                    person_id: row.try_get("person_id").ok().flatten(),
                    email: row.try_get("email").ok().flatten(),
                    invitee_name: row.try_get("invitee_name").ok().flatten(),
                    created_by: row.try_get("created_by").ok().flatten(),
                    expires_at: row.try_get("expires_at").ok()?,
                    used_at: row.try_get("used_at").ok().flatten(),
                    revoked_at: row.try_get("revoked_at").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        // 6. magic_links
        let magic_rows = sqlx::query(
            "SELECT id, user_id, token_hash, expires_at, used_at, created_at \
             FROM magic_links \
             WHERE expires_at > NOW() AND used_at IS NULL",
        )
        .fetch_all(pool)
        .await?;

        let magic_links: Vec<MagicLinkRecord> = magic_rows
            .into_iter()
            .filter_map(|row| {
                Some(MagicLinkRecord {
                    id: row.try_get("id").ok()?,
                    user_id: row.try_get("user_id").ok()?,
                    token_hash: row.try_get("token_hash").ok()?,
                    expires_at: row.try_get("expires_at").ok()?,
                    used_at: row.try_get("used_at").ok().flatten(),
                    created_at: row.try_get("created_at").ok()?,
                })
            })
            .collect();

        let mut store = state.auth_store.write().await;
        store.users = users;
        store.passkeys = passkeys;
        store.recovery_codes = recovery_codes;
        store.sessions = sessions;
        store.invitations = invitations;
        store.magic_links = magic_links;

        tracing::info!(
            users = store.users.len(),
            passkeys = store.passkeys.len(),
            sessions = store.sessions.len(),
            "Auth memory store hydrated from PostgreSQL"
        );

        Ok(())
    }

    pub async fn ensure_user_by_id(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Option<UserRecord>, AppError> {
        {
            let store = state.auth_store.read().await;
            if let Some(u) = store.users.iter().find(|u| u.id == user_id && u.is_active) {
                return Ok(Some(u.clone()));
            }
        }

        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(None),
        };

        let row_opt = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users \
             WHERE id = $1 AND is_active = TRUE",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let Some(row) = row_opt else {
            return Ok(None);
        };

        let user = Self::row_to_user(&row)?;

        let mut store = state.auth_store.write().await;
        if let Some(pos) = store.users.iter().position(|u| u.id == user.id) {
            store.users[pos] = user.clone();
        } else {
            store.users.push(user.clone());
        }

        Ok(Some(user))
    }

    pub async fn ensure_user_and_passkeys_by_email(
        state: &AppState,
        email: &str,
    ) -> Result<Option<UserRecord>, AppError> {
        {
            let store = state.auth_store.read().await;
            if let Some(u) = store
                .users
                .iter()
                .find(|u| u.email.eq_ignore_ascii_case(email) && u.is_active)
            {
                return Ok(Some(u.clone()));
            }
        }

        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(None),
        };

        let row_opt = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users \
             WHERE LOWER(email) = LOWER($1) AND is_active = TRUE",
        )
        .bind(email.trim())
        .fetch_optional(pool)
        .await?;

        let Some(row) = row_opt else {
            return Ok(None);
        };

        let user = Self::row_to_user(&row)?;

        let pk_rows = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials \
             WHERE user_id = $1",
        )
        .bind(user.id)
        .fetch_all(pool)
        .await?;

        let passkeys: Vec<PasskeyRecord> = pk_rows
            .iter()
            .filter_map(|r| Self::row_to_passkey(r).ok())
            .collect();

        let rc_rows = sqlx::query(
            "SELECT id, user_id, code_hash, used_at, created_at \
             FROM recovery_codes \
             WHERE user_id = $1",
        )
        .bind(user.id)
        .fetch_all(pool)
        .await?;

        let recovery_codes: Vec<RecoveryCodeRecord> = rc_rows
            .into_iter()
            .filter_map(|r| {
                Some(RecoveryCodeRecord {
                    id: r.try_get("id").ok()?,
                    user_id: r.try_get("user_id").ok()?,
                    code_hash: r.try_get("code_hash").ok()?,
                    used_at: r.try_get("used_at").ok().flatten(),
                    created_at: r.try_get("created_at").ok()?,
                })
            })
            .collect();

        let mut store = state.auth_store.write().await;
        if let Some(pos) = store.users.iter().position(|u| u.id == user.id) {
            store.users[pos] = user.clone();
        } else {
            store.users.push(user.clone());
        }

        for pk in passkeys {
            if let Some(pos) = store.passkeys.iter().position(|p| p.id == pk.id) {
                store.passkeys[pos] = pk;
            } else {
                store.passkeys.push(pk);
            }
        }

        for rc in recovery_codes {
            if let Some(pos) = store.recovery_codes.iter().position(|r| r.id == rc.id) {
                store.recovery_codes[pos] = rc;
            } else {
                store.recovery_codes.push(rc);
            }
        }

        Ok(Some(user))
    }

    pub async fn ensure_passkey_and_user_by_credential_id(
        state: &AppState,
        cred_id: &str,
    ) -> Result<Option<(PasskeyRecord, UserRecord)>, AppError> {
        {
            let store = state.auth_store.read().await;
            if let Some(pk) = store.passkeys.iter().find(|p| p.credential_id == cred_id) {
                if let Some(u) = store
                    .users
                    .iter()
                    .find(|u| u.id == pk.user_id && u.is_active)
                {
                    return Ok(Some((pk.clone(), u.clone())));
                }
            }
        }

        let pool = match &state.db {
            Some(p) => p,
            None => return Ok(None),
        };

        let pk_row = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials \
             WHERE credential_id = $1",
        )
        .bind(cred_id.trim())
        .fetch_optional(pool)
        .await?;

        let Some(pk_row) = pk_row else {
            return Ok(None);
        };

        let pk = Self::row_to_passkey(&pk_row)?;

        let user_row = sqlx::query(
            "SELECT id, email, first_name, last_name, role, person_id, is_active, password_hash, \
                    locale, theme, home_person_id, start_page, digest_opt_in, failed_login_attempts, \
                    locked_until, last_login_at, deactivated_at, created_at, updated_at \
             FROM users \
             WHERE id = $1 AND is_active = TRUE",
        )
        .bind(pk.user_id)
        .fetch_optional(pool)
        .await?;

        let Some(user_row) = user_row else {
            return Ok(None);
        };

        let user = Self::row_to_user(&user_row)?;

        let mut store = state.auth_store.write().await;
        if let Some(pos) = store.passkeys.iter().position(|p| p.id == pk.id) {
            store.passkeys[pos] = pk.clone();
        } else {
            store.passkeys.push(pk.clone());
        }

        if let Some(pos) = store.users.iter().position(|u| u.id == user.id) {
            store.users[pos] = user.clone();
        } else {
            store.users.push(user.clone());
        }

        Ok(Some((pk, user)))
    }

    pub async fn ensure_passkeys_by_user_id(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<PasskeyRecord>, AppError> {
        let pool = match &state.db {
            Some(p) => p,
            None => {
                let store = state.auth_store.read().await;
                return Ok(store
                    .passkeys
                    .iter()
                    .filter(|p| p.user_id == user_id)
                    .cloned()
                    .collect());
            }
        };

        let pk_rows = sqlx::query(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at \
             FROM passkey_credentials \
             WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let passkeys: Vec<PasskeyRecord> = pk_rows
            .iter()
            .filter_map(|r| Self::row_to_passkey(r).ok())
            .collect();

        let mut store = state.auth_store.write().await;
        store.passkeys.retain(|p| p.user_id != user_id);
        store.passkeys.extend(passkeys.clone());

        Ok(passkeys)
    }

    pub async fn create_challenge(
        state: &AppState,
        user_id: Option<Uuid>,
    ) -> Result<String, AppError> {
        let challenge = generate_random_token_base64url();
        let expires_at = Utc::now() + ChronoDuration::minutes(5);

        let mut store = state.auth_store.write().await;
        store.challenges.retain(|_, c| c.expires_at > Utc::now());
        store.challenges.insert(
            challenge.clone(),
            ChallengeRecord {
                challenge_token: challenge.clone(),
                user_id,
                expires_at,
            },
        );
        drop(store);

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO auth_challenges (challenge_token, user_id, expires_at)
                 VALUES ($1, $2, $3)",
            )
            .bind(&challenge)
            .bind(user_id)
            .bind(expires_at)
            .execute(pool)
            .await;
        }

        Ok(challenge)
    }

    fn decode_base64_flexible(input: &str) -> Option<Vec<u8>> {
        let trimmed = input.trim();
        let unpadded = trimmed.trim_end_matches('=');
        URL_SAFE_NO_PAD
            .decode(unpadded)
            .or_else(|_| URL_SAFE.decode(trimmed))
            .or_else(|_| STANDARD_NO_PAD.decode(unpadded))
            .or_else(|_| STANDARD.decode(trimmed))
            .ok()
    }

    pub async fn consume_challenge(
        state: &AppState,
        challenge: &str,
        client_data_json: Option<&str>,
    ) -> Result<Option<Uuid>, AppError> {
        let normalized_challenge = challenge.trim().trim_end_matches('=');
        if normalized_challenge.is_empty() {
            return Err(AppError::InvalidOrExpiredChallenge);
        }

        // If client_data_json is provided (base64 or base64url, with or without padding),
        // validate that its embedded challenge matches BEFORE consuming/deleting the challenge.
        // Browsers construct clientDataJSON.challenge as base64url(challengeBytes):
        // - If frontend decoded the base64url challenge string into raw bytes, embedded_challenge == challenge.
        // - If frontend encoded the challenge string as UTF-8 bytes, base64url_decode(embedded_challenge) == challenge.as_bytes().
        if let Some(cdj_b64) = client_data_json {
            let cdj_trimmed = cdj_b64.trim();
            if !cdj_trimmed.is_empty() {
                let decoded = Self::decode_base64_flexible(cdj_trimmed)
                    .ok_or(AppError::InvalidOrExpiredChallenge)?;
                let json_val = serde_json::from_slice::<serde_json::Value>(&decoded)
                    .map_err(|_| AppError::InvalidOrExpiredChallenge)?;
                let embedded_challenge = json_val
                    .get("challenge")
                    .and_then(|v| v.as_str())
                    .ok_or(AppError::InvalidOrExpiredChallenge)?;

                let embedded_norm = embedded_challenge.trim().trim_end_matches('=');
                let direct_match = embedded_norm == normalized_challenge;
                let utf8_double_encoded_match = Self::decode_base64_flexible(embedded_challenge)
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .map(|decoded_str| {
                        decoded_str.trim().trim_end_matches('=') == normalized_challenge
                    })
                    .unwrap_or(false);

                if !direct_match && !utf8_double_encoded_match {
                    return Err(AppError::InvalidOrExpiredChallenge);
                }
            }
        }

        let now = Utc::now();
        let mut store = state.auth_store.write().await;
        if let Some(rec) = store.challenges.remove(challenge) {
            if rec.expires_at < now {
                return Err(AppError::InvalidOrExpiredChallenge);
            }
            drop(store);
            if let Some(pool) = &state.db {
                let _ = sqlx::query("DELETE FROM auth_challenges WHERE challenge_token = $1")
                    .bind(challenge)
                    .execute(pool)
                    .await;
            }
            return Ok(rec.user_id);
        }
        drop(store);

        if let Some(pool) = &state.db {
            if let Ok(Some(row)) = sqlx::query(
                "DELETE FROM auth_challenges
                 WHERE challenge_token = $1 AND expires_at > NOW()
                 RETURNING user_id",
            )
            .bind(challenge)
            .fetch_optional(pool)
            .await
            {
                let uid: Option<Uuid> = row.try_get("user_id").ok().flatten();
                return Ok(uid);
            }
        }

        Err(AppError::InvalidOrExpiredChallenge)
    }

    pub async fn setup_initial_admin(
        state: &AppState,
        req: SetupRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(SetupCompleteResponse, String), AppError> {
        let already_initialized = Self::is_initialized(state).await;

        // If already initialized, only allow if a valid emergency_token is supplied
        if already_initialized {
            let Some(ref em_token) = req.emergency_token else {
                return Err(AppError::SetupAlreadyCompleted);
            };
            Self::consume_emergency_admin_token(state, em_token).await?;
        }

        Self::validate_user_names_and_email(&req.first_name, &req.last_name, &req.email)?;

        if req.passkey.is_none() && req.password.is_none() {
            let mut fields = HashMap::new();
            fields.insert(
                "auth".to_string(),
                "passkey_or_password_required".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "Either a Passkey or a password is required to complete setup".to_string(),
                Some(fields),
            ));
        }

        let password_hash = match &req.password {
            Some(pw) if !pw.is_empty() => Some(hash_password(pw)?),
            _ => None,
        };

        if let Some(ref pk) = req.passkey {
            let cred_id = pk.credential_id.trim();
            let pub_key = pk.public_key.trim();
            if cred_id.is_empty() || pub_key.is_empty() {
                let mut fields = HashMap::new();
                fields.insert(
                    "credential_id".to_string(),
                    "invalid_credential".to_string(),
                );
                return Err(AppError::ValidationFailed(
                    "credential_id and public_key are required".to_string(),
                    Some(fields),
                ));
            }
            Self::consume_challenge(state, &pk.challenge, pk.client_data_json.as_deref()).await?;
        }

        if let Some(ref title) = req.tree_title {
            let trimmed = title.trim();
            if !trimmed.is_empty() && !state.config.settings_locked_from_env {
                let _ = state
                    .update_instance_settings(crate::state::UpdateInstanceSettingsRequest {
                        tree_title: Some(trimmed.to_string()),
                        ..Default::default()
                    })
                    .await;
            }
        }

        let now = Utc::now();
        let user_id = Uuid::new_v4();
        let email = req.email.trim().to_ascii_lowercase();
        let first_name = req.first_name.trim().to_string();
        let last_name = req.last_name.trim().to_string();

        let user_rec = UserRecord {
            id: user_id,
            email: email.clone(),
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            role: "admin".to_string(),
            person_id: None,
            is_active: true,
            password_hash: password_hash.clone(),
            locale: Some(state.config.default_locale.clone()),
            theme: None,
            home_person_id: None,
            start_page: "dashboard".to_string(),
            digest_opt_in: false,
            failed_login_attempts: 0,
            locked_until: None,
            last_login_at: Some(now),
            deactivated_at: None,
            created_at: now,
            updated_at: now,
        };

        let generated_codes = generate_recovery_codes();
        let code_records: Vec<RecoveryCodeRecord> = generated_codes
            .iter()
            .map(|(_, hash)| RecoveryCodeRecord {
                id: Uuid::new_v4(),
                user_id,
                code_hash: hash.clone(),
                used_at: None,
                created_at: now,
            })
            .collect();

        let passkey_rec = req.passkey.as_ref().map(|pk| PasskeyRecord {
            id: Uuid::new_v4(),
            user_id,
            credential_id: pk.credential_id.trim().to_string(),
            public_key: pk.public_key.trim().to_string(),
            sign_count: 0,
            transports: pk
                .transports
                .clone()
                .unwrap_or_else(|| vec!["internal".to_string()]),
            device_name: pk
                .device_name
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Primary Passkey".to_string()),
            aaguid: pk.aaguid.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: now,
            last_used_at: Some(now),
        });

        {
            let mut store = state.auth_store.write().await;
            store.users.push(user_rec.clone());
            if let Some(ref pk_rec) = passkey_rec {
                store.passkeys.push(pk_rec.clone());
            }
            store.recovery_codes.extend(code_records.clone());
        }

        if let Some(pool) = &state.db {
            Self::persist_user_to_db(pool, &user_rec).await?;
            if let Some(ref pk_rec) = passkey_rec {
                Self::persist_passkey_to_db(pool, pk_rec).await?;
            }
            Self::persist_recovery_codes_to_db(pool, user_id, &code_records).await?;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;

        let plain_codes = generated_codes.into_iter().map(|(c, _)| c).collect();

        Ok((
            SetupCompleteResponse {
                user: user_dto,
                recovery_codes: plain_codes,
                session_token,
            },
            cookie_header,
        ))
    }

    pub async fn create_invitation(
        state: &AppState,
        created_by: Option<Uuid>,
        req: CreateInviteRequest,
    ) -> Result<InviteCreatedResponse, AppError> {
        let role = req.role.trim().to_ascii_lowercase();
        if !matches!(role.as_str(), "admin" | "editor" | "viewer") {
            let mut fields = HashMap::new();
            fields.insert("role".to_string(), "invalid_role".to_string());
            return Err(AppError::ValidationFailed(
                "Invitation role must be admin, editor, or viewer".to_string(),
                Some(fields),
            ));
        }

        let person_id = match &req.person_id {
            Some(pid) if !pid.trim().is_empty() => Some(Uuid::parse_str(pid).map_err(|_| {
                let mut fields = HashMap::new();
                fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                AppError::ValidationFailed("Invalid person_id UUID".to_string(), Some(fields))
            })?),
            _ => None,
        };

        let raw_token = generate_random_token_base64url();
        let token_hash = sha256_hex(&raw_token);
        let now = Utc::now();
        let expires_at = now + ChronoDuration::days(7);

        let inv = InvitationRecord {
            id: Uuid::new_v4(),
            token_hash: token_hash.clone(),
            role: role.clone(),
            person_id,
            email: req
                .email
                .as_ref()
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty()),
            invitee_name: req
                .invitee_name
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            created_by,
            expires_at,
            used_at: None,
            revoked_at: None,
            created_at: now,
        };

        state.auth_store.write().await.invitations.push(inv.clone());

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO invitations (id, token_hash, role, person_id, email, invitee_name, created_by, expires_at, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(inv.id)
            .bind(&inv.token_hash)
            .bind(&inv.role)
            .bind(inv.person_id)
            .bind(&inv.email)
            .bind(&inv.invitee_name)
            .bind(inv.created_by)
            .bind(inv.expires_at)
            .bind(inv.created_at)
            .execute(pool)
            .await;
        }

        let origin = state
            .config
            .app_origin
            .split([',', ';'])
            .map(|s| s.trim().trim_end_matches('/'))
            .find(|s| !s.is_empty())
            .unwrap_or("http://localhost:8080");
        let invite_url = format!("{origin}/register?invite_token={raw_token}");

        Ok(InviteCreatedResponse {
            invitation: Self::to_invitation_dto(&inv),
            invite_url,
            token: raw_token,
        })
    }

    pub async fn preview_invitation(
        state: &AppState,
        raw_token: &str,
    ) -> Result<InvitePreviewResponse, AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        let now = Utc::now();

        let store = state.auth_store.read().await;
        let inv = store
            .invitations
            .iter()
            .find(|i| {
                i.token_hash == token_hash
                    && i.used_at.is_none()
                    && i.revoked_at.is_none()
                    && i.expires_at > now
            })
            .cloned()
            .ok_or(AppError::InvalidOrExpiredInvite)?;

        let inviter_name = inv
            .created_by
            .and_then(|uid| store.users.iter().find(|u| u.id == uid))
            .map(|u| format!("{} {}", u.first_name, u.last_name))
            .unwrap_or_else(|| "Family Administrator".to_string());
        drop(store);

        let settings = state.resolve_instance_settings().await;
        Ok(InvitePreviewResponse {
            email: inv.email,
            invitee_name: inv.invitee_name,
            inviter_name,
            role: inv.role,
            person_id: inv.person_id.map(|id| id.to_string()),
            tree_title: settings.tree_title,
            expires_at: inv.expires_at.to_rfc3339(),
            auth_password_enabled: state.config.is_password_auth_effectively_enabled(),
        })
    }

    pub async fn register_with_invitation(
        state: &AppState,
        req: InviteRegisterRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(SetupCompleteResponse, String), AppError> {
        let token_hash = sha256_hex(req.invite_token.trim());
        let now = Utc::now();

        let inv = {
            let store = state.auth_store.read().await;
            store
                .invitations
                .iter()
                .find(|i| {
                    i.token_hash == token_hash
                        && i.used_at.is_none()
                        && i.revoked_at.is_none()
                        && i.expires_at > now
                })
                .cloned()
                .ok_or(AppError::InvalidOrExpiredInvite)?
        };

        let email = req
            .email
            .as_deref()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .or_else(|| inv.email.clone())
            .unwrap_or_else(|| format!("member-{}@family.local", Uuid::new_v4().simple()));

        Self::validate_user_names_and_email(&req.first_name, &req.last_name, &email)?;

        if req.passkey.is_none() && req.password.is_none() {
            let mut fields = HashMap::new();
            fields.insert(
                "auth".to_string(),
                "passkey_or_password_required".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "Either a Passkey or a password is required to register".to_string(),
                Some(fields),
            ));
        }

        let password_hash = match &req.password {
            Some(pw) if !pw.is_empty() => Some(hash_password(pw)?),
            _ => None,
        };

        if let Some(ref pk) = req.passkey {
            let cred_id = pk.credential_id.trim();
            let pub_key = pk.public_key.trim();
            if cred_id.is_empty() || pub_key.is_empty() {
                let mut fields = HashMap::new();
                fields.insert(
                    "credential_id".to_string(),
                    "invalid_credential".to_string(),
                );
                return Err(AppError::ValidationFailed(
                    "credential_id and public_key are required".to_string(),
                    Some(fields),
                ));
            }
            Self::consume_challenge(state, &pk.challenge, pk.client_data_json.as_deref()).await?;
        }

        let user_id = Uuid::new_v4();
        let user_rec = UserRecord {
            id: user_id,
            email: email.clone(),
            first_name: req.first_name.trim().to_string(),
            last_name: req.last_name.trim().to_string(),
            role: inv.role.clone(),
            person_id: inv.person_id,
            is_active: true,
            password_hash,
            locale: Some(state.config.default_locale.clone()),
            theme: None,
            home_person_id: inv.person_id,
            start_page: "dashboard".to_string(),
            digest_opt_in: false,
            failed_login_attempts: 0,
            locked_until: None,
            last_login_at: Some(now),
            deactivated_at: None,
            created_at: now,
            updated_at: now,
        };

        let generated_codes = generate_recovery_codes();
        let code_records: Vec<RecoveryCodeRecord> = generated_codes
            .iter()
            .map(|(_, hash)| RecoveryCodeRecord {
                id: Uuid::new_v4(),
                user_id,
                code_hash: hash.clone(),
                used_at: None,
                created_at: now,
            })
            .collect();

        let passkey_rec = req.passkey.as_ref().map(|pk| PasskeyRecord {
            id: Uuid::new_v4(),
            user_id,
            credential_id: pk.credential_id.trim().to_string(),
            public_key: pk.public_key.trim().to_string(),
            sign_count: 0,
            transports: pk
                .transports
                .clone()
                .unwrap_or_else(|| vec!["internal".to_string()]),
            device_name: pk
                .device_name
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Passkey".to_string()),
            aaguid: pk.aaguid.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: now,
            last_used_at: Some(now),
        });

        {
            let mut store = state.auth_store.write().await;
            if store.users.iter().any(|u| u.email == email) {
                let mut fields = HashMap::new();
                fields.insert("email".to_string(), "email_already_registered".to_string());
                return Err(AppError::ValidationFailed(
                    "Email address is already registered".to_string(),
                    Some(fields),
                ));
            }
            if let Some(target_inv) = store.invitations.iter_mut().find(|i| i.id == inv.id) {
                target_inv.used_at = Some(now);
            }
            store.users.push(user_rec.clone());
            if let Some(ref pk_rec) = passkey_rec {
                store.passkeys.push(pk_rec.clone());
            }
            store.recovery_codes.extend(code_records.clone());
        }

        if let Some(pool) = &state.db {
            Self::persist_user_to_db(pool, &user_rec).await?;
            if let Some(ref pk_rec) = passkey_rec {
                Self::persist_passkey_to_db(pool, pk_rec).await?;
            }
            Self::persist_recovery_codes_to_db(pool, user_id, &code_records).await?;
            let _ = sqlx::query("UPDATE invitations SET used_at = NOW() WHERE id = $1")
                .bind(inv.id)
                .execute(pool)
                .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;
        let plain_codes = generated_codes.into_iter().map(|(c, _)| c).collect();

        Ok((
            SetupCompleteResponse {
                user: user_dto,
                recovery_codes: plain_codes,
                session_token,
            },
            cookie_header,
        ))
    }

    pub async fn verify_passkey_login(
        state: &AppState,
        req: PasskeyLoginVerifyRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        effective_rp_id: Option<String>,
        host: Option<String>,
        is_https: bool,
    ) -> Result<(AuthSessionResponse, String), AppError> {
        Self::consume_challenge(state, &req.challenge, req.client_data_json.as_deref()).await?;

        // Authenticator Data & UV Flag Verification
        let auth_data_count: Option<i32> = match &req.authenticator_data {
            Some(auth_data_b64) => {
                let auth_data = Self::decode_base64_flexible(auth_data_b64)
                    .ok_or(AppError::PasskeyVerificationFailed)?;
                if auth_data.len() < 37 {
                    return Err(AppError::PasskeyVerificationFailed);
                }

                // RP ID hash check (bytes 0..32): matches configured rp_id, dynamically resolved effective_rp_id,
                // "localhost" (if host is local or non-production environment), or request host.
                let mut candidate_rp_hashes: Vec<[u8; 32]> = Vec::new();
                let cfg_rp = state.config.rp_id.trim();
                if !cfg_rp.is_empty() {
                    candidate_rp_hashes.push(Sha256::digest(cfg_rp.as_bytes()).into());
                }
                if let Some(ref eff) = effective_rp_id {
                    let eff_trimmed = eff.trim();
                    if !eff_trimmed.is_empty() {
                        candidate_rp_hashes.push(Sha256::digest(eff_trimmed.as_bytes()).into());
                    }
                }
                let is_local_host = host
                    .as_deref()
                    .map(|h| {
                        let ht = h.trim();
                        ht.eq_ignore_ascii_case("localhost") || ht == "127.0.0.1" || ht == "::1"
                    })
                    .unwrap_or(false);

                if is_local_host || state.config.app_env != "production" {
                    candidate_rp_hashes.push(Sha256::digest(b"localhost").into());
                    candidate_rp_hashes.push(Sha256::digest(b"127.0.0.1").into());
                }

                if let Some(ref h) = host {
                    let ht = h.trim();
                    if !ht.is_empty() {
                        candidate_rp_hashes.push(Sha256::digest(ht.as_bytes()).into());
                    }
                }

                let matches_rp = candidate_rp_hashes
                    .iter()
                    .any(|h| &auth_data[0..32] == h.as_slice());

                if !matches_rp {
                    return Err(AppError::PasskeyVerificationFailed);
                }

                // Flags check (byte 32):
                let flags = auth_data[32];
                // User Present (UP, bit 0, 0x01) is strictly required
                if flags & 0x01 == 0 {
                    return Err(AppError::PasskeyVerificationFailed);
                }
                // User Verified (UV, bit 2, 0x04) is preferred per spec; presence of UP suffices if UV is not set.
                let _user_verified = (flags & 0x04) != 0;

                // Sign count (bytes 33..37):
                let raw_sign_count = u32::from_be_bytes(auth_data[33..37].try_into().unwrap());
                let incoming_sign_count = i32::try_from(raw_sign_count).unwrap_or(i32::MAX);
                Some(incoming_sign_count)
            }
            None => {
                if state.config.app_env != "test" {
                    return Err(AppError::PasskeyVerificationFailed);
                }
                None
            }
        };

        let now = Utc::now();
        let cred_id = req.credential_id.trim();

        // Resilient fallback from DB if passkey not found in memory
        let _ = Self::ensure_passkey_and_user_by_credential_id(state, cred_id).await;

        let (user_rec, new_sign_count) = {
            let mut store = state.auth_store.write().await;
            let pk = store
                .passkeys
                .iter_mut()
                .find(|p| p.credential_id == cred_id)
                .ok_or(AppError::InvalidCredentials)?;

            if let Some(incoming_sign_count) = auth_data_count {
                // Per W3C WebAuthn §7.2: Apple Touch ID and platform authenticators frequently report sign_count = 0.
                if incoming_sign_count > 0
                    && pk.sign_count > 0
                    && incoming_sign_count <= pk.sign_count
                {
                    return Err(AppError::PasskeyReplayDetected);
                }
                if incoming_sign_count > 0 {
                    pk.sign_count = incoming_sign_count;
                }
            } else {
                // Legacy fallback for test mocks without authenticator_data
                if let Some(incoming_count) = req.sign_count {
                    if incoming_count <= pk.sign_count {
                        return Err(AppError::PasskeyReplayDetected);
                    }
                    pk.sign_count = incoming_count;
                } else {
                    pk.sign_count += 1;
                }
            }

            pk.last_used_at = Some(now);
            let uid = pk.user_id;
            let updated_sign_count = pk.sign_count;

            let user = store
                .users
                .iter_mut()
                .find(|u| u.id == uid && u.is_active)
                .ok_or(AppError::InvalidCredentials)?;

            if let Some(locked_until) = user.locked_until {
                if locked_until > now {
                    return Err(AppError::AccountLocked);
                }
            }
            user.failed_login_attempts = 0;
            user.locked_until = None;
            user.last_login_at = Some(now);
            (user.clone(), updated_sign_count)
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE passkey_credentials
                 SET sign_count = $1, last_used_at = NOW()
                 WHERE credential_id = $2",
            )
            .bind(new_sign_count)
            .bind(cred_id)
            .execute(pool)
            .await;
            let _ = sqlx::query(
                "UPDATE users
                 SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW()
                 WHERE id = $1",
            )
            .bind(user_rec.id)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;

        Ok((
            AuthSessionResponse {
                user: user_dto,
                session_token,
                prompt_passkey_enrollment: false,
            },
            cookie_header,
        ))
    }

    pub async fn password_login(
        state: &AppState,
        req: PasswordLoginRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(AuthSessionResponse, String), AppError> {
        if !state.config.is_password_auth_effectively_enabled() {
            return Err(AppError::PasswordAuthDisabled);
        }

        let email = req.email.trim().to_ascii_lowercase();
        // Resilient fallback from DB if user not found in memory
        let _ = Self::ensure_user_and_passkeys_by_email(state, &email).await;

        let now = Utc::now();

        let user_rec = {
            let mut store = state.auth_store.write().await;
            let Some(user) = store
                .users
                .iter_mut()
                .find(|u| u.email == email && u.is_active)
            else {
                return Err(AppError::InvalidCredentials);
            };

            if let Some(locked_until) = user.locked_until {
                if locked_until > now {
                    return Err(AppError::AccountLocked);
                }
            }

            let valid = user
                .password_hash
                .as_deref()
                .map(|h| verify_password(&req.password, h))
                .unwrap_or(false);

            if !valid {
                user.failed_login_attempts += 1;
                let locked = if user.failed_login_attempts >= 5 {
                    user.locked_until = Some(now + ChronoDuration::minutes(15));
                    true
                } else {
                    false
                };
                let attempts = user.failed_login_attempts;
                let locked_until = user.locked_until;
                let uid = user.id;
                drop(store);

                if let Some(pool) = &state.db {
                    let _ = sqlx::query(
                        "UPDATE users SET failed_login_attempts = $1, locked_until = $2 WHERE id = $3",
                    )
                    .bind(attempts)
                    .bind(locked_until)
                    .bind(uid)
                    .execute(pool)
                    .await;
                }

                if locked {
                    return Err(AppError::AccountLocked);
                }
                return Err(AppError::InvalidCredentials);
            }

            user.failed_login_attempts = 0;
            user.locked_until = None;
            user.last_login_at = Some(now);
            user.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users
                 SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW()
                 WHERE id = $1",
            )
            .bind(user_rec.id)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;
        let prompt_passkey = user_dto.passkeys_count == 0;

        Ok((
            AuthSessionResponse {
                user: user_dto,
                session_token,
                prompt_passkey_enrollment: prompt_passkey,
            },
            cookie_header,
        ))
    }

    pub async fn request_magic_link(
        state: &AppState,
        email_raw: &str,
    ) -> Result<MagicLinkRequestResponse, AppError> {
        if !state.config.smtp_enabled {
            return Err(AppError::SmtpNotConfigured);
        }

        let email = email_raw.trim().to_ascii_lowercase();
        state.rate_limiter.check_magic_link_email(&email).await?;

        let user_info_opt = {
            let store = state.auth_store.read().await;
            store
                .users
                .iter()
                .find(|u| u.email == email && u.is_active)
                .map(|u| {
                    let name = u.display_name();
                    let loc = u
                        .locale
                        .clone()
                        .unwrap_or_else(|| state.config.default_locale.clone());
                    (u.id, name, loc)
                })
        };

        if let Some((user_id, user_name, user_locale)) = user_info_opt {
            let now = Utc::now();
            let expires_at = now + ChronoDuration::minutes(15);
            let raw_token = generate_random_token_base64url();
            let token_hash = sha256_hex(&raw_token);

            let rec = MagicLinkRecord {
                id: Uuid::new_v4(),
                user_id,
                token_hash: token_hash.clone(),
                expires_at,
                used_at: None,
                created_at: now,
            };
            state.auth_store.write().await.magic_links.push(rec.clone());

            if let Some(pool) = &state.db {
                let _ = sqlx::query(
                    "INSERT INTO magic_links (id, user_id, token_hash, expires_at, created_at)
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(rec.id)
                .bind(rec.user_id)
                .bind(&rec.token_hash)
                .bind(rec.expires_at)
                .bind(rec.created_at)
                .execute(pool)
                .await;
            }

            state
                .email_service
                .send_magic_link(&email, &user_name, &raw_token, &user_locale)
                .await?;
        }

        Ok(MagicLinkRequestResponse {
            status: "sent".to_string(),
            message:
                "If an active account exists for this email address, a sign-in link has been sent."
                    .to_string(),
        })
    }

    pub async fn verify_magic_link(
        state: &AppState,
        raw_token: &str,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(AuthSessionResponse, String), AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        let now = Utc::now();

        let user_rec = {
            let mut store = state.auth_store.write().await;
            let ml = store
                .magic_links
                .iter_mut()
                .find(|m| m.token_hash == token_hash && m.used_at.is_none() && m.expires_at > now)
                .ok_or(AppError::InvalidOrExpiredToken)?;
            ml.used_at = Some(now);
            let uid = ml.user_id;

            let user = store
                .users
                .iter_mut()
                .find(|u| u.id == uid && u.is_active)
                .ok_or(AppError::InvalidOrExpiredToken)?;
            user.failed_login_attempts = 0;
            user.locked_until = None;
            user.last_login_at = Some(now);
            user.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query("UPDATE magic_links SET used_at = NOW() WHERE token_hash = $1")
                .bind(&token_hash)
                .execute(pool)
                .await;
            let _ = sqlx::query(
                "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, last_login_at = NOW() WHERE id = $1",
            )
            .bind(user_rec.id)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;

        Ok((
            AuthSessionResponse {
                user: user_dto,
                session_token,
                prompt_passkey_enrollment: true,
            },
            cookie_header,
        ))
    }

    pub async fn create_admin_sign_in_link(
        state: &AppState,
        target_user_id: Uuid,
    ) -> Result<AdminSignInLinkResponse, AppError> {
        let now = Utc::now();
        let expires_at = now + ChronoDuration::hours(24);
        let raw_token = generate_random_token_base64url();
        let token_hash = sha256_hex(&raw_token);

        {
            let mut store = state.auth_store.write().await;
            if !store.users.iter().any(|u| u.id == target_user_id) {
                return Err(AppError::NotFound);
            }
            store.magic_links.push(MagicLinkRecord {
                id: Uuid::new_v4(),
                user_id: target_user_id,
                token_hash: token_hash.clone(),
                expires_at,
                used_at: None,
                created_at: now,
            });
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO magic_links (user_id, token_hash, expires_at)
                 VALUES ($1, $2, $3)",
            )
            .bind(target_user_id)
            .bind(&token_hash)
            .bind(expires_at)
            .execute(pool)
            .await;
        }

        let origin = state
            .config
            .app_origin
            .split([',', ';'])
            .map(|s| s.trim().trim_end_matches('/'))
            .find(|s| !s.is_empty())
            .unwrap_or("http://localhost:8080");
        let sign_in_url = format!("{origin}/magic-link/verify?token={raw_token}");

        Ok(AdminSignInLinkResponse {
            sign_in_url,
            token: raw_token,
            expires_at: expires_at.to_rfc3339(),
        })
    }

    pub async fn login_with_recovery_code(
        state: &AppState,
        req: RecoveryCodeLoginRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(RecoveryCodeLoginResponse, String), AppError> {
        let email = req.email.trim().to_ascii_lowercase();
        let code_hash = hash_recovery_code(&req.code);
        let now = Utc::now();

        let user_rec =
            {
                let mut store = state.auth_store.write().await;
                let user = store
                    .users
                    .iter_mut()
                    .find(|u| u.email == email && u.is_active)
                    .ok_or(AppError::InvalidCredentials)?;

                if let Some(locked_until) = user.locked_until {
                    if locked_until > now {
                        return Err(AppError::AccountLocked);
                    }
                }

                let uid = user.id;
                let Some(code_rec) = store.recovery_codes.iter_mut().find(|rc| {
                    rc.user_id == uid && rc.used_at.is_none() && rc.code_hash == code_hash
                }) else {
                    let user_mut = store.users.iter_mut().find(|u| u.id == uid).unwrap();
                    user_mut.failed_login_attempts += 1;
                    if user_mut.failed_login_attempts >= 5 {
                        user_mut.locked_until = Some(now + ChronoDuration::minutes(15));
                        return Err(AppError::AccountLocked);
                    }
                    return Err(AppError::InvalidCredentials);
                };

                code_rec.used_at = Some(now);
                let user_mut = store.users.iter_mut().find(|u| u.id == uid).unwrap();
                user_mut.failed_login_attempts = 0;
                user_mut.locked_until = None;
                user_mut.last_login_at = Some(now);
                user_mut.clone()
            };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE recovery_codes SET used_at = NOW()
                 WHERE user_id = $1 AND code_hash = $2 AND used_at IS NULL",
            )
            .bind(user_rec.id)
            .bind(&code_hash)
            .execute(pool)
            .await;
        }

        let (session_token, cookie_header) =
            Self::create_session(state, user_rec.id, user_agent, ip_address, is_https).await?;
        let user_dto = Self::build_user_dto(state, &user_rec).await;
        let remaining = user_dto.remaining_recovery_codes;

        Ok((
            RecoveryCodeLoginResponse {
                user: user_dto,
                session_token,
                remaining_recovery_codes: remaining,
                prompt_passkey_enrollment: true,
            },
            cookie_header,
        ))
    }

    pub async fn regenerate_recovery_codes(
        state: &AppState,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let now = Utc::now();
        let generated = generate_recovery_codes();
        let records: Vec<RecoveryCodeRecord> = generated
            .iter()
            .map(|(_, hash)| RecoveryCodeRecord {
                id: Uuid::new_v4(),
                user_id,
                code_hash: hash.clone(),
                used_at: None,
                created_at: now,
            })
            .collect();

        {
            let mut store = state.auth_store.write().await;
            store.recovery_codes.retain(|rc| rc.user_id != user_id);
            store.recovery_codes.extend(records.clone());
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query("DELETE FROM recovery_codes WHERE user_id = $1")
                .bind(user_id)
                .execute(pool)
                .await;
            Self::persist_recovery_codes_to_db(pool, user_id, &records).await?;
        }

        Ok(generated.into_iter().map(|(c, _)| c).collect())
    }

    pub async fn add_passkey(
        state: &AppState,
        user_id: Uuid,
        req: PasskeyRegistrationInput,
    ) -> Result<PasskeyCredentialDto, AppError> {
        let cred_id = req.credential_id.trim().to_string();
        if cred_id.is_empty() || req.public_key.trim().is_empty() {
            let mut fields = HashMap::new();
            fields.insert(
                "credential_id".to_string(),
                "invalid_credential".to_string(),
            );
            return Err(AppError::ValidationFailed(
                "credential_id and public_key are required".to_string(),
                Some(fields),
            ));
        }
        Self::consume_challenge(state, &req.challenge, req.client_data_json.as_deref()).await?;
        let now = Utc::now();

        let pk_rec = PasskeyRecord {
            id: Uuid::new_v4(),
            user_id,
            credential_id: cred_id,
            public_key: req.public_key.trim().to_string(),
            sign_count: 0,
            transports: req
                .transports
                .unwrap_or_else(|| vec!["internal".to_string()]),
            device_name: req
                .device_name
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "Additional Passkey".to_string()),
            aaguid: req.aaguid.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
            created_at: now,
            last_used_at: Some(now),
        };

        {
            let mut store = state.auth_store.write().await;
            if store
                .passkeys
                .iter()
                .any(|p| p.credential_id == pk_rec.credential_id)
            {
                let mut fields = HashMap::new();
                fields.insert("credential_id".to_string(), "duplicate".to_string());
                return Err(AppError::ValidationFailed(
                    "Passkey is already registered".to_string(),
                    Some(fields),
                ));
            }
            store.passkeys.push(pk_rec.clone());
        }

        if let Some(pool) = &state.db {
            Self::persist_passkey_to_db(pool, &pk_rec).await?;
        }

        Ok(Self::to_passkey_dto(&pk_rec))
    }

    pub async fn rename_passkey(
        state: &AppState,
        user_id: Uuid,
        passkey_id: Uuid,
        device_name: &str,
    ) -> Result<PasskeyCredentialDto, AppError> {
        let trimmed = device_name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 100 {
            let mut fields = HashMap::new();
            fields.insert("device_name".to_string(), "invalid_length".to_string());
            return Err(AppError::ValidationFailed(
                "Passkey device name must be 1..=100 characters".to_string(),
                Some(fields),
            ));
        }

        let updated = {
            let mut store = state.auth_store.write().await;
            let pk = store
                .passkeys
                .iter_mut()
                .find(|p| p.id == passkey_id && p.user_id == user_id)
                .ok_or(AppError::NotFound)?;
            pk.device_name = trimmed.to_string();
            pk.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE passkey_credentials SET device_name = $1 WHERE id = $2 AND user_id = $3",
            )
            .bind(trimmed)
            .bind(passkey_id)
            .bind(user_id)
            .execute(pool)
            .await;
        }

        Ok(Self::to_passkey_dto(&updated))
    }

    pub async fn delete_passkey(
        state: &AppState,
        user_id: Uuid,
        passkey_id: Uuid,
    ) -> Result<(), AppError> {
        {
            let mut store = state.auth_store.write().await;
            let user_passkeys_count = store
                .passkeys
                .iter()
                .filter(|p| p.user_id == user_id)
                .count();
            let has_target = store
                .passkeys
                .iter()
                .any(|p| p.id == passkey_id && p.user_id == user_id);

            if !has_target {
                return Err(AppError::NotFound);
            }

            let has_password = store
                .users
                .iter()
                .find(|u| u.id == user_id)
                .and_then(|u| u.password_hash.as_ref())
                .is_some();

            // Prevent removing the last passkey when no password is set (or when only 1 passkey remains)
            if user_passkeys_count <= 1 && !has_password {
                return Err(AppError::LastAuthMethod);
            }

            store
                .passkeys
                .retain(|p| !(p.id == passkey_id && p.user_id == user_id));
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query("DELETE FROM passkey_credentials WHERE id = $1 AND user_id = $2")
                .bind(passkey_id)
                .bind(user_id)
                .execute(pool)
                .await;
        }

        Ok(())
    }

    pub async fn set_password(
        state: &AppState,
        user_id: Uuid,
        req: SetPasswordRequest,
    ) -> Result<(), AppError> {
        let new_hash = hash_password(&req.new_password)?;

        {
            let mut store = state.auth_store.write().await;
            let user = store
                .users
                .iter_mut()
                .find(|u| u.id == user_id)
                .ok_or(AppError::NotAuthenticated)?;

            if let Some(ref existing_hash) = user.password_hash {
                let current_ok = req
                    .current_password
                    .as_deref()
                    .map(|pw| verify_password(pw, existing_hash))
                    .unwrap_or(false);
                if !current_ok {
                    return Err(AppError::InvalidCredentials);
                }
            }

            user.password_hash = Some(new_hash.clone());
            user.updated_at = Utc::now();
        }

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&new_hash)
            .bind(user_id)
            .execute(pool)
            .await;
        }

        Ok(())
    }

    pub async fn create_session(
        state: &AppState,
        user_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
        is_https: bool,
    ) -> Result<(String, String), AppError> {
        let raw_token = generate_random_token_base64url();
        let token_hash = sha256_hex(&raw_token);
        let now = Utc::now();
        let ttl_days = i64::from(state.config.session_ttl_days.max(1));
        let expires_at = now + ChronoDuration::days(ttl_days);

        let session_rec = SessionRecord {
            id: Uuid::new_v4(),
            user_id,
            token_hash: token_hash.clone(),
            user_agent,
            ip_address,
            created_at: now,
            last_seen_at: now,
            expires_at,
        };

        state
            .auth_store
            .write()
            .await
            .sessions
            .push(session_rec.clone());

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "INSERT INTO sessions (id, user_id, token_hash, user_agent, ip_address, created_at, last_seen_at, expires_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(session_rec.id)
            .bind(session_rec.user_id)
            .bind(&session_rec.token_hash)
            .bind(&session_rec.user_agent)
            .bind(&session_rec.ip_address)
            .bind(session_rec.created_at)
            .bind(session_rec.last_seen_at)
            .bind(session_rec.expires_at)
            .execute(pool)
            .await;
        }

        let max_age_secs = ttl_days * 86_400;
        let is_configured_https = state
            .config
            .app_origin
            .split([',', ';'])
            .any(|o| o.trim().starts_with("https://"));
        let secure_flag = if is_https || is_configured_https {
            "; Secure"
        } else {
            ""
        };
        let cookie = format!(
            "session={raw_token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age_secs}{secure_flag}"
        );

        Ok((raw_token, cookie))
    }

    pub fn apply_forwarded_proto_secure_cookie(
        cookie: String,
        headers: &axum::http::HeaderMap,
    ) -> String {
        let is_forwarded_https = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .is_some_and(|proto| proto.trim().eq_ignore_ascii_case("https"));
        if is_forwarded_https && !cookie.contains("; Secure") {
            format!("{cookie}; Secure")
        } else {
            cookie
        }
    }

    pub async fn authenticate_session(
        state: &AppState,
        raw_token: &str,
    ) -> Result<(UserRecord, SessionRecord), AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        let now = Utc::now();
        let ttl_days = i64::from(state.config.session_ttl_days.max(1));

        let sess_opt = {
            let store = state.auth_store.read().await;
            store
                .sessions
                .iter()
                .find(|s| s.token_hash == token_hash)
                .cloned()
        };

        let session = match sess_opt {
            Some(s) => s,
            None => {
                if let Some(pool) = &state.db {
                    let row_opt = sqlx::query(
                        "SELECT id, user_id, token_hash, user_agent, ip_address, created_at, last_seen_at, expires_at \
                         FROM sessions \
                         WHERE token_hash = $1 AND expires_at > NOW()",
                    )
                    .bind(&token_hash)
                    .fetch_optional(pool)
                    .await?;

                    if let Some(r) = row_opt {
                        let s = SessionRecord {
                            id: r.try_get("id")?,
                            user_id: r.try_get("user_id")?,
                            token_hash: r.try_get("token_hash")?,
                            user_agent: r.try_get("user_agent").ok().flatten(),
                            ip_address: r.try_get("ip_address").ok().flatten(),
                            created_at: r.try_get("created_at")?,
                            last_seen_at: r.try_get("last_seen_at")?,
                            expires_at: r.try_get("expires_at")?,
                        };
                        let mut store = state.auth_store.write().await;
                        store.sessions.push(s.clone());
                        s
                    } else {
                        return Err(AppError::NotAuthenticated);
                    }
                } else {
                    return Err(AppError::NotAuthenticated);
                }
            }
        };

        if session.expires_at <= now {
            let mut store = state.auth_store.write().await;
            store.sessions.retain(|s| s.id != session.id);
            return Err(AppError::SessionExpired);
        }

        let _ = Self::ensure_user_by_id(state, session.user_id).await;

        let user = {
            let mut store = state.auth_store.write().await;
            if let Some(pos) = store.sessions.iter().position(|s| s.id == session.id) {
                store.sessions[pos].last_seen_at = now;
                store.sessions[pos].expires_at = now + ChronoDuration::days(ttl_days);
            }
            store
                .users
                .iter()
                .find(|u| u.id == session.user_id && u.is_active)
                .cloned()
                .ok_or(AppError::NotAuthenticated)?
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE sessions SET last_seen_at = NOW(), expires_at = $1 WHERE id = $2",
            )
            .bind(now + ChronoDuration::days(ttl_days))
            .bind(session.id)
            .execute(pool)
            .await;
        }

        Ok((user, session))
    }

    pub async fn build_user_dto(state: &AppState, user: &UserRecord) -> UserDto {
        let store = state.auth_store.read().await;
        let passkeys_count = store
            .passkeys
            .iter()
            .filter(|p| p.user_id == user.id)
            .count();
        let remaining_recovery_codes = store
            .recovery_codes
            .iter()
            .filter(|rc| rc.user_id == user.id && rc.used_at.is_none())
            .count();

        UserDto {
            id: user.id.to_string(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            display_name: format!("{} {}", user.first_name, user.last_name),
            role: user.role.clone(),
            person_id: user.person_id.map(|id| id.to_string()),
            is_active: user.is_active,
            locale: user.locale.clone(),
            theme: user.theme.clone(),
            home_person_id: user.home_person_id.map(|id| id.to_string()),
            start_page: user.start_page.clone(),
            digest_opt_in: user.digest_opt_in,
            has_password: user.password_hash.is_some(),
            passkeys_count,
            remaining_recovery_codes,
            last_login_at: user.last_login_at.map(|d| d.to_rfc3339()),
            created_at: user.created_at.to_rfc3339(),
        }
    }

    pub async fn update_profile(
        state: &AppState,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<UserDto, AppError> {
        let mut fields = HashMap::new();

        if let Some(ref first) = req.first_name {
            let trimmed = first.trim();
            if trimmed.is_empty() || trimmed.chars().count() > 60 {
                fields.insert("first_name".to_string(), "invalid_length".to_string());
            }
        }
        if let Some(ref last) = req.last_name {
            let trimmed = last.trim();
            if trimmed.is_empty() || trimmed.chars().count() > 60 {
                fields.insert("last_name".to_string(), "invalid_length".to_string());
            }
        }
        if let Some(ref locale) = req.locale {
            if !matches!(locale.trim(), "en" | "ru") {
                fields.insert("locale".to_string(), "invalid_locale".to_string());
            }
        }
        if let Some(ref theme) = req.theme {
            if !matches!(theme.trim(), "light" | "dark" | "system") {
                fields.insert("theme".to_string(), "invalid_theme".to_string());
            }
        }
        if let Some(ref sp) = req.start_page {
            if !matches!(sp.trim(), "tree" | "dashboard") {
                fields.insert("start_page".to_string(), "invalid_start_page".to_string());
            }
        }

        let resolved_home_person: Option<Option<Uuid>> = match req.home_person_id.as_deref() {
            Some(raw) if raw.trim().is_empty() => Some(None),
            Some(raw) => {
                let target = raw.trim();
                let p_store = state.privacy_store.read().await;
                if let Some(p) = p_store
                    .persons
                    .iter()
                    .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
                {
                    Some(Uuid::parse_str(&p.id).ok())
                } else if let Ok(uid) = Uuid::parse_str(target) {
                    Some(Some(uid))
                } else {
                    fields.insert("home_person_id".to_string(), "invalid_uuid".to_string());
                    None
                }
            }
            None => None,
        };

        if !fields.is_empty() {
            return Err(AppError::ValidationFailed(
                "Invalid user profile preferences".to_string(),
                Some(fields),
            ));
        }

        let updated_user = {
            let mut store = state.auth_store.write().await;
            let u = store
                .users
                .iter_mut()
                .find(|u| u.id == user_id)
                .ok_or(AppError::NotAuthenticated)?;

            if let Some(first) = req.first_name {
                u.first_name = first.trim().to_string();
            }
            if let Some(last) = req.last_name {
                u.last_name = last.trim().to_string();
            }
            if let Some(locale) = req.locale {
                u.locale = Some(locale.trim().to_string());
            }
            if let Some(theme) = req.theme {
                u.theme = Some(theme.trim().to_string());
            }
            if let Some(hp_opt) = resolved_home_person {
                u.home_person_id = hp_opt;
            }
            if let Some(sp) = req.start_page {
                u.start_page = sp.trim().to_string();
            }
            if let Some(digest) = req.digest_opt_in {
                u.digest_opt_in = digest;
            }
            u.updated_at = Utc::now();
            u.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users
                 SET first_name = $1, last_name = $2, locale = $3, theme = $4,
                     home_person_id = $5, start_page = $6, digest_opt_in = $7, updated_at = NOW()
                 WHERE id = $8",
            )
            .bind(&updated_user.first_name)
            .bind(&updated_user.last_name)
            .bind(&updated_user.locale)
            .bind(&updated_user.theme)
            .bind(updated_user.home_person_id)
            .bind(&updated_user.start_page)
            .bind(updated_user.digest_opt_in)
            .bind(updated_user.id)
            .execute(pool)
            .await;
        }

        Ok(Self::build_user_dto(state, &updated_user).await)
    }

    pub async fn set_home_person(
        state: &AppState,
        user_id: Uuid,
        person_id_raw: Option<String>,
    ) -> Result<UserDto, AppError> {
        let resolved_uuid: Option<Uuid> = match person_id_raw.as_deref() {
            None => None,
            Some(raw) if raw.trim().is_empty() => None,
            Some(raw) => {
                let target = raw.trim();
                let p_store = state.privacy_store.read().await;
                if let Some(p) = p_store
                    .persons
                    .iter()
                    .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
                {
                    let uid = Uuid::parse_str(&p.id).map_err(|_| {
                        let mut fields = HashMap::new();
                        fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                        AppError::ValidationFailed(
                            "Resolved person ID is not a valid UUID".to_string(),
                            Some(fields),
                        )
                    })?;
                    Some(uid)
                } else {
                    return Err(AppError::NotFound);
                }
            }
        };

        let updated_user = {
            let mut store = state.auth_store.write().await;
            let u = store
                .users
                .iter_mut()
                .find(|u| u.id == user_id)
                .ok_or(AppError::NotAuthenticated)?;
            u.home_person_id = resolved_uuid;
            u.updated_at = Utc::now();
            u.clone()
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users SET home_person_id = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(updated_user.home_person_id)
            .bind(updated_user.id)
            .execute(pool)
            .await;
        }

        Ok(Self::build_user_dto(state, &updated_user).await)
    }

    pub fn to_passkey_dto(pk: &PasskeyRecord) -> PasskeyCredentialDto {
        PasskeyCredentialDto {
            id: pk.id.to_string(),
            credential_id: pk.credential_id.clone(),
            device_name: pk.device_name.clone(),
            transports: pk.transports.clone(),
            sign_count: pk.sign_count,
            aaguid: pk.aaguid.map(|u| u.to_string()),
            created_at: pk.created_at.to_rfc3339(),
            last_used_at: pk.last_used_at.map(|d| d.to_rfc3339()),
        }
    }

    pub fn to_invitation_dto(inv: &InvitationRecord) -> InvitationDto {
        InvitationDto {
            id: inv.id.to_string(),
            email: inv.email.clone(),
            invitee_name: inv.invitee_name.clone(),
            role: inv.role.clone(),
            person_id: inv.person_id.map(|u| u.to_string()),
            created_by: inv.created_by.map(|u| u.to_string()),
            expires_at: inv.expires_at.to_rfc3339(),
            used_at: inv.used_at.map(|d| d.to_rfc3339()),
            revoked_at: inv.revoked_at.map(|d| d.to_rfc3339()),
            created_at: inv.created_at.to_rfc3339(),
        }
    }

    pub fn to_admin_user_summary(u: &UserRecord, passkeys_count: usize) -> AdminUserSummaryDto {
        AdminUserSummaryDto {
            id: u.id.to_string(),
            email: u.email.clone(),
            first_name: u.first_name.clone(),
            last_name: u.last_name.clone(),
            display_name: format!("{} {}", u.first_name, u.last_name),
            role: u.role.clone(),
            person_id: u.person_id.map(|p| p.to_string()),
            is_active: u.is_active,
            passkeys_count,
            has_password: u.password_hash.is_some(),
            last_login_at: u.last_login_at.map(|d| d.to_rfc3339()),
            deactivated_at: u.deactivated_at.map(|d| d.to_rfc3339()),
            created_at: u.created_at.to_rfc3339(),
        }
    }

    pub async fn update_admin_user(
        state: &AppState,
        target_user_id: Uuid,
        req: UpdateAdminUserRequest,
    ) -> Result<AdminUserSummaryDto, AppError> {
        let normalized_role = if let Some(ref raw_role) = req.role {
            let r = raw_role.trim().to_ascii_lowercase();
            if !matches!(r.as_str(), "admin" | "editor" | "viewer") {
                let mut fields = HashMap::new();
                fields.insert("role".to_string(), "invalid_role".to_string());
                return Err(AppError::ValidationFailed(
                    "Role must be 'admin', 'editor', or 'viewer'".to_string(),
                    Some(fields),
                ));
            }
            Some(r)
        } else {
            None
        };

        let parsed_person_id = match req.person_id.as_deref() {
            Some("") => Some(None),
            Some(s) => {
                let target = s.trim();
                let p_store = state.privacy_store.read().await;
                if let Some(p) = p_store
                    .persons
                    .iter()
                    .find(|p| p.id == target || p.gid.eq_ignore_ascii_case(target))
                {
                    let id = Uuid::parse_str(&p.id).map_err(|_| {
                        let mut fields = HashMap::new();
                        fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                        AppError::ValidationFailed(
                            "person_id must be a valid UUID".to_string(),
                            Some(fields),
                        )
                    })?;
                    Some(Some(id))
                } else {
                    let id = Uuid::parse_str(target).map_err(|_| {
                        let mut fields = HashMap::new();
                        fields.insert("person_id".to_string(), "invalid_uuid".to_string());
                        AppError::ValidationFailed(
                            "person_id must be a valid UUID".to_string(),
                            Some(fields),
                        )
                    })?;
                    Some(Some(id))
                }
            }
            None => None,
        };

        let now = Utc::now();
        let (updated_user, passkeys_count, purge_sessions) = {
            let mut store = state.auth_store.write().await;
            let active_admins_count = store
                .users
                .iter()
                .filter(|u| u.role == "admin" && u.is_active)
                .count();

            let target_idx = store
                .users
                .iter()
                .position(|u| u.id == target_user_id)
                .ok_or(AppError::NotFound)?;

            let target_currently_active_admin =
                store.users[target_idx].role == "admin" && store.users[target_idx].is_active;

            let would_lose_admin_role = normalized_role
                .as_deref()
                .map(|r| r != "admin")
                .unwrap_or(false);
            let would_be_deactivated = req.is_active == Some(false);

            // Sole Admin Protection (`02 §5.2 & §7 DoD #10`)
            if target_currently_active_admin
                && (would_lose_admin_role || would_be_deactivated)
                && active_admins_count <= 1
            {
                return Err(AppError::SoleAdminProtected);
            }

            let u = &mut store.users[target_idx];
            if let Some(ref r) = normalized_role {
                u.role = r.clone();
            }
            if let Some(pid_opt) = parsed_person_id {
                u.person_id = pid_opt;
                if u.home_person_id.is_none() {
                    u.home_person_id = pid_opt;
                }
            }

            let mut purge = false;
            if let Some(active) = req.is_active {
                u.is_active = active;
                if !active {
                    u.deactivated_at = Some(now);
                    purge = true;
                } else {
                    u.deactivated_at = None;
                }
            }
            u.updated_at = now;
            let cloned_user = u.clone();

            if purge {
                store.sessions.retain(|s| s.user_id != target_user_id);
            }

            let pk_cnt = store
                .passkeys
                .iter()
                .filter(|p| p.user_id == target_user_id)
                .count();

            (cloned_user, pk_cnt, purge)
        };

        if let Some(pool) = &state.db {
            let _ = sqlx::query(
                "UPDATE users
                 SET role = $1, person_id = $2, home_person_id = $3, is_active = $4, deactivated_at = $5, updated_at = NOW()
                 WHERE id = $6",
            )
            .bind(&updated_user.role)
            .bind(updated_user.person_id)
            .bind(updated_user.home_person_id)
            .bind(updated_user.is_active)
            .bind(updated_user.deactivated_at)
            .bind(updated_user.id)
            .execute(pool)
            .await;

            if purge_sessions {
                let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                    .bind(target_user_id)
                    .execute(pool)
                    .await;
            }
        }

        Ok(Self::to_admin_user_summary(&updated_user, passkeys_count))
    }

    fn validate_user_names_and_email(
        first_name: &str,
        last_name: &str,
        email: &str,
    ) -> Result<(), AppError> {
        let mut fields = HashMap::new();
        if first_name.trim().is_empty() || first_name.trim().chars().count() > 60 {
            fields.insert("first_name".to_string(), "invalid_length".to_string());
        }
        if last_name.trim().is_empty() || last_name.trim().chars().count() > 60 {
            fields.insert("last_name".to_string(), "invalid_length".to_string());
        }
        let email_trimmed = email.trim();
        if email_trimmed.is_empty() || !email_trimmed.contains('@') || email_trimmed.len() > 255 {
            fields.insert("email".to_string(), "invalid_email".to_string());
        }

        if !fields.is_empty() {
            return Err(AppError::ValidationFailed(
                "Invalid user profile fields".to_string(),
                Some(fields),
            ));
        }
        Ok(())
    }

    async fn consume_emergency_admin_token(
        state: &AppState,
        raw_token: &str,
    ) -> Result<(), AppError> {
        let token_hash = sha256_hex(raw_token.trim());
        if let Some(pool) = &state.db {
            if let Ok(Some(_)) = sqlx::query(
                "UPDATE emergency_admin_links
                 SET used_at = NOW()
                 WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
                 RETURNING id",
            )
            .bind(&token_hash)
            .fetch_optional(pool)
            .await
            {
                return Ok(());
            }
        }
        Err(AppError::InvalidOrExpiredToken)
    }

    async fn persist_user_to_db(pool: &sqlx::PgPool, u: &UserRecord) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO users (id, email, first_name, last_name, role, person_id, is_active, password_hash, locale, theme, home_person_id, start_page, digest_opt_in, last_login_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        )
        .bind(u.id)
        .bind(&u.email)
        .bind(&u.first_name)
        .bind(&u.last_name)
        .bind(&u.role)
        .bind(u.person_id)
        .bind(u.is_active)
        .bind(&u.password_hash)
        .bind(&u.locale)
        .bind(&u.theme)
        .bind(u.home_person_id)
        .bind(&u.start_page)
        .bind(u.digest_opt_in)
        .bind(u.last_login_at)
        .bind(u.created_at)
        .bind(u.updated_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn persist_passkey_to_db(
        pool: &sqlx::PgPool,
        pk: &PasskeyRecord,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO passkey_credentials (id, user_id, credential_id, public_key, sign_count, transports, device_name, aaguid, created_at, last_used_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(pk.id)
        .bind(pk.user_id)
        .bind(&pk.credential_id)
        .bind(&pk.public_key)
        .bind(pk.sign_count)
        .bind(&pk.transports)
        .bind(&pk.device_name)
        .bind(pk.aaguid)
        .bind(pk.created_at)
        .bind(pk.last_used_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn persist_recovery_codes_to_db(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        codes: &[RecoveryCodeRecord],
    ) -> Result<(), AppError> {
        for rc in codes {
            sqlx::query(
                "INSERT INTO recovery_codes (id, user_id, code_hash, created_at)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(rc.id)
            .bind(user_id)
            .bind(&rc.code_hash)
            .bind(rc.created_at)
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}
