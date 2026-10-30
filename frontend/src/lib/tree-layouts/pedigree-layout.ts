import { TreeViewResponse } from '../../api/client';
import {
  buildHorizontalBezierPath,
  buildSpouseConnectorPath,
} from './curve-routing';
import {
  buildTreeGraphIndex,
  CARD_HEIGHT,
  CARD_WIDTH,
  computeLayoutBounds,
  H_GAP,
  PlaceholderSlot,
  PositionedEdge,
  PositionedNode,
  SPOUSE_GAP,
  TreeLayoutResult,
  V_GAP,
  createPositionedEdge,
} from './types';

export function computePedigreeLayout(
  data: TreeViewResponse,
  maxGenerationsOverride?: number
): TreeLayoutResult {
  const { nodeMap, relativesMap, focalId } = buildTreeGraphIndex(data);
  const nodes: PositionedNode[] = [];
  const placeholders: PlaceholderSlot[] = [];
  const edges: PositionedEdge[] = [];

  if (!focalId || !nodeMap.has(focalId)) {
    return {
      mode: 'pedigree',
      focalId: null,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds(nodes, placeholders),
    };
  }

  const requestedGen = maxGenerationsOverride ?? data.generations ?? 4;
  // Determine actual ancestor depth (at least 1 so parent placeholders show, capped at 4 for clean binary spacing)
  const findMaxDepth = (id: string, depth: number, visited: Set<string>): number => {
    if (depth >= requestedGen - 1 || visited.has(id)) return depth;
    visited.add(id);
    const rel = relativesMap.get(id);
    if (!rel) return depth;
    let best = depth;
    if (rel.fatherId && nodeMap.has(rel.fatherId)) {
      best = Math.max(best, findMaxDepth(rel.fatherId, depth + 1, new Set(visited)));
    }
    if (rel.motherId && nodeMap.has(rel.motherId)) {
      best = Math.max(best, findMaxDepth(rel.motherId, depth + 1, new Set(visited)));
    }
    return best;
  };

  const detectedDepth = findMaxDepth(focalId, 0, new Set());
  const effectiveMaxGen = Math.min(4, Math.max(1, detectedDepth + 1, Math.min(2, requestedGen - 1)));

  const slotY = (gen: number, slotIndex: number): number => {
    // At effectiveMaxGen, leaves are spaced by CARD_HEIGHT + V_GAP
    // And at any gen <= effectiveMaxGen, slotIndex in [0, 2^gen - 1] is the midpoint of its two parent slots
    const scale = Math.pow(2, effectiveMaxGen - gen);
    const leafCenterIndex = slotIndex * scale + (scale - 1) / 2;
    const totalLeaves = Math.pow(2, effectiveMaxGen);
    return (leafCenterIndex - (totalLeaves - 1) / 2) * (CARD_HEIGHT + V_GAP);
  };

  const placedIds = new Set<string>();

  const traverseAncestors = (
    personId: string,
    gen: number,
    slotIdx: number
  ) => {
    const person = nodeMap.get(personId);
    if (!person || placedIds.has(personId)) return;
    placedIds.add(personId);

    const x = gen * (CARD_WIDTH + H_GAP);
    const y = slotY(gen, slotIdx);

    nodes.push({
      id: personId,
      node: person,
      x,
      y,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: gen,
      isFocal: gen === 0,
    });

    if (gen >= effectiveMaxGen) return;

    const rel = relativesMap.get(personId);
    const nextGen = gen + 1;
    const nextX = nextGen * (CARD_WIDTH + H_GAP);

    // Father slot (2 * slotIdx)
    const fatherSlotIdx = slotIdx * 2;
    const fatherY = slotY(nextGen, fatherSlotIdx);
    const fatherId = rel?.fatherId ?? null;

    if (fatherId && nodeMap.has(fatherId) && !placedIds.has(fatherId)) {
      traverseAncestors(fatherId, nextGen, fatherSlotIdx);
      const sx = x + CARD_WIDTH;
      const sy = y + CARD_HEIGHT / 2;
      const tx = nextX;
      const ty = fatherY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `ped-edge-${personId}-${fatherId}`,
          sourceId: personId,
          targetId: fatherId,
          kind: 'parent_child',
          sourceX: sx,
          sourceY: sy,
          targetX: tx,
          targetY: ty,
          pathD: buildHorizontalBezierPath(sx, sy, tx, ty),
          isDirectLineage: true,
        })
      );
    } else if (gen <= 1) {
      const phId = `ph-father-${personId}`;
      placeholders.push({
        id: phId,
        anchorId: personId,
        relation: 'father',
        x: nextX,
        y: fatherY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: nextGen,
      });
      const sx = x + CARD_WIDTH;
      const sy = y + CARD_HEIGHT / 2;
      const tx = nextX;
      const ty = fatherY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `ped-ph-edge-${phId}`,
          sourceId: personId,
          targetId: phId,
          kind: 'placeholder',
          sourceX: sx,
          sourceY: sy,
          targetX: tx,
          targetY: ty,
          pathD: buildHorizontalBezierPath(sx, sy, tx, ty),
        })
      );
    }

    // Mother slot (2 * slotIdx + 1)
    const motherSlotIdx = slotIdx * 2 + 1;
    const motherY = slotY(nextGen, motherSlotIdx);
    const motherId = rel?.motherId ?? null;

    if (motherId && nodeMap.has(motherId) && !placedIds.has(motherId)) {
      traverseAncestors(motherId, nextGen, motherSlotIdx);
      const sx = x + CARD_WIDTH;
      const sy = y + CARD_HEIGHT / 2;
      const tx = nextX;
      const ty = motherY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `ped-edge-${personId}-${motherId}`,
          sourceId: personId,
          targetId: motherId,
          kind: 'parent_child',
          sourceX: sx,
          sourceY: sy,
          targetX: tx,
          targetY: ty,
          pathD: buildHorizontalBezierPath(sx, sy, tx, ty),
          isDirectLineage: true,
        })
      );
    } else if (gen <= 1) {
      const phId = `ph-mother-${personId}`;
      placeholders.push({
        id: phId,
        anchorId: personId,
        relation: 'mother',
        x: nextX,
        y: motherY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: nextGen,
      });
      const sx = x + CARD_WIDTH;
      const sy = y + CARD_HEIGHT / 2;
      const tx = nextX;
      const ty = motherY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `ped-ph-edge-${phId}`,
          sourceId: personId,
          targetId: phId,
          kind: 'placeholder',
          sourceX: sx,
          sourceY: sy,
          targetX: tx,
          targetY: ty,
          pathD: buildHorizontalBezierPath(sx, sy, tx, ty),
        })
      );
    }
  };

  traverseAncestors(focalId, 0, 0);

  // Also place focal person's spouse below focal person if present
  const focalRel = relativesMap.get(focalId);
  const focalNode = nodes.find((n) => n.id === focalId);
  if (focalNode && focalRel && focalRel.spouseIds.length > 0) {
    const spouseId = focalRel.spouseIds[0];
    const spouseDto = nodeMap.get(spouseId);
    if (spouseDto && !placedIds.has(spouseId)) {
      placedIds.add(spouseId);
      const spouseY = focalNode.y + CARD_HEIGHT + SPOUSE_GAP;
      nodes.push({
        id: spouseId,
        node: spouseDto,
        x: focalNode.x,
        y: spouseY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: 0,
        isFocal: false,
        isSpouseOfFocal: true,
      });
      const sx = focalNode.x + CARD_WIDTH / 2;
      const sy = focalNode.y + CARD_HEIGHT;
      const tx = focalNode.x + CARD_WIDTH / 2;
      const ty = spouseY;
      edges.push(
        createPositionedEdge({
          id: `ped-spouse-${focalId}-${spouseId}`,
          sourceId: focalId,
          targetId: spouseId,
          kind: 'spouse',
          sourceX: sx,
          sourceY: sy,
          targetX: tx,
          targetY: ty,
          pathD: buildSpouseConnectorPath(sx, sy, tx, ty),
        })
      );
    }
  }

  return {
    mode: 'pedigree',
    focalId,
    nodes,
    placeholders,
    edges,
    bounds: computeLayoutBounds(nodes, placeholders),
  };
}
