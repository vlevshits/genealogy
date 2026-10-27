import React from 'react';
import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'node:fs';
import path from 'node:path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ConfirmProvider, ToastProvider } from '../../src/components/ui';
import { SettingsPage } from '../../src/pages/SettingsPage';
import {
  DEFAULT_GEDCOM_ANALYSIS_REPORT,
  DEFAULT_GEDCOM_IMPORT_RESULT,
  GedcomAnalysisModal,
  GedcomCleanSlateConfirm,
  GedcomExportCard,
  GedcomPostImportReport,
  GedcomSettingsPanel,
  isCleanSlateConfirmed,
} from '../../src/components/settings';
import { getGedcomExportUrl } from '../../src/api/client';

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

describe('Phase 1.10 — GEDCOM 5.5.1 + 7.0 Import & Export Settings UI', () => {
  it('mounts GedcomSettingsPanel on /settings/gedcom for Admin and restricts non-Admin roles', () => {
    const htmlAdmin = renderWithProviders(
      <SettingsPage currentPath="/settings/gedcom" userRole="admin" />,
      'en'
    );
    expect(htmlAdmin).toContain('Import GEDCOM Archive');
    expect(htmlAdmin).toContain('Export Family Tree to GEDCOM');
    expect(htmlAdmin).toContain('GEDCOM 5.5.1 (Recommended)');
    expect(htmlAdmin).toContain('GEDCOM 7.0 (Modern UTF-8)');

    const htmlEditor = renderWithProviders(
      <SettingsPage currentPath="/settings/gedcom" userRole="editor" />,
      'en'
    );
    expect(htmlEditor).not.toContain('Import GEDCOM Archive');
    expect(htmlEditor).toContain('Administrator Access Required');
  });

  it('renders GedcomAnalysisModal with Gramps Web 6.0.1 dialect highlights, counts, and warnings in EN and RU', () => {
    const htmlEn = renderWithProviders(
      <GedcomAnalysisModal
        report={DEFAULT_GEDCOM_ANALYSIS_REPORT}
        onProceed={() => {}}
        onCancel={() => {}}
      />,
      'en'
    );

    expect(htmlEn).toContain('Pre-Import Summary &amp; Dry Run');
    expect(htmlEn).toContain('Gramps Web 6.0.1');
    expect(htmlEn).toContain('GEDCOM 5.5.1');
    expect(htmlEn).toContain('284');
    expect(htmlEn).toContain('84');
    expect(htmlEn).toContain('129');
    expect(htmlEn).toContain('Geographic coordinates normalized (N/S/E/W)');
    expect(htmlEn).toContain('57');
    expect(htmlEn).toContain('339');
    expect(htmlEn).toContain('338');
    expect(htmlEn).toContain('Proceed to Clean Slate Import');

    const htmlRu = renderWithProviders(
      <GedcomAnalysisModal
        report={DEFAULT_GEDCOM_ANALYSIS_REPORT}
        onProceed={() => {}}
        onCancel={() => {}}
      />,
      'ru'
    );
    expect(htmlRu).toContain('Предварительный отчёт перед импортом');
    expect(htmlRu).toContain('Нормализовано географических координат (N/S/E/W)');
    expect(htmlRu).toContain('Перейти к полной замене древа');
  });

  it('validates Clean Slate confirmation phrases (CLEAN_SLATE / REPLACE) and renders rollback snapshot guarantee', () => {
    expect(isCleanSlateConfirmed('CLEAN_SLATE')).toBe(true);
    expect(isCleanSlateConfirmed('clean_slate')).toBe(true);
    expect(isCleanSlateConfirmed('REPLACE')).toBe(true);
    expect(isCleanSlateConfirmed(' replace ')).toBe(true);
    expect(isCleanSlateConfirmed('DELETE')).toBe(false);
    expect(isCleanSlateConfirmed('')).toBe(false);

    const htmlConfirm = renderWithProviders(
      <GedcomCleanSlateConfirm
        fileName="gramps-web-export-20261026130902.ged"
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
      'en'
    );
    expect(htmlConfirm).toContain('Confirm Clean Slate Replacement');
    expect(htmlConfirm).toContain('gramps-web-export-20261026130902.ged');
    expect(htmlConfirm).toContain('tree_snapshot');
    expect(htmlConfirm).toContain('Type CLEAN_SLATE or REPLACE to confirm');
  });

  it('renders GedcomPostImportReport with snapshot ID, missing media reconciliation CTA, Select Home Person, and Open Family Tree', () => {
    const htmlReport = renderWithProviders(
      <GedcomPostImportReport
        result={DEFAULT_GEDCOM_IMPORT_RESULT}
        onOpenTree={() => {}}
        onSelectHomePerson={() => {}}
        onUploadMissingMedia={() => {}}
        onReset={() => {}}
      />,
      'en'
    );

    expect(htmlReport).toContain('GEDCOM Import Completed');
    expect(htmlReport).toContain('snap-pre-gedcom-20261027');
    expect(htmlReport).toContain(
      '129 referenced media files are missing image binaries'
    );
    expect(htmlReport).toContain('Upload Media Folder');
    expect(htmlReport).toContain('Select Home Person');
    expect(htmlReport).toContain('Open Family Tree');
  });

  it('renders GedcomExportCard with 5.5.1/7.0 format options, privacy switches, and builds export URLs', () => {
    const htmlExport = renderWithProviders(
      <GedcomSettingsPanel />,
      'en'
    );
    expect(htmlExport).toContain('Bundle media files (.zip)');
    expect(htmlExport).toContain('Redact living individuals (initials only)');
    expect(htmlExport).toContain('Include private notes');
    expect(htmlExport).toContain('Download GEDCOM Export');

    const url = getGedcomExportUrl({
      version: '7.0',
      include_media: true,
      redact_living: true,
      include_private_notes: false,
    });
    expect(url).toBe(
      '/api/v1/gedcom/export?version=7.0&redact_living=true&include_media=true&include_private_notes=false'
    );
  });

  it('uses zero raw <button>, <input>, <select>, or <textarea> HTML elements in GEDCOM settings components', () => {
    const filesToCheck = [
      path.resolve(
        __dirname,
        '../../src/components/settings/gedcom/GedcomAnalysisModal.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/settings/gedcom/GedcomCleanSlateConfirm.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/settings/gedcom/GedcomPostImportReport.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/settings/gedcom/GedcomImportCard.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/settings/gedcom/GedcomExportCard.tsx'
      ),
      path.resolve(
        __dirname,
        '../../src/components/settings/gedcom/GedcomSettingsPanel.tsx'
      ),
    ];

    const rawTagRegex = /<(button|input|select|textarea)\b/g;

    for (const filePath of filesToCheck) {
      const source = fs.readFileSync(filePath, 'utf-8');
      const matches = source.match(rawTagRegex);
      expect(
        matches,
        `Found raw HTML form/button element in ${path.basename(filePath)}`
      ).toBeNull();
    }
  });
});
