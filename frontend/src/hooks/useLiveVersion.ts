import { useEffect, useState } from 'react';
import { EntityVersionDto, getEntityVersion } from '../api/client';

export interface UseLiveVersionOptions {
  entityType: string;
  entityId: string | null | undefined;
  initialVersion: number;
  enabled?: boolean;
  pollIntervalMs?: number;
}

export interface UseLiveVersionResult {
  hasRemoteUpdate: boolean;
  remoteVersion: number;
  updatedByName: string | null;
  latestMeta: EntityVersionDto | null;
}

/**
 * Lightweight live version awareness hook (14-change-history-concurrency.md §4):
 * Periodically polls GET /api/v1/:entity_type/:id/version every 30s while an edit
 * form is open/dirty, surfacing a gentle non-blocking warning banner if remote
 * version > initialVersion.
 */
export function useLiveVersion({
  entityType,
  entityId,
  initialVersion,
  enabled = true,
  pollIntervalMs = 30_000,
}: UseLiveVersionOptions): UseLiveVersionResult {
  const [latestMeta, setLatestMeta] = useState<EntityVersionDto | null>(null);

  useEffect(() => {
    setLatestMeta(null);
  }, [entityId, initialVersion]);

  useEffect(() => {
    if (!enabled || !entityId) return;

    let cancelled = false;

    const checkVersion = async () => {
      try {
        const res = await getEntityVersion(entityType, entityId);
        if (!cancelled && res && typeof res.version === 'number') {
          setLatestMeta(res);
        }
      } catch {
        // Silently ignore background polling errors when offline or mocked
      }
    };

    const timer = setInterval(() => {
      void checkVersion();
    }, pollIntervalMs);

    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [entityType, entityId, enabled, pollIntervalMs]);

  const remoteVersion = latestMeta?.version ?? initialVersion;
  const hasRemoteUpdate = remoteVersion > initialVersion;
  const updatedByName = latestMeta?.updated_by_name ?? null;

  return {
    hasRemoteUpdate,
    remoteVersion,
    updatedByName,
    latestMeta,
  };
}
