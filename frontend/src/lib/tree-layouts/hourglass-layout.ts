import { TreeViewResponse } from "../../api/client";
import {
  buildTreeGraphIndex,
  CARD_WIDTH,
  computeLayoutBounds,
  FamilyUnion,
  H_GAP,
  PlaceholderSlot,
  PositionedEdge,
  PositionedNode,
  TreeLayoutResult,
} from "./types";
import {
  createDownBranchEngine,
  DownBranch,
  HourglassContext,
} from "./hourglass/down-branch";
import { placeHourglassParentsGen } from "./hourglass/parents-gen";
import { placeHourglassGrandparentsGen } from "./hourglass/grandparents-gen";

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
      mode: "hourglass",
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

  const ctx: HourglassContext = {
    nodeMap,
    relativesMap,
    focalId,
    maxGen,
    collateral,
    placedIds,
    nodes,
    placeholders,
    edges,
  };

  const { buildDownBranch, renderDownBranch } = createDownBranchEngine(ctx);

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
  const focalBranch = buildDownBranch(focalId, 0, true, maxGen)!;

  const patHalfBranches: DownBranch[] = [];
  for (const sid of patHalfSiblingIds) {
    const sb = buildDownBranch(sid, 0, false, maxGen);
    if (sb) patHalfBranches.push(sb);
  }

  const fullSibBranches: DownBranch[] = [];
  for (const sid of fullSiblingIds) {
    const sb = buildDownBranch(sid, 0, false, maxGen);
    if (sb) fullSibBranches.push(sb);
  }

  const matHalfBranches: DownBranch[] = [];
  for (const sid of matHalfSiblingIds) {
    const sb = buildDownBranch(sid, 0, false, maxGen);
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


  const { fatherPlacedX, motherPlacedX } = placeHourglassParentsGen({
    ctx,
    fatherId,
    motherId,
    fatherDto,
    motherDto,
    hasFather,
    hasMother,
    patExtraSpouseId,
    patExtraUnion,
    matExtraSpouseId,
    matExtraUnion,
    focalNode,
    focalCenterX,
    fullSibBranches,
    patHalfBranches,
    matHalfBranches,
    allSiblingBranches,
  });

  placeHourglassGrandparentsGen({
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
  });

  return {
    mode: "hourglass",
    focalId,
    nodes,
    placeholders,
    edges,
    bounds: computeLayoutBounds(nodes, placeholders),
  };
}
