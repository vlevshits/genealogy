import {
  buildSpouseConnectorPath,
  buildVerticalBusPath,
} from "../curve-routing";
import {
  CARD_HEIGHT,
  CARD_WIDTH,
  FamilyUnion,
  H_GAP,
  PersonRelatives,
  PlaceholderSlot,
  PositionedEdge,
  PositionedNode,
  ROW_STEP,
  SPOUSE_GAP,
  V_GAP,
  createPositionedEdge,
} from "../types";
import { TreePersonNodeDto } from "../../../api/client";

export interface DownUnionBranch {
  union: FamilyUnion;
  partnerId: string;
  children: DownBranch[];
  width: number;
}

export interface DownBranch {
  personId: string;
  generation: number;
  isDirect: boolean;
  spouseIds: string[];
  unions: DownUnionBranch[];
  singleParentChildren: DownBranch[];
  unitWidth: number;
  subtreeWidth: number;
}

export interface HourglassContext {
  nodeMap: Map<string, TreePersonNodeDto>;
  relativesMap: Map<string, PersonRelatives>;
  focalId: string;
  maxGen: number;
  collateral: boolean;
  placedIds: Set<string>;
  nodes: PositionedNode[];
  placeholders: PlaceholderSlot[];
  edges: PositionedEdge[];
}

export function createDownBranchEngine(ctx: HourglassContext) {
  const { nodeMap, relativesMap, focalId, placedIds, nodes, edges } = ctx;
  const buildDownBranch = (
    personId: string,
    currentGen: number,
    isDirect: boolean,
    maxDepth: number
  ): DownBranch | null => {
    if (placedIds.has(personId) || !nodeMap.has(personId)) return null;
    placedIds.add(personId);

    const rel = relativesMap.get(personId);
    const placedSpouses: string[] = [];

    // Find spouses to place (up to 2 for symmetrical placement [S1, Person, S2])
    for (const u of rel?.unions ?? []) {
      const partner = u.partner1Id === personId ? u.partner2Id : u.partner1Id;
      if (partner && nodeMap.has(partner) && !placedIds.has(partner)) {
        if (!placedSpouses.includes(partner)) {
          placedSpouses.push(partner);
          placedIds.add(partner);
          if (placedSpouses.length === 2) break;
        }
      }
    }
    if (placedSpouses.length < 2 && rel) {
      for (const sid of rel.spouseIds) {
        if (nodeMap.has(sid) && !placedIds.has(sid) && !placedSpouses.includes(sid)) {
          placedSpouses.push(sid);
          placedIds.add(sid);
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

    const unionBranches: DownUnionBranch[] = [];
    const singleParentBranches: DownBranch[] = [];

    if (currentGen < maxDepth) {
      // 1. Build children for each placed spouse union
      for (const spId of placedSpouses) {
        const u = rel?.unions.find(
          (cand) =>
            (cand.partner1Id === personId && cand.partner2Id === spId) ||
            (cand.partner1Id === spId && cand.partner2Id === personId)
        ) ?? {
          familyId: `synth-${personId}-${spId}`,
          partner1Id: personId,
          partner2Id: spId,
          childIds:
            relativesMap
              .get(spId)
              ?.childIds.filter((cid) => rel?.childIds.includes(cid)) ?? [],
          marriageDate: null,
          marriagePlace: null,
        };

        const uChildren: DownBranch[] = [];
        for (const cid of u.childIds) {
          if (!placedIds.has(cid) && nodeMap.has(cid)) {
            const childDto = nodeMap.get(cid)!;
            const childIsDirect = Boolean(
              isDirect && childDto.is_direct_lineage !== false
            );
            const cp = buildDownBranch(cid, currentGen + 1, childIsDirect, maxDepth);
            if (cp) uChildren.push(cp);
          }
        }

        const uWidth =
          uChildren.length > 0
            ? uChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
              Math.max(0, uChildren.length - 1) * H_GAP
            : 0;

        unionBranches.push({
          union: u,
          partnerId: spId,
          children: uChildren,
          width: uWidth,
        });
      }

      // 2. Build single-parent children (strict child partition)
      const handledChildren = new Set<string>();
      for (const ub of unionBranches) {
        for (const ch of ub.children) {
          handledChildren.add(ch.personId);
        }
      }
      for (const cid of rel?.childIds ?? []) {
        if (!handledChildren.has(cid) && !placedIds.has(cid) && nodeMap.has(cid)) {
          const childDto = nodeMap.get(cid)!;
          const childIsDirect = Boolean(
            isDirect && childDto.is_direct_lineage !== false
          );
          const cp = buildDownBranch(cid, currentGen + 1, childIsDirect, maxDepth);
          if (cp) singleParentBranches.push(cp);
        }
      }
    }

    const spWidth =
      singleParentBranches.length > 0
        ? singleParentBranches.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, singleParentBranches.length - 1) * H_GAP
        : 0;

    const activePartWidths: number[] = [];
    if (placedSpouses.length === 2) {
      if (unionBranches[0]?.width > 0) activePartWidths.push(unionBranches[0].width);
      if (spWidth > 0) activePartWidths.push(spWidth);
      if (unionBranches[1]?.width > 0) activePartWidths.push(unionBranches[1].width);
    } else if (placedSpouses.length === 1) {
      if (spWidth > 0) activePartWidths.push(spWidth);
      if (unionBranches[0]?.width > 0) activePartWidths.push(unionBranches[0].width);
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
      generation: currentGen,
      isDirect,
      spouseIds: placedSpouses,
      unions: unionBranches,
      singleParentChildren: singleParentBranches,
      unitWidth,
      subtreeWidth: Math.max(unitWidth, totalChildrenWidth),
    };
  };

  const renderDownBranch = (
    branch: DownBranch,
    leftX: number,
    siblingIndex = 0
  ): { anchorX: number; anchorY: number } => {
    const y = branch.generation * ROW_STEP;
    const coupleStartX = leftX + (branch.subtreeWidth - branch.unitWidth) / 2;

    const pDto = nodeMap.get(branch.personId)!;
    let personX = coupleStartX;
    let personAnchorX = coupleStartX + CARD_WIDTH / 2;

    if (branch.spouseIds.length === 2) {
      // Symmetrical 2-spouse placement: [Spouse 1] ===== [Person] ===== [Spouse 2]
      const s1Id = branch.spouseIds[0];
      const s2Id = branch.spouseIds[1];
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
        generation: branch.generation,
        isFocal: false,
        isSpouseOfFocal: branch.personId === focalId,
        isDirectLineage: false,
      });

      // Person Card
      nodes.push({
        id: branch.personId,
        node: pDto,
        x: personX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: branch.generation,
        isFocal: branch.personId === focalId,
        isDirectLineage: branch.isDirect,
      });

      // S2 Card
      nodes.push({
        id: s2Id,
        node: s2Dto,
        x: s2X,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: branch.generation,
        isFocal: false,
        isSpouseOfFocal: branch.personId === focalId,
        isDirectLineage: false,
      });

      const spouseY = y + CARD_HEIGHT / 2;

      // Union 1 Ochre Connector: [S1] ===== [Person]
      const u1Fid = branch.unions[0]?.union.familyId;
      edges.push(
        createPositionedEdge({
          id: `hg-spouse-${s1Id}-${branch.personId}`,
          sourceId: s1Id,
          targetId: branch.personId,
          familyId: u1Fid,
          kind: 'spouse',
          sourceX: s1X + CARD_WIDTH,
          sourceY: spouseY,
          targetX: personX,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(s1X + CARD_WIDTH, spouseY, personX, spouseY),
          isDirectLineage: branch.isDirect,
        })
      );

      // Union 2 Ochre Connector: [Person] ===== [S2]
      const u2Fid = branch.unions[1]?.union.familyId;
      edges.push(
        createPositionedEdge({
          id: `hg-spouse-${branch.personId}-${s2Id}`,
          sourceId: branch.personId,
          targetId: s2Id,
          familyId: u2Fid,
          kind: 'spouse',
          sourceX: personX + CARD_WIDTH,
          sourceY: spouseY,
          targetX: s2X,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(personX + CARD_WIDTH, spouseY, s2X, spouseY),
          isDirectLineage: branch.isDirect,
        })
      );
    } else if (branch.spouseIds.length === 1) {
      // 1 Spouse: [Person] ===== [S1]
      const s1Id = branch.spouseIds[0];
      const s1Dto = nodeMap.get(s1Id)!;
      personX = coupleStartX;
      const s1X = coupleStartX + CARD_WIDTH + SPOUSE_GAP;
      personAnchorX = personX + CARD_WIDTH / 2;

      nodes.push({
        id: branch.personId,
        node: pDto,
        x: personX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: branch.generation,
        isFocal: branch.personId === focalId,
        isDirectLineage: branch.isDirect,
      });

      nodes.push({
        id: s1Id,
        node: s1Dto,
        x: s1X,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: branch.generation,
        isFocal: false,
        isSpouseOfFocal: branch.personId === focalId,
        isDirectLineage: false,
      });

      const spouseY = y + CARD_HEIGHT / 2;
      const u1Fid = branch.unions[0]?.union.familyId;
      edges.push(
        createPositionedEdge({
          id: `hg-spouse-${branch.personId}-${s1Id}`,
          sourceId: branch.personId,
          targetId: s1Id,
          familyId: u1Fid,
          kind: 'spouse',
          sourceX: personX + CARD_WIDTH,
          sourceY: spouseY,
          targetX: s1X,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(personX + CARD_WIDTH, spouseY, s1X, spouseY),
          isDirectLineage: branch.isDirect,
        })
      );
    } else {
      // 0 Spouses: [Person]
      personX = coupleStartX;
      personAnchorX = personX + CARD_WIDTH / 2;
      nodes.push({
        id: branch.personId,
        node: pDto,
        x: personX,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: branch.generation,
        isFocal: branch.personId === focalId,
        isDirectLineage: branch.isDirect,
      });
    }

    // -------------------------------------------------------------------------
    // Strict Child-to-Marriage Partitioning
    // -------------------------------------------------------------------------
    interface RenderPartition {
      children: DownBranch[];
      stemSourceX: number;
      stemSourceY: number;
      familyId?: string;
      isSingleParent: boolean;
      width: number;
    }

    const partitions: RenderPartition[] = [];
    if (branch.spouseIds.length === 2) {
      // Union 1 on left (hangs from Union 1 midpoint dot)
      if (branch.unions[0] && branch.unions[0].children.length > 0) {
        partitions.push({
          children: branch.unions[0].children,
          stemSourceX: coupleStartX + CARD_WIDTH + SPOUSE_GAP / 2,
          stemSourceY: y + CARD_HEIGHT / 2,
          familyId: branch.unions[0].union.familyId,
          isSingleParent: false,
          width: branch.unions[0].width,
        });
      }
      // Single-parent children hang from bottom-center of Person
      if (branch.singleParentChildren.length > 0) {
        const spWidth =
          branch.singleParentChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, branch.singleParentChildren.length - 1) * H_GAP;
        partitions.push({
          children: branch.singleParentChildren,
          stemSourceX: personX + CARD_WIDTH / 2,
          stemSourceY: y + CARD_HEIGHT,
          familyId: undefined,
          isSingleParent: true,
          width: spWidth,
        });
      }
      // Union 2 on right (hangs from Union 2 midpoint dot)
      if (branch.unions[1] && branch.unions[1].children.length > 0) {
        partitions.push({
          children: branch.unions[1].children,
          stemSourceX: personX + CARD_WIDTH + SPOUSE_GAP / 2,
          stemSourceY: y + CARD_HEIGHT / 2,
          familyId: branch.unions[1].union.familyId,
          isSingleParent: false,
          width: branch.unions[1].width,
        });
      }
    } else if (branch.spouseIds.length === 1) {
      if (branch.singleParentChildren.length > 0) {
        const spWidth =
          branch.singleParentChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, branch.singleParentChildren.length - 1) * H_GAP;
        partitions.push({
          children: branch.singleParentChildren,
          stemSourceX: personX + CARD_WIDTH / 2,
          stemSourceY: y + CARD_HEIGHT,
          familyId: undefined,
          isSingleParent: true,
          width: spWidth,
        });
      }
      if (branch.unions[0] && branch.unions[0].children.length > 0) {
        partitions.push({
          children: branch.unions[0].children,
          stemSourceX: personX + CARD_WIDTH + SPOUSE_GAP / 2,
          stemSourceY: y + CARD_HEIGHT / 2,
          familyId: branch.unions[0].union.familyId,
          isSingleParent: false,
          width: branch.unions[0].width,
        });
      }
    } else {
      if (branch.singleParentChildren.length > 0) {
        const spWidth =
          branch.singleParentChildren.reduce((acc, c) => acc + c.subtreeWidth, 0) +
          Math.max(0, branch.singleParentChildren.length - 1) * H_GAP;
        partitions.push({
          children: branch.singleParentChildren,
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
      let cursorX = leftX + (branch.subtreeWidth - totalPartsW) / 2;

      for (let pIdx = 0; pIdx < partitions.length; pIdx++) {
        const part = partitions[pIdx];
        // Anti-collinear staggered runner lanes: >= 12px clearance between adjacent family buses
        const busStagger =
          (pIdx === 0 ? -8 : pIdx === 1 ? 8 : -16) +
          (siblingIndex % 2 === 0 ? -4 : 4);
        const yBus = y + CARD_HEIGHT + V_GAP / 2 + busStagger;

        for (let chIdx = 0; chIdx < part.children.length; chIdx++) {
          const ch = part.children[chIdx];
          const chAnchor = renderDownBranch(ch, cursorX, chIdx);
          const effectiveSourceX = part.stemSourceX;

          edges.push(
            createPositionedEdge({
              id: `hg-edge-${branch.personId}-${ch.personId}`,
              sourceId: branch.personId,
              targetId: ch.personId,
              familyId: part.familyId,
              kind: 'parent_child',
              sourceX: effectiveSourceX,
              sourceY: part.stemSourceY,
              targetX: chAnchor.anchorX,
              targetY: chAnchor.anchorY,
              yBus,
              pathD: buildVerticalBusPath(
                effectiveSourceX,
                part.stemSourceY,
                chAnchor.anchorX,
                chAnchor.anchorY,
                yBus
              ),
              isDirectLineage: branch.isDirect && ch.isDirect,
            })
          );

          cursorX += ch.subtreeWidth + H_GAP;
        }
      }
    }

    return {
      anchorX: personAnchorX,
      anchorY: y,
    };
  };

  return { buildDownBranch, renderDownBranch };
}
