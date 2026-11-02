import React from 'react';
import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { AuthProvider } from '../../src/lib/auth-context';
import { ToastProvider, ConfirmProvider } from '../../src/components/ui';
import {
  PersonMergePage,
  buildDefaultMergePreview,
  computeDefaultFieldChoices,
  normalizeMergePreviewResponse,
} from '../../src/pages/PersonMergePage';
import { DataQualityPage } from '../../src/pages/DataQualityPage';
import { PersonMergePreviewDto } from '../../src/api/client';

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
});
