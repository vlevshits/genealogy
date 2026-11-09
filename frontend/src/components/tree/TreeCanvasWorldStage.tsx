import React, { useCallback, useMemo } from "react";
import { Route, UserPlus, X } from "lucide-react";
import { Badge, Button } from "../ui";
import {
  QuickAddRelation,
  TreePersonNodeDto,
  TreeProjectionMode,
  TreeViewResponse,
} from "../../api/client";
import { useI18n } from "../../i18n";
import {
  PositionedEdge,
  PositionedNode,
  TreeLayoutResult,
} from "../../lib/tree-layouts";
import { cn } from "../../lib/utils";
import { PersonNodeCard } from "./PersonNodeCard";
import { HighlightedTrace } from "./KinshipFinderDialog";

export interface TreeCanvasWorldStageProps {
  mode: TreeProjectionMode;
  layout: TreeLayoutResult;
  effectiveViewData: TreeViewResponse;
  treeView: TreeViewResponse;
  nodeMap: Map<string, TreePersonNodeDto>;
  pan: { x: number; y: number };
  zoom: number;
  isPanning: boolean;
  isAnimatingCamera: boolean;
  isMobile: boolean;
  currentLod: 0 | 1 | 2;
  canEdit: boolean;
  resolvedHomePersonId: string | null;
  selectedPersonId: string | null;
  selectedEdge: PositionedEdge | null;
  pulsingPersonId: string | null;
  highlightedPathIds: string[];
  highlightedTraces: HighlightedTrace[];
  isNodeInFrustum: (pn: PositionedNode) => boolean;
  onClearHighlights: () => void;
  onSelectEdge: (edge: PositionedEdge | null) => void;
  onSelectPerson: (id: string) => void;
  onFocusNode: (id: string) => void;
  onQuickAdd: (anchor: TreePersonNodeDto, relation: QuickAddRelation) => void;
  onEditPerson: (person: TreePersonNodeDto) => void;
}

export const TreeCanvasWorldStage: React.FC<TreeCanvasWorldStageProps> = ({
  mode,
  layout,
  effectiveViewData,
  treeView,
  nodeMap,
  pan,
  zoom,
  isPanning,
  isAnimatingCamera,
  isMobile,
  currentLod,
  canEdit,
  resolvedHomePersonId,
  selectedPersonId,
  selectedEdge,
  pulsingPersonId,
  highlightedPathIds,
  highlightedTraces,
  isNodeInFrustum,
  onClearHighlights,
  onSelectEdge,
  onSelectPerson,
  onFocusNode,
  onQuickAdd,
  onEditPerson,
}) => {
  const { t } = useI18n();

  const getTraceIndexForEdge = useCallback(
    (edge: PositionedEdge): number => {
      const checkTrace = (pids: string[]): boolean => {
        if (pids.includes(edge.sourceId) && pids.includes(edge.targetId)) {
          return true;
        }
        if (edge.kind === 'parent_child') {
          if (edge.familyId) {
            const fam = effectiveViewData.families?.find((f) => f.id === edge.familyId);
            if (fam && pids.includes(edge.targetId)) {
              const isPartner1 = fam.partner1_id && pids.includes(fam.partner1_id);
              const isPartner2 = fam.partner2_id && pids.includes(fam.partner2_id);
              if (isPartner1 || isPartner2) return true;
            }
          }
        } else if (edge.kind === 'spouse') {
          const fam = edge.familyId
            ? effectiveViewData.families?.find((f) => f.id === edge.familyId)
            : effectiveViewData.families?.find(
                (f) =>
                  (f.partner1_id === edge.sourceId && f.partner2_id === edge.targetId) ||
                  (f.partner1_id === edge.targetId && f.partner2_id === edge.sourceId)
              );
          if (fam) {
            const hasPartner =
              (fam.partner1_id && pids.includes(fam.partner1_id)) ||
              (fam.partner2_id && pids.includes(fam.partner2_id));
            const hasChild =
              fam.children?.some((c) => {
                const item = c as unknown as string | { child_id?: string; individual_id?: string };
                const cid = typeof item === 'string' ? item : item.child_id ?? item.individual_id;
                return cid ? pids.includes(cid) : false;
              }) ||
              layout.edges.some(
                (e) =>
                  e.kind === 'parent_child' &&
                  e.familyId === fam.id &&
                  pids.includes(e.targetId)
              );
            if (hasPartner && hasChild) return true;
          }
        }
        return false;
      };

      if (highlightedTraces.length > 0) {
        for (let i = 0; i < highlightedTraces.length; i++) {
          if (checkTrace(highlightedTraces[i].personIds)) {
            return i;
          }
        }
      } else if (highlightedPathIds.length >= 2 && checkTrace(highlightedPathIds)) {
        return 0;
      }
      return -1;
    },
    [highlightedTraces, highlightedPathIds, effectiveViewData.families, layout.edges]
  );

  // UX-038: Partition/sort edges so collateral edges render first (bottom),
  // direct-lineage edges render next (middle), kinship-highlighted next, and selectedEdge renders last (top).
  const sortedEdges = useMemo(() => {
    return [...layout.edges].sort((a, b) => {
      const traceIdxA = getTraceIndexForEdge(a);
      const isKinshipA = traceIdxA !== -1;
      const traceIdxB = getTraceIndexForEdge(b);
      const isKinshipB = traceIdxB !== -1;

      const isSelectedA = selectedEdge?.id === a.id;
      const isSelectedB = selectedEdge?.id === b.id;

      const scoreA = isSelectedA
        ? 100
        : isKinshipA
        ? 10 + (traceIdxA === 0 ? 3 : traceIdxA === 1 ? 2 : 1)
        : a.isDirectLineage
        ? 1
        : 0;
      const scoreB = isSelectedB
        ? 100
        : isKinshipB
        ? 10 + (traceIdxB === 0 ? 3 : traceIdxB === 1 ? 2 : 1)
        : b.isDirectLineage
        ? 1
        : 0;
      return scoreA - scoreB;
    });
  }, [layout.edges, getTraceIndexForEdge, selectedEdge]);

  return (
    <>
      {/* Floating Multi-Trace Kinship Legend HUD */}
      {(highlightedTraces.length > 0 || highlightedPathIds.length > 0) && (
        <div
          role="status"
          aria-live="polite"
          className="absolute top-28 sm:top-24 md:top-4 left-1/2 -translate-x-1/2 z-30 flex flex-wrap items-center justify-center gap-3 px-4 py-2 rounded-2xl bg-surface/90 backdrop-blur-md border border-border shadow-elevation-2 text-xs text-foreground pointer-events-auto w-max max-w-[calc(100vw-24px)] mx-auto"
        >
          <div className="flex items-center gap-1.5 font-medium text-secondary">
            <Route className="h-3.5 w-3.5 text-brand" />
            <span>{t("tree.kinship.legendTitle")}</span>
          </div>
          <div className="flex items-center gap-2.5">
            {highlightedTraces.length > 0 ? (
              highlightedTraces.map((trace, idx) => {
                const dotClass =
                  idx === 0
                    ? "bg-brand"
                    : idx === 1
                    ? "bg-amber-600 dark:bg-amber-400"
                    : "bg-purple-500";
                return (
                  <div
                    key={trace.id || idx}
                    className="flex items-center gap-1.5 font-medium"
                  >
                    <span
                      className={cn(
                        "w-2.5 h-2.5 rounded-full inline-block shrink-0",
                        dotClass
                      )}
                    />
                    <span className="text-secondary max-w-[140px] truncate">
                      {trace.label ||
                        t("tree.kinship.traceTab", {
                          index: idx + 1,
                          relationship: "",
                        })}
                    </span>
                  </div>
                );
              })
            ) : (
              <div className="flex items-center gap-1.5 font-medium">
                <span className="w-2.5 h-2.5 rounded-full inline-block shrink-0 bg-brand" />
                <span className="text-secondary">
                  {t("tree.kinship.highlightSelected")}
                </span>
              </div>
            )}
          </div>
          <Button
            variant="ghost"
            size="sm"
            leftIcon={<X className="h-3 w-3" />}
            onClick={onClearHighlights}
            className="h-8 min-h-8 px-2.5 text-xs ml-1 touch-target-expand"
          >
            {t("tree.kinship.clearHighlights")}
          </Button>
        </div>
      )}

      {mode !== "fan" && (
        <div
          style={{
            transform: `translate3d(${isPanning ? pan.x : Math.round(pan.x)}px, ${isPanning ? pan.y : Math.round(pan.y)}px, 0) scale(${zoom})`,
            transformOrigin: "0 0",
          }}
          className={
            isAnimatingCamera
              ? "absolute left-0 top-0 transition-transform duration-250 ease-out will-change-transform"
              : "absolute left-0 top-0 will-change-transform"
          }
        >
          {/* Background SVG Layer for Connector Curves & Union Bars (Dual-Layer) */}
          <svg
            role="group"
            aria-label={t('tree.connections_layer')}
            className="overflow-visible pointer-events-none absolute left-0 top-0"
            width={1}
            height={1}
          >
            {sortedEdges.map((edge) => {
              const traceIdx = getTraceIndexForEdge(edge);
              const isKinshipHighlighted = traceIdx !== -1;
              const isSelected = selectedEdge?.id === edge.id;

              // Semantic edge color classes (UX-067: collateral at 70% opacity for >= 3:1 contrast)
              let strokeClass = 'stroke-secondary/70';
              let traceDasharray: string | undefined =
                edge.kind === 'placeholder' ? '5 5' : undefined;
              let traceFilter: string | undefined = undefined;

              if (isKinshipHighlighted) {
                if (traceIdx === 0) {
                  strokeClass = 'stroke-brand';
                  traceDasharray = undefined;
                  traceFilter = 'drop-shadow(0 0 4px var(--color-brand))';
                } else if (traceIdx === 1) {
                  strokeClass = 'stroke-amber-600 dark:stroke-amber-400';
                  traceDasharray = '6 4';
                  traceFilter = 'drop-shadow(0 0 4px rgba(217, 119, 6, 0.6))';
                } else {
                  strokeClass = 'stroke-purple-500';
                  traceDasharray = '4 4';
                  traceFilter = 'drop-shadow(0 0 4px rgba(168, 85, 247, 0.6))';
                }
              } else if (edge.kind === 'spouse') {
                strokeClass = 'stroke-amber-600 dark:stroke-amber-400';
              } else if (edge.kind === 'placeholder') {
                strokeClass = 'stroke-secondary/70';
              } else if (edge.isDirectLineage) {
                strokeClass = 'stroke-brand/80';
              }

              const visualStrokeWidth = isKinshipHighlighted
                ? (traceIdx === 0 ? 3.5 : 3.0)
                : isSelected
                ? 3.0
                : edge.kind === 'spouse'
                ? 2.5
                : edge.isDirectLineage
                ? 2.5
                : 1.5;

              const edgeAriaLabel =
                edge.kind === 'spouse'
                  ? t('tree.connection.marriage')
                  : t('tree.connection.parent_child');

              return (
                <g key={edge.id} className="group">
                  {/* Lower transparent hit path (wide clickable target, UX-063 visible keyboard focus) */}
                  <path
                    d={edge.pathD}
                    fill="none"
                    stroke="transparent"
                    strokeWidth={isMobile ? 20 : 14}
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    style={{ pointerEvents: 'stroke' }}
                    className="cursor-pointer focus:outline-hidden focus-visible:outline-hidden focus-visible:stroke-brand/50 transition-colors"
                    role="button"
                    tabIndex={0}
                    aria-label={`${edgeAriaLabel}: ${t('tree.connection.select_hint')}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      onSelectEdge(edge);
                    }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        e.stopPropagation();
                        onSelectEdge(edge);
                      }
                    }}
                  />

                  {/* Upper visual path with geometric precision (UX-062 decorative aria-hidden) */}
                  <path
                    aria-hidden="true"
                    d={edge.pathD}
                    fill="none"
                    shapeRendering="geometricPrecision"
                    strokeWidth={visualStrokeWidth}
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeDasharray={traceDasharray}
                    style={
                      isSelected
                        ? {
                            filter:
                              'drop-shadow(0 0 4px var(--color-brand-accent)) drop-shadow(0 0 2px var(--color-brand))',
                          }
                        : traceFilter
                        ? { filter: traceFilter }
                        : undefined
                    }
                    className={cn(
                      'pointer-events-none transition-all',
                      strokeClass,
                      isSelected && 'stroke-amber-600 dark:stroke-amber-400'
                    )}
                  />

                  {/* Midpoint union circle dot for spouse connections (UX-062 decorative aria-hidden) */}
                  {edge.kind === 'spouse' && (
                    <circle
                      aria-hidden="true"
                      cx={edge.midX}
                      cy={edge.midY}
                      r={isSelected ? 4 : isKinshipHighlighted ? 3.5 : 3}
                      className={cn(
                        'pointer-events-none transition-all fill-amber-600 dark:fill-amber-400 stroke-surface',
                        isKinshipHighlighted &&
                          traceIdx === 0 &&
                          'fill-brand stroke-surface',
                        isKinshipHighlighted &&
                          traceIdx === 1 &&
                          'fill-amber-600 dark:fill-amber-400 stroke-surface',
                        isKinshipHighlighted &&
                          traceIdx >= 2 &&
                          'fill-purple-500 stroke-surface',
                        isSelected && 'fill-amber-500 dark:fill-amber-400 stroke-brand'
                      )}
                      strokeWidth={1}
                      shapeRendering="geometricPrecision"
                    />
                  )}
                </g>
              );
            })}
          </svg>

          {/* Component Header Badges for Multi-Component Forest */}
          {layout.components &&
            layout.components.map((comp) => {
              const title = comp.titleKey
                ? comp.titleKey === 'tree.components.branch'
                  ? t('tree.components.branch', {
                      number: comp.id.replace('comp-', ''),
                    })
                  : t(comp.titleKey)
                : comp.title;
              const badgeScale = Math.max(1, Math.min(2.5, 0.6 / zoom));
              return (
                <div
                  key={comp.id}
                  data-testid={`tree-component-badge-${comp.id}`}
                  role="heading"
                  aria-level={3}
                  aria-label={`${title} (${comp.nodeCount})`}
                  style={{
                    position: 'absolute',
                    left: `${comp.x}px`,
                    top: `${comp.y}px`,
                    transform: `scale(${badgeScale})`,
                    transformOrigin: 'left center',
                  }}
                  className="pointer-events-none select-none flex items-center will-change-transform"
                >
                  <Badge
                    variant={
                      comp.isPrimary
                        ? 'brand'
                        : comp.isShelf
                        ? 'neutral'
                        : 'outline'
                    }
                    size="md"
                    className="shadow-xs font-semibold backdrop-blur-xs"
                  >
                    {title} ({comp.nodeCount})
                  </Badge>
                </div>
              );
            })}

          {/* Foreground HTML Layer for Person Node Cards with Frustum Culling and Dynamic LOD */}
          {layout.nodes.filter(isNodeInFrustum).map((pn) => {
            const isHome = Boolean(
              resolvedHomePersonId &&
                (pn.id === resolvedHomePersonId ||
                  pn.node.gid === resolvedHomePersonId)
            );
            const isHighlightedInPath = highlightedPathIds.includes(pn.id);
            const isEndpoint = Boolean(
              selectedEdge &&
                (selectedEdge.sourceId === pn.id ||
                  selectedEdge.targetId === pn.id)
            );
            const isPulsing = pulsingPersonId === pn.id;
            const hasParentEdge =
              mode !== 'pedigree' &&
              layout.edges.some(
                (e) =>
                  (e.kind === 'parent_child' || e.kind === 'placeholder') &&
                  e.targetId === pn.id
              );
            const hasChildEdge =
              mode !== 'pedigree' &&
              layout.edges.some(
                (e) =>
                  (e.kind === 'parent_child' || e.kind === 'placeholder') &&
                  e.sourceId === pn.id &&
                  e.sourceY > pn.y + 50
              );
            return (
              <PersonNodeCard
                key={pn.id}
                node={pn.node}
                x={pn.x}
                y={pn.y}
                lod={currentLod}
                isFocal={pn.isFocal || isHighlightedInPath}
                isSelected={
                  selectedPersonId === pn.id || isHighlightedInPath
                }
                isHomePerson={isHome}
                isEdgeEndpointHighlighted={isEndpoint}
                isPulsing={isPulsing}
                accessTier={treeView.access_tier}
                canEdit={canEdit}
                hasParentEdge={hasParentEdge}
                hasChildEdge={hasChildEdge}
                onSelect={(node) => onSelectPerson(node.id)}
                onFocusNode={(node) => {
                  onFocusNode(node.id);
                }}
                onQuickAdd={(node, rel) =>
                  onQuickAdd(node, rel)
                }
                onEdit={(node) => onEditPerson(node)}
              />
            );
          })}

          {/* Empty Parent Placeholder Slots for Editors/Admins */}
          {canEdit &&
            layout.placeholders.map((slot) => {
              const anchorNode = nodeMap.get(slot.anchorId);
              if (!anchorNode) return null;
              return (
                <div
                  key={slot.id}
                  style={{
                    position: 'absolute',
                    left: `${slot.x}px`,
                    top: `${slot.y}px`,
                    width: `${slot.width}px`,
                    height: `${slot.height}px`,
                  }}
                  onPointerDown={(e) => e.stopPropagation()}
                  onClick={(e) => e.stopPropagation()}
                  className="flex items-center justify-center rounded-xl border border-dashed border-border bg-surface/60 p-2 hover:border-brand/60 hover:bg-surface transition-colors"
                >
                  <Button
                    size="sm"
                    variant="ghost"
                    leftIcon={<UserPlus className="h-3.5 w-3.5 text-brand" />}
                    onClick={() =>
                      onQuickAdd(anchorNode, slot.relation)
                    }
                  >
                    {slot.relation === 'father'
                      ? t('person.actions.addFather')
                      : t('person.actions.addMother')}
                  </Button>
                </div>
              );
            })}
        </div>
      )}
    </>
  );
};
