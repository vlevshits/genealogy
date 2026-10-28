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

export function computeHourglassLayout(
  data: TreeViewResponse,
  maxGenerationsOverride?: number
): TreeLayoutResult {
  const { nodeMap, relativesMap, focalId } = buildTreeGraphIndex(data);
  const nodes: PositionedNode[] = [];
  const placeholders: PlaceholderSlot[] = [];
  const edges: PositionedEdge[] = [];

  if (!focalId || !nodeMap.has(focalId)) {
    return {
      mode: 'hourglass',
      focalId: null,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds(nodes, placeholders),
    };
  }

  const maxGen = maxGenerationsOverride ?? data.generations ?? 4;
  const ROW_STEP = CARD_HEIGHT + V_GAP + 28;
  const placedIds = new Set<string>();

  // 1. Place Focal Person at (0, 0) + Primary Spouse side-by-side
  const focalDto = nodeMap.get(focalId)!;
  placedIds.add(focalId);
  const focalRel = relativesMap.get(focalId);

  let focalSpouseId: string | null = null;
  if (focalRel && focalRel.spouseIds.length > 0) {
    const candidate = focalRel.spouseIds.find((sid) => nodeMap.has(sid));
    if (candidate) {
      focalSpouseId = candidate;
      placedIds.add(candidate);
    }
  }

  const focalX = focalSpouseId ? -(CARD_WIDTH + SPOUSE_GAP / 2) : -CARD_WIDTH / 2;
  nodes.push({
    id: focalId,
    node: focalDto,
    x: focalX,
    y: 0,
    width: CARD_WIDTH,
    height: CARD_HEIGHT,
    generation: 0,
    isFocal: true,
  });

  let unionCenterX = focalX + CARD_WIDTH / 2;
  if (focalSpouseId) {
    const spouseDto = nodeMap.get(focalSpouseId)!;
    const spouseX = SPOUSE_GAP / 2;
    nodes.push({
      id: focalSpouseId,
      node: spouseDto,
      x: spouseX,
      y: 0,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: 0,
      isFocal: false,
      isSpouseOfFocal: true,
    });

    edges.push({
      id: `hg-spouse-${focalId}-${focalSpouseId}`,
      sourceId: focalId,
      targetId: focalSpouseId,
      kind: 'spouse',
      sourceX: focalX + CARD_WIDTH,
      sourceY: CARD_HEIGHT / 2,
      targetX: spouseX,
      targetY: CARD_HEIGHT / 2,
      pathD: buildSpouseConnectorPath(
        focalX + CARD_WIDTH,
        CARD_HEIGHT / 2,
        spouseX,
        CARD_HEIGHT / 2
      ),
    });
    unionCenterX = 0;
  }

  // 2. Place Direct Ancestors UPWARDS (Y < 0)
  const ancestorGenLimit = Math.min(3, Math.max(1, maxGen - 1));
  const ancestorSlotX = (gen: number, slotIndex: number): number => {
    const scale = Math.pow(2, ancestorGenLimit - gen);
    const leafCenterIndex = slotIndex * scale + (scale - 1) / 2;
    const totalLeaves = Math.pow(2, ancestorGenLimit);
    return (
      focalX +
      CARD_WIDTH / 2 +
      (leafCenterIndex - (totalLeaves - 1) / 2) * (CARD_WIDTH + 32) -
      CARD_WIDTH / 2
    );
  };

  const placeAncestorBranch = (
    childId: string,
    childX: number,
    childY: number,
    gen: number,
    slotIdx: number
  ) => {
    if (gen > ancestorGenLimit) return;
    const rel = relativesMap.get(childId);
    const parentY = -gen * ROW_STEP;

    const fatherSlotIdx = slotIdx * 2;
    const fatherX = ancestorSlotX(gen, fatherSlotIdx);
    let fatherId = rel?.fatherId ?? null;

    const motherSlotIdx = slotIdx * 2 + 1;
    const motherX = ancestorSlotX(gen, motherSlotIdx);
    let motherId = rel?.motherId ?? null;

    if (fatherId && motherId && fatherId === motherId) {
      const pNode = nodeMap.get(fatherId);
      if (pNode?.sex === 'F') {
        fatherId = null;
      } else {
        motherId = null;
      }
    }

    const fatherNode = fatherId ? nodeMap.get(fatherId) : undefined;
    const hasFather = Boolean(fatherId && fatherNode && !placedIds.has(fatherId));

    const motherNode = motherId ? nodeMap.get(motherId) : undefined;
    const hasMother = Boolean(motherId && motherNode && !placedIds.has(motherId));

    const childCenterX = childX + CARD_WIDTH / 2;

    if (hasFather && hasMother) {
      placedIds.add(fatherId!);
      placedIds.add(motherId!);
      nodes.push({
        id: fatherId!,
        node: fatherNode!,
        x: fatherX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -gen,
        isFocal: false,
      });
      nodes.push({
        id: motherId!,
        node: motherNode!,
        x: motherX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -gen,
        isFocal: false,
      });

      // Horizontal spouse connector between father and mother
      edges.push({
        id: `hg-anc-spouse-${fatherId}-${motherId}`,
        sourceId: fatherId!,
        targetId: motherId!,
        kind: 'spouse',
        sourceX: fatherX + CARD_WIDTH,
        sourceY: parentY + CARD_HEIGHT / 2,
        targetX: motherX,
        targetY: parentY + CARD_HEIGHT / 2,
        pathD: buildSpouseConnectorPath(
          fatherX + CARD_WIDTH,
          parentY + CARD_HEIGHT / 2,
          motherX,
          parentY + CARD_HEIGHT / 2
        ),
      });

      // Single direct lineage connector dropping from center of the couple's union down to child's top center
      const unionCenterX = (fatherX + CARD_WIDTH + motherX) / 2;
      edges.push({
        id: `hg-anc-${fatherId}-${childId}`,
        sourceId: fatherId!,
        targetId: childId,
        kind: 'parent_child',
        sourceX: unionCenterX,
        sourceY: parentY + CARD_HEIGHT / 2,
        targetX: childCenterX,
        targetY: childY,
        pathD: buildVerticalBusPath(
          unionCenterX,
          parentY + CARD_HEIGHT / 2,
          childCenterX,
          childY
        ),
        isDirectLineage: true,
      });

      placeAncestorBranch(fatherId!, fatherX, parentY, gen + 1, fatherSlotIdx);
      placeAncestorBranch(motherId!, motherX, parentY, gen + 1, motherSlotIdx);
    } else if (hasFather) {
      placedIds.add(fatherId!);
      nodes.push({
        id: fatherId!,
        node: fatherNode!,
        x: fatherX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -gen,
        isFocal: false,
      });
      edges.push({
        id: `hg-anc-${fatherId}-${childId}`,
        sourceId: fatherId!,
        targetId: childId,
        kind: 'parent_child',
        sourceX: fatherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: childCenterX,
        targetY: childY,
        pathD: buildVerticalBusPath(
          fatherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          childCenterX,
          childY
        ),
        isDirectLineage: true,
      });
      placeAncestorBranch(fatherId!, fatherX, parentY, gen + 1, fatherSlotIdx);

      if (gen === 1) {
        const phId = `hg-ph-mother-${childId}`;
        placeholders.push({
          id: phId,
          anchorId: childId,
          relation: 'mother',
          x: motherX,
          y: parentY,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: -gen,
        });
        edges.push({
          id: `hg-ph-edge-${phId}`,
          sourceId: phId,
          targetId: childId,
          kind: 'placeholder',
          sourceX: motherX + CARD_WIDTH / 2,
          sourceY: parentY + CARD_HEIGHT,
          targetX: childCenterX,
          targetY: childY,
          pathD: buildVerticalBusPath(
            motherX + CARD_WIDTH / 2,
            parentY + CARD_HEIGHT,
            childCenterX,
            childY
          ),
        });
      }
    } else if (hasMother) {
      placedIds.add(motherId!);
      nodes.push({
        id: motherId!,
        node: motherNode!,
        x: motherX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -gen,
        isFocal: false,
      });
      edges.push({
        id: `hg-anc-${motherId}-${childId}`,
        sourceId: motherId!,
        targetId: childId,
        kind: 'parent_child',
        sourceX: motherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: childCenterX,
        targetY: childY,
        pathD: buildVerticalBusPath(
          motherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          childCenterX,
          childY
        ),
        isDirectLineage: true,
      });
      placeAncestorBranch(motherId!, motherX, parentY, gen + 1, motherSlotIdx);

      if (gen === 1) {
        const phId = `hg-ph-father-${childId}`;
        placeholders.push({
          id: phId,
          anchorId: childId,
          relation: 'father',
          x: fatherX,
          y: parentY,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: -gen,
        });
        edges.push({
          id: `hg-ph-edge-${phId}`,
          sourceId: phId,
          targetId: childId,
          kind: 'placeholder',
          sourceX: fatherX + CARD_WIDTH / 2,
          sourceY: parentY + CARD_HEIGHT,
          targetX: childCenterX,
          targetY: childY,
          pathD: buildVerticalBusPath(
            fatherX + CARD_WIDTH / 2,
            parentY + CARD_HEIGHT,
            childCenterX,
            childY
          ),
        });
      }
    } else if (gen === 1) {
      const phFatherId = `hg-ph-father-${childId}`;
      placeholders.push({
        id: phFatherId,
        anchorId: childId,
        relation: 'father',
        x: fatherX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -gen,
      });
      edges.push({
        id: `hg-ph-edge-${phFatherId}`,
        sourceId: phFatherId,
        targetId: childId,
        kind: 'placeholder',
        sourceX: fatherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: childCenterX,
        targetY: childY,
        pathD: buildVerticalBusPath(
          fatherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          childCenterX,
          childY
        ),
      });

      const phMotherId = `hg-ph-mother-${childId}`;
      placeholders.push({
        id: phMotherId,
        anchorId: childId,
        relation: 'mother',
        x: motherX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -gen,
      });
      edges.push({
        id: `hg-ph-edge-${phMotherId}`,
        sourceId: phMotherId,
        targetId: childId,
        kind: 'placeholder',
        sourceX: motherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: childCenterX,
        targetY: childY,
        pathD: buildVerticalBusPath(
          motherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          childCenterX,
          childY
        ),
      });
    }
  };

  placeAncestorBranch(focalId, focalX, 0, 1, 0);

  // 3. Place Descendants DOWNWARDS (Y > 0)
  interface DescPlan {
    personId: string;
    spouseId: string | null;
    children: DescPlan[];
    coupleWidth: number;
    subtreeWidth: number;
  }

  const buildDescPlan = (personId: string, gen: number): DescPlan | null => {
    if (placedIds.has(personId) || !nodeMap.has(personId)) return null;
    placedIds.add(personId);

    const rel = relativesMap.get(personId);
    let spouseId: string | null = null;
    if (rel && rel.spouseIds.length > 0) {
      const cand = rel.spouseIds.find(
        (sid) => nodeMap.has(sid) && !placedIds.has(sid)
      );
      if (cand) {
        spouseId = cand;
        placedIds.add(cand);
      }
    }

    const coupleWidth = spouseId ? CARD_WIDTH * 2 + SPOUSE_GAP : CARD_WIDTH;
    const children: DescPlan[] = [];
    if (gen < maxGen - 1 && rel) {
      const childIds = [...rel.childIds];
      if (spouseId) {
        for (const cid of relativesMap.get(spouseId)?.childIds ?? []) {
          if (!childIds.includes(cid)) childIds.push(cid);
        }
      }
      for (const cid of childIds) {
        const cp = buildDescPlan(cid, gen + 1);
        if (cp) children.push(cp);
      }
    }

    const childrenWidth =
      children.reduce((acc, c) => acc + c.subtreeWidth, 0) +
      Math.max(0, children.length - 1) * H_GAP;

    return {
      personId,
      spouseId,
      children,
      coupleWidth,
      subtreeWidth: Math.max(coupleWidth, childrenWidth),
    };
  };

  const rootChildIds = [...(focalRel?.childIds ?? [])];
  if (focalSpouseId) {
    for (const cid of relativesMap.get(focalSpouseId)?.childIds ?? []) {
      if (!rootChildIds.includes(cid)) rootChildIds.push(cid);
    }
  }

  const rootChildPlans: DescPlan[] = [];
  for (const cid of rootChildIds) {
    const cp = buildDescPlan(cid, 1);
    if (cp) rootChildPlans.push(cp);
  }

  const placeDescPlan = (
    plan: DescPlan,
    leftX: number,
    gen: number
  ): { anchorX: number; anchorY: number } => {
    const y = gen * ROW_STEP;
    const coupleStartX = leftX + (plan.subtreeWidth - plan.coupleWidth) / 2;

    nodes.push({
      id: plan.personId,
      node: nodeMap.get(plan.personId)!,
      x: coupleStartX,
      y,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: gen,
      isFocal: false,
    });

    let pUnionCenterX = coupleStartX + CARD_WIDTH / 2;
    if (plan.spouseId && nodeMap.has(plan.spouseId)) {
      const spX = coupleStartX + CARD_WIDTH + SPOUSE_GAP;
      nodes.push({
        id: plan.spouseId,
        node: nodeMap.get(plan.spouseId)!,
        x: spX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: false,
      });
      edges.push({
        id: `hg-desc-spouse-${plan.personId}-${plan.spouseId}`,
        sourceId: plan.personId,
        targetId: plan.spouseId,
        kind: 'spouse',
        sourceX: coupleStartX + CARD_WIDTH,
        sourceY: y + CARD_HEIGHT / 2,
        targetX: spX,
        targetY: y + CARD_HEIGHT / 2,
        pathD: buildSpouseConnectorPath(
          coupleStartX + CARD_WIDTH,
          y + CARD_HEIGHT / 2,
          spX,
          y + CARD_HEIGHT / 2
        ),
      });
      pUnionCenterX = coupleStartX + CARD_WIDTH + SPOUSE_GAP / 2;
    }

    if (plan.children.length > 0) {
      const totalChW =
        plan.children.reduce((acc, c) => acc + c.subtreeWidth, 0) +
        Math.max(0, plan.children.length - 1) * H_GAP;
      let cursorX = leftX + (plan.subtreeWidth - totalChW) / 2;
      for (const ch of plan.children) {
        const chAnchor = placeDescPlan(ch, cursorX, gen + 1);
        edges.push({
          id: `hg-desc-edge-${plan.personId}-${ch.personId}`,
          sourceId: plan.personId,
          targetId: ch.personId,
          kind: 'parent_child',
          sourceX: pUnionCenterX,
          sourceY: y + CARD_HEIGHT,
          targetX: chAnchor.anchorX,
          targetY: chAnchor.anchorY,
          pathD: buildVerticalBusPath(
            pUnionCenterX,
            y + CARD_HEIGHT,
            chAnchor.anchorX,
            chAnchor.anchorY
          ),
          isDirectLineage: true,
        });
        cursorX += ch.subtreeWidth + H_GAP;
      }
    }

    return {
      anchorX: coupleStartX + CARD_WIDTH / 2,
      anchorY: y,
    };
  };

  if (rootChildPlans.length > 0) {
    const totalW =
      rootChildPlans.reduce((acc, c) => acc + c.subtreeWidth, 0) +
      Math.max(0, rootChildPlans.length - 1) * H_GAP;
    let cursorX = unionCenterX - totalW / 2;
    for (const cp of rootChildPlans) {
      const chAnchor = placeDescPlan(cp, cursorX, 1);
      edges.push({
        id: `hg-root-child-${focalId}-${cp.personId}`,
        sourceId: focalId,
        targetId: cp.personId,
        kind: 'parent_child',
        sourceX: unionCenterX,
        sourceY: CARD_HEIGHT,
        targetX: chAnchor.anchorX,
        targetY: chAnchor.anchorY,
        pathD: buildVerticalBusPath(
          unionCenterX,
          CARD_HEIGHT,
          chAnchor.anchorX,
          chAnchor.anchorY
        ),
        isDirectLineage: true,
      });
      cursorX += cp.subtreeWidth + H_GAP;
    }
  }

  return {
    mode: 'hourglass',
    focalId,
    nodes,
    placeholders,
    edges,
    bounds: computeLayoutBounds(nodes, placeholders),
  };
}
