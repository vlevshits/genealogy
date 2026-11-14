import type { Page, Route } from '@playwright/test';

/**
 * Stateful mock of the Data Quality & Duplicate Events API surface (`/quality/*`,
 * `/history/tx/:txId/undo`). Mutations snapshot previous state so Undo
 * restores merged groups and counts deterministically, mirroring the backend.
 */

export interface MockRecordedRequest {
  method: string;
  path: string;
  body: Record<string, unknown> | null;
}

export interface MockDuplicateEventItem {
  id: string;
  gid: string;
  event_type: string;
  custom_type: string | null;
  individual_id: string | null;
  family_id: string | null;
  date_text: string | null;
  date_sort: number | null;
  place_id: string | null;
  place_full_name: string | null;
  description: string | null;
  source_text: string | null;
  media_count: number;
  is_private: boolean;
}

export interface MockDuplicateEventGroup {
  id: string;
  event_type: string;
  scope: 'individual' | 'family';
  person: {
    id: string;
    gid: string;
    display_name: string;
    sex: 'M' | 'F';
    birth_date?: string | null;
    death_date?: string | null;
  };
  partner?: {
    id: string;
    gid: string;
    display_name: string;
    sex: 'M' | 'F';
    birth_date?: string | null;
    death_date?: string | null;
  } | null;
  family_ids?: string[];
  events: MockDuplicateEventItem[];
  proposed: {
    keep_event_id: string;
    keep_event_gid: string;
    event_type: string;
    date_text: string | null;
    place_id: string | null;
    place_full_name: string | null;
    description: string | null;
    source_text: string | null;
    combined_media_count: number;
  };
  has_conflict: boolean;
  conflict_fields: string[];
  confidence: 'high' | 'medium';
  reason: string;
}

export const BIRTH_SAFE_GROUP: MockDuplicateEventGroup = {
  id: 'dup-evt-birth-1',
  event_type: 'BIRT',
  scope: 'individual',
  person: {
    id: 'p-ivan',
    gid: 'I0101',
    display_name: 'Ivan Petrovich Smirnov',
    sex: 'M',
    birth_date: '1924-05-12',
  },
  events: [
    {
      id: 'evt-birth-1a',
      gid: 'E1001',
      event_type: 'BIRT',
      custom_type: null,
      individual_id: 'p-ivan',
      family_id: null,
      date_text: '1924-05-12',
      date_sort: 19240512,
      place_id: 'pl-samara',
      place_full_name: 'Samara, Samara Governorate',
      description: 'Church parish entry',
      source_text: 'GAOO Fond 32',
      media_count: 1,
      is_private: false,
    },
    {
      id: 'evt-birth-1b',
      gid: 'E1002',
      event_type: 'BIRT',
      custom_type: null,
      individual_id: 'p-ivan',
      family_id: null,
      date_text: '1924-05-12',
      date_sort: 19240512,
      place_id: 'pl-samara',
      place_full_name: 'Samara, Samara Governorate',
      description: null,
      source_text: 'Family bible note',
      media_count: 2,
      is_private: false,
    },
  ],
  proposed: {
    keep_event_id: 'evt-birth-1a',
    keep_event_gid: 'E1001',
    event_type: 'BIRT',
    date_text: '1924-05-12',
    place_id: 'pl-samara',
    place_full_name: 'Samara, Samara Governorate',
    description: 'Church parish entry',
    source_text: 'GAOO Fond 32 | Family bible note',
    combined_media_count: 3,
  },
  has_conflict: false,
  conflict_fields: [],
  confidence: 'high',
  reason: 'Matching event type, date, and place with zero conflicting fields.',
};

export const DEATH_SAFE_GROUP: MockDuplicateEventGroup = {
  id: 'dup-evt-death-2',
  event_type: 'DEAT',
  scope: 'individual',
  person: {
    id: 'p-elena',
    gid: 'I0109',
    display_name: 'Elena Vasilyevna Sokolova',
    sex: 'F',
    birth_date: '1902',
    death_date: '1978-11-04',
  },
  events: [
    {
      id: 'evt-death-2a',
      gid: 'E3001',
      event_type: 'DEAT',
      custom_type: null,
      individual_id: 'p-elena',
      family_id: null,
      date_text: '1978-11-04',
      date_sort: 19781104,
      place_id: 'pl-tver',
      place_full_name: 'Tver',
      description: 'Civil death certificate',
      source_text: 'ZAGS #2 Tver',
      media_count: 1,
      is_private: false,
    },
    {
      id: 'evt-death-2b',
      gid: 'E3002',
      event_type: 'DEAT',
      custom_type: null,
      individual_id: 'p-elena',
      family_id: null,
      date_text: '1978-11-04',
      date_sort: 19781104,
      place_id: 'pl-tver',
      place_full_name: 'Tver',
      description: null,
      source_text: null,
      media_count: 0,
      is_private: false,
    },
  ],
  proposed: {
    keep_event_id: 'evt-death-2a',
    keep_event_gid: 'E3001',
    event_type: 'DEAT',
    date_text: '1978-11-04',
    place_id: 'pl-tver',
    place_full_name: 'Tver',
    description: 'Civil death certificate',
    source_text: 'ZAGS #2 Tver',
    combined_media_count: 1,
  },
  has_conflict: false,
  conflict_fields: [],
  confidence: 'high',
  reason: 'Exact match on death date and place; secondary record lacks notes.',
};

export const MARRIAGE_CONFLICT_GROUP: MockDuplicateEventGroup = {
  id: 'dup-evt-marr-1',
  event_type: 'MARR',
  scope: 'family',
  person: {
    id: 'p-sergei',
    gid: 'I0201',
    display_name: 'Sergei Nikolayevich Orlov',
    sex: 'M',
    birth_date: '1950-03-14',
  },
  partner: {
    id: 'p-maria',
    gid: 'I0202',
    display_name: 'Maria Ivanovna Orlova',
    sex: 'F',
    birth_date: '1952-08-21',
  },
  family_ids: ['fam-1a', 'fam-1b'],
  events: [
    {
      id: 'evt-marr-1a',
      gid: 'E2001',
      event_type: 'MARR',
      custom_type: null,
      individual_id: null,
      family_id: 'fam-1a',
      date_text: '1974-06-15',
      date_sort: 19740615,
      place_id: 'pl-moscow',
      place_full_name: 'Moscow',
      description: 'Civil registry',
      source_text: 'ZAGS #4 Moscow',
      media_count: 1,
      is_private: false,
    },
    {
      id: 'evt-marr-1b',
      gid: 'E2002',
      event_type: 'MARR',
      custom_type: null,
      individual_id: null,
      family_id: 'fam-1b',
      date_text: '1974-07-02',
      date_sort: 19740702,
      place_id: 'pl-kazan',
      place_full_name: 'Kazan',
      description: 'Church ceremony',
      source_text: 'Parish book 1974',
      media_count: 2,
      is_private: false,
    },
  ],
  proposed: {
    keep_event_id: 'evt-marr-1a',
    keep_event_gid: 'E2001',
    event_type: 'MARR',
    date_text: '1974-06-15',
    place_id: 'pl-moscow',
    place_full_name: 'Moscow',
    description: 'Civil registry',
    source_text: 'ZAGS #4 Moscow',
    combined_media_count: 3,
  },
  has_conflict: true,
  conflict_fields: ['date', 'place'],
  confidence: 'medium',
  reason: 'Conflicting date and place across two marriage entries (date / place).',
};

const DEFAULT_WARNINGS = [
  {
    id: 'warn-too-old-1',
    rule_code: 'TOO_OLD',
    severity: 'warning',
    entity_type: 'individual',
    entity_id: 'p-old-1',
    person_id: 'p-old-1',
    person_name: 'Fyodor Matveyevich Gromov',
    person_gid: 'I0045',
  },
];

const DEFAULT_DUPLICATE_PAIRS = [
  {
    id: 'dup-pair-1',
    person_a_id: 'p-pyotr-a',
    person_a_gid: 'I0012',
    person_a_name: 'Pyotr Fyodorovich Shcheglov',
    person_a_birth: '1868',
    person_a_death: '1931',
    person_b_id: 'p-pyotr-b',
    person_b_gid: 'I0388',
    person_b_name: 'Pyotr Shcheglov',
    person_b_birth: '1868-02-03',
    person_b_death: '1931',
    score: 92,
    reasons: ['shared_parents', 'vital_dates'],
    is_detached: false,
  },
];

export const DUPLICATE_EVENT_WARNING = {
  id: 'warn-dup-evt-1',
  rule_code: 'DUPLICATE_EVENT',
  severity: 'warning',
  entity_type: 'individual',
  entity_id: BIRTH_SAFE_GROUP.id,
  person_id: BIRTH_SAFE_GROUP.person.id,
  person_name: BIRTH_SAFE_GROUP.person.display_name,
  person_gid: BIRTH_SAFE_GROUP.person.gid,
};

interface QualityMockState {
  warnings: typeof DEFAULT_WARNINGS;
  duplicates: typeof DEFAULT_DUPLICATE_PAIRS;
  duplicateEvents: MockDuplicateEventGroup[];
}

export interface InstallQualityApiMockOptions {
  includeSecondSafeGroup?: boolean;
  emptyEvents?: boolean;
  includeDuplicates?: boolean;
  includeDuplicateEventWarning?: boolean;
  mergeError?: {
    status: number;
    code: string;
    message?: string;
  };
}

export interface QualityApiMockHandle {
  requests: MockRecordedRequest[];
  find: (method: string, pathPart: string) => MockRecordedRequest[];
  last: (method: string, pathPart: string) => MockRecordedRequest | undefined;
}

function json(route: Route, status: number, body: unknown) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  });
}

export async function installQualityApiMock(
  page: Page,
  options: InstallQualityApiMockOptions = {}
): Promise<QualityApiMockHandle> {
  const initialEvents: MockDuplicateEventGroup[] = options.emptyEvents
    ? []
    : options.includeSecondSafeGroup
      ? [
          structuredClone(BIRTH_SAFE_GROUP),
          structuredClone(DEATH_SAFE_GROUP),
          structuredClone(MARRIAGE_CONFLICT_GROUP),
        ]
      : [
          structuredClone(BIRTH_SAFE_GROUP),
          structuredClone(MARRIAGE_CONFLICT_GROUP),
        ];

  let state: QualityMockState = {
    warnings: options.includeDuplicateEventWarning
      ? [structuredClone(DUPLICATE_EVENT_WARNING), ...structuredClone(DEFAULT_WARNINGS)]
      : structuredClone(DEFAULT_WARNINGS),
    duplicates:
      options.includeDuplicates === false
        ? []
        : structuredClone(DEFAULT_DUPLICATE_PAIRS),
    duplicateEvents: initialEvents,
  };

  const snapshots = new Map<string, QualityMockState>();
  const requests: MockRecordedRequest[] = [];
  let txCounter = 0;

  const commit = (mutate: () => void): string => {
    const txId = `tx-qual-${++txCounter}`;
    snapshots.set(txId, structuredClone(state));
    mutate();
    return txId;
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

    if (method === 'GET' && path === '/quality/warnings') {
      requests.push({ method, path, body: null });
      return json(route, 200, {
        items: state.warnings,
        next_cursor: null,
        total: state.warnings.length,
      });
    }

    if (method === 'GET' && path === '/quality/duplicates') {
      requests.push({ method, path, body: null });
      return json(route, 200, {
        items: state.duplicates,
        next_cursor: null,
        total: state.duplicates.length,
      });
    }

    if (method === 'GET' && path === '/quality/events/duplicates') {
      requests.push({ method, path, body: null });
      return json(route, 200, {
        items: state.duplicateEvents,
        total: state.duplicateEvents.length,
      });
    }

    if (method === 'GET' && path === '/quality/places/analysis') {
      requests.push({ method, path, body: null });
      return json(route, 200, {
        items: [],
        counts: {
          total: 0,
          high_confidence: 0,
          medium_confidence: 0,
          low_confidence: 0,
          unparsed_paths: 0,
          orphans: 0,
          type_mismatches: 0,
          duplicates: 0,
        },
      });
    }

    if (method === 'POST' && path === '/quality/events/merge') {
      requests.push({ method, path, body });
      if (options.mergeError) {
        return json(route, options.mergeError.status, {
          error: {
            code: options.mergeError.code,
            message: options.mergeError.message ?? 'Error',
          },
        });
      }
      const groupId = typeof body?.group_id === 'string' ? body.group_id : undefined;
      const keepId =
        typeof body?.keep_event_id === 'string'
          ? body.keep_event_id
          : typeof body?.surviving_event_id === 'string'
            ? body.surviving_event_id
            : undefined;
      const mergeIdsRaw = Array.isArray(body?.merge_event_ids)
        ? body.merge_event_ids
        : Array.isArray(body?.duplicate_event_ids)
          ? body.duplicate_event_ids
          : [];
      const mergeIds = mergeIdsRaw.map((id) => String(id));

      const targetGroup =
        state.duplicateEvents.find((g) => g.id === groupId) ??
        state.duplicateEvents.find((g) =>
          g.events.some((ev) => ev.id === keepId || mergeIds.includes(ev.id))
        );

      if (!targetGroup) {
        return json(route, 404, {
          error: { code: 'not_found', message: 'Duplicate event group not found' },
        });
      }

      const resolvedKeep =
        targetGroup.events.find((ev) => ev.id === keepId) ?? targetGroup.events[0];
      const survivingEvent: MockDuplicateEventItem = {
        ...resolvedKeep,
        date_text:
          body && 'date_text' in body
            ? (body.date_text as string | null)
            : resolvedKeep.date_text,
        place_id:
          body && 'place_id' in body
            ? (body.place_id as string | null)
            : body && 'place_name' in body && body.place_name === ''
              ? null
              : resolvedKeep.place_id,
        place_full_name:
          body && 'place_name' in body
            ? (body.place_name as string | null)
            : resolvedKeep.place_full_name,
        description:
          body && 'description' in body
            ? (body.description as string | null)
            : resolvedKeep.description,
        source_text:
          body && 'source_text' in body
            ? (body.source_text as string | null)
            : resolvedKeep.source_text,
        media_count: targetGroup.proposed.combined_media_count,
      };
      const mergedEventIds =
        mergeIds.length > 0
          ? mergeIds
          : targetGroup.events
              .filter((ev) => ev.id !== resolvedKeep.id)
              .map((ev) => ev.id);

      const txId = commit(() => {
        state.duplicateEvents = state.duplicateEvents.filter(
          (g) => g.id !== targetGroup.id
        );
        state.warnings = state.warnings.filter(
          (w) => !(w.rule_code === 'DUPLICATE_EVENT' && w.entity_id === targetGroup.id)
        );
      });

      return json(route, 200, {
        status: 'merged',
        tx_id: txId,
        group_id: targetGroup.id,
        surviving_event: survivingEvent,
        merged_event_ids: mergedEventIds,
        merged_family_ids: targetGroup.family_ids ?? [],
      });
    }

    if (method === 'POST' && path === '/quality/events/batch-merge') {
      requests.push({ method, path, body });
      const groupIds = Array.isArray(body?.group_ids)
        ? body.group_ids.map((id) => String(id))
        : null;
      const onlyHighConfidence = Boolean(body?.only_high_confidence);

      const eligible = state.duplicateEvents.filter((g) => {
        if (groupIds && !groupIds.includes(g.id)) return false;
        if (onlyHighConfidence && (g.has_conflict || g.confidence !== 'high')) {
          return false;
        }
        return !g.has_conflict;
      });

      const eligibleIds = new Set(eligible.map((g) => g.id));
      const removedEventsCount = eligible.reduce(
        (sum, g) => sum + Math.max(0, g.events.length - 1),
        0
      );
      const survivingEventIds = eligible.map((g) => g.proposed.keep_event_id);

      const txId =
        eligible.length > 0
          ? commit(() => {
              state.duplicateEvents = state.duplicateEvents.filter(
                (g) => !eligibleIds.has(g.id)
              );
              state.warnings = state.warnings.filter(
                (w) =>
                  !(w.rule_code === 'DUPLICATE_EVENT' && eligibleIds.has(w.entity_id))
              );
            })
          : null;

      return json(route, 200, {
        status: 'ok',
        tx_id: txId,
        merged_groups_count: eligible.length,
        removed_events_count: removedEventsCount,
        surviving_event_ids: survivingEventIds,
      });
    }

    if (method === 'POST' && path === '/quality/warnings/dismiss') {
      requests.push({ method, path, body });
      const ruleCode = String(body?.rule_code ?? body?.code ?? '');
      const entityId = String(body?.entity_id ?? '');

      state.warnings = state.warnings.filter(
        (w) => !(w.rule_code === ruleCode && w.entity_id === entityId)
      );
      if (ruleCode === 'DUPLICATE_EVENT' && entityId) {
        state.duplicateEvents = state.duplicateEvents.filter(
          (g) => g.id !== entityId
        );
      }

      return json(route, 200, { status: 'dismissed' });
    }

    const undoMatch = path.match(/^\/history\/tx\/([^/]+)\/undo$/);
    if (method === 'POST' && undoMatch) {
      requests.push({ method, path, body: null });
      const txId = decodeURIComponent(undoMatch[1]);
      const snap = snapshots.get(txId);
      if (!snap) {
        return json(route, 404, {
          error: { code: 'not_found', message: `Unknown transaction ${txId}` },
        });
      }
      state = structuredClone(snap);
      return json(route, 200, { tx_id: txId, status: 'undone' });
    }

    return route.fallback();
  });

  return {
    requests,
    find: (method, pathPart) =>
      requests.filter((r) => r.method === method && r.path.includes(pathPart)),
    last: (method, pathPart) => {
      const matched = requests.filter(
        (r) => r.method === method && r.path.includes(pathPart)
      );
      return matched[matched.length - 1];
    },
  };
}
