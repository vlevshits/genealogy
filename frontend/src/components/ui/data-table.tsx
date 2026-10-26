import React, { useState, useMemo, useRef } from 'react';
import { ArrowDown, ArrowUp, ArrowUpDown, Columns, SlidersHorizontal, X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Checkbox } from './checkbox';
import { DropdownMenu } from './dropdown-menu';
import { SegmentedControl } from './segmented-control';
import { Button } from './button';
import { Skeleton } from './skeleton';
import { useI18n } from '../../i18n';

export interface DataTableColumn<T> {
  id: string;
  header: string;
  accessor?: (row: T) => React.ReactNode;
  sortValue?: (row: T) => string | number;
  sortable?: boolean;
  align?: 'left' | 'center' | 'right';
  width?: string;
}

export type TableDensity = 'comfortable' | 'compact';

export interface DataTableProps<T> {
  data: T[];
  columns: DataTableColumn<T>[];
  getRowId: (row: T) => string;
  selectable?: boolean;
  selectedIds?: string[];
  onSelectionChange?: (ids: string[]) => void;
  onRowClick?: (row: T, event: React.MouseEvent) => void;
  mobileRow?: (row: T) => React.ReactNode;
  forceMobilePreview?: boolean;
  defaultDensity?: TableDensity;
  viewportHeight?: number;
  loading?: boolean;
  emptyState?: React.ReactNode;
  className?: string;
}

export function DataTable<T>({
  data,
  columns,
  getRowId,
  selectable = false,
  selectedIds,
  onSelectionChange,
  onRowClick,
  mobileRow,
  forceMobilePreview = false,
  defaultDensity = 'comfortable',
  viewportHeight = 480,
  loading = false,
  emptyState,
  className,
}: DataTableProps<T>) {
  const { t } = useI18n();
  const [sortColId, setSortColId] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc');
  const [hiddenCols, setHiddenCols] = useState<string[]>([]);
  const [density, setDensity] = useState<TableDensity>(defaultDensity);
  const [internalSelected, setInternalSelected] = useState<string[]>([]);
  const [lastClickedIdx, setLastClickedIdx] = useState<number | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const scrollContainerRef = useRef<HTMLDivElement | null>(null);

  const selected = selectedIds !== undefined ? selectedIds : internalSelected;
  const selectedSet = useMemo(() => new Set(selected), [selected]);

  const setSelection = (next: string[]) => {
    if (selectedIds === undefined) setInternalSelected(next);
    onSelectionChange?.(next);
  };

  const visibleColumns = columns.filter((c) => !hiddenCols.includes(c.id));

  const sortedData = useMemo(() => {
    if (!sortColId) return data;
    const col = columns.find((c) => c.id === sortColId);
    if (!col || !col.sortValue) return data;
    const copy = [...data];
    copy.sort((a, b) => {
      const va = col.sortValue!(a);
      const vb = col.sortValue!(b);
      if (va < vb) return sortDir === 'asc' ? -1 : 1;
      if (va > vb) return sortDir === 'asc' ? 1 : -1;
      return 0;
    });
    return copy;
  }, [data, sortColId, sortDir, columns]);

  const rowHeight = density === 'comfortable' ? 56 : 40;
  const totalHeight = sortedData.length * rowHeight;
  const overscan = 6;
  const startIndex = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
  const visibleCount = Math.ceil(viewportHeight / rowHeight) + overscan * 2;
  const endIndex = Math.min(sortedData.length, startIndex + visibleCount);
  const virtualRows = sortedData.slice(startIndex, endIndex);

  const handleSort = (colId: string) => {
    if (sortColId === colId) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortColId(colId);
      setSortDir('asc');
    }
  };

  const allSelected = sortedData.length > 0 && selected.length === sortedData.length;
  const isIndeterminate = selected.length > 0 && selected.length < sortedData.length;

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      // For 50k datasets, cap quick select or select all IDs
      setSelection(sortedData.slice(0, 500).map(getRowId));
    } else {
      setSelection([]);
    }
  };

  const handleRowSelect = (idx: number, id: string, shiftKey: boolean) => {
    if (shiftKey && lastClickedIdx !== null) {
      const [lo, hi] = [Math.min(lastClickedIdx, idx), Math.max(lastClickedIdx, idx)];
      const rangeIds = sortedData.slice(lo, hi + 1).map(getRowId);
      const merged = Array.from(new Set([...selected, ...rangeIds]));
      setSelection(merged);
    } else {
      if (selectedSet.has(id)) {
        setSelection(selected.filter((x) => x !== id));
      } else {
        setSelection([...selected, id]);
      }
    }
    setLastClickedIdx(idx);
  };

  return (
    <div className={cn('space-y-3', className)}>
      {/* Table Controls Toolbar */}
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="text-xs text-secondary font-mono">
          {sortedData.length.toLocaleString()} rows
        </div>
        <div className="flex items-center gap-2">
          <SegmentedControl
            size="sm"
            ariaLabel={t('common.density')}
            value={density}
            onChange={setDensity}
            options={[
              { value: 'comfortable', label: t('common.comfortable') },
              { value: 'compact', label: t('common.compact') },
            ]}
          />
          <DropdownMenu
            ariaLabel={t('common.columns')}
            trigger={
              <span className="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface px-2.5 py-1.5 text-xs font-medium text-secondary hover:text-primary">
                <Columns className="h-3.5 w-3.5" />
                <span>{t('common.columns')}</span>
              </span>
            }
            items={columns.map((c) => ({
              id: c.id,
              label: c.header,
              checked: !hiddenCols.includes(c.id),
              onSelect: () => {
                setHiddenCols((prev) =>
                  prev.includes(c.id) ? prev.filter((x) => x !== c.id) : [...prev, c.id]
                );
              },
            }))}
          />
        </div>
      </div>

      {loading ? (
        <div className="space-y-2 rounded-2xl border border-border bg-surface p-4">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton key={i} className="h-10 w-full" />
          ))}
        </div>
      ) : sortedData.length === 0 ? (
        <div>{emptyState}</div>
      ) : (
        <>
          {/* Mobile Card Rendering (< 768px or forceMobilePreview) */}
          {mobileRow && (
            <div
              className={cn(
                'space-y-2.5 max-h-[480px] overflow-y-auto',
                forceMobilePreview ? 'block' : 'md:hidden'
              )}
            >
              {sortedData.slice(0, 30).map((row) => (
                <div key={getRowId(row)}>{mobileRow(row)}</div>
              ))}
            </div>
          )}

          {/* Desktop Virtualized Table */}
          <div
            className={cn(
              'overflow-hidden rounded-2xl border border-border bg-surface shadow-xs',
              mobileRow && !forceMobilePreview && 'hidden md:block',
              forceMobilePreview && 'hidden'
            )}
          >
            {/* Sticky Header */}
            <div className="flex items-center border-b border-border bg-surface-muted/70 px-4 py-2.5 text-xs font-semibold text-secondary">
              {selectable && (
                <div className="w-10 shrink-0">
                  <Checkbox
                    aria-label={t('common.selectAll')}
                    checked={allSelected}
                    indeterminate={isIndeterminate}
                    onCheckedChange={handleSelectAll}
                  />
                </div>
              )}
              {visibleColumns.map((col) => {
                const isSorted = sortColId === col.id;
                const ariaSort = isSorted
                  ? sortDir === 'asc'
                    ? 'ascending'
                    : 'descending'
                  : 'none';
                return (
                  <div
                    key={col.id}
                    role="columnheader"
                    aria-sort={col.sortable ? ariaSort : undefined}
                    style={{ width: col.width }}
                    className={cn(
                      'flex-1 min-w-0 px-2',
                      col.align === 'right' && 'text-right',
                      col.align === 'center' && 'text-center'
                    )}
                  >
                    {col.sortable ? (
                      <button
                        type="button"
                        onClick={() => handleSort(col.id)}
                        className="inline-flex items-center gap-1 hover:text-primary cursor-pointer"
                      >
                        <span>{col.header}</span>
                        {isSorted ? (
                          sortDir === 'asc' ? (
                            <ArrowUp className="h-3.5 w-3.5 text-brand" />
                          ) : (
                            <ArrowDown className="h-3.5 w-3.5 text-brand" />
                          )
                        ) : (
                          <ArrowUpDown className="h-3 w-3 opacity-50" />
                        )}
                      </button>
                    ) : (
                      <span>{col.header}</span>
                    )}
                  </div>
                );
              })}
            </div>

            {/* Virtualized Scroll Body */}
            <div
              ref={scrollContainerRef}
              onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)}
              style={{ height: viewportHeight }}
              className="relative overflow-y-auto"
            >
              <div style={{ height: totalHeight, position: 'relative' }}>
                {virtualRows.map((row, localIdx) => {
                  const actualIdx = startIndex + localIdx;
                  const id = getRowId(row);
                  const isRowSelected = selectedSet.has(id);
                  const top = actualIdx * rowHeight;

                  return (
                    <div
                      key={id}
                      role="row"
                      aria-selected={isRowSelected}
                      style={{
                        position: 'absolute',
                        top,
                        left: 0,
                        right: 0,
                        height: rowHeight,
                      }}
                      onClick={(e) => onRowClick?.(row, e)}
                      className={cn(
                        'flex items-center border-b border-border/60 px-4 text-xs sm:text-sm transition-colors',
                        isRowSelected ? 'bg-brand/10' : 'hover:bg-surface-muted/50',
                        onRowClick && 'cursor-pointer'
                      )}
                    >
                      {selectable && (
                        <div
                          className="w-10 shrink-0"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleRowSelect(actualIdx, id, e.shiftKey);
                          }}
                        >
                          <Checkbox
                            aria-label={t('common.selectRow', { id })}
                            checked={isRowSelected}
                            onCheckedChange={() => {}}
                          />
                        </div>
                      )}
                      {visibleColumns.map((col) => (
                        <div
                          key={col.id}
                          role="cell"
                          style={{ width: col.width }}
                          className={cn(
                            'flex-1 min-w-0 truncate px-2',
                            col.align === 'right' && 'text-right',
                            col.align === 'center' && 'text-center'
                          )}
                        >
                          {col.accessor
                            ? col.accessor(row)
                            : String((row as Record<string, unknown>)[col.id] ?? '')}
                        </div>
                      ))}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export interface FilterChipProps {
  label: string;
  value: string;
  onRemove: () => void;
}

export const FilterChip: React.FC<FilterChipProps> = ({ label, value, onRemove }) => {
  const { t } = useI18n();
  return (
    <span className="inline-flex items-center gap-1.5 rounded-full border border-brand/30 bg-brand/10 px-3 py-1 text-xs font-medium text-brand">
      <span>
        {label}: <strong>{value}</strong>
      </span>
      <button
        type="button"
        aria-label={t('common.removeChip', { label })}
        onClick={onRemove}
        className="rounded-full p-0.5 hover:bg-brand/20 cursor-pointer"
      >
        <X className="h-3 w-3" />
      </button>
    </span>
  );
};

export interface FilterBarProps {
  children: React.ReactNode;
  onClearAll?: () => void;
  hasActiveFilters?: boolean;
  className?: string;
}

export const FilterBar: React.FC<FilterBarProps> = ({
  children,
  onClearAll,
  hasActiveFilters = false,
  className,
}) => {
  const { t } = useI18n();
  return (
    <div
      className={cn(
        'flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-surface p-3',
        className
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <SlidersHorizontal className="h-4 w-4 text-tertiary mr-1" />
        {children}
      </div>
      {hasActiveFilters && onClearAll && (
        <Button variant="ghost" size="sm" onClick={onClearAll}>
          {t('common.clear')}
        </Button>
      )}
    </div>
  );
};

export interface BulkActionBarProps {
  selectedCount: number;
  onClearSelection: () => void;
  actions?: React.ReactNode;
}

export const BulkActionBar: React.FC<BulkActionBarProps> = ({
  selectedCount,
  onClearSelection,
  actions,
}) => {
  const { t } = useI18n();
  if (selectedCount === 0) return null;

  return (
    <div className="sticky bottom-4 z-30 mx-auto flex max-w-xl items-center justify-between gap-4 rounded-2xl border border-brand/40 bg-surface/95 px-4 py-2.5 shadow-lg backdrop-blur-md">
      <div className="flex items-center gap-3 text-xs sm:text-sm font-semibold text-primary">
        <span className="inline-flex h-6 min-w-6 items-center justify-center rounded-full bg-brand px-2 font-mono text-xs text-surface">
          {selectedCount}
        </span>
        <span>{t('common.selectedCount', { count: selectedCount })}</span>
      </div>
      <div className="flex items-center gap-2">
        {actions}
        <Button variant="ghost" size="sm" onClick={onClearSelection}>
          {t('common.clearSelection')}
        </Button>
      </div>
    </div>
  );
};
