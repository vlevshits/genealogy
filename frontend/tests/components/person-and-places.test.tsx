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
  DeleteImpactDialog,
  LifeEventDialog,
  PersonEditDialog,
  QuickAddRelativeDialog,
} from '../../src/components/person';
import {
  PlaceFormDialog,
  PlaceMergeDialog,
} from '../../src/components/places';
import { DEFAULT_PLACES, PlacesPage } from '../../src/pages/PlacesPage';
import { computeSlavicPatronymic } from '../../src/lib/patronymic';
import { TreePersonNodeDto } from '../../src/api/client';

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

const SAMPLE_FATHER_NODE: TreePersonNodeDto = {
  id: 'person-father-1',
  gid: 'I0001',
  sex: 'M',
  is_living: false,
  is_private: false,
  primary_name: {
    given_name: 'Иван',
    patronymic: 'Петрович',
    surname: 'Левшиц',
  },
  birth: { date: '1890-05-12', place: 'Витебск' },
  death: { date: '1965-11-03', place: 'Ленинград' },
  avatar_url: null,
  notes: 'Глава ветви.',
};

describe('Phase 1.4 Core Data Model, Person/Event Editing, Slavic Patronymics & Places UI', () => {
  it('contains zero raw <button>, <input>, <select>, or <textarea> tags in components/person/*, components/places/*, and pages/PlacesPage.tsx', () => {
    const filesToCheck: string[] = [
      path.resolve(__dirname, '../../src/pages/PlacesPage.tsx'),
    ];

    for (const dir of [
      path.resolve(__dirname, '../../src/components/person'),
      path.resolve(__dirname, '../../src/components/places'),
    ]) {
      for (const entry of fs.readdirSync(dir)) {
        if (entry.endsWith('.tsx')) {
          filesToCheck.push(path.join(dir, entry));
        }
      }
    }

    for (const file of filesToCheck) {
      const src = fs.readFileSync(file, 'utf-8');
      expect(src, `${file} should not contain raw <button>`).not.toMatch(
        /<button[\s>]/
      );
      expect(src, `${file} should not contain raw <input>`).not.toMatch(
        /<input[\s>]/
      );
      expect(src, `${file} should not contain raw <select>`).not.toMatch(
        /<select[\s>]/
      );
      expect(src, `${file} should not contain raw <textarea>`).not.toMatch(
        /<textarea[\s>]/
      );
    }
  });

  it('computes Slavic patronymics accurately across regular and irregular Cyrillic and Latin father names', () => {
    expect(computeSlavicPatronymic('Иван', 'M')).toBe('Иванович');
    expect(computeSlavicPatronymic('Иван', 'F')).toBe('Ивановна');
    expect(computeSlavicPatronymic('Сергей', 'M')).toBe('Сергеевич');
    expect(computeSlavicPatronymic('Сергей', 'F')).toBe('Сергеевна');
    expect(computeSlavicPatronymic('Илья', 'M')).toBe('Ильич');
    expect(computeSlavicPatronymic('Илья', 'F')).toBe('Ильинична');
    expect(computeSlavicPatronymic('Никита', 'M')).toBe('Никитич');
    expect(computeSlavicPatronymic('Никита', 'F')).toBe('Никитична');
    expect(computeSlavicPatronymic('Игорь', 'M')).toBe('Игоревич');
    expect(computeSlavicPatronymic('Игорь', 'F')).toBe('Игоревна');
    expect(computeSlavicPatronymic('Павел', 'M')).toBe('Павлович');
    expect(computeSlavicPatronymic('Павел', 'F')).toBe('Павловна');
    expect(computeSlavicPatronymic('Лев', 'M')).toBe('Львович');
    expect(computeSlavicPatronymic('Лев', 'F')).toBe('Львовна');
    expect(computeSlavicPatronymic('Михаил', 'M')).toBe('Михайлович');
    expect(computeSlavicPatronymic('Михаил', 'F')).toBe('Михайловна');
    expect(computeSlavicPatronymic('Василий', 'M')).toBe('Васильевич');
    expect(computeSlavicPatronymic('Василий', 'F')).toBe('Васильевна');
    expect(computeSlavicPatronymic('Ivan', 'M')).toBe('Ivanovich');
    expect(computeSlavicPatronymic('Ivan', 'F')).toBe('Ivanovna');
    expect(computeSlavicPatronymic('Mikhail', 'M')).toBe('Mikhailovich');
    expect(computeSlavicPatronymic('Mikhail', 'F')).toBe('Mikhailovna');
    expect(computeSlavicPatronymic('Иван', 'U')).toBe('');
  });

  it('renders QuickAddRelativeDialog (with auto-prefilled patronymic & surname and Link Existing mode), PersonEditDialog, and LifeEventDialog', () => {
    const quickAddRu = renderWithProviders(
      <QuickAddRelativeDialog
        open
        onOpenChange={() => {}}
        anchorPerson={SAMPLE_FATHER_NODE}
        relation="child"
        existingPeople={[SAMPLE_FATHER_NODE]}
      />,
      'ru'
    );
    expect(quickAddRu).toContain('Добавить ребёнка для: Иван Петрович Левшиц');
    expect(quickAddRu).toContain('Иванович');
    expect(quickAddRu).toContain('Левшиц');
    expect(quickAddRu).toContain('Предложено по имени отца (Иван)');
    expect(quickAddRu).toContain('Связать с существующей персоной');

    const linkModeEn = renderWithProviders(
      <QuickAddRelativeDialog
        open
        onOpenChange={() => {}}
        anchorPerson={SAMPLE_FATHER_NODE}
        relation="spouse"
        initialMode="link"
        existingPeople={[SAMPLE_FATHER_NODE]}
      />,
      'en'
    );
    expect(linkModeEn).toContain('Existing Person in Tree');
    expect(linkModeEn).toContain('Link Relative');

    const editHtml = renderWithProviders(
      <PersonEditDialog
        open
        onOpenChange={() => {}}
        person={{
          ...SAMPLE_FATHER_NODE,
          birth_surname: 'Левшиц',
          version: 3,
          alt_names: [{ name_type: 'transliteration', surname: 'Levshits' }],
        }}
      />,
      'en'
    );
    expect(editHtml).toContain('Edit Person Profile');
    expect(editHtml).toContain('Birth / Maiden Surname');
    expect(editHtml).toContain('Alternate Names (Married, Alias, Transliteration)');
    expect(editHtml).toContain('Levshits');

    const eventHtml = renderWithProviders(
      <LifeEventDialog
        open
        onOpenChange={() => {}}
        individualId={SAMPLE_FATHER_NODE.id}
        initialSourceSuggestions={['Метрическая книга, 1912, л. 34']}
      />,
      'en'
    );
    expect(eventHtml).toContain('Add Life Event');
    expect(eventHtml).toContain('Archival Source Citation');
    expect(eventHtml).toContain('Метрическая книга, 1912, л. 34');
  });

  it('renders DeleteImpactDialog with events, families, media, and notes impact summary in EN and RU', () => {
    const impactDto = {
      person_id: SAMPLE_FATHER_NODE.id,
      gid: 'I0001',
      display_name: 'Иван Петрович Левшиц',
      events_count: 5,
      families_unlinked_count: 2,
      empty_families_deleted_count: 1,
      notes_count: 3,
      media_links_count: 4,
    };

    const htmlEn = renderWithProviders(
      <DeleteImpactDialog
        open
        onOpenChange={() => {}}
        person={SAMPLE_FATHER_NODE}
        initialImpact={impactDto}
      />,
      'en'
    );
    expect(htmlEn).toContain('Delete Person Record');
    expect(htmlEn).toContain('Deletes 5 personal life event(s)');
    expect(htmlEn).toContain(
      'Unlinks from 2 family unit(s) (1 empty family record(s) removed)'
    );
    expect(htmlEn).toContain(
      'Unlinks 4 tagged photo(s) (media files are preserved)'
    );
    expect(htmlEn).toContain('Deletes 3 attached note(s)');

    const htmlRu = renderWithProviders(
      <DeleteImpactDialog
        open
        onOpenChange={() => {}}
        person={SAMPLE_FATHER_NODE}
        initialImpact={impactDto}
      />,
      'ru'
    );
    expect(htmlRu).toContain('Удаление записи персоны');
    expect(htmlRu).toContain('Будет удалено личных событий: 5');
    expect(htmlRu).toContain(
      'Будет отвязано семейных союзов: 2 (удалено пустых карточек семей: 1)'
    );
  });

  it('renders PlacesPage with hierarchy tree, full parent chain disambiguation, historical alt-names, map-disabled privacy notice, PlaceFormDialog, and PlaceMergeDialog', () => {
    const placesEn = renderWithProviders(
      <PlacesPage initialPlaces={DEFAULT_PLACES} initialMapTileUrl="" />,
      'en'
    );
    expect(placesEn).toContain('Places &amp; Historical Jurisdictions');
    expect(placesEn).toContain(
      'Тверь, Тверская губерния, Российская империя'
    );
    expect(placesEn).toContain(
      'Покровка, Новоузенский уезд, Самарская губерния, Российская империя'
    );
    expect(placesEn).toContain('Калинин (1931–1990)');
    expect(placesEn).toContain(
      'External map tiles are disabled by instance privacy settings.'
    );

    const placesRu = renderWithProviders(
      <PlacesPage initialPlaces={DEFAULT_PLACES} initialMapTileUrl="" />,
      'ru'
    );
    expect(placesRu).toContain('Места и историческое деление');
    expect(placesRu).toContain(
      'Внешние картографические тайлы отключены настройками приватности архива.'
    );

    const formHtml = renderWithProviders(
      <PlaceFormDialog
        open
        onOpenChange={() => {}}
        availableParents={DEFAULT_PLACES}
      />,
      'en'
    );
    expect(formHtml).toContain('Add Place or Jurisdiction Chain');
    expect(formHtml).toContain(
      'Comma-Separated Place Path (leaf to root)'
    );

    const mergeHtml = renderWithProviders(
      <PlaceMergeDialog
        open
        onOpenChange={() => {}}
        sourcePlace={DEFAULT_PLACES[2]}
        availableTargets={DEFAULT_PLACES}
      />,
      'en'
    );
    expect(mergeHtml).toContain('Merge Duplicate Place');
    expect(mergeHtml).toContain('Target Place to Keep');
  });
});
