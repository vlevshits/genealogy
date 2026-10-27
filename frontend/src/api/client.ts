export interface ApiErrorDetail {
  code: string;
  message: string;
  details?: Record<string, unknown>;
  field_errors?: Record<string, string>;
}

export interface ApiErrorResponse {
  error: ApiErrorDetail;
}

export class ApiError extends Error {
  code: string;
  status: number;
  details?: Record<string, unknown>;
  field_errors?: Record<string, string>;

  constructor(status: number, error: ApiErrorDetail) {
    super(error.message);
    this.name = 'ApiError';
    this.status = status;
    this.code = error.code;
    this.details = error.details;
    this.field_errors = error.field_errors;
  }
}

export interface HealthResponse {
  status: string;
  version: string;
  database: string;
  environment: string;
}

export interface SystemInfoResponse {
  app_version: string;
  tree_title: string;
  default_locale: string;
  enable_public_access: boolean;
  auth_password_enabled: boolean;
  settings_locked_from_env: boolean;
}

export interface AdminDatabaseStatus {
  status: string;
  size_bytes: number;
  tables_count: number;
}

export interface AdminStorageStatus {
  media_size_bytes: number;
  media_files_count: number;
  free_disk_bytes: number;
  total_disk_bytes: number;
}

export interface AdminBackupStatus {
  last_backup_at: string | null;
  last_backup_status: string;
  total_backups: number;
}

export interface AdminJobsStatus {
  queue_length: number;
  active_workers: number;
}

export interface AdminSystemResponse {
  app_version: string;
  database: AdminDatabaseStatus;
  storage: AdminStorageStatus;
  backups: AdminBackupStatus;
  jobs: AdminJobsStatus;
}

export interface InstanceSettingsDto {
  tree_title: string;
  default_locale: string;
  enable_public_access: boolean;
  living_person_age_threshold: number;
  default_home_person_id?: string | null;
  settings_locked_from_env: boolean;
}

export interface UpdateInstanceSettingsRequest {
  tree_title?: string;
  default_locale?: string;
  enable_public_access?: boolean;
  living_person_age_threshold?: number;
  default_home_person_id?: string | null;
}

export type UpdateInstanceSettingsInput = UpdateInstanceSettingsRequest;

export interface BackupArchiveInfo {
  filename: string;
  size_bytes: number;
  created_at: string;
  trigger: string;
  app_version: string;
  schema_version: number;
  includes_gedcom: boolean;
}

export interface BackupListResponse {
  items: BackupArchiveInfo[];
  retention_days: number;
  cron_schedule: string;
  backup_enabled: boolean;
}

export interface TriggerBackupRequest {
  include_gedcom?: boolean;
}

export interface TriggerBackupResponse {
  job_id: string;
  status: string;
  archive: BackupArchiveInfo;
}

// =============================================================================
// Phase 1.2 — Authentication, Passkeys, Recovery Codes, Sessions & Invites DTOs
// =============================================================================

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
  expires_in_seconds: number;
  preview_url?: string | null;
  preview_token?: string | null;
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

const API_BASE = '/api/v1';
const SESSION_TOKEN_STORAGE_KEY = 'genealogy_session_token';

let inMemoryAuthToken: string | null = null;

export function getAuthToken(): string | null {
  if (inMemoryAuthToken) return inMemoryAuthToken;
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      return window.localStorage.getItem(SESSION_TOKEN_STORAGE_KEY);
    }
  } catch {
    // Ignore storage access errors
  }
  return null;
}

export function setAuthToken(token: string | null): void {
  inMemoryAuthToken = token;
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      if (token) {
        window.localStorage.setItem(SESSION_TOKEN_STORAGE_KEY, token);
      } else {
        window.localStorage.removeItem(SESSION_TOKEN_STORAGE_KEY);
      }
    }
  } catch {
    // Ignore storage access errors
  }
}

export async function apiFetch<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers || {});

  // Custom header required for CSRF protection on state mutations (17 §3.2)
  headers.set('X-Requested-With', 'genealogy');
  if (!headers.has('Content-Type') && options.body && typeof options.body === 'string') {
    headers.set('Content-Type', 'application/json');
  }

  const token = getAuthToken();
  if (token && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
    credentials: 'include',
    ...options,
    headers,
  });

  if (!response.ok) {
    let errData: ApiErrorResponse;
    try {
      errData = await response.json();
    } catch {
      errData = {
        error: {
          code: 'network_error',
          message: response.statusText || 'Unknown network error',
        },
      };
    }
    throw new ApiError(response.status, errData.error);
  }

  if (response.status === 204 || response.headers.get('content-length') === '0') {
    return undefined as T;
  }

  return response.json();
}

export async function fetchHealth(): Promise<HealthResponse> {
  return apiFetch<HealthResponse>('/health');
}

export async function fetchSystemInfo(): Promise<SystemInfoResponse> {
  return apiFetch<SystemInfoResponse>('/system/info');
}

export async function fetchAdminSystem(): Promise<AdminSystemResponse> {
  return apiFetch<AdminSystemResponse>('/admin/system');
}

export async function fetchAdminSettings(): Promise<InstanceSettingsDto> {
  return apiFetch<InstanceSettingsDto>('/admin/settings');
}

export async function updateAdminSettings(
  payload: UpdateInstanceSettingsRequest
): Promise<InstanceSettingsDto> {
  return apiFetch<InstanceSettingsDto>('/admin/settings', {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function fetchBackups(): Promise<BackupListResponse> {
  return apiFetch<BackupListResponse>('/admin/backups');
}

export async function triggerBackup(
  payload: TriggerBackupRequest = {}
): Promise<TriggerBackupResponse> {
  return apiFetch<TriggerBackupResponse>('/admin/backups', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export function getBackupDownloadUrl(filename: string): string {
  return `${API_BASE}/admin/backups/${encodeURIComponent(filename)}`;
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

// =============================================================================
// Phase 1.4 Core Data Model: Persons, Families, Life Events & Places Endpoints
// =============================================================================

export * from './openapi-types';
import {
  CreateFamilyInput,
  CreateLifeEventInput,
  CreateNoteInput,
  CreatePersonInput,
  CreatePlaceInput,
  DeleteImpactDto,
  FamilyDto,
  IndividualDto,
  LifeEventDto,
  MergePlaceResponse,
  NoteDto,
  OnboardingStatusResponse,
  PatronymicSuggestionDto,
  PersonDetailDto,
  PersonProfileResponseDto,
  PlaceDetailDto,
  PlaceDto,
  PlacesListResponse,
  QuickAddRelativeInput,
  QuickAddRelativeResponse,
  StartWithSelfInput,
  TreeViewQuery,
  TreeViewResponse,
  UpdateFamilyInput,
  UpdateLifeEventInput,
  UpdateNoteInput,
  UpdatePersonInput,
  UpdatePlaceInput,
} from './openapi-types';

export async function fetchPersonDetail(id: string): Promise<PersonDetailDto> {
  return apiFetch<PersonDetailDto>(`/persons/${encodeURIComponent(id)}`);
}

export async function createPerson(
  payload: CreatePersonInput
): Promise<PersonDetailDto> {
  return apiFetch<PersonDetailDto>('/persons', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updatePerson(
  id: string,
  payload: UpdatePersonInput
): Promise<PersonDetailDto> {
  return apiFetch<PersonDetailDto>(`/persons/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function fetchPersonDeleteImpact(
  id: string
): Promise<DeleteImpactDto> {
  return apiFetch<DeleteImpactDto>(
    `/persons/${encodeURIComponent(id)}/delete-impact`
  );
}

export async function deletePerson(
  id: string
): Promise<{ status: string; deleted_id?: string }> {
  return apiFetch<{ status: string; deleted_id?: string }>(
    `/persons/${encodeURIComponent(id)}`,
    {
      method: 'DELETE',
    }
  );
}

export async function quickAddRelative(
  personId: string,
  payload: QuickAddRelativeInput
): Promise<QuickAddRelativeResponse> {
  return apiFetch<QuickAddRelativeResponse>(
    `/persons/${encodeURIComponent(personId)}/quick-add`,
    {
      method: 'POST',
      body: JSON.stringify(payload),
    }
  );
}

export async function suggestSlavicPatronymic(
  fatherGivenName: string,
  sex: 'M' | 'F' | 'U' = 'M'
): Promise<PatronymicSuggestionDto> {
  const params = new URLSearchParams({
    father_given_name: fatherGivenName,
    sex,
  });
  return apiFetch<PatronymicSuggestionDto>(
    `/persons/patronymic-suggest?${params.toString()}`
  );
}

export async function createLifeEvent(
  payload: CreateLifeEventInput
): Promise<LifeEventDto> {
  return apiFetch<LifeEventDto>('/events', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updateLifeEvent(
  id: string,
  payload: UpdateLifeEventInput
): Promise<LifeEventDto> {
  return apiFetch<LifeEventDto>(`/events/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function deleteLifeEvent(
  id: string
): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(`/events/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}

export async function fetchEventSources(
  query?: string
): Promise<{ items: string[] }> {
  const qs = query ? `?q=${encodeURIComponent(query)}` : '';
  return apiFetch<{ items: string[] }>(`/events/sources${qs}`);
}

export async function createFamily(
  payload: CreateFamilyInput
): Promise<FamilyDto> {
  return apiFetch<FamilyDto>('/families', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updateFamily(
  id: string,
  payload: UpdateFamilyInput
): Promise<FamilyDto> {
  return apiFetch<FamilyDto>(`/families/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function deleteFamily(id: string): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(`/families/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}

export async function fetchPlaces(params?: {
  q?: string;
  parent_id?: string;
  missing_coords?: boolean;
}): Promise<PlacesListResponse> {
  const search = new URLSearchParams();
  if (params?.q) search.set('q', params.q);
  if (params?.parent_id) search.set('parent_id', params.parent_id);
  if (params?.missing_coords) search.set('missing_coords', 'true');
  const qs = search.toString();
  return apiFetch<PlacesListResponse>(`/places${qs ? `?${qs}` : ''}`);
}

export async function fetchPlaceDetail(id: string): Promise<PlaceDetailDto> {
  return apiFetch<PlaceDetailDto>(`/places/${encodeURIComponent(id)}`);
}

export async function createPlace(
  payload: CreatePlaceInput
): Promise<PlaceDto> {
  return apiFetch<PlaceDto>('/places', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function updatePlace(
  id: string,
  payload: UpdatePlaceInput
): Promise<PlaceDto> {
  return apiFetch<PlaceDto>(`/places/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function deletePlace(id: string): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(`/places/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}

export async function mergePlace(
  targetId: string,
  sourcePlaceId: string
): Promise<MergePlaceResponse> {
  return apiFetch<MergePlaceResponse>(
    `/places/${encodeURIComponent(targetId)}/merge`,
    {
      method: 'POST',
      body: JSON.stringify({ other_id: sourcePlaceId, source_place_id: sourcePlaceId }),
    }
  );
}

// =============================================================================
// Phase 1.5 — Empty-Tree Onboarding & Home Person Endpoints
// =============================================================================

export async function fetchOnboardingStatus(): Promise<OnboardingStatusResponse> {
  return apiFetch<OnboardingStatusResponse>('/onboarding/status');
}

export async function startWithSelf(
  data: StartWithSelfInput
): Promise<{ individual: IndividualDto; user: UserDto }> {
  return apiFetch<{ individual: IndividualDto; user: UserDto }>(
    '/onboarding/start-with-self',
    {
      method: 'POST',
      body: JSON.stringify(data),
    }
  );
}

export async function setHomePerson(
  personId: string | null
): Promise<UserDto> {
  return apiFetch<UserDto>('/me/home-person', {
    method: 'PUT',
    body: JSON.stringify({ person_id: personId }),
  });
}

export async function updateProfile(
  data: Partial<UserDto> | UpdateProfileRequest
): Promise<UserDto> {
  return apiFetch<UserDto>('/me', {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

// =============================================================================
// Phase 1.6 — Interactive Tree Canvas Endpoints
// =============================================================================

export async function fetchTreeView(
  params?: TreeViewQuery
): Promise<TreeViewResponse> {
  const search = new URLSearchParams();
  if (params?.focal_id) search.set('focal_id', params.focal_id);
  if (params?.mode) search.set('mode', params.mode);
  if (params?.generations !== undefined) {
    search.set('generations', String(params.generations));
  }
  const qs = search.toString();
  return apiFetch<TreeViewResponse>(`/tree/view${qs ? `?${qs}` : ''}`);
}

// =============================================================================
// Phase 1.7 — Person Profile Page & Notes Endpoints
// =============================================================================

export async function fetchPersonProfile(
  id: string,
  include?: string
): Promise<PersonProfileResponseDto> {
  const qs = include ? `?include=${encodeURIComponent(include)}` : '';
  return apiFetch<PersonProfileResponseDto>(
    `/persons/${encodeURIComponent(id)}/profile${qs}`
  );
}

export async function fetchNotes(params?: {
  entity_type?: string;
  entity_id?: string;
}): Promise<{ items: NoteDto[]; total: number }> {
  const search = new URLSearchParams();
  if (params?.entity_type) search.set('entity_type', params.entity_type);
  if (params?.entity_id) search.set('entity_id', params.entity_id);
  const qs = search.toString();
  return apiFetch<{ items: NoteDto[]; total: number }>(
    `/notes${qs ? `?${qs}` : ''}`
  );
}

export async function createNote(data: CreateNoteInput): Promise<NoteDto> {
  return apiFetch<NoteDto>('/notes', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateNote(
  id: string,
  data: UpdateNoteInput
): Promise<NoteDto> {
  return apiFetch<NoteDto>(`/notes/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function deleteNote(id: string): Promise<void> {
  return apiFetch<void>(`/notes/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}





