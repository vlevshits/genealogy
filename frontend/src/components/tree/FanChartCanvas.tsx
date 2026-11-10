import React, { useEffect, useMemo, useState } from 'react';
import { CircleDot, Compass, Disc } from 'lucide-react';
import {
  Badge,
  Button,
  SegmentedControl,
  Select,
} from '../ui';
import {
  FanChartNodeDto,
  treeApi,
  TreeEdgeDto,
  TreePersonNodeDto,
} from '../../api/client';
import { formatTreeCardName } from './PersonNodeCard';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export type FanAngleSpan = '180' | '360';

export interface FanChartCanvasProps {
  focalPersonId: string | null;
  nodes: TreePersonNodeDto[];
  edges: TreeEdgeDto[];
  generations?: number;
  onGenerationsChange?: (nextGenerations: number) => void;
  accessTier?: 'public_anonymous' | 'viewer' | 'editor' | 'admin';
  selectedPersonId?: string | null;
  onSelectPerson?: (person: TreePersonNodeDto) => void;
  onFocusPerson?: (person: TreePersonNodeDto) => void;
}

export interface ComputedFanSlice {
  ahnentafel: number;
  generation: number;
  slotIndex: number;
  slotsInRing: number;
  quadrant:
    | 'paternal_gf'
    | 'paternal_gm'
    | 'maternal_gf'
    | 'maternal_gm'
    | 'root';
  person: TreePersonNodeDto | null;
  startAngleDeg: number;
  endAngleDeg: number;
  innerRadius: number;
  outerRadius: number;
  pathD: string;
  labelX: number;
  labelY: number;
}

function polarToCartesian(
  cx: number,
  cy: number,
  radius: number,
  angleInDegrees: number
): { x: number; y: number } {
  const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180.0;
  return {
    x: Number((cx + radius * Math.cos(angleInRadians)).toFixed(2)),
    y: Number((cy + radius * Math.sin(angleInRadians)).toFixed(2)),
  };
}

export function describeAnnularSector(
  cx: number,
  cy: number,
  innerR: number,
  outerR: number,
  startAngle: number,
  endAngle: number
): string {
  const effectiveEnd =
    endAngle - startAngle >= 359.9 ? startAngle + 359.8 : endAngle;
  const outerStart = polarToCartesian(cx, cy, outerR, effectiveEnd);
  const outerEnd = polarToCartesian(cx, cy, outerR, startAngle);
  const innerStart = polarToCartesian(cx, cy, innerR, effectiveEnd);
  const innerEnd = polarToCartesian(cx, cy, innerR, startAngle);
  const largeArcFlag = effectiveEnd - startAngle <= 180 ? '0' : '1';

  return [
    'M',
    outerStart.x,
    outerStart.y,
    'A',
    outerR,
    outerR,
    0,
    largeArcFlag,
    0,
    outerEnd.x,
    outerEnd.y,
    'L',
    innerEnd.x,
    innerEnd.y,
    'A',
    innerR,
    innerR,
    0,
    largeArcFlag,
    1,
    innerStart.x,
    innerStart.y,
    'Z',
  ].join(' ');
}

export function buildAhnentafelMapFromGraph(
  focalId: string | null,
  nodes: TreePersonNodeDto[],
  edges: TreeEdgeDto[],
  maxGen: number
): Map<number, TreePersonNodeDto> {
  const map = new Map<number, TreePersonNodeDto>();
  if (nodes.length === 0) return map;

  const nodeById = new Map<string, TreePersonNodeDto>(
    nodes.map((n) => [n.id, n])
  );
  const root =
    (focalId ? nodeById.get(focalId) : undefined) ?? nodes[0];
  if (!root) return map;

  map.set(1, root);

  const parentsOfChild = new Map<string, TreePersonNodeDto[]>();
  for (const edge of edges) {
    if (edge.kind === 'parent_child') {
      const parent = nodeById.get(edge.source);
      if (parent) {
        const list = parentsOfChild.get(edge.target) ?? [];
        list.push(parent);
        parentsOfChild.set(edge.target, list);
      }
    }
  }

  for (let gen = 1; gen < maxGen; gen += 1) {
    const startAhn = Math.pow(2, gen - 1);
    const endAhn = Math.pow(2, gen) - 1;
    for (let ahn = startAhn; ahn <= endAhn; ahn += 1) {
      const child = map.get(ahn);
      if (!child) continue;
      const parents = parentsOfChild.get(child.id) ?? [];
      const father =
        parents.find((p) => p.sex === 'M') ?? parents[0] ?? null;
      const mother =
        parents.find((p) => p.sex === 'F' && p.id !== father?.id) ??
        parents.find((p) => p.id !== father?.id) ??
        null;
      if (father) map.set(ahn * 2, father);
      if (mother) map.set(ahn * 2 + 1, mother);
    }
  }

  // Fallback demo ancestor population if graph has nodes without explicit parent edges
  if (map.size === 1 && nodes.length > 1) {
    const remaining = nodes.filter((n) => n.id !== root.id);
    const fathers = remaining.filter((n) => n.sex === 'M');
    const mothers = remaining.filter((n) => n.sex === 'F');
    if (fathers[0]) map.set(2, fathers[0]);
    if (mothers[0]) map.set(3, mothers[0]);
    if (fathers[1]) map.set(4, fathers[1]);
    if (mothers[1]) map.set(5, mothers[1]);
  }

  return map;
}

export const FanChartCanvas: React.FC<FanChartCanvasProps> = ({
  focalPersonId,
  nodes,
  edges,
  generations: propGenerations = 5,
  onGenerationsChange,
  accessTier = 'admin',
  selectedPersonId,
  onSelectPerson,
  onFocusPerson,
}) => {
  const { t, locale } = useI18n();
  const [angleSpan, setAngleSpan] = useState<FanAngleSpan>('180');
  const [localGen, setLocalGen] = useState<number>(
    Math.min(8, Math.max(4, propGenerations))
  );
  const [remoteFanNodes, setRemoteFanNodes] = useState<FanChartNodeDto[] | null>(
    null
  );

  const effectiveGen = Math.min(8, Math.max(4, propGenerations || localGen));

  useEffect(() => {
    let cancelled = false;
    treeApi
      .getFanChart(focalPersonId, effectiveGen)
      .then((res) => {
        if (!cancelled && res.nodes && res.nodes.length > 0) {
          setRemoteFanNodes(res.nodes);
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [focalPersonId, effectiveGen]);

  const ahnentafelMap = useMemo(() => {
    const base = buildAhnentafelMapFromGraph(
      focalPersonId,
      nodes,
      edges,
      effectiveGen
    );
    if (remoteFanNodes && remoteFanNodes.length > 0) {
      for (const item of remoteFanNodes) {
        if (item.person) {
          base.set(item.ahnentafel, item.person);
        }
      }
    }
    return base;
  }, [focalPersonId, nodes, edges, effectiveGen, remoteFanNodes]);

  const rootPerson = ahnentafelMap.get(1) ?? nodes[0] ?? null;

  const slices = useMemo<ComputedFanSlice[]>(() => {
    const cx = 450;
    const cy = angleSpan === '180' ? 420 : 450;
    const centerR = 66;
    const maxOuterR = 390;
    const ringCount = effectiveGen - 1;
    const ringThickness = (maxOuterR - centerR) / Math.max(1, ringCount);

    const startSweep = angleSpan === '180' ? -90 : -180;
    const totalSweep = angleSpan === '180' ? 180 : 360;

    const list: ComputedFanSlice[] = [];
    for (let gen = 2; gen <= effectiveGen; gen += 1) {
      const slotsInRing = Math.pow(2, gen - 1);
      const startAhn = slotsInRing;
      const innerR = centerR + (gen - 2) * ringThickness;
      const outerR = innerR + ringThickness - 3;
      const sliceSweep = totalSweep / slotsInRing;

      for (let i = 0; i < slotsInRing; i += 1) {
        const ahn = startAhn + i;
        const sAngle = startSweep + i * sliceSweep;
        const eAngle = sAngle + sliceSweep;
        const midAngle = (sAngle + eAngle) / 2;
        const midR = (innerR + outerR) / 2;
        const labelPt = polarToCartesian(cx, cy, midR, midAngle);

        const fraction = (i + 0.5) / slotsInRing;
        const quadrant: ComputedFanSlice['quadrant'] =
          fraction < 0.25
            ? 'paternal_gf'
            : fraction < 0.5
            ? 'paternal_gm'
            : fraction < 0.75
            ? 'maternal_gf'
            : 'maternal_gm';

        list.push({
          ahnentafel: ahn,
          generation: gen,
          slotIndex: i,
          slotsInRing,
          quadrant,
          person: ahnentafelMap.get(ahn) ?? null,
          startAngleDeg: sAngle,
          endAngleDeg: eAngle,
          innerRadius: innerR,
          outerRadius: outerR,
          pathD: describeAnnularSector(
            cx,
            cy,
            innerR,
            outerR,
            sAngle + 0.4,
            eAngle - 0.4
          ),
          labelX: labelPt.x,
          labelY: labelPt.y,
        });
      }
    }
    return list;
  }, [angleSpan, effectiveGen, ahnentafelMap]);

  const getQuadrantClasses = (
    quadrant: ComputedFanSlice['quadrant'],
    hasPerson: boolean,
    isSelected: boolean
  ): string => {
    if (!hasPerson) {
      return 'fill-surface-muted/40 stroke-border/60 hover:fill-surface-muted/70';
    }
    if (isSelected) {
      return 'fill-brand/30 stroke-brand stroke-2';
    }
    if (quadrant === 'paternal_gf') {
      return 'fill-brand/15 stroke-brand/50 hover:fill-brand/25';
    }
    if (quadrant === 'paternal_gm') {
      return 'fill-success/15 stroke-success/50 hover:fill-success/25';
    }
    if (quadrant === 'maternal_gf') {
      return 'fill-warning/15 stroke-warning/50 hover:fill-warning/25';
    }
    return 'fill-danger/15 stroke-danger/50 hover:fill-danger/25';
  };

  const rootFormatted = rootPerson
    ? formatTreeCardName(rootPerson, accessTier, locale, 'card')
    : { displayName: '', isMaskedLiving: false };

  const cx = 450;
  const cy = angleSpan === '180' ? 420 : 450;
  const viewBox =
    angleSpan === '180' ? '0 0 900 490' : '0 0 900 900';

  return (
    <div
      data-testid="fan-chart-canvas"
      className="relative flex flex-col items-center justify-center w-full h-full overflow-hidden p-4 pt-18 pb-32 sm:pt-20 sm:pb-20"
    >
      {/* Fan Chart Controls Sub-Bar */}
      <div
        onPointerDown={(e) => e.stopPropagation()}
        className="z-10 mb-3 flex flex-wrap items-center justify-center gap-3 rounded-2xl border border-border bg-surface/95 px-4 py-2 shadow-xs backdrop-blur-md"
      >
        <SegmentedControl<FanAngleSpan>
          ariaLabel={t('tree.fan.angleSpanAria')}
          size="sm"
          value={angleSpan}
          onChange={setAngleSpan}
          options={[
            {
              value: '180',
              label: t('tree.fan.span180'),
              icon: <Compass className="h-3.5 w-3.5" />,
            },
            {
              value: '360',
              label: t('tree.fan.span360'),
              icon: <Disc className="h-3.5 w-3.5" />,
            },
          ]}
        />

        <div className="w-36">
          <Select
            ariaLabel={t('tree.fan.depthSelectAria')}
            value={String(effectiveGen)}
            onChange={(val) => {
              const num = Number(val);
              setLocalGen(num);
              onGenerationsChange?.(num);
            }}
            options={[4, 5, 6, 7, 8].map((g) => ({
              value: String(g),
              label: t('tree.toolbar.generationsCount', { count: g }),
            }))}
          />
        </div>

        {/* 4 Quadrant Color Legend */}
        <div className="hidden lg:flex items-center gap-2 text-[11px] text-secondary pl-2 border-l border-border">
          <Badge variant="brand" size="sm">
            {t('tree.fan.quadrantPaternalGf')}
          </Badge>
          <Badge variant="success" size="sm">
            {t('tree.fan.quadrantPaternalGm')}
          </Badge>
          <Badge variant="warning" size="sm">
            {t('tree.fan.quadrantMaternalGf')}
          </Badge>
          <Badge variant="danger" size="sm">
            {t('tree.fan.quadrantMaternalGm')}
          </Badge>
        </div>
      </div>

      {/* Radial Sunburst Ahnentafel SVG */}
      <div className="relative flex-1 w-full max-w-4xl flex items-center justify-center">
        <svg
          viewBox={viewBox}
          role="img"
          aria-label={t('tree.fan.svgAria')}
          className="w-full h-full max-h-[70vh] select-none"
        >
          {slices.map((slice) => {
            const person = slice.person;
            const isSelected = Boolean(
              person && selectedPersonId === person.id
            );
            const formatted = person
              ? formatTreeCardName(person, accessTier, locale, 'compact')
              : null;
            const birthYear =
              person && !formatted?.isMaskedLiving && person.birth?.date
                ? person.birth.date.slice(0, 4)
                : '';
            const deathYear =
              person && !person.is_living && person.death?.date
                ? `†${person.death.date.slice(0, 4)}`
                : '';

            return (
              <g
                key={slice.ahnentafel}
                data-testid={`fan-slice-${slice.ahnentafel}`}
                role={person ? 'button' : undefined}
                tabIndex={person ? 0 : undefined}
                aria-label={
                  person
                    ? `${formatted?.displayName || person.gid} (#${slice.ahnentafel})`
                    : `${t('tree.fan.emptySlot')} (#${slice.ahnentafel})`
                }
                onClick={(e) => {
                  e.stopPropagation();
                  if (person) {
                    onSelectPerson?.(person);
                  }
                }}
                onDoubleClick={(e) => {
                  e.stopPropagation();
                  if (person) {
                    onFocusPerson?.(person);
                  }
                }}
                onKeyDown={(e) => {
                  if (person && (e.key === 'Enter' || e.key === ' ')) {
                    e.preventDefault();
                    onSelectPerson?.(person);
                  }
                }}
                className={cn(
                  'transition-colors',
                  person
                    ? 'group cursor-pointer focus:outline-none focus-visible:outline-none'
                    : 'cursor-default'
                )}
              >
                <path
                  d={slice.pathD}
                  strokeWidth={1.25}
                  className={cn(
                    getQuadrantClasses(
                      slice.quadrant,
                      Boolean(person),
                      isSelected
                    ),
                    'group-focus-visible:stroke-brand group-focus-visible:stroke-[3] group-focus-visible:outline-none'
                  )}
                />
                {person && slice.generation <= 5 && (
                  <text
                    x={slice.labelX}
                    y={slice.labelY}
                    textAnchor="middle"
                    dominantBaseline="middle"
                    className="fill-primary pointer-events-none font-sans text-[10px] font-semibold"
                  >
                    <tspan x={slice.labelX} dy="-0.3em">
                      {(formatted?.displayName || '').slice(0, 16)}
                    </tspan>
                    {(birthYear || deathYear) && (
                      <tspan
                        x={slice.labelX}
                        dy="1.2em"
                        className="fill-secondary font-mono text-[9px]"
                      >
                        {[birthYear, deathYear].filter(Boolean).join(' ')}
                      </tspan>
                    )}
                  </text>
                )}
              </g>
            );
          })}

          {/* Focal Individual Root Hub (Ahnentafel #1) */}
          {rootPerson && (
            <g
              data-testid="fan-root-node"
              role="button"
              tabIndex={0}
              onClick={(e) => {
                e.stopPropagation();
                onSelectPerson?.(rootPerson);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  onSelectPerson?.(rootPerson);
                }
              }}
              className="cursor-pointer"
            >
              <circle
                cx={cx}
                cy={cy}
                r={60}
                className="fill-surface stroke-brand stroke-2 shadow-md"
              />
              <text
                x={cx}
                y={cy}
                textAnchor="middle"
                dominantBaseline="middle"
                className="fill-primary font-sans text-xs font-bold"
              >
                <tspan x={cx} dy="-0.4em">
                  {rootFormatted.displayName.slice(0, 18)}
                </tspan>
                <tspan
                  x={cx}
                  dy="1.4em"
                  className="fill-secondary font-mono text-[10px]"
                >
                  #1 · {rootPerson.gid}
                </tspan>
              </text>
            </g>
          )}
        </svg>
      </div>

      {rootPerson && onFocusPerson && (
        <div
          onPointerDown={(e) => e.stopPropagation()}
          className="mt-2 flex items-center gap-2 text-xs text-secondary"
        >
          <CircleDot className="h-3.5 w-3.5 text-brand" />
          <span>
            {t('tree.fan.centerHint', { name: rootFormatted.displayName })}
          </span>
          {selectedPersonId && selectedPersonId !== rootPerson.id && (
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                const target = nodes.find((n) => n.id === selectedPersonId);
                if (target) onFocusPerson(target);
              }}
            >
              {t('tree.fan.setCenterBtn')}
            </Button>
          )}
        </div>
      )}
    </div>
  );
};
