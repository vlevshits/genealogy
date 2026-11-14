import type { Page, Route } from '@playwright/test';

export const HISTORY_SUBJECT_ID = '550e8400-e29b-41d4-a716-446655440001';
export const HISTORY_PARTNER_ID = '550e8400-e29b-41d4-a716-446655440002';
export const HISTORY_CHILD_ID = '550e8400-e29b-41d4-a716-446655440003';
export const HISTORY_DUPE_PERSON_ID = '550e8400-e29b-41d4-a716-446655440099';
export const HISTORY_DELETED_PERSON_ID = '550e8400-e29b-41d4-a716-446655440077';
export const HISTORY_FAMILY_ID = '550e8400-e29b-41d4-a716-446655440100';
export const HISTORY_EVENT_ID = '550e8400-e29b-41d4-a716-446655440105';
export const HISTORY_SOURCE_EVENT_ID = '550e8400-e29b-41d4-a716-446655440104';
export const HISTORY_MEDIA_ID = '550e8400-e29b-41d4-a716-446655440201';
export const HISTORY_NOTE_ID = '550e8400-e29b-41d4-a716-446655440301';

export const HISTORY_TX_IDS = {
  profileUpdate: 'tx-profile-201',
  familyUpdate: 'tx-family-202',
  personMerge: 'tx-person-merge-203',
  eventMerge: 'tx-event-merge-204',
  undoAndRollback: 'tx-undo-rollback-205',
  mediaTags: 'tx-media-206',
  deletePerson: 'tx-delete-person-207',
} as const;

export const CONFLICT_UPDATED_AT = '2026-11-12T15:45:00Z';

export interface HistoryApiErrorStub {
  status: number;
  code: string;
  message: string;
}

export interface HistoryApiMockOptions {
  /** Inject HTTP failure on POST /history/tx/:id/undo. */
  undoError?: HistoryApiErrorStub;
  /** Inject HTTP failure on POST /history/:id/restore. */
  restoreError?: HistoryApiErrorStub;
  /** Trigger 409 Conflict when PATCH /persons/:id sends stale version 1. */
  conflictOnStalePatch?: boolean;
  /** Remote version number returned in 409 Conflict details (defaults to 2). */
  remoteVersion?: number;
  /** Return empty feed `[]` on GET /history. */
  emptyFeed?: boolean;
  /** Return empty items `[]` on GET /history/:entity_type/:entity_id. */
  emptyEntityHistory?: boolean;
  /** Inject HTTP failure on retry PATCH /persons/:id after 409 conflict. */
  patchRetryError?: HistoryApiErrorStub;
  /** Mock capability flags returned by GET /persons/:id/profile (default true). */
  canEdit?: boolean;
  canMerge?: boolean;
}

export interface RecordedHistoryRequest {
  method: string;
  path: string;
  search: string;
  body: Record<string, unknown> | null;
}

export interface HistoryApiMock {
  requests: RecordedHistoryRequest[];
  find: (method: string, pathPart: string) => RecordedHistoryRequest[];
}

const json = (route: Route, status: number, body: unknown) =>
  route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  });

function createInitialTransactionGroups() {
  return [
    {
      tx_id: HISTORY_TX_IDS.profileUpdate,
      tx_label: 'person.update_profile — Alexander Levshits',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Vera Levshits',
      created_at: '2026-11-12T10:30:00Z',
      can_undo: true,
      entries: [
        {
          id: 101,
          tx_id: HISTORY_TX_IDS.profileUpdate,
          tx_label: 'person.update_profile — Alexander Levshits',
          entity_type: 'person',
          entity_id: HISTORY_SUBJECT_ID,
          entity_gid: 'I0001',
          entity_label: 'Alexander Levshits (I0001)',
          op: 'update',
          action: 'updated',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Vera Levshits',
          can_revert: true,
          created_at: '2026-11-12T10:30:00Z',
          diffs: [
            {
              field: 'birth_date',
              before: {
                mod: 'ABT',
                start: { y: 1912, m: null, d: null },
                cal: 'GREGORIAN',
                text: 'ABT 1912',
              },
              after: {
                mod: 'EXACT',
                start: { y: 1912, m: 3, d: 19 },
                cal: 'GREGORIAN',
                text: '19 Mar 1912',
              },
            },
            {
              field: 'sex',
              before: 'U',
              after: 'M',
            },
          ],
        },
      ],
    },
    {
      tx_id: HISTORY_TX_IDS.familyUpdate,
      tx_label: 'family.update — Alexander Levshits & Vera Levshits',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Vera Levshits',
      created_at: '2026-11-12T09:45:00Z',
      can_undo: true,
      entries: [
        {
          id: 102,
          tx_id: HISTORY_TX_IDS.familyUpdate,
          tx_label: 'family.update — Alexander Levshits & Vera Levshits',
          entity_type: 'family',
          entity_id: HISTORY_FAMILY_ID,
          entity_gid: 'F0100',
          entity_label: 'Alexander Levshits & Vera Levshits (F0100)',
          op: 'update',
          action: 'updated',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Vera Levshits',
          can_revert: true,
          created_at: '2026-11-12T09:45:00Z',
          diffs: [
            {
              field: 'partner1_id',
              before: 'Pyotr Levshits (I0000)',
              after: 'Alexander Levshits (I0001)',
            },
            {
              field: 'children',
              before: [],
              after: [
                'Elena Levshits (I0042) [adopted]',
                {
                  child_id: HISTORY_CHILD_ID,
                  child_name: 'Viktor Levshits (I0043)',
                  rel_to_partner1: 'adopted',
                  rel_to_partner2: 'birth',
                },
              ],
            },
          ],
        },
      ],
    },
    {
      tx_id: HISTORY_TX_IDS.personMerge,
      tx_label: 'person.merge — Alexander Levshits',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Lev Levshits',
      created_at: '2026-11-11T16:15:00Z',
      can_undo: true,
      entries: [
        {
          id: 103,
          tx_id: HISTORY_TX_IDS.personMerge,
          tx_label: 'person.merge — Alexander Levshits',
          entity_type: 'person',
          entity_id: HISTORY_SUBJECT_ID,
          entity_gid: 'I0001',
          entity_label: 'Alexander Levshits (I0001)',
          op: 'update',
          action: 'merged',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Lev Levshits',
          can_revert: false,
          created_at: '2026-11-11T16:15:00Z',
          before_json: {
            kind: 'person_merge_bundle',
            merge_person: {
              id: HISTORY_DUPE_PERSON_ID,
              gid: 'I0099',
              primary_name: { given_name: 'Sasha', patronymic: null, surname: 'Levshits' },
            },
            keep_before: {
              id: HISTORY_SUBJECT_ID,
              gid: 'I0001',
              primary_name: { given_name: 'Alexander', patronymic: null, surname: 'Levshits' },
              biography: 'Early parish record',
            },
            families_before: [
              { id: HISTORY_FAMILY_ID, gid: 'F0100' },
            ],
          },
          after_json: {
            id: HISTORY_SUBJECT_ID,
            gid: 'I0001',
            primary_name: { given_name: 'Alexander', patronymic: null, surname: 'Levshits' },
            biography: 'Merged archive biography',
          },
          diffs: [
            {
              field: 'merged_profile',
              before: 'Sasha Levshits (I0099)',
              after: 'Alexander Levshits (I0001)',
            },
            {
              field: 'biography',
              before: 'Early parish record',
              after: 'Merged archive biography',
            },
          ],
        },
      ],
    },
    {
      tx_id: HISTORY_TX_IDS.eventMerge,
      tx_label: 'quality.events.merge — Maria Petrova',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Lev Levshits',
      created_at: '2026-11-11T14:00:00Z',
      can_undo: false,
      entries: [
        {
          id: 104,
          tx_id: HISTORY_TX_IDS.eventMerge,
          tx_label: 'quality.events.merge — Maria Petrova',
          entity_type: 'event',
          entity_id: HISTORY_EVENT_ID,
          entity_gid: 'E0105',
          entity_label: 'birth — Maria Petrova (E0105)',
          op: 'update',
          action: 'merged',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Lev Levshits',
          can_revert: false,
          created_at: '2026-11-11T14:00:00Z',
          before_json: {
            kind: 'event_merge_bundle',
            events_before: [
              {
                id: HISTORY_SOURCE_EVENT_ID,
                gid: 'E0104',
                event_type: 'birth',
                place_name: null,
              },
              {
                id: HISTORY_EVENT_ID,
                gid: 'E0105',
                event_type: 'birth',
                place_name: 'Kazan',
              },
            ],
            families_before: [{ id: HISTORY_FAMILY_ID, gid: 'F0100' }],
          },
          after_json: {
            id: HISTORY_EVENT_ID,
            gid: 'E0105',
            event_type: 'birth',
            place_name: 'Kazan',
          },
          diffs: [
            {
              field: 'merged_events',
              before: ['birth — Maria Petrova (E0104)'],
              after: 'birth — Maria Petrova (E0105)',
            },
          ],
        },
      ],
    },
    {
      tx_id: HISTORY_TX_IDS.undoAndRollback,
      tx_label: 'snapshot.rollback — Before GEDCOM import',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Vera Levshits',
      created_at: '2026-11-10T18:20:00Z',
      can_undo: false,
      entries: [
        {
          id: 105,
          tx_id: HISTORY_TX_IDS.undoAndRollback,
          tx_label: 'history.undo_transaction — person.update_profile — Alexander Levshits',
          entity_type: 'transaction',
          entity_id: HISTORY_TX_IDS.profileUpdate,
          entity_gid: null,
          entity_label: 'person.update_profile — Alexander Levshits',
          op: 'restore',
          action: 'undone',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Vera Levshits',
          can_revert: false,
          created_at: '2026-11-10T18:20:00Z',
          diffs: [
            {
              field: 'undone_tx_id',
              before: null,
              after: 'person.update_profile — Alexander Levshits',
            },
          ],
        },
        {
          id: 106,
          tx_id: HISTORY_TX_IDS.undoAndRollback,
          tx_label: 'snapshot.rollback — Before GEDCOM import',
          entity_type: 'tree',
          entity_id: '550e8400-e29b-41d4-a716-446655440900',
          entity_gid: null,
          entity_label: 'Before GEDCOM import',
          op: 'restore',
          action: 'restored',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Vera Levshits',
          can_revert: false,
          created_at: '2026-11-10T18:20:00Z',
          diffs: [
            {
              field: 'restored_snapshot_id',
              before: null,
              after: 'Before GEDCOM import',
            },
            {
              field: 'safety_snapshot_id',
              before: null,
              after: 'Safety checkpoint before rollback',
            },
            {
              field: 'record_counts',
              before: null,
              after: {
                individuals: 5,
                families: 2,
                events: 1,
                places: 4,
              },
            },
            {
              field: 'restored_entity_ids',
              before: null,
              after: ['Note — Alexander Levshits', 'birth — Maria Petrova'],
            },
          ],
        },
      ],
    },
    {
      tx_id: HISTORY_TX_IDS.mediaTags,
      tx_label: 'media.update — Family portrait 1924',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Vera Levshits',
      created_at: '2026-11-10T11:00:00Z',
      can_undo: false,
      entries: [
        {
          id: 107,
          tx_id: HISTORY_TX_IDS.mediaTags,
          tx_label: 'media.update — Family portrait 1924',
          entity_type: 'media',
          entity_id: HISTORY_MEDIA_ID,
          entity_gid: 'M0012',
          entity_label: 'Family portrait 1924 (M0012)',
          op: 'update',
          action: 'updated',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Vera Levshits',
          can_revert: true,
          created_at: '2026-11-10T11:00:00Z',
          diffs: [
            {
              field: 'face_tags',
              before: [],
              after: ['Pyotr Kalman (I0008) [primary avatar]'],
            },
          ],
        },
      ],
    },
    {
      tx_id: HISTORY_TX_IDS.deletePerson,
      tx_label: 'person.delete — Nikolai Orlov',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Lev Levshits',
      created_at: '2026-11-09T17:10:00Z',
      can_undo: true,
      entries: [
        {
          id: 108,
          tx_id: HISTORY_TX_IDS.deletePerson,
          tx_label: 'person.delete — Nikolai Orlov',
          entity_type: 'person',
          entity_id: HISTORY_DELETED_PERSON_ID,
          entity_gid: 'I0077',
          entity_label: 'Nikolai Orlov (I0077)',
          op: 'delete',
          action: 'deleted',
          actor_id: '550e8400-e29b-41d4-a716-446655440500',
          actor_name: 'Lev Levshits',
          can_revert: true,
          created_at: '2026-11-09T17:10:00Z',
          // Exercise client-side unwrapping of DeletedPersonSnapshotPayload without
          // leaking embedded UUIDs, families_before, or raw JSON keys.
          before_json: {
            person: {
              id: HISTORY_DELETED_PERSON_ID,
              gid: 'I0077',
              sex: 'M',
              is_living: false,
              primary_name: {
                given_name: 'Nikolai',
                patronymic: null,
                surname: 'Orlov',
              },
              birth: { date: '1890', place: null },
            },
            events: [],
            notes: [],
            families_before: [
              { id: HISTORY_FAMILY_ID, gid: 'F0100' },
            ],
          },
          after_json: null,
        },
      ],
    },
  ];
}

function createInitialEntityHistory() {
  return [
    {
      id: 901,
      tx_id: HISTORY_TX_IDS.profileUpdate,
      tx_label: 'person.update_profile — Alexander Levshits',
      entity_type: 'person',
      entity_id: HISTORY_SUBJECT_ID,
      entity_gid: 'I0001',
      entity_label: 'Alexander Levshits (I0001)',
      op: 'update',
      action: 'updated',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Vera Levshits',
      can_revert: true,
      created_at: '2026-11-12T10:30:00Z',
      diffs: [
        {
          field: 'birth_date',
          before: {
            mod: 'ABT',
            start: { y: 1912, m: null, d: null },
            cal: 'GREGORIAN',
            text: 'ABT 1912',
          },
          after: {
            mod: 'EXACT',
            start: { y: 1912, m: 3, d: 19 },
            cal: 'GREGORIAN',
            text: '19 Mar 1912',
          },
        },
        {
          field: 'sex',
          before: 'U',
          after: 'M',
        },
      ],
    },
    {
      id: 902,
      tx_id: 'tx-media-nonrevert-902',
      tx_label: 'media.update — Alexander Levshits',
      entity_type: 'person',
      entity_id: HISTORY_SUBJECT_ID,
      entity_gid: 'I0001',
      entity_label: 'Alexander Levshits (I0001)',
      op: 'update',
      action: 'updated',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Vera Levshits',
      can_revert: false,
      created_at: '2026-11-11T09:00:00Z',
      diffs: [
        {
          field: 'face_tags',
          before: [],
          after: ['Alexander Levshits (I0001) [primary avatar]'],
        },
      ],
    },
    {
      id: 903,
      tx_id: 'tx-create-903',
      tx_label: 'person.create — Alexander Levshits',
      entity_type: 'person',
      entity_id: HISTORY_SUBJECT_ID,
      entity_gid: 'I0001',
      entity_label: 'Alexander Levshits (I0001)',
      op: 'create',
      action: 'created',
      actor_id: '550e8400-e29b-41d4-a716-446655440500',
      actor_name: 'Lev Levshits',
      can_revert: false,
      created_at: '2026-11-08T12:00:00Z',
      diffs: [
        { field: 'given_name', before: null, after: 'Alexander' },
        { field: 'surname', before: null, after: 'Levshits' },
        { field: 'sex', before: null, after: 'U' },
      ],
    },
  ];
}

export async function installHistoryApiMock(
  page: Page,
  options: HistoryApiMockOptions = {}
): Promise<HistoryApiMock> {
  const requests: RecordedHistoryRequest[] = [];
  let groups = createInitialTransactionGroups();
  let entityHistory = createInitialEntityHistory();
  const remoteVersion = options.remoteVersion ?? 2;

  const personState = {
    id: HISTORY_SUBJECT_ID,
    gid: 'I0001',
    sex: 'M',
    is_living: true,
    living_override: null as string | null,
    is_private: false,
    primary_name: {
      given_name: 'Alexander',
      patronymic: 'Sergeyevich',
      surname: 'Levshits',
    },
    birth_surname: null as string | null,
    nickname: null as string | null,
    name_prefix: null as string | null,
    name_suffix: null as string | null,
    birth: { date: '19 Mar 1912', place: 'Kazan' },
    death: null,
    avatar_url: null,
    biography: 'Initial archival biography.',
    notes: 'Initial archival biography.',
    alt_names: [],
    version: 1,
    updated_at: '2026-11-10T10:00:00Z',
    updated_by_name: 'Vera Levshits',
  };

  await page.route(/\/api\/v1\//, async (route) => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname.replace(/^.*\/api\/v1/, '');
    const method = request.method();

    let body: Record<string, unknown> | null = null;
    try {
      body = request.postDataJSON() as Record<string, unknown> | null;
    } catch {
      body = null;
    }

    requests.push({ method, path, search: url.search, body });

    // GET /api/v1/history
    if (method === 'GET' && path === '/history') {
      if (options.emptyFeed) {
        return json(route, 200, {
          status: 'ok',
          operation: 'audit_history',
          message: 'History loaded',
          items: [],
          transactions: [],
          total: 0,
          page: 1,
          page_size: 25,
          next_cursor: null,
        });
      }

      const entityTypeFilter = url.searchParams.get('entity_type') || '';
      const opFilter = url.searchParams.get('op') || '';
      const onlyDeletions = url.searchParams.get('only_deletions') === 'true';
      const fromDate = url.searchParams.get('from_date') || '';
      const toDate = url.searchParams.get('to_date') || '';

      const filtered = groups.filter((group) => {
        const createdDate = String(group.created_at ?? '').slice(0, 10);
        if (fromDate && (!createdDate || createdDate < fromDate)) return false;
        if (toDate && (!createdDate || createdDate > toDate)) return false;
        return group.entries.some((entry) => {
          if (entityTypeFilter && entry.entity_type !== entityTypeFilter) return false;
          if (opFilter && entry.op !== opFilter) return false;
          if (onlyDeletions && !String(entry.op).startsWith('delet')) return false;
          return true;
        });
      });

      return json(route, 200, {
        status: 'ok',
        operation: 'audit_history',
        message: 'History loaded',
        items: filtered,
        transactions: filtered,
        total: filtered.length,
        page: 1,
        page_size: 25,
        next_cursor: null,
      });
    }

    // GET /api/v1/snapshots
    if (method === 'GET' && path === '/snapshots') {
      return json(route, 200, {
        items: [
          {
            id: 'snap-1',
            label: 'Before GEDCOM import',
            reason: 'pre_gedcom',
            created_at: '2026-11-10T18:00:00Z',
            expires_at: '2026-12-10T18:00:00Z',
            record_counts: {
              individuals: 5,
              families: 2,
              events: 1,
              places: 4,
            },
          },
        ],
      });
    }

    // POST /api/v1/history/tx/:tx_id/undo
    let match = path.match(/^\/history\/tx\/([^/]+)\/undo$/);
    if (method === 'POST' && match) {
      if (options.undoError) {
        return json(route, options.undoError.status, {
          error: {
            code: options.undoError.code,
            message: options.undoError.message,
          },
        });
      }
      const txId = decodeURIComponent(match[1]);
      groups = groups.map((g) => (g.tx_id === txId ? { ...g, can_undo: false } : g));
      return json(route, 200, {
        status: 'ok',
        tx_id: txId,
        undone_count: 1,
      });
    }

    // POST /api/v1/history/:change_id/restore
    match = path.match(/^\/history\/([^/]+)\/restore$/);
    if (method === 'POST' && match) {
      if (options.restoreError) {
        return json(route, options.restoreError.status, {
          error: {
            code: options.restoreError.code,
            message: options.restoreError.message,
          },
        });
      }
      const changeId = decodeURIComponent(match[1]);
      return json(route, 200, {
        status: 'ok',
        change_id: changeId,
        entity_type: 'person',
        entity_id: HISTORY_SUBJECT_ID,
      });
    }

    // GET /api/v1/history/:entity_type/:entity_id
    match = path.match(/^\/history\/([^/]+)\/([^/]+)$/);
    if (method === 'GET' && match) {
      const items = options.emptyEntityHistory ? [] : entityHistory;
      return json(route, 200, {
        entity_type: decodeURIComponent(match[1]),
        entity_id: decodeURIComponent(match[2]),
        entity_gid: 'I0001',
        entity_label: 'Alexander Levshits (I0001)',
        items,
        total: items.length,
      });
    }

    // GET /api/v1/persons/:id/profile
    match = path.match(/^\/persons\/([^/]+)\/profile$/);
    if (method === 'GET' && match) {
      return json(route, 200, {
        person: personState,
        kinship: null,
        events: [],
        families: {
          parent_families: [],
          spouse_families: [
            {
              family_id: HISTORY_FAMILY_ID,
              gid: 'F0100',
              relationship_type: 'married',
              spouse: {
                id: HISTORY_PARTNER_ID,
                gid: 'I0002',
                display_name: 'Vera Levshits',
                sex: 'F',
                is_living: true,
              },
              children: [],
              version: 1,
            },
          ],
        },
        media: [],
        notes: [],
        sources_summary: [],
        capabilities: {
          can_edit: options.canEdit ?? true,
          can_delete: options.canEdit ?? true,
          can_merge: options.canMerge ?? true,
          is_home_person: false,
        },
      });
    }

    // GET /api/v1/person/:id/version or /persons/:id/version
    match = path.match(/^\/(?:person|persons)\/([^/]+)\/version$/);
    if (method === 'GET' && match) {
      return json(route, 200, {
        entity_type: 'person',
        entity_id: decodeURIComponent(match[1]),
        version: personState.version,
        updated_at: personState.updated_at,
        updated_by_name: personState.updated_by_name,
      });
    }

    // PATCH /api/v1/persons/:id
    match = path.match(/^\/persons\/([^/]+)$/);
    if (method === 'PATCH' && match) {
      const sentVersion = typeof body?.version === 'number' ? body.version : 1;
      if (options.conflictOnStalePatch && sentVersion < remoteVersion) {
        return json(route, 409, {
          error: {
            code: 'version_conflict',
            message: `Individual ${personState.gid} was modified in another session`,
            details: {
              expected_version: sentVersion,
              current_version: remoteVersion,
              updated_by_name: 'Vera Levshits',
              updated_at: CONFLICT_UPDATED_AT,
              their_data: {
                id: HISTORY_SUBJECT_ID,
                gid: 'I0001',
                primary_name: {
                  given_name: 'Alexander',
                  patronymic: 'Sergeyevich',
                  surname: 'Levshits-Orlov',
                },
                sex: 'F',
                biography: 'Saved remotely by Vera from archive census.',
                version: remoteVersion,
                updated_at: CONFLICT_UPDATED_AT,
                updated_by_name: 'Vera Levshits',
              },
            },
          },
        });
      }

      if (options.patchRetryError && sentVersion >= remoteVersion) {
        return json(route, options.patchRetryError.status, {
          error: {
            code: options.patchRetryError.code,
            message: options.patchRetryError.message,
          },
        });
      }

      personState.version = Math.max(sentVersion + 1, remoteVersion + 1);
      if (typeof body?.given_name === 'string') {
        personState.primary_name.given_name = body.given_name;
      }
      if (typeof body?.surname === 'string') {
        personState.primary_name.surname = body.surname;
      }
      if (typeof body?.sex === 'string') {
        personState.sex = body.sex;
      }
      if (typeof body?.biography === 'string') {
        personState.biography = body.biography;
        personState.notes = body.biography;
      }
      return json(route, 200, personState);
    }

    return route.fallback();
  });

  return {
    requests,
    find: (method, pathPart) =>
      requests.filter((r) => r.method === method && r.path.includes(pathPart)),
  };
}
