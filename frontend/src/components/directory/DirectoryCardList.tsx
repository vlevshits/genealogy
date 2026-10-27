import React from 'react';
import { AlertCircle, ChevronRight, GitBranch } from 'lucide-react';
import { PeopleDirectoryItemDto } from '../../api/client';
import { Avatar, Badge, Button, Checkbox, EmptyState } from '../ui';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';

export interface DirectoryCardListProps {
  items: PeopleDirectoryItemDto[];
  selectedIds: string[];
  onToggleSelectRow: (id: string) => void;
  canEdit: boolean;
  onOpenPerson: (id: string) => void;
  onFocusInTree: (id: string) => void;
  nextCursor?: string | null;
  loadingMore?: boolean;
  onLoadMore?: () => void;
}

export const DirectoryCardList: React.FC<DirectoryCardListProps> = ({
  items,
  selectedIds,
  onToggleSelectRow,
  canEdit,
  onOpenPerson,
  onFocusInTree,
  nextCursor = null,
  loadingMore = false,
  onLoadMore,
}) => {
  const { t } = useI18n();

  if (items.length === 0) {
    return (
      <div className="md:hidden rounded-2xl border border-border bg-surface p-6">
        <EmptyState
          title={t('directory.emptyTitle')}
          description={t('directory.emptyDescription')}
        />
      </div>
    );
  }

  return (
    <div className="md:hidden space-y-2.5">
      {items.map((item) => {
        const isSelected = selectedIds.includes(item.id);
        const isIncomplete = !item.has_birth_date || !item.has_parents;
        const lifeSummaryParts: string[] = [];
        if (item.birth_date || item.birth_year) {
          lifeSummaryParts.push(
            `${t('domain.bornPrefix')} ${item.birth_date || item.birth_year}`
          );
        }
        if (!item.is_living && (item.death_date || item.death_year)) {
          lifeSummaryParts.push(`† ${item.death_date || item.death_year}`);
        }
        if (item.birth_place) {
          lifeSummaryParts.push(item.birth_place);
        }

        return (
          <div
            key={item.id}
            className={cn(
              'flex items-center justify-between gap-3 rounded-2xl border p-3.5 transition-colors',
              isSelected
                ? 'border-brand bg-brand/8'
                : 'border-border bg-surface'
            )}
          >
            <div className="flex items-center gap-3 min-w-0 flex-1">
              {canEdit && (
                <Checkbox
                  checked={isSelected}
                  onCheckedChange={() => onToggleSelectRow(item.id)}
                  aria-label={`${t('directory.selectRow')} ${item.display_name}`}
                />
              )}
              <Avatar
                name={item.display_name}
                src={item.avatar_url ?? undefined}
                size="md"
              />
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-1.5">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => onOpenPerson(item.id)}
                    className="h-auto min-h-0 p-0 text-left font-semibold text-primary truncate"
                  >
                    {item.display_name}
                  </Button>
                  <Badge variant="neutral">{item.gid}</Badge>
                </div>
                {lifeSummaryParts.length > 0 && (
                  <p className="text-xs text-secondary truncate mt-0.5">
                    {lifeSummaryParts.join(' · ')}
                  </p>
                )}
                {isIncomplete && (
                  <div className="mt-1">
                    <Badge
                      variant="warning"
                      icon={<AlertCircle className="h-3 w-3" />}
                    >
                      {!item.has_birth_date && !item.has_parents
                        ? t('directory.incompleteBothBadge')
                        : !item.has_birth_date
                        ? t('directory.incompleteNoBirthBadge')
                        : t('directory.incompleteNoParentsBadge')}
                    </Badge>
                  </div>
                )}
              </div>
            </div>

            <div className="flex items-center gap-1 shrink-0">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onFocusInTree(item.id)}
                leftIcon={<GitBranch className="h-4 w-4" />}
                aria-label={t('directory.actionTree')}
                className="px-2"
              />
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onOpenPerson(item.id)}
                rightIcon={<ChevronRight className="h-4 w-4" />}
                aria-label={t('directory.actionOpen')}
                className="px-2"
              />
            </div>
          </div>
        );
      })}

      {nextCursor && onLoadMore && (
        <div className="pt-2 flex justify-center">
          <Button
            variant="secondary"
            size="md"
            loading={loadingMore}
            onClick={onLoadMore}
          >
            {t('directory.loadMoreBtn')}
          </Button>
        </div>
      )}
    </div>
  );
};
