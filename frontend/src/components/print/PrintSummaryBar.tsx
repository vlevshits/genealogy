import React from 'react';
import {
  FileText,
  Maximize,
  Scissors,
  Shield,
  ShieldAlert,
  Users,
} from 'lucide-react';
import { Badge, Spinner } from '../ui';
import { PdfExportPreviewInfo, PdfExportRequest } from '../../api/client';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface PrintSummaryBarProps {
  previewInfo?: PdfExportPreviewInfo | null;
  config: PdfExportRequest;
  isLoading?: boolean;
  className?: string;
}

export const PrintSummaryBar: React.FC<PrintSummaryBarProps> = ({
  previewInfo,
  config,
  isLoading = false,
  className,
}) => {
  const { t } = useI18n();

  const isPoster =
    config.format === 'poster' ||
    previewInfo?.format?.toLowerCase() === 'poster';

  // Sheet count text
  const sheetSummaryText = isPoster
    ? t('print.singleSheet')
    : previewInfo
    ? t('print.sheetsGrid', {
        sheets: previewInfo.total_sheets,
        cols: previewInfo.grid_cols,
        rows: previewInfo.grid_rows,
      })
    : t('print.sheetsGrid', { sheets: 4, cols: 2, rows: 2 });

  // Dimensions
  const dimensionsText = previewInfo
    ? t('print.dimensionsMm', {
        width: Math.round(previewInfo.canvas_width_mm),
        height: Math.round(previewInfo.canvas_height_mm),
      })
    : isPoster
    ? t('print.dimensionsMm', { width: 420, height: 594 })
    : t('print.dimensionsMm', { width: 594, height: 420 });

  // Overlap
  const overlapText = isPoster
    ? t('print.noOverlap')
    : t('print.overlapValue', {
        mm: previewInfo?.overlap_mm ?? config.overlap_mm ?? 10,
      });

  return (
    <div
      data-testid="print-summary-bar"
      className={cn(
        'flex flex-wrap items-center justify-between gap-3 px-4 py-3 bg-surface border-b border-border text-xs',
        className
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        {/* Total Sheets & Grid */}
        <Badge
          variant="brand"
          size="md"
          icon={<FileText className="h-3.5 w-3.5" />}
        >
          {sheetSummaryText}
        </Badge>

        {/* Physical Dimensions */}
        <Badge
          variant="neutral"
          size="md"
          icon={<Maximize className="h-3.5 w-3.5" />}
        >
          {dimensionsText}
        </Badge>

        {/* Overlap Margin */}
        <Badge
          variant="neutral"
          size="md"
          icon={<Scissors className="h-3.5 w-3.5" />}
        >
          {overlapText}
        </Badge>

        {/* Privacy Badge */}
        <Badge
          variant={config.redact_living ? 'warning' : 'neutral'}
          size="md"
          icon={
            config.redact_living ? (
              <ShieldAlert className="h-3.5 w-3.5" />
            ) : (
              <Shield className="h-3.5 w-3.5" />
            )
          }
        >
          {config.redact_living
            ? t('print.privacyBadgeLivingMasked')
            : t('print.privacyBadgeFull')}
        </Badge>

        {/* Persons Count */}
        {previewInfo && previewInfo.total_persons > 0 && (
          <Badge
            variant="neutral"
            size="md"
            icon={<Users className="h-3.5 w-3.5" />}
          >
            {t('print.personsCount', { count: previewInfo.total_persons })}
          </Badge>
        )}
      </div>

      {/* Loading state indicator */}
      {isLoading && (
        <div className="flex items-center gap-1.5 text-secondary text-xs">
          <Spinner size="sm" />
          <span>{t('print.calculatingPreview')}</span>
        </div>
      )}
    </div>
  );
};
