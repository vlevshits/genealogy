import {
  buildSpouseConnectorPath,
  buildVerticalBusPath,
} from "../curve-routing";
import {
  CARD_HEIGHT,
  CARD_WIDTH,
  H_GAP,
  ROW_STEP,
  SPOUSE_GAP,
  V_GAP,
  createPositionedEdge,
} from "../types";
import { DownBranch, HourglassContext } from "./down-branch";
import { placeHourglassUpperAncestorsGen } from "./upper-ancestors-gen";

export interface GrandparentsPlacementInput {
  ctx: HourglassContext;
  fatherId: string | null;
  motherId: string | null;
  fatherPlacedX: number | null;
  motherPlacedX: number | null;
  centerStartX: number;
  centerGroupW: number;
  focalCenterX: number;
  buildDownBranch: (
    personId: string,
    currentGen: number,
    isDirect: boolean,
    maxDepth: number
  ) => DownBranch | null;
  renderDownBranch: (
    branch: DownBranch,
    leftX: number,
    siblingIndex?: number
  ) => { anchorX: number; anchorY: number };
}

export function placeHourglassGrandparentsGen(input: GrandparentsPlacementInput): void {
  const {
    ctx,
    fatherId,
    motherId,
    fatherPlacedX,
    motherPlacedX,
    centerStartX,
    centerGroupW,
    focalCenterX,
    buildDownBranch,
    renderDownBranch,
  } = input;
  const { nodeMap, relativesMap, maxGen, collateral, placedIds, nodes, edges } = ctx;
  const parentY = -ROW_STEP;

  const ancestorGenLimit = Math.max(1, maxGen);

  let patGfId: string | null = null;
  let patGmId: string | null = null;
  let matGfId: string | null = null;
  let matGmId: string | null = null;

  // 5A. Paternal Side (Grandparents + Aunts/Uncles at Gen -2 / Gen -1)
  if (ancestorGenLimit >= 2 && fatherId && fatherPlacedX !== null) {
    const fRel = relativesMap.get(fatherId);
    patGfId = fRel?.fatherId ?? null;
    patGmId = fRel?.motherId ?? null;
    if (patGfId && patGmId && patGfId === patGmId) {
      if (nodeMap.get(patGfId)?.sex === 'F') patGfId = null;
      else patGmId = null;
    }
    if (!patGfId && !patGmId && fRel?.parentIds.length) {
      patGfId = fRel.parentIds[0] ?? null;
      patGmId = fRel.parentIds[1] ?? null;
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
        const b = buildDownBranch(aid, -1, false, maxGen - 1);
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
      const singlePatGfStemX = gfX + CARD_WIDTH / 2;

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
      const singlePatGmStemX = gmX + CARD_WIDTH / 2;

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
    matGfId = mRel?.fatherId ?? null;
    matGmId = mRel?.motherId ?? null;
    if (matGfId && matGmId && matGfId === matGmId) {
      if (nodeMap.get(matGfId)?.sex === 'F') matGfId = null;
      else matGmId = null;
    }
    if (!matGfId && !matGmId && mRel?.parentIds.length) {
      matGfId = mRel.parentIds[0] ?? null;
      matGmId = mRel.parentIds[1] ?? null;
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
        const b = buildDownBranch(aid, -1, false, maxGen - 1);
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
      const singleMatGfStemX = gfX + CARD_WIDTH / 2;

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
      const singleMatGmStemX = gmX + CARD_WIDTH / 2;

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

  // 5C. Upper Ancestors (Gen -3, -4, -5, -6, ... up to -maxGen)
  if (ancestorGenLimit >= 3) {
    placeHourglassUpperAncestorsGen(
      ctx,
      patGfId,
      patGmId,
      matGfId,
      matGmId,
      ancestorGenLimit
    );
  }
}
