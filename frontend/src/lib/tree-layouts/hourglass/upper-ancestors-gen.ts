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
import { HourglassContext } from "./down-branch";

interface AncestorSlot {
  id: string;
  side: "paternal" | "maternal";
}

/**
 * Finds a non-overlapping horizontal X coordinate on a given generation row
 * starting near `preferredX` and stepping in `direction` until `[x, x + width]`
 * has at least `H_GAP` clearance from all existing nodes and placeholders on that row.
 */
function findFreeSlotOnRow(
  ctx: HourglassContext,
  generation: number,
  preferredX: number,
  width: number,
  direction: "left" | "right"
): number {
  const rowY = generation * ROW_STEP;
  const occupied: Array<{ left: number; right: number }> = [];

  for (const n of ctx.nodes) {
    if (n.generation === generation || Math.abs(n.y - rowY) < 10) {
      occupied.push({ left: n.x, right: n.x + n.width });
    }
  }
  for (const p of ctx.placeholders) {
    if (p.generation === generation || Math.abs(p.y - rowY) < 10) {
      occupied.push({ left: p.x, right: p.x + p.width });
    }
  }

  const overlapsAny = (x: number): { left: number; right: number } | null => {
    for (const interval of occupied) {
      if (x + width + H_GAP > interval.left && x - H_GAP < interval.right) {
        return interval;
      }
    }
    return null;
  };

  let candidateX = Math.round(preferredX);
  let guard = 0;
  while (guard < 200) {
    const hit = overlapsAny(candidateX);
    if (!hit) return candidateX;
    if (direction === "left") {
      candidateX = Math.round(hit.left - H_GAP - width);
    } else {
      candidateX = Math.round(hit.right + H_GAP);
    }
    guard++;
  }
  return candidateX;
}

/**
 * Places upper ancestor generations (Gen -3, -4, -5, -6, ... up to -maxGen)
 * along with their collateral siblings and extra spouses in Hourglass layout.
 */
export function placeHourglassUpperAncestorsGen(
  ctx: HourglassContext,
  patGfId: string | null,
  patGmId: string | null,
  matGfId: string | null,
  matGmId: string | null,
  maxGen: number
): void {
  const { nodeMap, relativesMap, collateral, placedIds, nodes, edges } = ctx;

  let currentLevelAncestors: AncestorSlot[] = [];
  if (patGfId && placedIds.has(patGfId)) {
    currentLevelAncestors.push({ id: patGfId, side: "paternal" });
  }
  if (patGmId && placedIds.has(patGmId)) {
    currentLevelAncestors.push({ id: patGmId, side: "paternal" });
  }
  if (matGfId && placedIds.has(matGfId)) {
    currentLevelAncestors.push({ id: matGfId, side: "maternal" });
  }
  if (matGmId && placedIds.has(matGmId)) {
    currentLevelAncestors.push({ id: matGmId, side: "maternal" });
  }

  for (let gen = -3; gen >= -maxGen; gen--) {
    if (currentLevelAncestors.length === 0) break;

    // Sort placed ancestors at (gen + 1) from left to right
    currentLevelAncestors.sort((a, b) => {
      const nodeA = nodes.find((n) => n.id === a.id);
      const nodeB = nodes.find((n) => n.id === b.id);
      return (nodeA?.x ?? 0) - (nodeB?.x ?? 0);
    });

    const nextLevelAncestors: AncestorSlot[] = [];
    const rowY = gen * ROW_STEP;
    const childRowY = (gen + 1) * ROW_STEP;
    const baseBusY = childRowY - V_GAP / 2;
    let busCounter = 0;

    for (const anc of currentLevelAncestors) {
      const childId = anc.id;
      const childNode = nodes.find((n) => n.id === childId);
      if (!childNode) continue;

      const rel = relativesMap.get(childId);
      let fId = rel?.fatherId ?? null;
      let mId = rel?.motherId ?? null;
      if (fId && mId && fId === mId) {
        if (nodeMap.get(fId)?.sex === "F") fId = null;
        else mId = null;
      }
      // Fallback to parentIds if sex was unknown
      if (!fId && !mId && rel?.parentIds.length) {
        fId = rel.parentIds[0] ?? null;
        mId = rel.parentIds[1] ?? null;
      }

      const fDto = fId ? nodeMap.get(fId) : undefined;
      const mDto = mId ? nodeMap.get(mId) : undefined;
      const hasF = Boolean(fId && fDto && !placedIds.has(fId));
      const hasM = Boolean(mId && mDto && !placedIds.has(mId));

      if (!hasF && !hasM) continue;

      // Identify extra spouses of fId or mId (secondary marriages at `gen`)
      let fExtraSpouseId: string | null = null;
      let mExtraSpouseId: string | null = null;

      if (collateral && hasF && fId) {
        const fRel = relativesMap.get(fId);
        for (const sid of fRel?.spouseIds ?? []) {
          if (sid !== mId && nodeMap.has(sid) && !placedIds.has(sid)) {
            fExtraSpouseId = sid;
            break;
          }
        }
      }
      if (collateral && hasM && mId) {
        const mRel = relativesMap.get(mId);
        for (const sid of mRel?.spouseIds ?? []) {
          if (
            sid !== fId &&
            sid !== fExtraSpouseId &&
            nodeMap.has(sid) &&
            !placedIds.has(sid)
          ) {
            mExtraSpouseId = sid;
            break;
          }
        }
      }

      // Identify unplaced half-siblings belonging strictly to fExtraSpouseId or mExtraSpouseId
      const fExtraChildIds = new Set<string>();
      if (fExtraSpouseId && fId) {
        const fUnion = relativesMap
          .get(fId)
          ?.unions.find(
            (u) =>
              (u.partner1Id === fId && u.partner2Id === fExtraSpouseId) ||
              (u.partner2Id === fId && u.partner1Id === fExtraSpouseId)
          );
        for (const cid of fUnion?.childIds ?? []) {
          if (cid !== childId && nodeMap.has(cid) && !placedIds.has(cid)) {
            fExtraChildIds.add(cid);
          }
        }
      }

      const mExtraChildIds = new Set<string>();
      if (mExtraSpouseId && mId) {
        const mUnion = relativesMap
          .get(mId)
          ?.unions.find(
            (u) =>
              (u.partner1Id === mId && u.partner2Id === mExtraSpouseId) ||
              (u.partner2Id === mId && u.partner1Id === mExtraSpouseId)
          );
        for (const cid of mUnion?.childIds ?? []) {
          if (
            cid !== childId &&
            !fExtraChildIds.has(cid) &&
            nodeMap.has(cid) &&
            !placedIds.has(cid)
          ) {
            mExtraChildIds.add(cid);
          }
        }
      }

      // 1. Place unplaced full/shared siblings of childId on row (gen + 1)
      const placedSiblingPositions: Array<{ id: string; x: number }> = [];
      if (collateral) {
        const candidateSibIds: string[] = [];
        if (hasF && fId) {
          for (const cid of relativesMap.get(fId)?.childIds ?? []) {
            if (
              cid !== childId &&
              !fExtraChildIds.has(cid) &&
              !mExtraChildIds.has(cid) &&
              !candidateSibIds.includes(cid) &&
              nodeMap.has(cid) &&
              !placedIds.has(cid)
            ) {
              candidateSibIds.push(cid);
            }
          }
        }
        if (hasM && mId) {
          for (const cid of relativesMap.get(mId)?.childIds ?? []) {
            if (
              cid !== childId &&
              !fExtraChildIds.has(cid) &&
              !mExtraChildIds.has(cid) &&
              !candidateSibIds.includes(cid) &&
              nodeMap.has(cid) &&
              !placedIds.has(cid)
            ) {
              candidateSibIds.push(cid);
            }
          }
        }

        const sibDir: "left" | "right" =
          anc.side === "paternal" ? "left" : "right";

        for (const sibId of candidateSibIds) {
          const sibDto = nodeMap.get(sibId);
          if (!sibDto || placedIds.has(sibId)) continue;

          // Check if sibling has an unplaced spouse in nodeMap
          const sibRel = relativesMap.get(sibId);
          let sibSpouseId: string | null = null;
          for (const spId of sibRel?.spouseIds ?? []) {
            if (nodeMap.has(spId) && !placedIds.has(spId)) {
              sibSpouseId = spId;
              break;
            }
          }

          const unitW = sibSpouseId
            ? CARD_WIDTH * 2 + SPOUSE_GAP
            : CARD_WIDTH;
          const preferredStart =
            sibDir === "left"
              ? childNode.x - H_GAP - unitW
              : childNode.x + CARD_WIDTH + H_GAP;
          const slotX = findFreeSlotOnRow(
            ctx,
            gen + 1,
            preferredStart,
            unitW,
            sibDir
          );

          placedIds.add(sibId);
          const sibX = slotX;
          nodes.push({
            id: sibId,
            node: sibDto,
            x: sibX,
            y: childRowY,
            width: CARD_WIDTH,
            height: CARD_HEIGHT,
            generation: gen + 1,
            isFocal: false,
            isDirectLineage: false,
          });
          placedSiblingPositions.push({ id: sibId, x: sibX });

          if (sibSpouseId) {
            const sibSpouseDto = nodeMap.get(sibSpouseId)!;
            placedIds.add(sibSpouseId);
            const spX = sibX + CARD_WIDTH + SPOUSE_GAP;
            nodes.push({
              id: sibSpouseId,
              node: sibSpouseDto,
              x: spX,
              y: childRowY,
              width: CARD_WIDTH,
              height: CARD_HEIGHT,
              generation: gen + 1,
              isFocal: false,
              isDirectLineage: false,
            });
            const spY = childRowY + CARD_HEIGHT / 2;
            edges.push(
              createPositionedEdge({
                id: `hg-anc-sib-spouse-${sibId}-${sibSpouseId}`,
                sourceId: sibId,
                targetId: sibSpouseId,
                kind: "spouse",
                sourceX: sibX + CARD_WIDTH,
                sourceY: spY,
                targetX: spX,
                targetY: spY,
                pathD: buildSpouseConnectorPath(
                  sibX + CARD_WIDTH,
                  spY,
                  spX,
                  spY
                ),
                isDirectLineage: false,
              })
            );
          }
        }
      }

      // 2. Compute horizontal span of child + placed full siblings to center parent group
      const allChildrenX = [
        childNode.x,
        ...placedSiblingPositions.map((s) => s.x),
      ];
      const minChildX = Math.min(...allChildrenX);
      const maxChildX = Math.max(...allChildrenX) + CARD_WIDTH;
      const childrenCenterX = (minChildX + maxChildX) / 2;

      // Build ordered parent row slots: [fExtraSpouse?, fId?, mId?, mExtraSpouse?]
      const parentGroupIds: string[] = [];
      if (fExtraSpouseId && hasF) parentGroupIds.push(fExtraSpouseId);
      if (hasF && fId) parentGroupIds.push(fId);
      if (hasM && mId) parentGroupIds.push(mId);
      if (mExtraSpouseId && hasM) parentGroupIds.push(mExtraSpouseId);

      const groupWidth =
        parentGroupIds.length * CARD_WIDTH +
        Math.max(0, parentGroupIds.length - 1) * SPOUSE_GAP;
      const preferredGroupLeft = childrenCenterX - groupWidth / 2;
      const parentDir: "left" | "right" =
        anc.side === "paternal" ? "left" : "right";
      const groupStartX = findFreeSlotOnRow(
        ctx,
        gen,
        preferredGroupLeft,
        groupWidth,
        parentDir
      );

      const placedParentX = new Map<string, number>();
      let cursorX = groupStartX;
      for (const pid of parentGroupIds) {
        const pDto = nodeMap.get(pid)!;
        const isDirect = pid === fId || pid === mId;
        placedIds.add(pid);
        placedParentX.set(pid, cursorX);
        nodes.push({
          id: pid,
          node: pDto,
          x: cursorX,
          y: rowY,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: gen,
          isFocal: false,
          isDirectLineage: isDirect,
        });
        if (isDirect) {
          nextLevelAncestors.push({ id: pid, side: anc.side });
        }
        cursorX += CARD_WIDTH + SPOUSE_GAP;
      }

      // 3. Connect primary parent(s) to childId + placed full siblings
      const fPlacedX = fId ? placedParentX.get(fId) ?? null : null;
      const mPlacedX = mId ? placedParentX.get(mId) ?? null : null;
      const spouseY = rowY + CARD_HEIGHT / 2;
      const yBus = baseBusY + (busCounter * 12 - 12);
      busCounter++;

      let primarySourceId: string;
      let primaryDropX: number;
      let primaryDropY: number;

      if (fPlacedX !== null && mPlacedX !== null && fId && mId) {
        edges.push(
          createPositionedEdge({
            id: `hg-anc-spouse-${fId}-${mId}`,
            sourceId: fId,
            targetId: mId,
            kind: "spouse",
            sourceX: fPlacedX + CARD_WIDTH,
            sourceY: spouseY,
            targetX: mPlacedX,
            targetY: spouseY,
            pathD: buildSpouseConnectorPath(
              fPlacedX + CARD_WIDTH,
              spouseY,
              mPlacedX,
              spouseY
            ),
            isDirectLineage: true,
          })
        );
        primarySourceId = fId;
        primaryDropX = (fPlacedX + CARD_WIDTH + mPlacedX) / 2;
        primaryDropY = spouseY;
      } else if (fPlacedX !== null && fId) {
        primarySourceId = fId;
        primaryDropX = fPlacedX + CARD_WIDTH / 2 + 12;
        primaryDropY = rowY + CARD_HEIGHT;
      } else {
        primarySourceId = mId!;
        primaryDropX = mPlacedX! + CARD_WIDTH / 2 - 12;
        primaryDropY = rowY + CARD_HEIGHT;
      }

      // Direct lineage edge to childId
      edges.push(
        createPositionedEdge({
          id: `hg-anc-${primarySourceId}-${childId}`,
          sourceId: primarySourceId,
          targetId: childId,
          kind: "parent_child",
          sourceX: primaryDropX,
          sourceY: primaryDropY,
          targetX: childNode.x + CARD_WIDTH / 2,
          targetY: childRowY,
          yBus,
          pathD: buildVerticalBusPath(
            primaryDropX,
            primaryDropY,
            childNode.x + CARD_WIDTH / 2,
            childRowY,
            yBus
          ),
          isDirectLineage: true,
        })
      );

      // Collateral edges to placed full siblings
      for (const sib of placedSiblingPositions) {
        edges.push(
          createPositionedEdge({
            id: `hg-anc-${primarySourceId}-${sib.id}`,
            sourceId: primarySourceId,
            targetId: sib.id,
            kind: "parent_child",
            sourceX: primaryDropX,
            sourceY: primaryDropY,
            targetX: sib.x + CARD_WIDTH / 2,
            targetY: childRowY,
            yBus,
            pathD: buildVerticalBusPath(
              primaryDropX,
              primaryDropY,
              sib.x + CARD_WIDTH / 2,
              childRowY,
              yBus
            ),
            isDirectLineage: false,
          })
        );
      }

      // 4. Connect extra spouses and any half-siblings at (gen + 1)
      const connectExtraUnion = (
        anchorParentId: string,
        extraSpouseId: string,
        halfSibIds: Set<string>,
        dir: "left" | "right"
      ) => {
        const pX = placedParentX.get(anchorParentId);
        const spX = placedParentX.get(extraSpouseId);
        if (pX === undefined || spX === undefined) return;

        const leftX = Math.min(pX, spX);
        const rightX = Math.max(pX, spX);

        edges.push(
          createPositionedEdge({
            id: `hg-anc-extra-spouse-${anchorParentId}-${extraSpouseId}`,
            sourceId: anchorParentId,
            targetId: extraSpouseId,
            kind: "spouse",
            sourceX: leftX + CARD_WIDTH,
            sourceY: spouseY,
            targetX: rightX,
            targetY: spouseY,
            pathD: buildSpouseConnectorPath(
              leftX + CARD_WIDTH,
              spouseY,
              rightX,
              spouseY
            ),
            isDirectLineage: false,
          })
        );

        if (halfSibIds.size > 0) {
          const unionCenterX = (leftX + CARD_WIDTH + rightX) / 2;
          const extraBusY = baseBusY + (busCounter * 12 - 12);
          busCounter++;

          for (const hsId of halfSibIds) {
            const hsDto = nodeMap.get(hsId);
            if (!hsDto || placedIds.has(hsId)) continue;
            placedIds.add(hsId);
            const hsX = findFreeSlotOnRow(
              ctx,
              gen + 1,
              spX,
              CARD_WIDTH,
              dir
            );
            nodes.push({
              id: hsId,
              node: hsDto,
              x: hsX,
              y: childRowY,
              width: CARD_WIDTH,
              height: CARD_HEIGHT,
              generation: gen + 1,
              isFocal: false,
              isDirectLineage: false,
            });
            edges.push(
              createPositionedEdge({
                id: `hg-anc-extra-child-${anchorParentId}-${hsId}`,
                sourceId: anchorParentId,
                targetId: hsId,
                kind: "parent_child",
                sourceX: unionCenterX,
                sourceY: spouseY,
                targetX: hsX + CARD_WIDTH / 2,
                targetY: childRowY,
                yBus: extraBusY,
                pathD: buildVerticalBusPath(
                  unionCenterX,
                  spouseY,
                  hsX + CARD_WIDTH / 2,
                  childRowY,
                  extraBusY
                ),
                isDirectLineage: false,
              })
            );
          }
        }
      };

      if (fExtraSpouseId && fId) {
        connectExtraUnion(fId, fExtraSpouseId, fExtraChildIds, "left");
      }
      if (mExtraSpouseId && mId) {
        connectExtraUnion(mId, mExtraSpouseId, mExtraChildIds, "right");
      }
    }

    currentLevelAncestors = nextLevelAncestors;
  }
}
