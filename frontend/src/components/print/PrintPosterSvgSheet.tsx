import React from "react";
import {
  PdfExportPreviewInfo,
  PdfExportRequest,
  PdfPosterLayoutDto,
} from "../../api/client";
import { useI18n } from "../../i18n";
import { cn } from "../../lib/utils";

export interface FallbackNode {
  id: string;
  name: string;
  generation: number;
  birthYear?: number;
  deathYear?: number;
  isLiving: boolean;
  sex: "M" | "F" | "U";
}

function truncateName(name: string, maxLen = 32): string {
  if (name.length <= maxLen) return name;
  return name.slice(0, maxLen - 1) + "…";
}

export interface PrintPosterSvgSheetProps {
  config: PdfExportRequest;
  previewInfo?: PdfExportPreviewInfo | null;
  mmDimensions: { w: number; h: number };
  isHeritage: boolean;
  isPoster: boolean;
  posterLayout?: PdfPosterLayoutDto | null;
  chartTitle: string;
  subtitle: string;
  hasServerCards: boolean;
  fallbackNodes: FallbackNode[];
  gridCols: number;
  gridRows: number;
  totalSheets: number;
}

export const PrintPosterSvgSheet: React.FC<PrintPosterSvgSheetProps> = ({
  config,
  previewInfo,
  mmDimensions,
  isHeritage,
  isPoster,
  posterLayout,
  chartTitle,
  subtitle,
  hasServerCards,
  fallbackNodes,
  gridCols,
  gridRows,
  totalSheets,
}) => {
  const { t } = useI18n();

  return (
            <svg
              viewBox={`0 0 ${mmDimensions.w} ${mmDimensions.h}`}
              className="w-full h-full block overflow-visible"
              aria-hidden="true"
            >
              {/* Paper Canvas Background Fill (always physical light paper in both light & dark mode) */}
              <rect
                width={mmDimensions.w}
                height={mmDimensions.h}
                className={isHeritage ? 'fill-paper-heritage' : 'fill-paper-modern'}
              />

              {/* Single Poster Decorative Border & Title */}
              {isPoster && posterLayout && (
                <g>
                  {/* Outer Frame */}
                  <rect
                    x={posterLayout.frame_margin}
                    y={posterLayout.frame_margin}
                    width={mmDimensions.w - 2 * posterLayout.frame_margin}
                    height={mmDimensions.h - 2 * posterLayout.frame_margin}
                    fill="none"
                    className={isHeritage ? 'stroke-amber-800/70' : 'stroke-slate-300'}
                    strokeWidth={1.2}
                  />

                  {/* Inner Frame for Heritage theme */}
                  {isHeritage && (
                    <rect
                      x={posterLayout.frame_margin + 2}
                      y={posterLayout.frame_margin + 2}
                      width={mmDimensions.w - 2 * posterLayout.frame_margin - 4}
                      height={mmDimensions.h - 2 * posterLayout.frame_margin - 4}
                      fill="none"
                      className="stroke-amber-800/40"
                      strokeWidth={0.5}
                      strokeDasharray="4 2"
                    />
                  )}

                  {/* Poster Title Banner */}
                  <text
                    x={posterLayout.offset_x + 6}
                    y={posterLayout.offset_y + 12}
                    fontSize={14 * posterLayout.scale}
                    fontWeight="bold"
                    className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-slate-900 font-sans'}
                  >
                    {chartTitle}
                  </text>
                  <text
                    x={posterLayout.offset_x + 6}
                    y={posterLayout.offset_y + 20 * posterLayout.scale}
                    fontSize={7 * posterLayout.scale}
                    className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-slate-600 font-sans'}
                  >
                    {subtitle}
                  </text>
                </g>
              )}

              {/* Tiled Mode Header Title (shifted below Sheet 1 coordinate tag y=4..11.5 so they never collide) */}
              {!isPoster && (
                <g>
                  <text
                    x={20}
                    y={23}
                    fontSize={11}
                    fontWeight="bold"
                    className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-slate-900 font-sans'}
                  >
                    {chartTitle}
                  </text>
                  <text
                    x={20}
                    y={30}
                    fontSize={5.5}
                    className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-slate-600 font-sans'}
                  >
                    {subtitle}
                  </text>
                </g>
              )}

              {/* Render Server Orthogonal Connectors */}
              {hasServerCards &&
                previewInfo?.connectors?.map((conn, idx) => {
                  if (conn.points.length < 2) return null;
                  const scale = isPoster && posterLayout ? posterLayout.scale : 1.0;
                  const offX = isPoster && posterLayout ? posterLayout.offset_x : 0.0;
                  const offY = isPoster && posterLayout ? posterLayout.offset_y : 0.0;

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
                          : 'stroke-slate-300'
                      }
                      strokeWidth={(isMarriageConnector ? 1.0 : 0.75) * scale}
                      strokeDasharray={conn.is_dashed ? `${3 * scale} ${1.5 * scale}` : undefined}
                    />
                  );
                })}

              {/* Render Server Real Cards */}
              {hasServerCards &&
                previewInfo?.cards?.map((card) => {
                  const scale = isPoster && posterLayout ? posterLayout.scale : 1.0;
                  const offX = isPoster && posterLayout ? posterLayout.offset_x : 0.0;
                  const offY = isPoster && posterLayout ? posterLayout.offset_y : 0.0;

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
                            : 'fill-sky-50 stroke-sky-700'
                            : isHeritage
                            ? 'fill-amber-50/95 stroke-amber-800/50'
                            : 'fill-white stroke-slate-300'
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
                            : 'fill-slate-400'
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
                            : 'fill-slate-100 stroke-slate-300'
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
                                : 'fill-slate-500'
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
                                : 'stroke-slate-500'
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
                        className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-slate-900 font-sans'}
                      >
                        {truncateName(card.full_name, 32)}
                      </text>

                      {/* Lifespan Text */}
                      <text
                        x={14 * scale}
                        y={h / 2 + 3.2 * scale}
                        fontSize={2.1 * scale}
                        className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-slate-600 font-sans'}
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
                    const scale = isPoster && posterLayout ? posterLayout.scale : 1.0;
                    const offX = isPoster && posterLayout ? posterLayout.offset_x : 0.0;
                    const offY = isPoster && posterLayout ? posterLayout.offset_y : 0.0;
                    const cardW = 56.0 * scale;
                    const col = gen - 1;
                    const x = offX + (20 + col * (56.0 + 16)) * scale;
                    const y = offY + 38 * scale;
                    return (
                      <text
                        key={`tier-hdr-${gen}`}
                        x={x + cardW / 2}
                        y={y}
                        textAnchor="middle"
                        fontSize={2.5 * scale}
                        fontWeight="bold"
                        className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-slate-500 font-sans uppercase tracking-wider text-[2.2px]'}
                      >
                        {t('print.generationTier', { gen })}
                      </text>
                    );
                  })}

              {/* Fallback Rendering if cards are not yet loaded */}
              {!hasServerCards &&
                fallbackNodes.map((node, i) => {
                  const scale = isPoster && posterLayout ? posterLayout.scale : 1.0;
                  const offX = isPoster && posterLayout ? posterLayout.offset_x : 0.0;
                  const offY = isPoster && posterLayout ? posterLayout.offset_y : 0.0;
                  const cardW = 56.0 * scale;
                  const cardH = 26.0 * scale;
                  const col = node.generation - 1;
                  const row = i % 3;
                  const x = offX + (20 + col * (56.0 + 16)) * scale;
                  const y = offY + (44 + row * (26.0 + 12)) * scale;
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
                    <g key={node.id} transform={`translate(${x.toFixed(1)}, ${y.toFixed(1)})`}>
                      <rect
                        width={cardW}
                        height={cardH}
                        rx={2.5 * scale}
                        className={isHeritage ? 'fill-amber-50 stroke-amber-800/40' : 'fill-white stroke-slate-300'}
                        strokeWidth={0.7 * scale}
                      />
                      <rect
                        x={0}
                        y={0}
                        width={1.6 * scale}
                        height={cardH}
                        rx={0.8 * scale}
                        className={isMale ? 'fill-lineage-male' : 'fill-lineage-female'}
                      />
                      <text
                        x={14 * scale}
                        y={cardH / 2 - 0.8 * scale}
                        fontSize={2.7 * scale}
                        fontWeight="bold"
                        className={isHeritage ? 'fill-amber-950 font-serif' : 'fill-slate-900 font-sans'}
                      >
                        {truncateName(displayName, 32)}
                      </text>
                      <text
                        x={14 * scale}
                        y={cardH / 2 + 3.2 * scale}
                        fontSize={2.1 * scale}
                        className={isHeritage ? 'fill-amber-800 font-serif' : 'fill-slate-600 font-sans'}
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
  );
};
