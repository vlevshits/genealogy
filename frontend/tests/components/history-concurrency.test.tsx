import React from 'react';
import { describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import fs from 'node:fs';
import path from 'node:path';
import { I18nProvider, translate } from '../../src/i18n';
import { ThemeProvider } from '../../src/lib/theme';
import {
  AccessDeniedCard,
  BottomNav,
  ConfirmProvider,
  DropdownMenu,
  RadioChoiceRow,
  Switch,
  ToastProvider,
  clampMenuHorizontalOffset,
} from '../../src/components/ui';
import {
  getActiveOverlayStack,
  isTopOverlay,
  registerOverlay,
  unregisterOverlay,
} from '../../src/components/ui/overlay-stack';
import { AuthProvider } from '../../src/lib/auth-context';
import { ApiError } from '../../src/api/client';
import { isOfflineFallbackError } from '../../src/lib/error-formatter';
import {
  ConflictResolutionModal,
  HistoryFilterBar,
  RecordHistoryDrawer,
  computeConflictFields,
  extractEntityGid,
  extractEntryDiffs,
  formatDiffValue,
  formatEntityLabel,
  formatEntityTypeLabel,
  formatFieldLabel,
  isRawUuid,
} from '../../src/components/history';
import { TreeSnapshotsCard } from '../../src/components/settings';
import {
  ProfileHeroHeader,
  buildProfileOverflowItems,
} from '../../src/components/profile/ProfileHeroHeader';
import { buildDefaultPersonProfile } from '../../src/pages/PersonProfilePage';
import {
  UndoTransactionResponseDto,
  RestoreChangeResponseDto,
} from '../../src/api/history-api';
import {
  HistoryPage,
  filterHistoryGroups,
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
      'src/components/history/history-diff-utils.ts',
      'src/components/history/history-diff-constants.ts',
      'src/components/history/history-label-formatters.ts',
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
    expect(htmlAdmin).toContain('Change History');
    expect(htmlAdmin).toContain('1 transaction');
    expect(htmlAdmin).not.toContain('(s)');
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
    // UX-012 & UX-147: Diff chips include break-words wrapping + column/mobile labels
    expect(htmlPage).toContain('break-words max-w-full overflow-hidden');
    expect(htmlPage).toContain('hidden sm:grid sm:grid-cols-3');
    // UX-013: Friendly transaction labels instead of raw database keys
    expect(htmlPage).toContain('Added Parent');
    expect(htmlPage).toContain('Edited Profile Details');
    expect(htmlPage).toContain('Deleted Person Record');
    expect(
      formatTxLabel('custom.archive_batch', (k) => k)
    ).toBe('Custom Archive Batch');

    // UX-141 & UX-142: ConflictResolutionModal starts in non-radio side-by-side comparison mode
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
    expect(htmlConflict).toContain('data-testid="conflict-resolution-modal"');
    expect(htmlConflict).not.toContain('role="radiogroup"');
    expect(htmlConflict).not.toContain('role="radio"');
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

  it('localizes field labels, transaction labels, entity types, and extracts non-UUID GIDs cleanly in EN and RU', () => {
    const tEn = (k: string, p?: Record<string, string | number>) => translate('en', k, p);
    const tRu = (k: string, p?: Record<string, string | number>) => translate('ru', k, p);

    expect(formatFieldLabel('maiden_name', tEn)).toBe('Maiden Surname');
    expect(formatFieldLabel('maiden_name', tRu)).toBe('Девичья фамилия');
    expect(formatFieldLabel('birthDate', tEn)).toBe('Birth Date');
    expect(formatFieldLabel('birthDate', tRu)).toBe('Дата рождения');
    expect(formatFieldLabel('place_full_name', tRu)).toBe('Место');
    expect(formatFieldLabel('face_tags', tRu)).toBe('Отметки лиц');
    expect(formatFieldLabel('custom_archive_field', tEn)).toBe('Custom Archive Field');

    expect(formatTxLabel('family.detach', tEn)).toBe('Detached Family Member');
    expect(formatTxLabel('family.move_child', tRu)).toBe('Перенос ребёнка в другую семью');
    expect(formatTxLabel('quality.place_hierarchy_batch_fix', tRu)).toBe('Пакетное исправление иерархии мест');
    expect(formatTxLabel('quality.events.batch_merge', tRu)).toBe('Пакетное слияние дублирующихся событий');
    expect(formatTxLabel('ai.story_commit', tEn)).toBe('Applied Family Story Extract');
    expect(formatTxLabel('mcp.create_individual', tRu)).toBe('Помощник: создание персоны');
    expect(formatTxLabel('history.undo_transaction', tEn)).toBe('Undid Change Transaction');
    expect(formatTxLabel('Merged Moisei Kalman (I500057) into Moisei Kalman (I500100)', tRu)).toBe(
      'Слияние «Moisei Kalman (I500057)» с «Moisei Kalman (I500100)»'
    );
    expect(formatTxLabel('Batch merge (3 groups)', tRu)).toBe('Пакетное слияние (3 группы)');
    expect(formatTxLabel('custom.archive_batch', (k) => k)).toBe('Custom Archive Batch');

    expect(formatEntityTypeLabel('individual', tRu)).toBe('Персона');
    expect(formatEntityTypeLabel('life_event', tEn)).toBe('Event');
    expect(formatEntityTypeLabel('media_asset', tRu)).toBe('Медиа');
    expect(formatEntityTypeLabel('tree', tRu)).toBe('Семейное древо');
    expect(formatEntityTypeLabel('transaction', tEn)).toBe('Transaction');

    expect(isRawUuid('550e8400-e29b-41d4-a716-446655440000')).toBe(true);
    expect(isRawUuid('I0042')).toBe(false);
    expect(
      extractEntityGid({
        entity_gid: null,
        entity_id: '550e8400-e29b-41d4-a716-446655440000',
        entity_label: 'Anna Ivanova (I0042)',
      })
    ).toBe('I0042');
    expect(
      extractEntityGid({
        entity_id: '550e8400-e29b-41d4-a716-446655440000',
        entity_label: 'Anna Ivanova',
      })
    ).toBeNull();
  });

  it('formats GenDate, FamilyChildDto, FaceTagDto, alt names, and >3-key objects without JSON.stringify or raw UUID leaks', () => {
    const tEn = (k: string, p?: Record<string, string | number>) => translate('en', k, p);
    const tRu = (k: string, p?: Record<string, string | number>) => translate('ru', k, p);

    expect(
      formatDiffValue({ mod: 'ABT', start: { y: 1900, m: null, d: null }, cal: 'GREGORIAN', text: 'ABT 1900' }, '—', { t: tEn })
    ).toBe('ABT 1900');
    expect(
      formatDiffValue({ text: '15 May 1923', precision: 'EXACT', calendar: 'GREGORIAN', year: 1923 }, '—', { t: tEn })
    ).toBe('15 May 1923');
    expect(
      formatDiffValue(
        { child_id: '550e8400-e29b-41d4-a716-446655440000', rel_to_partner1: 'biological', rel_to_partner2: 'biological' },
        '—',
        { t: tRu, field: 'children' }
      )
    ).toBe('#550e8400 (Кровное родство)');
    expect(
      formatDiffValue(
        { person_id: '550e8400-e29b-41d4-a716-446655440000', person_name: 'Pyotr Kalman', x: 0.25, y: 0.45 },
        '—',
        { t: tEn, field: 'face_tags' }
      )
    ).toBe('Pyotr Kalman');
    expect(
      formatDiffValue(
        { name_type: 'married', given_name: 'Elena', surname: 'Smirnova' },
        '—',
        { t: tRu, field: 'alt_names' }
      )
    ).toBe('Elena Smirnova (Фамилия в браке)');

    const largeObj = formatDiffValue(
      {
        given_name: 'Ivan',
        surname: 'Petrov',
        place: 'Tver',
        notes: 'Merchant',
        country: 'Russia',
      },
      '—',
      { t: tRu }
    );
    expect(largeObj).toContain('Имя: Ivan');
    expect(largeObj).toContain('Фамилия: Petrov');
    expect(largeObj).not.toContain('{');
    expect(largeObj).not.toContain('"');

    expect(formatDiffValue('550e8400-e29b-41d4-a716-446655440000', '—', { t: tEn })).toBe('#550e8400');
    expect(formatDiffValue(true, '—', { t: tRu, field: 'is_living' })).toBe('Да');
    expect(formatDiffValue('M', '—', { t: tRu, field: 'sex' })).toBe('Мужской');
    expect(formatDiffValue('marriage', '—', { t: tRu, field: 'event_type' })).toBe('Брак / Венчание');
    expect(formatDiffValue('city', '—', { t: tRu, field: 'place_type' })).toBe('Город / Населённый пункт');
  });

  it('unwraps DeletedPersonSnapshotPayload and event_merge_bundle and suppresses noise/sibling fields', () => {
    const deletedDiffs = extractEntryDiffs({
      id: 'chg-del-person',
      tx_id: 'tx-del-person',
      entity_type: 'individual',
      entity_id: '550e8400-e29b-41d4-a716-446655440000',
      op: 'delete',
      created_at: '2026-11-01T10:00:00Z',
      before_json: {
        person: {
          id: '550e8400-e29b-41d4-a716-446655440000',
          gid: 'I0042',
          primary_name: { given_name: 'Elena', surname: 'Kalman' },
          sex: 'F',
          birth_sort: 18900101,
        },
        events: [{ id: 'ev-1', event_type: 'birth' }],
      },
      after_json: null,
    });
    expect(deletedDiffs.some((d) => d.field === 'given_name' && d.before === 'Elena')).toBe(true);
    expect(deletedDiffs.some((d) => d.field === 'sex' && d.before === 'F')).toBe(true);
    expect(deletedDiffs.some((d) => d.field === 'birth_sort' || d.field === 'id' || d.field === 'gid')).toBe(false);

    const eventMergeDiffs = extractEntryDiffs({
      id: 'chg-ev-merge',
      tx_id: 'tx-ev-merge',
      entity_type: 'life_event',
      entity_id: '11111111-2222-3333-4444-555555555555',
      op: 'merge',
      created_at: '2026-11-02T11:00:00Z',
      before_json: {
        kind: 'event_merge_bundle',
        events_before: [
          {
            id: '11111111-2222-3333-4444-555555555555',
            event_type: 'census',
            place_id: '99999999-8888-7777-6666-555555555555',
            place_full_name: 'Tver',
            description: 'Old census note',
          },
          {
            id: '22222222-3333-4444-5555-666666666666',
            event_type: 'census',
            place_full_name: 'Tver, Russia',
          },
        ],
      },
      after_json: {
        id: '11111111-2222-3333-4444-555555555555',
        event_type: 'census',
        place_id: 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee',
        place_full_name: 'Tver, Russia',
        description: 'Merged census note',
      },
    });
    expect(eventMergeDiffs.some((d) => d.field === 'merged_events')).toBe(true);
    expect(eventMergeDiffs.some((d) => d.field === 'place_full_name')).toBe(true);
    expect(eventMergeDiffs.some((d) => d.field === 'place_id')).toBe(false);
    expect(eventMergeDiffs.some((d) => d.field === 'description')).toBe(true);
  });

  it('renders RecordHistoryDrawer, HistoryPage, and ConflictResolutionModal with localized labels and zero raw UUIDs or JSON strings', () => {
    const uuidA = '550e8400-e29b-41d4-a716-446655440000';
    const uuidB = '11111111-2222-3333-4444-555555555555';
    const uuidPlace = '99999999-8888-7777-6666-555555555555';

    const realisticEntry = {
      id: 'chg-real-1',
      tx_id: 'tx-real-1',
      tx_label: 'person.update_profile',
      actor_name: 'Valentin Levshits',
      entity_type: 'individual',
      entity_id: uuidA,
      entity_gid: 'I0008',
      entity_label: 'Maria Petrova (I0008)',
      op: 'update' as const,
      created_at: '2026-11-10T12:00:00Z',
      before_json: {
        id: uuidA,
        gid: 'I0008',
        maiden_name: 'Sokolova',
        birth_date: { mod: 'ABT', start: { y: 1898, m: null, d: null }, cal: 'GREGORIAN', text: 'ABT 1898' },
        place_id: uuidPlace,
        place_full_name: 'Tver',
        is_living: true,
        children: [{ child_id: uuidB, rel_to_partner1: 'biological' }],
      },
      after_json: {
        id: uuidA,
        gid: 'I0008',
        maiden_name: 'Petrova',
        birth_date: { mod: 'EXACT', start: { y: 1898, m: 5, d: 12 }, cal: 'GREGORIAN', text: '12 May 1898' },
        place_id: uuidPlace,
        place_full_name: 'Tver, Russia',
        is_living: false,
        children: [{ child_id: uuidB, rel_to_partner1: 'adopted' }],
      },
    };

    const drawerHtmlRu = renderWithProviders(
      <RecordHistoryDrawer
        open
        onOpenChange={vi.fn()}
        entityType="individual"
        entityId={uuidA}
        entityLabel="Maria Petrova (I0008)"
        initialEntries={[realisticEntry]}
      />,
      'ru'
    );
    expect(drawerHtmlRu).toContain('Девичья фамилия');
    expect(drawerHtmlRu).toContain('Дата рождения');
    expect(drawerHtmlRu).toContain('Место');
    expect(drawerHtmlRu).toContain('Статус жизни');
    expect(drawerHtmlRu).toContain('Дети');
    expect(drawerHtmlRu).toContain('Редактирование профиля');
    expect(drawerHtmlRu).toContain('около 1898 г.');
    expect(drawerHtmlRu).toContain('12 мая 1898 г.');
    expect(drawerHtmlRu).toContain('#11111111');
    expect(drawerHtmlRu).not.toContain(uuidA);
    expect(drawerHtmlRu).not.toContain(uuidB);
    expect(drawerHtmlRu).not.toContain(uuidPlace);
    expect(drawerHtmlRu).not.toContain('maiden_name');
    expect(drawerHtmlRu).not.toContain('birth_date');
    expect(drawerHtmlRu).not.toContain('place_full_name');
    expect(drawerHtmlRu).not.toContain('{"');

    const historyHtmlRu = renderWithProviders(
      <HistoryPage
        userRole="admin"
        onNavigate={vi.fn()}
        initialGroups={[
          {
            tx_id: 'tx-real-1',
            tx_label: 'person.update_profile',
            actor_name: 'Valentin Levshits',
            created_at: '2026-11-10T12:00:00Z',
            can_undo: true,
            entries: [realisticEntry],
          },
        ]}
      />,
      'ru'
    );
    expect(historyHtmlRu).toContain('Maria Petrova');
    expect(historyHtmlRu).toContain('>I0008<');
    expect(historyHtmlRu).not.toContain('Maria Petrova (I0008)');
    expect(historyHtmlRu).not.toContain('Maria Petrova (I0008) (Персона)');
    expect(historyHtmlRu).toContain('Девичья фамилия');
    expect(historyHtmlRu).toContain('Дата рождения');
    expect(historyHtmlRu).not.toContain(uuidA);
    expect(historyHtmlRu).not.toContain(uuidB);
    expect(historyHtmlRu).not.toContain('maiden_name');
    expect(historyHtmlRu).not.toContain('{"');

    const conflictHtmlRu = renderWithProviders(
      <ConflictResolutionModal
        open
        onOpenChange={vi.fn()}
        updatedByName="Admin"
        updatedAt="2026-11-10T14:30:00Z"
        remoteVersion={4}
        myChanges={{
          maiden_name: 'Volkova',
          birth_date: { mod: 'ABT', start: { y: 1900, m: null, d: null }, cal: 'GREGORIAN', text: 'ABT 1900' },
          is_living: true,
        }}
        theirData={{
          maiden_name: 'Orlova',
          birth_date: { mod: 'EXACT', start: { y: 1901, m: 1, d: 2 }, cal: 'GREGORIAN', text: '2 Jan 1901' },
          is_living: false,
        }}
        onKeepMine={vi.fn()}
        onUseTheirs={vi.fn()}
      />,
      'ru'
    );
    expect(conflictHtmlRu).toContain('Девичья фамилия');
    expect(conflictHtmlRu).toContain('Дата рождения');
    expect(conflictHtmlRu).toContain('Статус жизни');
    expect(conflictHtmlRu).toContain('около 1900 г.');
    expect(conflictHtmlRu).toContain('2 января 1901 г.');
    expect(conflictHtmlRu).not.toContain('ABT 1900');
    expect(conflictHtmlRu).toContain('Да');
    expect(conflictHtmlRu).toContain('Нет');
    expect(conflictHtmlRu).toContain('2026');
    expect(conflictHtmlRu).not.toContain('(4)');
    expect(conflictHtmlRu).not.toContain('>maiden_name<');
    expect(conflictHtmlRu).not.toContain('>birth_date<');
    expect(conflictHtmlRu).not.toContain('>is_living<');
    expect(conflictHtmlRu).not.toContain('{"');

    const conflictNoTimeEn = renderWithProviders(
      <ConflictResolutionModal
        open
        onOpenChange={vi.fn()}
        updatedByName="Admin"
        remoteVersion={3}
        myChanges={{ surname: 'A' }}
        theirData={{ surname: 'B' }}
        onKeepMine={vi.fn()}
        onUseTheirs={vi.fn()}
      />,
      'en'
    );
    expect(conflictNoTimeEn).toContain('(v3)');
  });

  it('guards history undo/restore against swallowing HTTP 4xx/5xx errors while supporting offline fallback', () => {
    expect(isOfflineFallbackError(new ApiError(401, { code: 'unauthorized', message: 'Unauthorized' }))).toBe(false);
    expect(isOfflineFallbackError(new ApiError(403, { code: 'forbidden', message: 'Forbidden' }))).toBe(false);
    expect(isOfflineFallbackError(new ApiError(404, { code: 'not_found', message: 'Not found' }))).toBe(false);
    expect(isOfflineFallbackError(new ApiError(409, { code: 'conflict', message: 'Conflict' }))).toBe(false);
    expect(isOfflineFallbackError(new ApiError(500, { code: 'internal_error', message: 'Server error' }))).toBe(false);
    expect(isOfflineFallbackError(new ApiError(0, { code: 'network_error', message: 'Offline' }))).toBe(true);
    expect(isOfflineFallbackError(new ApiError(503, { code: 'network_error', message: 'Network down' }))).toBe(true);

    for (const relPath of ['src/components/history/RecordHistoryDrawer.tsx', 'src/pages/HistoryPage.tsx']) {
      const fullPath = path.resolve(__dirname, '../../', relPath);
      const source = fs.readFileSync(fullPath, 'utf-8');
      expect(source).not.toMatch(/\.(?:undoTransaction|restoreChange)\([^)]*\)\s*\.catch\s*\(/);
      expect(source).toContain('isOfflineFallbackError');
    }
  });

  it('localizes composite tx_labels, undo/snapshot fields, entity labels, embedded suffixes, and record counts in EN and RU', () => {
    const tEn = (k: string, p?: Record<string, string | number>) => translate('en', k, p);
    const tRu = (k: string, p?: Record<string, string | number>) => translate('ru', k, p);

    expect(formatFieldLabel('undone_tx_id', tEn)).toBe('Undone Transaction');
    expect(formatFieldLabel('undone_tx_id', tRu)).toBe('Отменённая операция');
    expect(formatFieldLabel('undone_count', tEn)).toBe('Changes Reverted');
    expect(formatFieldLabel('undone_count', tRu)).toBe('Отменено изменений');
    expect(formatFieldLabel('restored_snapshot_id', tEn)).toBe('Restored Checkpoint');
    expect(formatFieldLabel('restored_snapshot_id', tRu)).toBe('Восстановленная точка отката');
    expect(formatFieldLabel('safety_snapshot_id', tEn)).toBe('Safety Checkpoint');
    expect(formatFieldLabel('safety_snapshot_id', tRu)).toBe('Страховочная точка отката');
    expect(formatFieldLabel('restored_entity_ids', tEn)).toBe('Restored Records');
    expect(formatFieldLabel('restored_entity_ids', tRu)).toBe('Восстановленные записи');
    expect(formatFieldLabel('record_counts', tEn)).toBe('Record Counts');
    expect(formatFieldLabel('record_counts', tRu)).toBe('Количество записей');

    expect(formatTxLabel('person.update_profile — Alexander Levshits', tEn)).toBe(
      'Edited Profile Details — Alexander Levshits'
    );
    expect(formatTxLabel('person.update_profile — Alexander Levshits', tRu)).toBe(
      'Редактирование профиля — Alexander Levshits'
    );
    expect(formatTxLabel('family.create - Ivan & Maria', tRu)).toBe(
      'Создание семейного союза — Ivan & Maria'
    );
    expect(formatTxLabel('history.undo_transaction — person.update_profile', tRu)).toBe(
      'Отмена операции из журнала — Редактирование профиля'
    );

    expect(formatEntityLabel('Note — Alexander Levshits', 'note', 'N0001', tRu)).toBe('Заметка — Alexander Levshits');
    expect(formatEntityLabel('birth — Maria Petrova (E0105)', 'event', 'E0105', tRu)).toBe('Рождение — Maria Petrova');
    expect(formatEntityLabel('marr — Ivan & Maria', 'event', null, tRu)).toBe('Брак / Венчание — Ivan & Maria');
    expect(formatEntityLabel('Undo transaction', 'transaction', null, tRu)).toBe('Отмена операции из журнала');
    expect(formatEntityLabel('Family Tree', 'tree', null, tRu)).toBe('Семейное древо');

    expect(formatDiffValue('Pyotr Kalman [primary avatar]', '—', { t: tRu, field: 'face_tags' })).toBe(
      'Pyotr Kalman [основное фото]'
    );
    expect(formatDiffValue('Pyotr Kalman [primary avatar]', '—', { t: tEn, field: 'face_tags' })).toBe(
      'Pyotr Kalman [primary photo]'
    );
    expect(formatDiffValue('Maria [adopted/foster]', '—', { t: tRu, field: 'children' })).toBe(
      'Maria (Усыновлён(а)/Приёмный ребёнок)'
    );
    expect(formatDiffValue('Note — Elena', '—', { t: tRu, field: 'restored_entity_ids' })).toBe('Заметка — Elena');
    expect(formatDiffValue('birth — Elena', '—', { t: tRu, field: 'merged_events' })).toBe('Рождение — Elena');
    expect(
      formatDiffValue('person.update_profile — Alexander Levshits', '—', { t: tRu, field: 'undone_tx_id' })
    ).toBe('Редактирование профиля — Alexander Levshits');

    expect(
      formatDiffValue({ individuals: 5, families: 2, events: 1, places: 4 }, '—', { t: tEn, field: 'record_counts' })
    ).toBe('5 individuals, 2 families, 1 event, 4 places');
    expect(
      formatDiffValue({ individuals: 5, families: 2, events: 1, places: 4 }, '—', { t: tRu, field: 'record_counts' })
    ).toBe('5 персон, 2 семьи, 1 событие, 4 места');
    expect(
      formatDiffValue('5 individuals, 2 families, 1 note', '—', { t: tRu, field: 'record_counts' })
    ).toBe('5 персон, 2 семьи, 1 заметка');
  });

  it('hides restore button in RecordHistoryDrawer for create or can_revert=false and omits duplicate GID pill on HistoryPage', () => {
    const createOnlyDrawer = renderWithProviders(
      <RecordHistoryDrawer
        open
        onOpenChange={vi.fn()}
        entityType="individual"
        entityId="p-create-1"
        initialEntries={[
          {
            id: 'chg-c1',
            tx_id: 'tx-c1',
            tx_label: 'person.create',
            entity_type: 'individual',
            entity_id: 'p-create-1',
            op: 'create',
            created_at: '2026-11-10T10:00:00Z',
            after_json: { given_name: 'Ivan' },
          },
          {
            id: 'chg-u-locked',
            tx_id: 'tx-u1',
            tx_label: 'person.update_profile',
            entity_type: 'individual',
            entity_id: 'p-create-1',
            op: 'update',
            can_revert: false,
            before_json: { given_name: 'Ivan' },
            after_json: { given_name: 'Pyotr' },
            created_at: '2026-11-10T11:00:00Z',
          },
        ]}
      />,
      'en'
    );
    expect(createOnlyDrawer).not.toContain('Restore this version');

    const gidAsLabelPage = renderWithProviders(
      <HistoryPage
        userRole="admin"
        initialGroups={[
          {
            tx_id: 'tx-gid-only',
            tx_label: 'note.create — Alexander',
            actor_name: 'Admin',
            created_at: '2026-11-10T12:00:00Z',
            can_undo: false,
            entries: [
              {
                id: 'chg-gid-1',
                tx_id: 'tx-gid-only',
                tx_label: 'note.create — Alexander',
                entity_type: 'note',
                entity_id: '550e8400-e29b-41d4-a716-446655440000',
                entity_gid: 'N0007',
                entity_label: 'N0007',
                op: 'create',
                created_at: '2026-11-10T12:00:00Z',
                after_json: { body: 'Archival source note' },
              },
            ],
          },
        ]}
      />,
      'ru'
    );
    expect(gidAsLabelPage).toContain('N0007');
    expect(gidAsLabelPage).toContain('Заметка');
    expect(gidAsLabelPage).not.toContain('N0007 (Заметка)');
    expect(gidAsLabelPage).not.toContain('font-mono text-[11px] text-tertiary shrink-0');
    expect(gidAsLabelPage).toContain('Добавление исследовательской заметки — Alexander');
  });

  it('fixes E2E-discovered UI defects: conflict modal version/close, /persons/:id route, mutation error titles, and ProfileHeroHeader menu trigger & RBAC', () => {
    const tEn = (k: string, p?: Record<string, string | number>) => translate('en', k, p);
    const tRu = (k: string, p?: Record<string, string | number>) => translate('ru', k, p);

    // 1. PersonEditDialog passes remoteVersion and resets conflictState.open on KeepMine / UseTheirs / Merge
    const personEditSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/person/PersonEditDialog.tsx'),
      'utf-8'
    );
    expect(personEditSrc).toContain('remoteVersion={conflictState.remoteVersion}');
    const closeMatches = personEditSrc.match(
      /setConflictState\(\(prev\)\s*=>\s*\(\{\s*\.\.\.prev,\s*open:\s*false\s*\}\)\)/g
    );
    expect(closeMatches?.length ?? 0).toBeGreaterThanOrEqual(3);

    // 2. HistoryPage navigates to /persons/:id and uses dedicated mutation/load error titles
    const historyPageSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/pages/HistoryPage.tsx'),
      'utf-8'
    );
    expect(historyPageSrc).toContain('/persons/${entry.entity_id}');
    expect(historyPageSrc).not.toContain('/people/${entry.entity_id}');
    expect(historyPageSrc).not.toContain("title: t('common.errorLoading')");
    expect(historyPageSrc).toContain("title: t('history.undoErrorTitle')");
    expect(historyPageSrc).toContain("title: t('history.restore.errorTitle')");
    expect(historyPageSrc).toContain("title: t('history.page.loadErrorTitle')");

    expect(tEn('history.undoErrorTitle')).toBe('Failed to undo transaction');
    expect(tRu('history.undoErrorTitle')).toBe('Не удалось отменить операцию');
    expect(tEn('history.restore.errorTitle')).toBe('Failed to restore record');
    expect(tRu('history.restore.errorTitle')).toBe('Не удалось восстановить запись');
    expect(tEn('history.page.loadErrorTitle')).toBe('Failed to load change history');
    expect(tRu('history.page.loadErrorTitle')).toBe('Не удалось загрузить историю изменений');

    // 3. ProfileHeroHeader overflow menu gates viewHistory on can_edit and renders accessible <button> trigger
    const sampleProfile = buildDefaultPersonProfile('I0001', 'admin');
    const editorOverflow = buildProfileOverflowItems({
      capabilities: {
        ...sampleProfile.capabilities,
        can_edit: true,
        can_merge: false,
        can_delete: false,
      },
      accessTier: 'editor',
      isPrivate: false,
      t: tEn,
      onSetHomePerson: vi.fn(),
      onOpenEditModal: vi.fn(),
      onTogglePrivate: vi.fn(),
      onOpenHistory: vi.fn(),
      onMergePerson: vi.fn(),
      onCopyLink: vi.fn(),
      onDeletePerson: vi.fn(),
    });
    expect(editorOverflow.some((item) => item.id === 'history')).toBe(true);
    expect(editorOverflow.some((item) => item.id === 'merge')).toBe(false);

    const viewerOverflow = buildProfileOverflowItems({
      capabilities: {
        ...sampleProfile.capabilities,
        can_edit: false,
        can_merge: false,
        can_delete: false,
      },
      accessTier: 'viewer',
      isPrivate: false,
      t: tEn,
      onSetHomePerson: vi.fn(),
      onOpenEditModal: vi.fn(),
      onTogglePrivate: vi.fn(),
      onOpenHistory: vi.fn(),
      onMergePerson: vi.fn(),
      onCopyLink: vi.fn(),
      onDeletePerson: vi.fn(),
    });
    expect(viewerOverflow.some((item) => item.id === 'history')).toBe(false);
    expect(viewerOverflow.some((item) => item.id === 'merge')).toBe(false);

    const heroHtml = renderWithProviders(
      <ProfileHeroHeader
        person={sampleProfile.person}
        kinship={sampleProfile.kinship}
        capabilities={{
          ...sampleProfile.capabilities,
          can_edit: true,
          can_merge: false,
        }}
        accessTier="editor"
        onViewInTree={vi.fn()}
        onOpenEditModal={vi.fn()}
        onAddEvent={vi.fn()}
        onQuickAdd={vi.fn()}
        onSetHomePerson={vi.fn()}
        onTogglePrivate={vi.fn()}
        onDeletePerson={vi.fn()}
        onOpenHistory={vi.fn()}
        onMergePerson={vi.fn()}
        onCopyLink={vi.fn()}
      />,
      'en'
    );
    expect(heroHtml).toMatch(
      /<button[^>]*aria-label="More profile actions"[^>]*>/
    );
  });

  it('fixes DEF-HIST-001 through DEF-HIST-005 (empty 200 OK feeds, date filters, conflict state reset & error safety, undo/snapshot offline guards, and DTO alignment)', () => {
    const tEn = (k: string, p?: Record<string, string | number>) => translate('en', k, p);
    const tRu = (k: string, p?: Record<string, string | number>) => translate('ru', k, p);

    // DEF-HIST-001: Empty HistoryPage renders EmptyState and filterHistoryGroups applies fromDate/toDate bounds
    const emptyHistoryEn = renderWithProviders(
      <HistoryPage userRole="admin" initialGroups={[]} />,
      'en'
    );
    expect(emptyHistoryEn).toContain('No Matching History Entries');
    const emptyHistoryRu = renderWithProviders(
      <HistoryPage userRole="admin" initialGroups={[]} />,
      'ru'
    );
    expect(emptyHistoryRu).toContain('Записи в журнале не найдены');

    const dateGroups = [
      {
        tx_id: 'tx-oct',
        actor_name: 'Admin',
        created_at: '2026-10-15T10:00:00Z',
        can_undo: false,
        entries: [
          {
            id: 'e-oct',
            tx_id: 'tx-oct',
            entity_type: 'person',
            entity_id: 'p-1',
            op: 'update',
            created_at: '2026-10-15T10:00:00Z',
          },
        ],
      },
      {
        tx_id: 'tx-nov',
        actor_name: 'Admin',
        created_at: '2026-11-10T10:00:00Z',
        can_undo: false,
        entries: [
          {
            id: 'e-nov',
            tx_id: 'tx-nov',
            entity_type: 'person',
            entity_id: 'p-1',
            op: 'update',
            created_at: '2026-11-10T10:00:00Z',
          },
        ],
      },
    ];
    expect(
      filterHistoryGroups(dateGroups, {
        entityType: 'all',
        op: 'all',
        onlyDeletions: false,
        fromDate: '2026-11-01',
        toDate: '2026-11-30',
      }).map((g) => g.tx_id)
    ).toEqual(['tx-nov']);

    const historySrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/pages/HistoryPage.tsx'),
      'utf-8'
    );
    expect(historySrc).toContain('setGroups(txGroups);');
    expect(historySrc).not.toContain('if (txGroups.length > 0)');

    // DEF-HIST-002: Empty RecordHistoryDrawer renders EmptyState and never replaces [] with demo entries
    const emptyDrawerHtml = renderWithProviders(
      <RecordHistoryDrawer
        open
        onOpenChange={vi.fn()}
        entityType="person"
        entityId="p-empty"
        initialEntries={[]}
      />,
      'en'
    );
    expect(emptyDrawerHtml).toContain('No Changes Recorded');

    const drawerSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/history/RecordHistoryDrawer.tsx'),
      'utf-8'
    );
    expect(drawerSrc).toContain('setEntries(res.items ?? []);');
    expect(drawerSrc).not.toContain('if (res.items && res.items.length > 0)');

    // DEF-HIST-003: ConflictResolutionModal resets merge/selections/error and wraps async actions cleanly
    const conflictSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/history/ConflictResolutionModal.tsx'),
      'utf-8'
    );
    expect(conflictSrc).toContain('setMergeMode(false);');
    expect(conflictSrc).toContain('setSelectedSourceByField({});');
    expect(conflictSrc).toContain('data-testid="conflict-error-alert"');
    expect(conflictSrc).toContain('setSubmitError(getLocalizedErrorMessage(err, t))');

    const personEditSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/person/PersonEditDialog.tsx'),
      'utf-8'
    );
    expect(personEditSrc).toContain('openConflictFromError');

    // DEF-HIST-004: useUndoToast, DashboardPage, and TreeSnapshotsCard guard offline fallbacks cleanly
    const useUndoSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/hooks/useUndoToast.ts'),
      'utf-8'
    );
    expect(useUndoSrc).not.toContain("t('common.errorLoading')");
    expect(useUndoSrc).toContain("t('history.undoErrorTitle')");

    const dashboardSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/pages/DashboardPage.tsx'),
      'utf-8'
    );
    expect(dashboardSrc).not.toContain('.catch(() => ({');
    expect(dashboardSrc).toContain('isOfflineFallbackError');
    expect(dashboardSrc).toContain("t('history.undoErrorTitle')");

    const snapshotsSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/settings/TreeSnapshotsCard.tsx'),
      'utf-8'
    );
    expect(snapshotsSrc).not.toContain('.catch(() => {})');
    expect(snapshotsSrc).not.toContain("t('common.errorLoading')");
    expect(snapshotsSrc).toContain('isOfflineFallbackError');
    expect(snapshotsSrc).toContain('setSnapshots(items ?? [])');
    expect(tEn('history.snapshots.createErrorTitle')).toBe('Failed to create checkpoint');
    expect(tRu('history.snapshots.createErrorTitle')).toBe('Не удалось создать точку отката');

    // DEF-HIST-005: Backend response field compatibility
    const undoDto: UndoTransactionResponseDto = {
      tx_id: 'tx-1',
      undone_count: 3,
      status: 'undone',
    };
    const restoreDto: RestoreChangeResponseDto = {
      change_id: 'chg-1',
      entity_type: 'individual',
      entity_id: 'p-1',
    };
    expect(undoDto.undone_count).toBe(3);
    expect(restoreDto.entity_id).toBe('p-1');
  });

  it('verifies UX-138 through UX-150 overlay stack, accessibility, mobile responsiveness, and GenDate localization fixes', () => {
    const tEn = (k: string, p?: Record<string, string | number>) => translate('en', k, p);
    const tRu = (k: string, p?: Record<string, string | number>) => translate('ru', k, p);

    // UX-138 & UX-139: Shared overlay stack ordering and Drawer modal focus/inert integration
    registerOverlay('drawer-1');
    registerOverlay('dialog-2');
    expect(isTopOverlay('drawer-1')).toBe(false);
    expect(isTopOverlay('dialog-2')).toBe(true);
    expect(getActiveOverlayStack()).toEqual(['drawer-1', 'dialog-2']);
    unregisterOverlay('dialog-2');
    expect(isTopOverlay('drawer-1')).toBe(true);
    unregisterOverlay('drawer-1');
    expect(getActiveOverlayStack()).toEqual([]);

    const drawerSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/ui/drawer.tsx'),
      'utf-8'
    );
    expect(drawerSrc).toContain('useModalOverlay');
    expect(drawerSrc).toContain('tabIndex={-1}');
    expect(drawerSrc).toContain('h-10 w-10 min-h-10 min-w-10');

    const overlaySrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/ui/overlay-stack.ts'),
      'utf-8'
    );
    expect(overlaySrc).toContain('data-toast-region');
    expect(overlaySrc).toContain('e.stopPropagation()');

    // UX-140: DropdownMenu horizontal viewport clamping and ARIA trigger semantics
    expect(clampMenuHorizontalOffset({ left: 250, right: 458, width: 208 }, 390)).toBe(-76);
    expect(clampMenuHorizontalOffset({ left: -40, right: 168, width: 208 }, 390)).toBe(48);
    expect(clampMenuHorizontalOffset({ left: 40, right: 248, width: 208 }, 390)).toBe(0);

    const dropdownHtml = renderWithProviders(
      <DropdownMenu
        ariaLabel="More actions"
        trigger={<button type="button">Menu</button>}
        items={[{ id: 'edit', label: 'Edit', onSelect: vi.fn() }]}
      />,
      'en'
    );
    expect(dropdownHtml).toContain('aria-haspopup="menu"');
    expect(dropdownHtml).toContain('aria-expanded="false"');

    // UX-141 & UX-142: RadioChoiceRow provides radiogroup + native radio + roving tabIndex
    const mergeRowHtml = renderWithProviders(
      <RadioChoiceRow
        data-testid="conflict-field-surname"
        rowLabel="Surname"
        ariaLabel="Surname"
        value="mine"
        onChange={vi.fn()}
        options={[
          { value: 'mine', label: 'Levshits-Orlov' },
          { value: 'theirs', label: 'Levshits-Smirnov' },
        ]}
      />,
      'en'
    );
    expect(mergeRowHtml).toContain('role="radiogroup"');
    expect(mergeRowHtml).toContain('type="radio"');
    expect(mergeRowHtml).toContain('min-h-11');
    expect(mergeRowHtml).toContain('tabindex="0"');
    expect(mergeRowHtml).toContain('tabindex="-1"');

    const conflictSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/history/ConflictResolutionModal.tsx'),
      'utf-8'
    );
    expect(conflictSrc).toContain("variant={mergeMode ? 'secondary' : 'primary'}");

    // UX-143: RecordHistoryDrawer renders mutually exclusive loadError + Retry states
    const recordDrawerSrc = fs.readFileSync(
      path.resolve(__dirname, '../../src/components/history/RecordHistoryDrawer.tsx'),
      'utf-8'
    );
    expect(recordDrawerSrc).toContain('data-testid="history-drawer-error"');
    expect(recordDrawerSrc).toContain("t('common.retry')");
    expect(recordDrawerSrc).toContain(': loadError ?');

    // UX-144: AccessDeniedCard wrapping & BottomNav admin route gating
    const deniedRu = renderWithProviders(
      <AccessDeniedCard onOpenProfile={vi.fn()} onBackToTree={vi.fn()} />,
      'ru'
    );
    expect(deniedRu).toContain('flex-wrap items-start justify-between');
    expect(deniedRu).toContain('whitespace-nowrap');
    expect(deniedRu).toContain('min-h-11');

    const editorBottomNav = renderWithProviders(
      <AuthProvider
        initialUser={{
          id: 'u-ed',
          email: 'ed@example.com',
          first_name: 'Editor',
          last_name: 'User',
          display_name: 'Editor User',
          role: 'editor',
          is_active: true,
        }}
      >
        <BottomNav currentPath="/tree" />
      </AuthProvider>,
      'en'
    );
    expect(editorBottomNav).not.toContain('href="/history"');

    // UX-145: GenDate & date-field string localization in EN and RU
    expect(
      formatDiffValue('abt 1914', '—', { t: tRu, locale: 'ru', field: 'birth_date' })
    ).toBe('около 1914 г.');
    expect(
      formatDiffValue('19 Mar 1912', '—', { t: tRu, locale: 'ru', field: 'death_date' })
    ).toBe('19 марта 1912 г.');
    expect(
      formatDiffValue(
        { mod: 'ABT', start: { y: 1912, m: null, d: null }, cal: 'GREGORIAN', text: 'ABT 1912' },
        '—',
        { t: tEn, locale: 'en', field: 'birth_date' }
      )
    ).toBe('about 1912');

    // UX-146: color-scheme CSS & Switch touch-target-expand
    const indexCss = fs.readFileSync(
      path.resolve(__dirname, '../../src/index.css'),
      'utf-8'
    );
    expect(indexCss).toContain('color-scheme: light;');
    expect(indexCss).toContain('color-scheme: dark;');
    const switchHtml = renderWithProviders(
      <Switch checked={false} onCheckedChange={vi.fn()} label="Filter" />,
      'en'
    );
    expect(switchHtml).toContain('touch-target-expand');

    // UX-147 & UX-148: undone_tx_id compact ref & TreeSnapshotsCard label deduplication
    expect(
      formatDiffValue('9f8e7d6c-5b4a-4321-8765-abcdef012345', '—', {
        t: tEn,
        field: 'undone_tx_id',
      })
    ).toBe('#9f8e7d6c');
    expect(
      formatDiffValue('tx-9f8e7d6c5b4a', '—', { t: tEn, field: 'undone_tx_id' })
    ).toBe('#tx-9f8e7');

    const multiEntryHtml = renderWithProviders(
      <HistoryPage
        userRole="admin"
        onNavigate={vi.fn()}
        initialGroups={[
          {
            tx_id: 'tx-multi-1',
            tx_label: 'person.merge',
            actor_name: 'Admin',
            created_at: '2026-11-10T12:00:00Z',
            can_undo: false,
            entries: [
              {
                id: 'e-m1',
                tx_id: 'tx-multi-1',
                entity_type: 'person',
                entity_id: 'p-1',
                entity_label: 'Maria Petrova (I0008)',
                entity_gid: 'I0008',
                op: 'update',
                created_at: '2026-11-10T12:00:00Z',
              },
              {
                id: 'e-m2',
                tx_id: 'tx-multi-1',
                entity_type: 'person',
                entity_id: 'p-2',
                entity_label: 'Ivan Ivanov (I0009)',
                entity_gid: 'I0009',
                op: 'delete',
                created_at: '2026-11-10T12:00:00Z',
              },
            ],
          },
        ]}
      />,
      'ru'
    );
    expect(multiEntryHtml).toContain('Maria Petrova (Персона)');
    expect(multiEntryHtml).toContain('Ivan Ivanov (Персона)');

    const snapCardHtml = renderWithProviders(
      <TreeSnapshotsCard
        initialSnapshots={[
          {
            id: 'snap-99',
            label: 'Manual Backup Before Merge',
            reason: 'manual',
            created_at: '2026-11-10T10:00:00Z',
            record_counts: { individuals: 10, families: 4, events: 8, places: 3 },
          },
        ]}
      />,
      'en'
    );
    expect(snapCardHtml).toContain('sm:w-72 md:w-80');
    expect(snapCardHtml).not.toContain('(snap-99)');

    // UX-149 & UX-150: ICU plural alignment & collapsible mobile HistoryFilterBar chips
    expect(tEn('nav.history')).toBe(tEn('history.page.title'));
    expect(tRu('nav.history')).toBe(tRu('history.page.title'));
    expect(tEn('history.drawer.fieldsChangedCount', { count: 1 })).toBe('1 field modified');
    expect(tEn('history.drawer.fieldsChangedCount', { count: 3 })).toBe('3 fields modified');

    const activeFilterHtml = renderWithProviders(
      <HistoryFilterBar
        filters={{
          entityType: 'person',
          op: 'update',
          fromDate: '2026-10-01',
          toDate: '',
          onlyDeletions: false,
        }}
        onChange={vi.fn()}
        onReset={vi.fn()}
      />,
      'en'
    );
    expect(activeFilterHtml).toContain('data-testid="history-mobile-filters-toggle"');
    expect(activeFilterHtml).toContain('2026-10-01');
  });
});
