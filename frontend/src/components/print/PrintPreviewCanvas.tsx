import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  Download,
  Eye,
  FileText,
  Layout,
  Maximize2,
  RotateCcw,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import { Button, IconButton, SegmentedControl, Spinner } from '../ui';
import {
  exportPrintablePdf,
  PdfExportPreviewInfo,
  PdfExportRequest,
} from '../../api/client';
import { Individual } from '../../types/person';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface PrintPreviewCanvasProps {
  config: PdfExportRequest;
  previewInfo?: PdfExportPreviewInfo | null;
  focalPerson?: Individual | null;
  focalPersonName?: string;
  isLoading?: boolean;
  onExport?: () => void;
  isExporting?: boolean;
  className?: string;
}

import {
  FallbackNode,
  PrintPosterSvgSheet,
} from "./PrintPosterSvgSheet";

export const PrintPreviewCanvas: React.FC<PrintPreviewCanvasProps> = ({
  config,
  previewInfo,
  focalPerson,
  focalPersonName,
  isLoading = false,
  onExport,
  isExporting = false,
  className,
}) => {
  const { t, locale } = useI18n();
  const [zoom, setZoom] = useState<number>(1.0);
  const [viewMode, setViewMode] = useState<'layout' | 'pdf'>('layout');
  const [pdfBlobUrl, setPdfBlobUrl] = useState<string | null>(null);
  const [isLoadingPdf, setIsLoadingPdf] = useState<boolean>(false);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const isHeritage = config.theme === 'heritage';
  const isPoster =
    config.format === 'poster' ||
    previewInfo?.format?.toLowerCase() === 'poster';

  const gridCols = previewInfo?.grid_cols ?? 2;
  const gridRows = previewInfo?.grid_rows ?? 2;
  const totalSheets = isPoster ? 1 : previewInfo?.total_sheets ?? gridCols * gridRows;

  // Resolved Focal Person Name
  const resolvedName = useMemo(() => {
    if (focalPersonName) return focalPersonName;
    if (focalPerson) {
      return [focalPerson.surname, focalPerson.given_name, focalPerson.patronymic]
        .filter(Boolean)
        .join(' ');
    }
    return locale === 'ru' ? 'Александр Петров' : 'Aleksandr Petrov';
  }, [focalPersonName, focalPerson, locale]);

  // Chart Title
  const chartTitle = useMemo(() => {
    if (previewInfo?.title && previewInfo.title.trim()) {
      return previewInfo.title.trim();
    }
    if (config.title && config.title.trim()) {
      return config.title.trim();
    }
    if (config.view_mode === 'descendants') {
      return t('print.treeDescendantsTitle', { name: resolvedName });
    }
    if (config.view_mode === 'hourglass') {
      return t('print.treeHourglassTitle', { name: resolvedName });
    }
    if (config.view_mode === 'all_nodes') {
      return t('print.layoutAllNodes');
    }
    return t('print.treeAncestorsTitle', { name: resolvedName });
  }, [previewInfo?.title, config.title, config.view_mode, resolvedName, t]);

  const subtitle = useMemo(() => {
    if (previewInfo?.subtitle && previewInfo.subtitle.trim()) {
      return previewInfo.subtitle.trim();
    }
    if (config.subtitle && config.subtitle.trim()) {
      return config.subtitle.trim();
    }
    if (config.view_mode === 'all_nodes') {
      return t('print.allNodesDesc');
    }
    const genText = t('print.generationsCount', { count: config.generations ?? 5 });
    const sheetText = isPoster
      ? t('print.singleSheet')
      : t('print.sheetsGrid', {
          sheets: totalSheets,
          cols: gridCols,
          rows: gridRows,
        });
    return `${genText} • ${sheetText} (${config.target_sheet_size || 'A4'})`;
  }, [previewInfo?.subtitle, config.subtitle, config.view_mode, config.generations, isPoster, totalSheets, gridCols, gridRows, config.target_sheet_size, t]);

  // Fallback poster layout memo when isPoster is true
  const posterLayout = useMemo(() => {
    if (!isPoster) return null;
    if (previewInfo?.poster_layout) return previewInfo.poster_layout;

    const posterSizes: Record<string, { w: number; h: number }> = {
      A0: { w: 1189, h: 841 },
      A1: { w: 841, h: 594 },
      A2: { w: 594, h: 420 },
      A3: { w: 420, h: 297 },
    };
    const sheet =
      previewInfo?.format === 'poster' && previewInfo?.sheet_width_mm && previewInfo?.sheet_height_mm
        ? { w: previewInfo.sheet_width_mm, h: previewInfo.sheet_height_mm }
        : posterSizes[config.target_sheet_size ?? 'A2'] ?? { w: 594, h: 420 };
    const frameMargin = 12.0;
    const printableW = sheet.w - 2 * frameMargin;
    const printableH = sheet.h - 2 * frameMargin;
    const rawW = previewInfo?.canvas_width_mm ?? 500;
    const rawH = previewInfo?.canvas_height_mm ?? 300;
    const scale = Math.min(1.0, Math.min(printableW / rawW, printableH / rawH));
    const offX = frameMargin + (printableW - rawW * scale) / 2.0;
    const offY = frameMargin + (printableH - rawH * scale) / 2.0;
    return {
      sheet_width_mm: sheet.w,
      sheet_height_mm: sheet.h,
      scale,
      offset_x: offX,
      offset_y: offY,
      frame_margin: frameMargin,
    };
  }, [
    isPoster,
    previewInfo?.poster_layout,
    previewInfo?.canvas_width_mm,
    previewInfo?.canvas_height_mm,
    previewInfo?.sheet_width_mm,
    previewInfo?.sheet_height_mm,
    previewInfo?.format,
    config.target_sheet_size,
  ]);

  // Dimensions in Millimeters
  const mmDimensions = useMemo(() => {
    if (isPoster) {
      if (posterLayout) {
        return {
          w: posterLayout.sheet_width_mm,
          h: posterLayout.sheet_height_mm,
        };
      }
      const posterSizes: Record<string, { w: number; h: number }> = {
        A0: { w: 1189, h: 841 },
        A1: { w: 841, h: 594 },
        A2: { w: 594, h: 420 },
        A3: { w: 420, h: 297 },
      };
      return posterSizes[config.target_sheet_size ?? 'A2'] ?? { w: 594, h: 420 };
    }

    if (previewInfo?.canvas_width_mm && previewInfo?.canvas_height_mm) {
      return {
        w: previewInfo.canvas_width_mm,
        h: previewInfo.canvas_height_mm,
      };
    }

    const isLetter = config.target_sheet_size === 'Letter';
    const singleW = isLetter ? 279.4 : 297;
    const singleH = isLetter ? 215.9 : 210;
    return {
      w: singleW * gridCols,
      h: singleH * gridRows,
    };
  }, [isPoster, posterLayout, previewInfo, config.target_sheet_size, gridCols, gridRows]);

  // Scaled pixel dimensions for screen wrapper
  const pixelDimensions = useMemo(() => {
    const aspectRatio = mmDimensions.w / mmDimensions.h;
    const isLandscape = aspectRatio >= 1;
    let width: number;
    let height: number;

    if (isLandscape) {
      width = 760;
      height = Math.round(760 / aspectRatio);
    } else {
      height = 760;
      width = Math.round(760 * aspectRatio);
    }

    return { width, height, aspectRatio, isLandscape };
  }, [mmDimensions]);

  // Check if server returned layout data
  const hasServerCards = Boolean(previewInfo?.cards && previewInfo.cards.length > 0);

  // Fallback nodes if server cards are not yet loaded (e.g. initial render or stub tests)
  const fallbackNodes = useMemo<FallbackNode[]>(() => {
    if (hasServerCards) return [];
    const isRu = locale === 'ru';
    return [
      { id: 'f1', name: resolvedName, generation: 1, birthYear: 1985, isLiving: true, sex: 'M' },
      { id: 'p1', name: isRu ? 'Пётр Фёдорович Петров' : 'Pyotr Fyodorovich Petrov', generation: 2, birthYear: 1958, deathYear: 2020, isLiving: false, sex: 'M' },
      { id: 'p2', name: isRu ? 'Елена Ивановна Петрова' : 'Elena Ivanovna Petrova', generation: 2, birthYear: 1962, isLiving: true, sex: 'F' },
      { id: 'g1', name: isRu ? 'Фёдор Михайлович Петров' : 'Fyodor Mikhailovich Petrov', generation: 3, birthYear: 1928, deathYear: 1999, isLiving: false, sex: 'M' },
      { id: 'g2', name: isRu ? 'Вера Николаевна Петрова' : 'Vera Nikolayevna Petrova', generation: 3, birthYear: 1932, deathYear: 2011, isLiving: false, sex: 'F' },
      { id: 'g3', name: isRu ? 'Иван Сергеевич Морозов' : 'Ivan Sergeyevich Morozov', generation: 3, birthYear: 1935, deathYear: 2004, isLiving: false, sex: 'M' },
      { id: 'g4', name: isRu ? 'Анна Дмитриевна Морозова' : 'Anna Dmitriyevna Morozova', generation: 3, birthYear: 1938, deathYear: 2018, isLiving: false, sex: 'F' },
    ];
  }, [hasServerCards, resolvedName, locale]);

  // Zoom handlers
  const handleZoomIn = () => setZoom((z) => Math.min(2.0, +(z + 0.15).toFixed(2)));
  const handleZoomOut = () => setZoom((z) => Math.max(0.3, +(z - 0.15).toFixed(2)));
  const handleZoomReset = () => setZoom(1.0);
  const handleZoomFit = useCallback(() => {
    if (!containerRef.current) return;
    const padding = 64;
    const availWidth = Math.max(containerRef.current.clientWidth - padding, 200);
    const availHeight = Math.max(containerRef.current.clientHeight - padding, 200);
    const scaleX = availWidth / pixelDimensions.width;
    const scaleY = availHeight / pixelDimensions.height;
    const fitScale = Math.min(scaleX, scaleY, 1.5);
    setZoom(Math.max(0.25, +fitScale.toFixed(2)));
  }, [pixelDimensions]);

  // Live PDF Generation & Lifecycle
  const loadLivePdf = useCallback(async () => {
    setIsLoadingPdf(true);
    try {
      const blob = await exportPrintablePdf(config);
      if (typeof window !== 'undefined' && window.URL && typeof window.URL.createObjectURL === 'function') {
        const url = window.URL.createObjectURL(blob);
        setPdfBlobUrl((prevUrl) => {
          if (prevUrl) {
            window.URL.revokeObjectURL(prevUrl);
          }
          return url;
        });
      }
    } catch {
      // Fallback
    } finally {
      setIsLoadingPdf(false);
    }
  }, [config]);

  const handleModeChange = (mode: 'layout' | 'pdf') => {
    setViewMode(mode);
    if (mode === 'pdf' && !pdfBlobUrl) {
      loadLivePdf();
    }
  };

  // Debounced live PDF refresh on config changes in PDF mode
  useEffect(() => {
    if (viewMode !== 'pdf') return;
    const timer = setTimeout(() => {
      loadLivePdf();
    }, 300);
    return () => clearTimeout(timer);
  }, [viewMode, config, loadLivePdf]);

  // Revoke blob URL on unmount
  useEffect(() => {
    return () => {
      if (pdfBlobUrl && typeof window !== 'undefined' && window.URL) {
        window.URL.revokeObjectURL(pdfBlobUrl);
      }
    };
  }, [pdfBlobUrl]);

  return (
    <section
      aria-label={t('print.previewTitle')}
      className={cn('flex flex-col flex-1 h-full min-w-0 bg-canvas overflow-hidden', className)}
    >
      {/* Top Preview Controls Toolbar */}
      <div className="flex flex-wrap items-center justify-between gap-2 px-3 sm:px-4 py-2.5 bg-surface border-b border-border z-10 shrink-0">
        <div className="flex items-center gap-2 text-xs font-semibold text-primary min-w-0">
          <Eye className="h-4 w-4 text-brand shrink-0" />
          <span className="truncate">{t('print.previewTitle')}</span>
          <span className="hidden xl:inline text-tertiary font-normal">
            ({t('print.previewHelp')})
          </span>
        </div>

        {/* Action Controls & View Switcher */}
        <div className="flex flex-wrap items-center gap-2">
          {isLoading && (
            <div className="flex items-center gap-1 text-secondary text-xs mr-2">
              <Spinner size="sm" />
            </div>
          )}

          {/* View Mode Toggle: Interactive Vector Layout vs Native PDF Document */}
          <div className="hidden sm:inline-flex">
            <SegmentedControl<'layout' | 'pdf'>
              size="sm"
              value={viewMode}
              onChange={handleModeChange}
              ariaLabel={t('print.mobileViewToggle')}
              options={[
                { value: 'layout', label: t('print.viewModeLayout'), icon: <Layout className="h-3 w-3" /> },
                { value: 'pdf', label: t('print.viewModePdf'), icon: <FileText className="h-3 w-3" /> },
              ]}
            />
          </div>

          {viewMode === 'layout' && (
            <div className="inline-flex items-center rounded-lg border border-border bg-surface-muted/40 p-0.5 shadow-xs">
              <IconButton
                label={t('print.zoomOut')}
                size="sm"
                variant="ghost"
                onClick={handleZoomOut}
              >
                <ZoomOut className="h-3.5 w-3.5" />
              </IconButton>
              <span className="px-2 font-mono text-xs font-medium text-secondary min-w-[3.5rem] text-center">
                {Math.round(zoom * 100)}%
              </span>
              <IconButton
                label={t('print.zoomIn')}
                size="sm"
                variant="ghost"
                onClick={handleZoomIn}
              >
                <ZoomIn className="h-3.5 w-3.5" />
              </IconButton>
              <IconButton
                label={t('print.zoomFit')}
                size="sm"
                variant="ghost"
                onClick={handleZoomFit}
              >
                <Maximize2 className="h-3.5 w-3.5" />
              </IconButton>
              <IconButton
                label={t('print.zoomReset')}
                size="sm"
                variant="ghost"
                onClick={handleZoomReset}
              >
                <RotateCcw className="h-3.5 w-3.5" />
              </IconButton>
            </div>
          )}

          {onExport && (
            <>
              <IconButton
                variant="secondary"
                size="sm"
                label={t('print.downloadPdf')}
                disabled={isExporting}
                onClick={onExport}
                className="xl:hidden"
              >
                <Download className="h-4 w-4" />
              </IconButton>
              <Button
                variant="secondary"
                size="sm"
                leftIcon={<Download className="h-3.5 w-3.5" />}
                loading={isExporting}
                disabled={isExporting}
                onClick={onExport}
                className="hidden xl:inline-flex whitespace-nowrap shrink-0"
              >
                {t('print.downloadPdf')}
              </Button>
            </>
          )}
        </div>
      </div>

      {/* Main Canvas Scroll Area */}
      <div
        ref={containerRef}
        tabIndex={0}
        role="region"
        aria-label={t('print.previewTitle')}
        className="flex-1 overflow-auto p-4 sm:p-8 flex items-center justify-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-inset"
      >
        {viewMode === 'pdf' ? (
          <div className="w-full h-full flex flex-col items-center justify-center">
            {isLoadingPdf ? (
              <div className="flex flex-col items-center gap-3 text-secondary text-sm">
                <Spinner size="lg" />
                <span>{t('print.loadingPdf')}</span>
              </div>
            ) : pdfBlobUrl ? (
              <iframe
                src={pdfBlobUrl}
                title={t('print.viewModePdf')}
                className="w-full h-full rounded-xl border border-border bg-surface shadow-2xl"
              />
            ) : null}
          </div>
        ) : (
          <div
            data-testid="print-canvas-root"
            style={{
              transform: `scale(${zoom})`,
              transformOrigin: 'center center',
              transition: 'transform 0.15s ease-out',
              width: `${pixelDimensions.width}px`,
              minHeight: `${pixelDimensions.height}px`,
            }}
            className={cn(
              'relative rounded-xl p-4 sm:p-6 transition-colors shadow-2xl select-none',
              isHeritage
                ? 'paper-sheet-heritage border-4 border-amber-800/40 font-serif'
                : 'paper-sheet-modern border border-slate-300 font-sans'
            )}
          >
            {/* SVG Visual Layout matching Server Coordinates 1:1 in Millimeters */}
            <PrintPosterSvgSheet
              config={config}
              previewInfo={previewInfo}
              mmDimensions={mmDimensions}
              isHeritage={isHeritage}
              isPoster={isPoster}
              posterLayout={posterLayout}
              chartTitle={chartTitle}
              subtitle={subtitle}
              hasServerCards={hasServerCards}
              fallbackNodes={fallbackNodes}
              gridCols={gridCols}
              gridRows={gridRows}
              totalSheets={totalSheets}
            />
          </div>
        )}
      </div>
    </section>
  );
};

