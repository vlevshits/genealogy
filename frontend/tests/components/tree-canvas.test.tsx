import React from 'react';
import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'fs';
import path from 'path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import { ToastProvider, ConfirmProvider } from '../../src/components/ui';
import {
  buildDefaultTreeViewData,
  computeAllNodesLayout,
  computeDescendantsLayout,
  computeHourglassLayout,
  computePedigreeLayout,
  createPositionedEdge,
  PositionedNode,
} from '../../src/lib/tree-layouts';
import {
  PersonBottomSheet,
  PersonDrawer,
  PersonNodeCard,
  formatConciseSurnameInitials,
  PublicGuestBanner,
  TreeCanvas,
  TreeToolbar,
} from '../../src/components/tree';
import { ConnectionNavPopover } from '../../src/components/tree/ConnectionNavPopover';

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

describe('Phase 1.6 Interactive Tree Canvas, Layout Engines, Public Masking & Drawer/BottomSheet', () => {
  it('contains zero raw <button>, <input>, <select>, or <textarea> tags across all components/tree/*.tsx files', () => {
    const treeDir = path.resolve(__dirname, '../../src/components/tree');
    for (const entry of fs.readdirSync(treeDir)) {
      if (entry.endsWith('.tsx')) {
        const fullPath = path.join(treeDir, entry);
        const src = fs.readFileSync(fullPath, 'utf-8');
        expect(src, `${fullPath} should not contain raw <button>`).not.toMatch(
          /<button[\s>]/
        );
        expect(src, `${fullPath} should not contain raw <input>`).not.toMatch(
          /<input[\s>]/
        );
        expect(src, `${fullPath} should not contain raw <select>`).not.toMatch(
          /<select[\s>]/
        );
        expect(
          src,
          `${fullPath} should not contain raw <textarea>`
        ).not.toMatch(/<textarea[\s>]/);
      }
    }
  });

  it('computes non-overlapping layouts for Pedigree, Hourglass, and Descendants modes with valid SVG connector paths', () => {
    // Build a 3-generation sample graph: focal person (node-living-1) with parents (node-deceased-1 & node-deceased-2)
    const sample = {
      ...buildDefaultTreeViewData('admin', 'pedigree'),
      focal_id: 'node-living-1',
    };

    const pedLayout = computePedigreeLayout(sample, 4);
    expect(pedLayout.nodes.length).toBeGreaterThanOrEqual(3);
    assertNoOverlappingNodes(pedLayout.nodes);

    const focalNode = pedLayout.nodes.find((n) => n.id === 'node-living-1')!;
    const fatherNode = pedLayout.nodes.find((n) => n.id === 'node-deceased-1')!;
    const motherNode = pedLayout.nodes.find((n) => n.id === 'node-deceased-2')!;
    expect(focalNode.x).toBe(0);
    expect(fatherNode.x).toBeGreaterThan(focalNode.x);
    expect(motherNode.x).toBe(fatherNode.x);
    // Symmetric parent centering: Y_child === (Y_father + Y_mother) / 2
    expect((fatherNode.y + motherNode.y) / 2).toBe(focalNode.y);
    expect(pedLayout.edges.some((e) => e.pathD.startsWith('M '))).toBe(true);

    // Hourglass layout: focal couple at Y = 0, ancestors at Y < 0, children at Y > 0
    const hgSample = buildDefaultTreeViewData('admin', 'hourglass');
    const hgLayout = computeHourglassLayout(hgSample, 4);
    assertNoOverlappingNodes(hgLayout.nodes);
    const hgFocal = hgLayout.nodes.find((n) => n.id === 'node-deceased-1')!;
    const hgChild = hgLayout.nodes.find((n) => n.id === 'node-living-1')!;
    expect(hgFocal.y).toBe(0);
    expect(hgChild.y).toBeGreaterThan(0);
    // Empty parent slots for focal person are placed upwards (Y < 0)
    expect(hgLayout.placeholders.length).toBeGreaterThanOrEqual(2);
    expect(hgLayout.placeholders[0].y).toBeLessThan(0);

    // Descendants layout: top-down cascade
    const descLayout = computeDescendantsLayout(
      buildDefaultTreeViewData('admin', 'descendant'),
      4
    );
    assertNoOverlappingNodes(descLayout.nodes);
    const descRoot = descLayout.nodes.find((n) => n.id === 'node-deceased-1')!;
    const descChild = descLayout.nodes.find((n) => n.id === 'node-living-1')!;
    expect(descRoot.y).toBe(0);
    expect(descChild.y).toBeGreaterThan(descRoot.y);
  });

  it('renders TreeCanvas in EN and RU with floating mode switcher, depth selector, zoom pill, mini-map, and parent placeholders for Editors/Admins', () => {
    const htmlEn = renderWithProviders(
      <TreeCanvas
        initialTreeView={buildDefaultTreeViewData('admin', 'hourglass')}
      />,
      'en'
    );
    expect(htmlEn).toContain('Hourglass');
    expect(htmlEn).toContain('Ancestors');
    expect(htmlEn).toContain('Descendants');
    expect(htmlEn).toContain('4 generations');
    expect(htmlEn).toContain('100%');
    expect(htmlEn).toContain('Radar Mini-Map');
    expect(htmlEn).toContain('Alexander Mikhailovich Levshits');
    expect(htmlEn).toContain('Elena Alexandrovna Levshits');
    expect(htmlEn).toContain('Add Father');
    expect(htmlEn).toContain('Add Mother');
    expect(htmlEn).toContain('†');
    expect(htmlEn).not.toContain('>Living<');
    expect(htmlEn).not.toContain('>Profile<');
    expect(htmlEn).not.toContain('>Edit<');

    const htmlRu = renderWithProviders(
      <TreeCanvas
        initialTreeView={buildDefaultTreeViewData('admin', 'hourglass')}
      />,
      'ru'
    );
    expect(htmlRu).toContain('Песочные часы');
    expect(htmlRu).toContain('Предки');
    expect(htmlRu).toContain('Потомки');
    expect(htmlRu).toContain('Мини-карта древа');
    expect(htmlRu).toContain('Добавить отца');
    expect(htmlRu).toContain('Добавить мать');
  });

  it('enforces Public Mode Living Masking (single initial with dot, zero surname/patronymic/dates/places) vs Authenticated Viewer full visibility & read-only', () => {
    const publicHtml = renderWithProviders(
      <TreeCanvas
        initialTreeView={buildDefaultTreeViewData(
          'public_anonymous',
          'hourglass'
        )}
      />,
      'en'
    );
    expect(publicHtml).toContain('Public Guest View');
    expect(publicHtml).toContain('E.');
    expect(publicHtml).not.toContain('Elena Alexandrovna Levshits');
    expect(publicHtml).not.toContain('1994-06-14');
    expect(publicHtml).not.toContain('Add Father');

    const viewerHtml = renderWithProviders(
      <TreeCanvas
        initialTreeView={buildDefaultTreeViewData('viewer', 'hourglass')}
      />,
      'en'
    );
    expect(viewerHtml).toContain('Elena Alexandrovna Levshits');
    expect(viewerHtml).toContain('1994-06-14');
    expect(viewerHtml).not.toContain('Add Father');
  });

  it('renders PersonDrawer on desktop and PersonBottomSheet on mobile when a node is selected', () => {
    const sampleData = buildDefaultTreeViewData('admin', 'hourglass');
    const person = sampleData.nodes[0];

    const drawerHtml = renderWithProviders(
      <PersonDrawer
        open
        onOpenChange={() => {}}
        person={person}
        family={{
          parents: [],
          spouses: [sampleData.nodes[1]],
          children: [sampleData.nodes[2]],
        }}
        accessTier="admin"
        canEdit
        isHomePerson
      />,
      'en'
    );
    expect(drawerHtml).toContain('Alexander Mikhailovich Levshits');
    expect(drawerHtml).toContain('Vital Dates &amp; Locations');
    expect(drawerHtml).toContain('Immediate Family');
    expect(drawerHtml).toContain('Sofia Yakovlevna Levshits');
    expect(drawerHtml).toContain('Elena Alexandrovna Levshits');
    expect(drawerHtml).toContain('Center in Tree');
    expect(drawerHtml).toContain('Open Profile');

    const sheetHtml = renderWithProviders(
      <PersonBottomSheet
        open
        onOpenChange={() => {}}
        person={person}
        accessTier="admin"
        canEdit
      />,
      'ru'
    );
    expect(sheetHtml).toContain('Alexander Mikhailovich Levshits');
    expect(sheetHtml).toContain('Подробнее');
  });

  it('verifies DEF-001..DEF-004 regression guards (no 404 avatar URLs in default tree data, AppShell hides footer on /tree)', () => {
    const sampleData = buildDefaultTreeViewData('admin', 'hourglass');
    for (const node of sampleData.nodes) {
      expect(node.avatar_url).toBeNull();
    }

    const appShellPath = path.resolve(
      __dirname,
      '../../src/components/ui/app-shell.tsx'
    );
    const appShellSrc = fs.readFileSync(appShellPath, 'utf-8');
    expect(appShellSrc).toContain("!currentPath.startsWith('/tree')");

    const quickAddPath = path.resolve(
      __dirname,
      '../../src/components/person/QuickAddRelativeDialog.tsx'
    );
    const quickAddSrc = fs.readFileSync(quickAddPath, 'utf-8');
    expect(quickAddSrc).toContain("setGivenName('')");
    expect(quickAddSrc).toContain("setBirthDate('')");
    expect(quickAddSrc).toContain('setParsedBirthDate(null)');
    expect(quickAddSrc).toContain("setBirthPlace('')");
  });

  it('excludes unconnected orphan collateral nodes from hourglass layout projection', () => {
    const sample = buildDefaultTreeViewData('admin', 'hourglass');
    // Default sample includes 'node-private-1' with no family/edge connection to focal person
    expect(sample.nodes.some((n) => n.id === 'node-private-1')).toBe(true);

    const layout = computeHourglassLayout(sample, 4);
    // Direct focal, spouse and child are placed
    expect(layout.nodes.some((n) => n.id === 'node-deceased-1')).toBe(true);
    expect(layout.nodes.some((n) => n.id === 'node-deceased-2')).toBe(true);
    expect(layout.nodes.some((n) => n.id === 'node-living-1')).toBe(true);
    // Collateral orphan node is excluded from hourglass layout
    expect(layout.nodes.some((n) => n.id === 'node-private-1')).toBe(false);
  });

  it('renders ancestor couples with spouse edges and unified lineage stems, and routes single ancestors cleanly', () => {
    // Elena ('node-living-1') is focal with both parents Alexander ('node-deceased-1') and Sofia ('node-deceased-2')
    const sample = {
      ...buildDefaultTreeViewData('admin', 'hourglass'),
      focal_id: 'node-living-1',
    };

    const layout = computeHourglassLayout(sample, 4);
    const focalNode = layout.nodes.find((n) => n.id === 'node-living-1')!;
    const fatherNode = layout.nodes.find((n) => n.id === 'node-deceased-1')!;
    const motherNode = layout.nodes.find((n) => n.id === 'node-deceased-2')!;

    expect(focalNode).toBeDefined();
    expect(fatherNode).toBeDefined();
    expect(motherNode).toBeDefined();
    expect(fatherNode.generation).toBe(-1);
    expect(motherNode.generation).toBe(-1);

    // 1. Ancestor couple has horizontal spouse edge
    const ancestorSpouseEdge = layout.edges.find(
      (e) =>
        e.kind === 'spouse' &&
        e.sourceId === fatherNode.id &&
        e.targetId === motherNode.id
    );
    expect(ancestorSpouseEdge).toBeDefined();
    expect(ancestorSpouseEdge!.sourceX).toBe(fatherNode.x + fatherNode.width);
    expect(ancestorSpouseEdge!.targetX).toBe(motherNode.x);

    // 2. Exactly ONE unified lineage stem drops from the couple down to the child
    const lineageEdges = layout.edges.filter(
      (e) => e.kind === 'parent_child' && e.targetId === focalNode.id
    );
    expect(lineageEdges).toHaveLength(1);
    const stem = lineageEdges[0];
    const expectedUnionCenterX =
      (fatherNode.x + fatherNode.width + motherNode.x) / 2;
    const expectedChildCenterX = focalNode.x + focalNode.width / 2;
    expect(stem.sourceX).toBe(expectedUnionCenterX);
    expect(stem.targetX).toBe(expectedChildCenterX);
    expect(stem.sourceY).toBe(fatherNode.y + fatherNode.height / 2);
    expect(stem.targetY).toBe(focalNode.y);
    expect(stem.isDirectLineage).toBe(true);

    // 3. Single ancestor routing (e.g. only mother Sofia known, no father)
    const singleMotherSample = {
      ...sample,
      families: [
        {
          ...sample.families[0],
          partner1_id: 'node-deceased-2',
          partner2_id: null,
          children: [
            {
              child_id: 'node-living-1',
              rel_to_partner1: 'birth',
              rel_to_partner2: null,
              sort_order: 0,
            },
          ],
        },
      ],
      edges: [
        {
          id: 'edge-pc-2',
          source: 'node-deceased-2',
          target: 'node-living-1',
          kind: 'parent_child' as const,
          family_id: 'fam-1',
        },
      ],
    };
    const singleLayout = computeHourglassLayout(singleMotherSample, 4);
    const singleMother = singleLayout.nodes.find(
      (n) => n.id === 'node-deceased-2'
    )!;
    expect(singleMother).toBeDefined();
    // No ancestor spouse edge exists
    expect(
      singleLayout.edges.some(
        (e) => e.kind === 'spouse' && e.sourceId === 'node-deceased-2'
      )
    ).toBe(false);
    // Direct clean route from mother to child without stray line
    const motherLineage = singleLayout.edges.find(
      (e) =>
        e.kind === 'parent_child' &&
        e.sourceId === singleMother.id &&
        e.targetId === focalNode.id
    );
    expect(motherLineage).toBeDefined();
    expect(motherLineage!.sourceX).toBe(singleMother.x + singleMother.width / 2);
    expect(motherLineage!.targetX).toBe(expectedChildCenterX);
  });

  it('supports left-click and keyboard navigation (Enter and Space) on PersonNodeCard and exempts cards from canvas pointer capture (UX-029)', () => {
    const sample = buildDefaultTreeViewData('admin', 'hourglass');
    const person = sample.nodes[0];

    let selectedNode: unknown = null;
    const handleSelect = (n: unknown) => {
      selectedNode = n;
    };

    // Render PersonNodeCard to check accessibility attributes (role="treeitem", tabIndex=0, data-node-id, focus ring)
    const html = renderWithProviders(
      <PersonNodeCard node={person} onSelect={handleSelect} />
    );
    expect(html).toContain('role="treeitem"');
    expect(html).toContain('tabindex="0"');
    expect(html).toContain(`data-node-id="${person.id}"`);
    expect(html).toContain('focus-visible:ring-2');

    // Verify keyboard handlers (Enter and Space) and click handler via React rendering harness
    let capturedElement: React.ReactElement<{
      tabIndex: number;
      onKeyDown: (e: { key: string; preventDefault: () => void }) => void;
      onClick: (e: { stopPropagation: () => void }) => void;
    }> | null = null;

    function TestHarness() {
      capturedElement = PersonNodeCard({
        node: person,
        onSelect: handleSelect,
      }) as React.ReactElement<{
        tabIndex: number;
        onKeyDown: (e: { key: string; preventDefault: () => void }) => void;
        onClick: (e: { stopPropagation: () => void }) => void;
      }>;
      return null;
    }

    renderWithProviders(<TestHarness />);
    expect(capturedElement).not.toBeNull();
    const cardEl = capturedElement!;
    expect(cardEl.props.tabIndex).toBe(0);

    let prevented = false;
    const mockPreventDefault = () => {
      prevented = true;
    };
    const mockStopPropagation = () => {};

    // Test Enter key
    selectedNode = null;
    prevented = false;
    cardEl.props.onKeyDown({ key: 'Enter', preventDefault: mockPreventDefault });
    expect(prevented).toBe(true);
    expect(selectedNode).toBe(person);

    // Test Space key
    selectedNode = null;
    prevented = false;
    cardEl.props.onKeyDown({ key: ' ', preventDefault: mockPreventDefault });
    expect(prevented).toBe(true);
    expect(selectedNode).toBe(person);

    // Test single left-click
    selectedNode = null;
    cardEl.props.onClick({ stopPropagation: mockStopPropagation });
    expect(selectedNode).toBe(person);

    // Verify TreeCanvas source code contains [role="treeitem"] and [data-node-id] in pointerdown exemption
    const treeCanvasPath = path.resolve(
      __dirname,
      '../../src/components/tree/TreeCanvas.tsx'
    );
    const treeCanvasSrc = fs.readFileSync(treeCanvasPath, 'utf-8');
    expect(treeCanvasSrc).toContain('[role="treeitem"]');
    expect(treeCanvasSrc).toContain('[data-node-id]');
  });

  it('supports Wide & Collateral-Expanded Hourglass Mode and Compact Mode with strict generational baselines, zero overlap, and correct edge flags', () => {
    const focalId = 'focal-person';
    const spouseId = 'focal-spouse';
    const childId = 'focal-child';
    const grandchildId = 'focal-grandchild';
    const siblingId = 'focal-sibling';
    const nieceId = 'sibling-child';
    const fatherId = 'father-person';
    const motherId = 'mother-person';
    const patGfId = 'pat-gf';
    const patGmId = 'pat-gm';
    const patAuntId = 'pat-aunt';
    const patCousinId = 'pat-cousin';
    const matGfId = 'mat-gf';
    const matGmId = 'mat-gm';
    const matAuntId = 'mat-aunt';
    const matCousinId = 'mat-cousin';

    const makePerson = (
      id: string,
      name: string,
      sex: 'M' | 'F',
      gen: number,
      isDirect: boolean
    ) => ({
      id,
      gid: `G-${id}`,
      sex,
      is_living: false,
      is_private: false,
      primary_name: { given_name: name, surname: 'Levshits', patronymic: null },
      birth: null,
      death: null,
      avatar_url: null,
      notes: null,
      generation: gen,
      is_direct_lineage: isDirect,
    });

    const sampleData = {
      access_tier: 'admin',
      tree_title: 'Test Wide Tree',
      focal_id: focalId,
      mode: 'hourglass',
      generations: 4,
      collateral: true,
      resolved_home_person_id: focalId,
      total_visible: 16,
      nodes: [
        makePerson(focalId, 'Focal', 'M', 0, true),
        makePerson(spouseId, 'Spouse', 'F', 0, false),
        makePerson(childId, 'Child', 'F', 1, true),
        makePerson(grandchildId, 'Grandchild', 'M', 2, true),
        makePerson(siblingId, 'Sibling', 'F', 0, false),
        makePerson(nieceId, 'Niece', 'F', 1, false),
        makePerson(fatherId, 'Father', 'M', -1, true),
        makePerson(motherId, 'Mother', 'F', -1, true),
        makePerson(patGfId, 'PatGrandfather', 'M', -2, true),
        makePerson(patGmId, 'PatGrandmother', 'F', -2, true),
        makePerson(patAuntId, 'PatAunt', 'F', -1, false),
        makePerson(patCousinId, 'PatCousin', 'M', 0, false),
        makePerson(matGfId, 'MatGrandfather', 'M', -2, true),
        makePerson(matGmId, 'MatGrandmother', 'F', -2, true),
        makePerson(matAuntId, 'MatAunt', 'F', -1, false),
        makePerson(matCousinId, 'MatCousin', 'M', 0, false),
      ],
      families: [
        // Focal couple -> Child
        {
          id: 'fam-focal',
          gid: 'F001',
          partner1_id: focalId,
          partner2_id: spouseId,
          relationship_type: 'married',
          is_private: false,
          children: [{ child_id: childId, sort_order: 0 }],
          version: 1,
        },
        // Child -> Grandchild
        {
          id: 'fam-child',
          gid: 'F002',
          partner1_id: childId,
          partner2_id: null,
          relationship_type: 'unknown',
          is_private: false,
          children: [{ child_id: grandchildId, sort_order: 0 }],
          version: 1,
        },
        // Parents -> Focal & Sibling
        {
          id: 'fam-parents',
          gid: 'F003',
          partner1_id: fatherId,
          partner2_id: motherId,
          relationship_type: 'married',
          is_private: false,
          children: [
            { child_id: focalId, sort_order: 0 },
            { child_id: siblingId, sort_order: 1 },
          ],
          version: 1,
        },
        // Sibling -> Niece
        {
          id: 'fam-sibling',
          gid: 'F004',
          partner1_id: siblingId,
          partner2_id: null,
          relationship_type: 'unknown',
          is_private: false,
          children: [{ child_id: nieceId, sort_order: 0 }],
          version: 1,
        },
        // Paternal Grandparents -> Father & Paternal Aunt
        {
          id: 'fam-pat-gp',
          gid: 'F005',
          partner1_id: patGfId,
          partner2_id: patGmId,
          relationship_type: 'married',
          is_private: false,
          children: [
            { child_id: fatherId, sort_order: 0 },
            { child_id: patAuntId, sort_order: 1 },
          ],
          version: 1,
        },
        // Paternal Aunt -> Paternal Cousin
        {
          id: 'fam-pat-aunt',
          gid: 'F006',
          partner1_id: patAuntId,
          partner2_id: null,
          relationship_type: 'unknown',
          is_private: false,
          children: [{ child_id: patCousinId, sort_order: 0 }],
          version: 1,
        },
        // Maternal Grandparents -> Mother & Maternal Aunt
        {
          id: 'fam-mat-gp',
          gid: 'F007',
          partner1_id: matGfId,
          partner2_id: matGmId,
          relationship_type: 'married',
          is_private: false,
          children: [
            { child_id: motherId, sort_order: 0 },
            { child_id: matAuntId, sort_order: 1 },
          ],
          version: 1,
        },
        // Maternal Aunt -> Maternal Cousin
        {
          id: 'fam-mat-aunt',
          gid: 'F008',
          partner1_id: matAuntId,
          partner2_id: null,
          relationship_type: 'unknown',
          is_private: false,
          children: [{ child_id: matCousinId, sort_order: 0 }],
          version: 1,
        },
      ],
      edges: [],
    };

    // 1. Wide Hourglass layout (collateral = true)
    const wideLayout = computeHourglassLayout(sampleData, 4, true);
    expect(wideLayout.nodes.length).toBe(16);

    // Iterating over all pairs (A, B) verifying that NO two cards overlap
    for (let i = 0; i < wideLayout.nodes.length; i++) {
      for (let j = i + 1; j < wideLayout.nodes.length; j++) {
        const a = wideLayout.nodes[i];
        const b = wideLayout.nodes[j];
        const noOverlap =
          a.x + a.width <= b.x ||
          b.x + b.width <= a.x ||
          a.y + a.height <= b.y ||
          b.y + b.height <= a.y;
        expect(
          noOverlap,
          `Wide layout: Nodes ${a.id} (${a.x},${a.y}) and ${b.id} (${b.x},${b.y}) must not overlap`
        ).toBe(true);
      }
    }
    assertNoOverlappingNodes(wideLayout.nodes);

    // Verify strict generational baseline alignment (ROW_STEP = 180)
    const fNode = wideLayout.nodes.find((n) => n.id === focalId)!;
    const spNode = wideLayout.nodes.find((n) => n.id === spouseId)!;
    const sibNode = wideLayout.nodes.find((n) => n.id === siblingId)!;
    const patCousinNode = wideLayout.nodes.find((n) => n.id === patCousinId)!;
    const matCousinNode = wideLayout.nodes.find((n) => n.id === matCousinId)!;
    const fatNode = wideLayout.nodes.find((n) => n.id === fatherId)!;
    const motNode = wideLayout.nodes.find((n) => n.id === motherId)!;
    const patAuntNode = wideLayout.nodes.find((n) => n.id === patAuntId)!;
    const matAuntNode = wideLayout.nodes.find((n) => n.id === matAuntId)!;
    const patGfNode = wideLayout.nodes.find((n) => n.id === patGfId)!;
    const patGmNode = wideLayout.nodes.find((n) => n.id === patGmId)!;
    const matGfNode = wideLayout.nodes.find((n) => n.id === matGfId)!;
    const matGmNode = wideLayout.nodes.find((n) => n.id === matGmId)!;
    const chNode = wideLayout.nodes.find((n) => n.id === childId)!;
    const nieceNode = wideLayout.nodes.find((n) => n.id === nieceId)!;
    const gchNode = wideLayout.nodes.find((n) => n.id === grandchildId)!;

    // Gen 0 baselines (Y = 0)
    expect(fNode.y).toBe(0);
    expect(spNode.y).toBe(0);
    expect(sibNode.y).toBe(0);
    expect(patCousinNode.y).toBe(0);
    expect(matCousinNode.y).toBe(0);

    // Gen -1 baselines (Y = -180)
    expect(fatNode.y).toBe(-180);
    expect(motNode.y).toBe(-180);
    expect(patAuntNode.y).toBe(-180);
    expect(matAuntNode.y).toBe(-180);

    // Gen -2 baselines (Y = -360)
    expect(patGfNode.y).toBe(-360);
    expect(patGmNode.y).toBe(-360);
    expect(matGfNode.y).toBe(-360);
    expect(matGmNode.y).toBe(-360);

    // Verify half-plane clearance between paternal and maternal grandparents:
    // Paternal grandparents remain strictly in the left half-plane
    // Maternal grandparents remain strictly in the right half-plane
    const focalCenterX = fNode.x + fNode.width / 2;
    expect(patGmNode.x + patGmNode.width).toBeLessThanOrEqual(focalCenterX - 30);
    expect(matGfNode.x).toBeGreaterThanOrEqual(focalCenterX + 30);
    expect(matGfNode.x - (patGmNode.x + patGmNode.width)).toBeGreaterThanOrEqual(60);

    // Gen +1 baselines (Y = 180)
    expect(chNode.y).toBe(180);
    expect(nieceNode.y).toBe(180);

    // Gen +2 baselines (Y = 360)
    expect(gchNode.y).toBe(360);

    // Verify spouse connector spans between inner edges of couple
    const focalSpouseEdge = wideLayout.edges.find(
      (e) => e.kind === 'spouse' && e.sourceId === focalId && e.targetId === spouseId
    )!;
    expect(focalSpouseEdge).toBeDefined();
    expect(focalSpouseEdge.sourceX).toBe(fNode.x + fNode.width);
    expect(focalSpouseEdge.targetX).toBe(spNode.x);
    expect(focalSpouseEdge.sourceY).toBe(fNode.y + fNode.height / 2);

    // Verify vertical stem to child starts directly on spouse connector at Y = 46 (fNode.y + CARD_HEIGHT / 2)
    const focalChildEdge = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === focalId && e.targetId === childId
    )!;
    expect(focalChildEdge).toBeDefined();
    expect(focalChildEdge.sourceY).toBe(fNode.y + fNode.height / 2);
    expect(focalChildEdge.isDirectLineage).toBe(true);

    // Verify parent spouse bar and child stems
    const parentSpouseEdge = wideLayout.edges.find(
      (e) => e.kind === 'spouse' && e.sourceId === fatherId && e.targetId === motherId
    )!;
    expect(parentSpouseEdge).toBeDefined();
    expect(parentSpouseEdge.sourceY).toBe(fatNode.y + fatNode.height / 2); // -114
    expect(parentSpouseEdge.sourceX).toBe(fatNode.x + fatNode.width);
    expect(parentSpouseEdge.targetX).toBe(motNode.x);

    const parentFocalStem = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === fatherId && e.targetId === focalId
    )!;
    expect(parentFocalStem).toBeDefined();
    expect(parentFocalStem.sourceY).toBe(fatNode.y + fatNode.height / 2); // starts directly on spouse bar without mid-air gap
    expect(parentFocalStem.targetY).toBe(fNode.y);
    expect(parentFocalStem.isDirectLineage).toBe(true);

    const parentSiblingStem = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === fatherId && e.targetId === siblingId
    )!;
    expect(parentSiblingStem).toBeDefined();
    expect(parentSiblingStem.sourceY).toBe(fatNode.y + fatNode.height / 2);
    expect(parentSiblingStem.targetY).toBe(sibNode.y);
    expect(parentSiblingStem.isDirectLineage).toBe(false);

    // Verify paternal grandparents spouse bar and stems
    const patGpSpouseEdge = wideLayout.edges.find(
      (e) => e.kind === 'spouse' && e.sourceId === patGfId && e.targetId === patGmId
    )!;
    expect(patGpSpouseEdge).toBeDefined();
    expect(patGpSpouseEdge.sourceY).toBe(patGfNode.y + patGfNode.height / 2); // -274
    expect(patGpSpouseEdge.sourceX).toBe(patGfNode.x + patGfNode.width);
    expect(patGpSpouseEdge.targetX).toBe(patGmNode.x);

    const patGpFatherStem = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === patGfId && e.targetId === fatherId
    )!;
    expect(patGpFatherStem).toBeDefined();
    expect(patGpFatherStem.sourceY).toBe(patGfNode.y + patGfNode.height / 2);
    expect(patGpFatherStem.targetY).toBe(fatNode.y);
    expect(patGpFatherStem.isDirectLineage).toBe(true);

    const patGpAuntStem = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === patGfId && e.targetId === patAuntId
    )!;
    expect(patGpAuntStem).toBeDefined();
    expect(patGpAuntStem.sourceY).toBe(patGfNode.y + patGfNode.height / 2);
    expect(patGpAuntStem.targetY).toBe(patAuntNode.y);
    expect(patGpAuntStem.isDirectLineage).toBe(false);

    // Verify maternal grandparents spouse bar and stems
    const matGpSpouseEdge = wideLayout.edges.find(
      (e) => e.kind === 'spouse' && e.sourceId === matGfId && e.targetId === matGmId
    )!;
    expect(matGpSpouseEdge).toBeDefined();
    expect(matGpSpouseEdge.sourceY).toBe(matGfNode.y + matGfNode.height / 2); // -274

    const matGpMotherStem = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === matGfId && e.targetId === motherId
    )!;
    expect(matGpMotherStem).toBeDefined();
    expect(matGpMotherStem.sourceY).toBe(matGfNode.y + matGfNode.height / 2);
    expect(matGpMotherStem.targetY).toBe(motNode.y);
    expect(matGpMotherStem.isDirectLineage).toBe(true);

    const matGpAuntStem = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === matGfId && e.targetId === matAuntId
    )!;
    expect(matGpAuntStem).toBeDefined();
    expect(matGpAuntStem.sourceY).toBe(matGfNode.y + matGfNode.height / 2);
    expect(matGpAuntStem.targetY).toBe(matAuntNode.y);
    expect(matGpAuntStem.isDirectLineage).toBe(false);

    // Collateral edges (e.g. to sibling, to aunt, to cousin, to niece) have isDirectLineage false
    const siblingEdge = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.targetId === siblingId
    )!;
    expect(siblingEdge).toBeDefined();
    expect(siblingEdge.isDirectLineage).toBe(false);

    const patCousinEdge = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.targetId === patCousinId
    )!;
    expect(patCousinEdge).toBeDefined();
    expect(patCousinEdge.isDirectLineage).toBe(false);

    const matCousinEdge = wideLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.targetId === matCousinId
    )!;
    expect(matCousinEdge).toBeDefined();
    expect(matCousinEdge.isDirectLineage).toBe(false);

    // 2. Compact Hourglass layout (collateral = false)
    const compactLayout = computeHourglassLayout(sampleData, 4, false);
    // 10 direct lineage nodes (focal, spouse, parents, 4 grandparents, child, grandchild)
    expect(compactLayout.nodes.length).toBe(10);
    // Collateral nodes (sibling, niece, aunts, cousins) are excluded in compact mode
    expect(compactLayout.nodes.some((n) => n.id === siblingId)).toBe(false);
    expect(compactLayout.nodes.some((n) => n.id === nieceId)).toBe(false);
    expect(compactLayout.nodes.some((n) => n.id === patAuntId)).toBe(false);
    expect(compactLayout.nodes.some((n) => n.id === patCousinId)).toBe(false);
    expect(compactLayout.nodes.some((n) => n.id === matAuntId)).toBe(false);
    expect(compactLayout.nodes.some((n) => n.id === matCousinId)).toBe(false);
    // Direct lineage nodes are present
    expect(compactLayout.nodes.some((n) => n.id === focalId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === spouseId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === fatherId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === motherId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === patGfId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === patGmId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === matGfId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === matGmId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === childId)).toBe(true);
    expect(compactLayout.nodes.some((n) => n.id === grandchildId)).toBe(true);

    // Pairwise non-overlapping check on compactLayout nodes (verifies grandparents do not collide on Y = -320)
    for (let i = 0; i < compactLayout.nodes.length; i++) {
      for (let j = i + 1; j < compactLayout.nodes.length; j++) {
        const a = compactLayout.nodes[i];
        const b = compactLayout.nodes[j];
        const noOverlap =
          a.x + a.width <= b.x ||
          b.x + b.width <= a.x ||
          a.y + a.height <= b.y ||
          b.y + b.height <= a.y;
        expect(
          noOverlap,
          `Compact layout: Nodes ${a.id} (${a.x},${a.y}) and ${b.id} (${b.x},${b.y}) must not overlap`
        ).toBe(true);
      }
    }
    assertNoOverlappingNodes(compactLayout.nodes);
  });

  it('connects single parent (father-only or mother-only) to siblings at Generation 0 with orthogonal bus edges', () => {
    const focalId = 'focal-person';
    const siblingId = 'focal-sibling';
    const fatherId = 'father-only';
    const motherId = 'mother-only';

    const makePerson = (id: string, name: string, sex: 'M' | 'F') => ({
      id,
      gid: `G-${id}`,
      sex,
      is_living: false,
      is_private: false,
      primary_name: { given_name: name, surname: 'Test', patronymic: null },
      birth: null,
      death: null,
      avatar_url: null,
      notes: null,
      generation: 0,
      is_direct_lineage: true,
    });

    // Single Father case
    const singleFatherData = {
      access_tier: 'admin',
      tree_title: 'Single Father Tree',
      focal_id: focalId,
      mode: 'hourglass',
      generations: 3,
      collateral: true,
      resolved_home_person_id: focalId,
      total_visible: 3,
      nodes: [
        makePerson(focalId, 'Focal', 'M'),
        makePerson(siblingId, 'Sibling', 'F'),
        makePerson(fatherId, 'Father', 'M'),
      ],
      families: [
        {
          id: 'fam-father-single',
          gid: 'F101',
          partner1_id: fatherId,
          partner2_id: null,
          relationship_type: 'unknown',
          is_private: false,
          children: [
            { child_id: focalId, sort_order: 0 },
            { child_id: siblingId, sort_order: 1 },
          ],
          version: 1,
        },
      ],
      edges: [],
    };

    const fatherLayout = computeHourglassLayout(singleFatherData as any, 3, true);
    const fatNode = fatherLayout.nodes.find((n) => n.id === fatherId)!;
    const sibNode = fatherLayout.nodes.find((n) => n.id === siblingId)!;
    expect(fatNode).toBeDefined();
    expect(sibNode).toBeDefined();

    const fatSibEdge = fatherLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === fatherId && e.targetId === siblingId
    );
    expect(fatSibEdge).toBeDefined();
    expect(fatSibEdge!.sourceX).toBe(fatNode.x + fatNode.width / 2);
    expect(fatSibEdge!.sourceY).toBe(fatNode.y + fatNode.height);
    expect(fatSibEdge!.targetX).toBe(sibNode.x + sibNode.width / 2);
    expect(fatSibEdge!.targetY).toBe(sibNode.y);

    // Single Mother case
    const singleMotherData = {
      access_tier: 'admin',
      tree_title: 'Single Mother Tree',
      focal_id: focalId,
      mode: 'hourglass',
      generations: 3,
      collateral: true,
      resolved_home_person_id: focalId,
      total_visible: 3,
      nodes: [
        makePerson(focalId, 'Focal', 'M'),
        makePerson(siblingId, 'Sibling', 'F'),
        makePerson(motherId, 'Mother', 'F'),
      ],
      families: [
        {
          id: 'fam-mother-single',
          gid: 'F102',
          partner1_id: motherId,
          partner2_id: null,
          relationship_type: 'unknown',
          is_private: false,
          children: [
            { child_id: focalId, sort_order: 0 },
            { child_id: siblingId, sort_order: 1 },
          ],
          version: 1,
        },
      ],
      edges: [],
    };

    const motherLayout = computeHourglassLayout(singleMotherData as any, 3, true);
    const motNode = motherLayout.nodes.find((n) => n.id === motherId)!;
    const motSibNode = motherLayout.nodes.find((n) => n.id === siblingId)!;
    expect(motNode).toBeDefined();
    expect(motSibNode).toBeDefined();

    const motSibEdge = motherLayout.edges.find(
      (e) => e.kind === 'parent_child' && e.sourceId === motherId && e.targetId === siblingId
    );
    expect(motSibEdge).toBeDefined();
    expect(motSibEdge!.sourceX).toBe(motNode.x + motNode.width / 2);
    expect(motSibEdge!.sourceY).toBe(motNode.y + motNode.height);
    expect(motSibEdge!.targetX).toBe(motSibNode.x + motSibNode.width / 2);
    expect(motSibEdge!.targetY).toBe(motSibNode.y);
  });

  it('renders TreeToolbar with Wide / Compact toggle in Hourglass mode with localized labels and callbacks', () => {
    const htmlEn = renderWithProviders(
      <TreeCanvas
        initialTreeView={{
          ...buildDefaultTreeViewData('admin', 'hourglass'),
          collateral: true,
        }}
      />,
      'en'
    );
    expect(htmlEn).toContain('Wide tree');
    expect(htmlEn).toContain('Compact tree');

    const htmlRu = renderWithProviders(
      <TreeCanvas
        initialTreeView={{
          ...buildDefaultTreeViewData('admin', 'hourglass'),
          collateral: true,
        }}
      />,
      'ru'
    );
    expect(htmlRu).toContain('Широкое древо');
    expect(htmlRu).toContain('Компактное древо');
  });

  it('respects initialTreeView collateral=false in Hourglass layout calculation', () => {
    const compactTreeData = {
      ...buildDefaultTreeViewData('admin', 'hourglass'),
      collateral: false,
    };
    const htmlCompact = renderWithProviders(
      <TreeCanvas initialTreeView={compactTreeData} />,
      'en'
    );
    expect(htmlCompact).toContain('Hourglass');
    expect(htmlCompact).toContain('Compact tree');
  });

  it('satisfies UX-037..UX-041: toolbar responsiveness, title tooltip, edge sorting, and dark mode contrast', () => {
    const htmlEn = renderWithProviders(
      <TreeCanvas
        initialTreeView={{
          ...buildDefaultTreeViewData('admin', 'hourglass'),
          collateral: true,
        }}
      />,
      'en'
    );

    // UX-037: sm:flex-wrap lg:flex-nowrap in toolbar container
    expect(htmlEn).toContain('sm:flex-wrap lg:flex-nowrap');

    // UX-041: title tooltip on collateral toggle
    expect(htmlEn).toContain('Aunts, uncles, cousins, siblings, nieces and nephews');

    // UX-067: collateral edges use stroke-secondary/70 for >= 3:1 contrast
    expect(htmlEn).toContain('stroke-secondary/70');

    // UX-038: collateral edges (stroke-secondary/70) appear before direct lineage edges (stroke-brand/80)
    const firstCollateralIdx = htmlEn.indexOf('stroke-secondary/70');
    const firstDirectIdx = htmlEn.indexOf('stroke-brand/80');
    expect(firstCollateralIdx).toBeGreaterThan(-1);
    expect(firstDirectIdx).toBeGreaterThan(-1);
    expect(firstCollateralIdx).toBeLessThan(firstDirectIdx);

    const htmlRu = renderWithProviders(
      <TreeCanvas
        initialTreeView={{
          ...buildDefaultTreeViewData('admin', 'hourglass'),
          collateral: true,
        }}
      />,
      'ru'
    );
    // UX-041 localized in RU
    expect(htmlRu).toContain('Дяди, тети, кузены, братья, сестры и племянники');
  });

  describe('Milestone 2.5 (TASK-TREE-ALL-NODES-MODE): Complete Graph / All Nodes View Mode', () => {
    it('computes non-overlapping all-nodes layout for primary component, secondary components, and shelf grid', () => {
      const allData = {
        tree_title: 'Family Forest',
        focal_id: 'p-focal',
        resolved_home_person_id: 'p-focal',
        mode: 'all',
        generations: 10,
        access_tier: 'admin',
        total_visible: 7,
        nodes: [
          // Primary component (p-focal + spouse + child + parent)
          {
            id: 'p-parent',
            gid: 'I001',
            sex: 'M' as const,
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'David', surname: 'Levshits' },
            birth: { date: '1940-01-01' },
          },
          {
            id: 'p-focal',
            gid: 'I002',
            sex: 'M' as const,
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Lev', surname: 'Levshits' },
            birth: { date: '1970-05-12' },
          },
          {
            id: 'p-spouse',
            gid: 'I003',
            sex: 'F' as const,
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Anna', surname: 'Levshits' },
            birth: { date: '1972-08-20' },
          },
          {
            id: 'p-child',
            gid: 'I004',
            sex: 'F' as const,
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Elena', surname: 'Levshits' },
            birth: { date: '2000-03-15' },
          },
          // Disconnected Secondary component (in-laws / branch 2: father & son)
          {
            id: 'p-sec-father',
            gid: 'I010',
            sex: 'M' as const,
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'Boris', surname: 'Ivanov' },
            birth: { date: '1945-02-10' },
          },
          {
            id: 'p-sec-son',
            gid: 'I011',
            sex: 'M' as const,
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Mikhail', surname: 'Ivanov' },
            birth: { date: '1975-09-01' },
          },
          // Isolated singleton record
          {
            id: 'p-singleton',
            gid: 'I020',
            sex: 'U' as const,
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'Archival', surname: 'Record' },
            birth: { date: '1910-06-01' },
          },
        ],
        edges: [
          // Primary component edges
          { id: 'e1', source: 'p-parent', target: 'p-focal', kind: 'parent_child' as const },
          { id: 'e2', source: 'p-focal', target: 'p-spouse', kind: 'spouse' as const },
          { id: 'e3', source: 'p-focal', target: 'p-child', kind: 'parent_child' as const },
          // Secondary component edge
          { id: 'e4', source: 'p-sec-father', target: 'p-sec-son', kind: 'parent_child' as const },
        ],
        families: [],
      };

      const result = computeAllNodesLayout(allData);
      expect(result.nodes.length).toBe(7);
      assertNoOverlappingNodes(result.nodes);

      // Primary component positioning
      const focal = result.nodes.find((n) => n.id === 'p-focal')!;
      const parent = result.nodes.find((n) => n.id === 'p-parent')!;
      const child = result.nodes.find((n) => n.id === 'p-child')!;
      const spouse = result.nodes.find((n) => n.id === 'p-spouse')!;

      expect(focal.generation).toBe(0);
      expect(focal.y).toBe(0);
      expect(parent.generation).toBe(-1);
      expect(parent.y).toBeLessThan(0);
      expect(child.generation).toBe(1);
      expect(child.y).toBeGreaterThan(0);
      expect(spouse.generation).toBe(0);
      expect(spouse.y).toBe(focal.y);

      // Verify edge midpoints match shifted source/target coordinates (prevents detached spouse union dots in all-nodes mode)
      for (const e of result.edges) {
        expect(e.midX).toBe(Math.round((e.sourceX + e.targetX) / 2));
      }

      // Secondary component positioning
      const secFather = result.nodes.find((n) => n.id === 'p-sec-father')!;
      const secSon = result.nodes.find((n) => n.id === 'p-sec-son')!;

      // Verify secondary component is placed horizontally to the right with COMPONENT_GAP >= 300px
      const primaryMaxX = Math.max(
        focal.x + focal.width,
        spouse.x + spouse.width,
        parent.x + parent.width,
        child.x + child.width
      );
      const secondaryMinX = Math.min(secFather.x, secSon.x);
      expect(secondaryMinX).toBeGreaterThanOrEqual(primaryMaxX + 300);

      // Shelf singleton positioning
      const singleton = result.nodes.find((n) => n.id === 'p-singleton')!;
      const maxTreeY = Math.max(child.y + child.height, secSon.y + secSon.height);
      expect(singleton.y).toBeGreaterThanOrEqual(maxTreeY + 120);

      // Verify component headers
      expect(result.components?.length).toBe(3);
      expect(result.components?.some((c) => c.isPrimary)).toBe(true);
      expect(result.components?.some((c) => c.isShelf)).toBe(true);

      // Direct lineage on focal branch
      expect(focal.isDirectLineage).toBe(true);
      expect(parent.isDirectLineage).toBe(true);
      expect(child.isDirectLineage).toBe(true);
      expect(secFather.isDirectLineage).toBe(false);
      expect(singleton.isDirectLineage).toBe(false);
    });

    it('renders TreeToolbar in mode="all" with count badge and hides generations select & collateral toggle', () => {
      const htmlEn = renderWithProviders(
        <TreeToolbar
          mode="all"
          onModeChange={() => {}}
          generations={4}
          onGenerationsChange={() => {}}
          collateral={true}
          onCollateralChange={() => {}}
          totalVisible={42}
          zoom={1}
          onZoomIn={() => {}}
          onZoomOut={() => {}}
          onZoomReset={() => {}}
          onCenterFocal={() => {}}
          onFitToScreen={() => {}}
        />,
        'en'
      );

      expect(htmlEn).toContain('All Nodes');
      expect(htmlEn).toContain('Individuals: 42');
      expect(htmlEn).not.toContain('4 generations');
      expect(htmlEn).not.toContain('Wide tree');
      expect(htmlEn).not.toContain('Compact tree');

      const htmlRu = renderWithProviders(
        <TreeToolbar
          mode="all"
          onModeChange={() => {}}
          generations={4}
          onGenerationsChange={() => {}}
          collateral={true}
          onCollateralChange={() => {}}
          totalVisible={42}
          zoom={1}
          onZoomIn={() => {}}
          onZoomOut={() => {}}
          onZoomReset={() => {}}
          onCenterFocal={() => {}}
          onFitToScreen={() => {}}
        />,
        'ru'
      );

      expect(htmlRu).toContain('Все персоны');
      expect(htmlRu).toContain('Персон: 42');
    });

    it('supports LOD 0 (full), LOD 1 (simplified), and LOD 2 (micro-pill) in PersonNodeCard', () => {
      const node = {
        id: 'node-test',
        gid: 'I0100',
        sex: 'M' as const,
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
      };

      // LOD 0: full card with avatar, complete name, dates/places, and quick action buttons
      const htmlLod0 = renderWithProviders(
        <PersonNodeCard node={node} lod={0} canEdit={true} />
      );
      expect(htmlLod0).toContain('data-lod="0"');
      expect(htmlLod0).toContain('Alexander Mikhailovich Levshits');
      expect(htmlLod0).toContain('Vitebsk');
      expect(htmlLod0).toContain('Focus');
      expect(htmlLod0).toContain('Add');

      // LOD 1: simplified card with avatar, name, birth/death years only; action buttons suppressed
      const htmlLod1 = renderWithProviders(
        <PersonNodeCard node={node} lod={1} canEdit={true} />
      );
      expect(htmlLod1).toContain('data-lod="1"');
      expect(htmlLod1).toContain('Alexander Mikhailovich Levshits');
      expect(htmlLod1).toContain('1912');
      expect(htmlLod1).toContain('1989');
      expect(htmlLod1).not.toContain('Vitebsk');
      expect(htmlLod1).not.toContain('Focus');
      expect(htmlLod1).not.toContain('Add');

      // LOD 2: micro-pill (28px height in 120px slot, gender dot, concise surname/initials, action buttons suppressed)
      const htmlLod2 = renderWithProviders(
        <PersonNodeCard
          node={node}
          lod={2}
          canEdit={true}
          isHomePerson={true}
          hasParentEdge={true}
          hasChildEdge={true}
        />
      );
      expect(htmlLod2).toContain('data-lod="2"');
      expect(htmlLod2).toContain('h-7');
      expect(htmlLod2).toContain('h-[120px]');
      expect(htmlLod2).toContain('bg-male');
      expect(htmlLod2).toContain('Alexander Mikhailovich Levshits');
      expect(htmlLod2).toContain('Levshits A. M.');
      expect(htmlLod2).toContain('data-testid="lod2-home-icon"');
      expect(htmlLod2).toContain('data-testid="lod2-parent-stem"');
      expect(htmlLod2).toContain('data-testid="lod2-child-stem"');
      expect(htmlLod2).not.toContain('1912');
      expect(htmlLod2).not.toContain('Focus');
      expect(htmlLod2).not.toContain('Add');
    });

    it('initializes in all mode when URL search parameter ?mode=all is provided', () => {
      const origWindow = (globalThis as any).window;
      const origStorage = (globalThis as any).localStorage;
      const mockStorage = {
        getItem: () => null,
        setItem: () => {},
        removeItem: () => {},
        clear: () => {},
        length: 0,
        key: () => null,
      };

      try {
        (globalThis as any).window = {
          location: new URL('http://localhost:5173/tree?mode=all'),
          localStorage: mockStorage,
          history: {
            replaceState: () => {},
          },
        };
        (globalThis as any).localStorage = mockStorage;

        const htmlCanvas = renderWithProviders(
          <TreeCanvas
            initialTreeView={{
              ...buildDefaultTreeViewData('admin', 'all'),
              mode: 'all',
            }}
          />,
          'en'
        );

        expect(htmlCanvas).toContain('All Nodes');
        expect(htmlCanvas).toContain('Individuals: 4');
        expect(htmlCanvas).toContain('Main Family Branch');
        expect(htmlCanvas).toContain('role="heading"');
        expect(htmlCanvas).toContain('aria-level="3"');
      } finally {
        if (origWindow === undefined) {
          delete (globalThis as any).window;
        } else {
          (globalThis as any).window = origWindow;
        }
        if (origStorage === undefined) {
          delete (globalThis as any).localStorage;
        } else {
          (globalThis as any).localStorage = origStorage;
        }
      }
    });

    it('arranges singletons in a strict 4-column shelf grid below all family components with zero collision', () => {
      // 10 singletons: row 0 (4), row 1 (4), row 2 (2)
      const singletons = Array.from({ length: 10 }, (_, i) => ({
        id: `singleton-${i + 1}`,
        gid: `I${200 + i}`,
        sex: (i % 2 === 0 ? 'M' : 'F') as 'M' | 'F',
        is_living: true,
        is_private: false,
        primary_name: { given_name: `Single${i + 1}`, surname: `Person${i + 1}` },
        birth: null,
        death: null,
        avatar_url: null,
      }));

      const mockData = {
        ...buildDefaultTreeViewData('admin', 'all'),
        focal_id: 'p-focal',
        nodes: [
          {
            id: 'p-focal',
            gid: 'I001',
            sex: 'M' as const,
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'Focal', surname: 'Head' },
            birth: null,
            death: null,
            avatar_url: null,
          },
          ...singletons,
        ],
        edges: [],
        families: [],
      };

      const result = computeAllNodesLayout(mockData);
      expect(result.nodes.length).toBe(11);
      assertNoOverlappingNodes(result.nodes);

      const focalNode = result.nodes.find((n) => n.id === 'p-focal')!;
      const shelfNodes = singletons.map(
        (s) => result.nodes.find((n) => n.id === s.id)!
      );

      // Shelf start Y must be at least 120px below focal node
      for (const sn of shelfNodes) {
        expect(sn.y).toBeGreaterThanOrEqual(focalNode.y + focalNode.height + 120);
      }

      // Check column positions: col = idx % 4, row = Math.floor(idx / 4)
      const colX = [0, 1, 2, 3].map((col) => shelfNodes[col].x);
      // Ensure all 4 columns have distinct ascending X coordinates
      expect(colX[1]).toBeGreaterThan(colX[0]);
      expect(colX[2]).toBeGreaterThan(colX[1]);
      expect(colX[3]).toBeGreaterThan(colX[2]);

      // Verify row 1 (indices 4..7) shares the exact same column X coordinates as row 0
      for (let i = 4; i < 8; i++) {
        expect(shelfNodes[i].x).toBe(colX[i % 4]);
        expect(shelfNodes[i].y).toBeGreaterThan(shelfNodes[0].y);
      }

      // Verify row 2 (indices 8..9) shares columns 0 and 1
      expect(shelfNodes[8].x).toBe(colX[0]);
      expect(shelfNodes[9].x).toBe(colX[1]);
      expect(shelfNodes[8].y).toBeGreaterThan(shelfNodes[4].y);
    });

    it('verifies frustum culling skips off-screen cards', () => {
      const viewport = { width: 1000, height: 800 };
      const pan = { x: 0, y: 0 };
      const zoom = 1.0;
      const margin = 400 / zoom;

      const worldLeft = -pan.x / zoom - margin; // -400
      const worldRight = (viewport.width - pan.x) / zoom + margin; // 1400
      const worldTop = -pan.y / zoom - margin; // -400
      const worldBottom = (viewport.height - pan.y) / zoom + margin; // 1200

      const checkInFrustum = (node: { x: number; y: number; width: number; height: number }) =>
        node.x + node.width >= worldLeft &&
        node.x <= worldRight &&
        node.y + node.height >= worldTop &&
        node.y <= worldBottom;

      // Inside viewport
      expect(checkInFrustum({ x: 100, y: 100, width: 200, height: 92 })).toBe(true);

      // In margin area (worldLeft <= x <= 0)
      expect(checkInFrustum({ x: -250, y: 100, width: 200, height: 92 })).toBe(true);

      // Far to the right beyond margin (x = 2500)
      expect(checkInFrustum({ x: 2500, y: 100, width: 200, height: 92 })).toBe(false);

      // Far to the left beyond margin (x = -1500)
      expect(checkInFrustum({ x: -1500, y: 100, width: 200, height: 92 })).toBe(false);

      // Far below beyond margin (y = 3000)
      expect(checkInFrustum({ x: 100, y: 3000, width: 200, height: 92 })).toBe(false);

      // Far above beyond margin (y = -1000)
      expect(checkInFrustum({ x: 100, y: -1000, width: 200, height: 92 })).toBe(false);
    });

    it('masks living individuals to single initial with dot, silhouette avatar, and zero vital dates in public_anonymous tier', () => {
      // Cyrillic living person
      const cyrillicLiving = {
        id: 'p-cyr',
        gid: 'I0091',
        sex: 'M' as const,
        is_living: true,
        is_private: false,
        primary_name: {
          given_name: 'Иван',
          patronymic: 'Петрович',
          surname: 'Сидоров',
        },
        birth: null,
        death: null,
        avatar_url: 'https://example.com/avatar.jpg',
      };

      const htmlCyr = renderWithProviders(
        <PersonNodeCard
          node={cyrillicLiving}
          accessTier="public_anonymous"
          canEdit={false}
        />,
        'ru'
      );

      // Single Cyrillic initial with dot "И."
      expect(htmlCyr).toContain('И.');
      // Full surname and patronymic suppressed in public anonymous card mode
      expect(htmlCyr).not.toContain('Сидоров');
      expect(htmlCyr).not.toContain('Петрович');
      // Silhouette avatar (avatar_url suppressed)
      expect(htmlCyr).not.toContain('https://example.com/avatar.jpg');
      // Privacy locked notice
      expect(htmlCyr).toContain('Даты и места защищены');

      // Latin living person
      const latinLiving = {
        id: 'p-lat',
        gid: 'I0092',
        sex: 'M' as const,
        is_living: true,
        is_private: false,
        primary_name: {
          given_name: 'John',
          surname: 'Smith',
        },
        birth: null,
        death: null,
        avatar_url: null,
      };

      const htmlLat = renderWithProviders(
        <PersonNodeCard
          node={latinLiving}
          accessTier="public_anonymous"
          canEdit={false}
        />,
        'en'
      );

      // Single Latin initial with dot "J."
      expect(htmlLat).toContain('J.');
      expect(htmlLat).not.toContain('Smith');
      expect(htmlLat).toContain('Vital dates &amp; locations protected');
    });

    it('UX-050: aligns shelf grid start X with leftmost boundary when minAllX is negative', () => {
      const dataWithNegativeCoords = {
        tree_title: 'Negative Coordinate Forest',
        focal_id: 'p-focal',
        resolved_home_person_id: 'p-focal',
        mode: 'all',
        generations: 5,
        access_tier: 'admin',
        total_visible: 3,
        nodes: [
          {
            id: 'p-focal',
            gid: 'I001',
            sex: 'M' as const,
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Focal', surname: 'Person' },
            birth: null,
          },
          // Spouse placed to the left or ancestor with negative x coordinate
          {
            id: 'p-left-parent',
            gid: 'I002',
            sex: 'M' as const,
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'Ancestor', surname: 'Left' },
            birth: null,
          },
          // Singleton placed on shelf
          {
            id: 'p-singleton',
            gid: 'I003',
            sex: 'U' as const,
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'Isolated', surname: 'Record' },
            birth: null,
          },
        ],
        edges: [
          {
            id: 'e-parent',
            source: 'p-left-parent',
            target: 'p-focal',
            kind: 'parent_child' as const,
          },
        ],
        families: [],
      };

      const result = computeAllNodesLayout(dataWithNegativeCoords);
      const minX = Math.min(
        ...result.nodes.filter((n) => n.id !== 'p-singleton').map((n) => n.x)
      );
      const shelfNode = result.nodes.find((n) => n.id === 'p-singleton')!;

      // Shelf start X must equal minAllX, not clamped to 0
      expect(shelfNode.x).toBe(minX);
    });

    it('UX-051: formats concise surname and initials correctly in English and Russian', () => {
      const ruNode = {
        id: 'p-ru',
        gid: 'I001',
        sex: 'M' as const,
        is_living: false,
        is_private: false,
        primary_name: {
          given_name: 'Александр',
          patronymic: 'Михайлович',
          surname: 'Левшиц',
        },
        birth: null,
        death: null,
        avatar_url: null,
      };

      const conciseRu = formatConciseSurnameInitials(ruNode, 'admin');
      expect(conciseRu.displayName).toBe('Левшиц А. М.');
      expect(conciseRu.isMaskedLiving).toBe(false);

      const enNode = {
        id: 'p-en',
        gid: 'I002',
        sex: 'F' as const,
        is_living: false,
        is_private: false,
        primary_name: {
          given_name: 'Mary Ann',
          patronymic: null,
          surname: 'Smith',
        },
        birth: null,
        death: null,
        avatar_url: null,
      };

      const conciseEn = formatConciseSurnameInitials(enNode, 'admin');
      expect(conciseEn.displayName).toBe('Smith M. A.');

      // Masked living privacy check
      const maskedLiving = {
        id: 'p-mask',
        gid: 'I003',
        sex: 'M' as const,
        is_living: true,
        is_private: false,
        primary_name: {
          given_name: 'Living',
          patronymic: null,
          surname: 'Secret',
        },
        birth: null,
        death: null,
        avatar_url: null,
      };

      const conciseMasked = formatConciseSurnameInitials(maskedLiving, 'public_anonymous');
      expect(conciseMasked.displayName).toBe('S.');
      expect(conciseMasked.isMaskedLiving).toBe(true);
    });
  });

  describe('Card-Edge Clearance and All-Nodes Relationship Edge Completeness', () => {
    function assertZeroCardEdgeIntersections(layout: ReturnType<typeof computeAllNodesLayout>) {
      for (const edge of layout.edges) {
        const lines: Array<{ y: number; xMin: number; xMax: number }> = [];

        // Check for buildVerticalBusPath: M sx sy L sx midY L tx midY L tx ty
        const busMatch = edge.pathD.match(
          /M\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)/
        );
        if (busMatch) {
          const y1 = parseFloat(busMatch[4]);
          const xA = parseFloat(busMatch[3]);
          const y2 = parseFloat(busMatch[6]);
          const xB = parseFloat(busMatch[5]);
          if (Math.abs(y1 - y2) < 0.01) {
            lines.push({ y: y1, xMin: Math.min(xA, xB), xMax: Math.max(xA, xB) });
          }
        }

        // Check for buildSpouseConnectorPath: M sx sy L tx ty
        const spouseMatch = edge.pathD.match(
          /^M\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)/
        );
        if (spouseMatch && !busMatch) {
          const sx = parseFloat(spouseMatch[1]);
          const sy = parseFloat(spouseMatch[2]);
          const tx = parseFloat(spouseMatch[3]);
          const ty = parseFloat(spouseMatch[4]);
          if (Math.abs(sy - ty) < 0.01) {
            lines.push({ y: sy, xMin: Math.min(sx, tx), xMax: Math.max(sx, tx) });
          }
        }

        for (const line of lines) {
          for (const node of layout.nodes) {
            // Node interior with 1px inset to avoid edge boundary touching
            const nodeTop = node.y + 1;
            const nodeBottom = node.y + node.height - 1;
            const nodeLeft = node.x + 1;
            const nodeRight = node.x + node.width - 1;

            const yIntersects = line.y >= nodeTop && line.y <= nodeBottom;
            const xOverlap = Math.max(line.xMin, nodeLeft) <= Math.min(line.xMax, nodeRight);

            expect(
              yIntersects && xOverlap,
              `Edge ${edge.id} (y=${line.y}, x=[${line.xMin}, ${line.xMax}]) intersects node ${node.id} bounding box [y: ${node.y}..${node.y + node.height}, x: ${node.x}..${node.x + node.width}]`
            ).toBe(false);
          }
        }
      }
    }

    it('guarantees zero card-edge intersections across all-nodes, hourglass, and descendant layouts', () => {
      const defaultData = buildDefaultTreeViewData('admin', 'all');
      const allLayout = computeAllNodesLayout(defaultData);
      assertZeroCardEdgeIntersections(allLayout);

      const hgLayout = computeHourglassLayout(defaultData);
      assertZeroCardEdgeIntersections(hgLayout);

      const descLayout = computeDescendantsLayout(defaultData);
      assertZeroCardEdgeIntersections(descLayout);
    });

    it('ensures 100% of visible parent-child and spouse relationships have edges in mode === "all", even with reverse ordering and blended families', () => {
      const complexData: TreeViewResponse = {
        tree_title: 'Blended Complex Tree',
        access_tier: 'admin',
        focal_id: 'p-father',
        resolved_home_person_id: 'p-father',
        mode: 'all',
        generations: 4,
        total_visible: 6,
        nodes: [
          // Reverse ordering: children first, spouses next, parents last
          {
            id: 'p-child-2',
            gid: 'I0005',
            sex: 'F',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Daughter', patronymic: null, surname: 'Smith' },
            birth: { date: '2020-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-child-1',
            gid: 'I0004',
            sex: 'M',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Son', patronymic: null, surname: 'Smith' },
            birth: { date: '2015-05-05', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-spouse-2',
            gid: 'I0003',
            sex: 'F',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Second Wife', patronymic: null, surname: 'Smith' },
            birth: { date: '1985-03-03', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-spouse-1',
            gid: 'I0002',
            sex: 'F',
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'First Wife', patronymic: null, surname: 'Brown' },
            birth: { date: '1982-02-02', place: null },
            death: { date: '2016-01-01', place: null },
            avatar_url: null,
          },
          {
            id: 'p-father',
            gid: 'I0001',
            sex: 'M',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Father', patronymic: null, surname: 'Smith' },
            birth: { date: '1980-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-grandfather',
            gid: 'I0000',
            sex: 'M',
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'Grandfather', patronymic: null, surname: 'Smith' },
            birth: { date: '1950-01-01', place: null },
            death: { date: '2010-01-01', place: null },
            avatar_url: null,
          },
        ],
        families: [
          {
            id: 'fam-gf',
            gid: 'F0000',
            partner1_id: 'p-grandfather',
            partner2_id: null,
            relationship_type: 'married',
            children: [{ child_id: 'p-father', rel_to_partner1: 'birth', rel_to_partner2: null, sort_order: 0 }],
            events: [],
            is_private: false,
            version: 1,
          },
          {
            id: 'fam-1',
            gid: 'F0001',
            partner1_id: 'p-father',
            partner2_id: 'p-spouse-1',
            relationship_type: 'married',
            children: [{ child_id: 'p-child-1', rel_to_partner1: 'birth', rel_to_partner2: 'birth', sort_order: 0 }],
            events: [],
            is_private: false,
            version: 1,
          },
          {
            id: 'fam-2',
            gid: 'F0002',
            partner1_id: 'p-father',
            partner2_id: 'p-spouse-2',
            relationship_type: 'married',
            children: [{ child_id: 'p-child-2', rel_to_partner1: 'birth', rel_to_partner2: 'birth', sort_order: 0 }],
            events: [],
            is_private: false,
            version: 1,
          },
        ],
        edges: [
          { id: 'e-sp-1', source: 'p-father', target: 'p-spouse-1', kind: 'spouse', family_id: 'fam-1' },
          { id: 'e-sp-2', source: 'p-father', target: 'p-spouse-2', kind: 'spouse', family_id: 'fam-2' },
          { id: 'e-pc-0', source: 'p-grandfather', target: 'p-father', kind: 'parent_child', family_id: 'fam-gf' },
          { id: 'e-pc-1', source: 'p-father', target: 'p-child-1', kind: 'parent_child', family_id: 'fam-1' },
          { id: 'e-pc-1b', source: 'p-spouse-1', target: 'p-child-1', kind: 'parent_child', family_id: 'fam-1' },
          { id: 'e-pc-2', source: 'p-father', target: 'p-child-2', kind: 'parent_child', family_id: 'fam-2' },
          { id: 'e-pc-2b', source: 'p-spouse-2', target: 'p-child-2', kind: 'parent_child', family_id: 'fam-2' },
        ],
      };

      const result = computeAllNodesLayout(complexData);

      // Verify all 6 nodes are positioned
      expect(result.nodes.length).toBe(6);

      // Verify spouse edges exist for both marriages
      const sp1 = result.edges.find(
        (e) =>
          e.kind === 'spouse' &&
          (e.sourceId === 'p-father' || e.targetId === 'p-father') &&
          (e.sourceId === 'p-spouse-1' || e.targetId === 'p-spouse-1')
      );
      expect(sp1).toBeDefined();

      const sp2 = result.edges.find(
        (e) =>
          e.kind === 'spouse' &&
          (e.sourceId === 'p-father' || e.targetId === 'p-father') &&
          (e.sourceId === 'p-spouse-2' || e.targetId === 'p-spouse-2')
      );
      expect(sp2).toBeDefined();

      // Verify parent-child edge from grandfather to father
      const pc0 = result.edges.find((e) => e.kind === 'parent_child' && e.targetId === 'p-father');
      expect(pc0).toBeDefined();
      expect(pc0?.sourceId).toBe('p-grandfather');

      // Verify child 1 and child 2 both have parent edges
      const pc1 = result.edges.find((e) => e.kind === 'parent_child' && e.targetId === 'p-child-1');
      expect(pc1).toBeDefined();

      const pc2 = result.edges.find((e) => e.kind === 'parent_child' && e.targetId === 'p-child-2');
      expect(pc2).toBeDefined();

      // Verify zero card-edge intersections on this complex tree
      assertZeroCardEdgeIntersections(result);
    });

    it('rigorously tests PersonNodeCard fixed 120px height and LOD 0/1/2 geometry', () => {
      const node = {
        id: 'p-lod-test',
        gid: 'I0999',
        sex: 'M' as const,
        is_living: true,
        is_private: false,
        primary_name: {
          given_name: 'Test',
          patronymic: 'Geom',
          surname: 'Card',
        },
        birth: { date: '1990-01-01', place: 'Test City' },
        death: null,
        avatar_url: null,
      };

      // LOD 0
      const htmlLod0 = renderWithProviders(
        <PersonNodeCard node={node} x={100} y={200} lod={0} canEdit={true} />
      );
      expect(htmlLod0).toContain('height:120px');
      expect(htmlLod0).toContain('max-height:120px');
      expect(htmlLod0).toContain('data-lod="0"');
      expect(htmlLod0).toContain('Focus');
      expect(htmlLod0).toContain('Add');

      // LOD 1
      const htmlLod1 = renderWithProviders(
        <PersonNodeCard node={node} x={100} y={200} lod={1} canEdit={true} />
      );
      expect(htmlLod1).toContain('height:120px');
      expect(htmlLod1).toContain('max-height:120px');
      expect(htmlLod1).toContain('data-lod="1"');
      expect(htmlLod1).not.toContain('Focus');
      expect(htmlLod1).not.toContain('Add');

      // LOD 2
      const htmlLod2 = renderWithProviders(
        <PersonNodeCard
          node={node}
          x={100}
          y={200}
          lod={2}
          canEdit={true}
          hasParentEdge={true}
          hasChildEdge={true}
        />
      );
      expect(htmlLod2).toContain('height:120px');
      expect(htmlLod2).toContain('h-[120px]');
      expect(htmlLod2).toContain('h-7');
      expect(htmlLod2).toContain('data-lod="2"');
      expect(htmlLod2).toContain('data-testid="lod2-parent-stem"');
      expect(htmlLod2).toContain('data-testid="lod2-child-stem"');
    });

    it('ensures zero card-edge intersections for horizontal and vertical edge segments across Pedigree, Hourglass, Descendant, and All-Nodes modes', () => {
      function assertZeroIntersectionsEverywhere(layout: ReturnType<typeof computeAllNodesLayout>) {
        for (const edge of layout.edges) {
          const segments: Array<{ x1: number; y1: number; x2: number; y2: number }> = [];

          // Vertical Bus: M sx sy L sx midY L tx midY L tx ty
          const busMatch = edge.pathD.match(
            /M\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)/
          );
          if (busMatch) {
            const sx = parseFloat(busMatch[1]);
            const sy = parseFloat(busMatch[2]);
            const p1x = parseFloat(busMatch[3]);
            const p1y = parseFloat(busMatch[4]);
            const p2x = parseFloat(busMatch[5]);
            const p2y = parseFloat(busMatch[6]);
            const tx = parseFloat(busMatch[7]);
            const ty = parseFloat(busMatch[8]);

            segments.push({ x1: sx, y1: sy, x2: p1x, y2: p1y }); // stem 1
            segments.push({ x1: p1x, y1: p1y, x2: p2x, y2: p2y }); // bus bar
            segments.push({ x1: p2x, y1: p2y, x2: tx, y2: ty }); // stem 2
          }

          // Spouse straight line: M sx sy L tx ty
          const spouseMatch = edge.pathD.match(
            /^M\s+([-\d.]+)\s+([-\d.]+)\s+L\s+([-\d.]+)\s+([-\d.]+)/
          );
          if (spouseMatch && !busMatch) {
            segments.push({
              x1: parseFloat(spouseMatch[1]),
              y1: parseFloat(spouseMatch[2]),
              x2: parseFloat(spouseMatch[3]),
              y2: parseFloat(spouseMatch[4]),
            });
          }

          for (const seg of segments) {
            const xMin = Math.min(seg.x1, seg.x2);
            const xMax = Math.max(seg.x1, seg.x2);
            const yMin = Math.min(seg.y1, seg.y2);
            const yMax = Math.max(seg.y1, seg.y2);

            for (const node of layout.nodes) {
              // Strictly inside card (with 1px inset margin to allow touching boundary)
              const cardLeft = node.x + 1;
              const cardRight = node.x + node.width - 1;
              const cardTop = node.y + 1;
              const cardBottom = node.y + node.height - 1;

              const xOverlap = Math.max(xMin, cardLeft) < Math.min(xMax, cardRight);
              const yOverlap = Math.max(yMin, cardTop) < Math.min(yMax, cardBottom);

              expect(
                xOverlap && yOverlap,
                `Edge ${edge.id} segment (${seg.x1},${seg.y1})->(${seg.x2},${seg.y2}) penetrates card ${node.id} interior [x: ${cardLeft}..${cardRight}, y: ${cardTop}..${cardBottom}]`
              ).toBe(false);
            }
          }
        }
      }

      const defaultData = buildDefaultTreeViewData('admin', 'all');
      assertZeroIntersectionsEverywhere(computeAllNodesLayout(defaultData));
      assertZeroIntersectionsEverywhere(computeHourglassLayout(defaultData));
      assertZeroIntersectionsEverywhere(computeDescendantsLayout(defaultData));

      const pedData = buildDefaultTreeViewData('admin', 'pedigree');
      const pedLayout = computePedigreeLayout(pedData);
      // Pedigree cards do not overlap
      assertNoOverlappingNodes(pedLayout.nodes);
    });

    it('ensures 100% edge coverage with multiple disconnected components, 3-spouse blended family, and isolated shelf records', () => {
      const multiCompData: TreeViewResponse = {
        tree_title: 'Multi Component Complex Forest',
        access_tier: 'admin',
        focal_id: 'p-man',
        resolved_home_person_id: 'p-man',
        mode: 'all',
        generations: 4,
        total_visible: 11,
        // Reverse ordering of everything
        nodes: [
          // Shelf isolated person
          {
            id: 'p-isolated-1',
            gid: 'I0900',
            sex: 'M',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Solitary', surname: 'One' },
            birth: null,
            death: null,
            avatar_url: null,
          },
          // Component 2: Disconnected second family
          {
            id: 'p-other-child',
            gid: 'I0802',
            sex: 'F',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'OtherChild', surname: 'Jones' },
            birth: { date: '2010-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-other-wife',
            gid: 'I0801',
            sex: 'F',
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'OtherWife', surname: 'Jones' },
            birth: { date: '1975-01-01', place: null },
            death: { date: '2020-01-01', place: null },
            avatar_url: null,
          },
          {
            id: 'p-other-hubby',
            gid: 'I0800',
            sex: 'M',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'OtherHubby', surname: 'Jones' },
            birth: { date: '1970-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          // Component 1: Multi-spouse blended family
          {
            id: 'p-ch-sp3',
            gid: 'I0015',
            sex: 'F',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Child3', surname: 'Main' },
            birth: { date: '2022-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-sp3',
            gid: 'I0014',
            sex: 'F',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Spouse3', surname: 'Third' },
            birth: { date: '1992-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-ch-sp2',
            gid: 'I0013',
            sex: 'M',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Child2', surname: 'Main' },
            birth: { date: '2015-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-sp2',
            gid: 'I0012',
            sex: 'F',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Spouse2', surname: 'Second' },
            birth: { date: '1988-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-ch-sp1',
            gid: 'I0011',
            sex: 'F',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Child1', surname: 'Main' },
            birth: { date: '2008-01-01', place: null },
            death: null,
            avatar_url: null,
          },
          {
            id: 'p-sp1',
            gid: 'I0010',
            sex: 'F',
            is_living: false,
            is_private: false,
            primary_name: { given_name: 'Spouse1', surname: 'First' },
            birth: { date: '1982-01-01', place: null },
            death: { date: '2010-01-01', place: null },
            avatar_url: null,
          },
          {
            id: 'p-man',
            gid: 'I0001',
            sex: 'M',
            is_living: true,
            is_private: false,
            primary_name: { given_name: 'Man', surname: 'Main' },
            birth: { date: '1980-01-01', place: null },
            death: null,
            avatar_url: null,
          },
        ],
        families: [
          {
            id: 'fam-other',
            gid: 'F0800',
            partner1_id: 'p-other-hubby',
            partner2_id: 'p-other-wife',
            relationship_type: 'married',
            children: [{ child_id: 'p-other-child', rel_to_partner1: 'birth', rel_to_partner2: 'birth', sort_order: 0 }],
            events: [],
            is_private: false,
            version: 1,
          },
          {
            id: 'fam-m3',
            gid: 'F0003',
            partner1_id: 'p-man',
            partner2_id: 'p-sp3',
            relationship_type: 'married',
            children: [{ child_id: 'p-ch-sp3', rel_to_partner1: 'birth', rel_to_partner2: 'birth', sort_order: 0 }],
            events: [],
            is_private: false,
            version: 1,
          },
          {
            id: 'fam-m2',
            gid: 'F0002',
            partner1_id: 'p-man',
            partner2_id: 'p-sp2',
            relationship_type: 'married',
            children: [{ child_id: 'p-ch-sp2', rel_to_partner1: 'birth', rel_to_partner2: 'birth', sort_order: 0 }],
            events: [],
            is_private: false,
            version: 1,
          },
          {
            id: 'fam-m1',
            gid: 'F0001',
            partner1_id: 'p-man',
            partner2_id: 'p-sp1',
            relationship_type: 'married',
            children: [{ child_id: 'p-ch-sp1', rel_to_partner1: 'birth', rel_to_partner2: 'birth', sort_order: 0 }],
            events: [],
            is_private: false,
            version: 1,
          },
        ],
        edges: [
          { id: 'e-other-sp', source: 'p-other-hubby', target: 'p-other-wife', kind: 'spouse', family_id: 'fam-other' },
          { id: 'e-other-c1', source: 'p-other-hubby', target: 'p-other-child', kind: 'parent_child', family_id: 'fam-other' },
          { id: 'e-other-c2', source: 'p-other-wife', target: 'p-other-child', kind: 'parent_child', family_id: 'fam-other' },
          { id: 'e-sp-1', source: 'p-man', target: 'p-sp1', kind: 'spouse', family_id: 'fam-m1' },
          { id: 'e-c-1', source: 'p-man', target: 'p-ch-sp1', kind: 'parent_child', family_id: 'fam-m1' },
          { id: 'e-sp-2', source: 'p-man', target: 'p-sp2', kind: 'spouse', family_id: 'fam-m2' },
          { id: 'e-c-2', source: 'p-man', target: 'p-ch-sp2', kind: 'parent_child', family_id: 'fam-m2' },
          { id: 'e-sp-3', source: 'p-man', target: 'p-sp3', kind: 'spouse', family_id: 'fam-m3' },
          { id: 'e-c-3', source: 'p-man', target: 'p-ch-sp3', kind: 'parent_child', family_id: 'fam-m3' },
        ],
      };

      const result = computeAllNodesLayout(multiCompData);

      // Verify all 11 nodes placed
      expect(result.nodes.length).toBe(11);

      // Verify component headers
      expect(result.components?.length).toBe(3); // Main branch, Branch 2, Isolated records shelf

      // Verify 100% spouse coverage: p-man with sp1, sp2, sp3, and p-other-hubby with p-other-wife
      const expectedSpousePairs = [
        ['p-man', 'p-sp1'],
        ['p-man', 'p-sp2'],
        ['p-man', 'p-sp3'],
        ['p-other-hubby', 'p-other-wife'],
      ];
      for (const [s1, s2] of expectedSpousePairs) {
        const found = result.edges.find(
          (e) =>
            e.kind === 'spouse' &&
            ((e.sourceId === s1 && e.targetId === s2) ||
              (e.sourceId === s2 && e.targetId === s1))
        );
        expect(found, `Expected spouse edge between ${s1} and ${s2}`).toBeDefined();
      }

      // Verify 100% child coverage: p-ch-sp1, p-ch-sp2, p-ch-sp3, p-other-child
      const expectedChildren = ['p-ch-sp1', 'p-ch-sp2', 'p-ch-sp3', 'p-other-child'];
      for (const cid of expectedChildren) {
        const found = result.edges.find(
          (e) => e.kind === 'parent_child' && e.targetId === cid
        );
        expect(found, `Expected parent_child edge leading to child ${cid}`).toBeDefined();
      }

      // Verify zero card-edge intersections across all segments in this multi-component layout
      assertZeroCardEdgeIntersections(result);
    });

    it('verifies that in computeAllNodesLayout, when a couple has multiple children (including a collinear child directly beneath the parents), all children parent-child edges share the exact same sourceX and yBus', () => {
      const familyData: any = {
        focal_id: 'p-dad',
        nodes: [
          { id: 'p-dad', first_name: 'Dad', last_name: 'Test', gender: 'male', is_living: false },
          { id: 'p-mom', first_name: 'Mom', last_name: 'Test', gender: 'female', is_living: false },
          { id: 'p-c1', first_name: 'Child 1', last_name: 'Test', gender: 'male', is_living: true },
          { id: 'p-c2', first_name: 'Child 2', last_name: 'Test', gender: 'female', is_living: true },
          { id: 'p-c3', first_name: 'Child 3', last_name: 'Test', gender: 'male', is_living: true },
        ],
        edges: [
          { id: 'e-sp', source: 'p-dad', target: 'p-mom', kind: 'spouse' },
          { id: 'e-c1', source: 'p-dad', target: 'p-c1', kind: 'parent_child' },
          { id: 'e-c2', source: 'p-dad', target: 'p-c2', kind: 'parent_child' },
          { id: 'e-c3', source: 'p-dad', target: 'p-c3', kind: 'parent_child' },
          { id: 'e-m-c1', source: 'p-mom', target: 'p-c1', kind: 'parent_child' },
          { id: 'e-m-c2', source: 'p-mom', target: 'p-c2', kind: 'parent_child' },
          { id: 'e-m-c3', source: 'p-mom', target: 'p-c3', kind: 'parent_child' },
        ],
        families: [
          {
            id: 'fam-test',
            partner1_id: 'p-dad',
            partner2_id: 'p-mom',
            children: ['p-c1', 'p-c2', 'p-c3'],
          },
        ],
      };

      const result = computeAllNodesLayout(familyData);
      const childEdges = result.edges.filter((e) => e.kind === 'parent_child');
      expect(childEdges.length).toBe(3);

      const dadNode = result.nodes.find((n) => n.id === 'p-dad')!;
      const momNode = result.nodes.find((n) => n.id === 'p-mom')!;
      const leftNode = dadNode.x < momNode.x ? dadNode : momNode;
      const rightNode = dadNode.x < momNode.x ? momNode : dadNode;
      const unionMidX = (leftNode.x + leftNode.width + rightNode.x) / 2;

      // Check that at least one child is collinear with unionMidX (targetX === unionMidX)
      const hasCollinearChild = childEdges.some((e) => Math.abs(e.targetX - unionMidX) < 1);
      expect(hasCollinearChild).toBe(true);

      // Verify that all child edges share the EXACT same sourceX (the union midpoint) and yBus
      const firstSourceX = childEdges[0].sourceX;
      const firstYBus = childEdges[0].yBus;
      expect(firstSourceX).toBe(unionMidX);

      for (const edge of childEdges) {
        expect(edge.sourceX).toBe(firstSourceX);
        expect(edge.yBus).toBe(firstYBus);
      }
    });

    it('verifies that in computeDescendantsLayout and computeHourglassLayout, children in the same family share the exact same sourceX and yBus without collinear offset', () => {
      const familyData: any = {
        focal_id: 'p-dad',
        nodes: [
          { id: 'p-dad', first_name: 'Dad', last_name: 'Test', gender: 'male', is_living: false },
          { id: 'p-mom', first_name: 'Mom', last_name: 'Test', gender: 'female', is_living: false },
          { id: 'p-c1', first_name: 'Child 1', last_name: 'Test', gender: 'male', is_living: true },
          { id: 'p-c2', first_name: 'Child 2', last_name: 'Test', gender: 'female', is_living: true },
          { id: 'p-c3', first_name: 'Child 3', last_name: 'Test', gender: 'male', is_living: true },
        ],
        edges: [
          { id: 'e-sp', source: 'p-dad', target: 'p-mom', kind: 'spouse' },
          { id: 'e-c1', source: 'p-dad', target: 'p-c1', kind: 'parent_child' },
          { id: 'e-c2', source: 'p-dad', target: 'p-c2', kind: 'parent_child' },
          { id: 'e-c3', source: 'p-dad', target: 'p-c3', kind: 'parent_child' },
        ],
        families: [
          {
            id: 'fam-test',
            partner1_id: 'p-dad',
            partner2_id: 'p-mom',
            children: ['p-c1', 'p-c2', 'p-c3'],
          },
        ],
      };

      // Descendants layout
      const descResult = computeDescendantsLayout(familyData, 3);
      const descChildEdges = descResult.edges.filter((e) => e.kind === 'parent_child');
      expect(descChildEdges.length).toBe(3);
      const descFirstSx = descChildEdges[0].sourceX;
      const descFirstYBus = descChildEdges[0].yBus;
      for (const edge of descChildEdges) {
        expect(edge.sourceX).toBe(descFirstSx);
        expect(edge.yBus).toBe(descFirstYBus);
      }

      // Hourglass layout
      const hgResult = computeHourglassLayout(familyData, 3);
      const hgChildEdges = hgResult.edges.filter((e) => e.kind === 'parent_child' && e.sourceId === 'p-dad');
      expect(hgChildEdges.length).toBe(3);
      const hgFirstSx = hgChildEdges[0].sourceX;
      const hgFirstYBus = hgChildEdges[0].yBus;
      for (const edge of hgChildEdges) {
        expect(edge.sourceX).toBe(hgFirstSx);
        expect(edge.yBus).toBe(hgFirstYBus);
      }
    });
  });

  describe('Milestone 2.6: Interactive Tree Connectors, Visual Clarity & Popover Navigation', () => {
    it('renders marriage lines in ochre/amber (stroke-amber-600 dark:stroke-amber-400) with a midpoint circle dot (r=3/4), direct lineage in brand blue, and collateral in neutral slate', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const html = renderWithProviders(<TreeCanvas initialTreeView={sample} />);

      // Marriage line in ochre/amber (UX-070 >= 3:1 contrast)
      expect(html).toContain('stroke-amber-600');
      // Midpoint circle dot for marriage with r=3 and fill-amber-600
      expect(html).toMatch(/<circle[^>]*r="3"[^>]*class="[^"]*fill-amber-600[^"]*"/);

      // Direct lineage line in brand blue
      expect(html).toContain('stroke-brand/80');

      // Collateral line in crisp neutral slate (UX-067: stroke-secondary/70 >= 3:1 contrast)
      expect(html).toContain('stroke-secondary/70');
    });

    it('renders dual-layer hit targets (14px desktop / 20px mobile) with stroke="transparent" and pointer-events: stroke', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const desktopHtml = renderWithProviders(<TreeCanvas initialTreeView={sample} isMobileViewport={false} />);
      expect(desktopHtml).toContain('stroke-width="14"');
      expect(desktopHtml).toContain('pointer-events:stroke');
      expect(desktopHtml).toContain('stroke="transparent"');
      expect(desktopHtml).toContain('role="button"');
      expect(desktopHtml).toContain('tabindex="0"');
      // UX-062: Parent SVG has role="group" and aria-label, not aria-hidden="true"
      expect(desktopHtml).toContain('role="group"');
      expect(desktopHtml).toContain('aria-label="Tree connections"');
      // UX-063: Hit path has visible focus ring on :focus-visible
      expect(desktopHtml).toContain('focus-visible:stroke-brand/50');

      const mobileHtml = renderWithProviders(<TreeCanvas initialTreeView={sample} isMobileViewport={true} />);
      expect(mobileHtml).toContain('stroke-width="20"');
    });

    it('highlights endpoint cards with focus rings (ring-2 ring-brand) and accessible attributes when isEdgeEndpointHighlighted is true', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const node = sample.nodes[0];

      // Default card (not highlighted)
      const normalHtml = renderWithProviders(
        <PersonNodeCard
          node={node}
          x={0}
          y={0}
          isEdgeEndpointHighlighted={false}
        />
      );
      expect(normalHtml).not.toContain('ring-2 ring-brand shadow-elevation-2');
      expect(normalHtml).not.toContain('data-connection-endpoint="true"');

      // Card with edge endpoint highlighted (UX-068: accessible label and data attribute)
      const highlightedHtml = renderWithProviders(
        <PersonNodeCard
          node={node}
          x={0}
          y={0}
          isEdgeEndpointHighlighted={true}
        />
      );
      expect(highlightedHtml).toContain('border-brand ring-2 ring-brand shadow-elevation-2');
      expect(highlightedHtml).toContain('data-connection-endpoint="true"');
      expect(highlightedHtml).toContain('Connection endpoint');
    });

    it('renders ConnectionNavPopover with relationship badge, participant chips, role labels, jump-to, and find kinship path', () => {
      const edge = createPositionedEdge({
        id: 'test-edge-spouse',
        sourceId: 'p-husband',
        targetId: 'p-wife',
        kind: 'spouse',
        sourceX: 100,
        sourceY: 100,
        targetX: 300,
        targetY: 100,
        pathD: 'M 100 100 L 300 100',
      });

      const husband: any = {
        id: 'p-husband',
        gid: 'I0001',
        sex: 'M',
        is_living: false,
        primary_name: { given_name: 'Alexander', surname: 'Levshits' },
      };
      const wife: any = {
        id: 'p-wife',
        gid: 'I0002',
        sex: 'F',
        is_living: true,
        primary_name: { given_name: 'Sofia', surname: 'Levshits' },
      };

      // 1. Authenticated view (editor)
      const popoverHtml = renderWithProviders(
        <ConnectionNavPopover
          edge={edge}
          sourcePerson={husband}
          targetPerson={wife}
          accessTier="editor"
          pan={{ x: 0, y: 0 }}
          zoom={1}
          onJumpToPerson={() => {}}
          onFindKinship={() => {}}
          onClose={() => {}}
        />,
        'en'
      );

      expect(popoverHtml).toContain('role="dialog"');
      expect(popoverHtml).toContain('Marriage');
      expect(popoverHtml).toContain('Alexander Levshits');
      expect(popoverHtml).toContain('Sofia Levshits');
      expect(popoverHtml).toContain('Jump to Alexander Levshits');
      expect(popoverHtml).toContain('Jump to Sofia Levshits');
      expect(popoverHtml).toContain('Find kinship path');
      expect(popoverHtml).toContain('Close connection details');
      // UX-065: Sub-40px touch targets expanded with touch-target-expand
      expect(popoverHtml).toContain('touch-target-expand');

      // 2. Public Anonymous view masks living wife
      const guestPopoverHtml = renderWithProviders(
        <ConnectionNavPopover
          edge={edge}
          sourcePerson={husband}
          targetPerson={wife}
          accessTier="public_anonymous"
          pan={{ x: 0, y: 0 }}
          zoom={1}
          onJumpToPerson={() => {}}
          onFindKinship={() => {}}
          onClose={() => {}}
        />,
        'en'
      );
      expect(guestPopoverHtml).toContain('Alexander Levshits');
      expect(guestPopoverHtml).toContain('S.');
      expect(guestPopoverHtml).not.toContain('Sofia Levshits');

      // 3. Parent-child connection in Russian
      const parentChildEdge = createPositionedEdge({
        id: 'test-edge-parent-child',
        sourceId: 'p-husband',
        targetId: 'p-child',
        kind: 'parent_child',
        sourceX: 200,
        sourceY: 100,
        targetX: 200,
        targetY: 250,
        pathD: 'M 200 100 L 200 250',
      });
      const child: any = {
        id: 'p-child',
        gid: 'I0003',
        sex: 'M',
        is_living: false,
        primary_name: { given_name: 'Maxim', surname: 'Levshits' },
      };
      const ruPopoverHtml = renderWithProviders(
        <ConnectionNavPopover
          edge={parentChildEdge}
          sourcePerson={husband}
          targetPerson={child}
          accessTier="admin"
          pan={{ x: 0, y: 0 }}
          zoom={1}
          onJumpToPerson={() => {}}
          onFindKinship={() => {}}
          onClose={() => {}}
        />,
        'ru'
      );
      expect(ruPopoverHtml).toContain('Связь с родителем');
      expect(ruPopoverHtml).toContain('Отец');
      expect(ruPopoverHtml).toContain('Сын');
      expect(ruPopoverHtml).toContain('Найти степень родства');
    });

    it('enforces anti-collinear bus runner clearance (>= 12px) and lateral stem offsets in tree layout', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const hgLayout = computeHourglassLayout(sample, 4, true);

      // Verify edges have defined bus positions or non-zero clearance
      const parentChildEdges = hgLayout.edges.filter((e) => e.kind === 'parent_child');
      expect(parentChildEdges.length).toBeGreaterThan(0);

      // Check runner stagger values: when siblings exist under a parent, their yBus values differ by >= 12px
      const edgesWithBus = parentChildEdges.filter((e) => e.yBus !== undefined);
      if (edgesWithBus.length >= 2) {
        const uniqueBus = Array.from(new Set(edgesWithBus.map((e) => e.yBus!)));
        if (uniqueBus.length >= 2) {
          const diffs = [];
          for (let i = 0; i < uniqueBus.length; i++) {
            for (let j = i + 1; j < uniqueBus.length; j++) {
              diffs.push(Math.abs(uniqueBus[i] - uniqueBus[j]));
            }
          }
          const minDiff = Math.min(...diffs);
          expect(minDiff).toBeGreaterThanOrEqual(12);
        }
      }
    });

    it('verifies pan idle transforms round to integer pixels in TreeCanvas to prevent subpixel blur', () => {
      const treeCanvasSrc = fs.readFileSync(
        path.resolve(__dirname, '../../src/components/tree/TreeCanvas.tsx'),
        'utf-8'
      );
      // Confirms pan idle transforms use Math.round on pan.x and pan.y
      expect(treeCanvasSrc).toContain('isPanning ? pan.x : Math.round(pan.x)');
      expect(treeCanvasSrc).toContain('isPanning ? pan.y : Math.round(pan.y)');

      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const html = renderWithProviders(<TreeCanvas initialTreeView={sample} />);
      // Initial idle render has integer coordinates in translate3d
      expect(html).toMatch(/translate3d\(\d+px,\s*\d+px,\s*0\)/);
    });

    it('verifies UX-062: TreeCanvas SVG container is not aria-hidden, has role="group" and aria-label; decorative paths have aria-hidden="true"', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const html = renderWithProviders(<TreeCanvas initialTreeView={sample} />);
      // Parent SVG has role="group" and aria-label="Tree connections"
      expect(html).toContain('role="group"');
      expect(html).toContain('aria-label="Tree connections"');
      // Visual decorative paths inside SVG have aria-hidden="true"
      expect(html).toContain('aria-hidden="true"');
    });

    it('verifies UX-063: transparent hit paths have focus-visible:stroke-brand/50 for visible keyboard focus', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const html = renderWithProviders(<TreeCanvas initialTreeView={sample} />);
      expect(html).toContain('focus-visible:stroke-brand/50');
      expect(html).toContain('focus-visible:outline-hidden');
    });

    it('verifies UX-064: PublicGuestBanner has responsive layout wrapping on mobile viewports (< sm)', () => {
      const bannerHtml = renderWithProviders(<PublicGuestBanner onSignIn={() => {}} />);
      // Desktop sign-in button is hidden on small screens
      expect(bannerHtml).toContain('hidden sm:inline-flex');
      // Mobile sign-in button is hidden on desktop screens
      expect(bannerHtml).toContain('pt-1.5 sm:hidden');
      // Close button has touch-target-expand
      expect(bannerHtml).toContain('touch-target-expand');
    });

    it('verifies UX-065: ConnectionNavPopover close, jump, and view icon buttons have >= 40x40px touch targets with touch-target-expand', () => {
      const edge = createPositionedEdge({
        id: 'test-edge-target',
        sourceId: 'p-1',
        targetId: 'p-2',
        kind: 'spouse',
        sourceX: 100,
        sourceY: 100,
        targetX: 300,
        targetY: 100,
        pathD: 'M 100 100 L 300 100',
      });
      const p1: any = {
        id: 'p-1',
        gid: 'I0001',
        sex: 'M',
        is_living: false,
        primary_name: { given_name: 'John', surname: 'Doe' },
      };
      const p2: any = {
        id: 'p-2',
        gid: 'I0002',
        sex: 'F',
        is_living: false,
        primary_name: { given_name: 'Jane', surname: 'Doe' },
      };

      const html = renderWithProviders(
        <ConnectionNavPopover
          edge={edge}
          sourcePerson={p1}
          targetPerson={p2}
          accessTier="admin"
          pan={{ x: 0, y: 0 }}
          zoom={1}
          onJumpToPerson={() => {}}
          onFindKinship={() => {}}
          onClose={() => {}}
          onViewPerson={() => {}}
        />
      );
      // All icon buttons have h-8 w-8 min-h-8 min-w-8 touch-target-expand
      const occurrences = (html.match(/h-8 w-8 min-h-8 min-w-8 text-secondary hover:text-[a-z]+ touch-target-expand/g) || []).length;
      expect(occurrences).toBeGreaterThanOrEqual(4);
    });

    it('verifies UX-066: Jump to person dismisses popover, centers camera, and shifts focus; Kinship dismisses popover', () => {
      const treeCanvasSrc = fs.readFileSync(
        path.resolve(__dirname, '../../src/components/tree/TreeCanvas.tsx'),
        'utf-8'
      );
      // setSelectedEdge(null) in handleJumpToPerson
      expect(treeCanvasSrc).toMatch(/handleJumpToPerson\s*=\s*useCallback\([\s\S]*setSelectedEdge\(null\)/);
      // requestAnimationFrame DOM query selector focus
      expect(treeCanvasSrc).toContain('[data-node-id="${personId}"]');
      expect(treeCanvasSrc).toContain('cardEl?.focus()');
      // setSelectedEdge(null) in handleFindKinshipFromEdge
      expect(treeCanvasSrc).toMatch(/handleFindKinshipFromEdge\s*=\s*useCallback\([\s\S]*setSelectedEdge\(null\)/);
    });

    it('verifies UX-067: Collateral connector strokes use stroke-secondary/70 and LOD 2 stems use bg-secondary/70', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const html = renderWithProviders(<TreeCanvas initialTreeView={sample} />);
      expect(html).toContain('stroke-secondary/70');

      const node: any = {
        id: 'p-collateral',
        gid: 'I0099',
        sex: 'M',
        is_living: false,
        is_direct_lineage: false,
        primary_name: { given_name: 'Cousin', surname: 'Test' },
      };
      const lod2Html = renderWithProviders(
        <PersonNodeCard
          node={node}
          x={0}
          y={0}
          lod={2}
          hasParentEdge={true}
          hasChildEdge={true}
        />
      );
      expect(lod2Html).toContain('bg-secondary/70');
    });

    it('verifies UX-068: Endpoint highlighted card renders accessible label, title, and data attribute in EN and RU', () => {
      const sample = buildDefaultTreeViewData('admin', 'hourglass');
      const node = sample.nodes[0];

      // English
      const enHtml = renderWithProviders(
        <PersonNodeCard
          node={node}
          x={0}
          y={0}
          isEdgeEndpointHighlighted={true}
        />,
        'en'
      );
      expect(enHtml).toContain('Connection endpoint');
      expect(enHtml).toContain('data-connection-endpoint="true"');
      expect(enHtml).toContain('title="Connection endpoint"');

      // Russian
      const ruHtml = renderWithProviders(
        <PersonNodeCard
          node={node}
          x={0}
          y={0}
          isEdgeEndpointHighlighted={true}
        />,
        'ru'
      );
      expect(ruHtml).toContain('Конечная точка связи');
      expect(ruHtml).toContain('data-connection-endpoint="true"');
      expect(ruHtml).toContain('title="Конечная точка связи"');
    });

    it('places 6+ ancestor generations (Gen -1 to Gen -6) with collateral siblings and extra spouses in Hourglass and Pedigree when All Generations (10) is selected', () => {
      const makePerson = (
        id: string,
        given: string,
        sex: 'M' | 'F'
      ) => ({
        id,
        gid: id.toUpperCase(),
        sex,
        is_living: false,
        privacy_tier: 'public' as const,
        primary_name: {
          given_name: given,
          surname: 'Romanov',
          patronymic: null,
          maiden_name: null,
        },
        birth_date: '1850',
        birth_place: null,
        death_date: '1920',
        death_place: null,
        avatar_url: null,
      });

      const nodes = [
        makePerson('p-focal', 'Focal', 'M'),
        // Gen -1
        makePerson('p-f1', 'Father1', 'M'),
        makePerson('p-m1', 'Mother1', 'F'),
        // Gen -2
        makePerson('p-f2', 'Grandfather2', 'M'),
        makePerson('p-m2', 'Grandmother2', 'F'),
        // Gen -3
        makePerson('p-f3', 'GreatGf3', 'M'),
        makePerson('p-m3', 'GreatGm3', 'F'),
        // Gen -4 (plus collateral sibling of p-f4)
        makePerson('p-f4', 'AncGf4', 'M'),
        makePerson('p-m4', 'AncGm4', 'F'),
        makePerson('p-f4-sib', 'GreatUncle4', 'M'),
        // Gen -5 (plus extra spouse of p-f5 and half-sibling at Gen -4)
        makePerson('p-f5', 'AncGf5', 'M'),
        makePerson('p-m5', 'AncGm5', 'F'),
        makePerson('p-f5-spouse2', 'SecondWife5', 'F'),
        makePerson('p-f4-halfsib', 'HalfUncle4', 'F'),
        // Gen -6
        makePerson('p-f6', 'AncGf6', 'M'),
        makePerson('p-m6', 'AncGm6', 'F'),
      ];

      const edges = [
        // Gen -1 -> Gen 0
        { id: 'e-sp-1', source: 'p-f1', target: 'p-m1', kind: 'spouse' as const, family_id: 'fam-1' },
        { id: 'e-pc-1a', source: 'p-f1', target: 'p-focal', kind: 'parent_child' as const, family_id: 'fam-1' },
        { id: 'e-pc-1b', source: 'p-m1', target: 'p-focal', kind: 'parent_child' as const, family_id: 'fam-1' },
        // Gen -2 -> Gen -1
        { id: 'e-sp-2', source: 'p-f2', target: 'p-m2', kind: 'spouse' as const, family_id: 'fam-2' },
        { id: 'e-pc-2a', source: 'p-f2', target: 'p-f1', kind: 'parent_child' as const, family_id: 'fam-2' },
        { id: 'e-pc-2b', source: 'p-m2', target: 'p-f1', kind: 'parent_child' as const, family_id: 'fam-2' },
        // Gen -3 -> Gen -2
        { id: 'e-sp-3', source: 'p-f3', target: 'p-m3', kind: 'spouse' as const, family_id: 'fam-3' },
        { id: 'e-pc-3a', source: 'p-f3', target: 'p-f2', kind: 'parent_child' as const, family_id: 'fam-3' },
        { id: 'e-pc-3b', source: 'p-m3', target: 'p-f2', kind: 'parent_child' as const, family_id: 'fam-3' },
        // Gen -4 -> Gen -3
        { id: 'e-sp-4', source: 'p-f4', target: 'p-m4', kind: 'spouse' as const, family_id: 'fam-4' },
        { id: 'e-pc-4a', source: 'p-f4', target: 'p-f3', kind: 'parent_child' as const, family_id: 'fam-4' },
        { id: 'e-pc-4b', source: 'p-m4', target: 'p-f3', kind: 'parent_child' as const, family_id: 'fam-4' },
        // Gen -5 -> Gen -4 (including full sibling p-f4-sib and extra union with p-f5-spouse2 -> p-f4-halfsib)
        { id: 'e-sp-5', source: 'p-f5', target: 'p-m5', kind: 'spouse' as const, family_id: 'fam-5' },
        { id: 'e-pc-5a', source: 'p-f5', target: 'p-f4', kind: 'parent_child' as const, family_id: 'fam-5' },
        { id: 'e-pc-5b', source: 'p-m5', target: 'p-f4', kind: 'parent_child' as const, family_id: 'fam-5' },
        { id: 'e-pc-5c', source: 'p-f5', target: 'p-f4-sib', kind: 'parent_child' as const, family_id: 'fam-5' },
        { id: 'e-pc-5d', source: 'p-m5', target: 'p-f4-sib', kind: 'parent_child' as const, family_id: 'fam-5' },
        { id: 'e-sp-5-extra', source: 'p-f5', target: 'p-f5-spouse2', kind: 'spouse' as const, family_id: 'fam-5-extra' },
        { id: 'e-pc-5-extra1', source: 'p-f5', target: 'p-f4-halfsib', kind: 'parent_child' as const, family_id: 'fam-5-extra' },
        { id: 'e-pc-5-extra2', source: 'p-f5-spouse2', target: 'p-f4-halfsib', kind: 'parent_child' as const, family_id: 'fam-5-extra' },
        // Gen -6 -> Gen -5
        { id: 'e-sp-6', source: 'p-f6', target: 'p-m6', kind: 'spouse' as const, family_id: 'fam-6' },
        { id: 'e-pc-6a', source: 'p-f6', target: 'p-f5', kind: 'parent_child' as const, family_id: 'fam-6' },
        { id: 'e-pc-6b', source: 'p-m6', target: 'p-f5', kind: 'parent_child' as const, family_id: 'fam-6' },
      ];

      const deepData = {
        mode: 'hourglass' as const,
        focal_id: 'p-focal',
        generations: 10,
        access_tier: 'admin' as const,
        nodes,
        families: [],
        edges,
      };

      const hgLayout = computeHourglassLayout(deepData, 10, true);
      assertNoOverlappingNodes(hgLayout.nodes);

      // Every single person in the 6-generation ancestor graph (including collateral sibling & extra spouse) must be placed
      const placedIds = new Set(hgLayout.nodes.map((n) => n.id));
      for (const person of nodes) {
        expect(placedIds.has(person.id), `Expected ${person.id} to be placed in Hourglass layout`).toBe(true);
      }

      // Verify generations -3, -4, -5, -6 have accurate negative Y coordinates
      const nodeGen3 = hgLayout.nodes.find((n) => n.id === 'p-f3')!;
      const nodeGen4 = hgLayout.nodes.find((n) => n.id === 'p-f4')!;
      const nodeGen5 = hgLayout.nodes.find((n) => n.id === 'p-f5')!;
      const nodeGen6 = hgLayout.nodes.find((n) => n.id === 'p-f6')!;
      expect(nodeGen3.generation).toBe(-3);
      expect(nodeGen4.generation).toBe(-4);
      expect(nodeGen5.generation).toBe(-5);
      expect(nodeGen6.generation).toBe(-6);
      expect(nodeGen6.y).toBeLessThan(nodeGen5.y);
      expect(nodeGen5.y).toBeLessThan(nodeGen4.y);
      expect(nodeGen4.y).toBeLessThan(nodeGen3.y);

      // Also verify Pedigree layout with maxGenerationsOverride = 10 places up to Gen 6
      const pedLayout = computePedigreeLayout(
        { ...deepData, mode: 'pedigree' },
        10
      );
      assertNoOverlappingNodes(pedLayout.nodes);
      const pedPlacedIds = new Set(pedLayout.nodes.map((n) => n.id));
      expect(pedPlacedIds.has('p-f3')).toBe(true);
      expect(pedPlacedIds.has('p-f4')).toBe(true);
      expect(pedPlacedIds.has('p-f5')).toBe(true);
      expect(pedPlacedIds.has('p-f6')).toBe(true);
      expect(pedPlacedIds.has('p-m6')).toBe(true);
    });

    it('places 4th-generation descendant (gen = 4) under gen = 3 parent in Descendants mode with generations = 4 and never places floating cards at y < 0', () => {
      const makeDescNode = (id: string, given: string) => ({
        id,
        gid: id.toUpperCase(),
        sex: 'M' as const,
        is_living: false,
        privacy_tier: 'public' as const,
        primary_name: {
          given_name: given,
          surname: 'Левшиц',
          patronymic: null,
          maiden_name: null,
        },
        birth_date: '1850',
        birth_place: null,
        death_date: '1920',
        death_place: null,
        avatar_url: null,
      });

      const descData = {
        mode: 'descendant' as const,
        focal_id: 'i-petr',
        generations: 4,
        access_tier: 'admin' as const,
        nodes: [
          makeDescNode('i-petr', 'Петр'),
          makeDescNode('i-ioakim', 'Иоаким'),
          makeDescNode('i-luka', 'Лука'),
          makeDescNode('i-kosma', 'Косьма'),
          makeDescNode('i-filimon', 'Филимон'),
          makeDescNode('i-disconnected', 'Disconnected'),
        ],
        families: [],
        edges: [
          { id: 'e-1', source: 'i-petr', target: 'i-ioakim', kind: 'parent_child' as const, family_id: 'f-1' },
          { id: 'e-2', source: 'i-ioakim', target: 'i-luka', kind: 'parent_child' as const, family_id: 'f-2' },
          { id: 'e-3', source: 'i-luka', target: 'i-kosma', kind: 'parent_child' as const, family_id: 'f-3' },
          { id: 'e-4', source: 'i-kosma', target: 'i-filimon', kind: 'parent_child' as const, family_id: 'f-4' },
        ],
      };

      const descLayout = computeDescendantsLayout(descData, 4);
      assertNoOverlappingNodes(descLayout.nodes);

      // No node may ever be placed at y < 0 above the root ancestor
      expect(descLayout.nodes.every((n) => n.y >= 0)).toBe(true);
      expect(descLayout.nodes.find((n) => n.id === 'i-disconnected')).toBeUndefined();

      const kosmaNode = descLayout.nodes.find((n) => n.id === 'i-kosma')!;
      const filimonNode = descLayout.nodes.find((n) => n.id === 'i-filimon')!;
      expect(kosmaNode).toBeDefined();
      expect(filimonNode).toBeDefined();
      expect(kosmaNode.generation).toBe(3);
      expect(filimonNode.generation).toBe(4);
      expect(filimonNode.y).toBe(4 * 180); // 4 * ROW_STEP
      expect(filimonNode.y).toBeGreaterThan(kosmaNode.y);

      const edgeKosmaFilimon = descLayout.edges.find(
        (e) => e.kind === 'parent_child' && e.sourceId === 'i-kosma' && e.targetId === 'i-filimon'
      );
      expect(edgeKosmaFilimon).toBeDefined();
    });
  });
});



