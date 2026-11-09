import { TreeViewResponse } from '../../api/client';
import {
  buildHorizontalBezierPath,
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
  ROW_STEP,
  SPOUSE_GAP,
  TreeLayoutOrientation,
  TreeLayoutResult,
  V_GAP,
  createPositionedEdge,
} from './types';

interface AncestorTreeNode {
  id: string | null; // null if placeholder
  isPlaceholder?: boolean;
  placeholderRelation?: 'father' | 'mother';
  placeholderAnchorId?: string;
  gen: number;
  father?: AncestorTreeNode;
  mother?: AncestorTreeNode;

  // Computed layout metrics (vertical)
  vSubtreeWidth: number;
  vNodeRelCenterX: number;
  vFatherRelCenterX?: number;
  vMotherRelCenterX?: number;
  vPlacedX?: number;
  vPlacedY?: number;

  // Computed layout metrics (horizontal)
  hSubtreeHeight: number;
  hNodeRelCenterY: number;
  hFatherRelCenterY?: number;
  hMotherRelCenterY?: number;
  hPlacedX?: number;
  hPlacedY?: number;
}

export function computePedigreeLayout(
  data: TreeViewResponse,
  maxGenerationsOverride?: number,
  orientation: TreeLayoutOrientation = 'horizontal'
): TreeLayoutResult {
  const { nodeMap, relativesMap, focalId } = buildTreeGraphIndex(data);
  const nodes: PositionedNode[] = [];
  const placeholders: PlaceholderSlot[] = [];
  const edges: PositionedEdge[] = [];

  if (!focalId || !nodeMap.has(focalId)) {
    return {
      mode: 'pedigree',
      orientation,
      focalId: null,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds(nodes, placeholders),
    };
  }

  const requestedGen = maxGenerationsOverride ?? data.generations ?? 4;
  const maxGenLimit = Math.max(1, requestedGen);
  const visitedAncestors = new Set<string>();

  // ---------------------------------------------------------------------------
  // 1. Build Ancestor Tree Structure (Bottom-Up)
  // ---------------------------------------------------------------------------
  const buildAncestorTree = (
    personId: string,
    gen: number
  ): AncestorTreeNode | undefined => {
    if (visitedAncestors.has(personId)) return undefined;
    visitedAncestors.add(personId);

    const treeNode: AncestorTreeNode = {
      id: personId,
      gen,
      vSubtreeWidth: CARD_WIDTH,
      vNodeRelCenterX: CARD_WIDTH / 2,
      hSubtreeHeight: CARD_HEIGHT,
      hNodeRelCenterY: CARD_HEIGHT / 2,
    };

    if (gen < maxGenLimit) {
      const rel = relativesMap.get(personId);
      const nextGen = gen + 1;
      const fatherId = rel?.fatherId;
      const motherId = rel?.motherId;

      const hasFather = Boolean(fatherId && nodeMap.has(fatherId));
      const hasMother = Boolean(motherId && nodeMap.has(motherId));

      if (hasFather) {
        treeNode.father = buildAncestorTree(fatherId!, nextGen);
      } else if (gen === 0) {
        // Immediate parent placeholder for focal person
        treeNode.father = {
          id: null,
          isPlaceholder: true,
          placeholderRelation: 'father',
          placeholderAnchorId: personId,
          gen: nextGen,
          vSubtreeWidth: CARD_WIDTH,
          vNodeRelCenterX: CARD_WIDTH / 2,
          hSubtreeHeight: CARD_HEIGHT,
          hNodeRelCenterY: CARD_HEIGHT / 2,
        };
      }

      if (hasMother) {
        treeNode.mother = buildAncestorTree(motherId!, nextGen);
      } else if (gen === 0) {
        // Immediate parent placeholder for focal person
        treeNode.mother = {
          id: null,
          isPlaceholder: true,
          placeholderRelation: 'mother',
          placeholderAnchorId: personId,
          gen: nextGen,
          vSubtreeWidth: CARD_WIDTH,
          vNodeRelCenterX: CARD_WIDTH / 2,
          hSubtreeHeight: CARD_HEIGHT,
          hNodeRelCenterY: CARD_HEIGHT / 2,
        };
      }
    }

    return treeNode;
  };

  const root = buildAncestorTree(focalId, 0);
  if (!root) {
    return {
      mode: 'pedigree',
      orientation,
      focalId,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds(nodes, placeholders),
    };
  }

  // ---------------------------------------------------------------------------
  // 2. Compute Compact Tidy Metrics Bottom-Up
  // ---------------------------------------------------------------------------
  const computeVerticalMetrics = (node: AncestorTreeNode) => {
    if (node.father) computeVerticalMetrics(node.father);
    if (node.mother) computeVerticalMetrics(node.mother);

    if (node.father && node.mother) {
      const f = node.father;
      const m = node.mother;
      const hasAncestorsAbove = Boolean(f.father || f.mother || m.father || m.mother);

      // Distance between father's center and mother's center:
      // If neither has ancestors, they sit directly adjacent separated by SPOUSE_GAP.
      // If they have ancestors, ensure their ancestor subtrees don't collide with H_GAP.
      const subtreeGapDistance =
        f.vSubtreeWidth - f.vNodeRelCenterX + H_GAP + m.vNodeRelCenterX;
      const minCoupleDistance = CARD_WIDTH + SPOUSE_GAP;
      const centerDistance = hasAncestorsAbove
        ? Math.max(minCoupleDistance, subtreeGapDistance)
        : minCoupleDistance;

      const fCenter = Math.max(CARD_WIDTH / 2, f.vNodeRelCenterX);
      const mCenter = fCenter + centerDistance;

      node.vFatherRelCenterX = fCenter;
      node.vMotherRelCenterX = mCenter;

      const totalW = mCenter + Math.max(CARD_WIDTH / 2, m.vSubtreeWidth - m.vNodeRelCenterX);
      node.vSubtreeWidth = Math.max(CARD_WIDTH, totalW);
      node.vNodeRelCenterX = (fCenter + mCenter) / 2;
    } else if (node.father) {
      const f = node.father;
      node.vSubtreeWidth = Math.max(CARD_WIDTH, f.vSubtreeWidth);
      node.vNodeRelCenterX = f.vNodeRelCenterX;
      node.vFatherRelCenterX = f.vNodeRelCenterX;
    } else if (node.mother) {
      const m = node.mother;
      node.vSubtreeWidth = Math.max(CARD_WIDTH, m.vSubtreeWidth);
      node.vNodeRelCenterX = m.vNodeRelCenterX;
      node.vMotherRelCenterX = m.vNodeRelCenterX;
    } else {
      node.vSubtreeWidth = CARD_WIDTH;
      node.vNodeRelCenterX = CARD_WIDTH / 2;
    }
  };

  const computeHorizontalMetrics = (node: AncestorTreeNode) => {
    if (node.father) computeHorizontalMetrics(node.father);
    if (node.mother) computeHorizontalMetrics(node.mother);

    if (node.father && node.mother) {
      const f = node.father;
      const m = node.mother;
      const hasAncestorsAbove = Boolean(f.father || f.mother || m.father || m.mother);

      const subtreeGapDistance =
        f.hSubtreeHeight - f.hNodeRelCenterY + V_GAP + m.hNodeRelCenterY;
      const minCoupleDistance = CARD_HEIGHT + V_GAP;
      const centerDistance = hasAncestorsAbove
        ? Math.max(minCoupleDistance, subtreeGapDistance)
        : minCoupleDistance;

      const fCenter = Math.max(CARD_HEIGHT / 2, f.hNodeRelCenterY);
      const mCenter = fCenter + centerDistance;

      node.hFatherRelCenterY = fCenter;
      node.hMotherRelCenterY = mCenter;

      const totalH = mCenter + Math.max(CARD_HEIGHT / 2, m.hSubtreeHeight - m.hNodeRelCenterY);
      node.hSubtreeHeight = Math.max(CARD_HEIGHT, totalH);
      node.hNodeRelCenterY = (fCenter + mCenter) / 2;
    } else if (node.father) {
      const f = node.father;
      node.hSubtreeHeight = Math.max(CARD_HEIGHT, f.hSubtreeHeight);
      node.hNodeRelCenterY = f.hNodeRelCenterY;
      node.hFatherRelCenterY = f.hNodeRelCenterY;
    } else if (node.mother) {
      const m = node.mother;
      node.hSubtreeHeight = Math.max(CARD_HEIGHT, m.hSubtreeHeight);
      node.hNodeRelCenterY = m.hNodeRelCenterY;
      node.hMotherRelCenterY = m.hNodeRelCenterY;
    } else {
      node.hSubtreeHeight = CARD_HEIGHT;
      node.hNodeRelCenterY = CARD_HEIGHT / 2;
    }
  };

  // ---------------------------------------------------------------------------
  // 3. Render Positions and Connectors
  // ---------------------------------------------------------------------------
  const placedIds = new Set<string>();

  if (orientation === 'vertical') {
    computeVerticalMetrics(root);

    const assignVerticalCoords = (
      node: AncestorTreeNode,
      centerX: number,
      gen: number
    ) => {
      const x = centerX - CARD_WIDTH / 2;
      const y = gen === 0 ? 0 : -gen * ROW_STEP;
      node.vPlacedX = x;
      node.vPlacedY = y;

      if (node.isPlaceholder) {
        placeholders.push({
          id: `ph-${node.placeholderRelation}-${node.placeholderAnchorId}`,
          anchorId: node.placeholderAnchorId!,
          relation: node.placeholderRelation!,
          x,
          y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: -gen,
        });
        return;
      }

      if (!node.id) return;
      const person = nodeMap.get(node.id);
      if (!person || placedIds.has(node.id)) return;
      placedIds.add(node.id);

      nodes.push({
        id: node.id,
        node: person,
        x,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -gen,
        isFocal: gen === 0,
        isDirectLineage: true,
      });

      const nextGen = gen + 1;
      const parentRowY = -nextGen * ROW_STEP;
      const parentBusY = parentRowY + CARD_HEIGHT + V_GAP / 2;
      const spouseY = parentRowY + CARD_HEIGHT / 2;

      if (node.father && node.mother) {
        const fCenter = centerX + (node.vFatherRelCenterX! - node.vNodeRelCenterX);
        const mCenter = centerX + (node.vMotherRelCenterX! - node.vNodeRelCenterX);

        assignVerticalCoords(node.father, fCenter, nextGen);
        assignVerticalCoords(node.mother, mCenter, nextGen);

        const fX = fCenter - CARD_WIDTH / 2;
        const mX = mCenter - CARD_WIDTH / 2;

        if (node.father.id && node.mother.id) {
          // Marital bar between parents
          edges.push(
            createPositionedEdge({
              id: `ped-anc-spouse-${node.father.id}-${node.mother.id}`,
              sourceId: node.father.id,
              targetId: node.mother.id,
              kind: 'spouse',
              sourceX: fX + CARD_WIDTH,
              sourceY: spouseY,
              targetX: mX,
              targetY: spouseY,
              pathD: buildSpouseConnectorPath(fX + CARD_WIDTH, spouseY, mX, spouseY),
              isDirectLineage: true,
            })
          );

          // Vertical drop stem from parents' union center down to child
          const unionCenterX = (fX + CARD_WIDTH + mX) / 2;
          edges.push(
            createPositionedEdge({
              id: `ped-anc-${node.father.id}-${node.id}`,
              sourceId: node.father.id,
              targetId: node.id,
              kind: 'parent_child',
              sourceX: unionCenterX,
              sourceY: spouseY,
              targetX: centerX,
              targetY: y,
              yBus: parentBusY,
              pathD: buildVerticalBusPath(unionCenterX, spouseY, centerX, y, parentBusY),
              isDirectLineage: true,
            })
          );
        } else {
          // Placeholder connectors
          const targetId = node.father.id ?? `ph-father-${node.id}`;
          edges.push(
            createPositionedEdge({
              id: `ped-ph-edge-${targetId}`,
              sourceId: node.id,
              targetId,
              kind: 'placeholder',
              sourceX: centerX,
              sourceY: y,
              targetX: fCenter,
              targetY: parentRowY + CARD_HEIGHT,
              yBus: parentBusY,
              pathD: buildVerticalBusPath(centerX, y, fCenter, parentRowY + CARD_HEIGHT, parentBusY),
            })
          );
          const mTargetId = node.mother.id ?? `ph-mother-${node.id}`;
          edges.push(
            createPositionedEdge({
              id: `ped-ph-edge-${mTargetId}`,
              sourceId: node.id,
              targetId: mTargetId,
              kind: 'placeholder',
              sourceX: centerX,
              sourceY: y,
              targetX: mCenter,
              targetY: parentRowY + CARD_HEIGHT,
              yBus: parentBusY,
              pathD: buildVerticalBusPath(centerX, y, mCenter, parentRowY + CARD_HEIGHT, parentBusY),
            })
          );
        }
      } else if (node.father) {
        const fCenter = centerX + (node.vFatherRelCenterX! - node.vNodeRelCenterX);
        assignVerticalCoords(node.father, fCenter, nextGen);
        if (node.father.id) {
          edges.push(
            createPositionedEdge({
              id: `ped-anc-${node.father.id}-${node.id}`,
              sourceId: node.father.id,
              targetId: node.id,
              kind: 'parent_child',
              sourceX: fCenter,
              sourceY: parentRowY + CARD_HEIGHT,
              targetX: centerX,
              targetY: y,
              yBus: parentBusY,
              pathD: buildVerticalBusPath(fCenter, parentRowY + CARD_HEIGHT, centerX, y, parentBusY),
              isDirectLineage: true,
            })
          );
        }
      } else if (node.mother) {
        const mCenter = centerX + (node.vMotherRelCenterX! - node.vNodeRelCenterX);
        assignVerticalCoords(node.mother, mCenter, nextGen);
        if (node.mother.id) {
          edges.push(
            createPositionedEdge({
              id: `ped-anc-${node.mother.id}-${node.id}`,
              sourceId: node.mother.id,
              targetId: node.id,
              kind: 'parent_child',
              sourceX: mCenter,
              sourceY: parentRowY + CARD_HEIGHT,
              targetX: centerX,
              targetY: y,
              yBus: parentBusY,
              pathD: buildVerticalBusPath(mCenter, parentRowY + CARD_HEIGHT, centerX, y, parentBusY),
              isDirectLineage: true,
            })
          );
        }
      }
    };

    assignVerticalCoords(root, 0, 0);

    // Place focal person's spouse horizontally adjacent if present
    const focalRel = relativesMap.get(focalId);
    const focalNode = nodes.find((n) => n.id === focalId);
    if (focalNode && focalRel && focalRel.spouseIds.length > 0) {
      const spouseId = focalRel.spouseIds[0];
      const spouseDto = nodeMap.get(spouseId);
      if (spouseDto && !placedIds.has(spouseId)) {
        placedIds.add(spouseId);
        const spouseX = focalNode.x + CARD_WIDTH + SPOUSE_GAP;
        nodes.push({
          id: spouseId,
          node: spouseDto,
          x: spouseX,
          y: focalNode.y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: 0,
          isFocal: false,
          isSpouseOfFocal: true,
        });

        const spY = focalNode.y + CARD_HEIGHT / 2;
        edges.push(
          createPositionedEdge({
            id: `ped-spouse-${focalId}-${spouseId}`,
            sourceId: focalId,
            targetId: spouseId,
            kind: 'spouse',
            sourceX: focalNode.x + CARD_WIDTH,
            sourceY: spY,
            targetX: spouseX,
            targetY: spY,
            pathD: buildSpouseConnectorPath(focalNode.x + CARD_WIDTH, spY, spouseX, spY),
          })
        );
      }
    }

    return {
      mode: 'pedigree',
      orientation: 'vertical',
      focalId,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds(nodes, placeholders),
    };
  }

  // =========================================================================
  // Horizontal Orientation (Left-to-Right Pedigree)
  // =========================================================================
  computeHorizontalMetrics(root);

  const assignHorizontalCoords = (
    node: AncestorTreeNode,
    centerY: number,
    gen: number
  ) => {
    const x = gen * (CARD_WIDTH + H_GAP);
    const y = centerY - CARD_HEIGHT / 2;
    node.hPlacedX = x;
    node.hPlacedY = y;

    if (node.isPlaceholder) {
      placeholders.push({
        id: `ph-${node.placeholderRelation}-${node.placeholderAnchorId}`,
        anchorId: node.placeholderAnchorId!,
        relation: node.placeholderRelation!,
        x,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
      });
      return;
    }

    if (!node.id) return;
    const person = nodeMap.get(node.id);
    if (!person || placedIds.has(node.id)) return;
    placedIds.add(node.id);

    nodes.push({
      id: node.id,
      node: person,
      x,
      y,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: gen,
      isFocal: gen === 0,
      isDirectLineage: true,
    });

    const nextGen = gen + 1;
    const nextX = nextGen * (CARD_WIDTH + H_GAP);

    if (node.father && node.mother) {
      const fCenter = centerY + (node.hFatherRelCenterY! - node.hNodeRelCenterY);
      const mCenter = centerY + (node.hMotherRelCenterY! - node.hNodeRelCenterY);

      assignHorizontalCoords(node.father, fCenter, nextGen);
      assignHorizontalCoords(node.mother, mCenter, nextGen);

      const sx = x + CARD_WIDTH;
      const sy = y + CARD_HEIGHT / 2;

      // Father edge
      const fTargetId = node.father.id ?? `ph-father-${node.id}`;
      edges.push(
        createPositionedEdge({
          id: `ped-edge-${node.id}-${fTargetId}`,
          sourceId: node.id,
          targetId: fTargetId,
          kind: node.father.isPlaceholder ? 'placeholder' : 'parent_child',
          sourceX: sx,
          sourceY: sy,
          targetX: nextX,
          targetY: fCenter,
          pathD: buildHorizontalBezierPath(sx, sy, nextX, fCenter),
          isDirectLineage: !node.father.isPlaceholder,
        })
      );

      // Mother edge
      const mTargetId = node.mother.id ?? `ph-mother-${node.id}`;
      edges.push(
        createPositionedEdge({
          id: `ped-edge-${node.id}-${mTargetId}`,
          sourceId: node.id,
          targetId: mTargetId,
          kind: node.mother.isPlaceholder ? 'placeholder' : 'parent_child',
          sourceX: sx,
          sourceY: sy,
          targetX: nextX,
          targetY: mCenter,
          pathD: buildHorizontalBezierPath(sx, sy, nextX, mCenter),
          isDirectLineage: !node.mother.isPlaceholder,
        })
      );
    } else if (node.father) {
      const fCenter = centerY + (node.hFatherRelCenterY! - node.hNodeRelCenterY);
      assignHorizontalCoords(node.father, fCenter, nextGen);
      if (node.father.id) {
        const sx = x + CARD_WIDTH;
        const sy = y + CARD_HEIGHT / 2;
        edges.push(
          createPositionedEdge({
            id: `ped-edge-${node.id}-${node.father.id}`,
            sourceId: node.id,
            targetId: node.father.id,
            kind: 'parent_child',
            sourceX: sx,
            sourceY: sy,
            targetX: nextX,
            targetY: fCenter,
            pathD: buildHorizontalBezierPath(sx, sy, nextX, fCenter),
            isDirectLineage: true,
          })
        );
      }
    } else if (node.mother) {
      const mCenter = centerY + (node.hMotherRelCenterY! - node.hNodeRelCenterY);
      assignHorizontalCoords(node.mother, mCenter, nextGen);
      if (node.mother.id) {
        const sx = x + CARD_WIDTH;
        const sy = y + CARD_HEIGHT / 2;
        edges.push(
          createPositionedEdge({
            id: `ped-edge-${node.id}-${node.mother.id}`,
            sourceId: node.id,
            targetId: node.mother.id,
            kind: 'parent_child',
            sourceX: sx,
            sourceY: sy,
            targetX: nextX,
            targetY: mCenter,
            pathD: buildHorizontalBezierPath(sx, sy, nextX, mCenter),
            isDirectLineage: true,
          })
        );
      }
    }
  };

  assignHorizontalCoords(root, 0, 0);

  // Place focal person's spouse below focal person if present
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
    orientation: 'horizontal',
    focalId,
    nodes,
    placeholders,
    edges,
    bounds: computeLayoutBounds(nodes, placeholders),
  };
}
