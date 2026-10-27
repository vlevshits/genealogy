import React from 'react';
import {
  FileText,
  Image as ImageIcon,
  Layers,
  Mic,
  RotateCcw,
  Search,
  Upload,
} from 'lucide-react';
import {
  Badge,
  Button,
  SegmentedControl,
  Select,
  Switch,
  TextInput,
} from '../ui';
import { useI18n } from '../../i18n';

export type MediaCategoryFilter = 'all' | 'photos' | 'documents' | 'audio';

export interface MediaFilterState {
  q: string;
  personId: string;
  decade: string;
  type: MediaCategoryFilter;
  untagged: boolean;
}

export interface MediaPersonOption {
  id: string;
  displayName: string;
  gid: string;
}

export interface MediaFilterBarProps {
  total: number;
  totalUnfiltered: number;
  filters: MediaFilterState;
  peopleOptions: MediaPersonOption[];
  decades?: string[];
  canEdit: boolean;
  onFiltersChange: (patch: Partial<MediaFilterState>) => void;
  onClearFilters: () => void;
  onOpenUpload: () => void;
}

export const DEFAULT_MEDIA_DECADES = [
  '1880s',
  '1900s',
  '1920s',
  '1940s',
  '1960s',
  '1980s',
  '2000s',
];

export const MediaFilterBar: React.FC<MediaFilterBarProps> = ({
  total,
  totalUnfiltered,
  filters,
  peopleOptions,
  decades = DEFAULT_MEDIA_DECADES,
  canEdit,
  onFiltersChange,
  onClearFilters,
  onOpenUpload,
}) => {
  const { t, locale } = useI18n();
  const formatNum = (n: number) =>
    n.toLocaleString(locale === 'ru' ? 'ru-RU' : 'en-US');

  const isFiltered = total !== totalUnfiltered && totalUnfiltered > 0;
  const hasActiveFilters =
    Boolean(filters.q.trim()) ||
    Boolean(filters.personId && filters.personId !== 'all') ||
    Boolean(filters.decade && filters.decade !== 'all') ||
    filters.type !== 'all' ||
    filters.untagged;

  return (
    <div className="space-y-4">
      {/* Top Header Row: Title, Count, Search & Upload Action */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
        <div className="space-y-1">
          <div className="flex flex-wrap items-center gap-2.5">
            <h1 className="font-sans text-2xl sm:text-3xl font-semibold tracking-tight text-primary">
              {t('mediaArchive.title')}
            </h1>
            <Badge variant="neutral">
              {isFiltered
                ? t('mediaArchive.countFiltered', {
                    shown: formatNum(total),
                    total: formatNum(totalUnfiltered),
                  })
                : formatNum(total)}
            </Badge>
          </div>
          <p className="text-sm text-secondary">{t('mediaArchive.subtitle')}</p>
        </div>

        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2.5">
          <div className="w-full sm:w-72">
            <TextInput
              value={filters.q}
              onChange={(e) => onFiltersChange({ q: e.target.value })}
              leftIcon={<Search className="h-4 w-4 text-tertiary" />}
              placeholder={t('mediaArchive.searchPlaceholder')}
              aria-label={t('mediaArchive.searchPlaceholder')}
            />
          </div>

          {canEdit && (
            <Button
              variant="primary"
              size="md"
              onClick={onOpenUpload}
              leftIcon={<Upload className="h-4 w-4" />}
            >
              {t('mediaArchive.uploadBtn')}
            </Button>
          )}
        </div>
      </div>

      {/* Filter Bar Card: Person Select, Asset Type, Decade Chips, Untagged Switch */}
      <div className="rounded-2xl border border-border bg-surface p-4 shadow-xs space-y-3">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3">
          {/* Asset Type Segmented Control */}
          <SegmentedControl
            ariaLabel={t('mediaArchive.filterTypeLabel')}
            value={filters.type}
            onChange={(val) =>
              onFiltersChange({ type: val as MediaCategoryFilter })
            }
            options={[
              {
                value: 'all',
                label: t('mediaArchive.typeAll'),
                icon: <Layers className="h-3.5 w-3.5" />,
              },
              {
                value: 'photos',
                label: t('mediaArchive.typePhotos'),
                icon: <ImageIcon className="h-3.5 w-3.5" />,
              },
              {
                value: 'documents',
                label: t('mediaArchive.typeDocuments'),
                icon: <FileText className="h-3.5 w-3.5" />,
              },
              {
                value: 'audio',
                label: t('mediaArchive.typeAudio'),
                icon: <Mic className="h-3.5 w-3.5" />,
              },
            ]}
          />

          {/* Person Filter Dropdown & Untagged Toggle */}
          <div className="flex flex-wrap items-center gap-3">
            <div className="w-full sm:w-64">
              <Select
                aria-label={t('mediaArchive.filterPersonLabel')}
                value={filters.personId || 'all'}
                onChange={(val) =>
                  onFiltersChange({ personId: val === 'all' ? '' : val })
                }
                options={[
                  {
                    value: 'all',
                    label: t('mediaArchive.filterPersonAll'),
                  },
                  ...peopleOptions.map((p) => ({
                    value: p.id,
                    label: `${p.displayName} (${p.gid})`,
                  })),
                ]}
              />
            </div>

            <Switch
              checked={filters.untagged}
              onCheckedChange={(checked) =>
                onFiltersChange({ untagged: checked })
              }
              label={t('mediaArchive.filterUntagged')}
            />
          </div>
        </div>

        {/* Decade Timeline Chips & Clear Filters */}
        <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-border">
          <div
            className="flex flex-wrap items-center gap-1.5"
            role="group"
            aria-label={t('mediaArchive.filterDecadeLabel')}
          >
            <span className="text-xs font-medium text-secondary mr-1">
              {t('mediaArchive.filterDecadeLabel')}:
            </span>
            <Button
              variant={
                !filters.decade || filters.decade === 'all'
                  ? 'primary'
                  : 'secondary'
              }
              size="sm"
              onClick={() => onFiltersChange({ decade: '' })}
              className="h-7 min-h-0 px-2.5 text-xs"
            >
              {t('mediaArchive.decadeAll')}
            </Button>
            {decades.map((dec) => {
              const active = filters.decade === dec;
              return (
                <Button
                  key={dec}
                  variant={active ? 'primary' : 'ghost'}
                  size="sm"
                  onClick={() =>
                    onFiltersChange({ decade: active ? '' : dec })
                  }
                  className="h-7 min-h-0 px-2.5 text-xs"
                >
                  {dec}
                </Button>
              );
            })}
          </div>

          {hasActiveFilters && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onClearFilters}
              leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
            >
              {t('mediaArchive.clearFilters')}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
};
