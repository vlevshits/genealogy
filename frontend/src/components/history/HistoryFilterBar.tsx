import React, { useState } from 'react';
import { ChevronDown, ChevronUp, RotateCcw, SlidersHorizontal } from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  FilterChip,
  Select,
  Switch,
  TextInput,
} from '../ui';
import { FormField } from '../form';
import { cn } from '../../lib/utils';
import { useI18n } from '../../i18n';

export interface HistoryFiltersState {
  entityType: string;
  op: string;
  fromDate: string;
  toDate: string;
  onlyDeletions: boolean;
}

export interface HistoryFilterBarProps {
  filters: HistoryFiltersState;
  onChange: (next: HistoryFiltersState) => void;
  onReset: () => void;
}

export const HistoryFilterBar: React.FC<HistoryFilterBarProps> = ({
  filters,
  onChange,
  onReset,
}) => {
  const { t } = useI18n();

  const [mobileOpen, setMobileOpen] = useState(false);

  const entityOptions = [
    { value: 'all', label: t('history.filters.entityAll') },
    { value: 'person', label: t('history.filters.entityPerson') },
    { value: 'family', label: t('history.filters.entityFamily') },
    { value: 'event', label: t('history.filters.entityEvent') },
    { value: 'place', label: t('history.filters.entityPlace') },
    { value: 'media', label: t('history.filters.entityMedia') },
    { value: 'note', label: t('history.filters.entityNote') },
  ];

  const actionOptions = [
    { value: 'all', label: t('history.filters.opAll') },
    { value: 'create', label: t('history.filters.opCreate') },
    { value: 'update', label: t('history.filters.opUpdate') },
    { value: 'delete', label: t('history.filters.opDelete') },
    { value: 'restore', label: t('history.filters.opRestore') },
  ];

  const activeChips: Array<{ id: string; label: string; value: string; onRemove: () => void }> = [];
  if (filters.entityType !== 'all') {
    const match = entityOptions.find((o) => o.value === filters.entityType);
    activeChips.push({
      id: 'entity',
      label: t('history.filters.entityTypeLabel'),
      value: match?.label ?? filters.entityType,
      onRemove: () => onChange({ ...filters, entityType: 'all' }),
    });
  }
  if (filters.op !== 'all') {
    const match = actionOptions.find((o) => o.value === filters.op);
    activeChips.push({
      id: 'op',
      label: t('history.filters.actionLabel'),
      value: match?.label ?? filters.op,
      onRemove: () =>
        onChange({
          ...filters,
          op: 'all',
          onlyDeletions: false,
        }),
    });
  }
  if (filters.fromDate) {
    activeChips.push({
      id: 'fromDate',
      label: t('history.filters.fromDateLabel'),
      value: filters.fromDate,
      onRemove: () => onChange({ ...filters, fromDate: '' }),
    });
  }
  if (filters.toDate) {
    activeChips.push({
      id: 'toDate',
      label: t('history.filters.toDateLabel'),
      value: filters.toDate,
      onRemove: () => onChange({ ...filters, toDate: '' }),
    });
  }
  if (filters.onlyDeletions) {
    activeChips.push({
      id: 'onlyDeletions',
      label: t('history.filters.onlyDeletionsLabel'),
      value: t('history.filters.onlyDeletionsChipValue'),
      onRemove: () =>
        onChange({
          ...filters,
          onlyDeletions: false,
          op: filters.op === 'delete' ? 'all' : filters.op,
        }),
    });
  }

  const hasActiveFilters = activeChips.length > 0;

  return (
    <Card className="p-4">
      <div
        data-testid="history-filter-bar"
        className="space-y-3 sm:space-y-4"
      >
        {/* Mobile collapsible filters trigger + active filter chips */}
        <div className="sm:hidden space-y-2.5">
          <div className="flex items-center justify-between gap-2">
            <Button
              type="button"
              size="md"
              variant="secondary"
              data-testid="history-mobile-filters-toggle"
              aria-expanded={mobileOpen}
              aria-controls="history-filters-panel"
              leftIcon={<SlidersHorizontal className="h-4 w-4" />}
              rightIcon={
                mobileOpen ? (
                  <ChevronUp className="h-4 w-4" />
                ) : (
                  <ChevronDown className="h-4 w-4" />
                )
              }
              onClick={() => setMobileOpen((prev) => !prev)}
              className="flex-1 justify-between"
            >
              <span className="flex items-center gap-2">
                <span>{t('history.filters.toggleFilters')}</span>
                {activeChips.length > 0 && (
                  <Badge variant="brand" size="sm">
                    {activeChips.length}
                  </Badge>
                )}
              </span>
            </Button>
          </div>

          {!mobileOpen && activeChips.length > 0 && (
            <div className="flex flex-wrap items-center gap-1.5">
              {activeChips.map((chip) => (
                <FilterChip
                  key={chip.id}
                  label={chip.label}
                  value={chip.value}
                  onRemove={chip.onRemove}
                />
              ))}
            </div>
          )}
        </div>

        <div
          id="history-filters-panel"
          className={cn('space-y-4', !mobileOpen && 'hidden sm:block')}
        >
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
          <FormField
            name="history_entity_type"
            label={t('history.filters.entityTypeLabel')}
          >
            <Select
              ariaLabel={t('history.filters.entityTypeLabel')}
              value={filters.entityType}
              onChange={(val) =>
                onChange({ ...filters, entityType: val })
              }
              options={entityOptions}
            />
          </FormField>

          <FormField
            name="history_op"
            label={t('history.filters.actionLabel')}
          >
            <Select
              ariaLabel={t('history.filters.actionLabel')}
              value={filters.op}
              onChange={(val) =>
                onChange({ ...filters, op: val })
              }
              options={actionOptions}
            />
          </FormField>

          <FormField
            name="history_from_date"
            label={t('history.filters.fromDateLabel')}
          >
            <TextInput
              type="date"
              value={filters.fromDate}
              aria-label={t('history.filters.fromDateLabel')}
              onChange={(e) =>
                onChange({ ...filters, fromDate: e.target.value })
              }
            />
          </FormField>

          <FormField
            name="history_to_date"
            label={t('history.filters.toDateLabel')}
          >
            <TextInput
              type="date"
              value={filters.toDate}
              aria-label={t('history.filters.toDateLabel')}
              onChange={(e) =>
                onChange({ ...filters, toDate: e.target.value })
              }
            />
          </FormField>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3 pt-1">
          <Switch
            checked={filters.onlyDeletions}
            onCheckedChange={(checked) =>
              onChange({
                ...filters,
                onlyDeletions: checked,
                op: checked ? 'delete' : filters.op === 'delete' ? 'all' : filters.op,
              })
            }
            label={t('history.filters.onlyDeletionsLabel')}
          />

          {hasActiveFilters && (
            <Button
              size="sm"
              variant="ghost"
              leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
              onClick={onReset}
              className="min-h-10 sm:min-h-8"
            >
              {t('directory.clearFilters')}
            </Button>
          )}
        </div>
        </div>
      </div>
    </Card>
  );
};
