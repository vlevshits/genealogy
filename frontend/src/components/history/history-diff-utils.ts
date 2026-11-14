import type { ChangeLogEntryDto, FieldDiffDto } from '../../api/client';
import { formatGenDate, parseGenDate } from '../../lib/gendate-parser';
import type { GenDate, GenDateModifier, PartialDate } from '../../types/gendate';
import type { Locale } from '../../types/common';
import {
  normalizeEventType,
  normalizePedigreeType,
  normalizePlaceType,
  normalizeRelationshipType,
  normalizeSex,
} from '../../lib/domain-enums';
import {
  ALT_NAME_TYPE_MAP,
  BUNDLE_RAW_KEYS,
  LIVING_OVERRIDE_MAP,
  NOISE_DIFF_KEYS,
} from './history-diff-constants';
import {
  HistoryTranslateFn,
  formatCompactRef,
  formatEntityTypeLabel,
  formatFieldLabel,
  formatRecordCounts,
  formatUndoneTxRef,
  isRawUuid,
  localizeEmbeddedDisplayString,
  resolveKey,
  toSnakeCaseKey,
  toTitleCase,
} from './history-label-formatters';

export {
  type HistoryTranslateFn,
  toSnakeCaseKey,
  isRawUuid,
  extractEntityGid,
  formatEntityTypeLabel,
  formatFieldLabel,
  formatTxLabel,
  formatEntityLabel,
  formatRecordCounts,
} from './history-label-formatters';

export interface FormatDiffValueOptions {
  t?: HistoryTranslateFn;
  field?: string;
  locale?: Locale;
}

const GEN_DATE_MOD_MAP: Record<string, GenDateModifier> = {
  exact: 'exact',
  abt: 'about',
  about: 'about',
  est: 'estimated',
  estimated: 'estimated',
  cal: 'calculated',
  calculated: 'calculated',
  bef: 'before',
  before: 'before',
  aft: 'after',
  after: 'after',
  bet: 'between',
  between: 'between',
  from: 'from_to',
  from_to: 'from_to',
  phrase: 'phrase',
};

const DATE_DIFF_FIELDS = new Set([
  'birth_date',
  'death_date',
  'marriage_date',
  'date',
  'gedcom_date',
  'date_text',
]);

function toPartialDate(raw: unknown): PartialDate | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  const y = typeof r.y === 'number' ? r.y : typeof r.year === 'number' ? r.year : null;
  if (y == null || Number.isNaN(y)) return null;
  const m = typeof r.m === 'number' ? r.m : typeof r.month === 'number' ? r.month : null;
  const d = typeof r.d === 'number' ? r.d : typeof r.day === 'number' ? r.day : null;
  return { y, m, d };
}

function tryFormatDateString(raw: string, locale: Locale): string | null {
  const parsed = parseGenDate(raw, locale);
  if (parsed.isValid && parsed.value.mod !== 'phrase') {
    const formatted = formatGenDate(parsed.value, locale);
    return formatted || null;
  }
  return null;
}

function formatGenDateObject(
  obj: Record<string, unknown>,
  emptyLabel: string,
  locale?: Locale
): string {
  if (locale) {
    const modRaw = String(obj.mod ?? obj.precision ?? '').trim().toLowerCase();
    const mod = GEN_DATE_MOD_MAP[modRaw] ?? 'exact';
    const calRaw = String(obj.cal ?? obj.calendar ?? '').trim().toLowerCase();
    const cal: GenDate['cal'] = calRaw === 'julian' ? 'julian' : 'gregorian';
    const explicitStart = toPartialDate(obj.start);
    const end = toPartialDate(obj.end);

    if (explicitStart && mod !== 'phrase') {
      const genDate: GenDate = {
        mod,
        start: explicitStart,
        end,
        cal,
        text: typeof obj.text === 'string' ? obj.text : '',
      };
      const formatted = formatGenDate(genDate, locale);
      if (formatted) return formatted;
    }

    if (typeof obj.text === 'string' && obj.text.trim()) {
      const formattedText = tryFormatDateString(obj.text, locale);
      if (formattedText) return formattedText;
    }

    if (typeof obj.year === 'number' && !Number.isNaN(obj.year) && mod !== 'phrase') {
      const formattedYear = formatGenDate(
        {
          mod,
          start: { y: obj.year, m: null, d: null },
          end: null,
          cal,
          text: typeof obj.text === 'string' ? obj.text : '',
        },
        locale
      );
      if (formattedYear) return formattedYear;
    }
  }

  const startObj = obj.start as Record<string, unknown> | null | undefined;
  const startStr =
    startObj && typeof startObj === 'object' && 'y' in startObj
      ? [startObj.d, startObj.m, startObj.y].filter(Boolean).join('.')
      : undefined;
  return String(obj.text || obj.gedcom || obj.year || startStr || emptyLabel);
}

function formatLocalizedScalar(
  val: string,
  field: string | undefined,
  t: HistoryTranslateFn | undefined,
  locale?: Locale
): string {
  const normField = field ? toSnakeCaseKey(field) : '';
  const cleaned = val.trim().toLowerCase();
  if (locale && DATE_DIFF_FIELDS.has(normField)) {
    const formattedDate = tryFormatDateString(val, locale);
    if (formattedDate) return formattedDate;
  }
  if (!t) return val;
  if (normField === 'undone_tx_id') {
    return formatUndoneTxRef(val, t);
  }
  if (normField === 'record_counts') {
    return formatRecordCounts(val, val, t);
  }
  if (normField === 'sex') {
    if (cleaned === 'other' || cleaned === 'o') return resolveKey(t, 'domain.sex.other') ?? val;
    const code = normalizeSex(val);
    const key = code === 'M' ? 'domain.sex.male' : code === 'F' ? 'domain.sex.female' : 'domain.sex.unknown';
    return resolveKey(t, key) ?? val;
  }
  if (normField === 'event_type') {
    return resolveKey(t, `events.types.${cleaned}`) ?? resolveKey(t, `domain.events.${normalizeEventType(val)}`) ?? val;
  }
  if (normField === 'relationship_type') {
    return resolveKey(t, `domain.relationships.${cleaned}`) ?? resolveKey(t, `domain.relationships.${normalizeRelationshipType(val)}`) ?? val;
  }
  if (normField === 'place_type') {
    return resolveKey(t, `places.types.${cleaned}`) ?? resolveKey(t, `places.types.${normalizePlaceType(val)}`) ?? val;
  }
  if (normField === 'rel_to_partner1' || normField === 'rel_to_partner2') {
    return resolveKey(t, `domain.pedigree.${normalizePedigreeType(val)}`) ?? val;
  }
  if (normField === 'living_override') {
    return (LIVING_OVERRIDE_MAP[cleaned] && resolveKey(t, LIVING_OVERRIDE_MAP[cleaned])) ?? val;
  }
  if (normField === 'status' || normField === 'name_type') {
    return resolveKey(t, `history.values.${cleaned}`) ?? val;
  }
  return localizeEmbeddedDisplayString(val, t);
}

function formatObjectValue(
  obj: Record<string, unknown>,
  emptyLabel: string,
  options?: FormatDiffValueOptions
): string {
  const t = options?.t;
  if (
    ('text' in obj && ('mod' in obj || 'cal' in obj || 'start' in obj || 'precision' in obj || 'calendar' in obj || 'year' in obj)) ||
    ('mod' in obj && 'start' in obj)
  ) {
    return formatGenDateObject(obj, emptyLabel, options?.locale);
  }
  if ('child_id' in obj) {
    const label = obj.child_name ?? obj.display_name ?? obj.name;
    const childRef = label ? String(label) : formatCompactRef(String(obj.child_id ?? ''));
    const rawRel = (typeof obj.rel_to_partner1 === 'string' && obj.rel_to_partner1) ||
      (typeof obj.rel_to_partner2 === 'string' && obj.rel_to_partner2) || '';
    if (rawRel) {
      const relLabel = resolveKey(t, `domain.pedigree.${normalizePedigreeType(rawRel)}`) ?? toTitleCase(rawRel);
      return `${childRef} (${relLabel})`;
    }
    return childRef || emptyLabel;
  }
  if ('name_type' in obj && ('given_name' in obj || 'surname' in obj || 'patronymic' in obj)) {
    const fullName = [obj.given_name, obj.patronymic, obj.surname].filter(Boolean).join(' ');
    const nt = String(obj.name_type ?? '').toLowerCase();
    const typeLabel = (ALT_NAME_TYPE_MAP[nt] && resolveKey(t, ALT_NAME_TYPE_MAP[nt])) ??
      resolveKey(t, `history.values.${nt}`) ?? toTitleCase(nt);
    return fullName ? (typeLabel ? `${fullName} (${typeLabel})` : fullName) : (typeLabel || emptyLabel);
  }
  if ('name' in obj && ('lang' in obj || 'note' in obj) && !('place_type' in obj)) {
    const nameStr = obj.name ? String(obj.name) : '';
    if (!nameStr) return emptyLabel;
    return obj.lang ? `${nameStr} (${String(obj.lang)})` : nameStr;
  }
  if ('person_id' in obj && ('x' in obj || 'person_name' in obj)) {
    return obj.person_name ? String(obj.person_name) : formatCompactRef(String(obj.person_id ?? '')) || emptyLabel;
  }
  if ('media_id' in obj && 'entity_type' in obj && 'entity_id' in obj) {
    const et = t ? formatEntityTypeLabel(String(obj.entity_type), t) : toTitleCase(String(obj.entity_type));
    const ref = formatDiffValue(obj.entity_id, emptyLabel, { t });
    return `${et} ${ref}${obj.role ? ` (${String(obj.role)})` : ''}`;
  }
  if ('gid' in obj || 'primary_name' in obj) {
    const pName = (obj.primary_name ?? obj) as Record<string, unknown>;
    const nameParts = [pName?.given_name ?? obj.given_name, pName?.patronymic ?? obj.patronymic, pName?.surname ?? obj.surname].filter(Boolean).join(' ');
    const fallback = t ? formatEntityTypeLabel('person', t) : 'Person';
    return `${nameParts || fallback}${obj.gid ? ` (${String(obj.gid)})` : ''}`;
  }
  const nonEmptyObjKeys = Object.keys(obj).filter(
    (k) => !NOISE_DIFF_KEYS.has(k) && obj[k] !== undefined && obj[k] !== null && obj[k] !== ''
  );
  if (nonEmptyObjKeys.length > 0 && nonEmptyObjKeys.every((k) => k === 'given_name' || k === 'patronymic' || k === 'surname')) {
    const parts = [obj.given_name, obj.patronymic, obj.surname].filter(Boolean);
    if (parts.length > 0) return parts.join(' ');
  }
  if ('event_type' in obj) {
    const evType = t ? formatLocalizedScalar(String(obj.event_type), 'event_type', t, options?.locale) : toTitleCase(String(obj.event_type));
    const dateVal = obj.date ?? obj.gedcom_date ?? obj.date_text;
    const datePart = dateVal ? formatDiffValue(dateVal, '', { t, locale: options?.locale, field: 'date' }) : '';
    const details = [datePart, String(obj.place_full_name || obj.place || '')].filter(Boolean).join(', ');
    return details ? `${evType} — ${details}` : evType;
  }
  if (
    nonEmptyObjKeys.length > 0 &&
    nonEmptyObjKeys.every((k) => k === 'date' || k === 'place' || k === 'place_full_name' || k === 'gedcom_date')
  ) {
    const rawDate = obj.date ?? obj.gedcom_date;
    const datePart = rawDate
      ? formatDiffValue(rawDate, '', { t, locale: options?.locale, field: 'date' })
      : rawDate;
    const parts = [datePart, obj.place ?? obj.place_full_name].filter(Boolean);
    return parts.length > 0 ? parts.join(', ') : emptyLabel;
  }
  const hasPlaceName = Boolean(obj.place_full_name || obj.place);
  const hasPersonName = Boolean(obj.person_name);
  const entries = Object.entries(obj).filter(([k, v]) => {
    if (NOISE_DIFF_KEYS.has(k) || k === 'sort_order') return false;
    if (k === 'place_id' && hasPlaceName) return false;
    if (k === 'person_id' && hasPersonName) return false;
    return !isEmptyDiffValue(v);
  });
  if (entries.length === 0) {
    if (typeof obj.gid === 'string' && obj.gid) return obj.gid;
    if (typeof obj.id === 'string' && isRawUuid(obj.id)) return formatCompactRef(obj.id);
    return emptyLabel;
  }
  const limit = 6;
  const formatted = entries.slice(0, limit).map(([k, v]) => {
    const keyLabel = t ? formatFieldLabel(k, t) : toTitleCase(k);
    const valLabel = formatDiffValue(v, emptyLabel, { t, locale: options?.locale, field: k });
    return `${keyLabel}: ${valLabel}`;
  });
  return entries.length > limit ? `${formatted.join(', ')} (+${entries.length - limit})` : formatted.join(', ');
}

export function formatDiffValue(
  val: unknown,
  emptyLabel: string = '—',
  options?: FormatDiffValueOptions
): string {
  if (val === null || val === undefined || val === '') return emptyLabel;
  if (typeof val === 'boolean') {
    return resolveKey(options?.t, val ? 'common.yes' : 'common.no') ?? (val ? 'Yes' : 'No');
  }
  if (typeof val === 'number') return String(val);
  if (typeof val === 'string') {
    if (isRawUuid(val)) return formatCompactRef(val);
    return formatLocalizedScalar(val, options?.field, options?.t, options?.locale);
  }
  if (Array.isArray(val)) {
    if (val.length === 0) return emptyLabel;
    const formatted = val
      .map((item) => formatDiffValue(item, emptyLabel, { t: options?.t, locale: options?.locale, field: options?.field }))
      .filter((s) => s && s !== emptyLabel);
    if (formatted.length === 0) return emptyLabel;
    return formatted.length <= 3 ? formatted.join(', ') : `${formatted.slice(0, 3).join(', ')} (+${formatted.length - 3})`;
  }
  if (typeof val === 'object') {
    if (options?.field && toSnakeCaseKey(options.field) === 'record_counts') {
      return formatRecordCounts(val, emptyLabel, options.t);
    }
    return formatObjectValue(val as Record<string, unknown>, emptyLabel, options);
  }
  return String(val);
}

function isEmptyDiffValue(val: unknown): boolean {
  if (val === null || val === undefined || val === '') return true;
  if (Array.isArray(val) && val.length === 0) return true;
  return typeof val === 'object' && !Array.isArray(val) && Object.keys(val as object).length === 0;
}

function flattenPrimaryName(obj: Record<string, unknown> | null | undefined): Record<string, unknown> {
  if (!obj || typeof obj !== 'object') return {};
  const copy: Record<string, unknown> = { ...obj };
  if (copy.primary_name && typeof copy.primary_name === 'object' && !Array.isArray(copy.primary_name)) {
    const pn = copy.primary_name as Record<string, unknown>;
    if (!('given_name' in copy) && pn.given_name !== undefined) copy.given_name = pn.given_name;
    if (!('patronymic' in copy) && pn.patronymic !== undefined) copy.patronymic = pn.patronymic;
    if (!('surname' in copy) && pn.surname !== undefined) copy.surname = pn.surname;
    delete copy.primary_name;
  }
  return copy;
}

function filterNoiseDiffs(diffs: FieldDiffDto[]): FieldDiffDto[] {
  const fieldsPresent = new Set(diffs.map((d) => d.field));
  return diffs.filter((d) => {
    if (NOISE_DIFF_KEYS.has(d.field)) return false;
    if (d.field === 'place_id' && fieldsPresent.has('place_full_name')) return false;
    if (d.field === 'gedcom_date' && fieldsPresent.has('date')) return false;
    return true;
  });
}

export function extractEntryDiffs(entry: ChangeLogEntryDto): FieldDiffDto[] {
  const rawEntry = entry as ChangeLogEntryDto & {
    before?: Record<string, unknown> | null;
    after?: Record<string, unknown> | null;
  };
  const rawBefore = (entry.before_json ?? rawEntry.before ?? {}) as Record<string, unknown>;
  const rawAfter = (entry.after_json ?? rawEntry.after ?? {}) as Record<string, unknown>;
  const isMergeBundle = rawBefore?.kind === 'person_merge_bundle' || rawBefore?.kind === 'event_merge_bundle';
  const hasRawBundleDiffs = isMergeBundle && Array.isArray(entry.diffs) && entry.diffs.some((d) => BUNDLE_RAW_KEYS.has(d.field));

  if (entry.diffs && entry.diffs.length > 0 && !hasRawBundleDiffs) {
    return filterNoiseDiffs(entry.diffs);
  }

  let beforeObj: Record<string, unknown> = rawBefore ?? {};
  const afterObj: Record<string, unknown> = rawAfter ?? {};
  const syntheticDiffs: FieldDiffDto[] = [];

  if (beforeObj?.kind === 'person_merge_bundle') {
    const keepBefore = (beforeObj.keep_before ?? {}) as Record<string, unknown>;
    const mergePerson = (beforeObj.merge_person ?? {}) as Record<string, unknown>;
    syntheticDiffs.push({
      field: 'merged_profile',
      before: formatDiffValue(mergePerson, '—'),
      after: formatDiffValue(keepBefore, '—'),
    });
    beforeObj = keepBefore;
  } else if (beforeObj?.kind === 'event_merge_bundle') {
    const eventsBefore = Array.isArray(beforeObj.events_before) ? (beforeObj.events_before as Record<string, unknown>[]) : [];
    if ('merged_groups_count' in afterObj || 'surviving_event_ids' in afterObj) {
      if (afterObj.merged_groups_count !== undefined) syntheticDiffs.push({ field: 'merged_groups_count', before: null, after: afterObj.merged_groups_count });
      if (afterObj.removed_events_count !== undefined) syntheticDiffs.push({ field: 'removed_events_count', before: null, after: afterObj.removed_events_count });
      if (eventsBefore.length > 0) syntheticDiffs.push({ field: 'merged_events', before: eventsBefore, after: afterObj.surviving_event_ids ?? null });
      return filterNoiseDiffs(syntheticDiffs);
    }
    const survivingId = (typeof afterObj.id === 'string' && afterObj.id) || entry.entity_id;
    const survivingBefore = eventsBefore.find((ev) => ev.id === survivingId) ?? eventsBefore[0] ?? {};
    const mergedAway = eventsBefore.filter((ev) => ev !== survivingBefore);
    if (eventsBefore.length > 0) {
      syntheticDiffs.push({
        field: 'merged_events',
        before: mergedAway.length > 0 ? mergedAway : eventsBefore,
        after: Object.keys(afterObj).length > 0 ? afterObj : survivingBefore,
      });
    }
    beforeObj = survivingBefore;
  } else if (beforeObj && typeof beforeObj.person === 'object' && beforeObj.person !== null && Array.isArray(beforeObj.events)) {
    beforeObj = beforeObj.person as Record<string, unknown>;
  }

  const flatBefore = flattenPrimaryName(beforeObj);
  const flatAfter = flattenPrimaryName(afterObj);
  const hasBefore = Object.keys(flatBefore).length > 0;
  const hasAfter = Object.keys(flatAfter).length > 0;
  const keys = Array.from(new Set([...Object.keys(flatBefore), ...Object.keys(flatAfter)]));

  const computed: FieldDiffDto[] = [...syntheticDiffs];
  for (const key of keys) {
    const bVal = flatBefore[key];
    const aVal = flatAfter[key];
    if (!hasBefore && hasAfter) {
      if (!isEmptyDiffValue(aVal)) computed.push({ field: key, before: null, after: aVal });
      continue;
    }
    if (hasBefore && !hasAfter) {
      if (!isEmptyDiffValue(bVal)) computed.push({ field: key, before: bVal, after: null });
      continue;
    }
    if (isEmptyDiffValue(bVal) && isEmptyDiffValue(aVal)) continue;
    if (JSON.stringify(bVal ?? null) !== JSON.stringify(aVal ?? null)) {
      computed.push({ field: key, before: bVal, after: aVal });
    }
  }
  return filterNoiseDiffs(computed);
}
