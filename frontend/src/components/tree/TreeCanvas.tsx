import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { UserPlus } from 'lucide-react';
import { Button, useToast } from '../ui';
import {
  DeleteImpactDialog,
  LifeEventDialog,
  PersonEditDialog,
  QuickAddRelativeDialog,
} from '../person';
import { EmptyTreeOnboardingCard } from '../onboarding';
import {
  fetchTreeView,
  QuickAddRelation,
  setHomePerson,
  TreePersonNodeDto,
  TreeProjectionMode,
  TreeViewResponse,
} from '../../api/client';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';
import {
  buildDefaultTreeViewData,
  buildTreeGraphIndex,
  CARD_HEIGHT,
  CARD_WIDTH,
  computeTreeLayout,
} from '../../lib/tree-layouts';
import { PublicGuestBanner } from './PublicGuestBanner';
import { PersonNodeCard } from './PersonNodeCard';
import { TreeToolbar } from './TreeToolbar';
import { TreeMiniMap } from './TreeMiniMap';
import { PersonDrawer, PersonImmediateFamily } from './PersonDrawer';
import { PersonBottomSheet } from './PersonBottomSheet';

export interface TreeCanvasProps {
  initialTreeView?: TreeViewResponse;
  initialSelectedPersonId?: string | null;
  focusPersonId?: string | null;
  isMobileViewport?: boolean;
  onSignIn?: () => void;
  onNavigate?: (path: string) => void;
}

const MIN_ZOOM = 0.25;
const MAX_ZOOM = 3.0;

export const TreeCanvas: React.FC<TreeCanvasProps> = ({
  initialTreeView,
  initialSelectedPersonId = null,
  focusPersonId = null,
  isMobileViewport,
  onSignIn,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user, updateUser } = useAuth();
  const { addToast } = useToast();

  const [treeView, setTreeView] = useState<TreeViewResponse>(
    initialTreeView ?? buildDefaultTreeViewData('admin', 'hourglass')
  );
  const [mode, setMode] = useState<TreeProjectionMode>(
    ((initialTreeView?.mode as TreeProjectionMode) || 'hourglass')
  );
  const [generations, setGenerations] = useState<number>(
    initialTreeView?.generations ?? 4
  );
  const [focalId, setFocalId] = useState<string | null>(
    focusPersonId ?? initialTreeView?.focal_id ?? null
  );

  // Camera state
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [viewport, setViewport] = useState<{ width: number; height: number }>({
    width: 1200,
    height: 720,
  });
  const [pan, setPan] = useState<{ x: number; y: number }>({
    x: 480,
    y: 280,
  });
  const [zoom, setZoom] = useState<number>(1);
  const [isAnimatingCamera, setIsAnimatingCamera] = useState<boolean>(false);

  // Pointer & multi-touch state
  const isDraggingRef = useRef<boolean>(false);
  const dragStartRef = useRef<{ x: number; y: number; panX: number; panY: number }>({
    x: 0,
    y: 0,
    panX: 0,
    panY: 0,
  });
  const activePointersRef = useRef<Map<number, { x: number; y: number }>>(
    new Map()
  );
  const pinchInitialDistRef = useRef<number | null>(null);
  const pinchInitialZoomRef = useRef<number>(1);

  // Selected person for Drawer / BottomSheet
  const [selectedPersonId, setSelectedPersonId] = useState<string | null>(
    initialSelectedPersonId
  );
  const [isMobile, setIsMobile] = useState<boolean>(() => {
    if (isMobileViewport !== undefined) return isMobileViewport;
    if (typeof window !== 'undefined') return window.innerWidth < 768;
    return false;
  });

  // Authoring modal states
  const [quickAddState, setQuickAddState] = useState<{
    anchor: TreePersonNodeDto;
    relation: QuickAddRelation;
  } | null>(null);
  const [editTarget, setEditTarget] = useState<TreePersonNodeDto | null>(null);
  const [eventTarget, setEventTarget] = useState<TreePersonNodeDto | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<TreePersonNodeDto | null>(
    null
  );

  useEffect(() => {
    if (isMobileViewport !== undefined) {
      setIsMobile(isMobileViewport);
      return;
    }
    if (typeof window === 'undefined') return;
    const updateViewport = () => {
      setIsMobile(window.innerWidth < 768);
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        if (rect.width > 0 && rect.height > 0) {
          setViewport({ width: rect.width, height: rect.height });
        }
      }
    };
    updateViewport();
    window.addEventListener('resize', updateViewport);
    return () => window.removeEventListener('resize', updateViewport);
  }, [isMobileViewport]);

  const loadTreeView = useCallback(
    (nextFocalId?: string | null, nextMode?: TreeProjectionMode, nextGen?: number) => {
      if (initialTreeView) return;
      fetchTreeView({
        focal_id: (nextFocalId ?? focalId) || undefined,
        mode: nextMode ?? mode,
        generations: nextGen ?? generations,
      })
        .then((res) => {
          setTreeView(res);
          if (res.focal_id && !focalId) {
            setFocalId(res.focal_id);
          }
        })
        .catch(() => {});
    },
    [initialTreeView, focalId, mode, generations]
  );

  useEffect(() => {
    if (focusPersonId) {
      setFocalId(focusPersonId);
    }
  }, [focusPersonId]);

  useEffect(() => {
    if (initialTreeView) {
      setTreeView(initialTreeView);
      return;
    }
    loadTreeView(focalId, mode, generations);
  }, [initialTreeView, loadTreeView, focalId, mode, generations]);

  const effectiveViewData = useMemo<TreeViewResponse>(
    () => ({
      ...treeView,
      focal_id: focalId ?? treeView.focal_id,
      mode,
      generations,
    }),
    [treeView, focalId, mode, generations]
  );

  const layout = useMemo(
    () => computeTreeLayout(effectiveViewData, { mode, generations }),
    [effectiveViewData, mode, generations]
  );

  const graphIndex = useMemo(
    () => buildTreeGraphIndex(effectiveViewData),
    [effectiveViewData]
  );

  const canEdit =
    treeView.access_tier === 'editor' || treeView.access_tier === 'admin';
  const resolvedHomePersonId =
    treeView.resolved_home_person_id ??
    user?.home_person_id ??
    user?.person_id ??
    null;

  const centerOnWorldPoint = useCallback(
    (worldX: number, worldY: number, nextZoom?: number) => {
      const targetZoom = nextZoom ?? zoom;
      setIsAnimatingCamera(true);
      setZoom(targetZoom);
      setPan({
        x: viewport.width / 2 - worldX * targetZoom,
        y: viewport.height / 2 - worldY * targetZoom,
      });
      setTimeout(() => setIsAnimatingCamera(false), 260);
    },
    [viewport.width, viewport.height, zoom]
  );

  const centerOnNode = useCallback(
    (nodeId: string | null) => {
      const target =
        layout.nodes.find((pn) => pn.id === nodeId) ??
        layout.nodes.find((pn) => pn.isFocal) ??
        layout.nodes[0];
      if (!target) return;
      centerOnWorldPoint(
        target.x + CARD_WIDTH / 2,
        target.y + CARD_HEIGHT / 2
      );
    },
    [layout.nodes, centerOnWorldPoint]
  );

  const fitToScreen = useCallback(() => {
    const b = layout.bounds;
    const pad = 120;
    const scaleX = viewport.width / Math.max(300, b.width + pad * 2);
    const scaleY = viewport.height / Math.max(240, b.height + pad * 2);
    const nextZoom = Math.min(1.15, Math.max(MIN_ZOOM, Math.min(scaleX, scaleY)));
    const centerX = b.minX + b.width / 2;
    const centerY = b.minY + b.height / 2;
    centerOnWorldPoint(centerX, centerY, nextZoom);
  }, [layout.bounds, viewport.width, viewport.height, centerOnWorldPoint]);

  // Pointer & Multi-Touch Pan / Zoom Handlers
  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0 && e.pointerType === 'mouse') return;
    const targetEl = e.target as HTMLElement | null;
    if (
      targetEl &&
      typeof targetEl.closest === 'function' &&
      targetEl.closest(
        'button, [role="button"], [role="radio"], select, input'
      )
    ) {
      return;
    }
    activePointersRef.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
    try {
      e.currentTarget.setPointerCapture(e.pointerId);
    } catch {
      // Ignore in synthetic environments
    }

    if (activePointersRef.current.size === 2) {
      const pts = Array.from(activePointersRef.current.values());
      const dist = Math.hypot(pts[1].x - pts[0].x, pts[1].y - pts[0].y);
      pinchInitialDistRef.current = Math.max(10, dist);
      pinchInitialZoomRef.current = zoom;
      isDraggingRef.current = false;
    } else if (activePointersRef.current.size === 1) {
      isDraggingRef.current = true;
      dragStartRef.current = {
        x: e.clientX,
        y: e.clientY,
        panX: pan.x,
        panY: pan.y,
      };
    }
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (activePointersRef.current.has(e.pointerId)) {
      activePointersRef.current.set(e.pointerId, {
        x: e.clientX,
        y: e.clientY,
      });
    }

    if (
      activePointersRef.current.size === 2 &&
      pinchInitialDistRef.current !== null
    ) {
      const pts = Array.from(activePointersRef.current.values());
      const dist = Math.hypot(pts[1].x - pts[0].x, pts[1].y - pts[0].y);
      const ratio = dist / pinchInitialDistRef.current;
      const nextZoom = Math.min(
        MAX_ZOOM,
        Math.max(MIN_ZOOM, pinchInitialZoomRef.current * ratio)
      );
      setZoom(nextZoom);
      return;
    }

    if (!isDraggingRef.current) return;
    const dx = e.clientX - dragStartRef.current.x;
    const dy = e.clientY - dragStartRef.current.y;
    setPan({
      x: dragStartRef.current.panX + dx,
      y: dragStartRef.current.panY + dy,
    });
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
    activePointersRef.current.delete(e.pointerId);
    if (activePointersRef.current.size < 2) {
      pinchInitialDistRef.current = null;
    }
    if (activePointersRef.current.size === 0) {
      isDraggingRef.current = false;
    }
  };

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const onWheelNative = (e: WheelEvent) => {
      e.preventDefault();
      const factor = e.deltaY < 0 ? 1.1 : 0.9;
      const nextZoom = Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom * factor));
      if (nextZoom === zoom) return;

      const rect = el.getBoundingClientRect();
      const cursorX = e.clientX - rect.left;
      const cursorY = e.clientY - rect.top;

      // Anchor zoom around cursor position
      const worldX = (cursorX - pan.x) / zoom;
      const worldY = (cursorY - pan.y) / zoom;

      setZoom(nextZoom);
      setPan({
        x: cursorX - worldX * nextZoom,
        y: cursorY - worldY * nextZoom,
      });
    };

    el.addEventListener('wheel', onWheelNative, { passive: false });
    return () => el.removeEventListener('wheel', onWheelNative);
  }, [zoom, pan.x, pan.y]);

  const handleSetHomePerson = async (person: TreePersonNodeDto) => {
    try {
      const updatedUser = await setHomePerson(person.id);
      updateUser(updatedUser);
      setTreeView((prev) => ({
        ...prev,
        resolved_home_person_id: person.id,
      }));
      addToast({
        type: 'success',
        title: t('tree.homePersonUpdatedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    }
  };

  const selectedPerson = useMemo(
    () =>
      selectedPersonId
        ? graphIndex.nodeMap.get(selectedPersonId) ?? null
        : null,
    [selectedPersonId, graphIndex.nodeMap]
  );

  const selectedImmediateFamily = useMemo<PersonImmediateFamily | undefined>(() => {
    if (!selectedPersonId) return undefined;
    const rel = graphIndex.relativesMap.get(selectedPersonId);
    if (!rel) return { parents: [], spouses: [], children: [] };
    const mapIds = (ids: string[]) =>
      ids
        .map((id) => graphIndex.nodeMap.get(id))
        .filter((item): item is TreePersonNodeDto => Boolean(item));
    return {
      parents: mapIds(rel.parentIds),
      spouses: mapIds(rel.spouseIds),
      children: mapIds(rel.childIds),
    };
  }, [selectedPersonId, graphIndex]);

  // Empty-Tree State -> EmptyTreeOnboardingCard
  if (treeView.nodes.length === 0 || treeView.total_visible === 0) {
    return (
      <div className="mx-auto max-w-5xl px-4 sm:px-6 lg:px-8 py-10 space-y-4">
        {treeView.access_tier === 'public_anonymous' && (
          <PublicGuestBanner onSignIn={onSignIn} />
        )}
        <EmptyTreeOnboardingCard
          canOnboard={canEdit}
          userRole={
            treeView.access_tier === 'public_anonymous'
              ? 'viewer'
              : treeView.access_tier
          }
          onPersonCreated={() => loadTreeView()}
          onImportGedcom={() => {
            if (onNavigate) {
              onNavigate('/settings/gedcom');
            } else if (typeof window !== 'undefined') {
              window.history.pushState({}, '', '/settings/gedcom');
              window.dispatchEvent(new PopStateEvent('popstate'));
            }
          }}
        />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-[calc(100vh-4rem)] w-full overflow-hidden">
      {treeView.access_tier === 'public_anonymous' && (
        <div className="px-4 sm:px-6 py-2 shrink-0">
          <PublicGuestBanner onSignIn={onSignIn} />
        </div>
      )}

      {/* Full-Viewport Interactive Canvas Container */}
      <div
        ref={containerRef}
        role="tree"
        aria-label={t('tree.canvas.ariaLabel')}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        className="relative flex-1 w-full overflow-hidden bg-canvas select-none touch-none cursor-grab active:cursor-grabbing"
      >
        {/* Floating Mode, Depth & Zoom Toolbars */}
        <TreeToolbar
          mode={mode}
          onModeChange={(nextMode) => setMode(nextMode)}
          generations={generations}
          onGenerationsChange={(nextGen) => setGenerations(nextGen)}
          zoom={zoom}
          onZoomIn={() =>
            setZoom((z) => Math.min(MAX_ZOOM, Number((z + 0.2).toFixed(2))))
          }
          onZoomOut={() =>
            setZoom((z) => Math.max(MIN_ZOOM, Number((z - 0.2).toFixed(2))))
          }
          onZoomReset={() => setZoom(1)}
          onCenterFocal={() =>
            centerOnNode(resolvedHomePersonId ?? layout.focalId)
          }
          onFitToScreen={fitToScreen}
        />

        {/* Hardware-Accelerated Transformed World Stage */}
        <div
          style={{
            transform: `translate3d(${pan.x}px, ${pan.y}px, 0) scale(${zoom})`,
            transformOrigin: '0 0',
          }}
          className={
            isAnimatingCamera
              ? 'absolute left-0 top-0 transition-transform duration-250 ease-out will-change-transform'
              : 'absolute left-0 top-0 will-change-transform'
          }
        >
          {/* Background SVG Layer for Connector Curves & Union Bars */}
          <svg
            aria-hidden="true"
            className="overflow-visible pointer-events-none absolute left-0 top-0"
            width={1}
            height={1}
          >
            {layout.edges.map((edge) => (
              <path
                key={edge.id}
                d={edge.pathD}
                fill="none"
                strokeWidth={edge.kind === 'spouse' ? 2.5 : 2}
                strokeDasharray={
                  edge.kind === 'placeholder' ? '5 5' : undefined
                }
                className={
                  edge.kind === 'spouse'
                    ? 'stroke-accent'
                    : edge.kind === 'placeholder'
                    ? 'stroke-border'
                    : edge.isDirectLineage
                    ? 'stroke-brand/70'
                    : 'stroke-border'
                }
              />
            ))}
          </svg>

          {/* Foreground HTML Layer for Person Node Cards */}
          {layout.nodes.map((pn) => {
            const isHome = Boolean(
              resolvedHomePersonId &&
                (pn.id === resolvedHomePersonId ||
                  pn.node.gid === resolvedHomePersonId)
            );
            return (
              <PersonNodeCard
                key={pn.id}
                node={pn.node}
                x={pn.x}
                y={pn.y}
                isFocal={pn.isFocal}
                isSelected={selectedPersonId === pn.id}
                isHomePerson={isHome}
                accessTier={treeView.access_tier}
                canEdit={canEdit}
                onSelect={(node) => setSelectedPersonId(node.id)}
                onFocusNode={(node) => {
                  setFocalId(node.id);
                  centerOnNode(node.id);
                }}
                onQuickAdd={(node, rel) =>
                  setQuickAddState({ anchor: node, relation: rel })
                }
                onEdit={(node) => setEditTarget(node)}
              />
            );
          })}

          {/* Empty Parent Placeholder Slots for Editors/Admins */}
          {canEdit &&
            layout.placeholders.map((slot) => {
              const anchorNode = graphIndex.nodeMap.get(slot.anchorId);
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
                      setQuickAddState({
                        anchor: anchorNode,
                        relation: slot.relation,
                      })
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

        {/* Collapsible Bottom-Right Mini-Map Radar */}
        <TreeMiniMap
          nodes={layout.nodes}
          bounds={layout.bounds}
          pan={pan}
          zoom={zoom}
          viewportWidth={viewport.width}
          viewportHeight={viewport.height}
          defaultCollapsed={isMobile}
          onPanToWorldPoint={(wx, wy) => centerOnWorldPoint(wx, wy)}
        />
      </div>

      {/* Desktop Right Slide-Over Drawer vs Mobile Bottom Sheet */}
      {!isMobile ? (
        <PersonDrawer
          open={Boolean(selectedPerson)}
          onOpenChange={(open) => {
            if (!open) setSelectedPersonId(null);
          }}
          person={selectedPerson}
          family={selectedImmediateFamily}
          accessTier={treeView.access_tier}
          canEdit={canEdit}
          isHomePerson={Boolean(
            selectedPerson &&
              resolvedHomePersonId &&
              (selectedPerson.id === resolvedHomePersonId ||
                selectedPerson.gid === resolvedHomePersonId)
          )}
          onMakeFocal={(p) => {
            setFocalId(p.id);
            centerOnNode(p.id);
          }}
          onSetHomePerson={(p) => void handleSetHomePerson(p)}
          onSelectRelative={(rel) => {
            setSelectedPersonId(rel.id);
            centerOnNode(rel.id);
          }}
          onQuickAdd={(p, relation) =>
            setQuickAddState({ anchor: p, relation })
          }
          onEditPerson={(p) => setEditTarget(p)}
          onAddEvent={(p) => setEventTarget(p)}
          onDeletePerson={(p) => setDeleteTarget(p)}
          onOpenProfile={(p) => {
            if (
              treeView.access_tier === 'public_anonymous' &&
              p.is_living
            ) {
              onSignIn?.();
              return;
            }
            onNavigate?.(`/persons/${p.id}`);
          }}
        />
      ) : (
        <PersonBottomSheet
          open={Boolean(selectedPerson)}
          onOpenChange={(open) => {
            if (!open) setSelectedPersonId(null);
          }}
          person={selectedPerson}
          family={selectedImmediateFamily}
          accessTier={treeView.access_tier}
          canEdit={canEdit}
          isHomePerson={Boolean(
            selectedPerson &&
              resolvedHomePersonId &&
              (selectedPerson.id === resolvedHomePersonId ||
                selectedPerson.gid === resolvedHomePersonId)
          )}
          onMakeFocal={(p) => {
            setFocalId(p.id);
            centerOnNode(p.id);
          }}
          onSetHomePerson={(p) => void handleSetHomePerson(p)}
          onSelectRelative={(rel) => {
            setSelectedPersonId(rel.id);
            centerOnNode(rel.id);
          }}
          onQuickAdd={(p, relation) =>
            setQuickAddState({ anchor: p, relation })
          }
          onEditPerson={(p) => setEditTarget(p)}
          onOpenProfile={(p) => {
            if (
              treeView.access_tier === 'public_anonymous' &&
              p.is_living
            ) {
              onSignIn?.();
              return;
            }
            onNavigate?.(`/persons/${p.id}`);
          }}
        />
      )}

      {/* Authoring Dialogs */}
      <QuickAddRelativeDialog
        open={Boolean(quickAddState)}
        onOpenChange={(open) => {
          if (!open) setQuickAddState(null);
        }}
        anchorPerson={quickAddState?.anchor ?? null}
        relation={quickAddState?.relation ?? 'child'}
        existingPeople={treeView.nodes}
        onRelativeAdded={() => loadTreeView()}
      />

      <PersonEditDialog
        open={Boolean(editTarget)}
        onOpenChange={(open) => {
          if (!open) setEditTarget(null);
        }}
        person={editTarget}
        onPersonUpdated={() => loadTreeView()}
      />

      <LifeEventDialog
        open={Boolean(eventTarget)}
        onOpenChange={(open) => {
          if (!open) setEventTarget(null);
        }}
        individualId={eventTarget?.id ?? null}
        onEventSaved={() => loadTreeView()}
      />

      <DeleteImpactDialog
        open={Boolean(deleteTarget)}
        onOpenChange={(open) => {
          if (!open) setDeleteTarget(null);
        }}
        person={deleteTarget}
        onPersonDeleted={(deletedId) => {
          if (selectedPersonId === deletedId) {
            setSelectedPersonId(null);
          }
          setTreeView((prev) => ({
            ...prev,
            nodes: prev.nodes.filter((n) => n.id !== deletedId),
            total_visible: Math.max(0, prev.total_visible - 1),
          }));
          loadTreeView();
        }}
      />
    </div>
  );
};
