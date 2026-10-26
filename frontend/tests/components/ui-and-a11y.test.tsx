import React from 'react';
import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'fs';
import path from 'path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import {
  ToastProvider,
  ConfirmProvider,
  BottomNav,
  AppShell,
  DataTable,
  BulkActionBar,
  Button,
} from '../../src/components/ui';
import { GenDateInput, Form, FormField, useForm } from '../../src/components/form';
import { DevUiWorkbench } from '../../src/pages/DevUiWorkbench';
import { App } from '../../src/App';
import { PersonName, LifeSpan, EventForm } from '../../src/components/domain';
import { SAMPLE_PEOPLE, generateLargePeopleDataset } from '../../src/lib/mock-data';
import { parseGenDate } from '../../src/lib/gendate-parser';

function renderWithProviders(node: React.ReactNode, locale: 'en' | 'ru' = 'en'): string {
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

describe('Component Library, Accessibility & Design Token Verification', () => {
  it('DoD #1 (Single Source): App.tsx and SystemStatusCard.tsx use reusable UI components and no raw <input>, <select>, <textarea>, or <button>', () => {
    const appFile = fs.readFileSync(path.resolve(__dirname, '../../src/App.tsx'), 'utf-8');
    const cardFile = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/SystemStatusCard.tsx'),
      'utf-8'
    );
    for (const content of [appFile, cardFile]) {
      expect(content).not.toMatch(/<(button|input|select|textarea)\b/);
    }
  });

  it('DoD #4 (Theme Tokens Only): contains zero hardcoded hex color literals in src/components/** and defines all required CSS tokens in index.css', () => {
    const componentsDir = path.resolve(__dirname, '../../src/components');
    const hexRegex = /#[0-9a-fA-F]{3,8}\b/;

    function scanDir(dir: string) {
      for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          scanDir(fullPath);
        } else if (entry.name.endsWith('.tsx') || entry.name.endsWith('.ts')) {
          const content = fs.readFileSync(fullPath, 'utf-8');
          const match = content.match(hexRegex);
          expect(
            match,
            `Found raw hex color "${match?.[0]}" in ${fullPath}`
          ).toBeNull();
        }
      }
    }

    scanDir(componentsDir);

    const indexCss = fs.readFileSync(path.resolve(__dirname, '../../src/index.css'), 'utf-8');
    expect(indexCss).toContain('--size-control-sm: 2rem');
    expect(indexCss).toContain('--size-control-md: 2.75rem');
    expect(indexCss).toContain('--size-control-lg: 3rem');
    expect(indexCss).toContain('--size-touch-min: 44px');
    expect(indexCss).toContain('--size-touch-lg: 48px');
    expect(indexCss).toContain('--inset-safe-top: env(safe-area-inset-top, 0px)');
    expect(indexCss).toContain('--inset-safe-bottom: env(safe-area-inset-bottom, 0px)');
  });

  it('DoD #5 (Anti-Fluff Copy): rendered Home page (App + AppShell + SystemStatusCard) contains zero forbidden buzzwords or tech-stack names in EN and RU', () => {
    const htmlEn = renderWithProviders(<App />, 'en');
    const htmlRu = renderWithProviders(<App />, 'ru');

    const forbidden = [
      'Next-Gen',
      'Zero-Cloud',
      'Axum',
      'React 19',
      'Tailwind',
      'PostgreSQL',
      'Swagger',
      '/dev/ui',
    ];
    for (const term of forbidden) {
      expect(htmlEn).not.toContain(term);
      expect(htmlRu).not.toContain(term);
    }
    expect(htmlEn).toContain('Private Family Archive');
    expect(htmlRu).toContain('Частный семейный архив');
  });

  it('formats PersonName per locale (RU: Surname Given Patronymic, EN: Given Patronymic Surname) and masks redacted living people', () => {
    const vera = SAMPLE_PEOPLE[1]; // Вера Ивановна Левшиц (Морозова)
    const ruHtml = renderWithProviders(<PersonName person={vera} variant="full" />, 'ru');
    expect(ruHtml).toContain('Левшиц (Морозова) Вера Ивановна');

    const enHtml = renderWithProviders(<PersonName person={vera} variant="full" />, 'en');
    expect(enHtml).toContain('Вера Ивановна Левшиц (Морозова)');

    const redacted = SAMPLE_PEOPLE[4]; // is_redacted: true, surname: Кузнецов
    const maskedEn = renderWithProviders(<PersonName person={redacted} />, 'en');
    expect(maskedEn).toContain('Living Кузнецов');
  });

  it('renders LifeSpan with computed age and living indicator', () => {
    const aleksandr = SAMPLE_PEOPLE[0]; // 1912 - 1989 (77 yrs)
    const html = renderWithProviders(
      <LifeSpan
        birthDate={aleksandr.birth_date}
        deathDate={aleksandr.death_date}
        isLiving={false}
      />,
      'en'
    );
    expect(html).toContain('1912 – 1989');
    expect(html).toContain('77 yrs');
  });

  it('DoD #7 (GenDateInput): renders live interpretation, Julian toggle, converted Gregorian date, and phrase fallback warning', () => {
    const julianVal = parseGenDate('14.07.1905 (ст. ст.)', 'ru').value;
    const htmlJulian = renderWithProviders(<GenDateInput value={julianVal} />, 'ru');
    expect(htmlJulian).toContain('27 июля 1905 г.');
    expect(htmlJulian).toContain('19050727');

    const phraseVal = parseGenDate('в детстве во время войны', 'ru').value;
    const htmlPhrase = renderWithProviders(<GenDateInput value={phraseVal} />, 'ru');
    expect(htmlPhrase).toContain('Будет сохранено как текстовая фраза');
    expect(htmlPhrase).toContain('в детстве во время войны');
  });

  it('DoD #9 (DataTable Scale): generates 50,000 rows and virtualizes DOM rendering so only visible slice is mounted', () => {
    const rows = generateLargePeopleDataset(50000);
    expect(rows).toHaveLength(50000);
    expect(rows[0].gid).toBe('I00001');
    expect(rows[49999].gid).toBe('I50000');

    const html = renderWithProviders(
      <>
        <DataTable
          data={rows}
          getRowId={(r) => r.id}
          selectable
          selectedIds={['person-1', 'person-2']}
          columns={[
            { id: 'gid', header: 'ID', accessor: (r) => r.gid },
            { id: 'name', header: 'Name', accessor: (r) => r.surname },
          ]}
        />
        <BulkActionBar selectedCount={2} onClearSelection={() => {}} />
      </>,
      'en'
    );

    expect(html).toContain('50,000 rows');
    expect(html).toContain('2 selected');
    // Verify virtualization: out of 50,000 rows, fewer than 35 role="row" elements are rendered in the DOM
    const renderedRows = html.match(/role="row"/g) ?? [];
    expect(renderedRows.length).toBeGreaterThan(5);
    expect(renderedRows.length).toBeLessThan(35);
  });

  it('DoD #10 (Form Integration): FormField and EventForm render server 422 field_errors and soft chronology warnings with ARIA attributes', () => {
    const FormHarness: React.FC = () => {
      const form = useForm({
        defaultValues: { description: 'Initial', date: '1900' },
        initialWarnings: {
          date: 'Warning: event date precedes person birth year (1912)',
        },
      });
      return (
        <Form form={form} onSubmit={() => {}}>
          <FormField
            name="description"
            label="Description"
            required
            error="Server 422: Description conflicts with locked archival record"
          >
            {({ id, invalid }) => <input id={id} aria-invalid={invalid} />}
          </FormField>
          <FormField name="date" label="Date">
            {({ id }) => <input id={id} />}
          </FormField>
        </Form>
      );
    };

    const html = renderWithProviders(
      <>
        <FormHarness />
        <EventForm onSubmit={() => {}} />
      </>,
      'en'
    );
    expect(html).toContain('Server 422: Description conflicts with locked archival record');
    expect(html).toContain('Warning: event date precedes person birth year (1912)');
    expect(html).toContain('role="alert"');
    expect(html).toContain('role="status"');
  });

  it('DoD #2 & #3 (Workbench Complete & Accessibility): renders all 10 /dev/ui sections in EN and RU with accessible roles and labels', () => {
    const htmlEn = renderWithProviders(<DevUiWorkbench />, 'en');
    const htmlRu = renderWithProviders(<DevUiWorkbench />, 'ru');

    expect(htmlEn).toContain('Design System &amp; Component Workbench');
    expect(htmlRu).toContain('Дизайн-система и стенд UI-компонентов');

    // Verify all 10 workbench showcase sections exist
    const sectionIds = [
      'id="wb-actions"',
      'id="wb-feedback"',
      'id="wb-inputs"',
      'id="wb-gendate"',
      'id="wb-form"',
      'id="wb-overlays"',
      'id="wb-navigation"',
      'id="wb-datatable"',
      'id="wb-comparison"',
      'id="wb-domain"',
    ];
    for (const id of sectionIds) {
      expect(htmlEn).toContain(id);
      expect(htmlRu).toContain(id);
    }

    // 1. Every <img ...> must have an alt attribute
    const imgTags = htmlEn.match(/<img[^>]*>/g) ?? [];
    for (const img of imgTags) {
      expect(img).toMatch(/\balt=/);
    }

    // 2. Interactive ARIA roles exist on workbench controls
    expect(htmlEn).toContain('role="radiogroup"');
    expect(htmlEn).toContain('role="progressbar"');
    expect(htmlEn).toContain('role="combobox"');
    expect(htmlEn).toContain('role="switch"');
    expect(htmlEn).toContain('role="tablist"');
    expect(htmlEn).toContain('role="tree"');
    expect(htmlEn).toContain('role="columnheader"');
  });

  it('DoD #6 & #11 (Mobile Ergonomics & Universal App Shell): renders BottomNav with 5 tabs (48x48px tap targets), safe-area insets, and Button touch targets (>=44px)', () => {
    const htmlBottomNav = renderWithProviders(<BottomNav currentPath="/tree" />, 'en');
    expect(htmlBottomNav).toContain('min-h-12 min-w-12');
    expect(htmlBottomNav).toContain('env(safe-area-inset-bottom,0px)');
    expect(htmlBottomNav).toContain('aria-current="page"');

    const htmlBtnMd = renderWithProviders(<Button size="md">Action</Button>, 'en');
    expect(htmlBtnMd).toContain('min-h-11'); // 2.75rem = 44px

    const htmlBtnSm = renderWithProviders(<Button size="sm">Compact</Button>, 'en');
    expect(htmlBtnSm).toContain('touch-target-expand');

    const htmlShell = renderWithProviders(
      <AppShell currentPath="/">
        <div>Content</div>
      </AppShell>,
      'en'
    );
    expect(htmlShell).toContain('env(safe-area-inset-top,0px)');
    expect(htmlShell).toContain('⌘K');
  });
});

