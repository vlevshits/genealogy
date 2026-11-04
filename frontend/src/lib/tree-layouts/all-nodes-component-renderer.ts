import { TreePersonNodeDto, TreeViewResponse } from "../../api/client";
import {
  buildSpouseConnectorPath,
  buildVerticalBusPath,
} from "./curve-routing";
import {
  CARD_HEIGHT,
  CARD_WIDTH,
  PersonRelatives,
  PositionedEdge,
  PositionedNode,
  ROW_STEP,
  SPOUSE_GAP,
  V_GAP,
  createPositionedEdge,
} from "./types";

export interface ComponentUnit {
  id: string;
  personIds: string[];
  generation: number;
  width: number;
  x: number;
  childrenUnitIds: string[];
  parentUnitIds: string[];
}

export interface RenderComponentInput {
  data: TreeViewResponse;
  compSet: Set<string>;
  isPrimary: boolean;
  focalId: string | null;
  directLineageSet: Set<string>;
  nodeMap: Map<string, TreePersonNodeDto>;
  relativesMap: Map<string, PersonRelatives>;
  unitMap: Map<string, ComponentUnit>;
  personToUnit: Map<string, string>;
}

export function renderAllNodesComponent(input: RenderComponentInput): {
  nodes: PositionedNode[];
  edges: PositionedEdge[];
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
} {
  const {
    data,
    compSet,
    isPrimary,
    focalId,
    directLineageSet,
    nodeMap,
    relativesMap,
    unitMap,
    personToUnit,
  } = input;
  const compNodes: PositionedNode[] = [];
  const compEdges: PositionedEdge[] = [];

    // 3e. Phase 1: Position all nodes in component
    const nodePositionMap = new Map<string, PositionedNode>();

    for (const u of unitMap.values()) {
      const y = u.generation * ROW_STEP;

      if (u.personIds.length === 3) {
        // [S1, Person, S2]
        const s1Id = u.personIds[0];
        const pId = u.personIds[1];
        const s2Id = u.personIds[2];
        const s1Dto = nodeMap.get(s1Id)!;
        const pDto = nodeMap.get(pId)!;
        const s2Dto = nodeMap.get(s2Id)!;

        const s1X = u.x;
        const pX = u.x + CARD_WIDTH + SPOUSE_GAP;
        const s2X = u.x + (CARD_WIDTH + SPOUSE_GAP) * 2;

        const nS1: PositionedNode = {
          id: s1Id,
          node: s1Dto,
          x: s1X,
          y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: u.generation,
          isFocal: s1Id === focalId,
          isSpouseOfFocal: pId === focalId,
          isDirectLineage: directLineageSet.has(s1Id),
        };
        compNodes.push(nS1);
        nodePositionMap.set(s1Id, nS1);

        const nP: PositionedNode = {
          id: pId,
          node: pDto,
          x: pX,
          y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: u.generation,
          isFocal: pId === focalId,
          isDirectLineage: directLineageSet.has(pId),
        };
        compNodes.push(nP);
        nodePositionMap.set(pId, nP);

        const nS2: PositionedNode = {
          id: s2Id,
          node: s2Dto,
          x: s2X,
          y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: u.generation,
          isFocal: s2Id === focalId,
          isSpouseOfFocal: pId === focalId,
          isDirectLineage: directLineageSet.has(s2Id),
        };
        compNodes.push(nS2);
        nodePositionMap.set(s2Id, nS2);
      } else if (u.personIds.length === 2) {
        const p1 = u.personIds[0];
        const p2 = u.personIds[1];
        const p1Dto = nodeMap.get(p1)!;
        const p2Dto = nodeMap.get(p2)!;
        const p1X = u.x;
        const p2X = u.x + CARD_WIDTH + SPOUSE_GAP;

        const n1: PositionedNode = {
          id: p1,
          node: p1Dto,
          x: p1X,
          y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: u.generation,
          isFocal: p1 === focalId,
          isDirectLineage: directLineageSet.has(p1),
        };
        compNodes.push(n1);
        nodePositionMap.set(p1, n1);

        const n2: PositionedNode = {
          id: p2,
          node: p2Dto,
          x: p2X,
          y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: u.generation,
          isFocal: p2 === focalId,
          isSpouseOfFocal: p1 === focalId || p2 === focalId,
          isDirectLineage: directLineageSet.has(p2),
        };
        compNodes.push(n2);
        nodePositionMap.set(p2, n2);
      } else {
        const p1 = u.personIds[0];
        const p1Dto = nodeMap.get(p1)!;
        const n1: PositionedNode = {
          id: p1,
          node: p1Dto,
          x: u.x,
          y,
          width: CARD_WIDTH,
          height: CARD_HEIGHT,
          generation: u.generation,
          isFocal: p1 === focalId,
          isDirectLineage: directLineageSet.has(p1),
        };
        compNodes.push(n1);
        nodePositionMap.set(p1, n1);
      }
    }

    // 3f. Phase 2: Generate all edges in component
    const createdSpouseEdges = new Set<string>();

    // Helper to add spouse edge
    const addSpouseEdge = (id1: string, id2: string) => {
      if (!compSet.has(id1) || !compSet.has(id2) || id1 === id2) return;
      const n1 = nodePositionMap.get(id1);
      const n2 = nodePositionMap.get(id2);
      if (!n1 || !n2) return;

      const [idA, idB] = [id1, id2].sort();
      const edgeId = `all-spouse-${idA}-${idB}`;
      if (createdSpouseEdges.has(edgeId)) return;
      createdSpouseEdges.add(edgeId);

      const inSameUnit = personToUnit.get(id1) && personToUnit.get(id1) === personToUnit.get(id2);

      const matchedSpouseFamily = (data.families ?? []).find(
        (f) =>
          (f.partner1_id === idA && f.partner2_id === idB) ||
          (f.partner1_id === idB && f.partner2_id === idA)
      );

      if (inSameUnit && Math.abs(n1.y - n2.y) < 10) {
        const leftNode = n1.x < n2.x ? n1 : n2;
        const rightNode = n1.x < n2.x ? n2 : n1;
        const sx = leftNode.x + CARD_WIDTH;
        const tx = rightNode.x;
        const sy = leftNode.y + CARD_HEIGHT / 2;
        const ty = leftNode.y + CARD_HEIGHT / 2;

        compEdges.push(
          createPositionedEdge({
            id: edgeId,
            sourceId: idA,
            targetId: idB,
            familyId: matchedSpouseFamily?.id,
            kind: 'spouse',
            sourceX: sx,
            sourceY: sy,
            targetX: tx,
            targetY: ty,
            pathD: buildSpouseConnectorPath(sx, sy, tx, ty),
            isDirectLineage: directLineageSet.has(idA) && directLineageSet.has(idB),
          })
        );
      } else if (Math.abs(n1.y - n2.y) < 10) {
        // Spouses on same generation but in different units (remarried / blended family)
        // Route through the upper gap to avoid intersecting any intervening cards
        const sx = n1.x + CARD_WIDTH / 2;
        const sy = n1.y;
        const tx = n2.x + CARD_WIDTH / 2;
        const ty = n2.y;
        const yBus = n1.y - V_GAP / 2;

        compEdges.push(
          createPositionedEdge({
            id: edgeId,
            sourceId: idA,
            targetId: idB,
            familyId: matchedSpouseFamily?.id,
            kind: 'spouse',
            sourceX: sx,
            sourceY: sy,
            targetX: tx,
            targetY: ty,
            yBus,
            pathD: buildVerticalBusPath(sx, sy, tx, ty, yBus),
            isDirectLineage: directLineageSet.has(idA) && directLineageSet.has(idB),
          })
        );
      } else {
        const topNode = n1.y <= n2.y ? n1 : n2;
        const bottomNode = n1.y <= n2.y ? n2 : n1;
        const sx = topNode.x + CARD_WIDTH / 2;
        const sy = topNode.y + CARD_HEIGHT;
        const tx = bottomNode.x + CARD_WIDTH / 2;
        const ty = bottomNode.y;
        const yBus = topNode.y + CARD_HEIGHT + V_GAP / 2;

        compEdges.push(
          createPositionedEdge({
            id: edgeId,
            sourceId: idA,
            targetId: idB,
            familyId: matchedSpouseFamily?.id,
            kind: 'spouse',
            sourceX: sx,
            sourceY: sy,
            targetX: tx,
            targetY: ty,
            yBus,
            pathD: buildVerticalBusPath(sx, sy, tx, ty, yBus),
            isDirectLineage: directLineageSet.has(idA) && directLineageSet.has(idB),
          })
        );
      }
    };

    // Populate spouse edges from families
    for (const fam of data.families ?? []) {
      if (fam.partner1_id && fam.partner2_id) {
        addSpouseEdge(fam.partner1_id, fam.partner2_id);
      }
    }

    // Populate spouse edges from relativesMap
    for (const pid of compSet) {
      const rel = relativesMap.get(pid);
      if (!rel) continue;
      for (const sid of rel.spouseIds) {
        addSpouseEdge(pid, sid);
      }
    }

    // Parent-Child Edges
    const createdChildEdges = new Set<string>();
    const familyStaggerMap = new Map<string, number>();
    let familyCounter = 0;
    const getFamilyBusStagger = (familyKey: string): number => {
      let stagger = familyStaggerMap.get(familyKey);
      if (stagger === undefined) {
        familyCounter++;
        stagger = (familyCounter % 2 === 0 ? -1 : 1) * 6;
        familyStaggerMap.set(familyKey, stagger);
      }
      return stagger;
    };

    for (const cid of compSet) {
      const childNode = nodePositionMap.get(cid);
      if (!childNode) continue;
      const childRel = relativesMap.get(cid);
      if (!childRel) continue;

      const parentsInComp = childRel.parentIds.filter(
        (pid) => compSet.has(pid) && nodePositionMap.has(pid)
      );
      if (parentsInComp.length === 0) continue;

      if (parentsInComp.length === 2) {
        const [p1, p2] = parentsInComp;
        const u1 = personToUnit.get(p1);
        const u2 = personToUnit.get(p2);

        if (u1 && u1 === u2) {
          // Both parents in same couple unit
          const n1 = nodePositionMap.get(p1)!;
          const n2 = nodePositionMap.get(p2)!;
          const leftNode = n1.x < n2.x ? n1 : n2;
          const rightNode = n1.x < n2.x ? n2 : n1;
          const rawSourceX = (leftNode.x + CARD_WIDTH + rightNode.x) / 2;
          const sourceY = leftNode.y + CARD_HEIGHT / 2;
          const targetX = childNode.x + CARD_WIDTH / 2;
          const targetY = childNode.y;
          const edgeId = `all-edge-${p1}-${cid}`;

          const isChildInFamily = (f: { children?: Array<unknown> }) =>
            Boolean(
              f.children?.some((c) => {
                const item = c as string | { child_id?: string; individual_id?: string };
                const childId = typeof item === 'string' ? item : item.child_id ?? item.individual_id;
                return childId === cid;
              })
            );

          const matchedFamily =
            (data.families ?? []).find(
              (f) =>
                isChildInFamily(f) &&
                ((f.partner1_id === p1 && f.partner2_id === p2) ||
                  (f.partner1_id === p2 && f.partner2_id === p1))
            ) ??
            (data.families ?? []).find(
              (f) =>
                (f.partner1_id === p1 && f.partner2_id === p2) ||
                (f.partner1_id === p2 && f.partner2_id === p1)
            ) ??
            (data.families ?? []).find((f) => isChildInFamily(f));

          const familyKey = matchedFamily?.id ?? [p1, p2].sort().join('-');
          const busStagger = getFamilyBusStagger(familyKey);
          const yBus = childNode.y - V_GAP / 2 + busStagger;
          const sourceX = rawSourceX;

          if (!createdChildEdges.has(edgeId)) {
            createdChildEdges.add(edgeId);
            compEdges.push(
              createPositionedEdge({
                id: edgeId,
                sourceId: p1,
                targetId: cid,
                familyId: matchedFamily?.id,
                kind: 'parent_child',
                sourceX,
                sourceY,
                targetX,
                targetY,
                yBus,
                pathD: buildVerticalBusPath(sourceX, sourceY, targetX, targetY, yBus),
                isDirectLineage:
                  (directLineageSet.has(p1) || directLineageSet.has(p2)) &&
                  directLineageSet.has(cid),
              })
            );
          }
          continue;
        }
      }

      // Single parent or parents not in the same couple unit
      for (const pid of parentsInComp) {
        const pNode = nodePositionMap.get(pid)!;
        const rawSourceX = pNode.x + CARD_WIDTH / 2;
        const sourceY = pNode.y + CARD_HEIGHT;
        const targetX = childNode.x + CARD_WIDTH / 2;
        const targetY = childNode.y;

        const isChildInFamily = (f: { children?: Array<unknown> }) =>
          Boolean(
            f.children?.some((c) => {
              const item = c as string | { child_id?: string; individual_id?: string };
              const childId = typeof item === 'string' ? item : item.child_id ?? item.individual_id;
              return childId === cid;
            })
          );

        const matchedFamily =
          (data.families ?? []).find(
            (f) =>
              isChildInFamily(f) &&
              (f.partner1_id === pid || f.partner2_id === pid)
          ) ?? (data.families ?? []).find((f) => isChildInFamily(f));

        const familyKey = matchedFamily?.id ?? `single-${pid}`;
        const busStagger = getFamilyBusStagger(familyKey);
        const yBus = childNode.y - V_GAP / 2 + busStagger;
        const sourceX = rawSourceX;
        const edgeId = `all-edge-${pid}-${cid}`;

        if (!createdChildEdges.has(edgeId)) {
          createdChildEdges.add(edgeId);
          compEdges.push(
            createPositionedEdge({
              id: edgeId,
              sourceId: pid,
              targetId: cid,
              familyId: matchedFamily?.id,
              kind: 'parent_child',
              sourceX,
              sourceY,
              targetX,
              targetY,
              yBus,
              pathD: buildVerticalBusPath(sourceX, sourceY, targetX, targetY, yBus),
              isDirectLineage: directLineageSet.has(pid) && directLineageSet.has(cid),
            })
          );
        }
      }
    }

    // Primary centering around X ≈ 0
    if (isPrimary) {
      let shiftX = 0;
      const focalPlaced = compNodes.find((n) => n.id === focalId);
      if (focalPlaced) {
        shiftX = -focalPlaced.x;
      } else if (compNodes.length > 0) {
        let minX = Infinity;
        let maxX = -Infinity;
        for (const n of compNodes) {
          if (n.x < minX) minX = n.x;
          if (n.x + n.width > maxX) maxX = n.x + n.width;
        }
        shiftX = -minX;
      }

      if (shiftX !== 0) {
        for (const n of compNodes) n.x += shiftX;
        for (const e of compEdges) {
          e.sourceX += shiftX;
          e.targetX += shiftX;
          e.midX += shiftX;
          if (e.yBus !== undefined) {
            e.pathD = buildVerticalBusPath(e.sourceX, e.sourceY, e.targetX, e.targetY, e.yBus);
          } else {
            e.pathD = buildSpouseConnectorPath(e.sourceX, e.sourceY, e.targetX, e.targetY);
          }
        }
      }
    }

    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;
    for (const n of compNodes) {
      if (n.x < minX) minX = n.x;
      if (n.x + n.width > maxX) maxX = n.x + n.width;
      if (n.y < minY) minY = n.y;
      if (n.y + n.height > maxY) maxY = n.y + n.height;
    }

    return {
      nodes: compNodes,
      edges: compEdges,
      minX: minX === Infinity ? 0 : minX,
      maxX: maxX === -Infinity ? CARD_WIDTH : maxX,
      minY: minY === Infinity ? 0 : minY,
      maxY: maxY === -Infinity ? CARD_HEIGHT : maxY,
    };
}
