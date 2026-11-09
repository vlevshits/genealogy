import {
  buildHorizontalBusPath,
  buildSpouseConnectorPath,
} from './curve-routing';
import {
  CARD_HEIGHT,
  CARD_WIDTH,
  computeLayoutBounds,
  createPositionedEdge,
  H_GAP,
  PlaceholderSlot,
  PositionedNode,
  TreeLayoutResult,
} from './types';

/**
 * Transforms any vertical hierarchical tree layout into a horizontal layout
 * by transposing the rank (generation) axis to X and lateral branching axis to Y.
 * Uses orthogonal bus routing with smooth filleted corners consistent with the rest
 * of the application, connecting parent marital stems directly from the spouse connector
 * and staggering family tracks in inter-column corridors to prevent line collisions.
 */
export function transformLayoutToHorizontal(
  layout: TreeLayoutResult
): TreeLayoutResult {
  const H_GAP_HORIZ = 100;
  const COL_STEP = CARD_WIDTH + H_GAP_HORIZ;
  const LATERAL_SCALE = (CARD_HEIGHT + 40) / (CARD_WIDTH + H_GAP);

  const nodeMap = new Map<string, PositionedNode>();
  const nodes = layout.nodes.map((node) => {
    const gen = node.generation;
    const x = gen * COL_STEP;
    const y = node.x * LATERAL_SCALE;
    const transformed: PositionedNode = {
      ...node,
      x,
      y,
    };
    nodeMap.set(node.id, transformed);
    return transformed;
  });

  const phMap = new Map<string, PlaceholderSlot>();
  const placeholders = layout.placeholders.map((ph) => {
    const gen = ph.generation;
    const x = gen * COL_STEP;
    const y = ph.x * LATERAL_SCALE;
    const transformed: PlaceholderSlot = {
      ...ph,
      x,
      y,
    };
    phMap.set(ph.id, transformed);
    return transformed;
  });

  // Build union points for couples so parent-child stems originate directly from marital bars
  const unionMap = new Map<string, { x: number; y: number }>();
  for (const edge of layout.edges) {
    if (edge.kind !== 'spouse') continue;
    const srcNode = nodeMap.get(edge.sourceId);
    const tgtNode = nodeMap.get(edge.targetId);
    if (!srcNode || !tgtNode) continue;
    const top = srcNode.y < tgtNode.y ? srcNode : tgtNode;
    const bottom = srcNode.y < tgtNode.y ? tgtNode : srcNode;
    const unionPt = {
      x: top.x + CARD_WIDTH / 2,
      y: (top.y + bottom.y + CARD_HEIGHT) / 2,
    };
    unionMap.set(srcNode.id, unionPt);
    unionMap.set(tgtNode.id, unionPt);
    if (edge.familyId) {
      unionMap.set(edge.familyId, unionPt);
    }
  }

  // Stagger orthogonal runner lanes per family in inter-column corridors
  const familyTrackMap = new Map<string, number>();
  let trackCounter = 0;
  for (const edge of layout.edges) {
    if (edge.kind !== 'parent_child' && edge.kind !== 'placeholder') continue;
    const famKey = edge.familyId ?? edge.sourceId;
    if (!familyTrackMap.has(famKey)) {
      familyTrackMap.set(famKey, trackCounter++);
    }
  }

  const edges = layout.edges.map((edge) => {
    const srcNode = nodeMap.get(edge.sourceId);
    const tgtNode = nodeMap.get(edge.targetId);

    if (edge.kind === 'spouse') {
      if (!srcNode || !tgtNode) return edge;
      const topNode = srcNode.y < tgtNode.y ? srcNode : tgtNode;
      const bottomNode = srcNode.y < tgtNode.y ? tgtNode : srcNode;
      const cx = topNode.x + CARD_WIDTH / 2;
      const sy = topNode.y + CARD_HEIGHT;
      const ty = bottomNode.y;
      return createPositionedEdge({
        id: edge.id,
        sourceId: edge.sourceId,
        targetId: edge.targetId,
        familyId: edge.familyId,
        kind: 'spouse',
        sourceX: cx,
        sourceY: sy,
        targetX: cx,
        targetY: ty,
        pathD: buildSpouseConnectorPath(cx, sy, cx, ty),
        isDirectLineage: edge.isDirectLineage,
      });
    }

    if (edge.kind === 'parent_child' || edge.kind === 'placeholder') {
      const tgtPh = phMap.get(edge.targetId);
      const srcX = srcNode ? srcNode.x : 0;
      const srcY = srcNode ? srcNode.y : 0;
      const tgtX = tgtNode ? tgtNode.x : (tgtPh ? tgtPh.x : edge.targetX);
      const tgtY = tgtNode ? tgtNode.y : (tgtPh ? tgtPh.y : edge.targetY);

      let sx: number;
      let sy: number;
      let tx: number;
      let ty: number;
      let corridorLeft: number;

      if (srcX <= tgtX) {
        // Source is to the left (parent / ancestor)
        const unionPt = edge.familyId
          ? unionMap.get(edge.familyId)
          : (srcNode ? unionMap.get(srcNode.id) : undefined);
        if (unionPt) {
          sx = unionPt.x;
          sy = unionPt.y;
        } else {
          sx = srcX + CARD_WIDTH;
          sy = srcY + CARD_HEIGHT / 2;
        }
        tx = tgtX;
        ty = tgtY + CARD_HEIGHT / 2;
        corridorLeft = srcX + CARD_WIDTH;
      } else {
        // Target is to the left
        const unionPt = edge.familyId
          ? unionMap.get(edge.familyId)
          : (tgtNode ? unionMap.get(tgtNode.id) : undefined);
        if (unionPt) {
          sx = unionPt.x;
          sy = unionPt.y;
        } else {
          sx = tgtX + CARD_WIDTH;
          sy = tgtY + CARD_HEIGHT / 2;
        }
        tx = srcX;
        ty = srcY + CARD_HEIGHT / 2;
        corridorLeft = tgtX + CARD_WIDTH;
      }

      const famKey = edge.familyId ?? edge.sourceId;
      const trackIdx = familyTrackMap.get(famKey) ?? 0;
      const staggerOffsets = [0, 16, -16, 28, -28, 8, -8];
      const offset = staggerOffsets[trackIdx % staggerOffsets.length];
      const xBus = corridorLeft + H_GAP_HORIZ / 2 + offset;

      return createPositionedEdge({
        id: edge.id,
        sourceId: edge.sourceId,
        targetId: edge.targetId,
        familyId: edge.familyId,
        kind: edge.kind,
        sourceX: sx,
        sourceY: sy,
        targetX: tx,
        targetY: ty,
        xBus,
        pathD: buildHorizontalBusPath(sx, sy, tx, ty, xBus),
        isDirectLineage: edge.isDirectLineage,
      });
    }

    return edge;
  });

  return {
    ...layout,
    orientation: 'horizontal',
    nodes,
    placeholders,
    edges,
    bounds: computeLayoutBounds(nodes, placeholders),
  };
}
