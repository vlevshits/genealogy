import React, { useState } from 'react';
import {
  AlertTriangle,
  CheckCircle2,
  Cpu,
  FileArchive,
  FolderGit2,
  GitMerge,
  Image as ImageIcon,
  MapPin,
  ShieldAlert,
  Sparkles,
  Users,
} from 'lucide-react';
import {
  Alert,
  Badge,
  Button,
  Card,
  CardBody,
  CardFooter,
  CardHeader,
  Dialog,
  RadioCardGroup,
} from '../../ui';
import { GedcomAnalysisReport } from '../../../api/client';
import { useI18n } from '../../../i18n';
import { formatGedcomWarning } from '../../../lib/error-formatter';
import { formatBytes } from '../SystemDiagnosticsCard';

export type GedcomUiImportMode = 'merge' | 'clean_slate';

export interface GedcomAnalysisModalProps {
  report: GedcomAnalysisReport;
  open?: boolean;
  asDialog?: boolean;
  loading?: boolean;
  isImporting?: boolean;
  importMode?: GedcomUiImportMode;
  onSelectImportMode?: (mode: GedcomUiImportMode) => void;
  onProceedMerge?: () => void;
  onProceedCleanSlate?: () => void;
  onProceed?: () => void;
  onOpenChange?: (open: boolean) => void;
  onCancel: () => void;
}

export const GedcomAnalysisModal: React.FC<GedcomAnalysisModalProps> = ({
  report,
  open = true,
  asDialog = false,
  loading = false,
  isImporting = false,
  importMode,
  onSelectImportMode,
  onProceedMerge,
  onProceedCleanSlate,
  onProceed,
  onOpenChange,
  onCancel,
}) => {
  const { t, locale } = useI18n();
  const [internalMode, setInternalMode] = useState<GedcomUiImportMode>(
    importMode ?? 'merge'
  );
  const effectiveMode: GedcomUiImportMode = importMode ?? internalMode;
  const isBusy = Boolean(loading || isImporting);

  const formatNum = (n: number) =>
    n.toLocaleString(locale === 'ru' ? 'ru-RU' : 'en-US');

  const d = report.dialect_summary;

  const handleSelectMode = (nextMode: GedcomUiImportMode) => {
    setInternalMode(nextMode);
    onSelectImportMode?.(nextMode);
  };

  const handlePrimaryAction = () => {
    if (effectiveMode === 'merge') {
      if (onProceedMerge) {
        onProceedMerge();
      } else {
        onProceed?.();
      }
    } else {
      if (onProceedCleanSlate) {
        onProceedCleanSlate();
      } else {
        onProceed?.();
      }
    }
  };

  const primaryBtnText =
    effectiveMode === 'merge'
      ? t('gedcom.analysis.proceedMergeBtn')
      : t('gedcom.analysis.proceedCleanSlateBtn');

  const content = (
    <div className="space-y-5" data-testid="gedcom-analysis-report">
      {/* File & Dialect Metadata Badges */}
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-border bg-surface-muted/50 px-3.5 py-2.5">
        <div className="flex flex-wrap items-center gap-2 min-w-0">
          <FileArchive className="h-4 w-4 text-brand shrink-0" />
          <span className="font-mono text-xs sm:text-sm font-semibold text-primary truncate">
            {report.file_name}
          </span>
          <span className="font-mono text-xs text-secondary">
            ({formatBytes(report.file_size_bytes)})
          </span>
        </div>

        <div className="flex flex-wrap items-center gap-1.5">
          <Badge variant="brand" size="sm" icon={<Cpu className="h-3 w-3" />}>
            {report.detected_software}
          </Badge>
          <Badge variant="neutral" size="sm">
            GEDCOM {report.gedcom_version}
          </Badge>
          <Badge variant="neutral" size="sm">
            {report.character_encoding}
          </Badge>
        </div>
      </div>

      {/* Primary Entity Counts Grid */}
      <div className="grid grid-cols-2 sm:grid-cols-5 gap-3">
        <div className="rounded-xl border border-border bg-surface p-3 space-y-1">
          <div className="flex items-center gap-1.5 text-xs text-secondary">
            <Users className="h-3.5 w-3.5 text-brand" />
            <span>{t('gedcom.analysis.statIndividuals')}</span>
          </div>
          <div className="font-mono text-lg font-semibold text-primary">
            {formatNum(report.total_individuals)}
          </div>
        </div>

        <div className="rounded-xl border border-border bg-surface p-3 space-y-1">
          <div className="flex items-center gap-1.5 text-xs text-secondary">
            <FolderGit2 className="h-3.5 w-3.5 text-brand" />
            <span>{t('gedcom.analysis.statFamilies')}</span>
          </div>
          <div className="font-mono text-lg font-semibold text-primary">
            {formatNum(report.total_families)}
          </div>
        </div>

        <div className="rounded-xl border border-border bg-surface p-3 space-y-1">
          <div className="flex items-center gap-1.5 text-xs text-secondary">
            <Sparkles className="h-3.5 w-3.5 text-brand" />
            <span>{t('gedcom.analysis.statEvents')}</span>
          </div>
          <div className="font-mono text-lg font-semibold text-primary">
            {formatNum(report.total_events)}
          </div>
        </div>

        <div className="rounded-xl border border-border bg-surface p-3 space-y-1">
          <div className="flex items-center gap-1.5 text-xs text-secondary">
            <MapPin className="h-3.5 w-3.5 text-brand" />
            <span>{t('gedcom.analysis.statPlaces')}</span>
          </div>
          <div className="font-mono text-lg font-semibold text-primary">
            {formatNum(report.total_places)}
          </div>
        </div>

        <div className="rounded-xl border border-border bg-surface p-3 space-y-1 col-span-2 sm:col-span-1">
          <div className="flex items-center gap-1.5 text-xs text-secondary">
            <ImageIcon className="h-3.5 w-3.5 text-brand" />
            <span>{t('gedcom.analysis.statMedia')}</span>
          </div>
          <div className="flex items-baseline gap-1.5">
            <span className="font-mono text-lg font-semibold text-primary">
              {formatNum(report.total_media)}
            </span>
            {report.missing_media_count > 0 && (
              <span className="text-[11px] font-medium text-warning">
                ({t('gedcom.analysis.missingMediaShort', {
                  count: formatNum(report.missing_media_count),
                })})
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Dialect & Vendor Extension Highlights (03 §2.2 & §3.2) */}
      <div className="rounded-xl border border-border bg-surface p-4 space-y-2.5">
        <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-secondary">
          <CheckCircle2 className="h-3.5 w-3.5 text-success" />
          <span>{t('gedcom.analysis.dialectHeading')}</span>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs text-secondary">
          <div className="flex items-center justify-between rounded-lg bg-surface-muted/50 px-3 py-1.5">
            <span>{t('gedcom.analysis.dialectCoordinates')}</span>
            <span className="font-mono font-semibold text-primary">
              {formatNum(d.coordinates_normalized)}
            </span>
          </div>
          <div className="flex items-center justify-between rounded-lg bg-surface-muted/50 px-3 py-1.5">
            <span>{t('gedcom.analysis.dialectUids')}</span>
            <span className="font-mono font-semibold text-primary">
              {formatNum(d.uids_preserved)}
            </span>
          </div>
          <div className="flex items-center justify-between rounded-lg bg-surface-muted/50 px-3 py-1.5">
            <span>{t('gedcom.analysis.dialectUpdFiltered')}</span>
            <span className="font-mono font-semibold text-primary">
              {formatNum(d.vendor_upd_filtered)}
            </span>
          </div>
          <div className="flex items-center justify-between rounded-lg bg-surface-muted/50 px-3 py-1.5">
            <span>{t('gedcom.analysis.dialectMarriedNames')}</span>
            <span className="font-mono font-semibold text-primary">
              {formatNum(d.married_names_resolved)}
            </span>
          </div>
          <div className="flex items-center justify-between rounded-lg bg-surface-muted/50 px-3 py-1.5">
            <span>{t('gedcom.analysis.dialectParentalAsso')}</span>
            <span className="font-mono font-semibold text-primary">
              {formatNum(d.parental_asso_deduplicated)}
            </span>
          </div>
          <div className="flex items-center justify-between rounded-lg bg-surface-muted/50 px-3 py-1.5">
            <span>{t('gedcom.analysis.dialectMissingMediaStubs')}</span>
            <span className="font-mono font-semibold text-primary">
              {formatNum(d.missing_media_stubs)}
            </span>
          </div>
        </div>
      </div>

      {/* Warnings List */}
      {report.warnings.length > 0 && (
        <Alert
          variant="warning"
          title={t('gedcom.analysis.warningsTitle', {
            count: report.warnings.length,
          })}
        >
          <ul className="list-disc pl-4 space-y-1 text-xs">
            {report.warnings.slice(0, 6).map((warning, idx) => (
              <li key={idx}>{formatGedcomWarning(warning, t)}</li>
            ))}
          </ul>
        </Alert>
      )}

      {/* Import Mode Selector (Merge & Enrich vs Full Replace) */}
      <div className="space-y-2.5" data-testid="gedcom-import-mode-section">
        <div className="text-xs font-semibold uppercase tracking-wider text-secondary">
          {t('gedcom.analysis.modeSectionTitle')}
        </div>

        <RadioCardGroup<GedcomUiImportMode>
          ariaLabel={t('gedcom.analysis.modeSectionTitle')}
          value={effectiveMode}
          onChange={handleSelectMode}
          disabled={isBusy}
          options={[
            {
              value: 'merge',
              dataTestId: 'gedcom-mode-merge',
              icon: <GitMerge className="h-4 w-4" />,
              label: (
                <span className="flex flex-wrap items-center gap-1.5">
                  <span>{t('gedcom.analysis.modeMergeTitle')}</span>
                  <Badge variant="success" size="sm">
                    {t('gedcom.analysis.modeMergeBadgeRecommended')}
                  </Badge>
                  <Badge variant="brand" size="sm">
                    {t('gedcom.analysis.modeMergeBadgeSafe')}
                  </Badge>
                </span>
              ),
              description: t('gedcom.analysis.modeMergeDesc'),
            },
            {
              value: 'clean_slate',
              dataTestId: 'gedcom-mode-clean-slate',
              icon: <ShieldAlert className="h-4 w-4" />,
              label: (
                <span className="flex flex-wrap items-center gap-1.5">
                  <span>{t('gedcom.analysis.modeCleanSlateTitle')}</span>
                  <Badge variant="danger" size="sm">
                    {t('gedcom.analysis.modeCleanSlateBadge')}
                  </Badge>
                </span>
              ),
              description: t('gedcom.analysis.modeCleanSlateDesc'),
            },
          ]}
        />
      </div>

      {/* Dynamic Safety Notice Box */}
      <Alert
        variant={effectiveMode === 'merge' ? 'info' : 'warning'}
        title={
          effectiveMode === 'merge'
            ? t('gedcom.analysis.safetyMergeTitle')
            : t('gedcom.analysis.safetyCleanSlateTitle')
        }
      >
        {effectiveMode === 'merge'
          ? t('gedcom.analysis.safetyMergeBody')
          : t('gedcom.analysis.safetyCleanSlateBody')}
      </Alert>
    </div>
  );

  if (asDialog) {
    return (
      <Dialog
        open={open}
        onOpenChange={(nextOpen) => {
          onOpenChange?.(nextOpen);
          if (!nextOpen) onCancel();
        }}
        size="xl"
        title={t('gedcom.analysis.title')}
        description={t('gedcom.analysis.subtitle')}
        footer={
          <>
            <Button variant="secondary" onClick={onCancel} disabled={isBusy}>
              {t('common.cancel')}
            </Button>
            <Button
              variant="primary"
              onClick={handlePrimaryAction}
              loading={isBusy}
              data-testid="gedcom-proceed-import-btn"
            >
              {primaryBtnText}
            </Button>
          </>
        }
      >
        {content}
      </Dialog>
    );
  }

  return (
    <Card className="border-brand/40">
      <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
        <div>
          <h3 className="text-sm font-semibold text-primary">
            {t('gedcom.analysis.title')}
          </h3>
          <p className="text-xs text-secondary">
            {t('gedcom.analysis.subtitle')}
          </p>
        </div>
        <Badge
          variant="accent"
          size="sm"
          icon={<AlertTriangle className="h-3 w-3" />}
        >
          {t('gedcom.analysis.dryRunBadge')}
        </Badge>
      </CardHeader>
      <CardBody>{content}</CardBody>
      <CardFooter className="flex items-center justify-end gap-2">
        <Button variant="secondary" size="sm" onClick={onCancel} disabled={isBusy}>
          {t('common.cancel')}
        </Button>
        <Button
          variant="primary"
          size="sm"
          onClick={handlePrimaryAction}
          loading={isBusy}
          data-testid="gedcom-proceed-import-btn"
        >
          {primaryBtnText}
        </Button>
      </CardFooter>
    </Card>
  );
};
