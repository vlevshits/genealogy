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

interface FallbackNode {
  id: string;
  name: string;
  generation: number;
  birthYear?: number;
  deathYear?: number;
  isLiving: boolean;
  sex: 'M' | 'F' | 'U';
}

function truncateName(name: string, maxLen = 32): string {
  if (name.length <= maxLen) return name;
  return name.slice(0, maxLen - 1) + '…';
}

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

  // Dimensions in Millimeters
  const mmDimensions = useMemo(() => {
    if (isPoster) {
      if (previewInfo?.poster_layout) {
        return {
          w: previewInfo.poster_layout.sheet_width_mm,
          h: previewInfo.poster_layout.sheet_height_mm,
        };
      }
      const posterSizes: Record<string, { w: number; h: number }> = {
        A0: { w: 841, h: 1189 },
        A1: { w: 594, h: 841 },
        A2: { w: 420, h: 594 },
        A3: { w: 297, h: 420 },
      };
      return posterSizes[config.target_sheet_size ?? 'A2'] ?? { w: 420, h: 594 };
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
  }, [isPoster, previewInfo, config.target_sheet_size, gridCols, gridRows]);

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
        if (pdfBlobUrl) {
          window.URL.revokeObjectURL(pdfBlobUrl);
        }
        const url = window.URL.createObjectURL(blob);
        setPdfBlobUrl(url);
      }
    } catch {
      // Fallback
    } finally {
      setIsLoadingPdf(false);
    }
  }, [config, pdfBlobUrl]);

  const handleModeChange = (mode: 'layout' | 'pdf') => {
    setViewMode(mode);
    if (mode === 'pdf' && !pdfBlobUrl) {
      loadLivePdf();
    }
  };

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
      <div className="flex items-center justify-between gap-3 px-4 py-2.5 bg-surface border-b border-border z-10 shrink-0">
        <div className="flex items-center gap-2 text-xs font-semibold text-primary">
          <Eye className="h-4 w-4 text-brand" />
          <span>{t('print.previewTitle')}</span>
          <span className="hidden xl:inline text-tertiary font-normal">
            ({t('print.previewHelp')})
          </span>
        </div>

        {/* Action Controls & View Switcher */}
        <div className="flex items-center gap-2">
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
            <Button
              variant="secondary"
              size="sm"
              leftIcon={<Download className="h-3.5 w-3.5" />}
              loading={isExporting}
              disabled={isExporting}
              onClick={onExport}
            >
              {t('print.downloadPdf')}
            </Button>
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
                ? 'bg-amber-50/80 text-amber-950 border-4 border-amber-800/40 dark:bg-stone-900/90 dark:text-amber-100 dark:border-amber-700/50 font-serif'
                : 'bg-surface text-primary border border-border font-sans'
            )}
          >
            {/* SVG Visual Layout matching Server Coordinates 1:1 in Millimeters */}
            <svg
              viewBox={`0 0 ${mmDimensions.w} ${mmDimensions.h}`}
              className="w-full h-full block overflow-visible"
              aria-hidden="true"
            >
              {/* Paper Canvas Background Fill */}
              <rect
                width={mmDimensions.w}
                height={mmDimensions.h}
                className={isHeritage ? 'fill-amber-50/90' : 'fill-surface'}
              />

              {/* Single Poster Decorative Border & Title */}
              {isPoster && (
                <g>
                  {/* Outer Frame */}
                  <rect
                    x={previewInfo?.poster_layout?.frame_margin ?? 10}
                    y={previewInfo?.poster_layout?.frame_margin ?? 10}
                    width={mmDimensions.w - 2 * (previewInfo?.poster_layout?.frame_margin ?? 10)}
                    height={mmDimensions.h - 2 * (previewInfo?.poster_layout?.frame_margin ?? 10)}
                    fill="none"
                    className={isHeritage ? 'stroke-amber-800/70' : 'stroke-border'}
                    strokeWidth={1.2}
                  />

                  {/* Inner Frame for Heritage theme */}
                  {isHeritage && (
                    <rect
                      x={(previewInfo?.poster_layout?.frame_margin ?? 10) + 2}
                      y={(previewInfo?.poster_layout?.frame_margin ?? 10) + 2}
                      width={mmDimensions.w - 2 * (previewInfo?.poster_layout?.frame_margin ?? 10) - 4}
                      height={mmDimensions.h - 2 * (previewInfo?.poster_layout?.frame_margin ?? 10) - 4}
                      fill="none"
                      className="stroke-amber-800/40"
                      strokeWidth={0.5}
                      strokeDasharray="4 2"
                    />
                  )}

                  {/* Poster Title Banner */}
                  <text
                    x={(previewInfo?.poster_layout?.offset_x ?? 15) + 6}
                    y={(previewInfo?.poster_layout?.offset_y ?? 15) + 12}
                    fontSize={14 * (previewInfo?.poster_layout?.scale ?? 1.0)}
                    fontWeight="bold"
                    className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-primary font-sans'}
                  >
                    {chartTitle}
                  </text>
                  <text
                    x={(previewInfo?.poster_layout?.offset_x ?? 15) + 6}
                    y={(previewInfo?.poster_layout?.offset_y ?? 15) + 20 * (previewInfo?.poster_layout?.scale ?? 1.0)}
                    fontSize={7 * (previewInfo?.poster_layout?.scale ?? 1.0)}
                    className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-secondary font-sans'}
                  >
                    {subtitle}
                  </text>
                </g>
              )}

              {/* Tiled Mode Header Title */}
              {!isPoster && (
                <g>
                  <text
                    x={20}
                    y={14}
                    fontSize={11}
                    fontWeight="bold"
                    className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-primary font-sans'}
                  >
                    {chartTitle}
                  </text>
                  <text
                    x={20}
                    y={21}
                    fontSize={5.5}
                    className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-secondary font-sans'}
                  >
                    {subtitle}
                  </text>
                </g>
              )}

              {/* Render Server Orthogonal Connectors */}
              {hasServerCards &&
                previewInfo?.connectors?.map((conn, idx) => {
                  if (conn.points.length < 2) return null;
                  const scale = isPoster ? previewInfo.poster_layout?.scale ?? 1.0 : 1.0;
                  const offX = isPoster ? previewInfo.poster_layout?.offset_x ?? 0.0 : 0.0;
                  const offY = isPoster ? previewInfo.poster_layout?.offset_y ?? 0.0 : 0.0;

                  const d = conn.points
                    .map(([x, y], i) => `${i === 0 ? 'M' : 'L'} ${(offX + x * scale).toFixed(1)} ${(offY + y * scale).toFixed(1)}`)
                    .join(' ');

                  const isMarriageConnector = Boolean(
                    conn.color && (conn.color.includes('d97706') || conn.color.includes('amber'))
                  );

                  return (
                    <path
                      key={idx}
                      d={d}
                      fill="none"
                      className={
                        isMarriageConnector
                          ? 'stroke-amber-600 dark:stroke-amber-500'
                          : isHeritage
                          ? 'stroke-amber-800/60'
                          : 'stroke-border'
                      }
                      strokeWidth={(isMarriageConnector ? 1.0 : 0.75) * scale}
                      strokeDasharray={conn.is_dashed ? `${3 * scale} ${1.5 * scale}` : undefined}
                    />
                  );
                })}

              {/* Render Server Real Cards */}
              {hasServerCards &&
                previewInfo?.cards?.map((card) => {
                  const scale = isPoster ? previewInfo.poster_layout?.scale ?? 1.0 : 1.0;
                  const offX = isPoster ? previewInfo.poster_layout?.offset_x ?? 0.0 : 0.0;
                  const offY = isPoster ? previewInfo.poster_layout?.offset_y ?? 0.0 : 0.0;

                  const x = offX + card.x * scale;
                  const y = offY + card.y * scale;
                  const w = card.width * scale;
                  const h = card.height * scale;
                  const isMale = card.sex === 'M' || card.sex === 'male';
                  const isFemale = card.sex === 'F' || card.sex === 'female';

                  return (
                    <g
                      key={card.id}
                      transform={`translate(${x.toFixed(1)}, ${y.toFixed(1)})`}
                      className="cursor-default"
                    >
                      {/* Card Box */}
                      <rect
                        width={w}
                        height={h}
                        rx={2.5 * scale}
                        className={cn(
                          card.is_focal
                            ? isHeritage
                              ? 'fill-amber-100 stroke-amber-800'
                              : 'fill-brand/10 stroke-brand'
                            : isHeritage
                            ? 'fill-amber-50/95 stroke-amber-800/50'
                            : 'fill-surface stroke-border'
                        )}
                        strokeWidth={(card.is_focal ? 1.4 : 0.7) * scale}
                      />

                      {/* Sex Lineage Indicator */}
                      <rect
                        x={0}
                        y={0}
                        width={1.6 * scale}
                        height={h}
                        rx={0.8 * scale}
                        className={
                          isMale
                            ? 'fill-lineage-male'
                            : isFemale
                            ? 'fill-lineage-female'
                            : 'fill-border'
                        }
                      />

                      {/* Avatar Circle / Portrait */}
                      <clipPath id={`avatar-clip-${card.id}`}>
                        <circle
                          cx={7.5 * scale}
                          cy={h / 2}
                          r={4.2 * scale}
                        />
                      </clipPath>
                      <circle
                        cx={7.5 * scale}
                        cy={h / 2}
                        r={4.2 * scale}
                        className={
                          isMale
                            ? 'fill-lineage-male/15 stroke-lineage-male'
                            : isFemale
                            ? 'fill-lineage-female/15 stroke-lineage-female'
                            : 'fill-surface-muted stroke-border'
                        }
                        strokeWidth={0.4 * scale}
                      />
                      {card.has_avatar && config.show_photos !== false ? (
                        <image
                          href={`/api/people/${card.id}/avatar`}
                          x={(7.5 - 4.2) * scale}
                          y={h / 2 - 4.2 * scale}
                          width={8.4 * scale}
                          height={8.4 * scale}
                          clipPath={`url(#avatar-clip-${card.id})`}
                          preserveAspectRatio="xMidYMid slice"
                        />
                      ) : (
                        <>
                          <circle
                            cx={7.5 * scale}
                            cy={h / 2 - 1.2 * scale}
                            r={1.7 * scale}
                            className={
                              isMale
                                ? 'fill-lineage-male'
                                : isFemale
                                ? 'fill-lineage-female'
                                : 'fill-secondary'
                            }
                          />
                          <path
                            d={`M ${(7.5 - 2.6) * scale} ${(h / 2 + 3.0) * scale} Q ${(7.5) * scale} ${(h / 2 + 1.2) * scale} ${(7.5 + 2.6) * scale} ${(h / 2 + 3.0) * scale}`}
                            fill="none"
                            className={
                              isMale
                                ? 'stroke-lineage-male'
                                : isFemale
                                ? 'stroke-lineage-female'
                                : 'stroke-secondary'
                            }
                            strokeWidth={0.6 * scale}
                          />
                        </>
                      )}

                      {/* Name Text */}
                      <text
                        x={14 * scale}
                        y={h / 2 - 0.8 * scale}
                        fontSize={2.7 * scale}
                        fontWeight="bold"
                        className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-primary font-sans'}
                      >
                        {truncateName(card.full_name, 32)}
                      </text>

                      {/* Lifespan Text */}
                      <text
                        x={14 * scale}
                        y={h / 2 + 3.2 * scale}
                        fontSize={2.1 * scale}
                        className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-secondary font-sans'}
                      >
                        {card.lifespan}
                      </text>
                    </g>
                  );
                })}

              {/* Fallback Generation Tier Headers */}
              {!hasServerCards &&
                Array.from(new Set(fallbackNodes.map((n) => n.generation)))
                  .sort((a, b) => a - b)
                  .map((gen) => {
                    const cardW = 56.0;
                    const col = gen - 1;
                    const x = 20 + col * (cardW + 16);
                    return (
                      <text
                        key={`tier-hdr-${gen}`}
                        x={x + cardW / 2}
                        y={28}
                        textAnchor="middle"
                        fontSize={2.5}
                        fontWeight="bold"
                        className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-tertiary font-sans uppercase tracking-wider text-[2.2px]'}
                      >
                        {t('print.generationTier', { gen })}
                      </text>
                    );
                  })}

              {/* Fallback Rendering if cards are not yet loaded */}
              {!hasServerCards &&
                fallbackNodes.map((node, i) => {
                  const cardW = 56.0;
                  const cardH = 26.0;
                  const col = node.generation - 1;
                  const row = i % 3;
                  const x = 20 + col * (cardW + 16);
                  const y = 35 + row * (cardH + 12);
                  const isMale = node.sex === 'M';
                  const isMasked = Boolean(config.redact_living && node.isLiving);
                  const displayName = isMasked
                    ? `${node.name.slice(0, 1)}. (${t('print.living')})`
                    : node.name;
                  const datesText = isMasked
                    ? t('print.living')
                    : node.deathYear
                    ? `${node.birthYear} — ${node.deathYear}`
                    : `${t('print.born')} ${node.birthYear}`;

                  return (
                    <g key={node.id} transform={`translate(${x}, ${y})`}>
                      <rect
                        width={cardW}
                        height={cardH}
                        rx={2.5}
                        className={isHeritage ? 'fill-amber-50 stroke-amber-800/40' : 'fill-surface stroke-border'}
                        strokeWidth={0.7}
                      />
                      <rect
                        x={0}
                        y={0}
                        width={1.6}
                        height={cardH}
                        rx={0.8}
                        className={isMale ? 'fill-lineage-male' : 'fill-lineage-female'}
                      />
                      <text
                        x={14}
                        y={cardH / 2 - 0.8}
                        fontSize={2.7}
                        fontWeight="bold"
                        className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-primary font-sans'}
                      >
                        {truncateName(displayName, 32)}
                      </text>
                      <text
                        x={14}
                        y={cardH / 2 + 3.2}
                        fontSize={2.1}
                        className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-secondary font-sans'}
                      >
                        {datesText}
                      </text>
                    </g>
                  );
                })}

              {/* Tiled Grid, Bleed, Cut Marks & Assembly Guides */}
              {!isPoster && (
                <g data-testid="print-tiling-grid-overlay">
                  {Array.from({ length: gridRows * gridCols }).map((_, index) => {
                    const row = Math.floor(index / gridCols);
                    const col = index % gridCols;
                    const sheetNumber = index + 1;
                    const cellW = mmDimensions.w / gridCols;
                    const cellH = mmDimensions.h / gridRows;
                    const winX = col * cellW;
                    const winY = row * cellH;
                    const overlap = config.overlap_mm ?? 10;
                    const isGuide = sheetNumber === 1 && config.include_assembly_guide;

                    return (
                      <g key={index}>
                        {/* Sheet boundary dashed line */}
                        <rect
                          x={winX}
                          y={winY}
                          width={cellW}
                          height={cellH}
                          fill="none"
                          className="stroke-sky-500/60"
                          strokeWidth={0.5}
                          strokeDasharray="4 2"
                        />

                        {/* Overlap bleed margin strips */}
                        {col < gridCols - 1 && overlap > 0 && (
                          <rect
                            x={winX + cellW - overlap}
                            y={winY}
                            width={overlap}
                            height={cellH}
                            className="fill-sky-500/10 stroke-sky-500/40"
                            strokeWidth={0.3}
                            strokeDasharray="2 2"
                          />
                        )}
                        {row < gridRows - 1 && overlap > 0 && (
                          <rect
                            x={winX}
                            y={winY + cellH - overlap}
                            width={cellW}
                            height={overlap}
                            className="fill-sky-500/10 stroke-sky-500/40"
                            strokeWidth={0.3}
                            strokeDasharray="2 2"
                          />
                        )}

                        {/* Sheet coordinate tag */}
                        <g transform={`translate(${winX + 4}, ${winY + 4})`}>
                          <rect
                            width={56}
                            height={7.5}
                            rx={1.5}
                            className="fill-sky-600/90"
                          />
                          <text
                            x={28}
                            y={5.2}
                            textAnchor="middle"
                            fontSize={2.7}
                            fontWeight="bold"
                            className="fill-surface font-mono"
                          >
                            {t('print.sheetTag', {
                              num: sheetNumber,
                              total: totalSheets,
                              row: row + 1,
                              col: col + 1,
                            })}
                          </text>
                        </g>

                        {/* Assembly Guide Indicator on Sheet 1 */}
                        {isGuide && (
                          <g transform={`translate(${winX + cellW - 62}, ${winY + 4})`}>
                            <rect
                              width={58}
                              height={7.5}
                              rx={1.5}
                              className="fill-brand/90"
                            />
                            <text
                              x={29}
                              y={5.2}
                              textAnchor="middle"
                              fontSize={2.5}
                              fontWeight="bold"
                              className="fill-surface"
                            >
                              {t('print.sheetGuideTag')}
                            </text>
                          </g>
                        )}

                        {/* Corner Crosshairs (+) */}
                        {config.show_crop_marks !== false && (
                          <g aria-hidden="true" className="stroke-sky-600 dark:stroke-sky-400 font-mono select-none" strokeWidth={0.5}>
                            <text x={winX - 1.5} y={winY - 1} fontSize={4.5} className="fill-sky-600 font-bold">+</text>
                            <text x={winX + cellW - 1.5} y={winY - 1} fontSize={4.5} className="fill-sky-600 font-bold">+</text>
                            <text x={winX - 1.5} y={winY + cellH + 3.5} fontSize={4.5} className="fill-sky-600 font-bold">+</text>
                            <text x={winX + cellW - 1.5} y={winY + cellH + 3.5} fontSize={4.5} className="fill-sky-600 font-bold">+</text>
                          </g>
                        )}

                        {/* Cut Line Scissors indicator */}
                        {config.show_crop_marks !== false && (col < gridCols - 1 || row < gridRows - 1) && (
                          <g transform={`translate(${winX + cellW - 8}, ${winY + cellH - 8})`} aria-hidden="true">
                            <text fontSize={4} className="fill-sky-600 opacity-60">✂</text>
                          </g>
                        )}
                      </g>
                    );
                  })}
                </g>
              )}
            </svg>
          </div>
        )}
      </div>
    </section>
  );
};
