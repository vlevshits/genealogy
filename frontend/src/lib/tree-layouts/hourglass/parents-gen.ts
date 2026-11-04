import {
  buildSpouseConnectorPath,
  buildVerticalBusPath,
} from "../curve-routing";
import {
  CARD_HEIGHT,
  CARD_WIDTH,
  FamilyUnion,
  PositionedNode,
  ROW_STEP,
  SPOUSE_GAP,
  V_GAP,
  createPositionedEdge,
} from "../types";
import { TreePersonNodeDto } from "../../../api/client";
import { DownBranch, HourglassContext } from "./down-branch";

export interface ParentsPlacementInput {
  ctx: HourglassContext;
  fatherId: string | null;
  motherId: string | null;
  fatherDto: TreePersonNodeDto | undefined;
  motherDto: TreePersonNodeDto | undefined;
  hasFather: boolean;
  hasMother: boolean;
  patExtraSpouseId: string | null;
  patExtraUnion: FamilyUnion | null;
  matExtraSpouseId: string | null;
  matExtraUnion: FamilyUnion | null;
  focalNode: PositionedNode;
  focalCenterX: number;
  fullSibBranches: DownBranch[];
  patHalfBranches: DownBranch[];
  matHalfBranches: DownBranch[];
  allSiblingBranches: DownBranch[];
}

export function placeHourglassParentsGen(input: ParentsPlacementInput): {
  fatherPlacedX: number | null;
  motherPlacedX: number | null;
} {
  const {
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
  } = input;
  const { nodeMap, focalId, placedIds, nodes, placeholders, edges } = ctx;

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

  return { fatherPlacedX, motherPlacedX };
}
