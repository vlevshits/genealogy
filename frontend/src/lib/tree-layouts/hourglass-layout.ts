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

interface DownUnionBranch {
  union: FamilyUnion;
  partnerId: string;
  children: DownBranch[];
  width: number;
}

interface DownBranch {
  personId: string;
  generation: number;
  isDirect: boolean;
  spouseIds: string[];
  unions: DownUnionBranch[];
  singleParentChildren: DownBranch[];
  unitWidth: number;
  subtreeWidth: number;
}

export function computeHourglassLayout(
  data: TreeViewResponse,
  maxGenerationsOverride?: number,
  collateralOverride?: boolean
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
  const collateral = collateralOverride ?? data.collateral ?? true;
  const placedIds = new Set<string>();

  // ---------------------------------------------------------------------------
  // 1. Recursive Downward Branch Builder & Renderer
  // ---------------------------------------------------------------------------
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

  // ---------------------------------------------------------------------------
  // 2. Identify Relatives around Focal Person
  // ---------------------------------------------------------------------------
  const focalRel = relativesMap.get(focalId);

  // Parents
  let fatherId = focalRel?.fatherId ?? null;
  let motherId = focalRel?.motherId ?? null;
  if (fatherId && motherId && fatherId === motherId) {
    const pNode = nodeMap.get(fatherId);
    if (pNode?.sex === 'F') {
      fatherId = null;
    } else {
      motherId = null;
    }
  }

  const fatherDto = fatherId ? nodeMap.get(fatherId) : undefined;
  const hasFather = Boolean(fatherId && fatherDto);

  const motherDto = motherId ? nodeMap.get(motherId) : undefined;
  const hasMother = Boolean(motherId && motherDto);

  // In Wide Hourglass mode, detect ancestor remarriages on Gen -1
  let patExtraSpouseId: string | null = null;
  let patExtraUnion: FamilyUnion | null = null;
  let matExtraSpouseId: string | null = null;
  let matExtraUnion: FamilyUnion | null = null;

  if (collateral && fatherId) {
    const fRel = relativesMap.get(fatherId);
    const otherUnion = fRel?.unions.find(
      (u) =>
        (u.partner1Id === fatherId &&
          u.partner2Id &&
          u.partner2Id !== motherId &&
          nodeMap.has(u.partner2Id)) ||
        (u.partner2Id === fatherId &&
          u.partner1Id &&
          u.partner1Id !== motherId &&
          nodeMap.has(u.partner1Id))
    );
    if (otherUnion) {
      const sp =
        otherUnion.partner1Id === fatherId
          ? otherUnion.partner2Id
          : otherUnion.partner1Id;
      if (sp && !placedIds.has(sp)) {
        patExtraSpouseId = sp;
        patExtraUnion = otherUnion;
      }
    }
  }

  if (collateral && motherId) {
    const mRel = relativesMap.get(motherId);
    const otherUnion = mRel?.unions.find(
      (u) =>
        (u.partner1Id === motherId &&
          u.partner2Id &&
          u.partner2Id !== fatherId &&
          nodeMap.has(u.partner2Id)) ||
        (u.partner2Id === motherId &&
          u.partner1Id &&
          u.partner1Id !== fatherId &&
          nodeMap.has(u.partner1Id))
    );
    if (otherUnion) {
      const sp =
        otherUnion.partner1Id === motherId
          ? otherUnion.partner2Id
          : otherUnion.partner1Id;
      if (sp && !placedIds.has(sp)) {
        matExtraSpouseId = sp;
        matExtraUnion = otherUnion;
      }
    }
  }

  // Siblings of Focal Person (Partitioned into full, paternal-half, maternal-half)
  const fullSiblingIds: string[] = [];
  const patHalfSiblingIds: string[] = [];
  const matHalfSiblingIds: string[] = [];

  if (collateral) {
    const fRel = fatherId ? relativesMap.get(fatherId) : null;
    const mRel = motherId ? relativesMap.get(motherId) : null;

    if (hasFather && hasMother) {
      for (const cid of fRel?.childIds ?? []) {
        if (cid === focalId || !nodeMap.has(cid)) continue;
        if (mRel?.childIds.includes(cid)) {
          if (!fullSiblingIds.includes(cid)) fullSiblingIds.push(cid);
        } else {
          if (!patHalfSiblingIds.includes(cid)) patHalfSiblingIds.push(cid);
        }
      }
      for (const cid of mRel?.childIds ?? []) {
        if (cid === focalId || !nodeMap.has(cid)) continue;
        if (!fRel?.childIds.includes(cid)) {
          if (!matHalfSiblingIds.includes(cid)) matHalfSiblingIds.push(cid);
        }
      }
    } else if (hasFather) {
      for (const cid of fRel?.childIds ?? []) {
        if (cid !== focalId && nodeMap.has(cid) && !fullSiblingIds.includes(cid)) {
          fullSiblingIds.push(cid);
        }
      }
    } else if (hasMother) {
      for (const cid of mRel?.childIds ?? []) {
        if (cid !== focalId && nodeMap.has(cid) && !fullSiblingIds.includes(cid)) {
          fullSiblingIds.push(cid);
        }
      }
    }
  }

  // ---------------------------------------------------------------------------
  // 3. Build Focal and Sibling DownBranches (Gen 0 downwards)
  // ---------------------------------------------------------------------------
  const focalBranch = buildDownBranch(focalId, 0, true, maxGen - 1)!;

  const patHalfBranches: DownBranch[] = [];
  for (const sid of patHalfSiblingIds) {
    const sb = buildDownBranch(sid, 0, false, maxGen - 1);
    if (sb) patHalfBranches.push(sb);
  }

  const fullSibBranches: DownBranch[] = [];
  for (const sid of fullSiblingIds) {
    const sb = buildDownBranch(sid, 0, false, maxGen - 1);
    if (sb) fullSibBranches.push(sb);
  }

  const matHalfBranches: DownBranch[] = [];
  for (const sid of matHalfSiblingIds) {
    const sb = buildDownBranch(sid, 0, false, maxGen - 1);
    if (sb) matHalfBranches.push(sb);
  }

  const allSiblingBranches: DownBranch[] = [
    ...patHalfBranches,
    ...fullSibBranches,
    ...matHalfBranches,
  ];

  // Split full siblings evenly around focal branch
  const halfFull = Math.floor(fullSibBranches.length / 2);
  const leftFullSibs = fullSibBranches.slice(0, halfFull);
  const rightFullSibs = fullSibBranches.slice(halfFull);

  const patHalfW =
    patHalfBranches.reduce((sum, b) => sum + b.subtreeWidth, 0) +
    Math.max(0, patHalfBranches.length - 1) * H_GAP;
  const leftFullW =
    leftFullSibs.reduce((sum, b) => sum + b.subtreeWidth, 0) +
    Math.max(0, leftFullSibs.length - 1) * H_GAP;
  const rightFullW =
    rightFullSibs.reduce((sum, b) => sum + b.subtreeWidth, 0) +
    Math.max(0, rightFullSibs.length - 1) * H_GAP;
  const matHalfW =
    matHalfBranches.reduce((sum, b) => sum + b.subtreeWidth, 0) +
    Math.max(0, matHalfBranches.length - 1) * H_GAP;

  const centerGroupW =
    (patHalfW > 0 ? patHalfW + H_GAP : 0) +
    (leftFullW > 0 ? leftFullW + H_GAP : 0) +
    focalBranch.subtreeWidth +
    (rightFullW > 0 ? H_GAP + rightFullW : 0) +
    (matHalfW > 0 ? H_GAP + matHalfW : 0);

  const centerStartX = -centerGroupW / 2;
  let currentCursor = centerStartX;

  // 1. Render Paternal Half-siblings (leftmost)
  for (let i = 0; i < patHalfBranches.length; i++) {
    const b = patHalfBranches[i];
    renderDownBranch(b, currentCursor, i);
    currentCursor += b.subtreeWidth + H_GAP;
  }

  // 2. Render Left Full Siblings
  for (let i = 0; i < leftFullSibs.length; i++) {
    const b = leftFullSibs[i];
    renderDownBranch(b, currentCursor, patHalfBranches.length + i);
    currentCursor += b.subtreeWidth + H_GAP;
  }

  // 3. Render Focal Branch
  const focalStartX = currentCursor;
  renderDownBranch(
    focalBranch,
    focalStartX,
    patHalfBranches.length + leftFullSibs.length
  );
  currentCursor +=
    focalBranch.subtreeWidth +
    (rightFullW > 0 || matHalfW > 0 ? H_GAP : 0);

  // 4. Render Right Full Siblings
  for (let i = 0; i < rightFullSibs.length; i++) {
    const b = rightFullSibs[i];
    renderDownBranch(
      b,
      currentCursor,
      patHalfBranches.length + leftFullSibs.length + 1 + i
    );
    currentCursor += b.subtreeWidth + (matHalfW > 0 ? H_GAP : 0);
  }

  // 5. Render Maternal Half-siblings (rightmost)
  for (let i = 0; i < matHalfBranches.length; i++) {
    const b = matHalfBranches[i];
    renderDownBranch(
      b,
      currentCursor,
      patHalfBranches.length + leftFullSibs.length + 1 + rightFullSibs.length + i
    );
    currentCursor += b.subtreeWidth + H_GAP;
  }

  // Focal card position
  const focalNode = nodes.find((n) => n.id === focalId)!;
  const focalCenterX = focalNode.x + CARD_WIDTH / 2;

  // ---------------------------------------------------------------------------
  // 4. Place Parents & Ancestor Remarriages at Gen -1
  // ---------------------------------------------------------------------------
  const parentGen = 1;
  const parentY = -parentGen * ROW_STEP; // -180px
  const parentBusY = parentY + CARD_HEIGHT + V_GAP / 2;
  let parentUnionCenterX = focalCenterX;
  let fatherPlacedX: number | null = null;
  let motherPlacedX: number | null = null;
  let patExtraPlacedX: number | null = null;
  let matExtraPlacedX: number | null = null;

  if (hasFather && hasMother) {
    placedIds.add(fatherId!);
    placedIds.add(motherId!);

    const fatherX = focalCenterX - CARD_WIDTH - SPOUSE_GAP / 2;
    const motherX = focalCenterX + SPOUSE_GAP / 2;
    fatherPlacedX = fatherX;
    motherPlacedX = motherX;

    // Ancestor remarriage: Father's extra spouse placed on left of father
    if (patExtraSpouseId && nodeMap.has(patExtraSpouseId)) {
      placedIds.add(patExtraSpouseId);
      const patExtraDto = nodeMap.get(patExtraSpouseId)!;
      patExtraPlacedX = fatherX - CARD_WIDTH - SPOUSE_GAP;

      nodes.push({
        id: patExtraSpouseId,
        node: patExtraDto,
        x: patExtraPlacedX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -parentGen,
        isFocal: false,
        isDirectLineage: false,
      });

      const spY = parentY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `hg-anc-spouse-${patExtraSpouseId}-${fatherId}`,
          sourceId: patExtraSpouseId,
          targetId: fatherId!,
          familyId: patExtraUnion?.familyId,
          kind: 'spouse',
          sourceX: patExtraPlacedX + CARD_WIDTH,
          sourceY: spY,
          targetX: fatherX,
          targetY: spY,
          pathD: buildSpouseConnectorPath(
            patExtraPlacedX + CARD_WIDTH,
            spY,
            fatherX,
            spY
          ),
          isDirectLineage: false,
        })
      );
    }

    // Father Card
    nodes.push({
      id: fatherId!,
      node: fatherDto!,
      x: fatherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
      isFocal: false,
      isDirectLineage: true,
    });

    // Mother Card
    nodes.push({
      id: motherId!,
      node: motherDto!,
      x: motherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
      isFocal: false,
      isDirectLineage: true,
    });

    // Ancestor remarriage: Mother's extra spouse placed on right of mother
    if (matExtraSpouseId && nodeMap.has(matExtraSpouseId)) {
      placedIds.add(matExtraSpouseId);
      const matExtraDto = nodeMap.get(matExtraSpouseId)!;
      matExtraPlacedX = motherX + CARD_WIDTH + SPOUSE_GAP;

      nodes.push({
        id: matExtraSpouseId,
        node: matExtraDto,
        x: matExtraPlacedX,
        y: parentY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -parentGen,
        isFocal: false,
        isSpouseOfFocal: false,
        isDirectLineage: false,
      });

      const spY = parentY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `hg-anc-spouse-${motherId}-${matExtraSpouseId}`,
          sourceId: motherId!,
          targetId: matExtraSpouseId,
          familyId: matExtraUnion?.familyId,
          kind: 'spouse',
          sourceX: motherX + CARD_WIDTH,
          sourceY: spY,
          targetX: matExtraPlacedX,
          targetY: spY,
          pathD: buildSpouseConnectorPath(
            motherX + CARD_WIDTH,
            spY,
            matExtraPlacedX,
            spY
          ),
          isDirectLineage: false,
        })
      );
    }

    // Horizontal spouse connector between father and mother
    const spouseY = parentY + CARD_HEIGHT / 2;
    edges.push(
      createPositionedEdge({
        id: `hg-anc-spouse-${fatherId}-${motherId}`,
        sourceId: fatherId!,
        targetId: motherId!,
        kind: 'spouse',
        sourceX: fatherX + CARD_WIDTH,
        sourceY: spouseY,
        targetX: motherX,
        targetY: spouseY,
        pathD: buildSpouseConnectorPath(
          fatherX + CARD_WIDTH,
          spouseY,
          motherX,
          spouseY
        ),
        isDirectLineage: true,
      })
    );

    parentUnionCenterX = (fatherX + CARD_WIDTH + motherX) / 2;

    // Single direct lineage stem dropping from center of couple down to focal child
    edges.push(
      createPositionedEdge({
        id: `hg-anc-${fatherId}-${focalId}`,
        sourceId: fatherId!,
        targetId: focalId,
        kind: 'parent_child',
        sourceX: parentUnionCenterX,
        sourceY: spouseY,
        targetX: focalCenterX,
        targetY: focalNode.y,
        pathD: buildVerticalBusPath(
          parentUnionCenterX,
          spouseY,
          focalCenterX,
          focalNode.y,
          parentBusY
        ),
        isDirectLineage: true,
      })
    );

    // Full sibling connections from parent union
    for (const sb of fullSibBranches) {
      const sNode = nodes.find((n) => n.id === sb.personId);
      if (sNode) {
        const sCenterX = sNode.x + CARD_WIDTH / 2;
        edges.push(
          createPositionedEdge({
            id: `hg-anc-sib-${fatherId}-${sb.personId}`,
            sourceId: fatherId!,
            targetId: sb.personId,
            kind: 'parent_child',
            sourceX: parentUnionCenterX,
            sourceY: spouseY,
            targetX: sCenterX,
            targetY: sNode.y,
            pathD: buildVerticalBusPath(
              parentUnionCenterX,
              spouseY,
              sCenterX,
              sNode.y,
              parentBusY
            ),
            isDirectLineage: false,
          })
        );
      }
    }

    // Paternal half-siblings connect to Father & extra spouse (or Father)
    if (patHalfBranches.length > 0) {
      const patMidX =
        patExtraPlacedX !== null
          ? (patExtraPlacedX + CARD_WIDTH + fatherX) / 2
          : fatherX + CARD_WIDTH / 2;
      const patSourceY =
        patExtraPlacedX !== null ? parentY + CARD_HEIGHT / 2 : parentY + CARD_HEIGHT;
      const patBusY = parentBusY - 12; // >= 12px clearance

      for (const sb of patHalfBranches) {
        const sNode = nodes.find((n) => n.id === sb.personId);
        if (sNode) {
          const sCenterX = sNode.x + CARD_WIDTH / 2;
          edges.push(
            createPositionedEdge({
              id: `hg-anc-pathalf-${fatherId}-${sb.personId}`,
              sourceId: fatherId!,
              targetId: sb.personId,
              familyId: patExtraUnion?.familyId,
              kind: 'parent_child',
              sourceX: patMidX,
              sourceY: patSourceY,
              targetX: sCenterX,
              targetY: sNode.y,
              yBus: patBusY,
              pathD: buildVerticalBusPath(
                patMidX,
                patSourceY,
                sCenterX,
                sNode.y,
                patBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
    }

    // Maternal half-siblings connect to Mother & extra spouse (or Mother)
    if (matHalfBranches.length > 0) {
      const matMidX =
        matExtraPlacedX !== null
          ? (motherX + CARD_WIDTH + matExtraPlacedX) / 2
          : motherX + CARD_WIDTH / 2;
      const matSourceY =
        matExtraPlacedX !== null ? parentY + CARD_HEIGHT / 2 : parentY + CARD_HEIGHT;
      const matBusY = parentBusY + 12; // >= 12px clearance

      for (const sb of matHalfBranches) {
        const sNode = nodes.find((n) => n.id === sb.personId);
        if (sNode) {
          const sCenterX = sNode.x + CARD_WIDTH / 2;
          edges.push(
            createPositionedEdge({
              id: `hg-anc-mathalf-${motherId}-${sb.personId}`,
              sourceId: motherId!,
              targetId: sb.personId,
              familyId: matExtraUnion?.familyId,
              kind: 'parent_child',
              sourceX: matMidX,
              sourceY: matSourceY,
              targetX: sCenterX,
              targetY: sNode.y,
              yBus: matBusY,
              pathD: buildVerticalBusPath(
                matMidX,
                matSourceY,
                sCenterX,
                sNode.y,
                matBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
    }
  } else if (hasFather) {
    placedIds.add(fatherId!);
    const fatherX = focalCenterX - CARD_WIDTH / 2;
    fatherPlacedX = fatherX;

    nodes.push({
      id: fatherId!,
      node: fatherDto!,
      x: fatherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
      isFocal: false,
      isDirectLineage: true,
    });

    const singleFatherStemX = fatherX + CARD_WIDTH / 2;

    edges.push(
      createPositionedEdge({
        id: `hg-anc-${fatherId}-${focalId}`,
        sourceId: fatherId!,
        targetId: focalId,
        kind: 'parent_child',
        sourceX: singleFatherStemX,
        sourceY: parentY + CARD_HEIGHT,
        targetX: focalCenterX,
        targetY: focalNode.y,
        pathD: buildVerticalBusPath(
          singleFatherStemX,
          parentY + CARD_HEIGHT,
          focalCenterX,
          focalNode.y,
          parentBusY
        ),
        isDirectLineage: true,
      })
    );

    // Sibling connections from single father
    for (const sb of allSiblingBranches) {
      const sNode = nodes.find((n) => n.id === sb.personId);
      if (sNode) {
        const sCenterX = sNode.x + CARD_WIDTH / 2;
        edges.push(
          createPositionedEdge({
            id: `hg-anc-sib-${fatherId}-${sb.personId}`,
            sourceId: fatherId!,
            targetId: sb.personId,
            kind: 'parent_child',
            sourceX: singleFatherStemX,
            sourceY: parentY + CARD_HEIGHT,
            targetX: sCenterX,
            targetY: sNode.y,
            pathD: buildVerticalBusPath(
              singleFatherStemX,
              parentY + CARD_HEIGHT,
              sCenterX,
              sNode.y,
              parentBusY
            ),
            isDirectLineage: false,
          })
        );
      }
    }

    // Mother placeholder
    const phMotherId = `hg-ph-mother-${focalId}`;
    const phMotherX = fatherX + CARD_WIDTH + SPOUSE_GAP;
    placeholders.push({
      id: phMotherId,
      anchorId: focalId,
      relation: 'mother',
      x: phMotherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
    });
    edges.push(
      createPositionedEdge({
        id: `hg-ph-edge-${phMotherId}`,
        sourceId: phMotherId,
        targetId: focalId,
        kind: 'placeholder',
        sourceX: phMotherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: focalCenterX,
        targetY: focalNode.y,
        pathD: buildVerticalBusPath(
          phMotherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          focalCenterX,
          focalNode.y,
          parentBusY
        ),
      })
    );
  } else if (hasMother) {
    placedIds.add(motherId!);
    const motherX = focalCenterX - CARD_WIDTH / 2;
    motherPlacedX = motherX;

    nodes.push({
      id: motherId!,
      node: motherDto!,
      x: motherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
      isFocal: false,
      isDirectLineage: true,
    });

    const singleMotherStemX = motherX + CARD_WIDTH / 2;

    edges.push(
      createPositionedEdge({
        id: `hg-anc-${motherId}-${focalId}`,
        sourceId: motherId!,
        targetId: focalId,
        kind: 'parent_child',
        sourceX: singleMotherStemX,
        sourceY: parentY + CARD_HEIGHT,
        targetX: focalCenterX,
        targetY: focalNode.y,
        pathD: buildVerticalBusPath(
          singleMotherStemX,
          parentY + CARD_HEIGHT,
          focalCenterX,
          focalNode.y,
          parentBusY
        ),
        isDirectLineage: true,
      })
    );

    // Sibling connections from single mother
    for (const sb of allSiblingBranches) {
      const sNode = nodes.find((n) => n.id === sb.personId);
      if (sNode) {
        const sCenterX = sNode.x + CARD_WIDTH / 2;
        edges.push(
          createPositionedEdge({
            id: `hg-anc-sib-${motherId}-${sb.personId}`,
            sourceId: motherId!,
            targetId: sb.personId,
            kind: 'parent_child',
            sourceX: singleMotherStemX,
            sourceY: parentY + CARD_HEIGHT,
            targetX: sCenterX,
            targetY: sNode.y,
            pathD: buildVerticalBusPath(
              singleMotherStemX,
              parentY + CARD_HEIGHT,
              sCenterX,
              sNode.y,
              parentBusY
            ),
            isDirectLineage: false,
          })
        );
      }
    }

    // Father placeholder
    const phFatherId = `hg-ph-father-${focalId}`;
    const phFatherX = motherX - CARD_WIDTH - SPOUSE_GAP;
    placeholders.push({
      id: phFatherId,
      anchorId: focalId,
      relation: 'father',
      x: phFatherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
    });
    edges.push(
      createPositionedEdge({
        id: `hg-ph-edge-${phFatherId}`,
        sourceId: phFatherId,
        targetId: focalId,
        kind: 'placeholder',
        sourceX: phFatherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: focalCenterX,
        targetY: focalNode.y,
        pathD: buildVerticalBusPath(
          phFatherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          focalCenterX,
          focalNode.y,
          parentBusY
        ),
      })
    );
  } else {
    // Both parents missing: add placeholders for both
    const phFatherId = `hg-ph-father-${focalId}`;
    const phFatherX = focalCenterX - CARD_WIDTH - SPOUSE_GAP / 2;
    placeholders.push({
      id: phFatherId,
      anchorId: focalId,
      relation: 'father',
      x: phFatherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
    });
    edges.push(
      createPositionedEdge({
        id: `hg-ph-edge-${phFatherId}`,
        sourceId: phFatherId,
        targetId: focalId,
        kind: 'placeholder',
        sourceX: phFatherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: focalCenterX,
        targetY: focalNode.y,
        pathD: buildVerticalBusPath(
          phFatherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          focalCenterX,
          focalNode.y,
          parentBusY
        ),
      })
    );

    const phMotherId = `hg-ph-mother-${focalId}`;
    const phMotherX = focalCenterX + SPOUSE_GAP / 2;
    placeholders.push({
      id: phMotherId,
      anchorId: focalId,
      relation: 'mother',
      x: phMotherX,
      y: parentY,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
      generation: -parentGen,
    });
    edges.push(
      createPositionedEdge({
        id: `hg-ph-edge-${phMotherId}`,
        sourceId: phMotherId,
        targetId: focalId,
        kind: 'placeholder',
        sourceX: phMotherX + CARD_WIDTH / 2,
        sourceY: parentY + CARD_HEIGHT,
        targetX: focalCenterX,
        targetY: focalNode.y,
        pathD: buildVerticalBusPath(
          phMotherX + CARD_WIDTH / 2,
          parentY + CARD_HEIGHT,
          focalCenterX,
          focalNode.y,
          parentBusY
        ),
      })
    );
  }

  // ---------------------------------------------------------------------------
  // 5. Paternal & Maternal Ancestors and Collateral (Grandparents & Aunts/Uncles)
  // ---------------------------------------------------------------------------
  const ancestorGenLimit = Math.min(3, Math.max(1, maxGen - 1));

  // 5A. Paternal Side (Grandparents + Aunts/Uncles at Gen -2 / Gen -1)
  if (ancestorGenLimit >= 2 && fatherId && fatherPlacedX !== null) {
    const fRel = relativesMap.get(fatherId);
    let patGfId = fRel?.fatherId ?? null;
    let patGmId = fRel?.motherId ?? null;
    if (patGfId && patGmId && patGfId === patGmId) {
      if (nodeMap.get(patGfId)?.sex === 'F') patGfId = null;
      else patGmId = null;
    }

    const patGfDto = patGfId ? nodeMap.get(patGfId) : undefined;
    const patGmDto = patGmId ? nodeMap.get(patGmId) : undefined;
    const hasPatGf = Boolean(patGfId && patGfDto && !placedIds.has(patGfId));
    const hasPatGm = Boolean(patGmId && patGmDto && !placedIds.has(patGmId));

    // Paternal Aunts and Uncles (children of paternal grandparents, excluding father)
    const patAuntIds: string[] = [];
    if (collateral) {
      if (patGfId) {
        for (const cid of relativesMap.get(patGfId)?.childIds ?? []) {
          if (cid !== fatherId && !patAuntIds.includes(cid) && nodeMap.has(cid)) {
            patAuntIds.push(cid);
          }
        }
      }
      if (patGmId) {
        for (const cid of relativesMap.get(patGmId)?.childIds ?? []) {
          if (cid !== fatherId && !patAuntIds.includes(cid) && nodeMap.has(cid)) {
            patAuntIds.push(cid);
          }
        }
      }
    }

    // Place paternal aunts to the left of the center group
    let patAuntsW = 0;
    let patAuntBranches: DownBranch[] = [];
    if (patAuntIds.length > 0) {
      const probeBranches: DownBranch[] = [];
      for (const aid of patAuntIds) {
        // compute width probe
        const b = buildDownBranch(aid, -1, false, maxGen - 2);
        if (b) probeBranches.push(b);
      }
      patAuntsW =
        probeBranches.reduce((sum, b) => sum + b.subtreeWidth, 0) +
        Math.max(0, probeBranches.length - 1) * H_GAP;
      // Start placed to the left of both centerStartX and fatherPlacedX
      const patAuntsStartX = Math.min(centerStartX, fatherPlacedX) - H_GAP - patAuntsW;
      let cur = patAuntsStartX;
      for (const b of probeBranches) {
        renderDownBranch(b, cur);
        cur += b.subtreeWidth + H_GAP;
      }
      patAuntBranches = probeBranches;
    }

    // Place Paternal Grandparents at Gen -2 (bus staggered by -6px)
    const gpY = -2 * ROW_STEP;
    const gpBusY = gpY + CARD_HEIGHT + V_GAP / 2 - 6;
    const patChildLeftX =
      patAuntsW > 0
        ? Math.min(centerStartX, fatherPlacedX) - H_GAP - patAuntsW
        : fatherPlacedX;
    const patChildRightX = fatherPlacedX + CARD_WIDTH;
    let patSpanCenterX = (patChildLeftX + patChildRightX) / 2;

    // Enforce half-plane clearance: paternal ancestors must remain strictly in left half-plane
    const patGrandparentWidth =
      hasPatGf && hasPatGm
        ? CARD_WIDTH * 2 + SPOUSE_GAP
        : CARD_WIDTH;
    const patCoupleHalfWidth = patGrandparentWidth / 2;
    const patMaxSpanCenterX = focalCenterX - H_GAP / 2 - patCoupleHalfWidth;
    patSpanCenterX = Math.min(patSpanCenterX, patMaxSpanCenterX);

    if (hasPatGf && hasPatGm) {
      placedIds.add(patGfId!);
      placedIds.add(patGmId!);

      const gfX = patSpanCenterX - CARD_WIDTH - SPOUSE_GAP / 2;
      const gmX = patSpanCenterX + SPOUSE_GAP / 2;

      nodes.push({
        id: patGfId!,
        node: patGfDto!,
        x: gfX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });

      nodes.push({
        id: patGmId!,
        node: patGmDto!,
        x: gmX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });

      const spouseY = gpY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `hg-anc-spouse-${patGfId}-${patGmId}`,
          sourceId: patGfId!,
          targetId: patGmId!,
          kind: 'spouse',
          sourceX: gfX + CARD_WIDTH,
          sourceY: spouseY,
          targetX: gmX,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(gfX + CARD_WIDTH, spouseY, gmX, spouseY),
          isDirectLineage: true,
        })
      );

      const gpUnionCenterX = (gfX + CARD_WIDTH + gmX) / 2;

      // Stem to Father
      edges.push(
        createPositionedEdge({
          id: `hg-anc-${patGfId}-${fatherId}`,
          sourceId: patGfId!,
          targetId: fatherId,
          kind: 'parent_child',
          sourceX: gpUnionCenterX,
          sourceY: spouseY,
          targetX: fatherPlacedX + CARD_WIDTH / 2,
          targetY: parentY,
          yBus: gpBusY,
          pathD: buildVerticalBusPath(
            gpUnionCenterX,
            spouseY,
            fatherPlacedX + CARD_WIDTH / 2,
            parentY,
            gpBusY
          ),
          isDirectLineage: true,
        })
      );

      // Stems to Paternal Aunts/Uncles
      for (const ab of patAuntBranches) {
        const aNode = nodes.find((n) => n.id === ab.personId);
        if (aNode) {
          edges.push(
            createPositionedEdge({
              id: `hg-anc-${patGfId}-${ab.personId}`,
              sourceId: patGfId!,
              targetId: ab.personId,
              kind: 'parent_child',
              sourceX: gpUnionCenterX,
              sourceY: spouseY,
              targetX: aNode.x + CARD_WIDTH / 2,
              targetY: aNode.y,
              yBus: gpBusY,
              pathD: buildVerticalBusPath(
                gpUnionCenterX,
                spouseY,
                aNode.x + CARD_WIDTH / 2,
                aNode.y,
                gpBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
    } else if (hasPatGf) {
      placedIds.add(patGfId!);
      const gfX = patSpanCenterX - CARD_WIDTH / 2;
      nodes.push({
        id: patGfId!,
        node: patGfDto!,
        x: gfX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });
      const singlePatGfStemX = gfX + CARD_WIDTH / 2 + 12;

      edges.push(
        createPositionedEdge({
          id: `hg-anc-${patGfId}-${fatherId}`,
          sourceId: patGfId!,
          targetId: fatherId,
          kind: 'parent_child',
          sourceX: singlePatGfStemX,
          sourceY: gpY + CARD_HEIGHT,
          targetX: fatherPlacedX + CARD_WIDTH / 2,
          targetY: parentY,
          yBus: gpBusY,
          pathD: buildVerticalBusPath(
            singlePatGfStemX,
            gpY + CARD_HEIGHT,
            fatherPlacedX + CARD_WIDTH / 2,
            parentY,
            gpBusY
          ),
          isDirectLineage: true,
        })
      );

      for (const ab of patAuntBranches) {
        const aNode = nodes.find((n) => n.id === ab.personId);
        if (aNode) {
          edges.push(
            createPositionedEdge({
              id: `hg-anc-${patGfId}-${ab.personId}`,
              sourceId: patGfId!,
              targetId: ab.personId,
              kind: 'parent_child',
              sourceX: singlePatGfStemX,
              sourceY: gpY + CARD_HEIGHT,
              targetX: aNode.x + CARD_WIDTH / 2,
              targetY: aNode.y,
              yBus: gpBusY,
              pathD: buildVerticalBusPath(
                singlePatGfStemX,
                gpY + CARD_HEIGHT,
                aNode.x + CARD_WIDTH / 2,
                aNode.y,
                gpBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
    } else if (hasPatGm) {
      placedIds.add(patGmId!);
      const gmX = patSpanCenterX - CARD_WIDTH / 2;
      nodes.push({
        id: patGmId!,
        node: patGmDto!,
        x: gmX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });
      const singlePatGmStemX = gmX + CARD_WIDTH / 2 - 12;

      edges.push(
        createPositionedEdge({
          id: `hg-anc-${patGmId}-${fatherId}`,
          sourceId: patGmId!,
          targetId: fatherId,
          kind: 'parent_child',
          sourceX: singlePatGmStemX,
          sourceY: gpY + CARD_HEIGHT,
          targetX: fatherPlacedX + CARD_WIDTH / 2,
          targetY: parentY,
          yBus: gpBusY,
          pathD: buildVerticalBusPath(
            singlePatGmStemX,
            gpY + CARD_HEIGHT,
            fatherPlacedX + CARD_WIDTH / 2,
            parentY,
            gpBusY
          ),
          isDirectLineage: true,
        })
      );

      for (const ab of patAuntBranches) {
        const aNode = nodes.find((n) => n.id === ab.personId);
        if (aNode) {
          edges.push(
            createPositionedEdge({
              id: `hg-anc-${patGmId}-${ab.personId}`,
              sourceId: patGmId!,
              targetId: ab.personId,
              kind: 'parent_child',
              sourceX: singlePatGmStemX,
              sourceY: gpY + CARD_HEIGHT,
              targetX: aNode.x + CARD_WIDTH / 2,
              targetY: aNode.y,
              yBus: gpBusY,
              pathD: buildVerticalBusPath(
                singlePatGmStemX,
                gpY + CARD_HEIGHT,
                aNode.x + CARD_WIDTH / 2,
                aNode.y,
                gpBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
    }
  }

  // 5B. Maternal Side (Grandparents + Aunts/Uncles at Gen -2 / Gen -1)
  if (ancestorGenLimit >= 2 && motherId && motherPlacedX !== null) {
    const mRel = relativesMap.get(motherId);
    let matGfId = mRel?.fatherId ?? null;
    let matGmId = mRel?.motherId ?? null;
    if (matGfId && matGmId && matGfId === matGmId) {
      if (nodeMap.get(matGfId)?.sex === 'F') matGfId = null;
      else matGmId = null;
    }

    const matGfDto = matGfId ? nodeMap.get(matGfId) : undefined;
    const matGmDto = matGmId ? nodeMap.get(matGmId) : undefined;
    const hasMatGf = Boolean(matGfId && matGfDto && !placedIds.has(matGfId));
    const hasMatGm = Boolean(matGmId && matGmDto && !placedIds.has(matGmId));

    // Maternal Aunts and Uncles (children of maternal grandparents, excluding mother)
    const matAuntIds: string[] = [];
    if (collateral) {
      if (matGfId) {
        for (const cid of relativesMap.get(matGfId)?.childIds ?? []) {
          if (cid !== motherId && !matAuntIds.includes(cid) && nodeMap.has(cid)) {
            matAuntIds.push(cid);
          }
        }
      }
      if (matGmId) {
        for (const cid of relativesMap.get(matGmId)?.childIds ?? []) {
          if (cid !== motherId && !matAuntIds.includes(cid) && nodeMap.has(cid)) {
            matAuntIds.push(cid);
          }
        }
      }
    }

    // Place maternal aunts to the right of centerEndX
    let matAuntsW = 0;
    let matAuntBranches: DownBranch[] = [];
    if (matAuntIds.length > 0) {
      const probeBranches: DownBranch[] = [];
      for (const aid of matAuntIds) {
        const b = buildDownBranch(aid, -1, false, maxGen - 2);
        if (b) probeBranches.push(b);
      }
      matAuntsW =
        probeBranches.reduce((sum, b) => sum + b.subtreeWidth, 0) +
        Math.max(0, probeBranches.length - 1) * H_GAP;

      const centerEndX = centerStartX + centerGroupW;
      const matAuntsStartX = Math.max(centerEndX, motherPlacedX + CARD_WIDTH) + H_GAP;
      let cur = matAuntsStartX;
      for (const b of probeBranches) {
        renderDownBranch(b, cur);
        cur += b.subtreeWidth + H_GAP;
      }
      matAuntBranches = probeBranches;
    }

    // Place Maternal Grandparents at Gen -2 (bus staggered by +6px)
    const gpY = -2 * ROW_STEP;
    const gpBusY = gpY + CARD_HEIGHT + V_GAP / 2 + 6;
    const matChildLeftX = motherPlacedX;
    const matChildRightX =
      matAuntsW > 0
        ? Math.max(centerStartX + centerGroupW, motherPlacedX + CARD_WIDTH) + H_GAP + matAuntsW
        : motherPlacedX + CARD_WIDTH;
    let matSpanCenterX = (matChildLeftX + matChildRightX) / 2;

    // Enforce half-plane clearance: maternal ancestors must remain strictly in right half-plane
    const matGrandparentWidth =
      hasMatGf && hasMatGm
        ? CARD_WIDTH * 2 + SPOUSE_GAP
        : CARD_WIDTH;
    const matCoupleHalfWidth = matGrandparentWidth / 2;
    const matMinSpanCenterX = focalCenterX + H_GAP / 2 + matCoupleHalfWidth;
    matSpanCenterX = Math.max(matSpanCenterX, matMinSpanCenterX);

    if (hasMatGf && hasMatGm) {
      placedIds.add(matGfId!);
      placedIds.add(matGmId!);

      const gfX = matSpanCenterX - CARD_WIDTH - SPOUSE_GAP / 2;
      const gmX = matSpanCenterX + SPOUSE_GAP / 2;

      nodes.push({
        id: matGfId!,
        node: matGfDto!,
        x: gfX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });

      nodes.push({
        id: matGmId!,
        node: matGmDto!,
        x: gmX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });

      const spouseY = gpY + CARD_HEIGHT / 2;
      edges.push(
        createPositionedEdge({
          id: `hg-anc-spouse-${matGfId}-${matGmId}`,
          sourceId: matGfId!,
          targetId: matGmId!,
          kind: 'spouse',
          sourceX: gfX + CARD_WIDTH,
          sourceY: spouseY,
          targetX: gmX,
          targetY: spouseY,
          pathD: buildSpouseConnectorPath(gfX + CARD_WIDTH, spouseY, gmX, spouseY),
          isDirectLineage: true,
        })
      );

      const gpUnionCenterX = (gfX + CARD_WIDTH + gmX) / 2;

      // Stem to Mother
      edges.push(
        createPositionedEdge({
          id: `hg-anc-${matGfId}-${motherId}`,
          sourceId: matGfId!,
          targetId: motherId,
          kind: 'parent_child',
          sourceX: gpUnionCenterX,
          sourceY: spouseY,
          targetX: motherPlacedX + CARD_WIDTH / 2,
          targetY: parentY,
          yBus: gpBusY,
          pathD: buildVerticalBusPath(
            gpUnionCenterX,
            spouseY,
            motherPlacedX + CARD_WIDTH / 2,
            parentY,
            gpBusY
          ),
          isDirectLineage: true,
        })
      );

      // Stems to Maternal Aunts/Uncles
      for (const ab of matAuntBranches) {
        const aNode = nodes.find((n) => n.id === ab.personId);
        if (aNode) {
          edges.push(
            createPositionedEdge({
              id: `hg-anc-${matGfId}-${ab.personId}`,
              sourceId: matGfId!,
              targetId: ab.personId,
              kind: 'parent_child',
              sourceX: gpUnionCenterX,
              sourceY: spouseY,
              targetX: aNode.x + CARD_WIDTH / 2,
              targetY: aNode.y,
              yBus: gpBusY,
              pathD: buildVerticalBusPath(
                gpUnionCenterX,
                spouseY,
                aNode.x + CARD_WIDTH / 2,
                aNode.y,
                gpBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
    } else if (hasMatGf) {
      placedIds.add(matGfId!);
      const gfX = matSpanCenterX - CARD_WIDTH / 2;
      nodes.push({
        id: matGfId!,
        node: matGfDto!,
        x: gfX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });
      const singleMatGfStemX = gfX + CARD_WIDTH / 2 + 12;

      edges.push(
        createPositionedEdge({
          id: `hg-anc-${matGfId}-${motherId}`,
          sourceId: matGfId!,
          targetId: motherId,
          kind: 'parent_child',
          sourceX: singleMatGfStemX,
          sourceY: gpY + CARD_HEIGHT,
          targetX: motherPlacedX + CARD_WIDTH / 2,
          targetY: parentY,
          yBus: gpBusY,
          pathD: buildVerticalBusPath(
            singleMatGfStemX,
            gpY + CARD_HEIGHT,
            motherPlacedX + CARD_WIDTH / 2,
            parentY,
            gpBusY
          ),
          isDirectLineage: true,
        })
      );

      for (const ab of matAuntBranches) {
        const aNode = nodes.find((n) => n.id === ab.personId);
        if (aNode) {
          edges.push(
            createPositionedEdge({
              id: `hg-anc-${matGfId}-${ab.personId}`,
              sourceId: matGfId!,
              targetId: ab.personId,
              kind: 'parent_child',
              sourceX: singleMatGfStemX,
              sourceY: gpY + CARD_HEIGHT,
              targetX: aNode.x + CARD_WIDTH / 2,
              targetY: aNode.y,
              yBus: gpBusY,
              pathD: buildVerticalBusPath(
                singleMatGfStemX,
                gpY + CARD_HEIGHT,
                aNode.x + CARD_WIDTH / 2,
                aNode.y,
                gpBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
    } else if (hasMatGm) {
      placedIds.add(matGmId!);
      const gmX = matSpanCenterX - CARD_WIDTH / 2;
      nodes.push({
        id: matGmId!,
        node: matGmDto!,
        x: gmX,
        y: gpY,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: -2,
        isFocal: false,
        isDirectLineage: true,
      });
      const singleMatGmStemX = gmX + CARD_WIDTH / 2 - 12;

      edges.push(
        createPositionedEdge({
          id: `hg-anc-${matGmId}-${motherId}`,
          sourceId: matGmId!,
          targetId: motherId,
          kind: 'parent_child',
          sourceX: singleMatGmStemX,
          sourceY: gpY + CARD_HEIGHT,
          targetX: motherPlacedX + CARD_WIDTH / 2,
          targetY: parentY,
          yBus: gpBusY,
          pathD: buildVerticalBusPath(
            singleMatGmStemX,
            gpY + CARD_HEIGHT,
            motherPlacedX + CARD_WIDTH / 2,
            parentY,
            gpBusY
          ),
          isDirectLineage: true,
        })
      );

      for (const ab of matAuntBranches) {
        const aNode = nodes.find((n) => n.id === ab.personId);
        if (aNode) {
          edges.push(
            createPositionedEdge({
              id: `hg-anc-${matGmId}-${ab.personId}`,
              sourceId: matGmId!,
              targetId: ab.personId,
              kind: 'parent_child',
              sourceX: singleMatGmStemX,
              sourceY: gpY + CARD_HEIGHT,
              targetX: aNode.x + CARD_WIDTH / 2,
              targetY: aNode.y,
              yBus: gpBusY,
              pathD: buildVerticalBusPath(
                singleMatGmStemX,
                gpY + CARD_HEIGHT,
                aNode.x + CARD_WIDTH / 2,
                aNode.y,
                gpBusY
              ),
              isDirectLineage: false,
            })
          );
        }
      }
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
