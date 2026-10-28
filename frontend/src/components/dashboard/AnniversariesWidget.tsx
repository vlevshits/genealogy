import React from 'react';
import { Calendar, Heart, Sparkles } from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  EmptyState,
  SegmentedControl,
} from '../ui';
import { AnniversaryItemDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface AnniversariesWidgetProps {
  items: AnniversaryItemDto[];
  scope: 'all' | 'close';
  onScopeChange: (nextScope: 'all' | 'close') => void;
  accessTier?: 'public_anonymous' | 'viewer' | 'editor' | 'admin';
  onSelectPerson?: (personId: string) => void;
}

export function maskLivingSingleInitial(
  displayName: string,
  isLiving: boolean,
  accessTier?: string
): string {
  if (!isLiving || accessTier !== 'public_anonymous') {
    return displayName;
  }
  const trimmed = displayName.trim();
  if (!trimmed) return 'L.';
  return `${trimmed.charAt(0).toUpperCase()}.`;
}

export const AnniversariesWidget: React.FC<AnniversariesWidgetProps> = ({
  items,
  scope,
  onScopeChange,
  accessTier = 'admin',
  onSelectPerson,
}) => {
  const { t } = useI18n();

  return (
    <Card
      data-testid="dashboard-anniversaries-widget"
      className="p-5 sm:p-6 space-y-4 flex flex-col justify-between"
    >
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Calendar className="h-4 w-4 text-brand shrink-0" />
            <h2 className="font-sans text-base font-semibold text-primary">
              {t('dashboard.anniversaries.title')}
            </h2>
            <Badge variant="neutral" size="sm">
              {items.length}
            </Badge>
          </div>

          <SegmentedControl<'all' | 'close'>
            ariaLabel={t('dashboard.anniversaries.scopeAria')}
            size="sm"
            value={scope}
            onChange={onScopeChange}
            options={[
              {
                value: 'all',
                label: t('dashboard.anniversaries.scopeAll'),
              },
              {
                value: 'close',
                label: t('dashboard.anniversaries.scopeClose'),
              },
            ]}
          />
        </div>

        {items.length === 0 ? (
          <EmptyState
            icon={<Calendar className="h-6 w-6 text-tertiary" />}
            title={t('dashboard.anniversaries.emptyTitle')}
            description={t('dashboard.anniversaries.emptyDesc')}
          />
        ) : (
          <ul className="divide-y divide-border">
            {items.map((item, idx) => {
              const rawName =
                item.person?.display_name || item.display_name || '';
              const personId = item.person?.id || item.person_id || null;
              const isLiving = item.person?.is_living ?? item.is_living ?? false;
              const nameDisplay = maskLivingSingleInitial(
                rawName,
                isLiving,
                accessTier
              );
              const isToday = item.days_until === 0;
              const kind = item.kind || item.type || 'birthday';
              const years = item.years_elapsed ?? item.years_count ?? 0;
              const dateStr =
                item.date_label ||
                item.next_occurrence_date ||
                item.date_month_day ||
                '';
              const itemKey = item.id || `${personId || 'ann'}-${kind}-${idx}`;

              return (
                <li
                  key={itemKey}
                  data-testid={`anniversary-item-${itemKey}`}
                  className="py-3 first:pt-0 last:pb-0 flex items-center justify-between gap-3"
                >
                  <div className="min-w-0 space-y-0.5">
                    <div className="flex flex-wrap items-center gap-2">
                      <Badge
                        variant={isToday ? 'brand' : 'neutral'}
                        size="sm"
                      >
                        {isToday
                          ? t('dashboard.anniversaries.todayBadge')
                          : dateStr}
                      </Badge>
                      {personId && onSelectPerson ? (
                        <Button
                          variant="link"
                          size="sm"
                          onClick={() => onSelectPerson(personId)}
                          className="text-sm font-semibold text-primary hover:text-brand"
                        >
                          {nameDisplay}
                        </Button>
                      ) : (
                        <span className="text-sm font-semibold text-primary">
                          {nameDisplay}
                        </span>
                      )}
                      {!isLiving && item.death_year && (
                        <span className="text-xs font-mono text-tertiary">
                          (†{item.death_year})
                        </span>
                      )}
                    </div>

                    <div className="text-xs text-secondary flex items-center gap-1.5">
                      {kind === 'wedding' ? (
                        <Heart className="h-3 w-3 text-accent shrink-0" />
                      ) : (
                        <Sparkles className="h-3 w-3 text-brand shrink-0" />
                      )}
                      <span>
                        {kind === 'birthday'
                          ? t('dashboard.anniversaries.turnsYears', {
                              date: dateStr,
                              years,
                            })
                          : kind === 'memorial'
                          ? t('dashboard.anniversaries.memorialYears', {
                              date: dateStr,
                              years,
                            })
                          : t('dashboard.anniversaries.weddingYears', {
                              date: dateStr,
                              years,
                            })}
                      </span>
                    </div>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </Card>
  );
};
