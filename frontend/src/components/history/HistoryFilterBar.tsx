import React from 'react';
import { RotateCcw } from 'lucide-react';
import {
  Button,
  Card,
  Select,
  Switch,
  TextInput,
} from '../ui';
import { FormField } from '../form';
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

  const hasActiveFilters =
    filters.entityType !== 'all' ||
    filters.op !== 'all' ||
    Boolean(filters.fromDate) ||
    Boolean(filters.toDate) ||
    filters.onlyDeletions;

  return (
    <Card className="p-4">
      <div
        data-testid="history-filter-bar"
        className="space-y-4"
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
            >
              {t('directory.clearFilters')}
            </Button>
          )}
        </div>
      </div>
    </Card>
  );
};
