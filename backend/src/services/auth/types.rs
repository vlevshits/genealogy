#[allow(unused_imports)]
use super::*;

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

pub(crate) fn default_start_page() -> String {
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
