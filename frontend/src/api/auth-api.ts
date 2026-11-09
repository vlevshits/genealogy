import type { FamilyDto } from "./openapi-types";
import { apiFetch } from "./core";

export interface UserDto {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  display_name: string;
  role: 'admin' | 'editor' | 'viewer' | string;
  person_id: string | null;
  is_active: boolean;
  has_password: boolean;
  passkeys_count: number;
  remaining_recovery_codes: number;
  locale: string | null;
  theme: string | null;
  home_person_id: string | null;
  start_page?: 'tree' | 'dashboard';
  digest_opt_in?: boolean;
  created_at: string;
  last_login_at: string | null;
}

export interface SetupStatusResponse {
  is_initialized: boolean;
  tree_title: string;
  default_locale: string;
  enable_public_access: boolean;
  auth_password_enabled: boolean;
  smtp_enabled: boolean;
  rp_id: string;
  rp_name: string;
}

export interface WebAuthnChallengeResponse {
  challenge: string;
  rp_id: string;
  rp_name: string;
  timeout_ms: number;
}

export interface PasskeyRegistrationInput {
  challenge: string;
  credential_id: string;
  public_key: string;
  client_data_json?: string;
  device_name?: string;
  transports?: string[];
  aaguid?: string | null;
}

export interface SetupRequest {
  tree_title?: string;
  first_name: string;
  last_name: string;
  email: string;
  password?: string;
  passkey?: PasskeyRegistrationInput;
  emergency_token?: string;
}

export interface SetupCompleteResponse {
  user: UserDto;
  recovery_codes: string[];
  session_token: string;
}

export interface InvitePreviewResponse {
  email: string | null;
  invitee_name: string | null;
  inviter_name: string;
  role: string;
  person_id: string | null;
  tree_title: string;
  expires_at: string;
  auth_password_enabled: boolean;
}

export interface InviteRegisterRequest {
  invite_token: string;
  first_name: string;
  last_name: string;
  email?: string;
  password?: string;
  passkey?: PasskeyRegistrationInput;
}

export interface LoginChallengeRequest {
  email?: string;
}

export interface LoginChallengeResponse {
  challenge: string;
  rp_id: string;
  timeout_ms: number;
  allow_credentials: string[];
}

export interface PasskeyLoginVerifyRequest {
  challenge: string;
  credential_id: string;
  authenticator_data?: string;
  client_data_json?: string;
  signature?: string;
  sign_count?: number;
}

export interface AuthSessionResponse {
  user: UserDto;
  session_token: string;
  prompt_passkey_enrollment: boolean;
}

export interface PasswordLoginRequest {
  email: string;
  password: string;
}

export interface MagicLinkRequestInput {
  email: string;
}

export interface MagicLinkRequestResponse {
  status: string;
  message: string;
  expires_in_seconds?: number;
}

export interface MagicLinkVerifyInput {
  token: string;
}

export interface RecoveryCodeLoginInput {
  email: string;
  code: string;
}

export interface RecoveryCodeLoginResponse {
  user: UserDto;
  session_token: string;
  remaining_recovery_codes: number;
  prompt_passkey_enrollment: boolean;
}

export interface PasskeyCredentialDto {
  id: string;
  credential_id: string;
  device_name: string;
  transports: string[];
  aaguid: string | null;
  sign_count: number;
  created_at: string;
  last_used_at: string | null;
}

export interface PasskeyListResponse {
  items: PasskeyCredentialDto[];
}

export interface RenamePasskeyRequest {
  device_name: string;
}

export interface SetPasswordRequest {
  current_password?: string;
  new_password: string;
}

export interface UpdateProfileRequest {
  first_name?: string;
  last_name?: string;
  locale?: string;
  theme?: string;
  home_person_id?: string | null;
  start_page?: 'tree' | 'dashboard';
  digest_opt_in?: boolean;
}

export interface RegenerateRecoveryCodesResponse {
  recovery_codes: string[];
}

export interface SessionDto {
  id: string;
  user_agent: string | null;
  ip_address: string | null;
  is_current: boolean;
  created_at: string;
  last_seen_at: string;
  expires_at: string;
}

export interface SessionListResponse {
  items: SessionDto[];
}

export interface CreateInviteRequest {
  email?: string;
  invitee_name?: string;
  role: 'editor' | 'viewer' | string;
  person_id?: string | null;
  expires_in_hours?: number;
}

export interface InvitationDto {
  id: string;
  role: string;
  person_id: string | null;
  email: string | null;
  invitee_name: string | null;
  created_by: string;
  created_at: string;
  expires_at: string;
  used_at: string | null;
  revoked_at: string | null;
}

export interface InviteCreatedResponse {
  invitation: InvitationDto;
  invite_url: string;
  token: string;
}

export interface InvitationListResponse {
  items: InvitationDto[];
}

export interface AdminUserSummaryDto {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  display_name: string;
  role: string;
  person_id: string | null;
  is_active: boolean;
  deactivated_at?: string | null;
  passkeys_count: number;
  has_password: boolean;
  created_at: string;
  last_login_at: string | null;
}

export interface UpdateAdminUserInput {
  role?: 'admin' | 'editor' | 'viewer';
  is_active?: boolean;
  linked_person_id?: string | null;
}

export interface TreePersonNodeDto {
  id: string;
  gid: string;
  sex: 'M' | 'F' | 'U';
  is_living: boolean;
  living_override?: boolean | null;
  is_private: boolean;
  primary_name: {
    given_name: string;
    patronymic: string | null;
    surname: string;
  };
  birth_surname?: string | null;
  nickname?: string | null;
  name_prefix?: string | null;
  name_suffix?: string | null;
  biography?: string | null;
  birth: { date: string | null; place: string | null } | null;
  death: { date: string | null; place: string | null } | null;
  avatar_url: string | null;
  notes: string | null;
  birth_sort?: number | null;
  death_sort?: number | null;
  has_death_event?: boolean;
  alt_names?: import('./openapi-types').IndividualAltNameDto[];
  version?: number;
  generation?: number | null;
  is_direct_lineage?: boolean | null;
}

export interface TreeDataResponse {
  access_tier: 'public_anonymous' | 'viewer' | 'editor' | 'admin';
  enable_public_access: boolean;
  resolved_home_person_id?: string | null;
  nodes: TreePersonNodeDto[];
  families?: FamilyDto[];
}

export type TreeResponse = TreeDataResponse;

export interface AdminUsersListResponse {
  items: AdminUserSummaryDto[];
}

export interface AdminSignInLinkResponse {
  sign_in_url: string;
  token: string;
  expires_at: string;
}


// =============================================================================
// Phase 1.2 Auth, Passkeys, Recovery, Sessions & Admin User/Invite Endpoints
// =============================================================================

export async function fetchSetupStatus(): Promise<SetupStatusResponse> {
  return apiFetch<SetupStatusResponse>('/auth/setup/status');
}

export async function requestRegisterChallenge(): Promise<WebAuthnChallengeResponse> {
  return apiFetch<WebAuthnChallengeResponse>('/auth/register/challenge', {
    method: 'POST',
  });
}

export async function submitInitialSetup(
  payload: SetupRequest
): Promise<SetupCompleteResponse> {
  return apiFetch<SetupCompleteResponse>('/auth/setup', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function fetchInvitePreview(
  token: string
): Promise<InvitePreviewResponse> {
  return apiFetch<InvitePreviewResponse>(
    `/auth/invites/${encodeURIComponent(token)}`
  );
}

export async function registerWithInvite(
  payload: InviteRegisterRequest
): Promise<SetupCompleteResponse> {
  return apiFetch<SetupCompleteResponse>('/auth/register', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function requestLoginChallenge(
  payload: LoginChallengeRequest = {}
): Promise<LoginChallengeResponse> {
  return apiFetch<LoginChallengeResponse>('/auth/login/challenge', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function verifyPasskeyLogin(
  payload: PasskeyLoginVerifyRequest
): Promise<AuthSessionResponse> {
  return apiFetch<AuthSessionResponse>('/auth/login/verify', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function loginWithPassword(
  payload: PasswordLoginRequest
): Promise<AuthSessionResponse> {
  return apiFetch<AuthSessionResponse>('/auth/password/login', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function requestMagicLink(
  payload: MagicLinkRequestInput
): Promise<MagicLinkRequestResponse> {
  return apiFetch<MagicLinkRequestResponse>('/auth/magic-link/request', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function verifyMagicLink(
  payload: MagicLinkVerifyInput
): Promise<AuthSessionResponse> {
  return apiFetch<AuthSessionResponse>('/auth/magic-link/verify', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function loginWithRecoveryCode(
  payload: RecoveryCodeLoginInput
): Promise<RecoveryCodeLoginResponse> {
  return apiFetch<RecoveryCodeLoginResponse>('/auth/recovery-code', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function fetchCurrentUser(): Promise<UserDto> {
  return apiFetch<UserDto>('/auth/me');
}

export async function updateCurrentUserProfile(
  payload: UpdateProfileRequest
): Promise<UserDto> {
  return apiFetch<UserDto>('/auth/me', {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function logoutSession(): Promise<{ status: string }> {
  return apiFetch<{ status: string }>('/auth/logout', {
    method: 'POST',
  });
}

export async function fetchPasskeys(): Promise<PasskeyListResponse> {
  return apiFetch<PasskeyListResponse>('/auth/passkeys');
}

export async function requestAddPasskeyChallenge(): Promise<WebAuthnChallengeResponse> {
  return apiFetch<WebAuthnChallengeResponse>('/auth/passkeys/add/challenge', {
    method: 'POST',
  });
}

export async function verifyAddPasskey(
  payload: PasskeyRegistrationInput
): Promise<PasskeyCredentialDto> {
  return apiFetch<PasskeyCredentialDto>('/auth/passkeys/add/verify', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function renamePasskey(
  id: string,
  payload: RenamePasskeyRequest
): Promise<PasskeyCredentialDto> {
  return apiFetch<PasskeyCredentialDto>(
    `/auth/passkeys/${encodeURIComponent(id)}`,
    {
      method: 'PATCH',
      body: JSON.stringify(payload),
    }
  );
}

export async function deletePasskey(id: string): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(
    `/auth/passkeys/${encodeURIComponent(id)}`,
    {
      method: 'DELETE',
    }
  );
}

export async function regenerateRecoveryCodes(): Promise<RegenerateRecoveryCodesResponse> {
  return apiFetch<RegenerateRecoveryCodesResponse>(
    '/auth/recovery-codes/regenerate',
    {
      method: 'POST',
    }
  );
}

export async function updatePassword(
  payload: SetPasswordRequest
): Promise<{ status: string }> {
  return apiFetch<{ status: string }>('/auth/password', {
    method: 'PUT',
    body: JSON.stringify(payload),
  });
}

export async function fetchSessions(): Promise<SessionListResponse> {
  return apiFetch<SessionListResponse>('/auth/sessions');
}

export async function revokeSession(id: string): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(
    `/auth/sessions/${encodeURIComponent(id)}`,
    {
      method: 'DELETE',
    }
  );
}

export async function revokeOtherSessions(): Promise<{ revoked_count: number }> {
  return apiFetch<{ revoked_count: number }>('/auth/sessions', {
    method: 'DELETE',
  });
}

export async function fetchAdminUsers(): Promise<AdminUsersListResponse> {
  return apiFetch<AdminUsersListResponse>('/admin/users');
}

export async function fetchAdminInvites(): Promise<InvitationListResponse> {
  return apiFetch<InvitationListResponse>('/admin/invites');
}

export async function createAdminInvite(
  payload: CreateInviteRequest
): Promise<InviteCreatedResponse> {
  return apiFetch<InviteCreatedResponse>('/admin/invites', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function revokeAdminInvite(
  id: string
): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(
    `/admin/invites/${encodeURIComponent(id)}`,
    {
      method: 'DELETE',
    }
  );
}

export async function createAdminSignInLink(
  userId: string
): Promise<AdminSignInLinkResponse> {
  return apiFetch<AdminSignInLinkResponse>(
    `/admin/users/${encodeURIComponent(userId)}/sign-in-link`,
    {
      method: 'POST',
    }
  );
}

export async function revokeAdminUserSessions(
  userId: string
): Promise<{ revoked_count: number }> {
  return apiFetch<{ revoked_count: number }>(
    `/admin/users/${encodeURIComponent(userId)}/sessions`,
    {
      method: 'DELETE',
    }
  );
}

// =============================================================================
// Phase 1.3 RBAC User Governance & Tree Privacy Endpoints
// =============================================================================

export async function updateAdminUser(
  userId: string,
  payload: UpdateAdminUserInput
): Promise<AdminUserSummaryDto> {
  return apiFetch<AdminUserSummaryDto>(
    `/admin/users/${encodeURIComponent(userId)}`,
    {
      method: 'PATCH',
      body: JSON.stringify(payload),
    }
  );
}

export async function fetchTreeData(): Promise<TreeDataResponse> {
  return apiFetch<TreeDataResponse>('/tree');
}

