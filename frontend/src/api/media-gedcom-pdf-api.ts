import { API_BASE, ApiError, type ApiErrorDetail, type ApiErrorResponse, apiFetch, getAuthToken } from "./core";
import type {
  CreateFaceTagInput,
  CreateMediaInput,
  CreateMediaLinkInput,
  FaceTagDto,
  MediaArchiveResponse,
  MediaAssetSummaryDto,
  MediaBundleReconciliationReport,
  MediaFilterQuery,
  MediaLinkDto,
  UpdateFaceTagInput,
  UpdateMediaInput,
} from "./openapi-types";

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
// Feature 05 — Printable PDF Tree Export (Single & Multi-Page Tiled)
// =============================================================================

export interface PdfExportRequest {
  focal_person_id?: string;
  view_mode?: string;
  generations?: number;
  theme?: string;
  format?: string;
  target_sheet_size?: string;
  overlap_mm?: number;
  show_crop_marks?: boolean;
  show_photos?: boolean;
  redact_living?: boolean;
  title?: string;
  subtitle?: string;
  include_partner?: boolean;
  include_siblings?: boolean;
  collateral?: boolean;
  include_assembly_guide?: boolean;
}

export interface PdfPreviewCardDto {
  id: string;
  gid: string;
  full_name: string;
  lifespan: string;
  is_living: boolean;
  is_focal: boolean;
  sex: string;
  x: number;
  y: number;
  width: number;
  height: number;
  generation: number;
  has_avatar: boolean;
}

export interface PdfPreviewConnectorDto {
  points: [number, number][];
  is_dashed: boolean;
  color?: string;
}

export interface PdfPosterLayoutDto {
  sheet_width_mm: number;
  sheet_height_mm: number;
  scale: number;
  offset_x: number;
  offset_y: number;
  frame_margin: number;
}

export interface PdfTileSheetDto {
  sheet_index: number;
  row: number;
  col: number;
  canvas_window_x: number;
  canvas_window_y: number;
  window_width: number;
  window_height: number;
  has_cut_right: boolean;
  has_cut_bottom: boolean;
  label: string;
}

export interface PdfExportPreviewInfo {
  total_sheets: number;
  grid_rows: number;
  grid_cols: number;
  canvas_width_mm: number;
  canvas_height_mm: number;
  sheet_width_mm: number;
  sheet_height_mm: number;
  overlap_mm: number;
  format: string;
  target_sheet_size: string;
  total_persons: number;
  summary_text: string;
  title?: string;
  subtitle?: string;
  cards?: PdfPreviewCardDto[];
  connectors?: PdfPreviewConnectorDto[];
  poster_layout?: PdfPosterLayoutDto;
  tiles?: PdfTileSheetDto[];
}

export async function exportPrintablePdf(
  params: PdfExportRequest
): Promise<Blob> {
  const headers = new Headers();
  headers.set('X-Requested-With', 'genealogy');
  headers.set('Content-Type', 'application/json');

  const token = getAuthToken();
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  let response: Response;
  try {
    response = await fetch(`${API_BASE}/export/pdf`, {
      method: 'POST',
      credentials: 'include',
      headers,
      body: JSON.stringify(params),
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

  return await response.blob();
}

export async function fetchPdfPreviewInfo(
  params: PdfExportRequest
): Promise<PdfExportPreviewInfo> {
  return apiFetch<PdfExportPreviewInfo>('/export/pdf/preview-info', {
    method: 'POST',
    body: JSON.stringify(params),
  });
}

