import React, { useState } from 'react';
import {
  AlertCircle,
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  Columns,
  ExternalLink,
  GitBranch,
  Lock,
  UserPlus,
} from 'lucide-react';
import { PeopleDirectoryItemDto } from '../../api/client';
import {
  Avatar,
  Badge,
  Button,
  Checkbox,
  EmptyState,
  Popover,
} from '../ui';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';

export type DirectorySortKey =
  | 'name_asc'
  | 'name_desc'
  | 'birth_asc'
  | 'birth_desc'
  | 'death_asc'
  | 'death_desc'
  | 'updated_desc';

export interface DirectoryTableProps {
  items: PeopleDirectoryItemDto[];
  sort: DirectorySortKey;
  onSortChange: (nextSort: DirectorySortKey) => void;
  selectedIds: string[];
  onToggleSelectRow: (id: string) => void;
  onToggleSelectAll: () => void;
  canEdit: boolean;
  onOpenPerson: (id: string) => void;
  onFocusInTree: (id: string) => void;
  onAddRelative?: (item: PeopleDirectoryItemDto) => void;
  nextCursor?: string | null;
  loadingMore?: boolean;
  onLoadMore?: () => void;
}

const COLUMN_PREFS_STORAGE_KEY = 'genealogy_directory_columns';

interface ColumnVisibilityState {
  showGid: boolean;
  showUpdated: boolean;
}

function loadColumnPrefs(): ColumnVisibilityState {
  try {
    if (typeof window === 'undefined' || !window.localStorage) {
      return { showGid: false, showUpdated: false };
    }
    const raw = window.localStorage.getItem(COLUMN_PREFS_STORAGE_KEY);
    if (!raw) return { showGid: false, showUpdated: false };
    const parsed = JSON.parse(raw);
    return {
      showGid: Boolean(parsed.showGid ?? false),
      showUpdated: Boolean(parsed.showUpdated ?? false),
    };
  } catch {
    return { showGid: false, showUpdated: false };
  }
}

export const DirectoryTable: React.FC<DirectoryTableProps> = ({
  items,
  sort,
  onSortChange,
  selectedIds,
  onToggleSelectRow,
  onToggleSelectAll,
  canEdit,
  onOpenPerson,
  onFocusInTree,
  onAddRelative,
  nextCursor = null,
  loadingMore = false,
  onLoadMore,
}) => {
  const { t } = useI18n();
  const [colPrefs, setColPrefs] = useState<ColumnVisibilityState>(() =>
    loadColumnPrefs()
  );
  const [scrollTop, setScrollTop] = useState<number>(0);

  const updateColPrefs = (next: Partial<ColumnVisibilityState>) => {
    const merged = { ...colPrefs, ...next };
    setColPrefs(merged);
    try {
      if (typeof window !== 'undefined' && window.localStorage) {
        window.localStorage.setItem(
          COLUMN_PREFS_STORAGE_KEY,
          JSON.stringify(merged)
        );
      }
    } catch {
      // Ignore storage errors
    }
  };

  const allSelected =
    items.length > 0 && items.every((item) => selectedIds.includes(item.id));

  const toggleSortField = (field: 'name' | 'birth' | 'death' | 'updated') => {
    if (field === 'name') {
      onSortChange(sort === 'name_asc' ? 'name_desc' : 'name_asc');
    } else if (field === 'birth') {
      onSortChange(sort === 'birth_asc' ? 'birth_desc' : 'birth_asc');
    } else if (field === 'death') {
      onSortChange(sort === 'death_asc' ? 'death_desc' : 'death_asc');
    } else {
      onSortChange('updated_desc');
    }
  };

  const renderSortIcon = (field: 'name' | 'birth' | 'death' | 'updated') => {
    if (field === 'name') {
      if (sort === 'name_asc') return <ArrowUp className="h-3.5 w-3.5 text-brand" />;
      if (sort === 'name_desc') return <ArrowDown className="h-3.5 w-3.5 text-brand" />;
    }
    if (field === 'birth') {
      if (sort === 'birth_asc') return <ArrowUp className="h-3.5 w-3.5 text-brand" />;
      if (sort === 'birth_desc') return <ArrowDown className="h-3.5 w-3.5 text-brand" />;
    }
    if (field === 'death') {
      if (sort === 'death_asc') return <ArrowUp className="h-3.5 w-3.5 text-brand" />;
      if (sort === 'death_desc') return <ArrowDown className="h-3.5 w-3.5 text-brand" />;
    }
    if (field === 'updated' && sort === 'updated_desc') {
      return <ArrowDown className="h-3.5 w-3.5 text-brand" />;
    }
    return <ArrowUpDown className="h-3.5 w-3.5 text-tertiary" />;
  };

  if (items.length === 0) {
    return (
      <div className="hidden md:block rounded-2xl border border-border bg-surface p-8">
        <EmptyState
          title={t('directory.emptyTitle')}
          description={t('directory.emptyDescription')}
        />
      </div>
    );
  }

  // Virtualization window for large datasets (13 §4.2, §6 DoD #6)
  const ROW_HEIGHT = 56;
  const VIEWPORT_HEIGHT = 640;
  const isVirtualized = items.length > 100;
  const startIndex = isVirtualized
    ? Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - 5)
    : 0;
  const visibleCount = isVirtualized
    ? Math.ceil(VIEWPORT_HEIGHT / ROW_HEIGHT) + 12
    : items.length;
  const endIndex = Math.min(items.length, startIndex + visibleCount);
  const visibleItems = items.slice(startIndex, endIndex);
  const topSpacerHeight = isVirtualized ? startIndex * ROW_HEIGHT : 0;
  const bottomSpacerHeight = isVirtualized
    ? Math.max(0, (items.length - endIndex) * ROW_HEIGHT)
    : 0;

  return (
    <div className="hidden md:block rounded-2xl border border-border bg-surface shadow-xs overflow-hidden">
      {/* Toolbar: Column Visibility & Sort Status */}
      <div className="flex items-center justify-between border-b border-border bg-surface-muted/40 px-4 py-2 text-xs text-secondary">
        <span>
          {t('directory.showingRows', {
            count: items.length,
          })}
        </span>
        <Popover
          align="end"
          trigger={
            <span className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1 text-xs font-medium text-secondary hover:text-primary">
              <Columns className="h-3.5 w-3.5" />
              <span>{t('directory.columnsBtn')}</span>
            </span>
          }
        >
          <div className="space-y-2.5 p-1">
            <div className="text-xs font-semibold text-primary">
              {t('directory.columnsBtn')}
            </div>
            <Checkbox
              checked={colPrefs.showGid}
              onCheckedChange={(checked) => updateColPrefs({ showGid: checked })}
              label={t('directory.colGid')}
            />
            <Checkbox
              checked={colPrefs.showUpdated}
              onCheckedChange={(checked) =>
                updateColPrefs({ showUpdated: checked })
              }
              label={t('directory.colUpdated')}
            />
          </div>
        </Popover>
      </div>

      {/* Scrollable / Virtualized Table Container */}
      <div
        onScroll={(e) => {
          if (isVirtualized) {
            setScrollTop(e.currentTarget.scrollTop);
          }
        }}
        className={cn(
          'overflow-x-auto',
          isVirtualized && 'max-h-[640px] overflow-y-auto'
        )}
      >
        <table className="w-full border-collapse text-left text-sm" role="grid">
          <thead className="sticky top-0 z-10 border-b border-border bg-surface/95 backdrop-blur-xs text-xs font-semibold uppercase tracking-wider text-secondary">
            <tr role="row">
              {canEdit && (
                <th className="w-12 px-4 py-3" role="columnheader">
                  <Checkbox
                    checked={allSelected}
                    onCheckedChange={onToggleSelectAll}
                    aria-label={t('directory.selectAll')}
                  />
                </th>
              )}
              <th className="px-4 py-3" role="columnheader">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => toggleSortField('name')}
                  rightIcon={renderSortIcon('name')}
                  className="-ml-2 h-8 min-h-0 px-2 text-xs font-semibold uppercase tracking-wider text-secondary hover:text-primary"
                >
                  {t('directory.colName')}
                </Button>
              </th>
              <th className="px-4 py-3" role="columnheader">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => toggleSortField('birth')}
                  rightIcon={renderSortIcon('birth')}
                  className="-ml-2 h-8 min-h-0 px-2 text-xs font-semibold uppercase tracking-wider text-secondary hover:text-primary"
                >
                  {t('directory.colBorn')}
                </Button>
              </th>
              <th className="px-4 py-3" role="columnheader">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => toggleSortField('death')}
                  rightIcon={renderSortIcon('death')}
                  className="-ml-2 h-8 min-h-0 px-2 text-xs font-semibold uppercase tracking-wider text-secondary hover:text-primary"
                >
                  {t('directory.colDied')}
                </Button>
              </th>
              <th className="px-4 py-3" role="columnheader">
                {t('directory.colSex')}
              </th>
              {colPrefs.showGid && (
                <th className="px-4 py-3" role="columnheader">
                  {t('directory.colGid')}
                </th>
              )}
              {colPrefs.showUpdated && (
                <th className="px-4 py-3" role="columnheader">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => toggleSortField('updated')}
                    rightIcon={renderSortIcon('updated')}
                    className="-ml-2 h-8 min-h-0 px-2 text-xs font-semibold uppercase tracking-wider text-secondary hover:text-primary"
                  >
                    {t('directory.colUpdated')}
                  </Button>
                </th>
              )}
              <th className="px-4 py-3 text-right" role="columnheader">
                {t('directory.colActions')}
              </th>
            </tr>
          </thead>

          <tbody className="divide-y divide-border">
            {topSpacerHeight > 0 && (
              <tr aria-hidden="true" style={{ height: `${topSpacerHeight}px` }}>
                <td colSpan={8} />
              </tr>
            )}

            {visibleItems.map((item) => {
              const isSelected = selectedIds.includes(item.id);
              const isIncomplete = !item.has_birth_date || !item.has_parents;
              return (
                <tr
                  key={item.id}
                  role="row"
                  aria-selected={isSelected}
                  className={cn(
                    'h-14 transition-colors',
                    isSelected
                      ? 'bg-brand/8'
                      : 'hover:bg-surface-muted/60'
                  )}
                >
                  {canEdit && (
                    <td className="px-4 py-2.5">
                      <Checkbox
                        checked={isSelected}
                        onCheckedChange={() => onToggleSelectRow(item.id)}
                        aria-label={`${t('directory.selectRow')} ${item.display_name}`}
                      />
                    </td>
                  )}

                  {/* Name & Avatar */}
                  <td className="px-4 py-2.5">
                    <div className="flex items-center gap-3">
                      <Avatar
                        name={item.display_name}
                        src={item.avatar_url ?? undefined}
                        size="sm"
                      />
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-1.5">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onOpenPerson(item.id)}
                            className="h-auto min-h-0 p-0 font-semibold text-primary hover:text-brand hover:underline"
                          >
                            {item.display_name}
                          </Button>
                          {item.is_private && (
                            <Badge
                              variant="warning"
                              icon={<Lock className="h-3 w-3" />}
                            >
                              {t('profile.privateBadge')}
                            </Badge>
                          )}
                          {isIncomplete && (
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
                          )}
                        </div>
                      </div>
                    </div>
                  </td>

                  {/* Born (Date + Place) */}
                  <td className="px-4 py-2.5 text-secondary">
                    {item.birth_date || item.birth_year ? (
                      <div>
                        <div className="text-primary font-medium">
                          {item.birth_date || item.birth_year}
                        </div>
                        {item.birth_place && (
                          <div className="text-xs text-secondary truncate max-w-48">
                            {item.birth_place}
                          </div>
                        )}
                      </div>
                    ) : (
                      <span className="text-tertiary">—</span>
                    )}
                  </td>

                  {/* Died (Date + Place with subtle † for deceased, — for living) */}
                  <td className="px-4 py-2.5 text-secondary">
                    {!item.is_living ? (
                      item.death_date || item.death_year ? (
                        <div>
                          <div className="text-primary font-medium">
                            † {item.death_date || item.death_year}
                          </div>
                          {item.death_place && (
                            <div className="text-xs text-secondary truncate max-w-48">
                              {item.death_place}
                            </div>
                          )}
                        </div>
                      ) : (
                        <span className="text-secondary">†</span>
                      )
                    ) : (
                      <span className="text-tertiary">—</span>
                    )}
                  </td>

                  {/* Sex */}
                  <td className="px-4 py-2.5">
                    <Badge variant="neutral">
                      {item.sex === 'M'
                        ? t('person.sex.male')
                        : item.sex === 'F'
                        ? t('person.sex.female')
                        : t('person.sex.unknown')}
                    </Badge>
                  </td>

                  {/* GID */}
                  {colPrefs.showGid && (
                    <td className="px-4 py-2.5 font-mono text-xs text-secondary">
                      {item.gid}
                    </td>
                  )}

                  {/* Last Modified */}
                  {colPrefs.showUpdated && (
                    <td className="px-4 py-2.5 text-xs text-secondary">
                      {item.updated_at || '—'}
                    </td>
                  )}

                  {/* Row Actions */}
                  <td className="px-4 py-2.5 text-right">
                    <div className="inline-flex items-center justify-end gap-1">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onOpenPerson(item.id)}
                        leftIcon={<ExternalLink className="h-3.5 w-3.5" />}
                        className="h-8 min-h-0 px-2 text-xs"
                      >
                        {t('directory.actionOpen')}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onFocusInTree(item.id)}
                        leftIcon={<GitBranch className="h-3.5 w-3.5" />}
                        className="h-8 min-h-0 px-2 text-xs"
                      >
                        {t('directory.actionTree')}
                      </Button>
                      {canEdit && onAddRelative && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => onAddRelative(item)}
                          leftIcon={<UserPlus className="h-3.5 w-3.5" />}
                          className="h-8 min-h-0 px-2 text-xs"
                        >
                          {t('directory.actionAddRelative')}
                        </Button>
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}

            {bottomSpacerHeight > 0 && (
              <tr
                aria-hidden="true"
                style={{ height: `${bottomSpacerHeight}px` }}
              >
                <td colSpan={8} />
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Server-side cursor pagination load more bar */}
      {nextCursor && onLoadMore && (
        <div className="flex items-center justify-center border-t border-border bg-surface-muted/30 p-3">
          <Button
            variant="secondary"
            size="sm"
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
