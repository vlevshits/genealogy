import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { Route, UserPlus, X } from 'lucide-react';
import { Badge, Button, useToast } from '../ui';
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
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';
import {
  buildTreeGraphIndex,
  CARD_HEIGHT,
  CARD_WIDTH,
  computeTreeLayout,
  PositionedEdge,
} from '../../lib/tree-layouts';
import { cn } from '../../lib/utils';
import { PublicGuestBanner } from './PublicGuestBanner';
import { PersonNodeCard } from './PersonNodeCard';
import { TreeToolbar } from './TreeToolbar';
import { TreeMiniMap } from './TreeMiniMap';
import { PersonDrawer, PersonImmediateFamily } from './PersonDrawer';
import { PersonBottomSheet } from './PersonBottomSheet';
import { FanChartCanvas } from './FanChartCanvas';
import { KinshipFinderDialog, HighlightedTrace } from './KinshipFinderDialog';
import { ConnectionNavPopover } from './ConnectionNavPopover';

export interface TreeCanvasProps {
  initialTreeView?: TreeViewResponse;
  initialSelectedPersonId?: string | null;
  focusPersonId?: string | null;
  isMobileViewport?: boolean;
  onSignIn?: () => void;
  onNavigate?: (path: string) => void;
}

const MIN_ZOOM = 0.1;
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

  const [generations, setGenerations] = useState<number>(
    initialTreeView?.generations ?? 4
  );
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

  // Camera state
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [viewport, setViewport] = useState<{ width: number; height: number }>({
    width: 1200,
    height: 720,
  });
  const [pan, setPan] = useState<{ x: number; y: number }>({
    x: 0,
    y: 0,
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
  const pinchInitialCenterRef = useRef<{ x: number; y: number; panX: number; panY: number } | null>(null);

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
  const [kinshipDialogOpen, setKinshipDialogOpen] = useState<boolean>(false);
  const [highlightedPathIds, setHighlightedPathIds] = useState<string[]>([]);
  const [highlightedTraces, setHighlightedTraces] = useState<HighlightedTrace[]>([]);
  const [selectedEdge, setSelectedEdge] = useState<PositionedEdge | null>(null);
  const [pulsingPersonId, setPulsingPersonId] = useState<string | null>(null);
  const [kinshipInitialA, setKinshipInitialA] = useState<string | null>(null);
  const [kinshipInitialB, setKinshipInitialB] = useState<string | null>(null);
  const [isPanning, setIsPanning] = useState<boolean>(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setSelectedEdge(null);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

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
    (
      nextFocalId?: string | null,
      nextMode?: TreeProjectionMode,
      nextGen?: number,
      nextCollateral?: boolean
    ) => {
      if (initialTreeView) return;
      const effectiveMode = nextMode ?? mode;
      const effectiveCollateral = nextCollateral ?? collateral;
      fetchTreeView({
        focal_id: (nextFocalId ?? focalId) || undefined,
        mode: effectiveMode,
        generations: nextGen ?? generations,
        collateral: effectiveMode === 'hourglass' ? effectiveCollateral : undefined,
      })
        .then((res) => {
          setTreeView(res);
          if (res.focal_id && !focalId) {
            setFocalId(res.focal_id);
          }
        })
        .catch(() => {});
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
    loadTreeView(focalId, mode, generations, collateral);
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
    () => computeTreeLayout(effectiveViewData, { mode, generations, collateral }),
    [effectiveViewData, mode, generations, collateral]
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

  // Dynamic Level of Detail (LOD) based on zoom level:
  // LOD 0 (zoom >= 0.5): full detail
  // LOD 1 (0.25 <= zoom < 0.5): simplified card
  // LOD 2 (zoom < 0.25): micro-pill
  const currentLod: 0 | 1 | 2 = zoom >= 0.5 ? 0 : zoom >= 0.25 ? 1 : 2;

  // Viewport frustum culling: skip rendering cards far outside the viewport in DOM
  const isNodeInFrustum = useCallback(
    (node: { x: number; y: number; width: number; height: number }) => {
      if (typeof window === 'undefined') return true;
      if (viewport.width <= 0 || viewport.height <= 0) return true;
      const margin = 400 / zoom;
      const worldLeft = -pan.x / zoom - margin;
      const worldRight = (viewport.width - pan.x) / zoom + margin;
      const worldTop = -pan.y / zoom - margin;
      const worldBottom = (viewport.height - pan.y) / zoom + margin;

      return (
        node.x + node.width >= worldLeft &&
        node.x <= worldRight &&
        node.y + node.height >= worldTop &&
        node.y <= worldBottom
      );
    },
    [pan.x, pan.y, zoom, viewport.width, viewport.height]
  );

  const centerOnWorldPoint = useCallback(
    (worldX: number, worldY: number, nextZoom?: number, animate = true) => {
      const targetZoom = nextZoom ?? zoom;
      if (animate) {
        setIsAnimatingCamera(true);
      }
      setZoom(targetZoom);
      setPan({
        x: viewport.width / 2 - worldX * targetZoom,
        y: viewport.height / 2 - worldY * targetZoom,
      });
      if (animate) {
        setTimeout(() => setIsAnimatingCamera(false), 260);
      }
    },
    [viewport.width, viewport.height, zoom]
  );

  const centerOnNode = useCallback(
    (nodeId: string | null, animate = true) => {
      const target =
        layout.nodes.find((pn) => pn.id === nodeId) ??
        layout.nodes.find((pn) => pn.isFocal) ??
        layout.nodes[0];
      if (!target) return;
      centerOnWorldPoint(
        target.x + CARD_WIDTH / 2,
        target.y + CARD_HEIGHT / 2,
        undefined,
        animate
      );
    },
    [layout.nodes, centerOnWorldPoint]
  );

  const frameHighlightedNodes = useCallback(
    (ids: string[]) => {
      if (ids.length === 0) return;
      const targetNodes = layout.nodes.filter((n) => ids.includes(n.id));
      if (targetNodes.length === 0) {
        if (ids[0]) centerOnNode(ids[0]);
        return;
      }

      let minX = Infinity;
      let minY = Infinity;
      let maxX = -Infinity;
      let maxY = -Infinity;

      for (const n of targetNodes) {
        minX = Math.min(minX, n.x);
        minY = Math.min(minY, n.y);
        maxX = Math.max(maxX, n.x + n.width);
        maxY = Math.max(maxY, n.y + n.height);
      }

      const pathWidth = maxX - minX;
      const pathHeight = maxY - minY;
      const centerX = minX + pathWidth / 2;
      const centerY = minY + pathHeight / 2;

      // Add comfortable padding around the framed path
      const padX = isMobile ? 60 : 120;
      const padY = isMobile ? 80 : 140;
      const scaleX = viewport.width / Math.max(300, pathWidth + padX * 2);
      const scaleY = viewport.height / Math.max(240, pathHeight + padY * 2);
      const targetZoom = Math.min(1.0, Math.max(MIN_ZOOM, Math.min(scaleX, scaleY)));

      centerOnWorldPoint(centerX, centerY, targetZoom, true);
    },
    [layout.nodes, isMobile, viewport.width, viewport.height, centerOnNode, centerOnWorldPoint]
  );

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

  const fitToScreen = useCallback(
    (animate = true) => {
      const b = layout.bounds;
      const pad = 120;
      const scaleX = viewport.width / Math.max(300, b.width + pad * 2);
      const scaleY = viewport.height / Math.max(240, b.height + pad * 2);
      const nextZoom = Math.min(1.15, Math.max(MIN_ZOOM, Math.min(scaleX, scaleY)));
      const centerX = b.minX + b.width / 2;
      const centerY = b.minY + b.height / 2;
      centerOnWorldPoint(centerX, centerY, nextZoom, animate);
    },
    [layout.bounds, viewport.width, viewport.height, centerOnWorldPoint]
  );

  // UX-040: Dynamic initial centering on focal node or fitToScreen
  const hasInitialCenteredRef = useRef<boolean>(false);

  useEffect(() => {
    if (hasInitialCenteredRef.current) return;
    if (layout.nodes.length === 0) return;
    if (viewport.width <= 0 || viewport.height <= 0) return;

    hasInitialCenteredRef.current = true;
    const focalNode =
      layout.nodes.find((pn) => pn.id === focalId) ??
      layout.nodes.find((pn) => pn.isFocal) ??
      layout.nodes[0];

    if (isMobile) {
      fitToScreen(false);
    } else if (focalNode) {
      centerOnNode(focalNode.id, false);
    } else {
      fitToScreen(false);
    }
  }, [
    layout.nodes,
    viewport.width,
    viewport.height,
    focalId,
    isMobile,
    centerOnNode,
    fitToScreen,
  ]);

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

  // Pointer & Multi-Touch Pan / Zoom Handlers
  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0 && e.pointerType === 'mouse') return;
    const targetEl = e.target as HTMLElement | null;
    if (
      targetEl &&
      typeof targetEl.closest === 'function' &&
      targetEl.closest(
        'button, [role="button"], [role="radio"], [role="treeitem"], [data-node-id], select, input'
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
      pinchInitialCenterRef.current = {
        x: (pts[0].x + pts[1].x) / 2,
        y: (pts[0].y + pts[1].y) / 2,
        panX: pan.x,
        panY: pan.y,
      };
      isDraggingRef.current = false;
    } else if (activePointersRef.current.size === 1) {
      isDraggingRef.current = true;
      setIsPanning(true);
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
      const midX = (pts[0].x + pts[1].x) / 2;
      const midY = (pts[0].y + pts[1].y) / 2;
      if (pinchInitialCenterRef.current && containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        const initLocalX = pinchInitialCenterRef.current.x - rect.left;
        const initLocalY = pinchInitialCenterRef.current.y - rect.top;
        const curLocalX = midX - rect.left;
        const curLocalY = midY - rect.top;
        const worldX =
          (initLocalX - pinchInitialCenterRef.current.panX) /
          pinchInitialZoomRef.current;
        const worldY =
          (initLocalY - pinchInitialCenterRef.current.panY) /
          pinchInitialZoomRef.current;
        setPan({
          x: curLocalX - worldX * nextZoom,
          y: curLocalY - worldY * nextZoom,
        });
      }
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
      pinchInitialCenterRef.current = null;
    }
    if (activePointersRef.current.size === 0) {
      isDraggingRef.current = false;
      setIsPanning(false);
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
          onGenerationsChange={(nextGen) => setGenerations(nextGen)}
          collateral={collateral}
          onCollateralChange={handleCollateralChange}
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
            centerOnNode(resolvedHomePersonId ?? layout.focalId)
          }
          onFitToScreen={fitToScreen}
          onOpenKinshipFinder={() => setKinshipDialogOpen(true)}
        />

        {/* Floating Multi-Trace Kinship Legend HUD */}
        {(highlightedTraces.length > 0 || highlightedPathIds.length > 0) && (
          <div
            role="status"
            aria-live="polite"
            className="absolute top-28 sm:top-24 md:top-4 left-1/2 -translate-x-1/2 z-30 flex flex-wrap items-center justify-center gap-3 px-4 py-2 rounded-2xl bg-surface/90 backdrop-blur-md border border-border shadow-elevation-2 text-xs text-foreground pointer-events-auto w-max max-w-[calc(100vw-24px)] mx-auto"
          >
            <div className="flex items-center gap-1.5 font-medium text-secondary">
              <Route className="h-3.5 w-3.5 text-brand" />
              <span>{t('tree.kinship.legendTitle')}</span>
            </div>
            <div className="flex items-center gap-2.5">
              {highlightedTraces.length > 0 ? (
                highlightedTraces.map((trace, idx) => {
                  const dotClass =
                    idx === 0
                      ? 'bg-brand'
                      : idx === 1
                      ? 'bg-amber-600 dark:bg-amber-400'
                      : 'bg-purple-500';
                  return (
                    <div
                      key={trace.id || idx}
                      className="flex items-center gap-1.5 font-medium"
                    >
                      <span
                        className={cn(
                          'w-2.5 h-2.5 rounded-full inline-block shrink-0',
                          dotClass
                        )}
                      />
                      <span className="text-secondary max-w-[140px] truncate">
                        {trace.label ||
                          t('tree.kinship.traceTab', {
                            index: idx + 1,
                            relationship: '',
                          })}
                      </span>
                    </div>
                  );
                })
              ) : (
                <div className="flex items-center gap-1.5 font-medium">
                  <span className="w-2.5 h-2.5 rounded-full inline-block shrink-0 bg-brand" />
                  <span className="text-secondary">
                    {t('tree.kinship.highlightSelected')}
                  </span>
                </div>
              )}
            </div>
            <Button
              variant="ghost"
              size="sm"
              leftIcon={<X className="h-3 w-3" />}
              onClick={() => {
                setHighlightedPathIds([]);
                setHighlightedTraces([]);
              }}
              className="h-8 min-h-8 px-2.5 text-xs ml-1 touch-target-expand"
            >
              {t('tree.kinship.clearHighlights')}
            </Button>
          </div>
        )}

        {mode === 'fan' ? (
          <FanChartCanvas
            focalPersonId={focalId ?? layout.focalId}
            nodes={treeView.nodes}
            edges={treeView.edges}
            generations={generations}
            onGenerationsChange={setGenerations}
            accessTier={
              treeView.access_tier as
                | 'public_anonymous'
                | 'viewer'
                | 'editor'
                | 'admin'
            }
            selectedPersonId={selectedPersonId}
            onSelectPerson={(p) => setSelectedPersonId(p.id)}
            onFocusPerson={(p) => setFocalId(p.id)}
          />
        ) : (
        /* Hardware-Accelerated Transformed World Stage */
        <div
          style={{
            transform: `translate3d(${isPanning ? pan.x : Math.round(pan.x)}px, ${isPanning ? pan.y : Math.round(pan.y)}px, 0) scale(${zoom})`,
            transformOrigin: '0 0',
          }}
          className={
            isAnimatingCamera
              ? 'absolute left-0 top-0 transition-transform duration-250 ease-out will-change-transform'
              : 'absolute left-0 top-0 will-change-transform'
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
                    style={{ pointerEvents: 'stroke' }}
                    className="cursor-pointer focus:outline-hidden focus-visible:outline-hidden focus-visible:stroke-brand/50 transition-colors"
                    role="button"
                    tabIndex={0}
                    aria-label={`${edgeAriaLabel}: ${t('tree.connection.select_hint')}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedEdge(edge);
                    }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        e.stopPropagation();
                        setSelectedEdge(edge);
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
        )}

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

      <KinshipFinderDialog
        open={kinshipDialogOpen}
        onOpenChange={(open) => {
          setKinshipDialogOpen(open);
          if (!open) {
            setKinshipInitialA(null);
            setKinshipInitialB(null);
          }
        }}
        people={treeView.nodes}
        edges={treeView.edges}
        initialPersonAId={
          kinshipInitialA ??
          resolvedHomePersonId ??
          focalId ??
          treeView.nodes[0]?.id
        }
        initialPersonBId={
          kinshipInitialB ??
          selectedPersonId ??
          treeView.nodes[1]?.id ??
          treeView.nodes[0]?.id
        }
        accessTier={
          treeView.access_tier as
            | 'public_anonymous'
            | 'viewer'
            | 'editor'
            | 'admin'
        }
        onHighlightPath={(ids, traces) => {
          setHighlightedPathIds(ids);
          setHighlightedTraces(traces ?? []);
          if (mode === 'fan') {
            setMode('hourglass');
          }
          requestAnimationFrame(() => {
            frameHighlightedNodes(ids);
          });
        }}
      />
    </div>
  );
};
