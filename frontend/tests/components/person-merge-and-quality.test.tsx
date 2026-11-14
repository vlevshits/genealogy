import React from 'react';
import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import {
  ToastProvider,
  ConfirmProvider,
  Dialog,
  SegmentedControl,
  handleDialogFocusTrapKeyDown,
} from '../../src/components/ui';
import enJson from '../../src/i18n/en.json';
import ruJson from '../../src/i18n/ru.json';
import {
  PersonMergePage,
  buildDefaultMergePreview,
  computeDefaultFieldChoices,
  normalizeMergePreviewResponse,
  rankMergeCandidatesForTarget,
} from '../../src/pages/PersonMergePage';
import { DataQualityPage } from '../../src/pages/DataQualityPage';
import { PersonDrawer } from '../../src/components/tree/PersonDrawer';
import {
  ApiError,
  DuplicateEventGroupDto,
  PersonMergePreviewDto,
  QualityWarningDto,
  TreePersonNodeDto,
} from '../../src/api/client';
import {
  buildCustomEventMergePayload,
  buildDefaultDuplicateEventGroups,
  DuplicateEventCard,
  DuplicateEventCustomizeModal,
  isOfflineFallbackError,
} from '../../src/components/quality';

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
        <AuthProvider initialUser={initialUser}>
          <ToastProvider>
            <ConfirmProvider>{node}</ConfirmProvider>
          </ToastProvider>
        </AuthProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

describe('PersonMergePage & DataQualityPage Fixes', () => {
  describe('computeDefaultFieldChoices', () => {
    it('chooses side with value when one side is empty or dash', () => {
      const preview: PersonMergePreviewDto = {
        target: {
          id: 'p1',
          gid: 'I001',
          sex: 'M',
          is_living: false,
          is_private: false,
          primary_name: { given_name: 'Alexander', surname: 'Kalman' },
          birth: { date: '1905-06-12', place: 'Odessa' },
          death: { date: null, place: null },
          biography: 'Architect and builder.',
          notes: null,
          avatar_url: null,
        },
        source: {
          id: 'p2',
          gid: 'I002',
          sex: 'U',
          is_living: false,
          is_private: false,
          primary_name: { given_name: 'Alexander', surname: 'Kalman' },
          birth: { date: '—', place: null },
          death: { date: '1978-01-20', place: 'Kyiv' },
          biography: null,
          notes: null,
          avatar_url: null,
        },
        combined_counts: { events: 2, media: 1, families: 1, notes: 0 },
      };

      const choices = computeDefaultFieldChoices(preview);

      // Target has birth_date & birth_place; Source has empty ('—' / null) -> pick target
      expect(choices.birth_date).toBe('target');
      expect(choices.birth_place).toBe('target');

      // Source has death_date; Target has null -> pick source
      expect(choices.death_date).toBe('source');

      // Target has biography; Source has null -> pick target
      expect(choices.biography).toBe('target');

      // Target has sex 'M'; Source has 'U' -> pick target
      expect(choices.sex).toBe('target');
    });

    it('prefers more specific date over approximate or year-only date', () => {
      const preview: PersonMergePreviewDto = {
        target: {
          id: 'p1',
          gid: 'I001',
          sex: 'F',
          is_living: false,
          is_private: false,
          primary_name: { given_name: 'Tatiana', surname: 'Shurubeiko' },
          birth: { date: 'abt 1890', place: null },
          death: { date: '1965-04-12', place: 'Minsk' },
          biography: null,
          notes: null,
          avatar_url: null,
        },
        source: {
          id: 'p2',
          gid: 'I002',
          sex: 'F',
          is_living: false,
          is_private: false,
          primary_name: { given_name: 'Tatiana', surname: 'Shurubeiko' },
          birth: { date: '1890-11-03', place: 'Grodno' },
          death: { date: '1965', place: 'Minsk' },
          biography: null,
          notes: null,
          avatar_url: null,
        },
        combined_counts: { events: 0, media: 0, families: 0, notes: 0 },
      };

      const choices = computeDefaultFieldChoices(preview);

      // Source has exact date ('1890-11-03') vs Target approx ('abt 1890') -> pick source
      expect(choices.birth_date).toBe('source');

      // Target has exact date ('1965-04-12') vs Source year-only ('1965') -> pick target
      expect(choices.death_date).toBe('target');

      // Both empty biography -> defaults to target
      expect(choices.biography).toBe('target');
    });
  });

  describe('normalizeMergePreviewResponse - mock data leakage prevention', () => {
    it('does not leak mock 19th-century merchant biographies into real people with null biography', () => {
      const rawApiPayload = {
        target: {
          id: 'p-real-1',
          gid: 'I0100',
          display_name: 'Alexander Kalman',
          sex: 'M',
          birth_date: '1905',
          biography: null,
          notes: null,
        },
        source: {
          id: 'p-real-2',
          gid: 'I0101',
          display_name: 'Alexander Kalman',
          sex: 'M',
          birth_date: '1905-06-12',
          biography: null,
          notes: null,
        },
      };

      const normalized = normalizeMergePreviewResponse(
        rawApiPayload as unknown as Partial<PersonMergePreviewDto> &
        Record<string, unknown>,
        'p-real-1',
        'p-real-2'
      );

      // Neither target nor source should have the mock Merchant strings
      expect(normalized.target.biography).toBeNull();
      expect(normalized.target.notes).toBeNull();
      expect(normalized.source.biography).toBeNull();
      expect(normalized.source.notes).toBeNull();

      // Render PersonMergePage with this preview and ensure mock strings are absent
      const html = renderWithProviders(
        <PersonMergePage initialPreview={normalized} />,
        'en',
        'admin'
      );

      expect(html).not.toContain('Merchant of the 2nd Guild');
      expect(html).not.toContain('Parish birth register entry');
      expect(html).toContain('Alexander Kalman');
    });
  });

  describe('PersonMergePage profile inspection buttons', () => {
    it('renders profile buttons for target and source with accessible attributes and URLs in English', () => {
      const preview = {
        ...buildDefaultMergePreview('target-person-uuid-1', 'source-person-uuid-2'),
      };

      const html = renderWithProviders(
        <PersonMergePage initialPreview={preview} />,
        'en',
        'admin'
      );

      // Desktop table header profile buttons
      expect(html).toContain('data-testid="merge-target-open-profile-btn"');
      expect(html).toContain('data-url="/persons/target-person-uuid-1"');
      expect(html).toContain('aria-label="Open profile for Pyotr Fyodorovich Shcheglov in new tab"');
      expect(html).toContain('Profile');

      expect(html).toContain('data-testid="merge-source-open-profile-btn"');
      expect(html).toContain('data-url="/persons/source-person-uuid-2"');
      expect(html).toContain('aria-label="Open profile for Pyotr Shcheglov in new tab"');

      // Responsive chip buttons for mobile/desktop
      expect(html).toContain('data-testid="merge-target-open-profile-chip-btn"');
      expect(html).toContain('data-testid="merge-source-open-profile-chip-btn"');
      expect(html).toContain('Open Profile');
    });

    it('renders localized profile buttons and aria labels in Russian', () => {
      const preview = {
        ...buildDefaultMergePreview('target-person-uuid-1', 'source-person-uuid-2'),
      };

      const html = renderWithProviders(
        <PersonMergePage initialPreview={preview} />,
        'ru',
        'admin'
      );

      // Russian header button labels and aria
      expect(html).toContain('data-testid="merge-target-open-profile-btn"');
      expect(html).toContain('aria-label="Открыть профиль Pyotr Fyodorovich Shcheglov в новой вкладке"');
      expect(html).toContain('Профиль');

      expect(html).toContain('data-testid="merge-source-open-profile-btn"');
      expect(html).toContain('aria-label="Открыть профиль Pyotr Shcheglov в новой вкладке"');

      // Russian chip button labels
      expect(html).toContain('data-testid="merge-target-open-profile-chip-btn"');
      expect(html).toContain('data-testid="merge-source-open-profile-chip-btn"');
      expect(html).toContain('Открыть профиль');
    });
  });

  describe('DataQualityPage duplicate count update', () => {
    it('displays 0 duplicates when initialDuplicates is empty array', () => {
      const html = renderWithProviders(
        <DataQualityPage initialDuplicates={[]} />,
        'en',
        'admin'
      );

      expect(html).toContain('Duplicates (0)');
    });

    it('passes returnTo=/quality?tab=duplicates in compare and merge button link', () => {
      const sampleDuplicates = [
        {
          id: 'dup-pair-1',
          person_a: {
            id: 'p-1',
            gid: 'I001',
            full_name: 'Aleksandr Kalman',
            primary_name: { given_name: 'Aleksandr', surname: 'Kalman' },
          },
          person_b: {
            id: 'p-2',
            gid: 'I002',
            full_name: 'Aleksandr Kalman',
            primary_name: { given_name: 'Aleksandr', surname: 'Kalman' },
          },
          score: 85,
          confidence: 'high',
          reasons: ['name_match'],
        },
      ];

      const html = renderWithProviders(
        <DataQualityPage initialDuplicates={sampleDuplicates as any} />,
        'ru',
        'admin'
      );

      // Verify returnTo parameter is included in the merge URL
      expect(html).toContain('returnTo=%2Fquality%3Ftab%3Dduplicates');
    });
  });

  describe('Manual 2-Person Merge Selector Bar & Tree Drawer Merge Action', () => {
    const sampleNodes: TreePersonNodeDto[] = [
      {
        id: 'p-antonina-1',
        gid: 'I0016',
        sex: 'F',
        is_living: false,
        is_private: false,
        primary_name: {
          given_name: 'Antonina',
          patronymic: 'Ivanovna',
          surname: 'Petrova',
        },
        birth: { date: '1912-03-14', place: 'Tver' },
        death: null,
        avatar_url: null,
        notes: null,
      },
      {
        id: 'p-antonina-dup',
        gid: 'I0099',
        sex: 'F',
        is_living: false,
        is_private: false,
        primary_name: {
          given_name: 'Antonina',
          patronymic: null,
          surname: 'Petrova',
        },
        birth: { date: '1912', place: 'Tver Governorate' },
        death: { date: '1988', place: 'Moscow' },
        avatar_url: null,
        notes: null,
      },
      {
        id: 'p-other',
        gid: 'I0050',
        sex: 'M',
        is_living: false,
        is_private: false,
        primary_name: {
          given_name: 'Nikolai',
          patronymic: 'Petrovich',
          surname: 'Smirnov',
        },
        birth: { date: '1880', place: 'Kazan' },
        death: null,
        avatar_url: null,
        notes: null,
      },
    ];

    it('ranks similar duplicate candidates highest and renders PersonMergeSelectorBar in EN and RU', () => {
      const ranked = rankMergeCandidatesForTarget(sampleNodes[0], sampleNodes);
      expect(ranked).toHaveLength(2);
      expect(ranked[0].person.id).toBe('p-antonina-dup');
      expect(ranked[0].score).toBeGreaterThanOrEqual(40);

      const htmlEn = renderWithProviders(
        <PersonMergePage
          targetId="p-antonina-1"
          initialPeople={sampleNodes}
        />,
        'en',
        'admin'
      );
      expect(htmlEn).toContain('data-testid="person-merge-selector-bar"');
      expect(htmlEn).toContain('data-testid="merge-swap-direction-btn"');
      expect(htmlEn).toContain('data-testid="merge-candidate-search-input"');
      expect(htmlEn).toContain('Select Profiles to Compare &amp; Merge');
      expect(htmlEn).toContain('Swap Primary &amp; Second');

      const htmlRu = renderWithProviders(
        <PersonMergePage
          targetId="p-antonina-1"
          initialPeople={sampleNodes}
        />,
        'ru',
        'admin'
      );
      expect(htmlRu).toContain('Выбор профилей для сравнения и объединения');
      expect(htmlRu).toContain('Поменять местами');
    });

    it('renders Merge with duplicate button in PersonDrawer Quick Actions', () => {
      const htmlDrawer = renderWithProviders(
        <PersonDrawer
          open={true}
          onOpenChange={() => { }}
          person={sampleNodes[0]}
          accessTier="admin"
          canEdit={true}
        />,
        'en',
        'admin'
      );
      expect(htmlDrawer).toContain('data-testid="drawer-merge-person-btn"');
      expect(htmlDrawer).toContain('Merge with duplicate...');
    });
  });

  describe('Duplicate Life & Family Events Quality Sub-View', () => {
    it('renders default individual birth and couple marriage duplicate event groups in EN and RU', () => {
      const htmlEn = renderWithProviders(<DataQualityPage />, 'en', 'admin');
      expect(htmlEn).toContain('data-testid="duplicate-events-section"');
      expect(htmlEn).toContain('data-testid="batch-merge-events-btn"');
      expect(htmlEn).toContain('Auto-Merge All (1)');
      expect(htmlEn).toContain('Duplicate Life &amp; Family Events');
      expect(htmlEn).toContain('Birth');
      expect(htmlEn).toContain('Marriage');
      expect(htmlEn).toContain('Ready to Merge');
      expect(htmlEn).toContain('Review Conflict');
      expect(htmlEn).toContain('Duplicate Events: 2');
      expect(htmlEn).toContain(
        'data-testid="merge-duplicate-event-btn-dup-evt-sample-1"'
      );
      expect(htmlEn).toContain(
        'data-testid="customize-duplicate-event-btn-dup-evt-sample-1"'
      );
      expect(htmlEn).toContain(
        'data-testid="dismiss-duplicate-event-btn-dup-evt-sample-1"'
      );

      const htmlRu = renderWithProviders(<DataQualityPage />, 'ru', 'admin');
      expect(htmlRu).toContain('Дубликаты событий жизни и семьи');
      expect(htmlRu).toContain('Объединить все (1)');
      expect(htmlRu).toContain('Рождение');
      expect(htmlRu).toContain('Брак / Венчание');
      expect(htmlRu).toContain('Готово к объединению');
      expect(htmlRu).toContain('Требует проверки');
      expect(htmlRu).toContain('Дубликаты событий: 2');
    });

    it('preserves Duplicates (0) count when initialDuplicates={[]} is passed and renders empty state', () => {
      const htmlEn = renderWithProviders(
        <DataQualityPage
          initialWarnings={[]}
          initialDuplicates={[]}
          initialFilter="duplicates"
        />,
        'en',
        'admin'
      );
      expect(htmlEn).toContain('Duplicates (0)');
      expect(htmlEn).toContain('No duplicate events detected');
      expect(htmlEn).not.toContain('data-testid="batch-merge-events-btn"');

      const htmlRu = renderWithProviders(
        <DataQualityPage
          initialWarnings={[]}
          initialDuplicates={[]}
          initialFilter="duplicates"
        />,
        'ru',
        'admin'
      );
      expect(htmlRu).toContain('Дубликаты (0)');
      expect(htmlRu).toContain('Дубликатов событий не обнаружено');
    });

    it('renders DUPLICATE_EVENT warning action and custom death/marriage groups accurately', () => {
      const defaults = buildDefaultDuplicateEventGroups();
      const customGroups: DuplicateEventGroupDto[] = [
        {
          ...defaults[1],
          id: 'dup-evt-death-1',
          event_type: 'death',
          scope: 'individual',
          partner: null,
          has_conflict: true,
          conflict_fields: ['date_text', 'place_full_name'],
          confidence: 'medium',
          events: [
            {
              id: 'ev-d1',
              gid: 'E0901',
              event_type: 'death',
              individual_id: 'sample-ivan-petrov',
              date_text: '10 NOV 2004',
              place_full_name: 'Moscow, Russia',
              media_count: 1,
              is_private: false,
            },
            {
              id: 'ev-d2',
              gid: 'E0902',
              event_type: 'death',
              individual_id: 'sample-ivan-petrov',
              date_text: 'NOV 2004',
              place_full_name: 'Moscow',
              media_count: 1,
              is_private: false,
            },
          ],
          proposed: {
            keep_event_id: 'ev-d1',
            keep_event_gid: 'E0901',
            event_type: 'death',
            date_text: '10 NOV 2004',
            place_full_name: 'Moscow, Russia',
            combined_media_count: 2,
          },
        },
      ];

      const warnings: QualityWarningDto[] = [
        {
          id: 'w-dup-evt-1',
          rule_code: 'DUPLICATE_EVENT',
          code: 'DUPLICATE_EVENT',
          severity: 'warning',
          entity_id: 'dup-evt-death-1',
          person: {
            id: 'sample-ivan-petrov',
            gid: 'I0001',
            display_name: 'Ivan Petrov',
          },
          message: 'Duplicate death events',
        },
      ];

      const htmlEn = renderWithProviders(
        <DataQualityPage
          initialWarnings={warnings}
          initialDuplicates={[]}
          initialDuplicateEvents={customGroups}
          initialFilter="all"
        />,
        'en',
        'admin'
      );
      expect(htmlEn).toContain('Duplicate Life or Family Event');
      expect(htmlEn).toContain(
        'data-testid="warning-review-dup-events-btn-dup-evt-death-1"'
      );
      expect(htmlEn).toContain('Review Duplicates');
      expect(htmlEn).toContain('data-testid="duplicate-event-card-dup-evt-death-1"');
      expect(htmlEn).toContain('2 combined media items');
      expect(htmlEn).not.toContain('data-testid="batch-merge-events-btn"');

      const htmlRu = renderWithProviders(
        <DataQualityPage
          initialWarnings={warnings}
          initialDuplicates={[]}
          initialDuplicateEvents={customGroups}
          initialFilter="all"
        />,
        'ru',
        'admin'
      );
      expect(htmlRu).toContain('Дубликат события жизни или семьи');
      expect(htmlRu).toContain(
        'data-testid="warning-review-dup-events-btn-dup-evt-death-1"'
      );
      expect(htmlRu).toContain('Перейти к дубликатам');
      expect(htmlRu).toContain('Объединено: 2 медиафайла');
      expect(htmlRu).toContain('Смерть');

      const eventIdPrecedenceHtml = renderWithProviders(
        <DataQualityPage
          initialWarnings={[{ ...warnings[0], entity_id: 'ev-d1' }]}
          initialDuplicates={[]}
          initialDuplicateEvents={[defaults[0], customGroups[0]]}
          initialFilter="all"
        />,
        'en',
        'admin'
      );
      expect(eventIdPrecedenceHtml).toContain(
        'data-testid="warning-review-dup-events-btn-dup-evt-death-1"'
      );
      expect(eventIdPrecedenceHtml).not.toContain(
        'data-testid="warning-review-dup-events-btn-dup-evt-sample-1"'
      );
    });

    it('explicitly sends empty strings to clear fields and omits place_id when place is cleared or modified in buildCustomEventMergePayload', () => {
      const defaults = buildDefaultDuplicateEventGroups();
      const group: DuplicateEventGroupDto = {
        ...defaults[0],
        events: [
          {
            ...defaults[0].events[0],
            place_id: 'place-yaroslavl',
            place_full_name: 'Yaroslavl, Russia',
          },
          {
            ...defaults[0].events[1],
            place_id: 'place-yaroslavl-short',
            place_full_name: 'Yaroslavl',
          },
        ],
        proposed: {
          ...defaults[0].proposed,
          place_id: 'place-yaroslavl',
          place_full_name: 'Yaroslavl, Russia',
        },
      };

      const clearedPayload = buildCustomEventMergePayload(group, {
        keepEventId: group.events[0].id,
        dateText: '   ',
        placeName: '',
        description: '',
        sourceText: '  ',
      });

      expect(clearedPayload).toEqual({
        group_id: group.id,
        keep_event_id: group.events[0].id,
        merge_event_ids: [group.events[1].id],
        date_text: '',
        place_name: '',
        description: '',
        source_text: '',
      });
      expect(clearedPayload.place_id).toBeUndefined();

      const matchedPlacePayload = buildCustomEventMergePayload(group, {
        keepEventId: group.events[1].id,
        dateText: '12 MAY 1938',
        placeName: 'Yaroslavl',
        description: 'Note',
        sourceText: 'Archive',
      });
      expect(matchedPlacePayload.place_id).toBe('place-yaroslavl-short');
      expect(matchedPlacePayload.place_name).toBe('Yaroslavl');
      expect(matchedPlacePayload.merge_event_ids).toEqual([group.events[0].id]);

      const customPlacePayload = buildCustomEventMergePayload(group, {
        keepEventId: group.events[0].id,
        dateText: '12 MAY 1938',
        placeName: 'Kostroma, Russia',
        description: '',
        sourceText: '',
      });
      expect(customPlacePayload.place_id).toBeUndefined();
      expect(customPlacePayload.place_name).toBe('Kostroma, Russia');
    });

    it('renders AccessDeniedCard and omits duplicate events UI for non-admin viewer and editor roles', () => {
      for (const role of ['viewer', 'editor'] as const) {
        const htmlEn = renderWithProviders(<DataQualityPage />, 'en', role);
        expect(htmlEn).not.toContain('data-testid="duplicate-events-section"');
        expect(htmlEn).toContain('Administrator Access Required');

        const htmlRu = renderWithProviders(<DataQualityPage />, 'ru', role);
        expect(htmlRu).not.toContain('data-testid="duplicate-events-section"');
        expect(htmlRu).toContain('Требуются права администратора');
      }
    });

    it('DEF-003: does not swallow HTTP 404 or 4xx API errors as offline fallbacks', () => {
      expect(
        isOfflineFallbackError(
          new ApiError(404, { code: 'not_found', message: 'Not found' })
        )
      ).toBe(false);
      expect(
        isOfflineFallbackError(
          new ApiError(400, { code: 'validation_error', message: 'Invalid' })
        )
      ).toBe(false);
      expect(
        isOfflineFallbackError(
          new ApiError(0, { code: 'request_failed', message: 'Network offline' })
        )
      ).toBe(true);
      expect(
        isOfflineFallbackError(
          new ApiError(503, { code: 'network_error', message: 'Network error' })
        )
      ).toBe(true);
    });

    it('UX-128 & UX-129: renders field-level conflict highlights, primary Customize CTA, and localized rationale in EN & RU', () => {
      const defaultsEn = buildDefaultDuplicateEventGroups('en');
      const defaultsRu = buildDefaultDuplicateEventGroups('ru');
      const conflictEn = defaultsEn[1];
      const conflictRu = defaultsRu[1];

      expect(JSON.stringify(defaultsRu)).not.toMatch(/\b(MAY|JUN|Ivan Petrov)\b/);
      expect(JSON.stringify(defaultsRu)).not.toContain(' / ');

      const cardEn = renderWithProviders(
        <DuplicateEventCard
          group={conflictEn}
          onMerge={() => undefined}
          onCustomize={() => undefined}
          onDismiss={() => undefined}
        />,
        'en',
        'admin'
      );
      expect(cardEn).toContain('Conflict');
      expect(cardEn).toContain('bg-warning/10');
      expect(cardEn).toContain('border-warning/40');
      expect(cardEn).toContain(
        '2 matching records of Marriage for Ivan Petrov &amp; Elena Sokolova differ in: date, place.'
      );
      // On conflict cards, Customize is promoted to primary (bg-brand) and Merge is secondary
      expect(cardEn).toMatch(
        /class="[^"]*bg-brand[^"]*"[^>]*data-testid="customize-duplicate-event-btn-dup-evt-sample-2"/
      );
      expect(cardEn).toMatch(
        /class="[^"]*border-border[^"]*"[^>]*data-testid="merge-duplicate-event-btn-dup-evt-sample-2"/
      );

      const safeEn = renderWithProviders(
        <DuplicateEventCard
          group={defaultsEn[0]}
          onMerge={() => undefined}
          onCustomize={() => undefined}
          onDismiss={() => undefined}
        />,
        'en',
        'admin'
      );
      // On non-conflict cards, Merge remains primary (bg-brand)
      expect(safeEn).toMatch(
        /class="[^"]*bg-brand[^"]*"[^>]*data-testid="merge-duplicate-event-btn-dup-evt-sample-1"/
      );

      const cardRu = renderWithProviders(
        <DuplicateEventCard
          group={conflictRu}
          onMerge={() => undefined}
          onCustomize={() => undefined}
          onDismiss={() => undefined}
        />,
        'ru',
        'admin'
      );
      expect(cardRu).toContain('Расхождение');
      expect(cardRu).toContain('2 записи события «Брак / Венчание» для');
      expect(cardRu).toContain('имеют расхождения: дата, место.');
      expect(cardRu).not.toContain('differ in');
      expect(cardRu).not.toContain('matching records');
    });

    it('UX-134, UX-133, UX-135 & UX-136: renders Combine Sources chip, accessible button sizes, zero-count neutral badge, and appearance i18n key', () => {
      const defaults = buildDefaultDuplicateEventGroups('en');
      const groupWithSources: DuplicateEventGroupDto = {
        ...defaults[1],
        events: [
          { ...defaults[1].events[0], source_text: 'Church register 1962' },
          { ...defaults[1].events[1], source_text: 'Family album notes' },
        ],
      };

      const modalHtml = renderWithProviders(
        <DuplicateEventCustomizeModal
          open={true}
          onOpenChange={() => undefined}
          group={groupWithSources}
          onSubmit={() => undefined}
        />,
        'en',
        'admin'
      );
      expect(modalHtml).toContain('data-testid="combine-sources-chip"');
      expect(modalHtml).toContain('Combine sources');
      expect(modalHtml).toContain('gap-2.5');
      expect(modalHtml).toContain('min-h-8');
      expect(modalHtml).toMatch(
        /class="[^"]*min-h-11[^"]*"[^>]*data-testid="confirm-customize-merge-event-btn"/
      );

      const zeroHtml = renderWithProviders(
        <DataQualityPage
          initialWarnings={[]}
          initialDuplicates={[]}
          initialDuplicateEvents={[]}
          initialFilter="all"
        />,
        'en',
        'admin'
      );
      expect(zeroHtml).toContain('Duplicate Events: 0');
      expect(zeroHtml).toMatch(
        /bg-surface-muted text-secondary border border-border[^>]*>.*?Duplicate Events: 0/
      );
      expect(zeroHtml).not.toContain('data-testid="batch-merge-events-btn"');

      expect(enJson.settings.tabs.appearance).toBeTruthy();
      expect(ruJson.settings.tabs.appearance).toBeTruthy();
    });

    it('UX-127 & UX-131: Dialog renders tabIndex=-1 focus container, handleDialogFocusTrapKeyDown wraps focus cleanly, and SegmentedControl prevents label wrapping', () => {
      const dialogHtml = renderWithProviders(
        <Dialog open={true} onOpenChange={() => undefined} title="Test Dialog">
          <div>Body</div>
        </Dialog>,
        'en',
        'admin'
      );
      expect(dialogHtml).toContain('tabindex="-1"');
      expect(dialogHtml).toContain('min-h-10 min-w-10');

      const segHtml = renderWithProviders(
        <SegmentedControl
          value="all"
          onChange={() => undefined}
          options={[
            { value: 'all', label: 'All (3)' },
            { value: 'places_hierarchy', label: 'Places (0)' },
          ]}
        />,
        'en',
        'admin'
      );
      expect(segHtml).toContain('whitespace-nowrap');
      expect(segHtml).toContain('min-h-10');

      // Verify focus trap wrap helper in both forward (Tab) and backward (Shift+Tab) directions
      let focusedId = '';
      const makeMockEl = (id: string) => ({
        id,
        hasAttribute: () => false,
        getAttribute: () => null,
        closest: () => null,
        focus: () => {
          focusedId = id;
        },
      });
      const firstEl = makeMockEl('first');
      const lastEl = makeMockEl('last');
      const mockContainer = {
        querySelectorAll: () => [firstEl, lastEl],
        contains: (el: unknown) => el === firstEl || el === lastEl,
        focus: () => {
          focusedId = 'container';
        },
      } as unknown as HTMLElement;

      let preventedForward = false;
      const wrappedForward = handleDialogFocusTrapKeyDown(
        {
          key: 'Tab',
          shiftKey: false,
          preventDefault: () => {
            preventedForward = true;
          },
        },
        mockContainer,
        lastEl as unknown as Element
      );
      expect(wrappedForward).toBe(true);
      expect(preventedForward).toBe(true);
      expect(focusedId).toBe('first');

      let preventedBackward = false;
      const wrappedBackward = handleDialogFocusTrapKeyDown(
        {
          key: 'Tab',
          shiftKey: true,
          preventDefault: () => {
            preventedBackward = true;
          },
        },
        mockContainer,
        firstEl as unknown as Element
      );
      expect(wrappedBackward).toBe(true);
      expect(preventedBackward).toBe(true);
      expect(focusedId).toBe('last');
    });
  });
});
