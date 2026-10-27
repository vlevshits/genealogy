import React from 'react';
import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'fs';
import path from 'path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ToastProvider, ConfirmProvider, Avatar } from '../../src/components/ui';
import {
  OmniboxDialog,
  OmniboxResultsList,
  OmniboxSelectableItem,
} from '../../src/components/search';
import { DirectoryBulkBar } from '../../src/components/directory';
import {
  PeoplePage,
  serializeDirectoryUrlParams,
} from '../../src/pages/PeoplePage';
import { SurnamesPage } from '../../src/pages/SurnamesPage';
import { SearchResultResponseDto } from '../../src/api/client';

function renderWithProviders(
  node: React.ReactNode,
  locale: 'en' | 'ru' = 'en'
): string {
  return renderToStaticMarkup(
    <ThemeProvider>
      <I18nProvider initialLocale={locale}>
        <ToastProvider>
          <ConfirmProvider>{node}</ConfirmProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}

const SAMPLE_SEARCH_RESPONSE: SearchResultResponseDto = {
  query: 'petrov',
  people: [
    {
      id: 'p-1',
      gid: 'I0001',
      display_name: 'Петров Александр Иванович',
      given_name: 'Александр',
      patronymic: 'Иванович',
      surname: 'Петров',
      sex: 'M',
      birth_year: 1912,
      death_year: 1989,
      birth_place: 'Тверь',
      avatar_url: null,
      is_living: false,
      is_private: false,
      score: 135,
    },
  ],
  places: [
    {
      id: 'pl-1',
      gid: 'P0001',
      name: 'Тверь',
      full_name: 'Тверь, Тверская губерния',
      place_type: 'city',
      events_count: 6,
      has_coordinates: true,
    },
  ],
  media: [
    {
      id: 'm-1',
      gid: 'M0001',
      title: 'Портрет семьи Петровых (1950)',
      mime_type: 'image/webp',
      file_url: '/api/v1/media/m-1/file',
      is_private: false,
    },
  ],
  actions: [
    {
      id: 'nav-people',
      label: 'People Directory',
      hint: 'Browse and filter all individuals',
      path: '/people',
      icon: 'users',
      required_role: 'viewer',
    },
  ],
  timing_ms: 4,
};

describe('Phase 1.8 — Global Omnibox Search, People Directory & Surnames Index', () => {
  it('renders OmniboxDialog with grouped People, Places, Media, Actions, and Shift+Enter tree focus hint', () => {
    const htmlEn = renderWithProviders(
      <OmniboxDialog
        open
        onOpenChange={() => {}}
        initialQuery="petrov"
        initialResults={SAMPLE_SEARCH_RESPONSE}
      />,
      'en'
    );

    expect(htmlEn).toContain('Петров Александр Иванович');
    expect(htmlEn).toContain('b. 1912 · † 1989 · Тверь');
    expect(htmlEn).toContain('Тверь, Тверская губерния');
    expect(htmlEn).toContain('Портрет семьи Петровых (1950)');
    expect(htmlEn).toContain('People Directory');
    expect(htmlEn).toContain('⇧↵');
    expect(htmlEn).toContain('to focus in tree');
    expect(htmlEn).toContain('4 ms');
  });

  it('renders OmniboxResultsList empty-query state with Recently Viewed People and Recent Searches', () => {
    const selectable: OmniboxSelectableItem[] = [
      {
        kind: 'person',
        key: 'recent-person-p-1',
        person: {
          id: 'p-1',
          gid: 'I0001',
          display_name: 'Левшиц Михаил Александрович',
          birth_year: 1888,
          death_year: 1958,
          is_living: false,
        },
      },
      {
        kind: 'recent-search',
        key: 'recent-search-иван 1890',
        queryText: 'иван 1890',
      },
    ];

    const htmlRu = renderWithProviders(
      <OmniboxResultsList
        query=""
        people={[]}
        places={[]}
        media={[]}
        actions={[]}
        recentPeople={[
          {
            id: 'p-1',
            gid: 'I0001',
            display_name: 'Левшиц Михаил Александрович',
            birth_year: 1888,
            death_year: 1958,
            is_living: false,
          },
        ]}
        recentSearches={['иван 1890']}
        selectableItems={selectable}
        activeIndex={0}
        onActiveIndexChange={() => {}}
        onSelectItem={() => {}}
        onClearRecent={() => {}}
      />,
      'ru'
    );

    expect(htmlRu).toContain('Недавно просмотренные персоны');
    expect(htmlRu).toContain('Левшиц Михаил Александрович');
    expect(htmlRu).toContain('Недавние поисковые запросы');
    expect(htmlRu).toContain('иван 1890');
  });

  it('renders PeoplePage and restores gender-folded surname & incomplete filters from URL search params', () => {
    // 1. Surname filter "Петров" matches both "Петров" and "Петрова" (13 §6 DoD #2, #7)
    const htmlPetrov = renderWithProviders(
      <PeoplePage initialSearch="?surname=Петров&born=1900-1920" />,
      'ru'
    );
    expect(htmlPetrov).toContain('Петров Александр Иванович');
    expect(htmlPetrov).toContain('Петрова (Морозова) Елена Васильевна');
    expect(htmlPetrov).not.toContain('Левшиц Михаил Александрович');
    expect(htmlPetrov).toContain('† 1989-10-22');
    // DEF-001 regression guard: translated sex labels, zero raw person.sexMale keys
    expect(htmlPetrov).toContain('Мужской');
    expect(htmlPetrov).toContain('Женский');
    expect(htmlPetrov).not.toContain('person.sexMale');
    expect(htmlPetrov).not.toContain('person.sexFemale');
    expect(htmlPetrov).not.toContain('person.sexUnknown');
    // DEF-002 regression guard: maiden name in parentheses stripped from Avatar initials
    const avatarHtml = renderWithProviders(
      <Avatar name="Петрова (Морозова) Анна" />,
      'ru'
    );
    expect(avatarHtml).toContain('ПА');
    expect(avatarHtml).not.toContain('П(');

    // 2. Incomplete filter shows only records missing birth date or parents (13 §6 DoD #8)
    const htmlIncomplete = renderWithProviders(
      <PeoplePage initialSearch="?incomplete=true" />,
      'en'
    );
    expect(htmlIncomplete).toContain('Левшиц Михаил Александрович');
    expect(htmlIncomplete).toContain('Щеглова Анна Сергеевна');
    expect(htmlIncomplete).not.toContain('Петров Александр Иванович');
    expect(htmlIncomplete).toContain('No birth date or parents');

    // 3. URL param serializer produces clean shareable query strings
    const qs = serializeDirectoryUrlParams(
      'иван',
      {
        surname: 'Петров',
        bornFrom: '1880',
        bornTo: '1920',
        sex: 'M',
        living: 'deceased',
        hasPhoto: 'yes',
        incomplete: true,
      },
      'birth_asc'
    );
    expect(qs).toContain('q=%D0%B8%D0%B2%D0%B0%D0%BD');
    expect(qs).toContain('surname=%D0%9F%D0%B5%D1%82%D1%80%D0%BE%D0%B2');
    expect(qs).toContain('born_from=1880');
    expect(qs).toContain('born_to=1920');
    expect(qs).toContain('incomplete=true');
    expect(qs).toContain('sort=birth_asc');
  });

  it('renders DirectoryBulkBar with Compare / Merge active only when 2 rows are selected', () => {
    const htmlOne = renderWithProviders(
      <DirectoryBulkBar
        selectedIds={['id-1']}
        canMerge
        onCompareMerge={() => {}}
        onExportGedcom={() => {}}
        onClearSelection={() => {}}
      />,
      'en'
    );
    expect(htmlOne).toContain('Selected: 1');
    expect(htmlOne).toContain('Select exactly 2 records to compare and merge');

    const htmlTwo = renderWithProviders(
      <DirectoryBulkBar
        selectedIds={['id-1', 'id-2']}
        canMerge
        onCompareMerge={() => {}}
        onExportGedcom={() => {}}
        onClearSelection={() => {}}
      />,
      'en'
    );
    expect(htmlTwo).toContain('Selected: 2');
    expect(htmlTwo).toContain('Compare / Merge');
    expect(htmlTwo).toContain('Export Selection to GEDCOM');
  });

  it('renders SurnamesPage with Cyrillic/Latin alphabet bar, folded variants, counts, and year spans', () => {
    const htmlRu = renderWithProviders(<SurnamesPage />, 'ru');
    expect(htmlRu).toContain('Указатель фамилий');
    expect(htmlRu).toContain('Петров / Петрова');
    expect(htmlRu).toContain('1912–1990');
    expect(htmlRu).toContain('Левшиц / Левшицъ / Levshits');
    expect(htmlRu).toContain('1888–1994');
  });

  it('contains zero raw <button>, <input>, <select>, or <textarea> HTML tags across search and directory files', () => {
    const filesToCheck = [
      'src/components/search/OmniboxDialog.tsx',
      'src/components/search/OmniboxResultsList.tsx',
      'src/components/directory/DirectoryHeader.tsx',
      'src/components/directory/DirectoryFilterBar.tsx',
      'src/components/directory/DirectoryTable.tsx',
      'src/components/directory/DirectoryCardList.tsx',
      'src/components/directory/DirectoryBulkBar.tsx',
      'src/pages/PeoplePage.tsx',
      'src/pages/SurnamesPage.tsx',
    ];

    for (const relPath of filesToCheck) {
      const fullPath = path.resolve(__dirname, '../../', relPath);
      const content = fs.readFileSync(fullPath, 'utf-8');
      expect(
        content,
        `Found raw HTML form/button element in ${relPath}`
      ).not.toMatch(/<(button|input|select|textarea)\b/);
    }
  });
});
