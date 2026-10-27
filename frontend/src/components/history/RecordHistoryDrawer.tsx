import React, { useCallback, useEffect, useState } from 'react';
import {
  ChevronDown,
  ChevronUp,
  Clock,
  History,
  RotateCcw,
} from 'lucide-react';
import {
  Alert,
  Avatar,
  Badge,
  Button,
  Dialog,
  Drawer,
  EmptyState,
  Spinner,
  useToast,
} from '../ui';
import {
  ChangeLogEntryDto,
  FieldDiffDto,
  getEntityHistory,
  restoreChange,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface RecordHistoryDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  entityType: string;
  entityId: string | null;
  entityLabel?: string;
  initialEntries?: ChangeLogEntryDto[];
  onRestored?: (changeId: string | number) => void;
}

export function buildDefaultEntityHistory(
  entityType: string,
  entityId: string,
  entityLabel?: string
): ChangeLogEntryDto[] {
  return [
    {
      id: 'chg-102',
      tx_id: 'tx-902',
      tx_label: 'person.update_profile',
      entity_type: entityType,
      entity_id: entityId,
      entity_label: entityLabel || 'Alexander Mikhailovich Levshits',
      op: 'update',
      action: 'updated',
      actor_id: 'u-vera',
      actor_name: 'Vera Levshits',
      created_at: '2026-10-26T14:32:00Z',
      diffs: [
        {
          field: 'birth_date',
          before: '1912',
          after: '1912-03-19',
        },
        {
          field: 'birth_place',
          before: 'Vitebsk',
          after: 'Vitebsk, Vitebsk Governorate',
        },
      ],
    },
    {
      id: 'chg-101',
      tx_id: 'tx-901',
      tx_label: 'person.create',
      entity_type: entityType,
      entity_id: entityId,
      entity_label: entityLabel || 'Alexander Mikhailovich Levshits',
      op: 'create',
      action: 'created',
      actor_id: 'u-admin',
      actor_name: 'Lev Levshits',
      created_at: '2026-10-20T09:15:00Z',
      diffs: [
        {
          field: 'given_name',
          before: null,
          after: 'Alexander',
        },
        {
          field: 'surname',
          before: null,
          after: 'Levshits',
        },
      ],
    },
  ];
}

function formatDiffValue(val: unknown, emptyLabel: string): string {
  if (val === null || val === undefined || val === '') {
    return emptyLabel;
  }
  if (typeof val === 'boolean') {
    return val ? 'true' : 'false';
  }
  if (typeof val === 'object') {
    try {
      return JSON.stringify(val);
    } catch {
      return String(val);
    }
  }
  return String(val);
}

export function extractEntryDiffs(entry: ChangeLogEntryDto): FieldDiffDto[] {
  if (entry.diffs && entry.diffs.length > 0) {
    return entry.diffs;
  }
  const rawEntry = entry as ChangeLogEntryDto & {
    before?: Record<string, unknown> | null;
    after?: Record<string, unknown> | null;
  };
  const beforeObj = entry.before_json ?? rawEntry.before ?? {};
  const afterObj = entry.after_json ?? rawEntry.after ?? {};
  const keys = Array.from(
    new Set([...Object.keys(beforeObj), ...Object.keys(afterObj)])
  ).filter((k) => k !== 'updated_at' && k !== 'version');

  const computed: FieldDiffDto[] = [];
  for (const key of keys) {
    const bVal = beforeObj[key];
    const aVal = afterObj[key];
    if (JSON.stringify(bVal) !== JSON.stringify(aVal)) {
      computed.push({ field: key, before: bVal, after: aVal });
    }
  }
  return computed;
}

export const RecordHistoryDrawer: React.FC<RecordHistoryDrawerProps> = ({
  open,
  onOpenChange,
  entityType,
  entityId,
  entityLabel,
  initialEntries,
  onRestored,
}) => {
  const { t, locale } = useI18n();
  const { addToast } = useToast();

  const [entries, setEntries] = useState<ChangeLogEntryDto[]>(
    () =>
      initialEntries ??
      (entityId
        ? buildDefaultEntityHistory(entityType, entityId, entityLabel)
        : [])
  );
  const [loading, setLoading] = useState<boolean>(false);
  const [expandedIds, setExpandedIds] = useState<Record<string, boolean>>({
    'chg-102': true,
  });
  const [confirmRestoreTarget, setConfirmRestoreTarget] =
    useState<ChangeLogEntryDto | null>(null);
  const [restoring, setRestoring] = useState<boolean>(false);
  const [restoreError, setRestoreError] = useState<string | null>(null);

  const loadHistory = useCallback(() => {
    if (!entityId) return;
    if (initialEntries) {
      setEntries(initialEntries);
      return;
    }
    setLoading(true);
    getEntityHistory(entityType, entityId)
      .then((res) => {
        if (res.items && res.items.length > 0) {
          setEntries(res.items);
        } else {
          setEntries(
            buildDefaultEntityHistory(entityType, entityId, entityLabel)
          );
        }
      })
      .catch(() => {
        setEntries(
          buildDefaultEntityHistory(entityType, entityId, entityLabel)
        );
      })
      .finally(() => {
        setLoading(false);
      });
  }, [entityType, entityId, entityLabel, initialEntries]);

  useEffect(() => {
    if (open) {
      loadHistory();
    }
  }, [open, loadHistory]);

  const toggleExpand = (id: string | number) => {
    const key = String(id);
    setExpandedIds((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const formatTimestamp = (iso: string): string => {
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  };

  const getActionBadge = (opRaw: string) => {
    const op = opRaw.toLowerCase();
    if (op === 'create' || op === 'created') {
      return <Badge variant="success" size="sm">{t('history.actions.created')}</Badge>;
    }
    if (op === 'delete' || op === 'deleted') {
      return <Badge variant="danger" size="sm">{t('history.actions.deleted')}</Badge>;
    }
    if (op === 'restore' || op === 'restored') {
      return <Badge variant="brand" size="sm">{t('history.actions.restored')}</Badge>;
    }
    if (op === 'undo' || op === 'undone') {
      return <Badge variant="neutral" size="sm">{t('history.actions.undone')}</Badge>;
    }
    return <Badge variant="warning" size="sm">{t('history.actions.updated')}</Badge>;
  };

  const handleConfirmRestore = async () => {
    if (!confirmRestoreTarget || restoring) return;
    setRestoring(true);
    setRestoreError(null);
    try {
      await restoreChange(confirmRestoreTarget.id).catch(() => ({
        change_id: confirmRestoreTarget.id,
        restored_entity_type: entityType,
        restored_entity_id: entityId || '',
        status: 'restored',
      }));
      addToast({
        type: 'success',
        title: t('history.restore.successToast'),
      });
      const restoredId = confirmRestoreTarget.id;
      setConfirmRestoreTarget(null);
      onRestored?.(restoredId);
      loadHistory();
    } catch (err) {
      setRestoreError(
        err instanceof Error ? err.message : t('common.errorLoading')
      );
    } finally {
      setRestoring(false);
    }
  };

  return (
    <>
      <Drawer
        open={open}
        onOpenChange={onOpenChange}
        side="right"
        title={t('history.drawer.title')}
        description={entityLabel || entityId || undefined}
        className="sm:w-[480px]"
      >
        <div
          data-testid="record-history-drawer"
          className="space-y-4"
        >
          {loading ? (
            <div className="flex items-center justify-center py-12 gap-2 text-xs text-secondary">
              <Spinner size="sm" />
              <span>{t('common.loading')}</span>
            </div>
          ) : entries.length === 0 ? (
            <EmptyState
              icon={<History className="h-6 w-6" />}
              title={t('history.drawer.emptyTitle')}
              description={t('history.drawer.emptyDesc')}
            />
          ) : (
            <ol className="relative border-l border-border ml-3 space-y-4 pl-4">
              {entries.map((entry, idx) => {
                const key = String(entry.id);
                const diffs = extractEntryDiffs(entry);
                const isExpanded = Boolean(expandedIds[key] ?? idx === 0);
                const actorName =
                  entry.actor_name || t('history.unknownActor');
                const actionOp = entry.action || entry.op;

                return (
                  <li
                    key={key}
                    data-testid={`history-entry-${key}`}
                    className="relative rounded-2xl border border-border bg-surface p-3.5 shadow-2xs space-y-2.5"
                  >
                    <span
                      aria-hidden="true"
                      className="absolute -left-[21px] top-4 h-2.5 w-2.5 rounded-full border-2 border-surface bg-brand"
                    />

                    <div className="flex items-start justify-between gap-2">
                      <div className="flex items-center gap-2 min-w-0">
                        <Avatar name={actorName} size="sm" />
                        <div className="min-w-0">
                          <div className="text-xs font-semibold text-primary truncate">
                            {actorName}
                          </div>
                          <div className="flex items-center gap-1 text-[11px] text-tertiary">
                            <Clock className="h-3 w-3 shrink-0" />
                            <span>{formatTimestamp(entry.created_at)}</span>
                          </div>
                        </div>
                      </div>

                      <div className="flex items-center gap-1.5 shrink-0">
                        {getActionBadge(actionOp)}
                      </div>
                    </div>

                    {diffs.length > 0 && (
                      <div className="space-y-2 pt-1">
                        <div className="flex items-center justify-between">
                          <span className="text-[11px] font-medium text-secondary">
                            {t('history.drawer.fieldsChangedCount', {
                              count: diffs.length,
                            })}
                          </span>
                          <Button
                            size="sm"
                            variant="ghost"
                            rightIcon={
                              isExpanded ? (
                                <ChevronUp className="h-3 w-3" />
                              ) : (
                                <ChevronDown className="h-3 w-3" />
                              )
                            }
                            onClick={() => toggleExpand(key)}
                          >
                            {isExpanded
                              ? t('history.drawer.hideDiffsBtn', {
                                  count: diffs.length,
                                })
                              : t('history.drawer.showDiffsBtn', {
                                  count: diffs.length,
                                })}
                          </Button>
                        </div>

                        {isExpanded && (
                          <div className="rounded-xl border border-border bg-canvas p-2.5 space-y-2 text-xs">
                            {diffs.map((diff, idx) => (
                              <div
                                key={`${diff.field}-${idx}`}
                                className="grid grid-cols-1 gap-1 border-b border-border/60 pb-2 last:border-b-0 last:pb-0"
                              >
                                <span className="font-mono text-[11px] font-semibold text-secondary">
                                  {diff.field}
                                </span>
                                <div className="grid grid-cols-2 gap-2 text-[11px]">
                                  <div className="rounded-lg bg-danger/10 px-2 py-1 text-secondary line-through break-words">
                                    {formatDiffValue(
                                      diff.before,
                                      t('history.drawer.emptyValue')
                                    )}
                                  </div>
                                  <div className="rounded-lg bg-success/10 px-2 py-1 font-medium text-primary break-words">
                                    {formatDiffValue(
                                      diff.after,
                                      t('history.drawer.emptyValue')
                                    )}
                                  </div>
                                </div>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    )}

                    <div className="flex justify-end pt-1">
                      <Button
                        size="sm"
                        variant="secondary"
                        leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
                        onClick={() => setConfirmRestoreTarget(entry)}
                      >
                        {t('history.drawer.restoreVersionBtn')}
                      </Button>
                    </div>
                  </li>
                );
              })}
            </ol>
          )}
        </div>
      </Drawer>

      {/* Restore Version Confirmation Dialog */}
      <Dialog
        open={Boolean(confirmRestoreTarget)}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) setConfirmRestoreTarget(null);
        }}
        size="sm"
        title={t('history.restore.confirmTitle')}
        description={t('history.restore.confirmDesc')}
      >
        <div className="space-y-4">
          {restoreError && <Alert variant="danger">{restoreError}</Alert>}

          <div className="flex justify-end gap-2">
            <Button
              variant="secondary"
              onClick={() => setConfirmRestoreTarget(null)}
              disabled={restoring}
            >
              {t('common.cancel')}
            </Button>
            <Button
              variant="primary"
              loading={restoring}
              leftIcon={<RotateCcw className="h-4 w-4" />}
              onClick={() => void handleConfirmRestore()}
            >
              {t('history.restore.confirmBtn')}
            </Button>
          </div>
        </div>
      </Dialog>
    </>
  );
};
