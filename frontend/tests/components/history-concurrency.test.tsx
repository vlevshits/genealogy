import React from 'react';
import { describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'node:fs';
import path from 'node:path';
import { I18nProvider } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import { ConfirmProvider, ToastProvider } from '../../src/components/ui';
import {
  ConflictResolutionModal,
  HistoryFilterBar,
  RecordHistoryDrawer,
  computeConflictFields,
  extractEntryDiffs,
  formatDiffValue,
} from '../../src/components/history';
import { TreeSnapshotsCard } from '../../src/components/settings';
import {
  HistoryPage,
  formatTxLabel,
  getGroupEntries,
} from '../../src/pages/HistoryPage';

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

describe('Phase 1.11 — Change History, Undo Toast, Restore Deleted, Optimistic Concurrency & Tree Snapshots', () => {
  it('enforces zero raw <button>, <input>, <select>, or <textarea> HTML tags and zero hex colors across history & snapshot components', () => {
    const filesToCheck = [
      'src/components/history/RecordHistoryDrawer.tsx',
      'src/components/history/ConflictResolutionModal.tsx',
      'src/components/history/HistoryFilterBar.tsx',
      'src/components/settings/TreeSnapshotsCard.tsx',
      'src/pages/HistoryPage.tsx',
    ];

    for (const relPath of filesToCheck) {
      const fullPath = path.resolve(__dirname, '../../', relPath);
      const source = fs.readFileSync(fullPath, 'utf-8');
      expect(
        /<(button|input|select|textarea)\b/.test(source),
        `Found raw HTML form/button element in ${relPath}`
      ).toBe(false);
      expect(
        /#[0-9a-fA-F]{3,8}\b/.test(source),
        `Found hardcoded hex color in ${relPath}`
      ).toBe(false);
    }
  });

  it('extracts field-by-field before/after diffs and renders RecordHistoryDrawer in EN and RU', () => {
    const entry = {
      id: 'chg-1',
      tx_id: 'tx-100',
      actor_name: 'Elena Morozova',
      entity_type: 'person',
      entity_id: 'person-1',
      entity_label: 'Ivan Petrovich Morozov',
      op: 'update' as const,
      created_at: '2025-02-10T12:00:00Z',
      before: {
        given_name: 'Ivan',
        birth_surname: 'Morozov',
        biography: 'Old biography note',
      },
      after: {
        given_name: 'Ivan',
        birth_surname: 'Morozov-Smirnov',
        biography: 'Updated archival biography',
      },
    };

    const diffs = extractEntryDiffs(entry);
    expect(diffs).toHaveLength(2);
    expect(diffs[0].field).toBe('birth_surname');
    expect(diffs[0].before).toBe('Morozov');
    expect(diffs[0].after).toBe('Morozov-Smirnov');

    const htmlEn = renderWithProviders(
      <RecordHistoryDrawer
        open={true}
        onOpenChange={vi.fn()}
        entityType="person"
        entityId="person-1"
        entityLabel="Ivan Petrovich Morozov"
        initialEntries={[entry]}
      />,
      'en'
    );
    expect(htmlEn).toContain('Record Change History');
    expect(htmlEn).toContain('Elena Morozova');
    expect(htmlEn).toContain('Restore this version');

    const htmlRu = renderWithProviders(
      <RecordHistoryDrawer
        open={true}
        onOpenChange={vi.fn()}
        entityType="person"
        entityId="person-1"
        entityLabel="Иван Петрович Морозов"
        initialEntries={[entry]}
      />,
      'ru'
    );
    expect(htmlRu).toContain('История изменений записи');
    expect(htmlRu).toContain('Восстановить эту версию');
  });

  it('computes conflicting fields and renders ConflictResolutionModal for HTTP 409 in EN and RU', () => {
    const myChanges = {
      given_name: 'Aleksandr',
      surname: 'Volkov',
      biography: 'My local bio draft',
    };
    const theirChanges = {
      given_name: 'Aleksandr',
      surname: 'Volkov-Rostov',
      biography: 'Remote bio saved by Dmitry',
    };

    const conflicts = computeConflictFields(myChanges, theirChanges);
    expect(conflicts).toHaveLength(2);
    expect(conflicts.map((c) => c.field)).toEqual(['surname', 'biography']);

    const htmlEn = renderWithProviders(
      <ConflictResolutionModal
        open={true}
        onOpenChange={vi.fn()}
        updatedByName="Dmitry Volkov"
        updatedAt="2 minutes ago"
        myChanges={myChanges}
        theirChanges={theirChanges}
        onKeepMine={vi.fn()}
        onUseTheirs={vi.fn()}
        onMerge={vi.fn()}
      />,
      'en'
    );
    expect(htmlEn).toContain('Concurrent Edit Detected');
    expect(htmlEn).toContain('Volkov-Rostov');
    expect(htmlEn).toContain('Use their changes');
    expect(htmlEn).toContain('Merge field by field');
    expect(htmlEn).toContain('Keep my changes');

    const htmlRu = renderWithProviders(
      <ConflictResolutionModal
        open={true}
        onOpenChange={vi.fn()}
        updatedByName="Дмитрий Волков"
        myChanges={myChanges}
        theirChanges={theirChanges}
        onKeepMine={vi.fn()}
        onUseTheirs={vi.fn()}
        onMerge={vi.fn()}
      />,
      'ru'
    );
    expect(htmlRu).toContain('Обнаружено параллельное редактирование');
    expect(htmlRu).toContain('Принять чужие изменения');
    expect(htmlRu).toContain('Объединить по полям');
    expect(htmlRu).toContain('Оставить мои изменения');
  });

  it('renders TreeSnapshotsCard with checkpoints, record counts, and reason badges in EN and RU', () => {
    const snapshots = [
      {
        id: 'snap-1',
        label: 'Pre-GEDCOM Import Checkpoint',
        reason: 'pre_gedcom_import' as const,
        created_by_name: 'Admin',
        created_at: '2025-02-11T09:30:00Z',
        record_counts: {
          individuals: 142,
          families: 48,
          events: 310,
          places: 29,
        },
      },
    ];

    const htmlEn = renderWithProviders(
      <TreeSnapshotsCard initialSnapshots={snapshots} />,
      'en'
    );
    expect(htmlEn).toContain('Tree Checkpoints &amp; Rollback Snapshots');
    expect(htmlEn).toContain('Pre-GEDCOM Import Checkpoint');
    expect(htmlEn).toContain('Persons: 142');
    expect(htmlEn).toContain('Roll Back to Checkpoint');

    const htmlRu = renderWithProviders(
      <TreeSnapshotsCard initialSnapshots={snapshots} />,
      'ru'
    );
    expect(htmlRu).toContain('Точки отката и снимки состояния древа');
    expect(htmlRu).toContain('Персоны: 142');
    expect(htmlRu).toContain('Откатить к этой точке');
  });

  it('renders Admin HistoryPage with grouped transactions, undo action, and deleted record restore action, and blocks non-Admin roles', () => {
    const groups = [
      {
        tx_id: 'tx-501',
        actor_name: 'Anna Karenina',
        created_at: '2025-02-11T15:00:00Z',
        can_undo: true,
        entries: [
          {
            id: 'entry-del-1',
            tx_id: 'tx-501',
            actor_name: 'Anna Karenina',
            entity_type: 'person',
            entity_id: 'p-99',
            entity_label: 'Nikolai Levin',
            op: 'delete' as const,
            created_at: '2025-02-11T15:00:00Z',
            can_restore: true,
            before: { given_name: 'Nikolai', surname: 'Levin' },
            after: null,
          },
        ],
      },
    ];

    const htmlAdmin = renderWithProviders(
      <HistoryPage userRole="admin" initialGroups={groups} />,
      'en'
    );
    expect(htmlAdmin).toContain('Change History &amp; Audit Log');
    expect(htmlAdmin).toContain('Nikolai Levin');
    expect(htmlAdmin).toContain('Restore Record');
    expect(htmlAdmin).toContain('history-undo-tx-tx-501');

    const htmlEditor = renderWithProviders(
      <HistoryPage userRole="editor" initialGroups={groups} />,
      'en'
    );
    expect(htmlEditor).toContain('Administrator Access Required');
  });

  it('renders HistoryFilterBar with entity type, action op, date range, and only-deletions switch', () => {
    const html = renderWithProviders(
      <HistoryFilterBar
        filters={{
          entityType: 'all',
          op: 'all',
          from: '',
          to: '',
          onlyDeletions: false,
        }}
        onChange={vi.fn()}
        onReset={vi.fn()}
      />,
      'en'
    );
    expect(html).toContain('Record Type');
    expect(html).toContain('Only deleted records');
  });

  it('verifies UX-009 through UX-014 fixes (elevated mobile toast, radiogroup a11y, interpolated diff counts, overflow wrapping, and friendly tx_label)', () => {
    const htmlPage = renderWithProviders(
      <HistoryPage userRole="admin" />,
      'en'
    );
    // UX-009: Toast container elevated above mobile BottomNav
    expect(htmlPage).toContain(
      'bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))]'
    );
    // UX-011: No raw {count} placeholder in diff toggle buttons
    expect(htmlPage).not.toContain('{count}');
    const htmlDrawerDefault = renderWithProviders(
      <RecordHistoryDrawer
        open={true}
        onOpenChange={vi.fn()}
        entityType="person"
        entityId="p-1"
      />,
      'en'
    );
    expect(htmlDrawerDefault).not.toContain('{count}');
    expect(htmlDrawerDefault).toContain('Show field changes (2)');
    // UX-012: Diff chips include break-words max-w-full overflow-hidden text-ellipsis
    expect(htmlPage).toContain(
      'break-words max-w-full overflow-hidden text-ellipsis'
    );
    // UX-013: Friendly transaction labels instead of raw database keys
    expect(htmlPage).toContain('Added Parent');
    expect(htmlPage).toContain('Edited Profile Details');
    expect(htmlPage).toContain('Deleted Person Record');
    expect(
      formatTxLabel('custom.archive_batch', (k) => k)
    ).toBe('Custom Archive Batch');

    // UX-010: ConflictResolutionModal includes role="radiogroup" and focus-visible:ring-2
    const htmlConflict = renderWithProviders(
      <ConflictResolutionModal
        open={true}
        onOpenChange={vi.fn()}
        myChanges={{ surname: 'Morozov' }}
        theirChanges={{ surname: 'Morozov-Smirnov' }}
        onKeepMine={vi.fn()}
        onUseTheirs={vi.fn()}
        onMerge={vi.fn()}
      />,
      'en'
    );
    expect(htmlConflict).toContain('role="radiogroup"');
    expect(htmlConflict).toContain('focus-visible:ring-2');
    expect(htmlConflict).toContain('hidden sm:grid sm:grid-cols-3');

    // DEF-001: HistoryPage safely handles backend transaction groups with `changes` instead of `entries`
    const groupWithChanges = {
      tx_id: 'tx-live-1',
      tx_label: 'person.update_profile',
      actor_name: 'Admin',
      created_at: '2026-10-28T00:00:00Z',
      can_undo: true,
      changes: [
        {
          id: 'chg-live-1',
          tx_id: 'tx-live-1',
          entity_type: 'person',
          entity_id: 'p-1',
          entity_label: 'Maria Petrova',
          op: 'update' as const,
          created_at: '2026-10-28T00:00:00Z',
          diffs: [{ field: 'surname', before: 'Ivanova', after: 'Petrova' }],
        },
      ],
    };
    expect(getGroupEntries(groupWithChanges)).toHaveLength(1);
    const htmlLiveGroup = renderWithProviders(
      <HistoryPage userRole="admin" initialGroups={[groupWithChanges]} />,
      'en'
    );
    expect(htmlLiveGroup).toContain('Maria Petrova');
  });

  it('correctly handles person_merge_bundle in extractEntryDiffs and formatDiffValue without [object Object]', () => {
    const mergeEntry = {
      id: 'chg-merge-1',
      tx_id: 'tx-merge-101',
      op: 'merge',
      action: 'merge',
      entity_type: 'individual',
      entity_id: 'keep-1',
      entity_label: 'Moisei Kalman',
      summary: 'Merged Moisei Kalman (I500057) into Moisei Kalman (I500100)',
      before_json: {
        kind: 'person_merge_bundle',
        keep_before: {
          id: 'keep-1',
          gid: 'I500100',
          primary_name: { given_name: 'Moisei', surname: 'Kalman' },
          birth: { date: '1900', place: 'Tver' },
        },
        merge_person: {
          id: 'merge-2',
          gid: 'I500057',
          primary_name: { given_name: 'Moisei', surname: 'Kalman' },
          birth: { date: '1900-03-15', place: 'Tver, Russia' },
        },
        families_before: [{ id: 'fam-1' }, { id: 'fam-2' }],
        events_before: [{ id: 'ev-1' }, { id: 'ev-2' }, { id: 'ev-3' }],
        notes_before: [],
        media_before: [{ id: 'med-1' }],
        gid_aliases_before: { I500057: 'keep-1' },
      },
      after_json: {
        id: 'keep-1',
        gid: 'I500100',
        primary_name: { given_name: 'Moisei', surname: 'Kalman' },
        birth: { date: '1900-03-15', place: 'Tver, Russia' },
      },
      created_at: '2026-10-30T14:35:00Z',
    };

    const diffs = extractEntryDiffs(mergeEntry as any);
    // Should extract synthetic merged_profile and changed birth field
    expect(diffs.some((d) => d.field === 'merged_profile')).toBe(true);
    expect(diffs.some((d) => d.field === 'birth')).toBe(true);

    // None of the internal backup bundle arrays should appear as raw fields
    expect(diffs.some((d) => d.field === 'events_before')).toBe(false);
    expect(diffs.some((d) => d.field === 'families_before')).toBe(false);
    expect(diffs.some((d) => d.field === 'media_before')).toBe(false);
    expect(diffs.some((d) => d.field === 'notes_before')).toBe(false);
    expect(diffs.some((d) => d.field === 'keep_before')).toBe(false);
    expect(diffs.some((d) => d.field === 'merge_person')).toBe(false);

    // formatDiffValue verification
    expect(formatDiffValue(null)).toBe('—');
    expect(formatDiffValue({ given_name: 'Moisei', surname: 'Kalman' })).toBe('Moisei Kalman');
    expect(formatDiffValue({ date: '1900-03-15', place: 'Tver' })).toBe('1900-03-15, Tver');

    // Render in HistoryPage with expanded tx to ensure zero [object Object]
    const group = {
      tx_id: 'tx-merge-101',
      tx_label: 'Merged Moisei Kalman (I500057) into Moisei Kalman (I500100)',
      actor_name: 'Valentin Levshits',
      created_at: '2026-10-30T14:35:00Z',
      can_undo: true,
      changes: [mergeEntry as any],
    };

    const html = renderWithProviders(
      <HistoryPage userRole="admin" initialGroups={[group]} />,
      'ru'
    );
    expect(html).not.toContain('[object Object]');
  });
});
