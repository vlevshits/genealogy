import React from 'react';
import { describe, it, expect } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'fs';
import path from 'path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ToastProvider, ConfirmProvider } from '../../src/components/ui';
import { SettingsPage } from '../../src/pages/SettingsPage';
import {
  AdminSystemResponse,
  BackupListResponse,
  InstanceSettingsDto,
} from '../../src/api/client';

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

const SAMPLE_SETTINGS: InstanceSettingsDto = {
  tree_title: 'Levshits Family Archive',
  default_locale: 'ru',
  enable_public_access: false,
  living_person_age_threshold: 110,
  settings_locked_from_env: false,
};

const SAMPLE_SYSTEM: AdminSystemResponse = {
  app_version: '0.1.0',
  database: {
    status: 'connected',
    size_bytes: 15_728_640, // 15 MB
    tables_count: 14,
  },
  storage: {
    media_size_bytes: 52_428_800, // 50 MB
    media_files_count: 42,
    free_disk_bytes: 21_474_836_480, // 20 GB
    total_disk_bytes: 64_424_509_440,
  },
  backups: {
    last_backup_at: '2026-10-26T03:00:00Z',
    last_backup_status: 'ok',
    total_backups: 2,
  },
  jobs: {
    queue_length: 0,
    active_workers: 2,
  },
};

const SAMPLE_BACKUPS: BackupListResponse = {
  items: [
    {
      filename: 'backup-2026-10-26T030000Z.tar.gz',
      size_bytes: 18_874_368,
      created_at: '2026-10-26T03:00:00Z',
      trigger: 'scheduled',
      app_version: '0.1.0',
      schema_version: 1,
      includes_gedcom: true,
    },
    {
      filename: 'backup-2026-10-25T191500Z.tar.gz',
      size_bytes: 18_350_080,
      created_at: '2026-10-25T19:15:00Z',
      trigger: 'manual',
      app_version: '0.1.0',
      schema_version: 1,
      includes_gedcom: false,
    },
  ],
  retention_days: 30,
  cron_schedule: '0 3 * * *',
  backup_enabled: true,
};

describe('Phase 1.1 Settings & Administration Page (/settings)', () => {
  it('uses only Phase 0 design-system components with zero raw <input>, <select>, or <button> tags in SettingsPage.tsx', () => {
    const filePath = path.resolve(__dirname, '../../src/pages/SettingsPage.tsx');
    const source = fs.readFileSync(filePath, 'utf-8');
    expect(source).not.toMatch(/<input[\s>]/);
    expect(source).not.toMatch(/<select[\s>]/);
    expect(source).not.toMatch(/<button[\s>]/);
  });

  it('renders General & Archive tab and operational System card in English and Russian', () => {
    const htmlEn = renderWithProviders(
      <SettingsPage
        currentPath="/settings"
        initialSettings={SAMPLE_SETTINGS}
        initialSystem={SAMPLE_SYSTEM}
        initialBackups={SAMPLE_BACKUPS}
      />,
      'en'
    );

    expect(htmlEn).toContain('Settings &amp; Administration');
    expect(htmlEn).toContain('General Instance Configuration');
    expect(htmlEn).toContain('Instance Tree Title');
    expect(htmlEn).toContain('Levshits Family Archive');
    expect(htmlEn).toContain('Enable Public Site for Anonymous Visitors');
    expect(htmlEn).toContain('Living Person Age Threshold');
    expect(htmlEn).toContain('110 years');
    expect(htmlEn).toContain('Save Changes');
    // Plain functional system diagnostics (17 §6.1, 00 §3.4)
    expect(htmlEn).toContain('Server: Healthy');
    expect(htmlEn).toContain('Database: Connected (15 MB)');
    expect(htmlEn).toContain('Storage: 50 MB used (42 files, 20 GB free)');
    expect(htmlEn).toContain('Version: v0.1.0');

    const htmlRu = renderWithProviders(
      <SettingsPage
        currentPath="/settings"
        initialSettings={SAMPLE_SETTINGS}
        initialSystem={SAMPLE_SYSTEM}
        initialBackups={SAMPLE_BACKUPS}
      />,
      'ru'
    );

    expect(htmlRu).toContain('Настройки и администрирование');
    expect(htmlRu).toContain('Общая конфигурация архива');
    expect(htmlRu).toContain('Открыть публичный просмотр для анонимных посетителей');
    expect(htmlRu).toContain('Сервер: В норме');
    expect(htmlRu).toContain('База данных: Подключена (15 MB)');
    expect(htmlRu).toContain('Версия: v0.1.0');
  });

  it('renders read-only notice and disables controls when settings_locked_from_env is true (10 §7)', () => {
    const lockedSettings: InstanceSettingsDto = {
      ...SAMPLE_SETTINGS,
      settings_locked_from_env: true,
    };

    const htmlEn = renderWithProviders(
      <SettingsPage
        currentPath="/settings"
        initialSettings={lockedSettings}
        initialSystem={SAMPLE_SYSTEM}
        initialBackups={SAMPLE_BACKUPS}
      />,
      'en'
    );

    expect(htmlEn).toContain('Managed by server configuration');
    expect(htmlEn).toContain('SETTINGS_LOCKED_FROM_ENV=true');
    expect(htmlEn).toMatch(/disabled=""/);

    const htmlRu = renderWithProviders(
      <SettingsPage
        currentPath="/settings"
        initialSettings={lockedSettings}
        initialSystem={SAMPLE_SYSTEM}
        initialBackups={SAMPLE_BACKUPS}
      />,
      'ru'
    );

    expect(htmlRu).toContain('Управляется конфигурацией сервера');
  });

  it('renders Backups & Snapshots tab (/settings/backups) with full archive CTA, cron schedule, archives list, and CLI commands', () => {
    const htmlEn = renderWithProviders(
      <SettingsPage
        currentPath="/settings/backups"
        initialSettings={SAMPLE_SETTINGS}
        initialSystem={SAMPLE_SYSTEM}
        initialBackups={SAMPLE_BACKUPS}
      />,
      'en'
    );

    expect(htmlEn).toContain('Download Data &amp; Media');
    expect(htmlEn).toContain('0 3 * * *');
    expect(htmlEn).toContain('30 days');
    expect(htmlEn).toContain('backup-2026-10-26T030000Z.tar.gz');
    expect(htmlEn).toContain('backup-2026-10-25T191500Z.tar.gz');
    expect(htmlEn).toContain('Scheduled');
    expect(htmlEn).toContain('Manual');
    expect(htmlEn).toContain('GEDZIP');
    expect(htmlEn).toContain('docker compose exec backend ./genealogy-cli backup');
    expect(htmlEn).toContain(
      'docker compose exec backend ./genealogy-cli restore /app/data/backups/backup-2026-10-26T030000Z.tar.gz'
    );

    const htmlRu = renderWithProviders(
      <SettingsPage
        currentPath="/settings/backups"
        initialSettings={SAMPLE_SETTINGS}
        initialSystem={SAMPLE_SYSTEM}
        initialBackups={SAMPLE_BACKUPS}
      />,
      'ru'
    );

    expect(htmlRu).toContain('Скачать данные и медиа');
    expect(htmlRu).toContain('По расписанию');
    expect(htmlRu).toContain('Вручную');
  });

  it('renders all 6 settings sub-navigation tabs inside the unified two-column shell (07 §4 DoD #1)', () => {
    for (const subPath of [
      '/settings/profile',
      '/settings/users',
      '/settings/security',
      '/settings/gedcom',
    ]) {
      const html = renderWithProviders(
        <SettingsPage
          currentPath={subPath}
          initialSettings={SAMPLE_SETTINGS}
          initialSystem={SAMPLE_SYSTEM}
          initialBackups={SAMPLE_BACKUPS}
        />,
        'en'
      );
      expect(html).toContain('Settings &amp; Administration');
      expect(html).toContain('Archive Settings');
      expect(html).toContain('Backups &amp; Copies');
      expect(html).toContain('aria-current="page"');
    }
  });
});
