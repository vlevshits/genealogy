import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Eye, Printer } from 'lucide-react';
import {
  AccessDeniedCard,
  SegmentedControl,
  Spinner,
  useToast,
} from '../components/ui';
import {
  PrintConfigSidebar,
  PrintPreviewCanvas,
  PrintSummaryBar,
} from '../components/print';
import {
  exportPrintablePdf,
  fetchPdfPreviewInfo,
  fetchPeopleDirectory,
  PdfExportPreviewInfo,
  PdfExportRequest,
} from '../api/client';
import { Individual } from '../types/person';
import { useAuth } from '../lib/auth-context';
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';
import { cn } from '../lib/utils';

export interface PrintExportPageProps {
  initialConfig?: PdfExportRequest;
  initialPreview?: PdfExportPreviewInfo | null;
  userRole?: 'admin' | 'editor' | 'viewer';
  onNavigate?: (path: string) => void;
  className?: string;
}

const DEFAULT_CONFIG: PdfExportRequest = {
  view_mode: 'pedigree',
  generations: 5,
  theme: 'modern',
  format: 'tiled',
  target_sheet_size: 'A4',
  overlap_mm: 10,
  show_crop_marks: true,
  show_photos: true,
  redact_living: false,
  include_partner: true,
  include_siblings: false,
  collateral: true,
  include_assembly_guide: false,
  title: '',
  subtitle: '',
};

export const PrintExportPage: React.FC<PrintExportPageProps> = ({
  initialConfig,
  initialPreview = null,
  userRole,
  onNavigate,
  className,
}) => {
  const { t } = useI18n();
  const { user, isLoading: isAuthLoading } = useAuth();
  const { addToast } = useToast();

  const effectiveRole = userRole ?? user?.role ?? 'admin';
  const isAdmin = effectiveRole === 'admin';

  const [config, setConfig] = useState<PdfExportRequest>(() => ({
    ...DEFAULT_CONFIG,
    ...initialConfig,
  }));
  const [previewInfo, setPreviewInfo] = useState<PdfExportPreviewInfo | null>(
    initialPreview
  );
  const [people, setPeople] = useState<Individual[]>([]);
  const [focalPerson, setFocalPerson] = useState<Individual | null>(null);
  const [isLoadingPreview, setIsLoadingPreview] = useState<boolean>(false);
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [mobileTab, setMobileTab] = useState<'config' | 'preview'>('config');

  const previewAbortRef = useRef<AbortController | null>(null);

  // Load initial candidates for focal person
  useEffect(() => {
    let active = true;
    fetchPeopleDirectory({ limit: 15 })
      .then((res) => {
        if (!active) return;
        const mapped: Individual[] = res.items.map((item) => ({
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
        setPeople(mapped);

        if (mapped.length > 0 && !config.focal_person_id) {
          const first = mapped[0];
          setConfig((prev) => ({ ...prev, focal_person_id: first.id }));
          setFocalPerson(first);
        }
      })
      .catch(() => {});

    return () => {
      active = false;
    };
  }, []);

  // Fetch preview info whenever config changes (with debounce)
  useEffect(() => {
    if (!isAdmin) return;

    if (previewAbortRef.current) {
      previewAbortRef.current.abort();
    }
    const controller = new AbortController();
    previewAbortRef.current = controller;

    setIsLoadingPreview(true);

    const timer = setTimeout(() => {
      fetchPdfPreviewInfo(config)
        .then((info) => {
          if (!controller.signal.aborted) {
            setPreviewInfo(info);
          }
        })
        .catch(() => {
          // Fallback to local calculation if server fails/unreachable
          if (!controller.signal.aborted) {
            const isPoster = config.format === 'poster';
            const cols = isPoster ? 1 : 2;
            const rows = isPoster ? 1 : 2;
            const posterSizes: Record<string, { w: number; h: number }> = {
              A0: { w: 1189, h: 841 },
              A1: { w: 841, h: 594 },
              A2: { w: 594, h: 420 },
              A3: { w: 420, h: 297 },
            };
            const posterSheet = posterSizes[config.target_sheet_size ?? 'A2'] ?? { w: 594, h: 420 };
            const sheetW = isPoster ? posterSheet.w : 297;
            const sheetH = isPoster ? posterSheet.h : 210;
            const canvasW = 594;
            const canvasH = 420;
            const frameMargin = 12.0;
            const printableW = sheetW - 2 * frameMargin;
            const printableH = sheetH - 2 * frameMargin;
            const scale = Math.min(1.0, Math.min(printableW / canvasW, printableH / canvasH));
            const offX = frameMargin + (printableW - canvasW * scale) / 2.0;
            const offY = frameMargin + (printableH - canvasH * scale) / 2.0;

            setPreviewInfo({
              total_sheets: cols * rows,
              grid_rows: rows,
              grid_cols: cols,
              canvas_width_mm: isPoster ? 594 : 594,
              canvas_height_mm: isPoster ? 420 : 420,
              sheet_width_mm: isPoster ? sheetW : 297,
              sheet_height_mm: isPoster ? sheetH : 210,
              overlap_mm: isPoster ? 0 : config.overlap_mm ?? 10,
              format: config.format || 'tiled',
              target_sheet_size: config.target_sheet_size || (isPoster ? 'A2' : 'A4'),
              total_persons: 15,
              summary_text: isPoster ? '1 Poster Sheet' : '4 Sheets (2×2 Grid)',
              poster_layout: isPoster
                ? {
                    sheet_width_mm: sheetW,
                    sheet_height_mm: sheetH,
                    scale,
                    offset_x: offX,
                    offset_y: offY,
                    frame_margin: frameMargin,
                  }
                : undefined,
            });
          }
        })
        .finally(() => {
          if (!controller.signal.aborted) {
            setIsLoadingPreview(false);
          }
        });
    }, 250);

    return () => {
      clearTimeout(timer);
      controller.abort();
    };
  }, [config, isAdmin]);

  const handleExport = useCallback(async () => {
    if (isExporting) return;
    setIsExporting(true);

    try {
      const blob = await exportPrintablePdf(config);

      // Trigger browser download
      const filename =
        config.format === 'poster'
          ? 'family-tree-poster.pdf'
          : 'family-tree-tiled.pdf';

      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);

      addToast({
        type: 'success',
        title: t('print.downloadSuccess'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('print.downloadError'),
        description: getLocalizedErrorMessage(err, t),
      });
    } finally {
      setIsExporting(false);
    }
  }, [config, isExporting, addToast, t]);

  // Auth loading state
  if (isAuthLoading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <Spinner size="lg" />
      </div>
    );
  }

  // RBAC Role Guard: Admin only
  if (!isAdmin) {
    return (
      <div className="max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <AccessDeniedCard
          currentRole={effectiveRole}
          onOpenProfile={() => onNavigate?.('/settings/profile')}
          onBackToTree={() => onNavigate?.('/tree')}
        />
      </div>
    );
  }

  return (
    <div
      data-testid="print-export-page"
      className={cn('flex flex-col lg:flex-row h-[calc(100dvh-4rem)] w-full overflow-hidden', className)}
    >
      {/* Mobile/Tablet Segmented Toggle (< 1024px) */}
      <div className="lg:hidden shrink-0 border-b border-border bg-surface px-4 py-2.5 flex justify-center">
        <SegmentedControl<'config' | 'preview'>
          options={[
            {
              value: 'config',
              label: t('print.tabConfig'),
              icon: <Printer className="h-4 w-4" />,
            },
            {
              value: 'preview',
              label: t('print.tabPreview'),
              icon: <Eye className="h-4 w-4" />,
            },
          ]}
          value={mobileTab}
          onChange={setMobileTab}
          size="sm"
          ariaLabel={t('print.mobileViewToggle')}
        />
      </div>

      {/* Left Configuration Sidebar */}
      <PrintConfigSidebar
        config={config}
        onChange={setConfig}
        onExport={handleExport}
        isExporting={isExporting}
        people={people}
        onSelectFocalPerson={(_id, person) => {
          if (person) setFocalPerson(person);
        }}
        className={cn(mobileTab === 'config' ? 'flex' : 'hidden lg:flex')}
      />

      {/* Right Content Area: Summary Bar + Live Preview Canvas */}
      <div
        className={cn(
          'flex-1 flex-col min-w-0 h-full overflow-hidden',
          mobileTab === 'preview' ? 'flex' : 'hidden lg:flex'
        )}
      >
        <PrintSummaryBar
          previewInfo={previewInfo}
          config={config}
          isLoading={isLoadingPreview}
        />

        <PrintPreviewCanvas
          config={config}
          previewInfo={previewInfo}
          focalPerson={focalPerson}
          isLoading={isLoadingPreview}
          onExport={handleExport}
          isExporting={isExporting}
        />
      </div>
    </div>
  );
};

export default PrintExportPage;
