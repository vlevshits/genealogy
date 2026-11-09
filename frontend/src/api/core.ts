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
  ai_enabled?: boolean;
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


export const API_BASE = '/api/v1';
export const TREE_DATA_UPDATED_EVENT = 'genealogy:tree-updated';
const SESSION_TOKEN_STORAGE_KEY = 'genealogy_session_token';

export function notifyTreeDataChanged(): void {
  if (typeof window !== 'undefined' && typeof window.dispatchEvent === 'function') {
    window.dispatchEvent(new CustomEvent(TREE_DATA_UPDATED_EVENT));
  }
}

function isNonTreeMutatingEndpoint(endpoint: string): boolean {
  return (
    endpoint.startsWith('/auth/') ||
    endpoint.startsWith('/gedcom/analyze') ||
    endpoint.startsWith('/export/pdf/preview-info') ||
    endpoint.startsWith('/ai/transcribe') ||
    endpoint.startsWith('/ai/parse-story')
  );
}

let inMemoryAuthToken: string | null = null;

function cleanToken(token: string | null | undefined): string | null {
  if (typeof token !== 'string') return null;
  const trimmed = token.trim();
  if (!trimmed || trimmed === 'null' || trimmed === 'undefined') {
    return null;
  }
  return trimmed;
}

function getStorage(): Storage | null {
  try {
    if (typeof window !== 'undefined' && window.localStorage) {
      return window.localStorage;
    }
    if (typeof localStorage !== 'undefined') {
      return localStorage;
    }
  } catch {
    // Ignore storage access errors
  }
  return null;
}

export function getAuthToken(): string | null {
  if (inMemoryAuthToken !== null) {
    const cleaned = cleanToken(inMemoryAuthToken);
    if (!cleaned) {
      inMemoryAuthToken = null;
      try {
        getStorage()?.removeItem(SESSION_TOKEN_STORAGE_KEY);
      } catch {
        // Ignore storage access errors
      }
      return null;
    }
    return cleaned;
  }

  const storage = getStorage();
  if (storage) {
    try {
      const stored = storage.getItem(SESSION_TOKEN_STORAGE_KEY);
      if (stored !== null) {
        const cleaned = cleanToken(stored);
        if (!cleaned) {
          storage.removeItem(SESSION_TOKEN_STORAGE_KEY);
          return null;
        }
        inMemoryAuthToken = cleaned;
        return cleaned;
      }
    } catch {
      // Ignore storage access errors
    }
  }
  return null;
}

export function setAuthToken(token: string | null): void {
  const cleaned = cleanToken(token);
  inMemoryAuthToken = cleaned;
  const storage = getStorage();
  if (storage) {
    try {
      if (cleaned) {
        storage.setItem(SESSION_TOKEN_STORAGE_KEY, cleaned);
      } else {
        storage.removeItem(SESSION_TOKEN_STORAGE_KEY);
      }
    } catch {
      // Ignore storage access errors
    }
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
  } catch (err) {
    if (
      (typeof DOMException !== 'undefined' &&
        err instanceof DOMException &&
        err.name === 'AbortError') ||
      (err instanceof Error && err.name === 'AbortError')
    ) {
      throw err;
    }
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
    if (
      response.status === 401 ||
      errData.error?.code === 'not_authenticated' ||
      errData.error?.code === 'session_expired'
    ) {
      setAuthToken(null);
    }
    throw new ApiError(response.status, errData.error);
  }

  const method = (options.method || 'GET').toUpperCase();
  const shouldNotifyTreeChange =
    (method === 'POST' ||
      method === 'PUT' ||
      method === 'PATCH' ||
      method === 'DELETE') &&
    !isNonTreeMutatingEndpoint(endpoint);

  if (response.status === 204 || response.headers.get('content-length') === '0') {
    if (shouldNotifyTreeChange) {
      notifyTreeDataChanged();
    }
    return undefined as T;
  }

  const data = (await response.json()) as T;
  if (shouldNotifyTreeChange) {
    notifyTreeDataChanged();
  }
  return data;
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

