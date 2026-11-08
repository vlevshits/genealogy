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
  FamilyUnion,
  H_GAP,
  PlaceholderSlot,
  PositionedEdge,
  PositionedNode,
  ROW_STEP,
  SPOUSE_GAP,
  TreeLayoutResult,
  V_GAP,
  createPositionedEdge,
} from './types';

interface DescendantUnionPlan {
  union: FamilyUnion;
  partnerId: string;
  childrenPlans: SubtreePlan[];
  width: number;
}

interface SubtreePlan {
  personId: string;
  spouseIds: string[];
  unions: DescendantUnionPlan[];
  singleParentChildren: SubtreePlan[];
  unitWidth: number;
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
    const placedSpouses: string[] = [];

    // Find up to 2 spouses from unions
    for (const u of rel?.unions ?? []) {
      const partner = u.partner1Id === personId ? u.partner2Id : u.partner1Id;
      if (partner && nodeMap.has(partner) && !visited.has(partner)) {
        if (!placedSpouses.includes(partner)) {
          placedSpouses.push(partner);
          visited.add(partner);
          if (placedSpouses.length === 2) break;
        }
      }
    }
    if (placedSpouses.length < 2 && rel) {
      for (const sid of rel.spouseIds) {
        if (nodeMap.has(sid) && !visited.has(sid) && !placedSpouses.includes(sid)) {
          placedSpouses.push(sid);
          visited.add(sid);
          if (placedSpouses.length === 2) break;
        }
      }
    }

    const unitWidth =
      placedSpouses.length === 2
        ? CARD_WIDTH * 3 + SPOUSE_GAP * 2
        : placedSpouses.length === 1
        ? CARD_WIDTH * 2 + SPOUSE_GAP
        : CARD_WIDTH;

    const unionPlans: DescendantUnionPlan[] = [];
    const singleParentPlans: SubtreePlan[] = [];

    if (gen < maxGen) {
      // 1. Build children for each placed spouse union
      for (const spId of placedSpouses) {
        const u = rel?.unions.find(
          (cand) =>
            (cand.partner1Id === personId && cand.partner2Id === spId) ||
            (cand.partner1Id === spId && cand.partner2Id === personId)
        ) ?? {
          familyId: `desc-synth-${personId}-${spId}`,
          partner1Id: personId,
          partner2Id: spId,
          childIds:
            relativesMap
              .get(spId)
              ?.childIds.filter((cid) => rel?.childIds.includes(cid)) ?? [],
          marriageDate: null,
          marriagePlace: null,
        };

        const uChildren: SubtreePlan[] = [];
        for (const cid of u.childIds) {
          if (!visited.has(cid) && nodeMap.has(cid)) {
            const childPlan = buildPlan(cid, gen + 1);
            if (childPlan) uChildren.push(childPlan);
          }
        }

        const uWidth =
          uChildren.length > 0
            ? uChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
              Math.max(0, uChildren.length - 1) * H_GAP
            : 0;

        unionPlans.push({
          union: u,
          partnerId: spId,
          childrenPlans: uChildren,
          width: uWidth,
        });
      }

      // 2. Build single-parent children
      const handledChildren = new Set<string>();
      for (const up of unionPlans) {
        for (const ch of up.childrenPlans) {
          handledChildren.add(ch.personId);
        }
      }
      for (const cid of rel?.childIds ?? []) {
        if (!handledChildren.has(cid) && !visited.has(cid) && nodeMap.has(cid)) {
          const childPlan = buildPlan(cid, gen + 1);
          if (childPlan) singleParentPlans.push(childPlan);
        }
      }
    }

    const spWidth =
      singleParentPlans.length > 0
        ? singleParentPlans.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, singleParentPlans.length - 1) * H_GAP
        : 0;

    const activePartWidths: number[] = [];
    if (placedSpouses.length === 2) {
      if (unionPlans[0]?.width > 0) activePartWidths.push(unionPlans[0].width);
      if (spWidth > 0) activePartWidths.push(spWidth);
      if (unionPlans[1]?.width > 0) activePartWidths.push(unionPlans[1].width);
    } else if (placedSpouses.length === 1) {
      if (spWidth > 0) activePartWidths.push(spWidth);
      if (unionPlans[0]?.width > 0) activePartWidths.push(unionPlans[0].width);
    } else {
      if (spWidth > 0) activePartWidths.push(spWidth);
    }

    const totalChildrenWidth =
      activePartWidths.length > 0
        ? activePartWidths.reduce((acc, w) => acc + w, 0) +
          Math.max(0, activePartWidths.length - 1) * H_GAP
        : 0;

    return {
      personId,
      spouseIds: placedSpouses,
      unions: unionPlans,
      singleParentChildren: singleParentPlans,
      unitWidth,
      subtreeWidth: Math.max(unitWidth, totalChildrenWidth),
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

  const placeSubtree = (
    plan: SubtreePlan,
    leftX: number,
    gen: number,
    siblingIndex = 0
  ): { anchorX: number; anchorY: number } => {
    const y = gen * ROW_STEP;
    const coupleStartX = leftX + (plan.subtreeWidth - plan.unitWidth) / 2;

    const personDto = nodeMap.get(plan.personId)!;
    let personX = coupleStartX;
    let personAnchorX = coupleStartX + CARD_WIDTH / 2;

    if (plan.spouseIds.length === 2) {
      // Symmetrical 2-spouse placement: [S1] ===== [Person] ===== [S2]
      const s1Id = plan.spouseIds[0];
      const s2Id = plan.spouseIds[1];
      const s1Dto = nodeMap.get(s1Id)!;
      const s2Dto = nodeMap.get(s2Id)!;

      const s1X = coupleStartX;
      personX = coupleStartX + CARD_WIDTH + SPOUSE_GAP;
      const s2X = personX + CARD_WIDTH + SPOUSE_GAP;
      personAnchorX = personX + CARD_WIDTH / 2;

      // S1 Card
      nodes.push({
        id: s1Id,
        node: s1Dto,
        x: s1X,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: false,
        isSpouseOfFocal: plan.personId === focalId,
        isDirectLineage: false,
      });

      // Person Card
      nodes.push({
        id: plan.personId,
        node: personDto,
        x: personX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: plan.personId === focalId,
        isDirectLineage: true,
      });

      // S2 Card
      nodes.push({
        id: s2Id,
        node: s2Dto,
        x: s2X,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: false,
        isSpouseOfFocal: plan.personId === focalId,
        isDirectLineage: false,
      });

      const spouseY = y + CARD_HEIGHT / 2;

      // Ochre Connector 1: [S1] ===== [Person]
      const u1Fid = plan.unions[0]?.union.familyId;
      edges.push(
        createPositionedEdge({
          id: `desc-spouse-${s1Id}-${plan.personId}`,
          sourceId: s1Id,
          targetId: plan.personId,
          familyId: u1Fid,
          kind: 'spouse',
          sourceX: s1X + CARD_WIDTH,
          sourceY: spouseY,
          targetX: personX,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(s1X + CARD_WIDTH, spouseY, personX, spouseY),
        })
      );

      // Ochre Connector 2: [Person] ===== [S2]
      const u2Fid = plan.unions[1]?.union.familyId;
      edges.push(
        createPositionedEdge({
          id: `desc-spouse-${plan.personId}-${s2Id}`,
          sourceId: plan.personId,
          targetId: s2Id,
          familyId: u2Fid,
          kind: 'spouse',
          sourceX: personX + CARD_WIDTH,
          sourceY: spouseY,
          targetX: s2X,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(personX + CARD_WIDTH, spouseY, s2X, spouseY),
        })
      );
    } else if (plan.spouseIds.length === 1) {
      // 1 Spouse: [Person] ===== [S1]
      const s1Id = plan.spouseIds[0];
      const s1Dto = nodeMap.get(s1Id)!;
      personX = coupleStartX;
      const s1X = coupleStartX + CARD_WIDTH + SPOUSE_GAP;
      personAnchorX = personX + CARD_WIDTH / 2;

      nodes.push({
        id: plan.personId,
        node: personDto,
        x: personX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: plan.personId === focalId,
        isDirectLineage: true,
      });

      nodes.push({
        id: s1Id,
        node: s1Dto,
        x: s1X,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: false,
        isSpouseOfFocal: plan.personId === focalId,
        isDirectLineage: false,
      });

      const spouseY = y + CARD_HEIGHT / 2;
      const u1Fid = plan.unions[0]?.union.familyId;
      edges.push(
        createPositionedEdge({
          id: `desc-spouse-${plan.personId}-${s1Id}`,
          sourceId: plan.personId,
          targetId: s1Id,
          familyId: u1Fid,
          kind: 'spouse',
          sourceX: personX + CARD_WIDTH,
          sourceY: spouseY,
          targetX: s1X,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(personX + CARD_WIDTH, spouseY, s1X, spouseY),
        })
      );
    } else {
      // 0 Spouses: [Person]
      personX = coupleStartX;
      personAnchorX = personX + CARD_WIDTH / 2;
      nodes.push({
        id: plan.personId,
        node: personDto,
        x: personX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: gen,
        isFocal: plan.personId === focalId,
        isDirectLineage: true,
      });
    }

    // Partitioned Children Distribution Buses
    interface DescRenderPartition {
      children: SubtreePlan[];
      stemSourceX: number;
      stemSourceY: number;
      familyId?: string;
      isSingleParent: boolean;
      width: number;
    }

    const partitions: DescRenderPartition[] = [];
    if (plan.spouseIds.length === 2) {
      if (plan.unions[0] && plan.unions[0].childrenPlans.length > 0) {
        partitions.push({
          children: plan.unions[0].childrenPlans,
          stemSourceX: coupleStartX + CARD_WIDTH + SPOUSE_GAP / 2,
          stemSourceY: y + CARD_HEIGHT / 2,
          familyId: plan.unions[0].union.familyId,
          isSingleParent: false,
          width: plan.unions[0].width,
        });
      }
      if (plan.singleParentChildren.length > 0) {
        const spWidth =
          plan.singleParentChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, plan.singleParentChildren.length - 1) * H_GAP;
        partitions.push({
          children: plan.singleParentChildren,
          stemSourceX: personX + CARD_WIDTH / 2,
          stemSourceY: y + CARD_HEIGHT,
          familyId: undefined,
          isSingleParent: true,
          width: spWidth,
        });
      }
      if (plan.unions[1] && plan.unions[1].childrenPlans.length > 0) {
        partitions.push({
          children: plan.unions[1].childrenPlans,
          stemSourceX: personX + CARD_WIDTH + SPOUSE_GAP / 2,
          stemSourceY: y + CARD_HEIGHT / 2,
          familyId: plan.unions[1].union.familyId,
          isSingleParent: false,
          width: plan.unions[1].width,
        });
      }
    } else if (plan.spouseIds.length === 1) {
      if (plan.singleParentChildren.length > 0) {
        const spWidth =
          plan.singleParentChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, plan.singleParentChildren.length - 1) * H_GAP;
        partitions.push({
          children: plan.singleParentChildren,
          stemSourceX: personX + CARD_WIDTH / 2,
          stemSourceY: y + CARD_HEIGHT,
          familyId: undefined,
          isSingleParent: true,
          width: spWidth,
        });
      }
      if (plan.unions[0] && plan.unions[0].childrenPlans.length > 0) {
        partitions.push({
          children: plan.unions[0].childrenPlans,
          stemSourceX: personX + CARD_WIDTH + SPOUSE_GAP / 2,
          stemSourceY: y + CARD_HEIGHT / 2,
          familyId: plan.unions[0].union.familyId,
          isSingleParent: false,
          width: plan.unions[0].width,
        });
      }
    } else {
      if (plan.singleParentChildren.length > 0) {
        const spWidth =
          plan.singleParentChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, plan.singleParentChildren.length - 1) * H_GAP;
        partitions.push({
          children: plan.singleParentChildren,
          stemSourceX: personX + CARD_WIDTH / 2,
          stemSourceY: y + CARD_HEIGHT,
          familyId: undefined,
          isSingleParent: true,
          width: spWidth,
        });
      }
    }

    if (partitions.length > 0) {
      const totalPartsW =
        partitions.reduce((acc, p) => acc + p.width, 0) +
        Math.max(0, partitions.length - 1) * H_GAP;
      let childCursorX = leftX + (plan.subtreeWidth - totalPartsW) / 2;

      for (let pIdx = 0; pIdx < partitions.length; pIdx++) {
        const part = partitions[pIdx];
        // Anti-collinear staggered runner lanes: >= 12px clearance between adjacent family buses
        const busStagger =
          (pIdx === 0 ? -8 : pIdx === 1 ? 8 : -16) +
          (siblingIndex % 2 === 0 ? -4 : 4);
        const yBus = y + CARD_HEIGHT + V_GAP / 2 + busStagger;

        for (let i = 0; i < part.children.length; i++) {
          const childPlan = part.children[i];
          const childAnchor = placeSubtree(childPlan, childCursorX, gen + 1, i);
          const effectiveSourceX = part.stemSourceX;

          edges.push(
            createPositionedEdge({
              id: `desc-edge-${plan.personId}-${childPlan.personId}`,
              sourceId: plan.personId,
              targetId: childPlan.personId,
              familyId: part.familyId,
              kind: 'parent_child',
              sourceX: effectiveSourceX,
              sourceY: part.stemSourceY,
              targetX: childAnchor.anchorX,
              targetY: childAnchor.anchorY,
              yBus,
              pathD: buildVerticalBusPath(
                effectiveSourceX,
                part.stemSourceY,
                childAnchor.anchorX,
                childAnchor.anchorY,
                yBus
              ),
              isDirectLineage: true,
            })
          );
          childCursorX += childPlan.subtreeWidth + H_GAP;
        }
      }
    }

    return {
      anchorX: personAnchorX,
      anchorY: y,
    };
  };

  placeSubtree(rootPlan, -rootPlan.subtreeWidth / 2, 0);

  return {
    mode: 'descendant',
    focalId,
    nodes,
    placeholders,
    edges,
    bounds: computeLayoutBounds(nodes, placeholders),
  };
}
