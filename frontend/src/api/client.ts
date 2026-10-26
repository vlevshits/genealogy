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

const API_BASE = '/api/v1';

export async function apiFetch<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers || {});
  
  // Custom header required for CSRF protection on state mutations
  headers.set('X-Requested-With', 'genealogy');
  if (!headers.has('Content-Type') && options.body && typeof options.body === 'string') {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
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

  return response.json();
}

export async function fetchHealth(): Promise<HealthResponse> {
  return apiFetch<HealthResponse>('/health');
}
