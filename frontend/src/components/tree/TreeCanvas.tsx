// Exempt interactive cards ([role="treeitem"], [data-node-id]) from canvas pointer capture
import { PublicGuestBanner } from "./PublicGuestBanner";
import React, {
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { useToast } from "../ui";
import { EmptyTreeOnboardingCard } from "../onboarding";
import {
  fetchTreeView,
  QuickAddRelation,
  setHomePerson,
  TreePersonNodeDto,
  TreeProjectionMode,
  TreeViewResponse,
} from "../../api/client";
import { useAuth } from "../../lib/auth-context";
import { getLocalizedErrorMessage } from "../../lib/error-formatter";
import { useI18n } from "../../i18n";
import {
  buildTreeGraphIndex,
  computeTreeLayout,
  PositionedEdge,
  TreeLayoutOrientation,
} from "../../lib/tree-layouts";
import { TreeToolbar } from "./TreeToolbar";
import { TreeMiniMap } from "./TreeMiniMap";
import { PersonImmediateFamily } from "./PersonDrawer";
import { FanChartCanvas } from "./FanChartCanvas";
import { HighlightedTrace } from "./KinshipFinderDialog";
import { ConnectionNavPopover } from "./ConnectionNavPopover";
import { TreeCanvasWorldStage } from "./TreeCanvasWorldStage";
import { TreeCanvasOverlaysAndDialogs } from "./TreeCanvasOverlaysAndDialogs";

export interface TreeCanvasProps {
  initialTreeView?: TreeViewResponse;
  initialSelectedPersonId?: string | null;
  focusPersonId?: string | null;
  isMobileViewport?: boolean;
  onSignIn?: () => void;
  onNavigate?: (path: string) => void;
}

import {
  MAX_ZOOM,
  MIN_ZOOM,
  useTreeCanvasCamera,
} from "./useTreeCanvasCamera";


export const TreeCanvas: React.FC<TreeCanvasProps> = ({
  initialTreeView,
  initialSelectedPersonId = null,
  focusPersonId = null,
  isMobileViewport,
  onSignIn,
  onNavigate,
}) => {
  const { t } = useI18n();
  const { user, isAuthenticated, updateUser } = useAuth();
  const { addToast } = useToast();

  const defaultAccessTier =
    isAuthenticated && user?.role
      ? user.role
      : isAuthenticated
      ? 'viewer'
      : 'public_anonymous';

  const [treeView, setTreeView] = useState<TreeViewResponse>(
    () =>
      initialTreeView ?? {
        tree_title: '',
        focal_id: null,
        resolved_home_person_id: null,
        mode: 'hourglass',
        generations: 4,
        access_tier: defaultAccessTier,
        total_visible: 0,
        nodes: [],
        edges: [],
        families: [],
      }
  );
  const [mode, setMode] = useState<TreeProjectionMode>(() => {
    if (typeof window !== 'undefined') {
      const searchParams = new URLSearchParams(window.location.search);
      const urlMode = searchParams.get('mode') as TreeProjectionMode | null;
      if (
        urlMode &&
        ['hourglass', 'pedigree', 'descendant', 'fan', 'all'].includes(urlMode)
      ) {
        return urlMode;
      }
    }
    return ((initialTreeView?.mode as TreeProjectionMode) || 'hourglass');
  });

  const handleModeChange = useCallback((nextMode: TreeProjectionMode) => {
    setMode(nextMode);
    if (typeof window !== 'undefined') {
      try {
        const url = new URL(window.location.href);
        url.searchParams.set('mode', nextMode);
        window.history.replaceState(null, '', url.toString());
      } catch {
        // Fallback for jsdom / tests
      }
    }
  }, []);

  const [generations, setGenerations] = useState<number>(() => {
    if (typeof window !== 'undefined') {
      const searchParams = new URLSearchParams(window.location.search);
      const genParam = searchParams.get('generations');
      if (genParam) {
        const parsed = parseInt(genParam, 10);
        if (!isNaN(parsed) && parsed >= 1 && parsed <= 10) {
          return parsed;
        }
      }
    }
    return initialTreeView?.generations ?? 4;
  });

  const handleGenerationsChange = useCallback((nextGen: number) => {
    setGenerations(nextGen);
    if (typeof window !== 'undefined') {
      try {
        const url = new URL(window.location.href);
        url.searchParams.set('generations', String(nextGen));
        window.history.replaceState(null, '', url.toString());
      } catch {
        // Fallback for jsdom / tests
      }
    }
  }, []);

  const [orientation, setOrientation] = useState<TreeLayoutOrientation>(() => {
    if (typeof window !== 'undefined') {
      const searchParams = new URLSearchParams(window.location.search);
      const urlOri = searchParams.get('orientation');
      if (urlOri === 'vertical' || urlOri === 'horizontal') {
        return urlOri;
      }
    }
    return 'vertical';
  });

  const handleOrientationChange = useCallback((nextOri: TreeLayoutOrientation) => {
    setOrientation(nextOri);
    if (typeof window !== 'undefined') {
      try {
        const url = new URL(window.location.href);
        url.searchParams.set('orientation', nextOri);
        window.history.replaceState(null, '', url.toString());
      } catch {
        // Fallback for jsdom / tests
      }
    }
  }, []);
  const [focalId, setFocalId] = useState<string | null>(
    focusPersonId ?? initialTreeView?.focal_id ?? null
  );
  const [collateral, setCollateral] = useState<boolean>(() => {
    if (initialTreeView?.collateral !== undefined && initialTreeView.collateral !== null) {
      return initialTreeView.collateral;
    }
    if (typeof window !== 'undefined') {
      const searchParams = new URLSearchParams(window.location.search);
      const collParam = searchParams.get('collateral');
      if (collParam !== null) {
        return collParam === 'true' || collParam === '1';
      }
    }
    return true;
  });

  const handleCollateralChange = useCallback((nextCollateral: boolean) => {
    setCollateral(nextCollateral);
    if (typeof window !== 'undefined') {
      try {
        const url = new URL(window.location.href);
        url.searchParams.set('collateral', String(nextCollateral));
        window.history.replaceState(null, '', url.toString());
      } catch {
        // Fallback for jsdom / tests
      }
    }
  }, []);


  // Selected person for Drawer / BottomSheet
  const [selectedPersonId, setSelectedPersonId] = useState<string | null>(
    initialSelectedPersonId
  );

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
  const [kinshipDialogOpen, setKinshipDialogOpen] = useState<boolean>(false);
  const [highlightedPathIds, setHighlightedPathIds] = useState<string[]>([]);
  const [highlightedTraces, setHighlightedTraces] = useState<HighlightedTrace[]>([]);
  const [selectedEdge, setSelectedEdge] = useState<PositionedEdge | null>(null);
  const [pulsingPersonId, setPulsingPersonId] = useState<string | null>(null);
  const [kinshipInitialA, setKinshipInitialA] = useState<string | null>(null);
  const [kinshipInitialB, setKinshipInitialB] = useState<string | null>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setSelectedEdge(null);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const loadTreeView = useCallback(
    (
      nextFocalId?: string | null,
      nextMode?: TreeProjectionMode,
      nextGen?: number,
      nextCollateral?: boolean
    ) => {
      if (initialTreeView) return () => {};
      let cancelled = false;
      const effectiveMode = nextMode ?? mode;
      const effectiveCollateral = nextCollateral ?? collateral;
      fetchTreeView({
        focal_id: (nextFocalId ?? focalId) || undefined,
        mode: effectiveMode,
        generations: nextGen ?? generations,
        collateral: effectiveMode === 'hourglass' ? effectiveCollateral : undefined,
      })
        .then((res) => {
          if (cancelled) return;
          setTreeView(res);
          if (res.focal_id && !focalId) {
            setFocalId(res.focal_id);
          }
        })
        .catch(() => {});
      return () => {
        cancelled = true;
      };
    },
    [initialTreeView, focalId, mode, generations, collateral]
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
    return loadTreeView(focalId, mode, generations, collateral);
  }, [initialTreeView, loadTreeView, focalId, mode, generations, collateral]);

  const effectiveViewData = useMemo<TreeViewResponse>(
    () => ({
      ...treeView,
      focal_id: focalId ?? treeView.focal_id,
      mode,
      generations,
      collateral,
    }),
    [treeView, focalId, mode, generations, collateral]
  );

  const layout = useMemo(
    () =>
      computeTreeLayout(effectiveViewData, {
        mode,
        generations,
        collateral,
        orientation,
      }),
    [effectiveViewData, mode, generations, collateral, orientation]
  );

  const graphIndex = useMemo(
    () => buildTreeGraphIndex(effectiveViewData),
    [effectiveViewData]
  );

  const effectiveAccessTier =
    initialTreeView?.access_tier ??
    (isAuthenticated && user?.role
      ? user.role
      : isAuthenticated
      ? treeView.access_tier === 'public_anonymous'
        ? 'viewer'
        : treeView.access_tier
      : 'public_anonymous');

  const canEdit =
    effectiveAccessTier === 'editor' || effectiveAccessTier === 'admin';
  const resolvedHomePersonId =
    treeView.resolved_home_person_id ??
    user?.home_person_id ??
    user?.person_id ??
    null;

  const {
    containerRef,
    viewport,
    pan,
    zoom,
    setZoom,
    isAnimatingCamera,
    isPanning,
    isMobile,
    currentLod,
    isNodeInFrustum,
    centerOnWorldPoint,
    centerOnNode,
    frameHighlightedNodes,
    fitToScreen,
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
  } = useTreeCanvasCamera({
    layout,
    focalId,
    isMobileViewport,
  });

  const handleJumpToPerson = useCallback(
    (personId: string) => {
      setSelectedEdge(null);
      setPulsingPersonId(personId);
      centerOnNode(personId, true);
      requestAnimationFrame(() => {
        const cardEl = containerRef.current?.querySelector<HTMLElement>(
          `[data-node-id="${personId}"]`
        );
        cardEl?.focus();
      });
      setTimeout(() => {
        setPulsingPersonId((curr) => (curr === personId ? null : curr));
      }, 2000);
    },
    [centerOnNode]
  );

  const handleFindKinshipFromEdge = useCallback(
    (personAId: string, personBId: string) => {
      setSelectedEdge(null);
      setKinshipInitialA(personAId);
      setKinshipInitialB(personBId);
      setKinshipDialogOpen(true);
    },
    []
  );


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
        description: getLocalizedErrorMessage(err, t),
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
    <div className="flex flex-col h-[calc(100dvh-4rem)] w-full overflow-hidden">
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
        onClick={() => setSelectedEdge(null)}
        className="relative flex-1 w-full overflow-hidden bg-canvas select-none touch-none cursor-grab active:cursor-grabbing"
      >
        {/* Floating Mode, Depth & Zoom Toolbars */}
        <TreeToolbar
          mode={mode}
          onModeChange={handleModeChange}
          generations={generations}
          onGenerationsChange={handleGenerationsChange}
          collateral={collateral}
          onCollateralChange={handleCollateralChange}
          orientation={orientation}
          onOrientationChange={handleOrientationChange}
          totalVisible={layout.nodes.length}
          zoom={zoom}
          onZoomIn={() =>
            setZoom((z) => Math.min(MAX_ZOOM, Number((z + 0.2).toFixed(2))))
          }
          onZoomOut={() =>
            setZoom((z) => Math.max(MIN_ZOOM, Number((z - 0.2).toFixed(2))))
          }
          onZoomReset={() => setZoom(1)}
          onCenterFocal={() =>
            centerOnNode(focalId ?? layout.focalId ?? resolvedHomePersonId)
          }
          onFitToScreen={fitToScreen}
          onOpenKinshipFinder={() => setKinshipDialogOpen(true)}
        />


        {mode === "fan" && (
          <FanChartCanvas
            focalPersonId={focalId ?? layout.focalId}
            nodes={treeView.nodes}
            edges={treeView.edges}
            generations={generations}
            onGenerationsChange={setGenerations}
            accessTier={
              treeView.access_tier as
                | "public_anonymous"
                | "viewer"
                | "editor"
                | "admin"
            }
            selectedPersonId={selectedPersonId}
            onSelectPerson={(p) => setSelectedPersonId(p.id)}
            onFocusPerson={(p) => setFocalId(p.id)}
          />
        )}

        <TreeCanvasWorldStage
          mode={mode}
          layout={layout}
          effectiveViewData={effectiveViewData}
          treeView={treeView}
          nodeMap={graphIndex.nodeMap}
          pan={pan}
          zoom={zoom}
          isPanning={isPanning}
          isAnimatingCamera={isAnimatingCamera}
          isMobile={isMobile}
          currentLod={currentLod}
          canEdit={canEdit}
          resolvedHomePersonId={resolvedHomePersonId}
          selectedPersonId={selectedPersonId}
          selectedEdge={selectedEdge}
          pulsingPersonId={pulsingPersonId}
          highlightedPathIds={highlightedPathIds}
          highlightedTraces={highlightedTraces}
          isNodeInFrustum={isNodeInFrustum}
          onClearHighlights={() => {
            setHighlightedPathIds([]);
            setHighlightedTraces([]);
          }}
          onSelectEdge={setSelectedEdge}
          onSelectPerson={setSelectedPersonId}
          onFocusNode={(id) => {
            setFocalId(id);
            centerOnNode(id);
          }}
          onQuickAdd={(anchor, relation) =>
            setQuickAddState({ anchor, relation })
          }
          onEditPerson={(node) => setEditTarget(node)}
        />

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

        {/* Connection HUD Popover */}
        {selectedEdge && (
          <ConnectionNavPopover
            edge={selectedEdge}
            sourcePerson={
              graphIndex.nodeMap.get(selectedEdge.sourceId) ??
              treeView.nodes.find((n) => n.id === selectedEdge.sourceId) ??
              null
            }
            targetPerson={
              graphIndex.nodeMap.get(selectedEdge.targetId) ??
              treeView.nodes.find((n) => n.id === selectedEdge.targetId) ??
              null
            }
            family={
              effectiveViewData.families?.find(
                (f) =>
                  f.id === selectedEdge.familyId ||
                  (f.partner1_id === selectedEdge.sourceId &&
                    f.partner2_id === selectedEdge.targetId) ||
                  (f.partner1_id === selectedEdge.targetId &&
                    f.partner2_id === selectedEdge.sourceId)
              ) ?? null
            }
            accessTier={treeView.access_tier}
            isMobile={isMobile}
            pan={{
              x: isPanning ? pan.x : Math.round(pan.x),
              y: isPanning ? pan.y : Math.round(pan.y),
            }}
            zoom={zoom}
            containerRect={
              containerRef.current
                ? {
                    width: viewport.width,
                    height: viewport.height,
                    left: 0,
                    top: 0,
                  }
                : null
            }
            onJumpToPerson={handleJumpToPerson}
            onFindKinship={handleFindKinshipFromEdge}
            onClose={() => setSelectedEdge(null)}
            onViewPerson={(id) => onNavigate?.(`/persons/${id}`)}
          />
        )}
      </div>


      <TreeCanvasOverlaysAndDialogs
        isMobile={isMobile}
        treeView={treeView}
        canEdit={canEdit}
        resolvedHomePersonId={resolvedHomePersonId}
        focalId={focalId}
        selectedPersonId={selectedPersonId}
        selectedPerson={selectedPerson}
        selectedImmediateFamily={selectedImmediateFamily}
        quickAddState={quickAddState}
        editTarget={editTarget}
        eventTarget={eventTarget}
        deleteTarget={deleteTarget}
        kinshipDialogOpen={kinshipDialogOpen}
        kinshipInitialA={kinshipInitialA}
        kinshipInitialB={kinshipInitialB}
        onSelectPersonId={setSelectedPersonId}
        onMakeFocal={(p) => {
          setFocalId(p.id);
          centerOnNode(p.id);
        }}
        onSetHomePerson={(p) => void handleSetHomePerson(p)}
        onSelectRelative={(rel) => {
          setSelectedPersonId(rel.id);
          centerOnNode(rel.id);
        }}
        onQuickAdd={(p, relation) => setQuickAddState({ anchor: p, relation })}
        onCloseQuickAdd={() => setQuickAddState(null)}
        onEditTargetChange={setEditTarget}
        onEventTargetChange={setEventTarget}
        onDeleteTargetChange={setDeleteTarget}
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
        onReloadTree={() => loadTreeView()}
        onOpenProfile={(p) => {
          if (treeView.access_tier === "public_anonymous" && p.is_living) {
            onSignIn?.();
            return;
          }
          onNavigate?.(`/persons/${p.id}`);
        }}
        onMergePerson={(p) => {
          const url = `/people/merge?target=${encodeURIComponent(
            p.id
          )}&returnTo=${encodeURIComponent("/tree?focus=" + p.id)}`;
          if (onNavigate) {
            onNavigate(url);
          } else if (typeof window !== "undefined") {
            window.history.pushState({}, "", url);
            window.dispatchEvent(new PopStateEvent("popstate"));
          }
        }}
        onKinshipOpenChange={(open) => {
          setKinshipDialogOpen(open);
          if (!open) {
            setKinshipInitialA(null);
            setKinshipInitialB(null);
          }
        }}
        onHighlightPath={(ids, traces) => {
          setHighlightedPathIds(ids);
          setHighlightedTraces(traces ?? []);
          if (mode === "fan") {
            setMode("hourglass");
          }
          requestAnimationFrame(() => {
            frameHighlightedNodes(ids);
          });
        }}
      />
    </div>
  );
};

