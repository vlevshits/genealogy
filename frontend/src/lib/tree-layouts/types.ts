import {
  FamilyDto,
  TreeEdgeDto,
  TreePersonNodeDto,
  TreeProjectionMode,
  TreeViewResponse,
} from '../../api/client';

export const CARD_WIDTH = 240;
export const CARD_HEIGHT = 92;
export const H_GAP = 60;
export const V_GAP = 40;
export const SPOUSE_GAP = 32;

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
  pathD: string;
  isDirectLineage?: boolean;
}

export interface LayoutBounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
}

export interface TreeLayoutResult {
  mode: TreeProjectionMode;
  focalId: string | null;
  nodes: PositionedNode[];
  placeholders: PlaceholderSlot[];
  edges: PositionedEdge[];
  bounds: LayoutBounds;
}

export interface PersonRelatives {
  fatherId: string | null;
  motherId: string | null;
  parentIds: string[];
  spouseIds: string[];
  childIds: string[];
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

  // 1. Populate from structured families
  const families: FamilyDto[] = data.families ?? [];
  for (const fam of families) {
    const p1 = fam.partner1_id ?? null;
    const p2 = fam.partner2_id ?? null;
    if (p1 && p2) {
      linkSpouses(p1, p2);
    }
    for (const ch of fam.children ?? []) {
      const childId = ch.child_id || ch.individual_id;
      if (!childId) continue;
      if (p1) linkParentChild(p1, childId);
      if (p2) linkParentChild(p2, childId);
    }
  }

  // 2. Populate from explicit edges
  const edges: TreeEdgeDto[] = data.edges ?? [];
  for (const edge of edges) {
    if (edge.kind === 'spouse') {
      linkSpouses(edge.source, edge.target);
    } else if (edge.kind === 'parent_child') {
      linkParentChild(edge.source, edge.target);
    }
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
