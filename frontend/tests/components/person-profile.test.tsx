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
  buildDefaultPersonProfile,
  PersonProfilePage,
} from '../../src/pages/PersonProfilePage';
import { NoteEditDialog } from '../../src/components/profile';

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

describe('Phase 1.7 Person Profile Page: Hero Header, Timeline & Map, Family, Media & Notes Tabs', () => {
  it('contains zero raw <button>, <input>, <select>, or <textarea> tags in components/profile/*.tsx and pages/PersonProfilePage.tsx', () => {
    const profileDir = path.resolve(__dirname, '../../src/components/profile');
    const filesToCheck = fs
      .readdirSync(profileDir)
      .filter((f) => f.endsWith('.tsx'))
      .map((f) => path.join(profileDir, f));
    filesToCheck.push(
      path.resolve(__dirname, '../../src/pages/PersonProfilePage.tsx')
    );

    for (const fullPath of filesToCheck) {
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
      expect(src, `${fullPath} should not contain raw <textarea>`).not.toMatch(
        /<textarea[\s>]/
      );
    }
  });

  it('renders PersonProfilePage Hero Header & Chronology Tab in EN and RU with kinship pill, † cross, migration map, and biography (no Living pill)', () => {
    const adminProfile = buildDefaultPersonProfile('I0001', 'admin');

    const htmlEn = renderWithProviders(
      <PersonProfilePage
        personId="I0001"
        initialProfile={adminProfile}
        initialTab="chronology"
        accessTier="admin"
      />,
      'en'
    );

    expect(htmlEn).toContain('Alexander Mikhailovich Levshits');
    expect(htmlEn).toContain('Kinship: Grandfather');
    expect(htmlEn).toContain('1912-03-19');
    expect(htmlEn).toContain('†');
    expect(htmlEn).toContain('Age 77');
    expect(htmlEn).not.toContain('>Living<');
    expect(htmlEn).toContain('View in Tree');
    expect(htmlEn).toContain('Edit Profile');
    expect(htmlEn).toContain('Add...');
    expect(htmlEn).toContain('break-normal hyphens-none');
    expect(htmlEn).toContain('Add Event');
    expect(htmlEn).toContain('Chronological Life Events');
    expect(htmlEn).toContain('Age 26');
    expect(htmlEn).toContain('Vitebsk Synagogue Metrical Book, 1912, entry #48');
    expect(htmlEn).toContain('Geographic Life Migration Map');
    expect(htmlEn).toContain('https://tile.openstreetmap.org/');
    expect(htmlEn).toContain('© OpenStreetMap contributors');
    expect(htmlEn).toContain('Biographical Narrative');
    expect(htmlEn).toContain('Early Life &amp; Engineering Career');

    const appSource = fs.readFileSync(
      path.resolve(__dirname, '../../src/App.tsx'),
      'utf-8'
    );
    expect(appSource).toContain(
      "label: activePersonName || t('profile.breadcrumbLabel')"
    );
    expect(appSource).not.toContain(
      "label: rawPersonId || t('profile.breadcrumbLabel')"
    );

    const htmlRu = renderWithProviders(
      <PersonProfilePage
        personId="I0001"
        initialProfile={adminProfile}
        initialTab="chronology"
        accessTier="admin"
      />,
      'ru'
    );

    expect(htmlRu).toContain('Родство: Дедушка');
    expect(htmlRu).toContain('Показать на древе');
    expect(htmlRu).toContain('Изменить профиль');
    expect(htmlRu).toContain('Хронология событий жизни');
    expect(htmlRu).toContain('Карта жизненного пути и миграций');
    expect(htmlRu).toContain('Биографический очерк');
  });

  it('renders Family & Relations, Media & Documents, and Notes & Sources tabs cleanly', () => {
    const adminProfile = buildDefaultPersonProfile('I0001', 'admin');

    const familyHtml = renderWithProviders(
      <PersonProfilePage
        personId="I0001"
        initialProfile={adminProfile}
        initialTab="family"
        accessTier="admin"
      />,
      'en'
    );
    expect(familyHtml).toContain('Parents &amp; Siblings');
    expect(familyHtml).toContain('Mikhail Isaakovich Levshits');
    expect(familyHtml).toContain('Spouses &amp; Children');
    expect(familyHtml).toContain('Sofia Yakovlevna Levshits');
    expect(familyHtml).toContain('Elena Alexandrovna Levshits');
    expect(familyHtml).toContain('Add Father');
    expect(familyHtml).toContain('Add Spouse');

    const mediaHtml = renderWithProviders(
      <PersonProfilePage
        personId="I0001"
        initialProfile={adminProfile}
        initialTab="media"
        accessTier="admin"
      />,
      'en'
    );
    expect(mediaHtml).toContain('Portraits &amp; Family Photographs');
    expect(mediaHtml).toContain('Alexander Levshits Portrait (1955)');
    expect(mediaHtml).toContain('Documents &amp; Archival Scans');
    expect(mediaHtml).toContain('Vitebsk Birth Certificate Scan (1912)');

    const notesHtml = renderWithProviders(
      <PersonProfilePage
        personId="I0001"
        initialProfile={adminProfile}
        initialTab="notes"
        accessTier="admin"
      />,
      'en'
    );
    expect(notesHtml).toContain('Research Notes &amp; Transcriptions');
    expect(notesHtml).toContain('Vitebsk Rabbinate Metrical Book');
    expect(notesHtml).toContain('Sources Used Summary');
    expect(notesHtml).toContain('Leningrad ZAGS Civil Registry Archive, 1938, act #214');
    expect(notesHtml).toContain('Referenced in 2 event(s)');

    const noteDialogHtml = renderWithProviders(
      <NoteEditDialog
        open
        onOpenChange={() => {}}
        entityId="I0001"
      />,
      'en'
    );
    expect(noteDialogHtml).toContain('Add Research Note');
    expect(noteDialogHtml).toContain('Note Content (Markdown)');
  });

  it('enforces Public Living Privacy Guard (single initial E., zero dates/events/notes) and Read-Only Viewer capabilities', () => {
    const publicLivingProfile = buildDefaultPersonProfile(
      'I0042',
      'public_anonymous'
    );
    const publicHtml = renderWithProviders(
      <PersonProfilePage
        personId="I0042"
        initialProfile={publicLivingProfile}
        initialTab="chronology"
        accessTier="public_anonymous"
      />,
      'en'
    );

    expect(publicHtml).toContain('Public Guest View');
    expect(publicHtml).toContain('E.');
    expect(publicHtml).toContain(
      'Living family member — personal dates, locations, and biography are protected in Public Guest View.'
    );
    expect(publicHtml).toContain('Timeline hidden for living person');
    expect(publicHtml).not.toContain('Edit Profile');
    expect(publicHtml).not.toContain('Add Event');

    // Viewer on a full profile: sees complete biography and timeline, but zero edit controls
    const viewerProfile = buildDefaultPersonProfile('I0001', 'viewer');
    const viewerHtml = renderWithProviders(
      <PersonProfilePage
        personId="I0001"
        initialProfile={viewerProfile}
        initialTab="chronology"
        accessTier="viewer"
      />,
      'en'
    );
    expect(viewerHtml).toContain('Alexander Mikhailovich Levshits');
    expect(viewerHtml).toContain('Vitebsk Synagogue Metrical Book, 1912, entry #48');
    expect(viewerHtml).not.toContain('Edit Profile');
    expect(viewerHtml).not.toContain('Add Event');
  });
});
