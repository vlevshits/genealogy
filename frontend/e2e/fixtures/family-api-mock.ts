import type { Page, Route } from '@playwright/test';

/**
 * Stateful mock of the family-link API surface used by the person profile
 * Family tab (profile, unlink/detach, move-to-family, undo, people search,
 * quick-add). Mutations push a snapshot so `POST /history/tx/:id/undo`
 * restores the previous state, mirroring the real backend contract.
 */

export interface MockPerson {
  id: string;
  gid: string;
  given_name: string;
  surname: string;
  sex: 'M' | 'F';
  birth_year: number;
}

export const SUBJECT: MockPerson = {
  id: 'p-subject',
  gid: 'I0100',
  given_name: 'Alexei',
  surname: 'Morozov',
  sex: 'M',
  birth_year: 1990,
};
export const FATHER: MockPerson = {
  id: 'p-father',
  gid: 'I0101',
  given_name: 'Viktor',
  surname: 'Morozov',
  sex: 'M',
  birth_year: 1960,
};
export const MOTHER: MockPerson = {
  id: 'p-mother',
  gid: 'I0102',
  given_name: 'Elena',
  surname: 'Morozova',
  sex: 'F',
  birth_year: 1962,
};
export const SPOUSE: MockPerson = {
  id: 'p-spouse',
  gid: 'I0103',
  given_name: 'Olga',
  surname: 'Morozova',
  sex: 'F',
  birth_year: 1991,
};
export const CHILD: MockPerson = {
  id: 'p-child',
  gid: 'I0104',
  given_name: 'Nikita',
  surname: 'Morozov',
  sex: 'M',
  birth_year: 2018,
};
/** Candidates for "Change parents" and "Link Existing Person". */
export const GALINA: MockPerson = {
  id: 'p-galina',
  gid: 'I0201',
  given_name: 'Galina',
  surname: 'Sidorova',
  sex: 'F',
  birth_year: 1965,
};
export const BORIS: MockPerson = {
  id: 'p-boris',
  gid: 'I0202',
  given_name: 'Boris',
  surname: 'Sidorov',
  sex: 'M',
  birth_year: 1963,
};
/** Descendant of the subject: choosing it as a parent yields `cycle_detected`. */
export const DMITRI: MockPerson = {
  id: 'p-dmitri',
  gid: 'I0203',
  given_name: 'Dmitri',
  surname: 'Orlov',
  sex: 'M',
  birth_year: 2020,
};

const DIRECTORY: MockPerson[] = [SUBJECT, FATHER, MOTHER, SPOUSE, CHILD, GALINA, BORIS, DMITRI];

export const displayName = (p: MockPerson) => `${p.given_name} ${p.surname}`;

interface RelDto {
  id: string;
  gid: string;
  given_name: string;
  surname: string;
  display_name: string;
  sex: string;
  is_living: boolean;
  birth_year: number;
}

const rel = (p: MockPerson): RelDto => ({
  id: p.id,
  gid: p.gid,
  given_name: p.given_name,
  surname: p.surname,
  display_name: displayName(p),
  sex: p.sex,
  is_living: true,
  birth_year: p.birth_year,
});

interface ParentFamily {
  family_id: string;
  gid: string;
  father: RelDto | null;
  mother: RelDto | null;
  child_relation: string;
  siblings: RelDto[];
  version: number;
}

interface SpouseFamily {
  family_id: string;
  gid: string;
  relationship_type: string;
  spouse: RelDto | null;
  children: Array<{ person: RelDto; sort_order: number; rel_to_partner1: string }>;
  version: number;
}

interface FamilyState {
  parent_families: ParentFamily[];
  spouse_families: SpouseFamily[];
}

export interface RecordedRequest {
  method: string;
  path: string;
  body: Record<string, unknown> | null;
}

export interface FamilyApiMockOptions {
  /** Parent family holds only the father (single-parent union). */
  singleParent?: boolean;
  /** Mock role-based capability (`viewer` has `can_edit: false`). */
  canEdit?: boolean;
}

export interface FamilyApiMock {
  requests: RecordedRequest[];
  /** Requests matching method + path suffix. */
  find: (method: string, pathPart: string) => RecordedRequest[];
}

const json = (route: Route, status: number, body: unknown, headers?: Record<string, string>) =>
  route.fulfill({
    status,
    contentType: 'application/json',
    headers,
    body: JSON.stringify(body),
  });

function initialState(singleParent: boolean): FamilyState {
  return {
    parent_families: [
      {
        family_id: 'fam-parents',
        gid: 'F0100',
        father: rel(FATHER),
        mother: singleParent ? null : rel(MOTHER),
        child_relation: 'birth',
        siblings: [],
        version: 1,
      },
    ],
    spouse_families: [
      {
        family_id: 'fam-marriage',
        gid: 'F0101',
        relationship_type: 'married',
        spouse: rel(SPOUSE),
        children: [{ person: rel(CHILD), sort_order: 0, rel_to_partner1: 'birth' }],
        version: 1,
      },
    ],
  };
}

function subjectNode() {
  return {
    id: SUBJECT.id,
    gid: SUBJECT.gid,
    sex: SUBJECT.sex,
    is_living: true,
    is_private: false,
    primary_name: { given_name: SUBJECT.given_name, patronymic: null, surname: SUBJECT.surname },
    birth: { date: String(SUBJECT.birth_year), place: null },
    death: null,
    avatar_url: null,
    notes: null,
    version: 1,
  };
}

const familyStub = (id: string) => ({ id, gid: id, version: 2 });

export async function installFamilyApiMock(
  page: Page,
  options: FamilyApiMockOptions = {}
): Promise<FamilyApiMock> {
  const canEdit = options.canEdit ?? true;
  let state = initialState(options.singleParent ?? false);
  const snapshots = new Map<string, FamilyState>();
  const requests: RecordedRequest[] = [];
  let txCounter = 0;

  const commit = (mutate: () => void): string => {
    const txId = `tx-${++txCounter}`;
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

    // Profile of the subject person.
    if (method === 'GET' && path === `/persons/${SUBJECT.id}/profile`) {
      return json(route, 200, {
        person: subjectNode(),
        kinship: null,
        events: [],
        families: state,
        media: [],
        notes: [],
        sources_summary: [],
        capabilities: {
          can_edit: canEdit,
          can_delete: canEdit,
          can_merge: false,
          is_home_person: false,
        },
      });
    }

    // People directory search (used by Combobox pickers).
    if (method === 'GET' && path === '/people') {
      requests.push({ method, path: `${path}${url.search}`, body: null });
      const q = (url.searchParams.get('q') ?? '').trim().toLowerCase();
      const items = DIRECTORY.filter(
        (p) => !q || `${displayName(p)} ${p.gid}`.toLowerCase().includes(q)
      ).map((p) => ({
        id: p.id,
        gid: p.gid,
        given_name: p.given_name,
        patronymic: null,
        surname: p.surname,
        display_name: displayName(p),
        sex: p.sex,
        birth_date: String(p.birth_year),
        birth_year: p.birth_year,
        is_living: true,
        is_private: false,
        has_parents: false,
        has_birth_date: true,
        version: 1,
      }));
      return json(route, 200, {
        items,
        next_cursor: null,
        total: items.length,
        total_unfiltered: DIRECTORY.length,
      });
    }

    const isMutation = method !== 'GET';
    if (isMutation) requests.push({ method, path, body });

    // Detach spouse.
    let m = path.match(/^\/families\/([^/]+)\/partners\/([^/]+)$/);
    if (method === 'DELETE' && m) {
      const txId = commit(() => {
        const fam = state.spouse_families.find((f) => f.family_id === m![1]);
        if (fam) fam.spouse = null;
      });
      return json(route, 200, { ...familyStub(m[1]), tx_id: txId, dissolved: false });
    }

    // Detach child (works for spouse-family children and for the subject leaving a parent family).
    m = path.match(/^\/families\/([^/]+)\/children\/([^/]+)$/);
    if (method === 'DELETE' && m) {
      const txId = commit(() => {
        const sf = state.spouse_families.find((f) => f.family_id === m![1]);
        if (sf) sf.children = sf.children.filter((c) => c.person.id !== m![2]);
        state.parent_families = state.parent_families.filter((f) => f.family_id !== m![1]);
      });
      return json(route, 200, { ...familyStub(m[1]), tx_id: txId, dissolved: false });
    }

    // Move to family: "unlink one parent only" and "Change parents".
    if (method === 'POST' && path === `/persons/${SUBJECT.id}/move-to-family`) {
      const p1 = body?.partner1_id as string | undefined;
      const p2 = body?.partner2_id as string | undefined;
      if ([p1, p2].includes(DMITRI.id)) {
        return json(route, 400, {
          error: {
            code: 'cycle_detected',
            message: 'Relationship cycle detected',
          },
        });
      }
      const txId = commit(() => {
        const chosen = [p1, p2]
          .filter((id): id is string => Boolean(id))
          .map((id) => DIRECTORY.find((p) => p.id === id))
          .filter((p): p is MockPerson => Boolean(p));
        const father = chosen.find((p) => p.sex === 'M') ?? null;
        const mother = chosen.find((p) => p.sex === 'F') ?? null;
        state.parent_families = [
          {
            family_id: 'fam-moved',
            gid: 'F0199',
            father: father ? rel(father) : null,
            mother: mother ? rel(mother) : null,
            child_relation: 'birth',
            siblings: [],
            version: 1,
          },
        ];
      });
      return json(route, 200, {
        person_id: SUBJECT.id,
        target_family: familyStub('fam-moved'),
        target_family_created: true,
        source_family_id: 'fam-parents',
        source_family: null,
        source_family_dissolved: false,
        tx_id: txId,
      });
    }

    // Undo.
    m = path.match(/^\/history\/tx\/([^/]+)\/undo$/);
    if (method === 'POST' && m) {
      const snap = snapshots.get(m[1]);
      if (!snap) return json(route, 404, { error: { code: 'not_found', message: 'Unknown tx' } });
      state = structuredClone(snap);
      return json(route, 200, { tx_id: m[1], status: 'undone' });
    }

    // Quick-add (create or link existing relative).
    if (method === 'POST' && path === `/persons/${SUBJECT.id}/quick-add`) {
      const existing = DIRECTORY.find((p) => p.id === body?.existing_person_id);
      if (!existing) {
        return json(route, 400, { error: { code: 'validation_failed', message: 'Validation failed' } });
      }
      const relation = body?.relation as string;
      if (relation === 'sibling') state.parent_families[0]?.siblings.push(rel(existing));
      if (relation === 'child') {
        state.spouse_families[0]?.children.push({
          person: rel(existing),
          sort_order: 1,
          rel_to_partner1: 'birth',
        });
      }
      if (relation === 'spouse') {
        state.spouse_families.push({
          family_id: 'fam-new-union',
          gid: 'F0102',
          relationship_type: 'married',
          spouse: rel(existing),
          children: [],
          version: 1,
        });
      }
      return json(route, 201, { person: { ...subjectNode(), id: existing.id }, family: null });
    }

    return route.fallback();
  });

  return {
    requests,
    find: (method, pathPart) =>
      requests.filter((r) => r.method === method && r.path.includes(pathPart)),
  };
}
