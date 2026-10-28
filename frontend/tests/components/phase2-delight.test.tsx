import React from 'react';
import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'node:fs';
import path from 'node:path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import { AppShell, BottomNav, ConfirmProvider, ToastProvider } from '../../src/components/ui';
import {
  AnniversariesWidget,
  LatestPhotosWidget,
  RecentActivityWidget,
  TreeGlanceWidget,
} from '../../src/components/dashboard';
import {
  buildDefaultDashboardPayload,
  DashboardPage,
} from '../../src/pages/DashboardPage';
import {
  FanChartCanvas,
  KinshipFinderDialog,
  normalizeRemoteKinshipResponse,
  PersonBottomSheet,
  PersonDrawer,
  TreeCanvas,
} from '../../src/components/tree';
import { buildDefaultTreeViewData } from '../../src/lib/tree-layouts';
import { QuickAddDuplicateHint } from '../../src/components/quality';
import { DataQualityPage } from '../../src/pages/DataQualityPage';
import { PersonMergePage } from '../../src/pages/PersonMergePage';
import {
  buildDefaultPersonProfile,
  PersonProfilePage,
} from '../../src/pages/PersonProfilePage';
import { PlacesPage } from '../../src/pages/PlacesPage';
import { App } from '../../src/App';

const DEMO_TREE_DATA = buildDefaultTreeViewData('admin', 'hourglass');

function renderWithProviders(
  node: React.ReactElement,
  locale: 'en' | 'ru' = 'en',
  role: 'admin' | 'editor' | 'viewer' | null = 'admin'
): string {
  const initialUser = role
    ? {
        id: 'u-test',
        email: 'test@example.com',
        first_name: 'Elena',
        last_name: 'Volkova',
        role,
        person_id: 'p-elena',
        home_person_id: 'p-elena',
        preferred_locale: locale,
        theme_preference: 'system' as const,
        is_active: true,
      }
    : null;

  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <AuthProvider
          initialUser={initialUser}
          initialSetupStatus={{
            is_initialized: true,
            enable_public_access: role === null,
            tree_title: 'Volkov & Levshits Archive',
          }}
        >
          <ToastProvider>
            <ConfirmProvider>{node}</ConfirmProvider>
          </ToastProvider>
        </AuthProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

describe('Phase 2 ("Delight") — Components, Pages & Engineering Guardrails', () => {
  const phase2Files = [
    'src/components/dashboard/AnniversariesWidget.tsx',
    'src/components/dashboard/RecentActivityWidget.tsx',
    'src/components/dashboard/TreeGlanceWidget.tsx',
    'src/components/dashboard/LatestPhotosWidget.tsx',
    'src/pages/DashboardPage.tsx',
    'src/components/tree/FanChartCanvas.tsx',
    'src/components/tree/KinshipFinderDialog.tsx',
    'src/components/quality/QuickAddDuplicateHint.tsx',
    'src/pages/DataQualityPage.tsx',
    'src/pages/PersonMergePage.tsx',
    'src/components/places/PlacesMapView.tsx',
  ];

  it('enforces zero raw HTML interactive elements and zero hardcoded hex colors in all Phase 2 files', () => {
    const rawHtmlPattern = /<(button|input|select|textarea)\b/;
    const hexColorPattern = /#[0-9a-fA-F]{3,8}\b/;

    for (const relPath of phase2Files) {
      const absPath = path.resolve(__dirname, '../../', relPath);
      const source = fs.readFileSync(absPath, 'utf-8');
      expect(
        rawHtmlPattern.test(source),
        `Found raw HTML element in ${relPath}`
      ).toBe(false);
      expect(
        hexColorPattern.test(source),
        `Found hardcoded hex literal in ${relPath}`
      ).toBe(false);
    }
  });

  it('renders DashboardPage and all 4 live widgets in EN and RU with † for deceased and single-initial masking for public guests', () => {
    const htmlEn = renderWithProviders(
      <DashboardPage
        initialDashboard={buildDefaultDashboardPayload()}
        instanceTitle="Volkov Family Archive"
        resolvedHomePersonId="p-elena"
      />,
      'en',
      'admin'
    );
    expect(htmlEn).toContain('Upcoming Anniversaries (Next 30 Days)');
    expect(htmlEn).toContain('Recent Tree Changes');
    expect(htmlEn).toContain('Tree at a Glance');
    expect(htmlEn).toContain('Latest Tagged Photographs');
    expect(htmlEn).toContain('†');
    expect(htmlEn).not.toContain('>Living<');

    const htmlRu = renderWithProviders(
      <DashboardPage
        initialDashboard={buildDefaultDashboardPayload()}
        instanceTitle="Семейный архив Волковых"
        resolvedHomePersonId="p-elena"
      />,
      'ru',
      'admin'
    );
    expect(htmlRu).toContain('Ближайшие годовщины (30 дней)');
    expect(htmlRu).toContain('Недавние изменения в древе');
    expect(htmlRu).toContain('Сводка семейного древа');
    expect(htmlRu).toContain('Новые отмеченные фотографии');

    // Public Guest mode masks living names to single initial
    const htmlGuest = renderWithProviders(
      <AnniversariesWidget
        items={[
          {
            id: 'ann-living',
            type: 'birthday',
            person_id: 'p-elena',
            display_name: 'Elena Volkova',
            is_living: true,
            date_month_day: '05-14',
            date_label: 'May 14',
            days_until: 2,
            years_count: 37,
          },
        ]}
        scope="all"
        onScopeChange={() => {}}
        accessTier="public_anonymous"
      />,
      'en',
      null
    );
    expect(htmlGuest).toContain('E.');
    expect(htmlGuest).not.toContain('Elena Volkova');
  });

  it('renders RecentActivityWidget, TreeGlanceWidget, and LatestPhotosWidget cleanly', () => {
    const htmlWidgets = renderWithProviders(
      <div>
        <RecentActivityWidget
          isAdmin={true}
          transactions={[
            {
              tx_id: 'tx-1',
              actor_id: 'u-1',
              actor_name: 'Elena Volkova',
              created_at: new Date().toISOString(),
              tx_label: 'personUpdateProfile',
              entity_type: 'person',
              entity_id: 'p-elena',
              entity_label: 'Elena Volkova',
              action: 'updated',
              is_undone: false,
              entries_count: 1,
            },
          ]}
        />
        <TreeGlanceWidget
          stats={{
            total_persons: 48,
            max_generations: 5,
            total_photos: 32,
            total_places: 14,
            total_families: 19,
            open_warnings_count: 3,
            missing_birth_dates_count: 4,
            oldest_person_name: 'Pyotr Volkov',
            oldest_birth_year: 1858,
          }}
          isAdmin={true}
        />
        <LatestPhotosWidget
          photos={[
            {
              id: 'm-1',
              gid: 'M0001',
              title: 'Summer in Tver',
              media_type: 'photo',
              mime_type: 'image/webp',
              file_size_bytes: 120000,
              original_url: '/media/1.webp',
              thumb_url: '/media/1-thumb.webp',
              is_private: false,
              tagged_people: [],
              created_at: new Date().toISOString(),
              version: 1,
            },
          ]}
        />
      </div>,
      'en',
      'admin'
    );
    expect(htmlWidgets).toContain('48');
    expect(htmlWidgets).toContain('Pyotr Volkov');
    expect(htmlWidgets).toContain('Summer in Tver');
  });

  it('renders FanChartCanvas (180°/360° Ahnentafel arcs) and KinshipFinderDialog', () => {
    const htmlFan = renderWithProviders(
      <FanChartCanvas
        focalPersonId="p-elena"
        nodes={DEMO_TREE_DATA.nodes}
        edges={DEMO_TREE_DATA.edges}
        selectedPersonId="p-elena"
        generations={5}
        onSelectPerson={() => {}}
        onFocusPerson={() => {}}
        onGenerationsChange={() => {}}
      />,
      'en',
      'admin'
    );
    expect(htmlFan).toContain('data-testid="fan-chart-canvas"');
    expect(htmlFan).toContain('180 deg Semicircle');
    expect(htmlFan).toContain('360 deg Full Wheel');
    expect(htmlFan).toContain('Paternal Grandfather Line');

    const htmlKinship = renderWithProviders(
      <KinshipFinderDialog
        open={true}
        onOpenChange={() => {}}
        people={DEMO_TREE_DATA.nodes}
        edges={DEMO_TREE_DATA.edges}
        initialPersonAId={DEMO_TREE_DATA.nodes[0].id}
        initialPersonBId={DEMO_TREE_DATA.nodes[1].id}
        onHighlightPath={() => {}}
      />,
      'en',
      'admin'
    );
    expect(htmlKinship).toContain('Kinship &amp; Relationship Path Finder');
    expect(htmlKinship).toContain('Relationship Step Chain');
    expect(htmlKinship).toContain('Highlight Path on Tree');
  });

  it('renders DataQualityPage, QuickAddDuplicateHint, and PersonMergePage in EN and RU', () => {
    const htmlQuality = renderWithProviders(<DataQualityPage />, 'en', 'admin');
    expect(htmlQuality).toContain('Data Quality &amp; Duplicate Detection');
    expect(htmlQuality).toContain('Death Before Birth');
    expect(htmlQuality).toContain('This is correct');

    const firstDeceased =
      DEMO_TREE_DATA.nodes.find((n) => !n.is_living) ?? DEMO_TREE_DATA.nodes[0];
    const htmlHint = renderWithProviders(
      <QuickAddDuplicateHint
        givenName={firstDeceased.primary_name.given_name}
        surname={firstDeceased.primary_name.surname}
        existingPeople={DEMO_TREE_DATA.nodes}
        onUseExisting={() => {}}
      />,
      'en',
      'admin'
    );
    expect(htmlHint).toContain('Possible duplicate in tree');
    expect(htmlHint).toContain(firstDeceased.primary_name.given_name);
    expect(htmlHint).toContain('†');

    const htmlMerge = renderWithProviders(
      <PersonMergePage targetId="p-ivan" sourceId="p-ivan-dup" />,
      'ru',
      'admin'
    );
    expect(htmlMerge).toContain('Сравнение и объединение дубликатов');
    expect(htmlMerge).toContain('role="radiogroup"');
    expect(htmlMerge).toContain('role="radio"');
    expect(htmlMerge).toContain('Подтвердить и объединить записи');
  });

  it('renders PlacesMapView and PlacesPage in map view mode (/places?view=map)', () => {
    const htmlMap = renderWithProviders(
      <PlacesPage initialView="map" />,
      'en',
      'admin'
    );
    expect(htmlMap).toContain('data-testid="places-map-view"');
    expect(htmlMap).toContain('Geographic Places &amp; Cluster Map');
    expect(htmlMap).toContain('Clustered Map');
    expect(htmlMap).toContain('Inspect Place');
  });

  it('normalizes live backend payloads for DEF-001 through DEF-005 without uninterpolated placeholders', () => {
    // DEF-001: TreeStatsResponse & nested AnniversaryItemDto
    const htmlGlance = renderWithProviders(
      <TreeGlanceWidget
        stats={{
          persons_count: 74,
          generations_count: 6,
          media_count: 18,
          places_count: 11,
          families_count: 23,
          earliest_birth_year: 1812,
          oldest_ancestor: {
            id: 'p-1',
            display_name: 'Gavriil Volkov',
            birth_year: 1812,
          },
          missing_birth_count: 5,
          quality_summary: {
            total_open_warnings: 7,
          },
        }}
        isAdmin={true}
        onOpenQuality={() => {}}
      />,
      'en',
      'admin'
    );
    expect(htmlGlance).toContain('74');
    expect(htmlGlance).toContain('18');
    expect(htmlGlance).toContain('Gavriil Volkov');
    expect(htmlGlance).toContain('7 warning(s)');

    const htmlAnniv = renderWithProviders(
      <AnniversariesWidget
        items={[
          {
            kind: 'birthday',
            person: {
              id: 'p-vera',
              display_name: 'Vera Levshits',
              is_living: true,
            },
            years_elapsed: 40,
            next_occurrence_date: 'Nov 12',
            days_until: 5,
          },
        ]}
        scope="all"
        onScopeChange={() => {}}
        accessTier="admin"
      />,
      'en',
      'admin'
    );
    expect(htmlAnniv).toContain('Vera Levshits');
    expect(htmlAnniv).toContain('Nov 12 · Turns 40');
    expect(htmlAnniv).not.toContain('{date}');

    // DEF-002: Nested warning.person and duplicate pair.person_a / person_b + severity badges without {count}
    const htmlQualityNested = renderWithProviders(
      <DataQualityPage
        initialWarnings={[
          {
            rule_code: 'MISSING_BIRTH',
            severity: 'info',
            entity_id: 'p-nested-1',
            person: {
              id: 'p-nested-1',
              gid: 'I0777',
              display_name: 'Arkady Petrovich Volkov',
            },
          },
        ]}
        initialDuplicates={[
          {
            person_a: {
              id: 'p-a',
              gid: 'I0101',
              display_name: 'Nikolai Volkov',
            },
            person_b: {
              id: 'p-b',
              gid: 'I0102',
              display_name: 'Nikolay Volkov',
            },
            score: 0.94,
          },
        ]}
      />,
      'en',
      'admin'
    );
    expect(htmlQualityNested).toContain('Arkady Petrovich Volkov');
    expect(htmlQualityNested).toContain('Nikolai Volkov');
    expect(htmlQualityNested).toContain('94% Match');
    expect(htmlQualityNested).not.toContain('{count}');

    // DEF-003: keep_person & merge_person + events_to_transfer
    const htmlMergeNested = renderWithProviders(
      <PersonMergePage
        initialPreview={
          {
            keep_person: {
              id: 'p-keep',
              gid: 'I0500',
              display_name: 'Stepan Ivanovich Volkov',
              sex: 'M',
              birth_date: '1880',
            },
            merge_person: {
              id: 'p-merge',
              gid: 'I0501',
              display_name: 'Stepan Volkov',
              sex: 'M',
              birth_date: '1880-04-10',
            },
            events_to_transfer: [{ id: 'ev-1' }, { id: 'ev-2' }, { id: 'ev-3' }],
          } as unknown as React.ComponentProps<typeof PersonMergePage>['initialPreview']
        }
      />,
      'en',
      'admin'
    );
    expect(htmlMergeNested).toContain('Stepan Ivanovich Volkov');
    expect(htmlMergeNested).toContain('Events: 3');
    expect(htmlMergeNested).not.toContain('{count}');

    // DEF-005: normalizeRemoteKinshipResponse with label_en, label_ru, degree_of_kinship, path_nodes
    const normalizedRu = normalizeRemoteKinshipResponse(
      {
        connected: true,
        label_en: 'Grandfather',
        label_ru: 'Дедушка',
        degree_of_kinship: 2,
        steps: [
          { from_id: 'n1', to_id: 'n2', edge_type: 'parent' },
          { from_id: 'n2', to_id: 'n3', edge_type: 'parent' },
        ],
        path_nodes: [
          {
            ...DEMO_TREE_DATA.nodes[0],
            id: 'n1',
            gid: 'I0001',
            display_name: 'Елена Волкова',
          },
          {
            ...DEMO_TREE_DATA.nodes[1],
            id: 'n2',
            gid: 'I0002',
            sex: 'M',
            display_name: 'Пётр Волков',
          },
          {
            ...DEMO_TREE_DATA.nodes[0],
            id: 'n3',
            gid: 'I0003',
            sex: 'M',
            display_name: 'Иван Волков',
          },
        ],
      },
      {
        connected: true,
        relationship_label: 'Relative',
        steps: [],
      },
      DEMO_TREE_DATA.nodes,
      'ru',
      'admin',
      (k) => k
    );
    expect(normalizedRu.relationship_label).toBe('Дедушка');
    expect(normalizedRu.degree).toBe(2);
    expect(normalizedRu.steps).toHaveLength(3);
    expect(normalizedRu.steps[0].display_name).toBeTruthy();
    expect(normalizedRu.steps[0].gid).toBe('I0001');
  });

  it('satisfies UX-015 through UX-020 accessibility and visual consistency audit requirements', () => {
    // UX-015: TreeGlanceWidget clickable stat cards have role="button", tabIndex=0, aria-label, focus ring
    const htmlGlance = renderWithProviders(
      <TreeGlanceWidget
        stats={{
          total_persons: 48,
          max_generations: 5,
          total_photos: 32,
          total_places: 14,
          total_families: 19,
        }}
        isAdmin={true}
        onOpenPeople={() => {}}
        onOpenPlaces={() => {}}
      />,
      'en',
      'admin'
    );
    expect(htmlGlance).toContain('aria-label="Individuals: 48"');
    expect(htmlGlance).toContain('aria-label="Places: 14"');
    expect(htmlGlance).toContain('focus-visible:ring-2 focus-visible:ring-focus');

    // UX-016: FanChartCanvas slice <g> has group focus:outline-none and child <path> has group-focus-visible:stroke-brand
    const htmlFan = renderWithProviders(
      <FanChartCanvas
        focalPersonId="p-elena"
        nodes={DEMO_TREE_DATA.nodes}
        edges={DEMO_TREE_DATA.edges}
        selectedPersonId="p-elena"
        generations={5}
      />,
      'en',
      'admin'
    );
    expect(htmlFan).toContain('group cursor-pointer focus:outline-none focus-visible:outline-none');
    expect(htmlFan).toContain('group-focus-visible:stroke-brand group-focus-visible:stroke-[3]');

    // UX-017: PlacesMapView cluster <g> has role="button", tabIndex=0, aria-label, group-focus-visible:stroke-brand
    const htmlMap = renderWithProviders(
      <PlacesPage initialView="map" />,
      'en',
      'admin'
    );
    expect(htmlMap).toContain('group cursor-pointer focus:outline-none focus-visible:outline-none');
    expect(htmlMap).toContain('group-focus-visible:stroke-brand group-focus-visible:stroke-[4]');

    // UX-018: PersonMergePage radio cards have aria-label and mobile-only header labels (colTargetShort / colSourceShort)
    const htmlMerge = renderWithProviders(
      <PersonMergePage targetId="p-ivan" sourceId="p-ivan-dup" />,
      'en',
      'admin'
    );
    expect(htmlMerge).toContain('Keep Primary');
    expect(htmlMerge).toContain('Merge Candidate');
    expect(htmlMerge).toContain('aria-label="Keep Primary:');
    expect(htmlMerge).toContain('aria-label="Merge Candidate:');

    // UX-019: AnniversariesWidget renders prefix (†YYYY) instead of suffix (YYYY†)
    const htmlAnnivDeceased = renderWithProviders(
      <AnniversariesWidget
        items={[
          {
            id: 'ann-mem',
            type: 'memorial',
            person_id: 'p-ivan',
            display_name: 'Ivan Volkov',
            is_living: false,
            death_year: 1989,
            date_month_day: '10-30',
            date_label: 'Oct 30',
            days_until: 2,
            years_count: 37,
          },
        ]}
        scope="all"
        onScopeChange={() => {}}
        accessTier="admin"
      />,
      'en',
      'admin'
    );
    expect(htmlAnnivDeceased).toContain('(†1989)');
    expect(htmlAnnivDeceased).not.toContain('(1989†)');

    // UX-020: KinshipFinderDialog <ol data-testid="kinship-path-chain"> only contains <li> per person node, no <li aria-hidden="true">
    const htmlKinship = renderWithProviders(
      <KinshipFinderDialog
        open={true}
        onOpenChange={() => {}}
        people={DEMO_TREE_DATA.nodes}
        edges={DEMO_TREE_DATA.edges}
        initialPersonAId={DEMO_TREE_DATA.nodes[0].id}
        initialPersonBId={DEMO_TREE_DATA.nodes[1].id}
      />,
      'en',
      'admin'
    );
    expect(htmlKinship).not.toContain('<li aria-hidden="true"');
    expect(htmlKinship).toContain('<span aria-hidden="true"');
  });

  it('enforces Unauthenticated Segregation, Clean Navigation, Zero Edit Controls, and Clean Slate defaults', () => {
    // 1. Clean Unauthenticated Navigation in AppShell and BottomNav
    const unauthShellHtml = renderWithProviders(
      <AppShell currentPath="/">
        <div>Guest Content</div>
      </AppShell>,
      'en',
      null
    );
    expect(unauthShellHtml).toContain('Home');
    expect(unauthShellHtml).toContain('Tree');
    expect(unauthShellHtml).toContain('Sign in');
    expect(unauthShellHtml).not.toContain('People');
    expect(unauthShellHtml).not.toContain('Media');
    expect(unauthShellHtml).not.toContain('More');
    expect(unauthShellHtml).not.toContain('⌘K');
    expect(unauthShellHtml).not.toContain('Add unconnected person');

    const unauthBottomNavHtml = renderWithProviders(
      <BottomNav currentPath="/" />,
      'en',
      null
    );
    expect(unauthBottomNavHtml).toContain('grid-cols-3');
    expect(unauthBottomNavHtml).toContain('Home');
    expect(unauthBottomNavHtml).toContain('Tree');
    expect(unauthBottomNavHtml).toContain('Sign in');
    expect(unauthBottomNavHtml).not.toContain('People');
    expect(unauthBottomNavHtml).not.toContain('Media');

    // 2. Navigating to / in public mode renders DashboardPage in public guest mode (not TreeCanvas) with clean empty slate
    const unauthAppHomeHtml = renderWithProviders(<App />, 'en', null);
    expect(unauthAppHomeHtml).toContain('data-testid="dashboard-page"');
    expect(unauthAppHomeHtml).toContain('Public Guest View');
    expect(unauthAppHomeHtml).toContain('No upcoming anniversaries in the next 30 days');
    expect(unauthAppHomeHtml).not.toContain('Add unconnected person');
    expect(unauthAppHomeHtml).not.toContain('Undo');

    // 3. Zero edit controls in PersonDrawer, PersonBottomSheet, and PersonProfilePage when unauthenticated or viewer
    const drawerGuestHtml = renderWithProviders(
      <PersonDrawer
        open
        onOpenChange={() => {}}
        person={DEMO_TREE_DATA.nodes[0]}
        accessTier="public_anonymous"
        canEdit={false}
      />,
      'en',
      null
    );
    expect(drawerGuestHtml).not.toContain('Set as my home person');
    expect(drawerGuestHtml).not.toContain('Edit Profile');
    expect(drawerGuestHtml).not.toContain('Add Father');
    expect(drawerGuestHtml).not.toContain('Delete person record');

    const sheetViewerHtml = renderWithProviders(
      <PersonBottomSheet
        open
        onOpenChange={() => {}}
        person={DEMO_TREE_DATA.nodes[0]}
        accessTier="viewer"
        canEdit={false}
      />,
      'en',
      'viewer'
    );
    expect(sheetViewerHtml).not.toContain('Set as my home person');
    expect(sheetViewerHtml).not.toContain('Edit Profile');

    // 4. PersonProfilePage without initialProfile shows loading skeleton instead of mock data
    const loadingProfileHtml = renderWithProviders(
      <PersonProfilePage personId="unknown-id" />,
      'en',
      null
    );
    expect(loadingProfileHtml).toContain('Loading');
    expect(loadingProfileHtml).not.toContain('Alexander Mikhailovich Levshits');

    // 5. PersonProfilePage in public guest mode has zero edit controls even if backend capabilities had can_edit=true
    const profileWithFakeCaps = buildDefaultPersonProfile('I0001', 'admin');
    const guestProfileHtml = renderWithProviders(
      <PersonProfilePage
        personId="I0001"
        initialProfile={profileWithFakeCaps}
        accessTier="public_anonymous"
      />,
      'en',
      null
    );
    expect(guestProfileHtml).not.toContain('Edit Profile');
    expect(guestProfileHtml).not.toContain('Add Event');
    expect(guestProfileHtml).not.toContain('Set as my home person');

    // 6. TreeCanvas without initialTreeView renders EmptyTreeOnboardingCard (zero mock nodes injected)
    const emptyTreeCanvasHtml = renderWithProviders(
      <TreeCanvas />,
      'en',
      null
    );
    expect(emptyTreeCanvasHtml).toContain('This family tree is still being prepared.');
    expect(emptyTreeCanvasHtml).not.toContain('Alexander Mikhailovich Levshits');
  });
});
