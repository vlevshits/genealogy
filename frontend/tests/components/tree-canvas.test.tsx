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
  computeDescendantsLayout,
  computeHourglassLayout,
  computePedigreeLayout,
  PositionedNode,
} from '../../src/lib/tree-layouts';
import {
  PersonBottomSheet,
  PersonDrawer,
  PersonNodeCard,
  TreeCanvas,
} from '../../src/components/tree';

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
    expect(htmlEn).toContain('Pedigree');
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
    expect(drawerHtml).toContain('Make Focal Person');
    expect(drawerHtml).toContain('View Full Profile');

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
});
