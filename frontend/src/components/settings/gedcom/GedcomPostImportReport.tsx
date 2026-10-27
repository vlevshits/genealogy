import React from 'react';
import {
  CheckCircle2,
  FolderGit2,
  History,
  Image as ImageIcon,
  MapPin,
  RotateCcw,
  Sparkles,
  Upload,
  UserCheck,
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
  CopyButton,
} from '../../ui';
import { GedcomImportResult } from '../../../api/client';
import { useI18n } from '../../../i18n';

export interface GedcomPostImportReportProps {
  result: GedcomImportResult;
  onOpenTree?: () => void;
  onSelectHomePerson?: () => void;
  onUploadMissingMedia?: () => void;
  onReset?: () => void;
}

export const GedcomPostImportReport: React.FC<GedcomPostImportReportProps> = ({
  result,
  onOpenTree,
  onSelectHomePerson,
  onUploadMissingMedia,
  onReset,
}) => {
  const { t, locale } = useI18n();
  const formatNum = (n: number) =>
    n.toLocaleString(locale === 'ru' ? 'ru-RU' : 'en-US');

  return (
    <Card className="border-success/40" data-testid="gedcom-post-import-report">
      <CardHeader className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
        <div className="flex items-center gap-2">
          <CheckCircle2 className="h-5 w-5 text-success shrink-0" />
          <div>
            <h3 className="text-sm font-semibold text-primary">
              {t('gedcom.report.title')}
            </h3>
            <p className="text-xs text-secondary">
              {t('gedcom.report.subtitle')}
            </p>
          </div>
        </div>

        <Badge variant="success" size="sm">
          {t('gedcom.report.completedBadge')}
        </Badge>
      </CardHeader>

      <CardBody className="space-y-4">
        {/* Imported Counts Grid */}
        <div className="grid grid-cols-2 sm:grid-cols-5 gap-3">
          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <Users className="h-3.5 w-3.5 text-brand" />
              <span>{t('gedcom.analysis.statIndividuals')}</span>
            </div>
            <div className="font-mono text-lg font-semibold text-primary">
              {formatNum(result.imported_individuals)}
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <FolderGit2 className="h-3.5 w-3.5 text-brand" />
              <span>{t('gedcom.analysis.statFamilies')}</span>
            </div>
            <div className="font-mono text-lg font-semibold text-primary">
              {formatNum(result.imported_families)}
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <Sparkles className="h-3.5 w-3.5 text-brand" />
              <span>{t('gedcom.analysis.statEvents')}</span>
            </div>
            <div className="font-mono text-lg font-semibold text-primary">
              {formatNum(result.imported_events)}
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 space-y-1">
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <MapPin className="h-3.5 w-3.5 text-brand" />
              <span>{t('gedcom.analysis.statPlaces')}</span>
            </div>
            <div className="font-mono text-lg font-semibold text-primary">
              {formatNum(result.imported_places)}
            </div>
          </div>

          <div className="rounded-xl border border-border bg-surface-muted/40 p-3 space-y-1 col-span-2 sm:col-span-1">
            <div className="flex items-center gap-1.5 text-xs text-secondary">
              <ImageIcon className="h-3.5 w-3.5 text-brand" />
              <span>{t('gedcom.analysis.statMedia')}</span>
            </div>
            <div className="font-mono text-lg font-semibold text-primary">
              {formatNum(result.imported_media)}
            </div>
          </div>
        </div>

        {/* Automatic Rollback Checkpoint Snapshot ID */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 rounded-xl border border-border bg-canvas p-3">
          <div className="flex items-center gap-2.5 min-w-0">
            <History className="h-4 w-4 text-brand shrink-0" />
            <div className="min-w-0">
              <div className="text-xs font-medium text-secondary">
                {t('gedcom.report.snapshotLabel')}
              </div>
              <div className="font-mono text-xs font-semibold text-primary truncate">
                {result.snapshot_id}
              </div>
            </div>
          </div>
          <CopyButton value={result.snapshot_id} />
        </div>

        {/* Post-Import Media Reconciliation Prompt (03 §3.5) */}
        {result.missing_media_count > 0 && (
          <Alert
            variant="warning"
            title={t('gedcom.report.missingMediaTitle', {
              count: formatNum(result.missing_media_count),
            })}
          >
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-1">
              <span className="text-xs">
                {t('gedcom.report.missingMediaDesc', {
                  count: formatNum(result.missing_media_count),
                })}
              </span>
              {onUploadMissingMedia && (
                <Button
                  variant="secondary"
                  size="sm"
                  leftIcon={<Upload className="h-3.5 w-3.5" />}
                  onClick={onUploadMissingMedia}
                  className="shrink-0"
                  data-testid="gedcom-upload-missing-media-btn"
                >
                  {t('gedcom.report.uploadMediaFolderBtn')}
                </Button>
              )}
            </div>
          </Alert>
        )}

        {/* Warnings if any */}
        {result.warnings.length > 0 && (
          <Alert
            variant="info"
            title={t('gedcom.analysis.warningsTitle', {
              count: result.warnings.length,
            })}
          >
            <ul className="list-disc pl-4 space-y-1 text-xs">
              {result.warnings.slice(0, 5).map((w, idx) => (
                <li key={idx}>{w}</li>
              ))}
            </ul>
          </Alert>
        )}
      </CardBody>

      <CardFooter className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap items-center gap-2">
          {onReset && (
            <Button
              variant="ghost"
              size="sm"
              leftIcon={<RotateCcw className="h-3.5 w-3.5" />}
              onClick={onReset}
            >
              {t('gedcom.report.resetBtn')}
            </Button>
          )}

          {result.prompt_home_person && onSelectHomePerson && (
            <Button
              variant="secondary"
              size="sm"
              leftIcon={<UserCheck className="h-3.5 w-3.5" />}
              onClick={onSelectHomePerson}
              data-testid="gedcom-select-home-person-btn"
            >
              {t('gedcom.report.selectHomePersonBtn')}
            </Button>
          )}
        </div>

        {onOpenTree && (
          <Button
            variant="primary"
            size="sm"
            leftIcon={<FolderGit2 className="h-3.5 w-3.5" />}
            onClick={onOpenTree}
            data-testid="gedcom-open-tree-btn"
          >
            {t('gedcom.report.openTreeBtn')}
          </Button>
        )}
      </CardFooter>
    </Card>
  );
};
