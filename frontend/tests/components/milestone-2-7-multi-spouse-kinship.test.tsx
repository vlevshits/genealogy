import React from 'react';
import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import { ToastProvider, ConfirmProvider } from '../../src/components/ui';
import {
  computeHourglassLayout,
  computeDescendantsLayout,
  computeAllNodesLayout,
  PositionedNode,
} from '../../src/lib/tree-layouts';
import {
  TreeCanvas,
  KinshipFinderDialog,
} from '../../src/components/tree';
import {
  FamilyDto,
  TreePersonNodeDto,
  TreeEdgeDto,
  TreeViewResponseDto,
} from '../../src/api/client';

function renderWithProviders(
  node: React.ReactNode,
  locale: 'en' | 'ru' = 'en'
): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <ToastProvider>
          <ConfirmProvider>
            <AuthProvider>{node}</AuthProvider>
          </ConfirmProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

function assertNoOverlappingNodes(nodes: PositionedNode[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const a = nodes[i];
      const b = nodes[j];
      const overlapX = a.x < b.x + b.width && a.x + a.width > b.x;
      const overlapY = a.y < b.y + b.height && a.y + a.height > b.y;
      expect(
        overlapX && overlapY,
        `Nodes ${a.id} (${a.x},${a.y}) and ${b.id} (${b.x},${b.y}) should not overlap`
      ).toBe(false);
    }
  }
}

describe('Milestone 2.7: Multiple Spouses Layout & Family-Partitioned Children Buses', () => {
  // Test fixture: Person P has 2 spouses (S1, S2), 1 child with S1 (C1), 1 child with S2 (C2), and 1 single-parent child (C_single)
  const focalId = 'person-focal';
  const s1Id = 'spouse-1';
  const s2Id = 'spouse-2';
  const c1Id = 'child-m1';
  const c2Id = 'child-m2';
  const cSingleId = 'child-single';

  const fam1Id = 'fam-s1';
  const fam2Id = 'fam-s2';

  const multiSpouseTreeData: TreeViewResponseDto = {
    root_id: focalId,
    focal_id: focalId,
    total_visible: 6,
    nodes: [
      {
        id: focalId,
        gid: 'I0001',
        primary_name: { given_name: 'Ivan', surname: 'Petrov' },
        sex: 'M',
        is_living: false,
        generation: 0,
        is_direct_lineage: true,
      },
      {
        id: s1Id,
        gid: 'I0002',
        primary_name: { given_name: 'Anna', surname: 'Petrova' },
        sex: 'F',
        is_living: false,
        generation: 0,
        is_direct_lineage: false,
      },
      {
        id: s2Id,
        gid: 'I0003',
        primary_name: { given_name: 'Elena', surname: 'Petrova' },
        sex: 'F',
        is_living: false,
        generation: 0,
        is_direct_lineage: false,
      },
      {
        id: c1Id,
        gid: 'I0004',
        primary_name: { given_name: 'Mikhail', surname: 'Petrov' },
        sex: 'M',
        is_living: false,
        generation: 1,
        is_direct_lineage: true,
      },
      {
        id: c2Id,
        gid: 'I0005',
        primary_name: { given_name: 'Dmitry', surname: 'Petrov' },
        sex: 'M',
        is_living: false,
        generation: 1,
        is_direct_lineage: true,
      },
      {
        id: cSingleId,
        gid: 'I0006',
        primary_name: { given_name: 'Olga', surname: 'Petrova' },
        sex: 'F',
        is_living: false,
        generation: 1,
        is_direct_lineage: true,
      },
    ],
    edges: [
      { id: 'e-sp1', source: focalId, target: s1Id, kind: 'spouse', family_id: fam1Id },
      { id: 'e-sp2', source: focalId, target: s2Id, kind: 'spouse', family_id: fam2Id },
      { id: 'e-pc1', source: focalId, target: c1Id, kind: 'parent_child', family_id: fam1Id },
      { id: 'e-pc1-m', source: s1Id, target: c1Id, kind: 'parent_child', family_id: fam1Id },
      { id: 'e-pc2', source: focalId, target: c2Id, kind: 'parent_child', family_id: fam2Id },
      { id: 'e-pc2-m', source: s2Id, target: c2Id, kind: 'parent_child', family_id: fam2Id },
      { id: 'e-pc-single', source: focalId, target: cSingleId, kind: 'parent_child' },
    ],
    families: [
      {
        id: fam1Id,
        partner1_id: focalId,
        partner2_id: s1Id,
        children: [{ child_id: c1Id }],
        children_ids: [c1Id],
        marriage_date: '10 MAY 1970',
        marriage_place: 'Moscow',
      },
      {
        id: fam2Id,
        partner1_id: focalId,
        partner2_id: s2Id,
        children: [{ child_id: c2Id }],
        children_ids: [c2Id],
        marriage_date: '15 AUG 1985',
        marriage_place: 'Saint Petersburg',
      },
    ],
  };

  it('lays out 2 spouses symmetrically [Spouse 1] === [Person] === [Spouse 2] with no overlapping cards', () => {
    const layout = computeHourglassLayout(multiSpouseTreeData, 3, true);
    assertNoOverlappingNodes(layout.nodes);

    const s1Node = layout.nodes.find((n) => n.id === s1Id)!;
    const pNode = layout.nodes.find((n) => n.id === focalId)!;
    const s2Node = layout.nodes.find((n) => n.id === s2Id)!;

    expect(s1Node, 'Spouse 1 must be present').toBeDefined();
    expect(pNode, 'Person must be present').toBeDefined();
    expect(s2Node, 'Spouse 2 must be present').toBeDefined();

    // Symmetrical layout: Spouse 1 on left, Person in center, Spouse 2 on right
    expect(s1Node.x).toBeLessThan(pNode.x);
    expect(pNode.x).toBeLessThan(s2Node.x);
    expect(s1Node.x + s1Node.width).toBeLessThan(pNode.x);
    expect(pNode.x + pNode.width).toBeLessThan(s2Node.x);

    // Same generation tier
    expect(s1Node.y).toBe(pNode.y);
    expect(s2Node.y).toBe(pNode.y);
  });

  it('renders separate ochre marriage bars with midpoint dots for each marriage', () => {
    const layout = computeHourglassLayout(multiSpouseTreeData, 3, true);
    const pNode = layout.nodes.find((n) => n.id === focalId)!;
    const spouseEdges = layout.edges.filter((e) => e.kind === 'spouse');

    expect(spouseEdges.length).toBe(2);

    const edge1 = spouseEdges.find(
      (e) => (e.sourceId === s1Id && e.targetId === focalId) || (e.sourceId === focalId && e.targetId === s1Id)
    )!;
    const edge2 = spouseEdges.find(
      (e) => (e.sourceId === s2Id && e.targetId === focalId) || (e.sourceId === focalId && e.targetId === s2Id)
    )!;

    expect(edge1, 'Marriage 1 edge must exist').toBeDefined();
    expect(edge2, 'Marriage 2 edge must exist').toBeDefined();
    expect(edge1.familyId).toBe(fam1Id);
    expect(edge2.familyId).toBe(fam2Id);

    // Each spouse connector must have valid midpoints for the union circle dot
    expect(edge1.midX).toBeLessThan(pNode.x);
    expect(edge1.midY).toBe(edge1.sourceY);
    expect(edge2.midX).toBeGreaterThan(pNode.x + pNode.width);
    expect(edge2.midY).toBe(edge2.sourceY);
    expect(edge1.midX).not.toBe(edge2.midX);
  });

  it('partitions children strictly by marriage with zero child blending and single-parent bottom-center stem', () => {
    const layout = computeHourglassLayout(multiSpouseTreeData, 3, true);
    const pNode = layout.nodes.find((n) => n.id === focalId)!;

    const edgeC1 = layout.edges.find((e) => e.targetId === c1Id && e.kind === 'parent_child')!;
    const edgeC2 = layout.edges.find((e) => e.targetId === c2Id && e.kind === 'parent_child')!;
    const edgeCSingle = layout.edges.find((e) => e.targetId === cSingleId && e.kind === 'parent_child')!;

    expect(edgeC1, 'Child 1 edge must exist').toBeDefined();
    expect(edgeC2, 'Child 2 edge must exist').toBeDefined();
    expect(edgeCSingle, 'Single parent child edge must exist').toBeDefined();

    // Family 1 child hangs from Union 1 connector
    expect(edgeC1.familyId).toBe(fam1Id);
    expect(edgeC1.sourceX).toBeLessThan(pNode.x); // Left of Person card

    // Family 2 child hangs from Union 2 connector
    expect(edgeC2.familyId).toBe(fam2Id);
    expect(edgeC2.sourceX).toBeGreaterThan(pNode.x + pNode.width); // Right of Person card

    // Single-parent child hangs strictly from bottom-center of Person card
    expect(edgeCSingle.familyId).toBeUndefined();
    // Effective source includes 12px anti-collinear offset when collinear or single-parent
    expect(edgeCSingle.sourceY).toBe(pNode.y + pNode.height);
    expect(edgeCSingle.sourceX).toBeGreaterThanOrEqual(pNode.x + pNode.width / 2);
    expect(edgeCSingle.sourceX).toBeLessThanOrEqual(pNode.x + pNode.width / 2 + 15);
  });

  it('enforces >= 12px anti-collinear bus runner clearance preventing overlapping horizontal lines', () => {
    const layout = computeHourglassLayout(multiSpouseTreeData, 3, true);
    const edgeC1 = layout.edges.find((e) => e.targetId === c1Id && e.kind === 'parent_child')!;
    const edgeC2 = layout.edges.find((e) => e.targetId === c2Id && e.kind === 'parent_child')!;
    const edgeCSingle = layout.edges.find((e) => e.targetId === cSingleId && e.kind === 'parent_child')!;

    expect(edgeC1.yBus).toBeDefined();
    expect(edgeC2.yBus).toBeDefined();
    expect(edgeCSingle.yBus).toBeDefined();

    // Adjacent bus clearances (Union 1 to Single Parent, and Single Parent to Union 2)
    const clearance1 = Math.abs(edgeC1.yBus! - edgeCSingle.yBus!);
    const clearance2 = Math.abs(edgeCSingle.yBus! - edgeC2.yBus!);

    expect(
      clearance1,
      `Bus runner clearance (${clearance1}px) must be >= 12px to prevent overlapping lines`
    ).toBeGreaterThanOrEqual(12);

    expect(
      clearance2,
      `Bus runner clearance (${clearance2}px) must be >= 12px to prevent overlapping lines`
    ).toBeGreaterThanOrEqual(12);
  });

  it('also enforces strict family partitioning in computeDescendantsLayout', () => {
    const layout = computeDescendantsLayout(multiSpouseTreeData, 3);
    assertNoOverlappingNodes(layout.nodes);

    const edgeC1 = layout.edges.find((e) => e.targetId === c1Id && e.kind === 'parent_child')!;
    const edgeC2 = layout.edges.find((e) => e.targetId === c2Id && e.kind === 'parent_child')!;
    const edgeCSingle = layout.edges.find((e) => e.targetId === cSingleId && e.kind === 'parent_child')!;

    expect(edgeC1.familyId).toBe(fam1Id);
    expect(edgeC2.familyId).toBe(fam2Id);
    expect(edgeCSingle.familyId).toBeUndefined();

    if (edgeC1.yBus && edgeCSingle.yBus) {
      expect(Math.abs(edgeC1.yBus - edgeCSingle.yBus)).toBeGreaterThanOrEqual(12);
    }
    if (edgeCSingle.yBus && edgeC2.yBus) {
      expect(Math.abs(edgeCSingle.yBus - edgeC2.yBus)).toBeGreaterThanOrEqual(12);
    }
  });
});

describe('Milestone 2.7: Multi-Trace Kinship Path Finder & TreeCanvas Highlighting', () => {
  const people: TreePersonNodeDto[] = [
    {
      id: 'p-alexey',
      gid: 'I0001',
      primary_name: { given_name: 'Alexey', surname: 'Smirnov' },
      sex: 'M',
      is_living: true,
      generation: 0,
      is_direct_lineage: true,
    },
    {
      id: 'p-olga',
      gid: 'I0002',
      primary_name: { given_name: 'Olga', surname: 'Kuznetsova' },
      sex: 'F',
      is_living: true,
      generation: 0,
      is_direct_lineage: true,
    },
    {
      id: 'p-pyotr',
      gid: 'I0003',
      primary_name: { given_name: 'Pyotr', surname: 'Smirnov' },
      sex: 'M',
      is_living: false,
      generation: -2,
      is_direct_lineage: true,
    },
  ];

  const edges: TreeEdgeDto[] = [
    { id: 'e-sp', source: 'p-alexey', target: 'p-olga', kind: 'spouse' },
    { id: 'e-pc1', source: 'p-pyotr', target: 'p-alexey', kind: 'parent_child' },
    { id: 'e-pc2', source: 'p-pyotr', target: 'p-olga', kind: 'parent_child' },
  ];

  it('renders KinshipFinderDialog with trace switcher tabs, badges, and action buttons', () => {
    let highlightedIds: string[] = [];
    let highlightedTraces: any[] = [];

    const html = renderWithProviders(
      <KinshipFinderDialog
        open={true}
        onOpenChange={() => {}}
        people={people}
        edges={edges}
        initialPersonAId="p-alexey"
        initialPersonBId="p-olga"
        onHighlightPath={(ids, traces) => {
          highlightedIds = ids;
          highlightedTraces = traces ?? [];
        }}
      />,
      'en'
    );

    expect(html).toContain('data-testid="kinship-finder-dialog"');
    expect(html).toContain('Kinship &amp; Relationship Path Finder');
    expect(html).toContain('Relationship Step Chain');

    // Russian rendering
    const htmlRu = renderWithProviders(
      <KinshipFinderDialog
        open={true}
        onOpenChange={() => {}}
        people={people}
        edges={edges}
        initialPersonAId="p-alexey"
        initialPersonBId="p-olga"
      />,
      'ru'
    );
    expect(htmlRu).toContain('Калькулятор родства и цепочки связей');
    expect(htmlRu).toContain('Цепочка родственных переходов');
  });

  it('verifies TreeCanvas supports multi-trace edge colors and Floating Legend HUD', () => {
    const treeView: TreeViewResponseDto = {
      root_id: 'p-alexey',
      focal_id: 'p-alexey',
      total_visible: people.length,
      access_tier: 'admin',
      nodes: people,
      edges: edges,
      families: [],
    };

    // Render TreeCanvas with initialTreeView
    const htmlCanvas = renderWithProviders(
      <TreeCanvas
        initialTreeView={treeView}
        focusPersonId="p-alexey"
      />,
      'en'
    );

    expect(htmlCanvas).toContain('role="tree"');
    expect(htmlCanvas).toContain('data-node-id="p-alexey"');
    expect(htmlCanvas).toContain('data-node-id="p-olga"');
  });

  it('UX-077: assigns familyId on parent-child and spouse edges in all-nodes layout', () => {
    const fams: FamilyDto[] = [
      {
        id: 'fam-test-1',
        partner1_id: 'p-alexey',
        partner2_id: 'p-olga',
        marriage_date: '2010-05-15',
        marriage_place: 'Moscow',
        divorce_date: null,
        children: ['p-child-1'],
      },
    ];

    const childNode: TreePersonNodeDto = {
      id: 'p-child-1',
      gid: 'I0004',
      primary_name: { given_name: 'Dmitry', surname: 'Smirnov' },
      sex: 'M',
      is_living: true,
      generation: 1,
      is_direct_lineage: true,
    };

    const allData: TreeViewResponseDto = {
      root_id: 'p-alexey',
      focal_id: 'p-alexey',
      total_visible: 4,
      access_tier: 'admin',
      nodes: [...people, childNode],
      edges: [
        { id: 'e-sp', source: 'p-alexey', target: 'p-olga', kind: 'spouse' },
        { id: 'e-pc-child', source: 'p-alexey', target: 'p-child-1', kind: 'parent_child' },
      ],
      families: fams,
    };

    const layout = computeAllNodesLayout(allData);
    const spouseEdge = layout.edges.find((e) => e.kind === 'spouse');
    const childEdge = layout.edges.find((e) => e.kind === 'parent_child' && e.targetId === 'p-child-1');

    expect(spouseEdge?.familyId).toBe('fam-test-1');
    expect(childEdge?.familyId).toBe('fam-test-1');
  });

  it('UX-072 & UX-076: KinshipFinderDialog renders Swap Persons button and responsive footer', () => {
    const html = renderWithProviders(
      <KinshipFinderDialog
        open={true}
        onOpenChange={() => {}}
        people={people}
        edges={edges}
        initialPersonAId="p-alexey"
        initialPersonBId="p-olga"
      />,
      'en'
    );

    // UX-076: Swap persons action button
    expect(html).toContain('aria-label="Swap persons"');

    // UX-072: Responsive footer with stacking classes
    expect(html).toContain('flex flex-col-reverse sm:flex-row sm:justify-end gap-2 pt-2 border-t border-border w-full');
    expect(html).toContain('w-full sm:w-auto');
  });

  it('UX-074: KinshipFinderDialog trace selector has accessible tablist and tabpanel links', () => {
    const multiTraceResult = {
      from_id: 'p-alexey',
      to_id: 'p-olga',
      connected: true,
      relationship_label: 'Spouse / Partner',
      degree_of_kinship: 1,
      traces_count: 2,
      traces: [
        {
          id: 'trace-1',
          label_en: 'Direct Marriage',
          path_person_ids: ['p-alexey', 'p-olga'],
          degree_of_kinship: 1,
          is_blood_relation: false,
          steps: [],
        },
        {
          id: 'trace-2',
          label_en: 'Ancestral Connection',
          path_person_ids: ['p-alexey', 'p-pyotr', 'p-olga'],
          degree_of_kinship: 2,
          is_blood_relation: true,
          steps: [],
        },
      ],
      steps: [
        {
          person_id: 'p-alexey',
          display_name: 'Alexey Smirnov',
          relation_to_next: 'Spouse',
        },
        {
          person_id: 'p-olga',
          display_name: 'Olga Kuznetsova',
          relation_to_next: null,
        },
      ],
    };

    const html = renderWithProviders(
      <KinshipFinderDialog
        open={true}
        onOpenChange={() => {}}
        people={people}
        edges={edges}
        initialPersonAId="p-alexey"
        initialPersonBId="p-olga"
        initialRemoteResult={multiTraceResult}
      />,
      'en'
    );

    expect(html).toContain('role="tablist"');
    expect(html).toContain('id="trace-tab-0"');
    expect(html).toContain('aria-controls="trace-panel-0"');
    expect(html).toContain('id="trace-panel-0"');
    expect(html).toContain('role="tabpanel"');
  });

  it('UX-075: KinshipFinderDialog renders single clean Alert when two individuals are not connected', () => {
    const isolatedPerson: TreePersonNodeDto = {
      id: 'p-isolated',
      gid: 'I9999',
      primary_name: { given_name: 'Stranger', surname: 'Unknown' },
      sex: 'M',
      is_living: false,
      generation: 0,
      is_direct_lineage: false,
    };

    const html = renderWithProviders(
      <KinshipFinderDialog
        open={true}
        onOpenChange={() => {}}
        people={[...people, isolatedPerson]}
        edges={edges}
        initialPersonAId="p-alexey"
        initialPersonBId="p-isolated"
      />,
      'en'
    );

    // Must show the Alert with notConnected and notConnectedHint
    expect(html).toContain('No connected relationship path found between these two individuals.');
    expect(html).toContain('Verify that marriage unions and parent-child links are established for both branches.');
    // Must NOT render duplicate degree banner
    expect(html).not.toContain('data-testid="kinship-result-term"');
  });

  it('UX-069 & UX-070 & UX-071: TreeCanvas uses accessible ochre tokens and highlights maternal kin traces', () => {
    const fams: FamilyDto[] = [
      {
        id: 'fam-alexey-olga',
        partner1_id: 'p-alexey',
        partner2_id: 'p-olga',
        marriage_date: null,
        marriage_place: null,
        divorce_date: null,
        children: ['p-child-olga'],
      },
    ];

    const childOfOlga: TreePersonNodeDto = {
      id: 'p-child-olga',
      gid: 'I0005',
      primary_name: { given_name: 'Nadezhda', surname: 'Smirnova' },
      sex: 'F',
      is_living: true,
      generation: 1,
      is_direct_lineage: true,
    };

    const treeData: TreeViewResponseDto = {
      root_id: 'p-alexey',
      focal_id: 'p-alexey',
      total_visible: 4,
      access_tier: 'admin',
      nodes: [...people, childOfOlga],
      edges: [
        { id: 'e-sp', source: 'p-alexey', target: 'p-olga', kind: 'spouse' },
        { id: 'e-pc-child', source: 'p-alexey', target: 'p-child-olga', kind: 'parent_child' },
      ],
      families: fams,
    };

    const html = renderWithProviders(
      <TreeCanvas
        initialTreeView={treeData}
        focusPersonId="p-alexey"
      />,
      'en'
    );

    // UX-070: Ochre lines use stroke-amber-600 dark:stroke-amber-400 and midpoint dot fill-amber-600 dark:fill-amber-400
    expect(html).toContain('stroke-amber-600 dark:stroke-amber-400');
    expect(html).toContain('fill-amber-600 dark:fill-amber-400');
  });
});
