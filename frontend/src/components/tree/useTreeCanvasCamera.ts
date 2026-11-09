import React, { useCallback, useEffect, useRef, useState } from "react";
import {
  CARD_HEIGHT,
  CARD_WIDTH,
  TreeLayoutResult,
} from "../../lib/tree-layouts";

export const MIN_ZOOM = 0.1;
export const MAX_ZOOM = 3.0;

export function useTreeCanvasCamera(params: {
  layout: TreeLayoutResult;
  focalId: string | null;
  isMobileViewport?: boolean;
}) {
  const { layout, focalId, isMobileViewport } = params;

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
  const [isPanning, setIsPanning] = useState<boolean>(false);

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

  const [isMobile, setIsMobile] = useState<boolean>(() => {
    if (isMobileViewport !== undefined) return isMobileViewport;
    if (typeof window !== "undefined") return window.innerWidth < 768;
    return false;
  });

  useEffect(() => {
    if (isMobileViewport !== undefined) {
      setIsMobile(isMobileViewport);
      return;
    }
    if (typeof window === "undefined") return;
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
    window.addEventListener("resize", updateViewport);
    return () => window.removeEventListener("resize", updateViewport);
  }, [isMobileViewport]);

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
        (nodeId ? layout.nodes.find((pn) => pn.id === nodeId) : null) ??
        layout.nodes.find((pn) => pn.isFocal) ??
        (focalId ? layout.nodes.find((pn) => pn.id === focalId) : null) ??
        layout.nodes[0];
      if (!target) return;
      const targetZoom = isMobile ? 0.85 : undefined;
      const nodeW = target.width || CARD_WIDTH;
      const nodeH = target.height || CARD_HEIGHT;
      centerOnWorldPoint(
        target.x + nodeW / 2,
        target.y + nodeH / 2,
        targetZoom,
        animate
      );
    },
    [layout.nodes, centerOnWorldPoint, isMobile, focalId]
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

    if (focalNode) {
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

  return {
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
  };
}
