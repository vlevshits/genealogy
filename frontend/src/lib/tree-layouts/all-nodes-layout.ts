import { TreePersonNodeDto, TreeViewResponse } from '../../api/client';
import {
  buildSpouseConnectorPath,
  buildVerticalBusPath,
} from './curve-routing';
import {
  buildTreeGraphIndex,
  CARD_HEIGHT,
  CARD_WIDTH,
  COMPONENT_GAP,
  computeLayoutBounds,
  H_GAP,
  LayoutComponentHeader,
  PlaceholderSlot,
  PositionedEdge,
  PositionedNode,
  ROW_STEP,
  SPOUSE_GAP,
  TreeLayoutResult,
  V_GAP,
  createPositionedEdge,
} from './types';

interface ComponentUnit {
  id: string;
  generation: number;
  personIds: string[];
  x: number;
  width: number;
  childrenUnitIds: string[];
  parentUnitIds: string[];
}

function extractBirthYear(node?: TreePersonNodeDto | null): number | null {
  if (!node?.birth?.date) return null;
  const match = node.birth.date.match(/\b(\d{4})\b/);
  return match ? parseInt(match[1], 10) : null;
}

export function computeAllNodesLayout(
  data: TreeViewResponse,
  options?: { focalIdOverride?: string | null }
): TreeLayoutResult {
  const { nodeMap, relativesMap, focalId: detectedFocalId } = buildTreeGraphIndex(data);
  const focalId = options?.focalIdOverride ?? detectedFocalId;

  const nodes: PositionedNode[] = [];
  const placeholders: PlaceholderSlot[] = [];
  const edges: PositionedEdge[] = [];
  const componentHeaders: LayoutComponentHeader[] = [];

  if (nodeMap.size === 0) {
    return {
      mode: 'all',
      focalId: null,
      nodes,
      placeholders,
      edges,
      bounds: computeLayoutBounds([], []),
      components: componentHeaders,
    };
  }

  // ---------------------------------------------------------------------------
  // 1. Direct Lineage Calculation for Focal Person
  // ---------------------------------------------------------------------------
  const directLineageSet = new Set<string>();
  if (focalId && nodeMap.has(focalId)) {
    directLineageSet.add(focalId);

    // Direct ancestors
    const collectAncestors = (currId: string) => {
      const rel = relativesMap.get(currId);
      if (!rel) return;
      for (const pid of rel.parentIds) {
        if (!directLineageSet.has(pid) && nodeMap.has(pid)) {
          directLineageSet.add(pid);
          collectAncestors(pid);
        }
      }
    };
    collectAncestors(focalId);

    // Direct descendants
    const collectDescendants = (currId: string) => {
      const rel = relativesMap.get(currId);
      if (!rel) return;
      for (const cid of rel.childIds) {
        if (!directLineageSet.has(cid) && nodeMap.has(cid)) {
          directLineageSet.add(cid);
          collectDescendants(cid);
        }
      }
    };
    collectDescendants(focalId);
  }

  // ---------------------------------------------------------------------------
  // 2. Connected Component Partitioning
  // ---------------------------------------------------------------------------
  const visitedGlobal = new Set<string>();
  const rawComponents: string[][] = [];

  for (const personId of nodeMap.keys()) {
    if (visitedGlobal.has(personId)) continue;

    const comp: string[] = [];
    const queue: string[] = [personId];
    visitedGlobal.add(personId);

    while (queue.length > 0) {
      const curr = queue.shift()!;
      comp.push(curr);

      const rel = relativesMap.get(curr);
      if (rel) {
        const neighbors = [
          ...rel.parentIds,
          ...rel.spouseIds,
          ...rel.childIds,
        ];
        for (const neighborId of neighbors) {
          if (!visitedGlobal.has(neighborId) && nodeMap.has(neighborId)) {
            visitedGlobal.add(neighborId);
            queue.push(neighborId);
          }
        }
      }
    }
    rawComponents.push(comp);
  }

  // Designate Primary Component (contains focalId, or largest component)
  let primaryIndex = -1;
  if (focalId && nodeMap.has(focalId)) {
    primaryIndex = rawComponents.findIndex((c) => c.includes(focalId));
  }
  if (primaryIndex === -1 && rawComponents.length > 0) {
    let maxLen = -1;
    rawComponents.forEach((c, idx) => {
      if (c.length > maxLen) {
        maxLen = c.length;
        primaryIndex = idx;
      }
    });
  }

  const primaryNodeIds = primaryIndex >= 0 ? rawComponents[primaryIndex] : [];
  const otherComponents: string[][] = [];
  const shelfNodeIds: string[] = [];

  rawComponents.forEach((comp, idx) => {
    if (idx === primaryIndex) return;
    if (comp.length === 1) {
      shelfNodeIds.push(comp[0]);
    } else {
      otherComponents.push(comp);
    }
  });

  // Sort Secondary Components deterministically:
  // 1. Descending node count
  // 2. Earliest recorded birth year
  // 3. Lexicographical GID/ID
  otherComponents.sort((a, b) => {
    if (b.length !== a.length) {
      return b.length - a.length;
    }
    const minBirthYear = (c: string[]) => {
      let earliest = Infinity;
      for (const id of c) {
        const yr = extractBirthYear(nodeMap.get(id));
        if (yr !== null && yr < earliest) earliest = yr;
      }
      return earliest;
    };
    const ea = minBirthYear(a);
    const eb = minBirthYear(b);
    if (ea !== eb) return ea - eb;

    const minGid = (c: string[]) => {
      const gids = c.map((id) => nodeMap.get(id)?.gid || id).sort();
      return gids[0] || '';
    };
    return minGid(a).localeCompare(minGid(b));
  });

  // ---------------------------------------------------------------------------
  // 3. Component Layout Routine
  // ---------------------------------------------------------------------------
  const focalNode = focalId ? nodeMap.get(focalId) : null;
  const focalBirthYear = extractBirthYear(focalNode);

  const layoutComponent = (
    compNodeIds: string[],
    isPrimary: boolean
  ): {
    nodes: PositionedNode[];
    edges: PositionedEdge[];
    minX: number;
    maxX: number;
    minY: number;
    maxY: number;
  } => {
    const compNodes: PositionedNode[] = [];
    const compEdges: PositionedEdge[] = [];
    const compSet = new Set(compNodeIds);

    // 3a. Assign Generations
    const genMap = new Map<string, number>();

    if (isPrimary && focalId && compSet.has(focalId)) {
      genMap.set(focalId, 0);
      const q = [focalId];
      while (q.length > 0) {
        const curr = q.shift()!;
        const currGen = genMap.get(curr)!;
        const rel = relativesMap.get(curr);
        if (!rel) continue;

        for (const pid of rel.parentIds) {
          if (compSet.has(pid) && !genMap.has(pid)) {
            genMap.set(pid, currGen - 1);
            q.push(pid);
          }
        }
        for (const cid of rel.childIds) {
          if (compSet.has(cid) && !genMap.has(cid)) {
            genMap.set(cid, currGen + 1);
            q.push(cid);
          }
        }
        for (const sid of rel.spouseIds) {
          if (compSet.has(sid) && !genMap.has(sid)) {
            genMap.set(sid, currGen);
            q.push(sid);
          }
        }
      }
    } else {
      // Disconnected component: identify topological roots (individuals with 0 parents in component)
      const roots = compNodeIds.filter((id) => {
        const rel = relativesMap.get(id);
        if (!rel) return true;
        const parentInComp = rel.parentIds.some((pid) => compSet.has(pid));
        return !parentInComp;
      });

      const effectiveRoots = roots.length > 0 ? roots : [compNodeIds[0]];

      // Chronological delta leveling
      const compYears: number[] = [];
      for (const id of compNodeIds) {
        const yr = extractBirthYear(nodeMap.get(id));
        if (yr !== null) compYears.push(yr);
      }
      let baseGen = 0;
      if (focalBirthYear !== null && compYears.length > 0) {
        compYears.sort((x, y) => x - y);
        const medianYear = compYears[Math.floor(compYears.length / 2)];
        baseGen = Math.round((medianYear - focalBirthYear) / 30);
      }

      const q: string[] = [];
      for (const r of effectiveRoots) {
        genMap.set(r, baseGen);
        q.push(r);
      }

      while (q.length > 0) {
        const curr = q.shift()!;
        const currGen = genMap.get(curr)!;
        const rel = relativesMap.get(curr);
        if (!rel) continue;

        for (const pid of rel.parentIds) {
          if (compSet.has(pid) && !genMap.has(pid)) {
            genMap.set(pid, currGen - 1);
            q.push(pid);
          }
        }
        for (const cid of rel.childIds) {
          if (compSet.has(cid) && !genMap.has(cid)) {
            genMap.set(cid, currGen + 1);
            q.push(cid);
          }
        }
        for (const sid of rel.spouseIds) {
          if (compSet.has(sid) && !genMap.has(sid)) {
            genMap.set(sid, currGen);
            q.push(sid);
          }
        }
      }
    }

    // Ensure all nodes in comp have a generation fallback
    for (const id of compNodeIds) {
      if (!genMap.has(id)) {
        genMap.set(id, 0);
      }
    }

    // 3b. Group into Couples and Single Units
    const unitMap = new Map<string, ComponentUnit>();
    const personToUnit = new Map<string, string>();
    const handledPersons = new Set<string>();

    for (const id of compNodeIds) {
      if (handledPersons.has(id)) continue;
      handledPersons.add(id);

      const gen = genMap.get(id)!;
      const rel = relativesMap.get(id);
      const spouseCandidates: string[] = [];

      // Check unions first for chronological ordering
      for (const u of rel?.unions ?? []) {
        const partner = u.partner1Id === id ? u.partner2Id : u.partner1Id;
        if (
          partner &&
          compSet.has(partner) &&
          !handledPersons.has(partner) &&
          genMap.get(partner) === gen
        ) {
          if (!spouseCandidates.includes(partner)) {
            spouseCandidates.push(partner);
            handledPersons.add(partner);
            if (spouseCandidates.length === 2) break;
          }
        }
      }

      // Check rel.spouseIds as fallback
      if (spouseCandidates.length < 2 && rel) {
        for (const sid of rel.spouseIds) {
          if (
            compSet.has(sid) &&
            !handledPersons.has(sid) &&
            genMap.get(sid) === gen &&
            !spouseCandidates.includes(sid)
          ) {
            spouseCandidates.push(sid);
            handledPersons.add(sid);
            if (spouseCandidates.length === 2) break;
          }
        }
      }

      let personIds: string[] = [];
      let width: number = CARD_WIDTH;
      let unitId: string = `unit-${id}`;

      if (spouseCandidates.length === 2) {
        // [S1, Person, S2]
        personIds = [spouseCandidates[0], id, spouseCandidates[1]];
        width = CARD_WIDTH * 3 + SPOUSE_GAP * 2;
        unitId = `unit-${spouseCandidates[0]}-${id}-${spouseCandidates[1]}`;
      } else if (spouseCandidates.length === 1) {
        // [Person, S1]
        personIds = [id, spouseCandidates[0]];
        width = CARD_WIDTH * 2 + SPOUSE_GAP;
        unitId = `unit-${id}-${spouseCandidates[0]}`;
      } else {
        personIds = [id];
        width = CARD_WIDTH;
        unitId = `unit-${id}`;
      }

      const unit: ComponentUnit = {
        id: unitId,
        generation: gen,
        personIds,
        x: 0,
        width,
        childrenUnitIds: [],
        parentUnitIds: [],
      };
      unitMap.set(unitId, unit);
      for (const pid of personIds) {
        personToUnit.set(pid, unitId);
      }
    }

    // Link Parent and Child units
    for (const unit of unitMap.values()) {
      const childUnitSet = new Set<string>();
      for (const pid of unit.personIds) {
        const rel = relativesMap.get(pid);
        if (!rel) continue;
        for (const cid of rel.childIds) {
          if (compSet.has(cid)) {
            const cuId = personToUnit.get(cid);
            if (cuId && cuId !== unit.id) {
              childUnitSet.add(cuId);
            }
          }
        }
      }
      unit.childrenUnitIds = Array.from(childUnitSet);
      for (const cuId of unit.childrenUnitIds) {
        const cu = unitMap.get(cuId);
        if (cu && !cu.parentUnitIds.includes(unit.id)) {
          cu.parentUnitIds.push(unit.id);
        }
      }
    }

    // 3c. Hierarchical Tree Placement (Bottom-up subtree widths + Top-down placement)
    const rootUnits = Array.from(unitMap.values()).filter(
      (u) => u.parentUnitIds.length === 0
    );
    if (rootUnits.length === 0 && unitMap.size > 0) {
      rootUnits.push(unitMap.values().next().value!);
    }

    // Sort root units deterministically by generation, then id
    rootUnits.sort((a, b) => a.generation - b.generation || a.id.localeCompare(b.id));

    const subtreeWidthMap = new Map<string, number>();
    const visitedSubtrees = new Set<string>();

    const calcSubtreeWidth = (unitId: string): number => {
      if (subtreeWidthMap.has(unitId)) return subtreeWidthMap.get(unitId)!;
      if (visitedSubtrees.has(unitId)) {
        const u = unitMap.get(unitId)!;
        return u.width;
      }
      visitedSubtrees.add(unitId);

      const u = unitMap.get(unitId)!;
      if (u.childrenUnitIds.length === 0) {
        subtreeWidthMap.set(unitId, u.width);
        return u.width;
      }

      let chWidth = 0;
      for (let i = 0; i < u.childrenUnitIds.length; i++) {
        const cid = u.childrenUnitIds[i];
        chWidth += calcSubtreeWidth(cid);
        if (i > 0) chWidth += H_GAP;
      }

      const totalW = Math.max(u.width, chWidth);
      subtreeWidthMap.set(unitId, totalW);
      return totalW;
    };

    for (const ru of rootUnits) {
      calcSubtreeWidth(ru.id);
    }

    const placedUnits = new Set<string>();
    const placeSubtree = (unitId: string, leftX: number) => {
      if (placedUnits.has(unitId)) return;
      placedUnits.add(unitId);

      const u = unitMap.get(unitId)!;
      const sw = subtreeWidthMap.get(unitId) ?? u.width;
      u.x = leftX + (sw - u.width) / 2;

      if (u.childrenUnitIds.length > 0) {
        const unplacedChildren = u.childrenUnitIds.filter(
          (cid) => !placedUnits.has(cid)
        );
        let chTotal = 0;
        for (let i = 0; i < unplacedChildren.length; i++) {
          const cid = unplacedChildren[i];
          chTotal += subtreeWidthMap.get(cid) ?? unitMap.get(cid)!.width;
          if (i > 0) chTotal += H_GAP;
        }

        let chCursor = leftX + (sw - chTotal) / 2;
        for (const cid of unplacedChildren) {
          const cw = subtreeWidthMap.get(cid) ?? unitMap.get(cid)!.width;
          placeSubtree(cid, chCursor);
          chCursor += cw + H_GAP;
        }
      }
    };

    let rootCursorX = 0;
    for (const ru of rootUnits) {
      const sw = subtreeWidthMap.get(ru.id) ?? ru.width;
      placeSubtree(ru.id, rootCursorX);
      rootCursorX += sw + H_GAP;
    }

    // Place any remaining unreached units
    for (const u of unitMap.values()) {
      if (!placedUnits.has(u.id)) {
        u.x = rootCursorX;
        rootCursorX += u.width + H_GAP;
        placedUnits.add(u.id);
      }
    }

    // 3d. Strict Generational Non-Overlap Sweep
    const unitsByGen = new Map<number, ComponentUnit[]>();
    for (const u of unitMap.values()) {
      const list = unitsByGen.get(u.generation) ?? [];
      list.push(u);
      unitsByGen.set(u.generation, list);
    }

    for (const list of unitsByGen.values()) {
      list.sort((a, b) => a.x - b.x);
      for (let i = 0; i < list.length - 1; i++) {
        const cur = list[i];
        const next = list[i + 1];
        const minReqX = cur.x + cur.width + H_GAP;
        if (next.x < minReqX) {
          const shift = minReqX - next.x;
          for (let k = i + 1; k < list.length; k++) {
            list[k].x += shift;
          }
        }
      }
    }

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
  };

  // ---------------------------------------------------------------------------
  // 4. Forest Assembly (Side-by-Side Placement)
  // ---------------------------------------------------------------------------
  let currentMaxRight = 0;

  if (primaryNodeIds.length > 0) {
    const primaryResult = layoutComponent(primaryNodeIds, true);
    nodes.push(...primaryResult.nodes);
    edges.push(...primaryResult.edges);
    currentMaxRight = primaryResult.maxX;

    componentHeaders.push({
      id: 'comp-primary',
      titleKey: 'tree.components.mainBranch',
      title: 'Main Family Branch',
      nodeCount: primaryNodeIds.length,
      x: primaryResult.minX,
      y: primaryResult.minY - 44,
      width: 180,
      height: 32,
      isPrimary: true,
    });
  }

  // Secondary components placed with COMPONENT_GAP
  otherComponents.forEach((compIds, idx) => {
    const res = layoutComponent(compIds, false);
    const shiftX = currentMaxRight + COMPONENT_GAP - res.minX;

    for (const n of res.nodes) n.x += shiftX;
    for (const e of res.edges) {
      e.sourceX += shiftX;
      e.targetX += shiftX;
      e.midX += shiftX;
      if (e.yBus !== undefined) {
        e.pathD = buildVerticalBusPath(e.sourceX, e.sourceY, e.targetX, e.targetY, e.yBus);
      } else {
        e.pathD = buildSpouseConnectorPath(e.sourceX, e.sourceY, e.targetX, e.targetY);
      }
    }

    const shiftedMinX = res.minX + shiftX;
    const shiftedMaxX = res.maxX + shiftX;

    nodes.push(...res.nodes);
    edges.push(...res.edges);
    currentMaxRight = shiftedMaxX;

    componentHeaders.push({
      id: `comp-${idx + 2}`,
      titleKey: 'tree.components.branch',
      title: `Branch ${idx + 2}`,
      nodeCount: compIds.length,
      x: shiftedMinX,
      y: res.minY - 44,
      width: 140,
      height: 32,
      isPrimary: false,
    });
  });

  // ---------------------------------------------------------------------------
  // 5. Isolated Records Shelf Grid (4 columns)
  // ---------------------------------------------------------------------------
  if (shelfNodeIds.length > 0) {
    let minAllX = 0;
    let maxAllY = 0;
    if (nodes.length > 0) {
      minAllX = Math.min(...nodes.map((n) => n.x));
      maxAllY = Math.max(...nodes.map((n) => n.y + n.height));
    }

    const shelfStartX = minAllX;
    const shelfStartY = maxAllY + 120; // Clear separation below all components

    shelfNodeIds.forEach((id, idx) => {
      const col = idx % 4;
      const row = Math.floor(idx / 4);
      const x = shelfStartX + col * (CARD_WIDTH + 24);
      const y = shelfStartY + row * (CARD_HEIGHT + 24);
      const dto = nodeMap.get(id)!;

      nodes.push({
        id,
        node: dto,
        x,
        y,
        width: CARD_WIDTH,
        height: CARD_HEIGHT,
        generation: 0,
        isFocal: id === focalId,
        isDirectLineage: false,
      });
    });

    componentHeaders.push({
      id: 'shelf-isolated',
      titleKey: 'tree.components.isolatedRecords',
      title: 'Isolated Records',
      nodeCount: shelfNodeIds.length,
      x: shelfStartX,
      y: shelfStartY - 44,
      width: 160,
      height: 32,
      isShelf: true,
    });
  }

  // ---------------------------------------------------------------------------
  // 6. Compute Final Bounds
  // ---------------------------------------------------------------------------
  const bounds = computeLayoutBounds(nodes, placeholders);
  if (componentHeaders.length > 0) {
    const minHeaderY = Math.min(...componentHeaders.map((c) => c.y));
    if (minHeaderY < bounds.minY) {
      bounds.minY = minHeaderY;
      bounds.height = bounds.maxY - bounds.minY;
    }
  }

  return {
    mode: 'all',
    focalId,
    nodes,
    placeholders,
    edges,
    bounds,
    components: componentHeaders,
  };
}
