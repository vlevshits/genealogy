import React from 'react';
import { ArrowRight, Clock, History, Undo2 } from 'lucide-react';
import {
  Avatar,
  Button,
  Card,
  EmptyState,
} from '../ui';
import { ChangeLogTransactionGroupDto } from '../../api/client';
import { formatTxLabel, getGroupEntries } from '../../pages/HistoryPage';
import { useI18n } from '../../i18n';

export interface RecentActivityWidgetProps {
  transactions: ChangeLogTransactionGroupDto[];
  isAdmin: boolean;
  onUndoTransaction?: (txId: string) => void;
  onOpenHistory?: () => void;
  onSelectPerson?: (personId: string) => void;
}

export const RecentActivityWidget: React.FC<RecentActivityWidgetProps> = ({
  transactions,
  isAdmin,
  onUndoTransaction,
  onOpenHistory,
  onSelectPerson,
}) => {
  const { t, locale } = useI18n();

  const formatShortDate = (iso: string): string => {
    try {
      return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  };

  return (
    <Card
      data-testid="dashboard-recent-activity-widget"
      className="p-5 sm:p-6 space-y-4 flex flex-col justify-between"
    >
      <div className="space-y-4">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <History className="h-4 w-4 text-brand shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {isAdmin
                ? t('dashboard.recentActivity.titleAdmin')
                : t('dashboard.recentActivity.titleMember')}
            </h2>
          </div>

          {isAdmin && onOpenHistory && (
            <Button
              size="sm"
              variant="ghost"
              rightIcon={<ArrowRight className="h-3.5 w-3.5" />}
              onClick={onOpenHistory}
            >
              {t('dashboard.recentActivity.allChangesBtn')}
            </Button>
          )}
        </div>

        {transactions.length === 0 ? (
          <EmptyState
            icon={<Clock className="h-6 w-6 text-tertiary" />}
            title={t('dashboard.recentActivity.emptyTitle')}
            description={t('dashboard.recentActivity.emptyDesc')}
          />
        ) : (
          <ul className="divide-y divide-border">
            {transactions.slice(0, 5).map((tx) => {
              const entries = getGroupEntries(tx);
              const firstEntry = entries[0];
              const actorName = tx.actor_name || t('history.unknownActor');
              return (
                <li
                  key={tx.tx_id}
                  data-testid={`dashboard-activity-${tx.tx_id}`}
                  className="py-3 first:pt-0 last:pb-0 flex items-center justify-between gap-3"
                >
                  <div className="flex items-start gap-2.5 min-w-0">
                    <Avatar name={actorName} size="sm" />
                    <div className="min-w-0 space-y-0.5">
                      <div className="flex flex-wrap items-center gap-1.5 text-xs">
                        <span className="font-semibold text-primary">
                          {actorName}
                        </span>
                        <span className="text-secondary">·</span>
                        <span className="text-secondary">
                          {formatTxLabel(tx.tx_label, t)}
                        </span>
                        {firstEntry?.entity_label && (
                          <>
                            <span className="text-secondary">—</span>
                            {firstEntry.entity_type === 'person' &&
                            onSelectPerson ? (
                              <Button
                                variant="link"
                                size="sm"
                                onClick={() =>
                                  onSelectPerson(firstEntry.entity_id)
                                }
                                className="text-xs font-semibold text-brand"
                              >
                                {firstEntry.entity_label}
                              </Button>
                            ) : (
                              <span className="font-medium text-primary">
                                {firstEntry.entity_label}
                              </span>
                            )}
                          </>
                        )}
                      </div>

                      <div className="text-[11px] text-tertiary flex items-center gap-1">
                        <Clock className="h-3 w-3 shrink-0" />
                        <span>{formatShortDate(tx.created_at)}</span>
                      </div>
                    </div>
                  </div>

                  {isAdmin && tx.can_undo && onUndoTransaction && (
                    <Button
                      size="sm"
                      variant="secondary"
                      data-testid={`dashboard-undo-${tx.tx_id}`}
                      leftIcon={<Undo2 className="h-3.5 w-3.5" />}
                      onClick={() => onUndoTransaction(tx.tx_id)}
                    >
                      {t('common.undo')}
                    </Button>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </Card>
  );
};
