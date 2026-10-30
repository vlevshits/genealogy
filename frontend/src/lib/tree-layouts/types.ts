import {
  FamilyDto,
  TreeEdgeDto,
  TreePersonNodeDto,
  TreeProjectionMode,
  TreeViewResponse,
} from '../../api/client';

export const CARD_WIDTH = 240;
export const CARD_HEIGHT = 120;
export const H_GAP = 60;
export const V_GAP = 60;
export const SPOUSE_GAP = 32;
export const COMPONENT_GAP = 300;
export const ROW_STEP = CARD_HEIGHT + V_GAP; // 180px

export interface PositionedNode {
  id: string;
  node: TreePersonNodeDto;
  x: number;
  y: number;
  width: number;
  height: number;
  generation: number;
  isFocal: boolean;
  isSpouseOfFocal?: boolean;
  isDirectLineage?: boolean;
}

export interface PlaceholderSlot {
  id: string;
  anchorId: string;
  relation: 'father' | 'mother';
  x: number;
  y: number;
  width: number;
  height: number;
  generation: number;
}

export interface PositionedEdge {
  id: string;
  sourceId: string;
  targetId: string;
  kind: 'spouse' | 'parent_child' | 'placeholder';
  familyId?: string;
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  midX: number;
  midY: number;
  yBus?: number;
  pathD: string;
  isDirectLineage?: boolean;
}

export function createPositionedEdge(edge: {
  id: string;
  sourceId: string;
  targetId: string;
  kind: 'spouse' | 'parent_child' | 'placeholder';
  familyId?: string;
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  yBus?: number;
  pathD: string;
  isDirectLineage?: boolean;
  midX?: number;
  midY?: number;
}): PositionedEdge {
  let midX = edge.midX;
  let midY = edge.midY;

  if (midX === undefined || midY === undefined) {
    if (edge.kind === 'spouse') {
      midX = Math.round((edge.sourceX + edge.targetX) / 2);
      midY = Math.round((edge.sourceY + edge.targetY) / 2);
    } else if (edge.yBus !== undefined) {
      if (Math.abs(edge.sourceX - edge.targetX) < 1) {
        midX = Math.round(edge.sourceX);
        midY = Math.round((edge.sourceY + edge.targetY) / 2);
      } else {
        midX = Math.round((edge.sourceX + edge.targetX) / 2);
        midY = Math.round(edge.yBus);
      }
    } else {
      midX = Math.round((edge.sourceX + edge.targetX) / 2);
      midY = Math.round((edge.sourceY + edge.targetY) / 2);
    }
  }

  return {
    ...edge,
    midX,
    midY,
  };
}

export interface LayoutBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
}

export interface LayoutComponentHeader {
  id: string;
  titleKey?: string;
  title?: string;
  nodeCount: number;
  x: number;
  y: number;
  width: number;
  height: number;
  isPrimary?: boolean;
  isShelf?: boolean;
}

export interface TreeLayoutResult {
  mode: TreeProjectionMode;
  focalId: string | null;
  nodes: PositionedNode[];
  placeholders: PlaceholderSlot[];
  edges: PositionedEdge[];
  bounds: LayoutBounds;
  components?: LayoutComponentHeader[];
}

export interface FamilyUnion {
  familyId: string;
  partner1Id: string | null;
  partner2Id: string | null;
  childIds: string[];
  marriageDate?: string | null;
  marriagePlace?: string | null;
}

export interface PersonRelatives {
  fatherId: string | null;
  motherId: string | null;
  parentIds: string[];
  spouseIds: string[];
  childIds: string[];
  unions: FamilyUnion[];
  singleParentChildIds: string[];
}

export interface TreeGraphIndex {
  nodeMap: Map<string, TreePersonNodeDto>;
  relativesMap: Map<string, PersonRelatives>;
  focalId: string | null;
}

export function computeLayoutBounds(
  nodes: PositionedNode[],
  placeholders: PlaceholderSlot[] = []
): LayoutBounds {
  const allItems = [...nodes, ...placeholders];
  if (allItems.length === 0) {
    return {
      minX: 0,
      minY: 0,
      maxX: CARD_WIDTH,
      maxY: CARD_HEIGHT,
      width: CARD_WIDTH,
      height: CARD_HEIGHT,
    };
  }

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const item of allItems) {
    if (item.x < minX) minX = item.x;
    if (item.y < minY) minY = item.y;
    if (item.x + item.width > maxX) maxX = item.x + item.width;
    if (item.y + item.height > maxY) maxY = item.y + item.height;
  }

  return {
    minX,
    minY,
    maxX,
    maxY,
    width: Math.max(CARD_WIDTH, maxX - minX),
    height: Math.max(CARD_HEIGHT, maxY - minY),
  };
}

function extractYearFromDate(dateStr?: string | null): number | null {
  if (!dateStr) return null;
  const match = dateStr.match(/\b(\d{4})\b/);
  return match ? parseInt(match[1], 10) : null;
}

export function buildTreeGraphIndex(data: TreeViewResponse): TreeGraphIndex {
  const nodeMap = new Map<string, TreePersonNodeDto>();
  const relativesMap = new Map<string, PersonRelatives>();

  for (const n of data.nodes) {
    nodeMap.set(n.id, n);
    relativesMap.set(n.id, {
      fatherId: null,
      motherId: null,
      parentIds: [],
      spouseIds: [],
      childIds: [],
      unions: [],
      singleParentChildIds: [],
    });
  }

  const ensureRel = (id: string): PersonRelatives | null => {
    const existing = relativesMap.get(id);
    return existing ?? null;
  };

  const linkParentChild = (parentId: string, childId: string) => {
    const parentNode = nodeMap.get(parentId);
    const parentRel = ensureRel(parentId);
    const childRel = ensureRel(childId);
    if (!parentNode || !parentRel || !childRel) return;

    if (!parentRel.childIds.includes(childId)) {
      parentRel.childIds.push(childId);
    }
    if (!childRel.parentIds.includes(parentId)) {
      childRel.parentIds.push(parentId);
    }

    if (childRel.fatherId === parentId || childRel.motherId === parentId) {
      return;
    }

    if (parentNode.sex === 'M' && !childRel.fatherId) {
      childRel.fatherId = parentId;
    } else if (parentNode.sex === 'F' && !childRel.motherId) {
      childRel.motherId = parentId;
    } else if (!childRel.fatherId && childRel.motherId !== parentId) {
      childRel.fatherId = parentId;
    } else if (!childRel.motherId && childRel.fatherId !== parentId) {
      childRel.motherId = parentId;
    }
  };

  const linkSpouses = (idA: string, idB: string) => {
    const relA = ensureRel(idA);
    const relB = ensureRel(idB);
    if (!relA || !relB || idA === idB) return;
    if (!relA.spouseIds.includes(idB)) relA.spouseIds.push(idB);
    if (!relB.spouseIds.includes(idA)) relB.spouseIds.push(idA);
  };

  const unionMap = new Map<string, FamilyUnion>();

  // 1. Populate from structured families
  const families: FamilyDto[] = data.families ?? [];
  for (const fam of families) {
    const p1 = fam.partner1_id ?? null;
    const p2 = fam.partner2_id ?? null;
    if (p1 && p2) {
      linkSpouses(p1, p2);
    }

    const childIds: string[] = [];
    for (const ch of fam.children ?? []) {
      const childId = ch.child_id || ch.individual_id;
      if (!childId) continue;
      if (!childIds.includes(childId)) {
        childIds.push(childId);
      }
      if (p1) linkParentChild(p1, childId);
      if (p2) linkParentChild(p2, childId);
    }

    const marriageEvent = fam.events?.find(
      (e) =>
        e.event_type?.toUpperCase() === 'MARR' ||
        e.event_type?.toLowerCase() === 'marriage'
    );
    const rawMarriageDate =
      fam.marriage_date ??
      marriageEvent?.date_text ??
      (marriageEvent?.date
        ? typeof marriageEvent.date === 'string'
          ? marriageEvent.date
          : marriageEvent.date.text ||
            (marriageEvent.date.start?.y ? String(marriageEvent.date.start.y) : null)
        : null);
    const marriageDate: string | null = rawMarriageDate ?? null;
    const marriagePlace: string | null =
      fam.marriage_place ?? marriageEvent?.place_full_name ?? null;

    const union: FamilyUnion = {
      familyId: fam.id,
      partner1Id: p1,
      partner2Id: p2,
      childIds,
      marriageDate,
      marriagePlace,
    };
    unionMap.set(fam.id, union);
  }

  // 2. Populate from explicit edges
  const edges: TreeEdgeDto[] = data.edges ?? [];
  for (const edge of edges) {
    if (edge.kind === 'spouse') {
      linkSpouses(edge.source, edge.target);
      const fid = edge.family_id || `fam-${edge.source}-${edge.target}`;
      if (!unionMap.has(fid)) {
        const existing = Array.from(unionMap.values()).find(
          (u) =>
            (u.partner1Id === edge.source && u.partner2Id === edge.target) ||
            (u.partner1Id === edge.target && u.partner2Id === edge.source)
        );
        if (!existing) {
          unionMap.set(fid, {
            familyId: fid,
            partner1Id: edge.source,
            partner2Id: edge.target,
            childIds: [],
            marriageDate: null,
            marriagePlace: null,
          });
        }
      }
    } else if (edge.kind === 'parent_child') {
      linkParentChild(edge.source, edge.target);
      if (edge.family_id && unionMap.has(edge.family_id)) {
        const u = unionMap.get(edge.family_id)!;
        if (!u.childIds.includes(edge.target)) {
          u.childIds.push(edge.target);
        }
      }
    }
  }

  // Helper for earliest child birth year
  const getEarliestChildYear = (union: FamilyUnion): number | null => {
    let earliest = Infinity;
    for (const cid of union.childIds) {
      const chNode = nodeMap.get(cid);
      if (!chNode) continue;
      const yr = chNode.birth_sort ?? extractYearFromDate(chNode.birth?.date);
      if (yr !== null && yr !== undefined && yr < earliest) {
        earliest = yr;
      }
    }
    return earliest === Infinity ? null : earliest;
  };

  // 3. Assign and sort unions & singleParentChildIds for each person
  for (const [pid, rel] of relativesMap.entries()) {
    const personUnions: FamilyUnion[] = [];
    for (const u of unionMap.values()) {
      if (u.partner1Id === pid || u.partner2Id === pid) {
        personUnions.push(u);
      }
    }

    // Sort chronologically: marriage date, then earliest child birth year, then partner id
    personUnions.sort((a, b) => {
      const yrA = extractYearFromDate(a.marriageDate);
      const yrB = extractYearFromDate(b.marriageDate);
      if (yrA !== null && yrB !== null && yrA !== yrB) {
        return yrA - yrB;
      }
      if (yrA !== null && yrB === null) return -1;
      if (yrA === null && yrB !== null) return 1;
      if (a.marriageDate && b.marriageDate && a.marriageDate !== b.marriageDate) {
        return a.marriageDate.localeCompare(b.marriageDate);
      }

      const childYrA = getEarliestChildYear(a);
      const childYrB = getEarliestChildYear(b);
      if (childYrA !== null && childYrB !== null && childYrA !== childYrB) {
        return childYrA - childYrB;
      }
      if (childYrA !== null && childYrB === null) return -1;
      if (childYrA === null && childYrB !== null) return 1;

      const partnerA = (a.partner1Id === pid ? a.partner2Id : a.partner1Id) ?? '';
      const partnerB = (b.partner1Id === pid ? b.partner2Id : b.partner1Id) ?? '';
      return partnerA.localeCompare(partnerB);
    });

    rel.unions = personUnions;

    // Order spouseIds to match union order
    const orderedSpouseIds: string[] = [];
    for (const u of personUnions) {
      const partner = u.partner1Id === pid ? u.partner2Id : u.partner1Id;
      if (partner && !orderedSpouseIds.includes(partner)) {
        orderedSpouseIds.push(partner);
      }
    }
    for (const sid of rel.spouseIds) {
      if (!orderedSpouseIds.includes(sid)) {
        orderedSpouseIds.push(sid);
      }
    }
    rel.spouseIds = orderedSpouseIds;

    // Strict single-parent children: not belonging to any of person's unions
    const unionChildren = new Set<string>();
    for (const u of personUnions) {
      for (const cid of u.childIds) {
        unionChildren.add(cid);
      }
    }
    rel.singleParentChildIds = rel.childIds.filter((cid) => !unionChildren.has(cid));
  }

  // Resolve focalId
  let focalId = data.focal_id ?? data.resolved_home_person_id ?? null;
  if (focalId && !nodeMap.has(focalId)) {
    // Check if focalId is a GID
    const byGid = data.nodes.find((n) => n.gid === focalId);
    focalId = byGid ? byGid.id : null;
  }
  if (!focalId && data.nodes.length > 0) {
    focalId = data.nodes[0].id;
  }

  return {
    nodeMap,
    relativesMap,
    focalId,
  };
}
