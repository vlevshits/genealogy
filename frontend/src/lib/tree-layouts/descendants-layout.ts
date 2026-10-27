import { TreeViewResponse } from '../../api/client';
import {
  buildSpouseConnectorPath,
  buildVerticalBusPath,
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
} from './types';

interface SubtreePlan {
  personId: string;
  spouseId: string | null;
  childrenPlans: SubtreePlan[];
  coupleWidth: number;
  subtreeWidth: number;
}

export function computeDescendantsLayout(
  data: TreeViewResponse,
  maxGenerationsOverride?: number
): TreeLayoutResult {
  const { nodeMap, relativesMap, focalId } = buildTreeGraphIndex(data);
  const nodes: PositionedNode[] = [];
  const placeholders: PlaceholderSlot[] = [];
  const edges: PositionedEdge[] = [];

  if (!focalId || !nodeMap.has(focalId)) {
    return {
      mode: 'descendant',
      focalId: null,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds(nodes, placeholders),
    };
  }

  const maxGen = maxGenerationsOverride ?? data.generations ?? 4;
  const visited = new Set<string>();

  const buildPlan = (personId: string, gen: number): SubtreePlan | null => {
    if (visited.has(personId) || !nodeMap.has(personId)) return null;
    visited.add(personId);

    const rel = relativesMap.get(personId);
    let spouseId: string | null = null;
    if (rel && rel.spouseIds.length > 0) {
      const candidate = rel.spouseIds.find(
        (sid) => nodeMap.has(sid) && !visited.has(sid)
      );
      if (candidate) {
        spouseId = candidate;
        visited.add(candidate);
      }
    }

    const coupleWidth = spouseId
      ? CARD_WIDTH * 2 + SPOUSE_GAP
      : CARD_WIDTH;

    const childrenPlans: SubtreePlan[] = [];
    if (gen < maxGen - 1 && rel) {
      // Collect children of person (and spouse)
      const childIds = [...rel.childIds];
      if (spouseId) {
        const spouseRel = relativesMap.get(spouseId);
        for (const cid of spouseRel?.childIds ?? []) {
          if (!childIds.includes(cid)) childIds.push(cid);
        }
      }
      for (const cid of childIds) {
        const childPlan = buildPlan(cid, gen + 1);
        if (childPlan) {
          childrenPlans.push(childPlan);
        }
      }
    }

    const childrenTotalWidth =
      childrenPlans.reduce((acc, cp) => acc + cp.subtreeWidth, 0) +
      Math.max(0, childrenPlans.length - 1) * H_GAP;

    const subtreeWidth = Math.max(coupleWidth, childrenTotalWidth);

    return {
      personId,
      spouseId,
      childrenPlans,
      coupleWidth,
      subtreeWidth,
    };
  };

  const rootPlan = buildPlan(focalId, 0);
  if (!rootPlan) {
    return {
      mode: 'descendant',
      focalId,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds(nodes, placeholders),
    };
  }

  const ROW_STEP = CARD_HEIGHT + V_GAP + 28;

  const placeSubtree = (plan: SubtreePlan, leftX: number, gen: number): { anchorX: number; anchorY: number } => {
    const y = gen * ROW_STEP;
    const coupleStartX = leftX + (plan.subtreeWidth - plan.coupleWidth) / 2;

    const personDto = nodeMap.get(plan.personId)!;
    nodes.push({
      id: plan.personId,
      node: personDto,
      x: coupleStartX,
      y,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: gen,
      isFocal: plan.personId === focalId,
    });

    let unionCenterX = coupleStartX + CARD_WIDTH / 2;
    const unionBottomY = y + CARD_HEIGHT;

    if (plan.spouseId && nodeMap.has(plan.spouseId)) {
      const spouseDto = nodeMap.get(plan.spouseId)!;
      const spouseX = coupleStartX + CARD_WIDTH + SPOUSE_GAP;
      nodes.push({
        id: plan.spouseId,
        node: spouseDto,
        x: spouseX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: false,
        isSpouseOfFocal: plan.personId === focalId,
      });

      const sx = coupleStartX + CARD_WIDTH;
      const sy = y + CARD_HEIGHT / 2;
      const tx = spouseX;
      const ty = y + CARD_HEIGHT / 2;
      edges.push({
        id: `desc-spouse-${plan.personId}-${plan.spouseId}`,
        sourceId: plan.personId,
        targetId: plan.spouseId,
        kind: 'spouse',
        sourceX: sx,
        sourceY: sy,
        targetX: tx,
        targetY: ty,
        pathD: buildSpouseConnectorPath(sx, sy, tx, ty),
      });

      unionCenterX = coupleStartX + CARD_WIDTH + SPOUSE_GAP / 2;
    }

    if (plan.childrenPlans.length > 0) {
      const childrenTotalWidth =
        plan.childrenPlans.reduce((acc, cp) => acc + cp.subtreeWidth, 0) +
        Math.max(0, plan.childrenPlans.length - 1) * H_GAP;

      let childCursorX = leftX + (plan.subtreeWidth - childrenTotalWidth) / 2;
      for (const childPlan of plan.childrenPlans) {
        const childAnchor = placeSubtree(childPlan, childCursorX, gen + 1);
        edges.push({
          id: `desc-edge-${plan.personId}-${childPlan.personId}`,
          sourceId: plan.personId,
          targetId: childPlan.personId,
          kind: 'parent_child',
          sourceX: unionCenterX,
          sourceY: unionBottomY,
          targetX: childAnchor.anchorX,
          targetY: childAnchor.anchorY,
          pathD: buildVerticalBusPath(
            unionCenterX,
            unionBottomY,
            childAnchor.anchorX,
            childAnchor.anchorY
          ),
          isDirectLineage: true,
        });
        childCursorX += childPlan.subtreeWidth + H_GAP;
      }
    }

    return {
      anchorX: coupleStartX + CARD_WIDTH / 2,
      anchorY: y,
    };
  };

  placeSubtree(rootPlan, -rootPlan.subtreeWidth / 2, 0);

  // Place any unvisited disconnected nodes in a tidy bottom row so no visible person is lost
  let extraIndex = 0;
  for (const n of data.nodes) {
    if (!visited.has(n.id)) {
      visited.add(n.id);
      nodes.push({
        id: n.id,
        node: n,
        x: extraIndex * (CARD_WIDTH + H_GAP),
        y: -ROW_STEP,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -1,
        isFocal: false,
      });
      extraIndex++;
    }
  }

  return {
    mode: 'descendant',
    focalId,
    nodes,
    placeholders,
    edges,
    bounds: computeLayoutBounds(nodes, placeholders),
  };
}
