import React from 'react';
import { AlertCircle, RotateCcw } from 'lucide-react';
import { Button, Select, Switch, TextInput } from '../ui';
import { useI18n } from '../../i18n';

export interface DirectoryFilterState {
  surname: string;
  bornFrom: string;
  bornTo: string;
  sex: string;
  living: string;
  hasPhoto: string;
  incomplete: boolean;
}

export interface DirectoryFilterBarProps {
  filters: DirectoryFilterState;
  onFiltersChange: (next: Partial<DirectoryFilterState>) => void;
  onClearFilters: () => void;
}

export const DirectoryFilterBar: React.FC<DirectoryFilterBarProps> = ({
  filters,
  onFiltersChange,
  onClearFilters,
}) => {
  const { t } = useI18n();

  const hasActiveFilters =
    Boolean(filters.surname.trim()) ||
    Boolean(filters.bornFrom.trim()) ||
    Boolean(filters.bornTo.trim()) ||
    (filters.sex !== '' && filters.sex !== 'all') ||
    (filters.living !== '' && filters.living !== 'all') ||
    (filters.hasPhoto !== '' && filters.hasPhoto !== 'all') ||
    filters.incomplete;

  return (
    <div className="rounded-2xl border border-border bg-surface p-4 shadow-xs space-y-3">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-6 gap-3">
        {/* Surname filter */}
        <div>
          <label className="block text-xs font-medium text-secondary mb-1">
            {t('directory.filterSurname')}
          </label>
          <TextInput
            value={filters.surname}
            onChange={(e) => onFiltersChange({ surname: e.target.value })}
            placeholder={t('directory.filterSurnamePlaceholder')}
            aria-label={t('directory.filterSurname')}
          />
        </div>

        {/* Born from year */}
        <div>
          <label className="block text-xs font-medium text-secondary mb-1">
            {t('directory.filterBornFrom')}
          </label>
          <TextInput
            value={filters.bornFrom}
            onChange={(e) =>
              onFiltersChange({
                bornFrom: e.target.value.replace(/[^0-9]/g, '').slice(0, 4),
              })
            }
            placeholder="1800"
            aria-label={t('directory.filterBornFrom')}
          />
        </div>

        {/* Born to year */}
        <div>
          <label className="block text-xs font-medium text-secondary mb-1">
            {t('directory.filterBornTo')}
          </label>
          <TextInput
            value={filters.bornTo}
            onChange={(e) =>
              onFiltersChange({
                bornTo: e.target.value.replace(/[^0-9]/g, '').slice(0, 4),
              })
            }
            placeholder="1950"
            aria-label={t('directory.filterBornTo')}
          />
        </div>

        {/* Sex filter */}
        <div>
          <label className="block text-xs font-medium text-secondary mb-1">
            {t('directory.filterSex')}
          </label>
          <Select
            aria-label={t('directory.filterSex')}
            value={filters.sex || 'all'}
            onChange={(val) => onFiltersChange({ sex: val })}
            options={[
              { value: 'all', label: t('directory.filterAll') },
              { value: 'M', label: t('person.sex.male') },
              { value: 'F', label: t('person.sex.female') },
              { value: 'U', label: t('person.sex.unknown') },
            ]}
          />
        </div>

        {/* Living / Deceased filter */}
        <div>
          <label className="block text-xs font-medium text-secondary mb-1">
            {t('directory.filterStatus')}
          </label>
          <Select
            aria-label={t('directory.filterStatus')}
            value={filters.living || 'all'}
            onChange={(val) => onFiltersChange({ living: val })}
            options={[
              { value: 'all', label: t('directory.filterAll') },
              { value: 'living', label: t('directory.filterLivingOnly') },
              { value: 'deceased', label: t('directory.filterDeceasedOnly') },
            ]}
          />
        </div>

        {/* Has photo filter */}
        <div>
          <label className="block text-xs font-medium text-secondary mb-1">
            {t('directory.filterPhoto')}
          </label>
          <Select
            aria-label={t('directory.filterPhoto')}
            value={filters.hasPhoto || 'all'}
            onChange={(val) => onFiltersChange({ hasPhoto: val })}
            options={[
              { value: 'all', label: t('directory.filterAll') },
              { value: 'yes', label: t('directory.filterWithPhoto') },
              { value: 'no', label: t('directory.filterWithoutPhoto') },
            ]}
          />
        </div>
      </div>

      {/* Bottom filter row: Incomplete research toggle & Clear Filters */}
      <div className="flex flex-wrap items-center justify-between gap-3 pt-2 border-t border-border">
        <div className="flex items-center gap-3">
          <Switch
            checked={filters.incomplete}
            onCheckedChange={(checked) =>
              onFiltersChange({ incomplete: checked })
            }
            label={t('directory.filterIncomplete')}
          />
          <span className="hidden sm:inline-flex items-center gap-1 text-xs text-secondary">
            <AlertCircle className="h-3.5 w-3.5 text-warning" />
            <span>{t('directory.filterIncompleteHint')}</span>
          </span>
        </div>

        {hasActiveFilters && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onClearFilters}
            leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
          >
            {t('directory.clearFilters')}
          </Button>
        )}
      </div>
    </div>
  );
};
