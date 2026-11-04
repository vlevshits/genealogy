import { apiFetch } from "./core";

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


export const historyApi = {
  getHistory,
  getEntityHistory,
  undoTransaction,
  restoreChange,
  getSnapshots,
  createSnapshot,
  rollbackSnapshot,
  getEntityVersion,
};
