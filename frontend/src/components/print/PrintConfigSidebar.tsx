import React, { useEffect, useState } from 'react';
import {
  Download,
  FileText,
  GitBranch,
  GitFork,
  Hourglass,
  Layers,
  Network,
  Palette,
  Printer,
  Shield,
  User,
  X,
} from 'lucide-react';
import {
  Button,
  Checkbox,
  IconButton,
  RadioGroup,
  RadioOption,
  SegmentedControl,
  SegmentedOption,
  Select,
  SelectOption,
  Slider,
  TextInput,
} from '../ui';
import { PersonPicker } from '../domain/person-picker';
import { fetchPeopleDirectory, PdfExportRequest } from '../../api/client';
import { Individual } from '../../types/person';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface PrintConfigSidebarProps {
  config: PdfExportRequest;
  onChange: (next: PdfExportRequest) => void;
  onExport: () => void;
  isExporting?: boolean;
  people?: Individual[];
  onSelectFocalPerson?: (personId: string | null, person?: Individual) => void;
  className?: string;
}

export const PrintConfigSidebar: React.FC<PrintConfigSidebarProps> = ({
  config,
  onChange,
  onExport,
  isExporting = false,
  people = [],
  onSelectFocalPerson,
  className,
}) => {
  const { t } = useI18n();
  const [internalPeople, setInternalPeople] = useState<Individual[]>(people);

  useEffect(() => {
    if (people.length > 0) {
      setInternalPeople(people);
    }
  }, [people]);

  const loadPeopleForPicker = async (query: string): Promise<Individual[]> => {
    try {
      const res = await fetchPeopleDirectory({ q: query, limit: 15 });
      const items: Individual[] = res.items.map((item) => ({
        id: item.id,
        gid: item.gid,
        surname: item.surname,
        given_name: item.given_name,
        patronymic: item.patronymic,
        sex:
          item.sex === 'M' || item.sex === 'male'
            ? 'male'
            : item.sex === 'F' || item.sex === 'female'
            ? 'female'
            : 'unknown',
        is_living: item.is_living,
        is_private: false,
        version: 1,
      }));
      setInternalPeople((prev) => {
        const existingIds = new Set(prev.map((p) => p.id));
        const combined = [...prev];
        for (const item of items) {
          if (!existingIds.has(item.id)) {
            combined.push(item);
          }
        }
        return combined;
      });
      return items;
    } catch {
      return [];
    }
  };

  const layoutOptions: SegmentedOption[] = [
    {
      value: 'pedigree',
      label: t('print.layoutPedigree'),
      icon: <GitFork className="h-4 w-4 shrink-0" />,
    },
    {
      value: 'hourglass',
      label: t('print.layoutHourglass'),
      icon: <Hourglass className="h-4 w-4 shrink-0" />,
    },
    {
      value: 'descendants',
      label: t('print.layoutDescendants'),
      icon: <GitBranch className="h-4 w-4 shrink-0" />,
    },
    {
      value: 'all_nodes',
      label: t('print.layoutAllNodes'),
      icon: <Network className="h-4 w-4 shrink-0" />,
    },
  ];

  const densityOptions: SegmentedOption<'wide' | 'compact'>[] = [
    {
      value: 'wide',
      label: t('print.densityWide'),
    },
    {
      value: 'compact',
      label: t('print.densityCompact'),
    },
  ];

  const isFan = config.view_mode === 'fan';

  const formatOptions: RadioOption[] = [
    {
      value: 'tiled',
      label: t('print.formatTiled'),
      description: t('print.formatTiledDesc'),
      icon: <Layers className="h-4 w-4 text-brand" />,
    },
    {
      value: 'poster',
      label: t('print.formatPoster'),
      description: t('print.formatPosterDesc'),
      icon: <FileText className="h-4 w-4 text-accent" />,
    },
  ];

  const tiledPaperSizes: SelectOption[] = [
    { value: 'A4', label: t('print.paperA4') },
    { value: 'Letter', label: t('print.paperLetter') },
  ];

  const posterPaperSizes: SelectOption[] = [
    { value: 'A0', label: t('print.paperA0') },
    { value: 'A1', label: t('print.paperA1') },
    { value: 'A2', label: t('print.paperA2') },
    { value: 'A3', label: t('print.paperA3') },
  ];

  const themeOptions: SegmentedOption[] = [
    {
      value: 'modern',
      label: t('print.themeModern'),
      icon: <Palette className="h-4 w-4" />,
    },
    {
      value: 'heritage',
      label: t('print.themeHeritage'),
      icon: <FileText className="h-4 w-4" />,
    },
  ];

  return (
    <aside
      aria-label={t('print.sidebarTitle')}
      className={cn(
        'w-full lg:w-96 flex flex-col bg-surface border-r border-border h-full overflow-y-auto',
        className
      )}
    >
      {/* Sidebar Header */}
      <div className="p-4 sm:p-5 border-b border-border space-y-1">
        <div className="flex items-center gap-2">
          <Printer className="h-5 w-5 text-brand" />
          <h2 className="text-base font-semibold text-primary">
            {t('print.sidebarTitle')}
          </h2>
        </div>
        <p className="text-xs text-secondary">{t('print.sidebarSubtitle')}</p>
      </div>

      <div className="p-4 sm:p-5 pb-28 lg:pb-6 space-y-6">
        {/* 1. Focal Individual */}
        <div className="space-y-1.5">
          <label className="flex items-center gap-1.5 text-xs font-semibold text-primary uppercase tracking-wider">
            <User className="h-3.5 w-3.5 text-secondary" />
            <span>{t('print.focalPerson')}</span>
          </label>
          <PersonPicker
            value={config.focal_person_id ?? null}
            people={internalPeople}
            loadPeople={loadPeopleForPicker}
            placeholder={t('print.focalPersonPlaceholder')}
            ariaLabel={t('print.focalPerson')}
            onChange={(personId, person) => {
              onChange({ ...config, focal_person_id: personId || undefined });
              onSelectFocalPerson?.(personId, person);
            }}
          />
          <p className="text-[11px] text-tertiary">{t('print.focalPersonHint')}</p>
        </div>

        {/* 2. Chart Layout */}
        <div className="space-y-1.5">
          <label className="text-xs font-semibold text-primary uppercase tracking-wider block">
            {t('print.chartLayout')}
          </label>
          <SegmentedControl
            options={layoutOptions}
            value={config.view_mode || 'pedigree'}
            onChange={(val) => onChange({ ...config, view_mode: val })}
            ariaLabel={t('print.chartLayout')}
            size="sm"
            className="w-full grid grid-cols-2 gap-1 items-stretch"
          />
        </div>

        {/* 3. Generation Depth */}
        {config.view_mode === 'all_nodes' ? (
          <div className="space-y-1.5 p-3 rounded-lg bg-surface-muted/50 border border-border">
            <p className="text-xs text-secondary">{t('print.allNodesDesc')}</p>
          </div>
        ) : (
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-primary uppercase tracking-wider block">
              {t('print.generations')}
            </label>
            <Slider
              min={3}
              max={8}
              step={1}
              value={config.generations ?? 5}
              label={t('print.generationsCount', { count: config.generations ?? 5 })}
              onChange={(val) => onChange({ ...config, generations: val })}
            />
          </div>
        )}

        {/* Tree Density (Hourglass only) */}
        {config.view_mode === 'hourglass' && (
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-primary uppercase tracking-wider block">
              {t('print.treeDensity')}
            </label>
            <SegmentedControl<'wide' | 'compact'>
              options={densityOptions}
              value={config.collateral !== false ? 'wide' : 'compact'}
              onChange={(val) =>
                onChange({ ...config, collateral: val === 'wide' })
              }
              ariaLabel={t('print.treeDensity')}
              disabled={isFan}
              size="sm"
            />
            <p className="text-[11px] text-tertiary">
              {config.collateral !== false
                ? t('print.densityWideDesc')
                : t('print.densityCompactDesc')}
            </p>
          </div>
        )}

        {/* 4. Printing Format (Radio buttons) */}
        <div className="space-y-2">
          <label className="text-xs font-semibold text-primary uppercase tracking-wider block">
            {t('print.format')}
          </label>
          <RadioGroup
            options={formatOptions}
            value={config.format || 'tiled'}
            onChange={(val) => {
              const nextPaper = val === 'poster' ? 'A2' : 'A4';
              onChange({ ...config, format: val, target_sheet_size: nextPaper });
            }}
            ariaLabel={t('print.format')}
          />
        </div>

        {/* 5. Format-Specific Options */}
        {config.format === 'poster' ? (
          <div className="space-y-1.5 p-3 rounded-lg bg-surface-muted/50 border border-border">
            <label className="text-xs font-semibold text-primary block">
              {t('print.sheetSize')}
            </label>
            <Select
              options={posterPaperSizes}
              value={config.target_sheet_size || 'A2'}
              onChange={(val) => onChange({ ...config, target_sheet_size: val })}
              ariaLabel={t('print.sheetSize')}
              size="sm"
            />
          </div>
        ) : (
          <div className="space-y-3.5 p-3 rounded-lg bg-surface-muted/50 border border-border">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-primary block">
                {t('print.sheetSize')}
              </label>
              <Select
                options={tiledPaperSizes}
                value={config.target_sheet_size || 'A4'}
                onChange={(val) => onChange({ ...config, target_sheet_size: val })}
                ariaLabel={t('print.sheetSize')}
                size="sm"
              />
            </div>

            <div className="space-y-1.5">
              <Slider
                min={5}
                max={20}
                step={1}
                unit=" mm"
                value={config.overlap_mm ?? 10}
                label={t('print.overlapBleed')}
                onChange={(val) => onChange({ ...config, overlap_mm: val })}
              />
            </div>

            <Checkbox
              checked={config.show_crop_marks !== false}
              onCheckedChange={(checked) =>
                onChange({ ...config, show_crop_marks: checked })
              }
              label={<span className="text-xs font-medium text-primary">{t('print.showCropMarks')}</span>}
              description={<span className="text-[11px] text-tertiary">{t('print.showCropMarksDesc')}</span>}
            />

            <Checkbox
              checked={Boolean(config.include_assembly_guide)}
              onCheckedChange={(checked) =>
                onChange({ ...config, include_assembly_guide: checked })
              }
              label={<span className="text-xs font-medium text-primary">{t('print.includeAssemblyGuide')}</span>}
              description={<span className="text-[11px] text-tertiary">{t('print.includeAssemblyGuideDesc')}</span>}
            />
          </div>
        )}

        {/* 6. Visual Theme */}
        <div className="space-y-2">
          <label className="text-xs font-semibold text-primary uppercase tracking-wider block">
            {t('print.visualTheme')}
          </label>
          <SegmentedControl
            options={themeOptions}
            value={config.theme || 'modern'}
            onChange={(val) => onChange({ ...config, theme: val })}
            ariaLabel={t('print.visualTheme')}
            size="sm"
          />
          <p className="text-[11px] text-secondary">
            {config.theme === 'heritage'
              ? t('print.themeHeritageDesc')
              : t('print.themeModernDesc')}
          </p>
        </div>

        {/* 7. Relatives & Content Density */}
        <div className="space-y-2.5">
          <label className="flex items-center gap-1.5 text-xs font-semibold text-primary uppercase tracking-wider">
            <Shield className="h-3.5 w-3.5 text-secondary" />
            <span>{t('print.contentDensity')}</span>
          </label>

          <Checkbox
            checked={config.include_partner !== false}
            disabled={isFan}
            onCheckedChange={(checked) =>
              onChange({ ...config, include_partner: checked })
            }
            label={<span className="text-xs font-medium text-primary">{t('print.includePartner')}</span>}
          />

          <Checkbox
            checked={Boolean(config.include_siblings)}
            disabled={isFan}
            onCheckedChange={(checked) =>
              onChange({ ...config, include_siblings: checked })
            }
            label={<span className="text-xs font-medium text-primary">{t('print.includeSiblings')}</span>}
          />

          <Checkbox
            checked={config.show_photos !== false}
            onCheckedChange={(checked) =>
              onChange({ ...config, show_photos: checked })
            }
            label={<span className="text-xs font-medium text-primary">{t('print.showPhotos')}</span>}
          />

          <Checkbox
            checked={Boolean(config.redact_living)}
            onCheckedChange={(checked) =>
              onChange({ ...config, redact_living: checked })
            }
            label={<span className="text-xs font-medium text-primary">{t('print.redactLiving')}</span>}
            description={<span className="text-[11px] text-tertiary">{t('print.redactLivingDesc')}</span>}
          />

          {isFan && (
            <p className="text-[11px] text-amber-600 dark:text-amber-500 font-medium">
              {t('print.fanDisabledHint')}
            </p>
          )}
        </div>

        {/* 8. Custom Banner Inputs */}
        <div className="space-y-4">
          <div className="space-y-1.5">
            <label
              htmlFor="print-chart-title-input"
              className="text-xs font-semibold text-primary uppercase tracking-wider block cursor-pointer"
            >
              {t('print.chartTitle')}
            </label>
            <TextInput
              id="print-chart-title-input"
              size="sm"
              placeholder={t('print.chartTitlePlaceholder')}
              value={config.title ?? ''}
              onChange={(e) =>
                onChange({ ...config, title: e.target.value || undefined })
              }
              rightSlot={
                config.title ? (
                  <IconButton
                    variant="ghost"
                    size="sm"
                    label={t('common.clear')}
                    onClick={() => onChange({ ...config, title: undefined })}
                    className="h-6 w-6 min-h-0 min-w-0 p-0 text-tertiary hover:text-primary"
                  >
                    <X className="h-3.5 w-3.5" />
                  </IconButton>
                ) : undefined
              }
            />
          </div>

          <div className="space-y-1.5">
            <label
              htmlFor="print-chart-subtitle-input"
              className="text-xs font-semibold text-primary uppercase tracking-wider block cursor-pointer"
            >
              {t('print.chartSubtitle')}
            </label>
            <TextInput
              id="print-chart-subtitle-input"
              aria-describedby="print-chart-subtitle-hint"
              size="sm"
              placeholder={t('print.chartSubtitlePlaceholder')}
              value={config.subtitle ?? ''}
              onChange={(e) =>
                onChange({ ...config, subtitle: e.target.value || undefined })
              }
              rightSlot={
                config.subtitle ? (
                  <IconButton
                    variant="ghost"
                    size="sm"
                    label={t('common.clear')}
                    onClick={() => onChange({ ...config, subtitle: undefined })}
                    className="h-6 w-6 min-h-0 min-w-0 p-0 text-tertiary hover:text-primary"
                  >
                    <X className="h-3.5 w-3.5" />
                  </IconButton>
                ) : undefined
              }
            />
            <p id="print-chart-subtitle-hint" className="text-[11px] text-tertiary">
              {t('print.chartSubtitleHint')}
            </p>
          </div>
        </div>

        {/* 9. Primary Action Button (Sticky) */}
        <div className="sticky bottom-0 bg-surface/95 backdrop-blur-xs p-4 border-t border-border -mx-4 -mb-4 mt-4 z-10">
          <Button
            variant="primary"
            size="lg"
            className="w-full shadow-sm"
            leftIcon={<Download className="h-4 w-4" />}
            loading={isExporting}
            disabled={isExporting}
            onClick={onExport}
          >
            {isExporting ? t('print.generatingPdf') : t('print.downloadPdf')}
          </Button>
        </div>
      </div>
    </aside>
  );
};
