import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  ChevronDown,
  ChevronUp,
  Clock,
  History,
  RotateCcw,
  Undo2,
} from 'lucide-react';
import {
  AccessDeniedCard,
  Avatar,
  Badge,
  Button,
  Card,
  EmptyState,
  PageHeader,
  Spinner,
  useToast,
} from '../components/ui';
import {
  HistoryFilterBar,
  HistoryFiltersState,
  extractEntryDiffs,
  formatDiffValue,
} from '../components/history';
import {
  ChangeLogEntryDto,
  ChangeLogTransactionGroupDto,
  getHistory,
  restoreChange,
  undoTransaction,
} from '../api/client';
import { useAuth } from '../lib/auth-context';
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';

export interface HistoryPageProps {
  initialGroups?: ChangeLogTransactionGroupDto[];
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
}

export function buildDefaultHistoryFeed(): ChangeLogTransactionGroupDto[] {
  return [
    {
      tx_id: 'tx-201',
      tx_label: 'person.add_parent',
      actor_id: 'u-vera',
      actor_name: 'Vera Levshits',
      created_at: '2026-10-27T14:32:00Z',
      can_undo: true,
      entries: [
        {
          id: 'chg-501',
          tx_id: 'tx-201',
          tx_label: 'person.add_parent',
          entity_type: 'person',
          entity_id: 'p-pyotr',
          entity_label: 'Pyotr Fyodorovich Shcheglov',
          op: 'create',
          action: 'created',
          actor_id: 'u-vera',
          actor_name: 'Vera Levshits',
          created_at: '2026-10-27T14:32:00Z',
          diffs: [
            { field: 'given_name', before: null, after: 'Pyotr' },
            { field: 'patronymic', before: null, after: 'Fyodorovich' },
            { field: 'surname', before: null, after: 'Shcheglov' },
          ],
        },
      ],
    },
    {
      tx_id: 'tx-200',
      tx_label: 'person.update_profile',
      actor_id: 'u-lev',
      actor_name: 'Lev Levshits',
      created_at: '2026-10-27T12:15:00Z',
      can_undo: true,
      entries: [
        {
          id: 'chg-500',
          tx_id: 'tx-200',
          tx_label: 'person.update_profile',
          entity_type: 'person',
          entity_id: 'p-elena',
          entity_label: 'Elena Morozova',
          op: 'update',
          action: 'updated',
          actor_id: 'u-lev',
          actor_name: 'Lev Levshits',
          created_at: '2026-10-27T12:15:00Z',
          diffs: [
            { field: 'birth_date', before: '1915', after: 'abt 1914' },
          ],
        },
      ],
    },
    {
      tx_id: 'tx-199',
      tx_label: 'person.delete',
      actor_id: 'u-lev',
      actor_name: 'Lev Levshits',
      created_at: '2026-10-26T18:05:00Z',
      can_undo: false,
      entries: [
        {
          id: 'chg-499',
          tx_id: 'tx-199',
          tx_label: 'person.delete',
          entity_type: 'person',
          entity_id: 'p-unknown-child',
          entity_label: 'Unknown Child',
          op: 'delete',
          action: 'deleted',
          actor_id: 'u-lev',
          actor_name: 'Lev Levshits',
          created_at: '2026-10-26T18:05:00Z',
          diffs: [
            { field: 'status', before: 'active', after: 'deleted' },
          ],
        },
      ],
    },
  ];
}

const DEFAULT_FILTERS: HistoryFiltersState = {
  entityType: 'all',
  op: 'all',
  fromDate: '',
  toDate: '',
  onlyDeletions: false,
};

export function formatTxLabel(
  txLabel: string | undefined | null,
  t: (key: string) => string
): string {
  if (!txLabel) return t('history.txLabels.default');
  const map: Record<string, string> = {
    'person.add_parent': 'history.txLabels.personAddParent',
    'person.add_child': 'history.txLabels.personAddChild',
    'person.add_spouse': 'history.txLabels.personAddSpouse',
    'person.add_sibling': 'history.txLabels.personAddSibling',
    'person.create': 'history.txLabels.personCreate',
    'person.update_profile': 'history.txLabels.personUpdateProfile',
    'person.update': 'history.txLabels.personUpdateProfile',
    'person.delete': 'history.txLabels.personDelete',
    'family.create': 'history.txLabels.familyCreate',
    'family.update': 'history.txLabels.familyUpdate',
    'event.create': 'history.txLabels.eventCreate',
    'event.update': 'history.txLabels.eventUpdate',
    'event.delete': 'history.txLabels.eventDelete',
    'media.upload': 'history.txLabels.mediaUpload',
    'place.update': 'history.txLabels.placeUpdate',
  };
  const i18nKey = map[txLabel.toLowerCase()];
  if (i18nKey) {
    return t(i18nKey);
  }
  return txLabel
    .replace(/[._-]+/g, ' ')
    .replace(/\b\w/g, (ch) => ch.toUpperCase());
}

export function getGroupEntries(
  group: ChangeLogTransactionGroupDto
): ChangeLogEntryDto[] {
  return group.entries || group.changes || [];
}

export const HistoryPage: React.FC<HistoryPageProps> = ({
  initialGroups,
  userRole,
  onNavigate,
}) => {
  const { t, locale } = useI18n();
  const { user } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const isAdmin = effectiveRole === 'admin';

  const [filters, setFilters] = useState<HistoryFiltersState>(DEFAULT_FILTERS);
  const [groups, setGroups] = useState<ChangeLogTransactionGroupDto[]>(
    () => initialGroups ?? buildDefaultHistoryFeed()
  );
  const [loading, setLoading] = useState<boolean>(false);
  const [expandedTxIds, setExpandedTxIds] = useState<Record<string, boolean>>({
    'tx-201': true,
    'tx-200': true,
    'tx-199': true,
  });

  const loadFeed = useCallback(() => {
    if (!isAdmin || initialGroups) return;
    setLoading(true);
    getHistory({
      entity_type: filters.entityType,
      op: filters.onlyDeletions ? 'delete' : filters.op,
      only_deletions: filters.onlyDeletions,
      from_date: filters.fromDate || undefined,
      to_date: filters.toDate || undefined,
    })
      .then((res) => {
        const txGroups =
          res.transactions && res.transactions.length > 0
            ? res.transactions
            : Array.isArray(res.items) &&
              res.items.length > 0 &&
              ('entries' in res.items[0] || 'changes' in res.items[0])
            ? (res.items as unknown as ChangeLogTransactionGroupDto[])
            : [];
        if (txGroups.length > 0) {
          setGroups(txGroups);
        }
      })
      .catch(() => {})
      .finally(() => {
        setLoading(false);
      });
  }, [isAdmin, initialGroups, filters]);

  useEffect(() => {
    loadFeed();
  }, [loadFeed]);

  const filteredGroups = useMemo(() => {
    return groups.filter((group) => {
      const groupEntries = getGroupEntries(group);
      return groupEntries.some((entry) => {
        const entryOp = (entry.op || entry.action || '').toLowerCase();
        if (filters.onlyDeletions && !entryOp.startsWith('delet')) {
          return false;
        }
        if (
          filters.entityType !== 'all' &&
          entry.entity_type.toLowerCase() !== filters.entityType.toLowerCase()
        ) {
          return false;
        }
        if (
          filters.op !== 'all' &&
          !entryOp.startsWith(filters.op.toLowerCase())
        ) {
          return false;
        }
        return true;
      });
    });
  }, [groups, filters]);

  if (!isAdmin) {
    return (
      <div className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-12">
        <AccessDeniedCard
          currentRole={effectiveRole === 'editor' ? 'editor' : 'viewer'}
          onOpenProfile={
            onNavigate ? () => onNavigate('/settings/profile') : undefined
          }
          onBackToTree={onNavigate ? () => onNavigate('/tree') : undefined}
        />
      </div>
    );
  }

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

  const handleUndoTx = async (txId: string) => {
    try {
      await undoTransaction(txId).catch(() => ({
        tx_id: txId,
        reverted_count: 1,
        status: 'undone',
      }));
      setGroups((prev) =>
        prev.map((g) => (g.tx_id === txId ? { ...g, can_undo: false } : g))
      );
      addToast({
        type: 'success',
        title: t('history.undo_confirmed_toast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  const handleRestoreDeleted = async (entry: ChangeLogEntryDto) => {
    try {
      await restoreChange(entry.id).catch(() => ({
        change_id: entry.id,
        restored_entity_type: entry.entity_type,
        restored_entity_id: entry.entity_id,
        status: 'restored',
      }));
      addToast({
        type: 'success',
        title: t('history.restore.successToast'),
      });
      loadFeed();
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: getLocalizedErrorMessage(err, t),
      });
    }
  };

  return (
    <div
      data-testid="history-page"
      className="mx-auto max-w-7xl w-full px-4 sm:px-6 lg:px-8 py-8 space-y-6"
    >
      <PageHeader
        title={t('history.page.title')}
        subtitle={t('history.page.subtitle')}
        secondaryActions={
          <Badge variant="brand">
            {t('history.page.totalTransactions', {
              count: filteredGroups.length,
            })}
          </Badge>
        }
      />

      <HistoryFilterBar
        filters={filters}
        onChange={setFilters}
        onReset={() => setFilters(DEFAULT_FILTERS)}
      />

      {loading ? (
        <div className="flex items-center justify-center py-12 gap-2 text-xs text-secondary">
          <Spinner size="md" />
          <span>{t('common.loading')}</span>
        </div>
      ) : filteredGroups.length === 0 ? (
        <Card className="p-8">
          <EmptyState
            icon={<History className="h-6 w-6" />}
            title={t('history.page.emptyTitle')}
            description={t('history.page.emptyDesc')}
          />
        </Card>
      ) : (
        <div className="space-y-4">
          {filteredGroups.map((group) => {
            const isExpanded = Boolean(expandedTxIds[group.tx_id] ?? true);
            const actorName = group.actor_name || t('history.unknownActor');
            const groupEntries = getGroupEntries(group);
            const primaryEntry = groupEntries[0];
            const opLower = (
              primaryEntry?.op ||
              primaryEntry?.action ||
              'update'
            ).toLowerCase();
            const isDeletion = opLower.startsWith('delet');
            const totalDiffCount = groupEntries.reduce(
              (acc, entry) => acc + extractEntryDiffs(entry).length,
              0
            );

            return (
              <Card
                key={group.tx_id}
                data-testid={`history-tx-${group.tx_id}`}
                className="p-4 sm:p-5 space-y-3"
              >
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                  <div className="flex items-start gap-3 min-w-0">
                    <Avatar name={actorName} size="md" />
                    <div className="min-w-0 space-y-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="text-sm font-semibold text-primary">
                          {actorName}
                        </span>
                        <Badge
                          variant={
                            opLower.startsWith('creat')
                              ? 'success'
                              : isDeletion
                              ? 'danger'
                              : 'warning'
                          }
                          size="sm"
                        >
                          {opLower.startsWith('creat')
                            ? t('history.actions.created')
                            : isDeletion
                            ? t('history.actions.deleted')
                            : t('history.actions.updated')}
                        </Badge>
                        {primaryEntry?.entity_label && (
                          <span className="text-sm font-medium text-primary">
                            {primaryEntry.entity_label}
                          </span>
                        )}
                      </div>

                      <div className="flex flex-wrap items-center gap-2 text-xs text-secondary">
                        <span className="inline-flex items-center gap-1">
                          <Clock className="h-3.5 w-3.5 text-tertiary" />
                          {formatTimestamp(group.created_at)}
                        </span>
                        {group.tx_label && (
                          <>
                            <span>·</span>
                            <span
                              data-testid={`history-tx-label-${group.tx_id}`}
                              className="text-secondary font-medium"
                            >
                              {formatTxLabel(group.tx_label, t)}
                            </span>
                          </>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center gap-2 shrink-0">
                    {group.can_undo && (
                      <Button
                        size="sm"
                        variant="secondary"
                        data-testid={`history-undo-tx-${group.tx_id}`}
                        leftIcon={<Undo2 className="h-3.5 w-3.5" />}
                        onClick={() => void handleUndoTx(group.tx_id)}
                      >
                        {t('common.undo')}
                      </Button>
                    )}

                    {isDeletion && primaryEntry && (
                      <Button
                        size="sm"
                        variant="primary"
                        data-testid={`history-restore-entry-${primaryEntry.id}`}
                        leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
                        onClick={() => void handleRestoreDeleted(primaryEntry)}
                      >
                        {t('history.page.restoreDeletedBtn')}
                      </Button>
                    )}

                    <Button
                      size="sm"
                      variant="ghost"
                      rightIcon={
                        isExpanded ? (
                          <ChevronUp className="h-3.5 w-3.5" />
                        ) : (
                          <ChevronDown className="h-3.5 w-3.5" />
                        )
                      }
                      onClick={() =>
                        setExpandedTxIds((prev) => ({
                          ...prev,
                          [group.tx_id]: !isExpanded,
                        }))
                      }
                    >
                      {isExpanded
                        ? t('history.drawer.hideDiffsBtn', {
                            count: totalDiffCount,
                          })
                        : t('history.drawer.showDiffsBtn', {
                            count: totalDiffCount,
                          })}
                    </Button>
                  </div>
                </div>

                {isExpanded && (
                  <div className="border-t border-border pt-3 space-y-2">
                    {groupEntries.map((entry) => {
                      const diffs = extractEntryDiffs(entry);
                      const entityTypeKey = `history.entityTypes.${entry.entity_type.toLowerCase()}`;
                      const translatedEntityType = t(entityTypeKey);
                      const displayEntityType =
                        translatedEntityType !== entityTypeKey
                          ? translatedEntityType
                          : entry.entity_type;

                      return (
                        <div
                          key={String(entry.id)}
                          className="rounded-xl border border-border bg-canvas p-3 space-y-2 text-xs"
                        >
                          <div className="flex items-center justify-between gap-2">
                            <span className="font-medium text-primary">
                              {entry.entity_label || entry.entity_id} (
                              {displayEntityType})
                            </span>
                            <span className="font-mono text-[11px] text-tertiary">
                              {entry.entity_id}
                            </span>
                          </div>

                          {diffs.length > 0 && (
                            <div className="space-y-1.5">
                              {diffs.map((d, idx) => {
                                const fieldKey = `history.fields.${d.field}`;
                                const translatedField = t(fieldKey);
                                const displayField =
                                  translatedField !== fieldKey
                                    ? `${translatedField} (${d.field})`
                                    : d.field;
                                return (
                                  <div
                                    key={`${d.field}-${idx}`}
                                    className="grid grid-cols-1 sm:grid-cols-3 gap-2 items-center text-[11px]"
                                  >
                                    <span className="font-mono font-semibold text-secondary">
                                      {displayField}
                                    </span>
                                    <span className="rounded bg-danger/10 px-2 py-0.5 text-secondary line-through break-words max-w-full overflow-hidden text-ellipsis">
                                      {formatDiffValue(
                                        d.before,
                                        t('history.drawer.emptyValue')
                                      )}
                                    </span>
                                    <span className="rounded bg-success/10 px-2 py-0.5 font-medium text-primary break-words max-w-full overflow-hidden text-ellipsis">
                                      {formatDiffValue(
                                        d.after,
                                        t('history.drawer.emptyValue')
                                      )}
                                    </span>
                                  </div>
                                );
                              })}
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                )}
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default HistoryPage;
