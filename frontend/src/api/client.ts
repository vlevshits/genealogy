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

  let response: Response;
  try {
    response = await fetch(`${API_BASE}${endpoint}`, {
      credentials: 'include',
      ...options,
      headers,
    });
  } catch {
    throw new ApiError(0, {
      code: 'network_error',
      message: 'Network request failed',
    });
  }

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
  CreateFaceTagInput,
  CreateFamilyInput,
  CreateLifeEventInput,
  CreateMediaInput,
  CreateMediaLinkInput,
  CreateNoteInput,
  CreatePersonInput,
  CreatePlaceInput,
  DeleteImpactDto,
  FaceTagDto,
  FamilyDto,
  GlobalSearchQuery,
  IndividualDto,
  LifeEventDto,
  MediaArchiveResponse,
  MediaAssetSummaryDto,
  MediaBundleReconciliationReport,
  MediaFilterQuery,
  MediaLinkDto,
  MergePlaceResponse,
  NoteDto,
  OnboardingStatusResponse,
  PatronymicSuggestionDto,
  PeopleDirectoryQuery,
  PeopleDirectoryResponse,
  PersonDetailDto,
  PersonProfileResponseDto,
  PlaceDetailDto,
  PlaceDto,
  PlacesListResponse,
  QuickAddRelativeInput,
  QuickAddRelativeResponse,
  SearchResultResponseDto,
  StartWithSelfInput,
  SurnamesIndexQuery,
  SurnamesIndexResponse,
  TreeViewQuery,
  TreeViewResponse,
  UpdateFaceTagInput,
  UpdateFamilyInput,
  UpdateLifeEventInput,
  UpdateMediaInput,
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
): Promise<{ status: string; deleted_id?: string; tx_id?: string } | undefined> {
  return apiFetch<{ status: string; deleted_id?: string; tx_id?: string } | undefined>(
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
): Promise<{ status: string; tx_id?: string } | undefined> {
  return apiFetch<{ status: string; tx_id?: string } | undefined>(`/events/${encodeURIComponent(id)}`, {
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
  if (params?.collateral !== undefined) {
    search.set('collateral', String(params.collateral));
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

export async function deleteNote(
  id: string
): Promise<{ status?: string; tx_id?: string } | void> {
  return apiFetch<{ status?: string; tx_id?: string } | void>(
    `/notes/${encodeURIComponent(id)}`,
    {
      method: 'DELETE',
    }
  );
}

// =============================================================================
// Phase 1.8 — Global Search (Omnibox), People Directory & Surnames Index
// =============================================================================

export async function fetchGlobalSearch(
  params?: GlobalSearchQuery
): Promise<SearchResultResponseDto> {
  const search = new URLSearchParams();
  if (params?.q !== undefined) search.set('q', params.q);
  if (params?.types) search.set('types', params.types);
  if (params?.limit !== undefined) search.set('limit', String(params.limit));
  const qs = search.toString();
  return apiFetch<SearchResultResponseDto>(`/search${qs ? `?${qs}` : ''}`);
}

export async function fetchPeopleDirectory(
  params?: PeopleDirectoryQuery
): Promise<PeopleDirectoryResponse> {
  const search = new URLSearchParams();
  if (params?.q) search.set('q', params.q);
  if (params?.surname) search.set('surname', params.surname);
  if (params?.born_from !== undefined) {
    search.set('born_from', String(params.born_from));
  }
  if (params?.born_to !== undefined) {
    search.set('born_to', String(params.born_to));
  }
  if (params?.sex) search.set('sex', params.sex);
  if (params?.living) search.set('living', params.living);
  if (params?.has_photo !== undefined) {
    search.set('has_photo', String(params.has_photo));
  }
  if (params?.incomplete !== undefined) {
    search.set('incomplete', String(params.incomplete));
  }
  if (params?.sort) search.set('sort', params.sort);
  if (params?.cursor) search.set('cursor', params.cursor);
  if (params?.limit !== undefined) search.set('limit', String(params.limit));
  const qs = search.toString();
  return apiFetch<PeopleDirectoryResponse>(`/people${qs ? `?${qs}` : ''}`);
}

export async function fetchSurnamesIndex(
  params?: SurnamesIndexQuery
): Promise<SurnamesIndexResponse> {
  const search = new URLSearchParams();
  if (params?.letter) search.set('letter', params.letter);
  if (params?.min_count !== undefined) {
    search.set('min_count', String(params.min_count));
  }
  const qs = search.toString();
  return apiFetch<SurnamesIndexResponse>(
    `/people/surnames${qs ? `?${qs}` : ''}`
  );
}

// =============================================================================
// Phase 1.9 — Media Upload, Gallery, Face Tagging & Avatars Endpoints
// =============================================================================

export async function fetchMediaArchive(
  params?: MediaFilterQuery
): Promise<MediaArchiveResponse> {
  const search = new URLSearchParams();
  if (params?.q) search.set('q', params.q);
  if (params?.person_id) search.set('person_id', params.person_id);
  if (params?.decade) search.set('decade', params.decade);
  if (params?.type) search.set('type', params.type);
  if (params?.untagged !== undefined) {
    search.set('untagged', String(params.untagged));
  }
  if (params?.is_missing !== undefined) {
    search.set('is_missing', String(params.is_missing));
  }
  if (params?.cursor) search.set('cursor', params.cursor);
  if (params?.limit !== undefined) search.set('limit', String(params.limit));
  const qs = search.toString();
  return apiFetch<MediaArchiveResponse>(`/media${qs ? `?${qs}` : ''}`);
}

export async function createMediaAsset(
  payload: CreateMediaInput
): Promise<MediaAssetSummaryDto> {
  return apiFetch<MediaAssetSummaryDto>('/media', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function fetchMediaDetail(
  id: string
): Promise<MediaAssetSummaryDto> {
  return apiFetch<MediaAssetSummaryDto>(`/media/${encodeURIComponent(id)}`);
}

export async function updateMediaAsset(
  id: string,
  payload: UpdateMediaInput
): Promise<MediaAssetSummaryDto> {
  return apiFetch<MediaAssetSummaryDto>(`/media/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export async function deleteMediaAsset(id: string): Promise<void> {
  return apiFetch<void>(`/media/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
}

export function getMediaFileUrl(id: string): string {
  return `${API_BASE}/media/${encodeURIComponent(id)}/file`;
}

export function getMediaCropUrl(
  id: string,
  crop: { x: number; y: number; width: number; height: number }
): string {
  const params = new URLSearchParams({
    x: crop.x.toFixed(4),
    y: crop.y.toFixed(4),
    w: crop.width.toFixed(4),
    h: crop.height.toFixed(4),
  });
  return `${API_BASE}/media/${encodeURIComponent(id)}/crop?${params.toString()}`;
}

export async function createFaceTag(
  mediaId: string,
  payload: CreateFaceTagInput
): Promise<FaceTagDto> {
  return apiFetch<FaceTagDto>(
    `/media/${encodeURIComponent(mediaId)}/tags`,
    {
      method: 'POST',
      body: JSON.stringify(payload),
    }
  );
}

export async function updateFaceTag(
  mediaId: string,
  tagId: string,
  payload: UpdateFaceTagInput
): Promise<FaceTagDto> {
  return apiFetch<FaceTagDto>(
    `/media/${encodeURIComponent(mediaId)}/tags/${encodeURIComponent(tagId)}`,
    {
      method: 'PATCH',
      body: JSON.stringify(payload),
    }
  );
}

export async function deleteFaceTag(
  mediaId: string,
  tagId: string
): Promise<void> {
  return apiFetch<void>(
    `/media/${encodeURIComponent(mediaId)}/tags/${encodeURIComponent(tagId)}`,
    {
      method: 'DELETE',
    }
  );
}

export async function createMediaLink(
  mediaId: string,
  payload: CreateMediaLinkInput
): Promise<MediaLinkDto> {
  return apiFetch<MediaLinkDto>(
    `/media/${encodeURIComponent(mediaId)}/links`,
    {
      method: 'POST',
      body: JSON.stringify(payload),
    }
  );
}

export async function deleteMediaLink(
  mediaId: string,
  linkId: string
): Promise<void> {
  return apiFetch<void>(
    `/media/${encodeURIComponent(mediaId)}/links/${encodeURIComponent(linkId)}`,
    {
      method: 'DELETE',
    }
  );
}

export type { MediaBundleReconciliationReport };

export interface UploadProgress {
  loaded: number;
  total: number;
  percent: number;
}

export interface MediaBundleUploadOptions {
  ingest_unmatched?: boolean;
  onProgress?: (progress: UploadProgress) => void;
  signal?: AbortSignal;
}

export function uploadFileWithProgress<T>(
  endpoint: string,
  formData: FormData,
  options?: {
    onProgress?: (progress: UploadProgress) => void;
    signal?: AbortSignal;
  }
): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    if (typeof XMLHttpRequest === 'undefined') {
      const headers = new Headers();
      headers.set('X-Requested-With', 'genealogy');
      const token = getAuthToken();
      if (token) headers.set('Authorization', `Bearer ${token}`);
      fetch(`${API_BASE}${endpoint}`, {
        method: 'POST',
        body: formData,
        headers,
        credentials: 'include',
        signal: options?.signal,
      })
        .then(async (res) => {
          if (!res.ok) {
            let errData: ApiErrorResponse;
            try {
              errData = await res.json();
            } catch {
              errData = {
                error: {
                  code: 'network_error',
                  message: res.statusText || 'Upload failed',
                },
              };
            }
            throw new ApiError(res.status, errData.error);
          }
          return res.json() as Promise<T>;
        })
        .then(resolve)
        .catch((err) => {
          if (err instanceof ApiError) {
            reject(err);
          } else {
            reject(
              new ApiError(0, {
                code: 'network_error',
                message: err instanceof Error ? err.message : 'Network request failed',
              })
            );
          }
        });
      return;
    }

    const xhr = new XMLHttpRequest();
    xhr.open('POST', `${API_BASE}${endpoint}`);
    xhr.withCredentials = true;
    xhr.setRequestHeader('X-Requested-With', 'genealogy');

    const token = getAuthToken();
    if (token) {
      xhr.setRequestHeader('Authorization', `Bearer ${token}`);
    }

    if (options?.signal) {
      if (options.signal.aborted) {
        xhr.abort();
        reject(new ApiError(0, { code: 'aborted', message: 'Upload aborted' }));
        return;
      }
      options.signal.addEventListener('abort', () => {
        xhr.abort();
        reject(new ApiError(0, { code: 'aborted', message: 'Upload aborted' }));
      });
    }

    if (xhr.upload && options?.onProgress) {
      xhr.upload.onprogress = (event) => {
        if (event.lengthComputable) {
          const percent = Math.min(100, Math.round((event.loaded / event.total) * 100));
          options.onProgress?.({
            loaded: event.loaded,
            total: event.total,
            percent,
          });
        }
      };
    }

    xhr.onload = () => {
      let responseData: (Record<string, unknown> & { error?: ApiErrorDetail; code?: string; message?: string; details?: Record<string, unknown>; field_errors?: Record<string, string> }) | null = null;
      try {
        responseData = xhr.responseText ? JSON.parse(xhr.responseText) : null;
      } catch {
        responseData = null;
      }

      if (xhr.status >= 200 && xhr.status < 300) {
        resolve(responseData as T);
      } else {
        const errorDetail: ApiErrorDetail = responseData?.error || {
          code: responseData?.code || 'network_error',
          message: responseData?.message || xhr.statusText || 'Upload failed',
          details: responseData?.details,
          field_errors: responseData?.field_errors,
        };
        reject(new ApiError(xhr.status, errorDetail));
      }
    };

    xhr.onerror = () => {
      reject(
        new ApiError(0, {
          code: 'network_error',
          message: 'Network request failed',
        })
      );
    };

    xhr.ontimeout = () => {
      reject(
        new ApiError(0, {
          code: 'timeout',
          message: 'Upload timed out',
        })
      );
    };

    xhr.send(formData);
  });
}

export async function uploadGedcomMediaBundle(
  file: File,
  options?: MediaBundleUploadOptions
): Promise<MediaBundleReconciliationReport> {
  const formData = new FormData();
  formData.append('bundle', file, file.name);
  const ingest = options?.ingest_unmatched ?? false;
  return uploadFileWithProgress<MediaBundleReconciliationReport>(
    `/gedcom/media-bundle?ingest_unmatched=${ingest}`,
    formData,
    options
  );
}

export async function uploadMediaBundle(
  file: File,
  options?: MediaBundleUploadOptions
): Promise<MediaBundleReconciliationReport> {
  const formData = new FormData();
  formData.append('bundle', file, file.name);
  const ingest = options?.ingest_unmatched ?? true;
  return uploadFileWithProgress<MediaBundleReconciliationReport>(
    `/media/bundle?ingest_unmatched=${ingest}`,
    formData,
    options
  );
}

// =============================================================================
// Phase 1.10 — GEDCOM 5.5.1 + 7.0 Import & Export Endpoints
// =============================================================================

import type {
  AnalyzeGedcomInput,
  GedcomAnalysisReport,
  GedcomDialectSummary,
  GedcomExportOptions,
  GedcomImportMode,
  GedcomImportResult,
  GedcomVersionOption,
  ImportGedcomInput,
} from './openapi-types';

export type {
  AnalyzeGedcomInput,
  GedcomAnalysisReport,
  GedcomDialectSummary,
  GedcomExportOptions,
  GedcomImportMode,
  GedcomImportResult,
  GedcomVersionOption,
  ImportGedcomInput,
};

export async function analyzeGedcom(
  payload: AnalyzeGedcomInput
): Promise<GedcomAnalysisReport> {
  return apiFetch<GedcomAnalysisReport>('/gedcom/analyze', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function importGedcom(
  payload: ImportGedcomInput
): Promise<GedcomImportResult> {
  return apiFetch<GedcomImportResult>('/gedcom/import', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export function getGedcomExportUrl(options: GedcomExportOptions): string {
  const params = new URLSearchParams({
    version: options.version,
    redact_living: options.redact_living ? 'true' : 'false',
    include_media: options.include_media ? 'true' : 'false',
    include_private_notes: options.include_private_notes ? 'true' : 'false',
  });
  return `${API_BASE}/gedcom/export?${params.toString()}`;
}

export async function exportGedcom(
  options: GedcomExportOptions
): Promise<{ blob: Blob; filename: string }> {
  const headers = new Headers();
  headers.set('X-Requested-With', 'genealogy');
  headers.set('Content-Type', 'application/json');

  const token = getAuthToken();
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  let response: Response;
  try {
    response = await fetch(`${API_BASE}/gedcom/export`, {
      method: 'POST',
      credentials: 'include',
      headers,
      body: JSON.stringify(options),
    });
  } catch {
    throw new ApiError(0, {
      code: 'network_error',
      message: 'Network request failed',
    });
  }

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

  const contentDisposition = response.headers.get('Content-Disposition') || '';
  const match = contentDisposition.match(/filename="?([^";]+)"?/i);
  const ext = options.include_media ? 'zip' : 'ged';
  const fallbackName = `family-tree-${options.version}.${ext}`;
  const filename = match?.[1] || fallbackName;
  const blob = await response.blob();
  return { blob, filename };
}

// =============================================================================
// Phase 1.11 — Change History, Undo Toast, Restore Deleted, Optimistic Concurrency & Tree Snapshots
// =============================================================================

export interface FieldDiffDto {
  field: string;
  before: unknown;
  after: unknown;
}

export type ChangeActionOp =
  | 'create'
  | 'update'
  | 'delete'
  | 'restore'
  | 'undo'
  | 'created'
  | 'updated'
  | 'deleted'
  | 'restored'
  | 'undone'
  | string;

export interface ChangeLogEntryDto {
  id: string | number;
  tx_id: string;
  tx_label: string;
  entity_type: string;
  entity_id: string;
  action?: ChangeActionOp;
  op: ChangeActionOp;
  entity_label?: string | null;
  actor_id?: string | null;
  actor_name?: string | null;
  created_at: string;
  diffs?: FieldDiffDto[];
  before_json?: Record<string, unknown> | null;
  after_json?: Record<string, unknown> | null;
}

export interface ChangeLogTransactionGroupDto {
  tx_id: string;
  tx_label?: string;
  actor_id?: string | null;
  actor_name?: string | null;
  created_at: string;
  can_undo: boolean;
  entries?: ChangeLogEntryDto[];
  changes?: ChangeLogEntryDto[];
}

export interface HistoryFeedQueryParams {
  user_id?: string;
  entity_type?: string;
  op?: string;
  only_deletions?: boolean;
  from_date?: string;
  to_date?: string;
  before?: string;
  limit?: number;
  page?: number;
}

export interface HistoryFeedResponseDto {
  items: ChangeLogTransactionGroupDto[] | ChangeLogEntryDto[];
  transactions?: ChangeLogTransactionGroupDto[];
  total: number;
  has_more?: boolean;
}

export interface EntityHistoryResponseDto {
  entity_type: string;
  entity_id: string;
  items: ChangeLogEntryDto[];
  total: number;
}

export interface UndoTransactionResponseDto {
  tx_id: string;
  reverted_count: number;
  status: string;
}

export interface RestoreChangeResponseDto {
  change_id: string | number;
  restored_entity_type: string;
  restored_entity_id: string;
  skipped_links?: string[];
  status: string;
}

export interface TreeSnapshotRecordCountsDto {
  individuals: number;
  families: number;
  events: number;
  places: number;
  media: number;
  notes: number;
}

export interface TreeSnapshotDto {
  id: string;
  label?: string | null;
  reason: string;
  record_counts: TreeSnapshotRecordCountsDto;
  size_bytes: number;
  created_at: string;
  expires_at: string;
}

export interface CreateSnapshotInput {
  label?: string;
  reason?: string;
}

export interface RollbackSnapshotResponseDto {
  restored_snapshot_id: string;
  pre_rollback_snapshot_id: string;
  status: string;
}

export interface EntityVersionDto {
  id: string;
  entity_type: string;
  version: number;
  updated_at?: string;
  updated_by_id?: string | null;
  updated_by_name?: string | null;
}

export async function getHistory(
  params: HistoryFeedQueryParams = {}
): Promise<HistoryFeedResponseDto> {
  const query = new URLSearchParams();
  if (params.user_id) query.set('user_id', params.user_id);
  if (params.entity_type && params.entity_type !== 'all') {
    query.set('entity_type', params.entity_type);
  }
  if (params.op && params.op !== 'all') {
    query.set('op', params.op);
  }
  if (params.only_deletions) {
    query.set('only_deletions', 'true');
  }
  if (params.from_date) query.set('from_date', params.from_date);
  if (params.to_date) query.set('to_date', params.to_date);
  if (params.before) query.set('before', params.before);
  if (params.limit !== undefined) query.set('limit', String(params.limit));
  if (params.page !== undefined) query.set('page', String(params.page));

  const qs = query.toString();
  const res = await apiFetch<
    HistoryFeedResponseDto | ChangeLogTransactionGroupDto[]
  >(`/history${qs ? `?${qs}` : ''}`);

  if (Array.isArray(res)) {
    return { items: res, total: res.length, has_more: false };
  }
  return res;
}

export async function getEntityHistory(
  entityType: string,
  entityId: string,
  page?: number
): Promise<EntityHistoryResponseDto> {
  const qs = page !== undefined ? `?page=${page}` : '';
  const res = await apiFetch<EntityHistoryResponseDto | ChangeLogEntryDto[]>(
    `/history/${encodeURIComponent(entityType)}/${encodeURIComponent(
      entityId
    )}${qs}`
  );
  if (Array.isArray(res)) {
    return {
      entity_type: entityType,
      entity_id: entityId,
      items: res,
      total: res.length,
    };
  }
  return res;
}

export async function undoTransaction(
  txId: string
): Promise<UndoTransactionResponseDto> {
  return apiFetch<UndoTransactionResponseDto>(
    `/history/tx/${encodeURIComponent(txId)}/undo`,
    {
      method: 'POST',
    }
  );
}

export async function restoreChange(
  changeId: string | number
): Promise<RestoreChangeResponseDto> {
  return apiFetch<RestoreChangeResponseDto>(
    `/history/${encodeURIComponent(String(changeId))}/restore`,
    {
      method: 'POST',
    }
  );
}

export async function getSnapshots(): Promise<TreeSnapshotDto[]> {
  const res = await apiFetch<
    TreeSnapshotDto[] | { items: TreeSnapshotDto[] }
  >('/snapshots');
  if (Array.isArray(res)) return res;
  return res.items ?? [];
}

export async function createSnapshot(
  body: CreateSnapshotInput = {}
): Promise<TreeSnapshotDto> {
  return apiFetch<TreeSnapshotDto>('/snapshots', {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export async function rollbackSnapshot(
  id: string,
  confirmText: string
): Promise<RollbackSnapshotResponseDto> {
  return apiFetch<RollbackSnapshotResponseDto>(
    `/snapshots/${encodeURIComponent(id)}/rollback`,
    {
      method: 'POST',
      body: JSON.stringify({ confirm_text: confirmText, confirm: confirmText }),
    }
  );
}

export async function getEntityVersion(
  entityType: string,
  id: string
): Promise<EntityVersionDto> {
  return apiFetch<EntityVersionDto>(
    `/${encodeURIComponent(entityType)}/${encodeURIComponent(id)}/version`
  );
}

// =============================================================================
// Phase 2 ("Delight") — Dashboard, Data Quality, Person Merge, Fan Chart,
// Kinship Path Finder & Clustered Places Map Endpoints
// =============================================================================

export type AnniversaryType = 'birthday' | 'memorial' | 'wedding';

export interface AnniversaryItemDto {
  id?: string;
  type?: AnniversaryType;
  kind?: AnniversaryType | string;
  person_id?: string | null;
  person?: {
    id: string;
    gid?: string;
    display_name: string;
    is_living?: boolean;
  } | null;
  partner_person_id?: string | null;
  family_id?: string | null;
  display_name?: string;
  secondary_name?: string | null;
  is_living?: boolean;
  date_month_day?: string;
  date_label?: string;
  next_occurrence_date?: string;
  years_count?: number | null;
  years_elapsed?: number | null;
  birth_year?: number | null;
  death_year?: number | null;
  days_until: number;
}

export interface TreeGlanceStatsDto {
  total_persons?: number;
  persons_count?: number;
  individuals_count?: number;
  total_families?: number;
  families_count?: number;
  total_photos?: number;
  media_count?: number;
  photos_count?: number;
  total_places?: number;
  places_count?: number;
  max_generations?: number;
  generations_count?: number;
  oldest_birth_year?: number | null;
  earliest_birth_year?: number | null;
  oldest_person_name?: string | null;
  oldest_ancestor?:
  | {
    id?: string;
    gid?: string;
    display_name?: string;
    birth_year?: number | null;
  }
  | string
  | null;
  open_warnings_count?: number;
  missing_birth_dates_count?: number;
  missing_birth_count?: number;
  quality_summary?: {
    total_open_warnings?: number;
  };
}

export interface DashboardPayloadDto {
  anniversaries: AnniversaryItemDto[];
  upcoming_anniversaries?: AnniversaryItemDto[];
  recent_activity: ChangeLogTransactionGroupDto[];
  stats: TreeGlanceStatsDto;
  tree_stats?: TreeGlanceStatsDto;
  latest_photos: MediaAssetSummaryDto[];
  resolved_home_person_id?: string | null;
}

export type QualitySeverity = 'critical' | 'warning' | 'info';

export type QualityRuleCode =
  | 'DEATH_BEFORE_BIRTH'
  | 'TOO_OLD'
  | 'PARENT_TOO_YOUNG'
  | 'MOTHER_TOO_OLD'
  | 'BORN_AFTER_PARENT_DEATH'
  | 'MARRIED_TOO_YOUNG'
  | 'EVENT_AFTER_DEATH'
  | 'POSSIBLE_DUPLICATE'
  | 'MISSING_BIRTH'
  | 'UNCONNECTED'
  | string;

export interface QualityWarningDto {
  id?: string;
  rule_code: QualityRuleCode;
  code?: QualityRuleCode;
  severity: QualitySeverity;
  entity_type?: string;
  entity_id: string;
  person_id?: string;
  person_name?: string;
  person_gid?: string;
  person?: {
    id: string;
    gid?: string;
    display_name: string;
  } | null;
  message?: string;
  details?: Record<string, unknown>;
}

export interface DuplicateCandidatePairDto {
  id?: string;
  person_a_id?: string;
  person_a_gid?: string;
  person_a_name?: string;
  person_a_birth?: string | null;
  person_a_death?: string | null;
  person_a?: {
    id: string;
    gid?: string;
    display_name: string;
    birth_date?: string | null;
    death_date?: string | null;
  } | null;
  person_b_id?: string;
  person_b_gid?: string;
  person_b_name?: string;
  person_b_birth?: string | null;
  person_b_death?: string | null;
  person_b?: {
    id: string;
    gid?: string;
    display_name: string;
    birth_date?: string | null;
    death_date?: string | null;
  } | null;
  score: number;
  reasons?: string[];
}

export interface QualityWarningsResponseDto {
  warnings: QualityWarningDto[];
  duplicates?: DuplicateCandidatePairDto[];
  counts?: {
    total: number;
    critical: number;
    warning: number;
    info: number;
    duplicates: number;
  };
}

export interface DuplicateQuickMatchDto {
  id: string;
  gid: string;
  display_name: string;
  given_name: string;
  patronymic?: string | null;
  surname: string;
  sex: 'M' | 'F' | 'U';
  birth_date?: string | null;
  death_date?: string | null;
  is_living: boolean;
  score: number;
  person?: {
    id: string;
    gid: string;
    display_name: string;
    given_name?: string;
    patronymic?: string | null;
    surname?: string;
    sex?: 'M' | 'F' | 'U';
    birth_date?: string | null;
    death_date?: string | null;
    is_living?: boolean;
  } | null;
}

export interface MergeScalarFieldDiffDto {
  field: string;
  target_value: string | null;
  source_value: string | null;
}

export interface PersonMergePreviewDto {
  target: TreePersonNodeDto & Partial<PersonDetailDto>;
  source: TreePersonNodeDto & Partial<PersonDetailDto>;
  keep_person?: TreePersonNodeDto & Partial<PersonDetailDto> & {
    display_name?: string;
    birth_date?: string | null;
    birth_place?: string | null;
    death_date?: string | null;
    death_place?: string | null;
  };
  merge_person?: TreePersonNodeDto & Partial<PersonDetailDto> & {
    display_name?: string;
    birth_date?: string | null;
    birth_place?: string | null;
    death_date?: string | null;
    death_place?: string | null;
  };
  fields?: MergeScalarFieldDiffDto[];
  events_to_transfer?: unknown[] | number;
  media_to_transfer?: unknown[] | number;
  families_to_transfer?: unknown[] | number;
  notes_to_transfer?: unknown[] | number;
  combined_counts: {
    events: number;
    media: number;
    families: number;
    notes: number;
  };
}

export interface MergePeopleInput {
  target_id: string;
  source_id: string;
  keep_id?: string;
  merge_id?: string;
  field_choices: Record<string, 'target' | 'source' | unknown>;
}

export interface MergePeopleResponseDto {
  surviving_person_id?: string;
  surviving_person?: {
    id: string;
    gid?: string;
    display_name?: string;
  } | null;
  merged_source_id?: string;
  tx_id?: string;
  status?: string;
}

export interface FanChartNodeDto {
  ahnentafel: number;
  generation: number;
  quadrant: 'paternal_gf' | 'paternal_gm' | 'maternal_gf' | 'maternal_gm' | 'root';
  person: TreePersonNodeDto;
}

export interface FanChartResponseDto {
  root_id: string;
  generations: number;
  nodes: FanChartNodeDto[];
}

export interface KinshipStepDto {
  person_id?: string;
  from_id?: string;
  to_id?: string;
  edge_type?: string;
  gid?: string;
  display_name?: string;
  sex?: 'M' | 'F' | 'U';
  is_living?: boolean;
  relation_to_next?: string | null;
  is_mrca?: boolean;
}

export interface KinshipTraceStepDto {
  person_id: string;
  relation_to_next?: string | null;
  relation_to_next_ru?: string | null;
}

export interface KinshipTraceDto {
  id: string;
  relationship_key: string;
  label_en: string;
  label_ru: string;
  degree_of_kinship: number;
  is_blood_relation: boolean;
  common_ancestor_ids: string[];
  path_person_ids: string[];
  steps: KinshipTraceStepDto[];
}

export interface KinshipPathResponseDto {
  from_id?: string;
  to_id?: string;
  connected: boolean;
  relationship_label?: string;
  label_en?: string;
  label_ru?: string;
  relationship_key?: string;
  degree?: number;
  degree_of_kinship?: number;
  mrca_ids?: string[];
  steps: KinshipStepDto[];
  path_nodes?: Array<
    TreePersonNodeDto & {
      display_name?: string;
      given_name?: string;
      surname?: string;
    }
  >;
  traces_count?: number;
  traces?: KinshipTraceDto[];
}

export interface PlaceMapPinDto {
  id: string;
  gid: string;
  name: string;
  full_name: string;
  place_type: string;
  latitude: number;
  longitude: number;
  events_count: number;
  persons_count: number;
  sample_people?: Array<{
    id: string;
    display_name: string;
    event_type?: string;
    year?: number | null;
  }>;
}

export interface PlacesMapResponseDto {
  items: PlaceMapPinDto[];
  pins?: PlaceMapPinDto[];
  map_tile_url?: string;
  map_attribution?: string;
}

export const dashboardApi = {
  async getDashboard(
    scope: 'all' | 'close' = 'all'
  ): Promise<DashboardPayloadDto> {
    return apiFetch<DashboardPayloadDto>(
      `/dashboard?scope=${encodeURIComponent(scope)}`
    );
  },
};

export const qualityApi = {
  async getWarnings(params?: {
    severity?: string;
    code?: string;
    cursor?: string;
  }): Promise<QualityWarningsResponseDto> {
    const search = new URLSearchParams();
    if (params?.severity && params.severity !== 'all') {
      search.set('severity', params.severity);
    }
    if (params?.code) search.set('code', params.code);
    if (params?.cursor) search.set('cursor', params.cursor);
    const qs = search.toString();
    const res = await apiFetch<
      QualityWarningsResponseDto | { items: QualityWarningDto[] }
    >(`/quality/warnings${qs ? `?${qs}` : ''}`);
    if ('warnings' in res) return res;
    return {
      warnings: res.items ?? [],
    };
  },

  async getDuplicates(): Promise<DuplicateCandidatePairDto[]> {
    const res = await apiFetch<
      DuplicateCandidatePairDto[] | { items: DuplicateCandidatePairDto[] }
    >('/quality/duplicates');
    if (Array.isArray(res)) return res;
    return res.items ?? [];
  },

  async dismissWarning(
    ruleCode: string,
    entityId: string
  ): Promise<{ status: string }> {
    return apiFetch<{ status: string }>('/quality/warnings/dismiss', {
      method: 'POST',
      body: JSON.stringify({
        rule_code: ruleCode,
        code: ruleCode,
        entity_id: entityId,
      }),
    });
  },
};

export const peopleApi = {
  async findDuplicates(
    name: string,
    birthYear?: number | string | null,
    gender?: string | null
  ): Promise<DuplicateQuickMatchDto[]> {
    const search = new URLSearchParams();
    const trimmed = name.trim();
    const parts = trimmed.split(/\s+/);
    const given = parts[0] || '';
    const surname = parts.length > 1 ? parts[parts.length - 1] : '';
    search.set('q', trimmed);
    if (given) search.set('given', given);
    if (surname) search.set('surname', surname);
    if (birthYear !== undefined && birthYear !== null && String(birthYear).trim()) {
      search.set('birth', String(birthYear).trim());
      search.set('birth_year', String(birthYear).trim());
    }
    if (gender) search.set('sex', gender);
    const res = await apiFetch<
      DuplicateQuickMatchDto[] | { items: DuplicateQuickMatchDto[] }
    >(`/people/match?${search.toString()}`);
    const rawItems = Array.isArray(res) ? res : (res.items ?? []);
    return rawItems.map((item) => ({
      ...item,
      id: item.person?.id ?? item.id,
      gid: item.person?.gid ?? item.gid,
      display_name: item.person?.display_name ?? item.display_name,
      given_name: item.person?.given_name ?? item.given_name ?? '',
      patronymic: item.person?.patronymic ?? item.patronymic ?? null,
      surname: item.person?.surname ?? item.surname ?? '',
      sex: item.person?.sex ?? item.sex ?? 'U',
      birth_date: item.person?.birth_date ?? item.birth_date ?? null,
      death_date: item.person?.death_date ?? item.death_date ?? null,
      is_living: item.person?.is_living ?? item.is_living ?? false,
      score: item.score,
    }));
  },

  async getMergePreview(
    targetId: string,
    sourceId: string
  ): Promise<PersonMergePreviewDto> {
    const search = new URLSearchParams({
      target: targetId,
      source: sourceId,
      keep_id: targetId,
      merge_id: sourceId,
      a: targetId,
      b: sourceId,
    });
    return apiFetch<PersonMergePreviewDto>(
      `/people/merge/preview?${search.toString()}`
    );
  },

  async merge(body: MergePeopleInput): Promise<MergePeopleResponseDto> {
    return apiFetch<MergePeopleResponseDto>('/people/merge', {
      method: 'POST',
      body: JSON.stringify({
        ...body,
        keep_id: body.keep_id ?? body.target_id,
        merge_id: body.merge_id ?? body.source_id,
      }),
    });
  },

  async markNotDuplicate(
    personA: string,
    personB: string
  ): Promise<{ status: string }> {
    return apiFetch<{ status: string }>('/quality/duplicates/not-duplicate', {
      method: 'POST',
      body: JSON.stringify({
        person_a: personA,
        person_b: personB,
        person_a_id: personA,
        person_b_id: personB,
      }),
    });
  },
};

export const treeApi = {
  async getFanChart(
    rootId?: string | null,
    generations: number = 5
  ): Promise<FanChartResponseDto> {
    const search = new URLSearchParams({
      mode: 'fan',
      generations: String(generations),
    });
    if (rootId) {
      search.set('root_id', rootId);
      search.set('focal_id', rootId);
    }
    return apiFetch<FanChartResponseDto>(`/tree/fan?${search.toString()}`);
  },
};

export async function fetchKinshipPath(
  fromId: string,
  toId: string,
  maxTraces?: number
): Promise<KinshipPathResponseDto> {
  const search = new URLSearchParams({
    from: fromId,
    to: toId,
    a: fromId,
    b: toId,
  });
  if (maxTraces !== undefined && maxTraces !== null) {
    search.set('max_traces', String(maxTraces));
  }
  return apiFetch<KinshipPathResponseDto>(`/kinship?${search.toString()}`);
}

export const kinshipApi = {
  async getKinship(
    fromId: string,
    toId: string,
    maxTraces?: number
  ): Promise<KinshipPathResponseDto> {
    return fetchKinshipPath(fromId, toId, maxTraces);
  },
};

export const placesApi = {
  async getPlacesMap(): Promise<PlacesMapResponseDto> {
    return apiFetch<PlacesMapResponseDto>('/places/map');
  },
};









