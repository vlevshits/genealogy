import {
  TreePersonNodeDto,
  TreeProjectionMode,
  TreeViewResponse,
} from '../../api/client';
import { computeAllNodesLayout } from './all-nodes-layout';
import { computeDescendantsLayout } from './descendants-layout';
import { computeHourglassLayout } from './hourglass-layout';
import { computePedigreeLayout } from './pedigree-layout';
import { TreeLayoutOrientation, TreeLayoutResult } from './types';
import { transformLayoutToHorizontal } from './horizontal-transform';

export * from './types';
export * from './curve-routing';
export * from './pedigree-layout';
export * from './descendants-layout';
export * from './hourglass-layout';
export * from './all-nodes-layout';
export * from './horizontal-transform';

export function computeTreeLayout(
  data: TreeViewResponse,
  options?: {
    mode?: TreeProjectionMode;
    generations?: number;
    collateral?: boolean;
    orientation?: TreeLayoutOrientation;
  }
): TreeLayoutResult {
  const mode = (options?.mode ?? data.mode ?? 'hourglass') as TreeProjectionMode;
  const generations = options?.generations ?? data.generations ?? 4;
  const collateral = options?.collateral ?? data.collateral ?? true;
  const orientation = options?.orientation ?? 'vertical';

  if (mode === 'all') {
    return computeAllNodesLayout(data);
  }
  if (mode === 'pedigree') {
    const res = computePedigreeLayout(data, generations, 'vertical');
    return orientation === 'horizontal' ? transformLayoutToHorizontal(res) : res;
  }
  if (mode === 'descendant') {
    const res = computeDescendantsLayout(data, generations);
    return orientation === 'horizontal' ? transformLayoutToHorizontal(res) : res;
  }
  const hgRes = computeHourglassLayout(data, generations, collateral);
  return orientation === 'horizontal' ? transformLayoutToHorizontal(hgRes) : hgRes;
}

export function buildDefaultTreeViewData(
  accessTier: 'public_anonymous' | 'viewer' | 'editor' | 'admin' = 'admin',
  mode: TreeProjectionMode = 'hourglass'
): TreeViewResponse {
  const isAnonymous = accessTier === 'public_anonymous';
  const canSeePrivate = accessTier === 'editor' || accessTier === 'admin';

  const nodes: TreePersonNodeDto[] = [
    {
      id: 'node-deceased-1',
      gid: 'I0001',
      sex: 'M',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Alexander',
        patronymic: 'Mikhailovich',
        surname: 'Levshits',
      },
      birth: { date: '1912-03-19', place: 'Vitebsk' },
      death: { date: '1989-11-04', place: 'Leningrad' },
      avatar_url: null,
      notes: 'Archival founder record.',
    },
    {
      id: 'node-deceased-2',
      gid: 'I0002',
      sex: 'F',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Sofia',
        patronymic: 'Yakovlevna',
        surname: 'Levshits',
      },
      birth: { date: '1915-08-22', place: 'Minsk' },
      death: { date: '1996-02-11', place: 'Saint Petersburg' },
      avatar_url: null,
      notes: 'Matriarch of the Leningrad branch.',
    },
    {
      id: 'node-living-1',
      gid: 'I0042',
      sex: 'F',
      is_living: true,
      is_private: false,
      primary_name: {
        given_name: isAnonymous ? 'E.' : 'Elena',
        patronymic: isAnonymous ? null : 'Alexandrovna',
        surname: isAnonymous ? '' : 'Levshits',
      },
      birth: isAnonymous
        ? null
        : { date: '1994-06-14', place: 'Saint Petersburg, Russia' },
      death: null,
      avatar_url: null,
      notes: isAnonymous ? null : 'Living family member.',
    },
  ];

  if (canSeePrivate) {
    nodes.push({
      id: 'node-private-1',
      gid: 'I0099',
      sex: 'M',
      is_living: false,
      is_private: true,
      primary_name: {
        given_name: 'Nikolai',
        patronymic: 'Petrovich',
        surname: 'Morozov',
      },
      birth: { date: '1938-09-02', place: 'Pskov' },
      death: { date: '2004-01-17', place: 'Saint Petersburg' },
      avatar_url: null,
      notes: 'Confidential archival branch record.',
    });
  }

  return {
    access_tier: accessTier,
    tree_title: 'Levshits Family Archive',
    focal_id: 'node-deceased-1',
    mode,
    generations: 4,
    nodes,
    families: [
      {
        id: 'fam-1',
        gid: 'F0001',
        partner1_id: 'node-deceased-1',
        partner2_id: 'node-deceased-2',
        relationship_type: 'married',
        children: [
          {
            child_id: 'node-living-1',
            rel_to_partner1: 'birth',
            rel_to_partner2: 'birth',
            sort_order: 0,
          },
        ],
        events: [],
        is_private: false,
        version: 1,
      },
    ],
    edges: [
      {
        id: 'edge-sp-1',
        source: 'node-deceased-1',
        target: 'node-deceased-2',
        kind: 'spouse',
        family_id: 'fam-1',
      },
      {
        id: 'edge-pc-1',
        source: 'node-deceased-1',
        target: 'node-living-1',
        kind: 'parent_child',
        family_id: 'fam-1',
      },
      {
        id: 'edge-pc-2',
        source: 'node-deceased-2',
        target: 'node-living-1',
        kind: 'parent_child',
        family_id: 'fam-1',
      },
    ],
    resolved_home_person_id: 'node-deceased-1',
    total_visible: nodes.length,
  };
}
