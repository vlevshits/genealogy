import { buildSpouseConnectorPath, buildVerticalBusPath } from "./curve-routing";
import { TreePersonNodeDto, TreeViewResponse } from "../../api/client";
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
  SPOUSE_GAP,
  TreeLayoutResult,
} from "./types";
import {
  ComponentUnit,
  renderAllNodesComponent,
} from "./all-nodes-component-renderer";

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


    return renderAllNodesComponent({
      data,
      compSet,
      isPrimary,
      focalId,
      directLineageSet,
      nodeMap,
      relativesMap,
      unitMap,
      personToUnit,
    });
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

