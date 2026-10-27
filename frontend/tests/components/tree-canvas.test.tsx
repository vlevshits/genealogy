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
});
