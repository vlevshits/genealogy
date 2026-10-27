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
  EmptyTreeOnboardingCard,
  StartWithSelfDialog,
} from '../../src/components/onboarding';
import {
  TreePrivacyPreviewCard,
  buildDefaultTreePreview,
} from '../../src/components/tree';
import { ProfileSettingsPanel } from '../../src/components/settings';
import { App } from '../../src/App';

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

describe('Phase 1.5 Empty-Tree Onboarding & Home Person UI', () => {
  it('contains zero raw <button>, <input>, <select>, or <textarea> tags across components/onboarding/*', () => {
    const onboardingDir = path.resolve(
      __dirname,
      '../../src/components/onboarding'
    );
    for (const entry of fs.readdirSync(onboardingDir)) {
      if (entry.endsWith('.tsx')) {
        const fullPath = path.join(onboardingDir, entry);
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

  it('renders EmptyTreeOnboardingCard for Admin/Editor with Start with yourself & Import GEDCOM actions vs Viewer waiting notice in EN and RU', () => {
    const adminEn = renderWithProviders(
      <EmptyTreeOnboardingCard canOnboard userRole="admin" />,
      'en'
    );
    expect(adminEn).toContain('Let&#x27;s start your family tree');
    expect(adminEn).toContain('Start with yourself');
    expect(adminEn).toContain('Import a GEDCOM file');
    expect(adminEn).toContain(
      'Coming from MyHeritage, Ancestry, Geni or another app?'
    );

    const adminRu = renderWithProviders(
      <EmptyTreeOnboardingCard canOnboard userRole="admin" />,
      'ru'
    );
    expect(adminRu).toContain('Начните составление семейного древа');
    expect(adminRu).toContain('Добавить себя');
    expect(adminRu).toContain('Импортировать GEDCOM-файл');

    const viewerEn = renderWithProviders(
      <EmptyTreeOnboardingCard canOnboard={false} userRole="viewer" />,
      'en'
    );
    expect(viewerEn).toContain('This family tree is still being prepared.');
    expect(viewerEn).not.toContain('Start with yourself');
    expect(viewerEn).not.toContain('Import a GEDCOM file');

    const viewerRu = renderWithProviders(
      <EmptyTreeOnboardingCard canOnboard={false} userRole="viewer" />,
      'ru'
    );
    expect(viewerRu).toContain('Семейное древо подготавливается.');
    expect(viewerRu).not.toContain('Добавить себя');
  });

  it('renders StartWithSelfDialog modal with Given Name, Patronymic, Surname, Sex, Birth Date, and Birth Place fields', () => {
    const dialogEn = renderWithProviders(
      <StartWithSelfDialog open onOpenChange={() => {}} />,
      'en'
    );
    expect(dialogEn).toContain('Start with Yourself');
    expect(dialogEn).toContain('Given Name');
    expect(dialogEn).toContain('Patronymic');
    expect(dialogEn).toContain('Surname');
    expect(dialogEn).toContain('Sex / Gender');
    expect(dialogEn).toContain('Birth Date');
    expect(dialogEn).toContain('Birth Place');
    expect(dialogEn).toContain('Create My Record &amp; Open Tree');

    const dialogRu = renderWithProviders(
      <StartWithSelfDialog open onOpenChange={() => {}} />,
      'ru'
    );
    expect(dialogRu).toContain('Добавить себя в семейное древо');
    expect(dialogRu).toContain('Создать карточку и открыть древо');
  });

  it('renders EmptyTreeOnboardingCard inside TreePrivacyPreviewCard and App when tree is empty, and Home Person badge + Set as my home person when populated', () => {
    const emptyTreeHtml = renderWithProviders(
      <TreePrivacyPreviewCard
        initialTreeData={{
          access_tier: 'admin',
          enable_public_access: false,
          resolved_home_person_id: null,
          nodes: [],
        }}
      />,
      'en'
    );
    expect(emptyTreeHtml).toContain('Let&#x27;s start your family tree');
    expect(emptyTreeHtml).toContain('Start with yourself');

    const appEmptyHtml = renderWithProviders(
      <App
        initialOnboardingStatus={{
          is_empty: true,
          total_individuals: 0,
          can_onboard: true,
          resolved_home_person_id: null,
          default_home_person_id: null,
          user_home_person_id: null,
          user_person_id: null,
          start_page: 'dashboard',
        }}
      />,
      'en'
    );
    expect(appEmptyHtml).toContain('Let&#x27;s start your family tree');

    const populatedTreeEn = renderWithProviders(
      <TreePrivacyPreviewCard
        initialTreeData={buildDefaultTreePreview('admin')}
      />,
      'en'
    );
    expect(populatedTreeEn).toContain('Home Person');
    expect(populatedTreeEn).toContain('Set as my home person');

    const populatedTreeRu = renderWithProviders(
      <TreePrivacyPreviewCard
        initialTreeData={buildDefaultTreePreview('admin')}
      />,
      'ru'
    );
    expect(populatedTreeRu).toContain('Ключевая персона');
    expect(populatedTreeRu).toContain('Сделать моей ключевой персоной');
  });

  it('renders Start Page preference, Home Person indicator, Clear Home Person button, and Monthly Digest switch in ProfileSettingsPanel', () => {
    const profileEn = renderWithProviders(
      <ProfileSettingsPanel
        availablePeople={buildDefaultTreePreview('admin').nodes}
      />,
      'en'
    );
    expect(profileEn).toContain('Default Start Page');
    expect(profileEn).toContain('Home Dashboard');
    expect(profileEn).toContain('My Home Person');
    expect(profileEn).toContain('Clear Home Person');
    expect(profileEn).toContain(
      'Receive monthly family anniversary digest by email'
    );

    const profileRu = renderWithProviders(
      <ProfileSettingsPanel
        availablePeople={buildDefaultTreePreview('admin').nodes}
      />,
      'ru'
    );
    expect(profileRu).toContain('Стартовая страница при входе');
    expect(profileRu).toContain('Моя ключевая персона');
    expect(profileRu).toContain('Сбросить ключевую персону');
  });
});
