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
  DEFAULT_RECONCILE_REPORT,
  GedcomAnalysisModal,
  GedcomCleanSlateConfirm,
  GedcomExportCard,
  GedcomMediaReconcileCard,
  GedcomPostImportReport,
  GedcomSettingsPanel,
  isCleanSlateConfirmed,
} from '../../src/components/settings';
import { getGedcomExportUrl } from '../../src/api/client';
import { formatGedcomWarning } from '../../src/lib/error-formatter';

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

  it('renders GedcomAnalysisModal with Gramps Web 6.0.1 dialect highlights, import mode selector (merge vs clean_slate), counts, and warnings in EN and RU', () => {
    const htmlMergeEn = renderWithProviders(
      <GedcomAnalysisModal
        report={DEFAULT_GEDCOM_ANALYSIS_REPORT}
        importMode="merge"
        onProceed={() => { }}
        onCancel={() => { }}
      />,
      'en'
    );

    expect(htmlMergeEn).toContain('Pre-Import Summary &amp; Dry Run');
    expect(htmlMergeEn).toContain('Gramps Web 6.0.1');
    expect(htmlMergeEn).toContain('GEDCOM 5.5.1');
    expect(htmlMergeEn).toContain('284');
    expect(htmlMergeEn).toContain('84');
    expect(htmlMergeEn).toContain('129');
    expect(htmlMergeEn).toContain('Geographic coordinates normalized (N/S/E/W)');
    expect(htmlMergeEn).toContain('57');
    expect(htmlMergeEn).toContain('339');
    expect(htmlMergeEn).toContain('338');
    expect(htmlMergeEn).toContain('role="radiogroup"');
    expect(htmlMergeEn).toContain('Merge &amp; Enrich Existing Tree');
    expect(htmlMergeEn).toContain('Recommended');
    expect(htmlMergeEn).toContain('Non-destructive');
    expect(htmlMergeEn).toContain('Full Replace (Clean Slate)');
    expect(htmlMergeEn).toContain('Destructive');
    expect(htmlMergeEn).toContain('Safe Non-Destructive Merge');
    expect(htmlMergeEn).toContain('Merge into Tree');

    const htmlCleanSlateEn = renderWithProviders(
      <GedcomAnalysisModal
        report={DEFAULT_GEDCOM_ANALYSIS_REPORT}
        importMode="clean_slate"
        onProceed={() => { }}
        onCancel={() => { }}
      />,
      'en'
    );
    expect(htmlCleanSlateEn).toContain('Full Replace Requires Confirmation');
    expect(htmlCleanSlateEn).toContain('Proceed to Full Replace...');

    const htmlMergeRu = renderWithProviders(
      <GedcomAnalysisModal
        report={DEFAULT_GEDCOM_ANALYSIS_REPORT}
        importMode="merge"
        onProceed={() => { }}
        onCancel={() => { }}
      />,
      'ru'
    );
    expect(htmlMergeRu).toContain('Предварительный отчёт перед импортом');
    expect(htmlMergeRu).toContain('Нормализовано географических координат (N/S/E/W)');
    expect(htmlMergeRu).toContain('Объединить и дополнить древо');
    expect(htmlMergeRu).toContain('Рекомендуется');
    expect(htmlMergeRu).toContain('Без удаления данных');
    expect(htmlMergeRu).toContain('Объединить с древом');

    const htmlCleanSlateRu = renderWithProviders(
      <GedcomAnalysisModal
        report={DEFAULT_GEDCOM_ANALYSIS_REPORT}
        importMode="clean_slate"
        onProceed={() => { }}
        onCancel={() => { }}
      />,
      'ru'
    );
    expect(htmlCleanSlateRu).toContain('Полная замена требует подтверждения');
    expect(htmlCleanSlateRu).toContain('Перейти к полной замене...');
  });

  it('validates Clean Slate confirmation phrases (CLEAN_SLATE / REPLACE / ЗАМЕНИТЬ) and renders rollback snapshot guarantee', () => {
    expect(isCleanSlateConfirmed('CLEAN_SLATE')).toBe(true);
    expect(isCleanSlateConfirmed('clean_slate')).toBe(true);
    expect(isCleanSlateConfirmed('REPLACE')).toBe(true);
    expect(isCleanSlateConfirmed(' replace ')).toBe(true);
    expect(isCleanSlateConfirmed('ЗАМЕНИТЬ')).toBe(true);
    expect(isCleanSlateConfirmed(' заменить ')).toBe(true);
    expect(isCleanSlateConfirmed('DELETE')).toBe(false);
    expect(isCleanSlateConfirmed('')).toBe(false);

    const htmlConfirm = renderWithProviders(
      <GedcomCleanSlateConfirm
        fileName="gramps-web-export-20261026130902.ged"
        onConfirm={() => { }}
        onCancel={() => { }}
      />,
      'en'
    );
    expect(htmlConfirm).toContain('Confirm Clean Slate Replacement');
    expect(htmlConfirm).toContain('gramps-web-export-20261026130902.ged');
    expect(htmlConfirm).toContain('tree_snapshot');
    expect(htmlConfirm).toContain('Type CLEAN_SLATE or REPLACE to confirm');
  });

  it('renders GedcomPostImportReport for both Clean Slate and Merge & Enrich modes with merged people/family counts', () => {
    const htmlReport = renderWithProviders(
      <GedcomPostImportReport
        result={DEFAULT_GEDCOM_IMPORT_RESULT}
        onOpenTree={() => { }}
        onSelectHomePerson={() => { }}
        onUploadMissingMedia={() => { }}
        onReset={() => { }}
      />,
      'en'
    );

    expect(htmlReport).toContain('GEDCOM Import Completed');
    expect(htmlReport).toContain('Full Replace');
    expect(htmlReport).toContain('snap-pre-gedcom-20261027');
    expect(htmlReport).toContain(
      '129 referenced media files are missing image binaries'
    );
    expect(htmlReport).toContain('Upload Media Folder');
    expect(htmlReport).toContain('Select Home Person');
    expect(htmlReport).toContain('Open Family Tree');

    const mergeResult = {
      ...DEFAULT_GEDCOM_IMPORT_RESULT,
      mode: 'merge' as const,
      merged_individuals: 42,
      merged_families: 15,
      matched_by_uid: 18,
      matched_by_name_dates: 19,
      matched_by_kinship: 5,
      chronology_conflicts_prevented: 2,
      matched_keys_preview: [
        'I0016 · Antonina Ivanovna (1912) [UID]',
        'I0024 · Pyotr Fyodorovich (1868) [Name+Birth]',
      ],
    };

    const htmlMergeReportEn = renderWithProviders(
      <GedcomPostImportReport result={mergeResult} />,
      'en'
    );
    expect(htmlMergeReportEn).toContain('Merge &amp; Enrich');
    expect(htmlMergeReportEn).toContain('Merged / Enriched People');
    expect(htmlMergeReportEn).toContain('42');
    expect(htmlMergeReportEn).toContain('Merged Families');
    expect(htmlMergeReportEn).toContain('15');
    expect(htmlMergeReportEn).toContain('Merge Keys &amp; Consistency Checks');
    expect(htmlMergeReportEn).toContain('Matched by _UID');
    expect(htmlMergeReportEn).toContain('Matched by Name &amp; Dates');
    expect(htmlMergeReportEn).toContain('Matched by Kinship Graph');
    expect(htmlMergeReportEn).toContain('Chronological Conflicts Prevented');
    expect(htmlMergeReportEn).toContain('I0016 · Antonina Ivanovna (1912) [UID]');

    const htmlMergeReportRu = renderWithProviders(
      <GedcomPostImportReport result={mergeResult} />,
      'ru'
    );
    expect(htmlMergeReportRu).toContain('Объединение и дополнение');
    expect(htmlMergeReportRu).toContain('Объединено / дополнено персон');
    expect(htmlMergeReportRu).toContain('Объединено семей');
    expect(htmlMergeReportRu).toContain('Ключи сопоставления и проверка целостности');
    expect(htmlMergeReportRu).toContain('Сопоставлено по _UID');
    expect(htmlMergeReportRu).toContain('Сопоставлено по имени и датам');
    expect(htmlMergeReportRu).toContain('Сопоставлено по графу родства');
    expect(htmlMergeReportRu).toContain('Предотвращено хронологических конфликтов');
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

  it('renders GedcomMediaReconcileCard with missing media stubs alert, dropzone, and post-upload statistics card in EN and RU', () => {
    // 1. Missing stubs alert state in EN
    const htmlMissingEn = renderWithProviders(
      <GedcomMediaReconcileCard
        initialMissingStubs={{
          total: 129,
          samplePaths: ['/root/gramps/000085_mikhail.jpg', '/root/gramps/000086_elena.jpg'],
        }}
      />,
      'en'
    );
    expect(htmlMissingEn).toContain('Media Archive Reconciliation');
    expect(htmlMissingEn).toContain('129 media items in your tree are missing images');
    expect(htmlMissingEn).toContain('/root/gramps/000085_mikhail.jpg, /root/gramps/000086_elena.jpg');
    expect(htmlMissingEn).toContain('ZIP / GDZ bundle (≤ 500 MB)');

    // 2. Missing stubs alert state in RU
    const htmlMissingRu = renderWithProviders(
      <GedcomMediaReconcileCard
        initialMissingStubs={{
          total: 129,
          samplePaths: ['/root/gramps/000085_mikhail.jpg'],
        }}
      />,
      'ru'
    );
    expect(htmlMissingRu).toContain('Согласование медиаархива');
    expect(htmlMissingRu).toContain('Медиафайлы без прикреплённых изображений: 129');
    expect(htmlMissingRu).toContain('Исходные пути к файлам:');

    // 3. Post-upload completed statistics report card in EN
    const htmlReportEn = renderWithProviders(
      <GedcomMediaReconcileCard
        initialMissingStubs={{ total: 0, samplePaths: [] }}
        initialReport={DEFAULT_RECONCILE_REPORT}
      />,
      'en'
    );
    expect(htmlReportEn).toContain('129 of 129 missing media files reconciled successfully');
    expect(htmlReportEn).toContain('Total Archive Files');
    expect(htmlReportEn).toContain('Reconciled Stubs');
    expect(htmlReportEn).toContain('Remaining Missing Stubs');
    expect(htmlReportEn).toContain('View in Media Gallery');
    expect(htmlReportEn).toContain('Upload Another Archive');

    // 4. Post-upload completed statistics report card in RU
    const htmlReportRu = renderWithProviders(
      <GedcomMediaReconcileCard
        initialMissingStubs={{ total: 0, samplePaths: [] }}
        initialReport={DEFAULT_RECONCILE_REPORT}
      />,
      'ru'
    );
    expect(htmlReportRu).toContain('Успешно согласовано 129 из 129 отсутствующих медиафайлов');
    expect(htmlReportRu).toContain('Всего файлов в архиве');
    expect(htmlReportRu).toContain('Согласовано записей');
    expect(htmlReportRu).toContain('Осталось отсутствующих');
    expect(htmlReportRu).toContain('Перейти в медиагалерею');
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
        '../../src/components/settings/gedcom/GedcomMediaReconcileCard.tsx'
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

  it('localizes GEDCOM parser warnings for missing media stubs and unresolved ASSO targets in EN and RU', () => {
    const tEn = (key: string, params?: Record<string, string | number>) => {
      const enJson = JSON.parse(
        fs.readFileSync(path.resolve(__dirname, '../../src/i18n/en.json'), 'utf-8')
      );
      const keys = key.split('.');
      let cur: any = enJson;
      for (const k of keys) cur = cur?.[k];
      let str = typeof cur === 'string' ? cur : key;
      if (params) {
        for (const [pk, pv] of Object.entries(params)) {
          str = str.replace(new RegExp(`\\{${pk}\\}`, 'g'), String(pv));
        }
      }
      return str;
    };

    const tRu = (key: string, params?: Record<string, string | number>) => {
      const ruJson = JSON.parse(
        fs.readFileSync(path.resolve(__dirname, '../../src/i18n/ru.json'), 'utf-8')
      );
      const keys = key.split('.');
      let cur: any = ruJson;
      for (const k of keys) cur = cur?.[k];
      let str = typeof cur === 'string' ? cur : key;
      if (params) {
        for (const [pk, pv] of Object.entries(params)) {
          str = str.replace(new RegExp(`\\{${pk}\\}`, 'g'), String(pv));
        }
      }
      return str;
    };

    const warning1 =
      '129 external media paths have no bundled binary; media asset stubs will be created for post-import reconciliation.';
    const localizedEn1 = formatGedcomWarning(warning1, tEn);
    const localizedRu1 = formatGedcomWarning(warning1, tRu);
    expect(localizedEn1).toContain('129 external media paths have no bundled binary');
    expect(localizedRu1).toContain('129 внешних путей к медиафайлам не содержат прикреплённых файлов');

    const warningSample =
      '129 external media paths (/root/gramps/...) have no bundled binary; media asset stubs will be created for post-import reconciliation.';
    const localizedEnSample = formatGedcomWarning(warningSample, tEn);
    const localizedRuSample = formatGedcomWarning(warningSample, tRu);
    expect(localizedEnSample).toContain('/root/gramps/...');
    expect(localizedRuSample).toContain('129 внешних путей к медиафайлам (/root/gramps/...)');

    const warningAsso = 'Unresolved ASSO target @I0012@';
    const localizedEnAsso = formatGedcomWarning(warningAsso, tEn);
    const localizedRuAsso = formatGedcomWarning(warningAsso, tRu);
    expect(localizedEnAsso).toBe('Unresolved ASSO association target @I0012@');
    expect(localizedRuAsso).toBe('Не удалось разрешить связанную запись ASSO @I0012@');

    // Test component rendering with localized warning in Russian
    const reportWithWarning = {
      ...DEFAULT_GEDCOM_ANALYSIS_REPORT,
      warnings: [warning1],
    };
    const htmlModal = renderWithProviders(
      <GedcomAnalysisModal
        report={reportWithWarning}
        open={true}
        onProceed={() => { }}
        onCancel={() => { }}
      />,
      'ru'
    );
    expect(htmlModal).toContain('129 внешних путей к медиафайлам не содержат прикреплённых файлов');

    const resultWithWarning = {
      ...DEFAULT_GEDCOM_IMPORT_RESULT,
      warnings: [warning1],
    };
    const htmlReport = renderWithProviders(
      <GedcomPostImportReport result={resultWithWarning} />,
      'ru'
    );
    expect(htmlReport).toContain('129 внешних путей к медиафайлам не содержат прикреплённых файлов');
  });
});
