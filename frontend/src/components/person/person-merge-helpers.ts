import { PersonMergePreviewDto, TreePersonNodeDto } from "../../api/client";

function extractYearFromNode(node: TreePersonNodeDto | undefined): number | null {
  if (!node) return null;
  if (typeof node.birth_sort === 'number' && node.birth_sort > 10000000) {
    return Math.floor(node.birth_sort / 10000);
  }
  const dateStr = node.birth?.date || '';
  const match = dateStr.match(/\b(1\d{3}|20\d{2})\b/);
  return match ? Number(match[1]) : null;
}

function surnamesOverlap(aRaw?: string | null, bRaw?: string | null): boolean {
  const a = (aRaw || '').trim().toLowerCase();
  const b = (bRaw || '').trim().toLowerCase();
  if (!a || !b) return false;
  if (a === b) return true;
  if (a.length >= 4 && b.length >= 4) {
    const prefixLen = Math.min(a.length, b.length) - 1;
    if (prefixLen >= 4 && a.slice(0, prefixLen) === b.slice(0, prefixLen)) {
      return true;
    }
  }
  return false;
}

export function formatTreePersonLabel(node: TreePersonNodeDto): string {
  const full = [
    node.primary_name?.given_name,
    node.primary_name?.patronymic,
    node.primary_name?.surname,
  ]
    .filter(Boolean)
    .join(' ')
    .trim();
  const namePart = full || node.gid || node.id;
  const birthYear = extractYearFromNode(node);
  return birthYear
    ? `${namePart} (${node.gid}) · *${birthYear}`
    : `${namePart} (${node.gid})`;
}

export function rankMergeCandidatesForTarget(
  targetNode: TreePersonNodeDto | undefined,
  allNodes: TreePersonNodeDto[]
): Array<{ person: TreePersonNodeDto; score: number }> {
  const others = allNodes.filter(
    (n) =>
      !targetNode ||
      (n.id !== targetNode.id && n.gid !== targetNode.gid)
  );
  if (!targetNode) {
    return others.map((person) => ({ person, score: 0 }));
  }

  const tgtGiven = (targetNode.primary_name?.given_name || '')
    .trim()
    .toLowerCase();
  const tgtSurname = targetNode.primary_name?.surname;
  const tgtBirthSurname = targetNode.birth_surname;
  const tgtYear = extractYearFromNode(targetNode);

  const scored = others.map((person) => {
    let score = 0;
    const candGiven = (person.primary_name?.given_name || '')
      .trim()
      .toLowerCase();
    if (tgtGiven && candGiven && tgtGiven === candGiven) {
      score += 20;
    }

    const candSurname = person.primary_name?.surname;
    const candBirthSurname = person.birth_surname;
    if (
      surnamesOverlap(tgtSurname, candSurname) ||
      surnamesOverlap(tgtSurname, candBirthSurname) ||
      surnamesOverlap(tgtBirthSurname, candSurname) ||
      surnamesOverlap(tgtBirthSurname, candBirthSurname)
    ) {
      score += 15;
    }

    if (
      targetNode.sex &&
      targetNode.sex !== 'U' &&
      person.sex === targetNode.sex
    ) {
      score += 5;
    }

    const candYear = extractYearFromNode(person);
    if (
      tgtYear !== null &&
      candYear !== null &&
      Math.abs(tgtYear - candYear) <= 5
    ) {
      score += 10;
    }

    return { person, score };
  });

  scored.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    return formatTreePersonLabel(a.person).localeCompare(
      formatTreePersonLabel(b.person)
    );
  });

  return scored;
}

export function buildMergePreviewFromTreeNodes(
  targetNode: TreePersonNodeDto | undefined,
  sourceNode: TreePersonNodeDto | undefined,
  targetId: string = 'node-deceased-1',
  sourceId: string = 'node-dup-2'
): PersonMergePreviewDto {
  const fallback = buildDefaultMergePreview(targetId, sourceId);
  return {
    target: targetNode
      ? {
        id: targetNode.id,
        gid: targetNode.gid,
        sex: targetNode.sex,
        is_living: targetNode.is_living,
        is_private: targetNode.is_private,
        primary_name: {
          given_name: targetNode.primary_name.given_name,
          patronymic: targetNode.primary_name.patronymic,
          surname: targetNode.primary_name.surname,
        },
        birth: targetNode.birth
          ? { date: targetNode.birth.date, place: targetNode.birth.place }
          : null,
        death: targetNode.death
          ? { date: targetNode.death.date, place: targetNode.death.place }
          : null,
        biography: targetNode.biography ?? null,
        avatar_url: targetNode.avatar_url ?? null,
        notes: targetNode.notes ?? null,
      }
      : fallback.target,
    source: sourceNode
      ? {
        id: sourceNode.id,
        gid: sourceNode.gid,
        sex: sourceNode.sex,
        is_living: sourceNode.is_living,
        is_private: sourceNode.is_private,
        primary_name: {
          given_name: sourceNode.primary_name.given_name,
          patronymic: sourceNode.primary_name.patronymic,
          surname: sourceNode.primary_name.surname,
        },
        birth: sourceNode.birth
          ? { date: sourceNode.birth.date, place: sourceNode.birth.place }
          : null,
        death: sourceNode.death
          ? { date: sourceNode.death.date, place: sourceNode.death.place }
          : null,
        biography: sourceNode.biography ?? null,
        avatar_url: sourceNode.avatar_url ?? null,
        notes: sourceNode.notes ?? null,
      }
      : fallback.source,
    combined_counts: fallback.combined_counts,
  };
}

export function buildDefaultMergePreview(
  targetId: string = 'node-deceased-1',
  sourceId: string = 'node-dup-2'
): PersonMergePreviewDto {
  return {
    target: {
      id: targetId,
      gid: 'I0012',
      sex: 'M',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Pyotr',
        patronymic: 'Fyodorovich',
        surname: 'Shcheglov',
      },
      birth: { date: 'abt 1868', place: 'Tver Governorate' },
      death: { date: '1931', place: 'Leningrad' },
      biography: 'Merchant of the 2nd Guild in Tver.',
      avatar_url: null,
      notes: 'Merchant of the 2nd Guild in Tver.',
    },
    source: {
      id: sourceId,
      gid: 'I0388',
      sex: 'M',
      is_living: false,
      is_private: false,
      primary_name: {
        given_name: 'Pyotr',
        patronymic: null,
        surname: 'Shcheglov',
      },
      birth: { date: '1868-02-03', place: 'Tver, Tver Governorate' },
      death: { date: '1931-11-14', place: 'Leningrad' },
      biography: 'Parish birth register entry #42.',
      avatar_url: null,
      notes: 'Parish birth register entry #42.',
    },
    combined_counts: {
      events: 6,
      media: 4,
      families: 2,
      notes: 3,
    },
  };
}

export interface ScalarMergeRow {
  field: string;
  labelKey: string;
  targetDisplay: string;
  sourceDisplay: string;
}

function countTransferField(val: unknown, fallback: number): number {
  if (typeof val === 'number') return val;
  if (Array.isArray(val)) return val.length;
  return fallback;
}

function normalizeMergeSide(
  raw: Record<string, unknown> | undefined | null,
  fallback: PersonMergePreviewDto['target']
): PersonMergePreviewDto['target'] {
  if (!raw) return fallback;
  const rawPrimary = raw.primary_name as
    | { given_name?: string; patronymic?: string | null; surname?: string }
    | undefined;
  const displayName =
    typeof raw.display_name === 'string' ? raw.display_name.trim() : '';
  const displayParts = displayName ? displayName.split(/\s+/) : [];
  const givenName =
    rawPrimary?.given_name ??
    (typeof raw.given_name === 'string' ? raw.given_name : undefined) ??
    displayParts[0] ??
    fallback.primary_name.given_name;
  const surname =
    rawPrimary?.surname ??
    (typeof raw.surname === 'string' ? raw.surname : undefined) ??
    (displayParts.length > 1 ? displayParts[displayParts.length - 1] : '') ??
    fallback.primary_name.surname;
  const patronymic =
    rawPrimary?.patronymic !== undefined
      ? rawPrimary.patronymic
      : typeof raw.patronymic === 'string'
        ? raw.patronymic
        : displayParts.length > 2
          ? displayParts.slice(1, -1).join(' ')
          : null;

  const rawBirth = raw.birth as
    | { date: string | null; place: string | null }
    | null
    | undefined;
  const rawDeath = raw.death as
    | { date: string | null; place: string | null }
    | null
    | undefined;

  return {
    ...fallback,
    ...(raw as Partial<PersonMergePreviewDto['target']>),
    id: typeof raw.id === 'string' ? raw.id : fallback.id,
    gid: typeof raw.gid === 'string' ? raw.gid : fallback.gid,
    sex:
      raw.sex === 'M' || raw.sex === 'F' || raw.sex === 'U'
        ? raw.sex
        : fallback.sex,
    primary_name: {
      given_name: givenName,
      patronymic,
      surname,
    },
    birth: rawBirth
      ? {
        date: rawBirth.date ?? null,
        place: rawBirth.place ?? null,
      }
      : {
        date: typeof raw.birth_date === 'string' ? raw.birth_date : null,
        place: typeof raw.birth_place === 'string' ? raw.birth_place : null,
      },
    death: rawDeath
      ? {
        date: rawDeath.date ?? null,
        place: rawDeath.place ?? null,
      }
      : {
        date: typeof raw.death_date === 'string' ? raw.death_date : null,
        place: typeof raw.death_place === 'string' ? raw.death_place : null,
      },
    biography: (raw.biography as string | null | undefined) ?? null,
    notes: (raw.notes as string | null | undefined) ?? null,
  };
}

export function computeDefaultFieldChoices(
  preview: PersonMergePreviewDto
): Record<string, 'target' | 'source'> {
  const tgt = preview.target;
  const src = preview.source;

  const isBlank = (val: unknown): boolean => {
    if (val === null || val === undefined) return true;
    const str = String(val).trim();
    return str === '' || str === '—' || str === '-';
  };

  const isSexEmpty = (val: unknown): boolean => {
    if (isBlank(val)) return true;
    const str = String(val).trim();
    return str === 'U';
  };

  const isDateApproximate = (dateStr: string): boolean => {
    return (
      /^(abt|about|calc|bef|before|aft|after|est|estimated|ок\.|около|до|после)\b/i.test(
        dateStr.trim()
      ) || dateStr.includes('?')
    );
  };

  const getDateSpecificity = (dateStr: string): number => {
    const trimmed = dateStr.trim();
    if (isBlank(trimmed)) return 0;

    const approx = isDateApproximate(trimmed);

    // Full date: YYYY-MM-DD or DD.MM.YYYY
    const isFullIso = /^\d{4}-\d{2}-\d{2}$/.test(trimmed);
    const isFullDmy = /^\d{1,2}[./-]\d{1,2}[./-]\d{2,4}$/.test(trimmed);
    // Year + Month: YYYY-MM
    const isYearMonth = /^\d{4}-\d{2}$/.test(trimmed);
    // Year only: YYYY
    const isYearOnly = /^\d{4}$/.test(trimmed);

    let score = 10;
    if (isFullIso || isFullDmy) {
      score = 50;
    } else if (isYearMonth) {
      score = 35;
    } else if (isYearOnly) {
      score = 20;
    } else {
      const digitCount = (trimmed.match(/\d/g) || []).length;
      score = 15 + Math.min(digitCount, 8);
    }

    if (approx) {
      score -= 10;
    }

    // Tie-breaker: longer date string usually has more precision/detail
    score += Math.min(trimmed.length, 20) * 0.1;

    return score;
  };

  const fields = [
    'name',
    'sex',
    'birth_date',
    'birth_place',
    'death_date',
    'biography',
  ] as const;

  const result: Record<string, 'target' | 'source'> = {};

  for (const field of fields) {
    let targetVal = '';
    let sourceVal = '';
    let targetEmpty = false;
    let sourceEmpty = false;

    switch (field) {
      case 'name': {
        targetVal = [
          tgt.primary_name?.given_name,
          tgt.primary_name?.patronymic,
          tgt.primary_name?.surname,
        ]
          .filter(Boolean)
          .join(' ')
          .trim();
        sourceVal = [
          src.primary_name?.given_name,
          src.primary_name?.patronymic,
          src.primary_name?.surname,
        ]
          .filter(Boolean)
          .join(' ')
          .trim();
        targetEmpty = isBlank(targetVal);
        sourceEmpty = isBlank(sourceVal);
        break;
      }
      case 'sex': {
        targetVal = tgt.sex ?? '';
        sourceVal = src.sex ?? '';
        targetEmpty = isSexEmpty(targetVal);
        sourceEmpty = isSexEmpty(sourceVal);
        break;
      }
      case 'birth_date': {
        targetVal = tgt.birth?.date ?? '';
        sourceVal = src.birth?.date ?? '';
        targetEmpty = isBlank(targetVal);
        sourceEmpty = isBlank(sourceVal);
        break;
      }
      case 'birth_place': {
        targetVal = tgt.birth?.place ?? '';
        sourceVal = src.birth?.place ?? '';
        targetEmpty = isBlank(targetVal);
        sourceEmpty = isBlank(sourceVal);
        break;
      }
      case 'death_date': {
        targetVal = tgt.death?.date ?? '';
        sourceVal = src.death?.date ?? '';
        targetEmpty = isBlank(targetVal);
        sourceEmpty = isBlank(sourceVal);
        break;
      }
      case 'biography': {
        targetVal = (tgt.biography || tgt.notes || '').trim();
        sourceVal = (src.biography || src.notes || '').trim();
        targetEmpty = isBlank(targetVal);
        sourceEmpty = isBlank(sourceVal);
        break;
      }
    }

    if (!targetEmpty && sourceEmpty) {
      result[field] = 'target';
    } else if (targetEmpty && !sourceEmpty) {
      result[field] = 'source';
    } else if (!targetEmpty && !sourceEmpty) {
      if (field === 'birth_date' || field === 'death_date') {
        const targetSpec = getDateSpecificity(targetVal);
        const sourceSpec = getDateSpecificity(sourceVal);
        if (sourceSpec > targetSpec) {
          result[field] = 'source';
        } else {
          result[field] = 'target';
        }
      } else {
        result[field] = 'target';
      }
    } else {
      // Both are empty
      result[field] = 'target';
    }
  }

  return result;
}

export function normalizeMergePreviewResponse(
  res: Partial<PersonMergePreviewDto> & Record<string, unknown>,
  targetId: string,
  sourceId: string
): PersonMergePreviewDto {
  const def = buildDefaultMergePreview(targetId, sourceId);
  const rawTarget = (res.target ?? res.keep_person) as
    | Record<string, unknown>
    | undefined;
  const rawSource = (res.source ?? res.merge_person) as
    | Record<string, unknown>
    | undefined;
  const target = normalizeMergeSide(rawTarget, def.target);
  const source = normalizeMergeSide(rawSource, def.source);

  const counts = res.combined_counts ?? {
    events: countTransferField(
      res.events_to_transfer,
      (Array.isArray(target.events) ? target.events.length : 0) +
      (Array.isArray(source.events) ? source.events.length : 0) ||
      def.combined_counts.events
    ),
    media: countTransferField(
      res.media_to_transfer,
      def.combined_counts.media
    ),
    families: countTransferField(
      res.families_to_transfer,
      (Array.isArray(target.families) ? target.families.length : 0) +
      (Array.isArray(source.families) ? source.families.length : 0) ||
      def.combined_counts.families
    ),
    notes: countTransferField(
      res.notes_to_transfer,
      def.combined_counts.notes
    ),
  };

  return {
    ...res,
    target,
    source,
    combined_counts: counts,
  };
}
