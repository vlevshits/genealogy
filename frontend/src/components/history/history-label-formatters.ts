import type { ChangeLogEntryDto } from '../../api/client';
import { normalizeEventType, normalizePedigreeType } from '../../lib/domain-enums';
import {
  ENTITY_TYPE_ALIAS_MAP,
  TRAILING_GID_REGEX,
  TX_LABEL_MAP,
  UUID_REGEX,
} from './history-diff-constants';

export type HistoryTranslateFn = (key: string, params?: Record<string, string | number>) => string;

const EXPLICIT_OTHER_EVENT_TOKENS = new Set(['even', 'event', 'fact', 'custom', 'other']);
const COMPOSITE_SEP_RE = /^(.+?)\s+[—-]\s+(.+)$/;
const PEDIGREE_BRACKET_RE =
  /(?:\[|\()((?:biological|birth|adopted|foster|step|stepchild)(?:\s*\/\s*(?:biological|birth|adopted|foster|step|stepchild))*)(?:\]|\))/gi;
const PRIMARY_AVATAR_BRACKET_RE = /\[primary\s+avatar\]/gi;

const RECORD_COUNT_KEY_MAP: Record<string, string> = {
  individual: 'individuals',
  individuals: 'individuals',
  person: 'individuals',
  persons: 'individuals',
  family: 'families',
  families: 'families',
  event: 'events',
  events: 'events',
  life_event: 'events',
  life_events: 'events',
  place: 'places',
  places: 'places',
  media: 'media',
  media_asset: 'media',
  media_assets: 'media',
  media_file: 'media',
  media_files: 'media',
  note: 'notes',
  notes: 'notes',
};

export function resolveKey(
  t: HistoryTranslateFn | undefined,
  key: string,
  params?: Record<string, string | number>
): string | null {
  if (!t) return null;
  const translated = t(key, params);
  return translated && translated !== key ? translated : null;
}

export function toTitleCase(raw: string): string {
  return raw.replace(/[._-]+/g, ' ').trim().replace(/\b\w/g, (ch) => ch.toUpperCase());
}

export function toSnakeCaseKey(field: string): string {
  return field
    .trim()
    .replace(/([a-z0-9])([A-Z])/g, '$1_$2')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1_$2')
    .replace(/[-\s.]+/g, '_')
    .toLowerCase();
}

export function isRawUuid(val: unknown): boolean {
  return typeof val === 'string' && UUID_REGEX.test(val.trim());
}

export function formatCompactRef(rawId: string): string {
  const trimmed = rawId.trim();
  if (!trimmed) return '';
  return isRawUuid(trimmed) || trimmed.length > 12 ? `#${trimmed.slice(0, 8)}` : `#${trimmed}`;
}

export function extractEntityGid(
  entry: Pick<ChangeLogEntryDto, 'entity_gid' | 'entity_id' | 'entity_label' | 'before_json' | 'after_json'> & {
    before?: Record<string, unknown> | null;
    after?: Record<string, unknown> | null;
  }
): string | null {
  if (typeof entry.entity_gid === 'string' && entry.entity_gid.trim() && !isRawUuid(entry.entity_gid)) {
    return entry.entity_gid.trim();
  }
  if (typeof entry.entity_label === 'string') {
    const gidMatch = entry.entity_label.trim().match(TRAILING_GID_REGEX);
    if (gidMatch?.[1]) return gidMatch[1].toUpperCase();
  }
  const afterObj = (entry.after_json ?? entry.after) as Record<string, unknown> | null | undefined;
  const beforeObj = (entry.before_json ?? entry.before) as Record<string, unknown> | null | undefined;
  const candidateGids = [
    afterObj?.gid,
    beforeObj?.gid,
    (beforeObj?.keep_before as Record<string, unknown> | undefined)?.gid,
    (beforeObj?.person as Record<string, unknown> | undefined)?.gid,
  ];
  for (const cand of candidateGids) {
    if (typeof cand === 'string' && cand.trim() && !isRawUuid(cand)) return cand.trim();
  }
  if (typeof entry.entity_id === 'string' && entry.entity_id.trim() && !isRawUuid(entry.entity_id)) {
    return entry.entity_id.trim();
  }
  return null;
}

export function formatEntityTypeLabel(entityType: string | undefined | null, t: HistoryTranslateFn): string {
  if (!entityType) return t('history.entityTypes.person');
  const norm = entityType.trim().toLowerCase();
  const key = ENTITY_TYPE_ALIAS_MAP[norm] ?? norm;
  return resolveKey(t, `history.entityTypes.${key}`) ?? toTitleCase(entityType);
}

export function formatFieldLabel(field: string, t: HistoryTranslateFn): string {
  const snake = toSnakeCaseKey(field);
  return resolveKey(t, `history.fields.${snake}`) ?? toTitleCase(snake);
}

export function tryTranslateEventPrefix(rawToken: string, t: HistoryTranslateFn): string | null {
  const cleaned = rawToken.trim().toLowerCase();
  if (!cleaned) return null;
  const direct = resolveKey(t, `events.types.${cleaned}`);
  if (direct) return direct;
  const canonical = normalizeEventType(cleaned);
  if (canonical !== 'custom' || EXPLICIT_OTHER_EVENT_TOKENS.has(cleaned)) {
    return resolveKey(t, `domain.events.${canonical}`);
  }
  return null;
}

export function localizeEmbeddedDisplayString(raw: string, t?: HistoryTranslateFn): string {
  if (!t || !raw) return raw;
  const primaryPhotoText = (resolveKey(t, 'history.fields.is_primary_avatar') ?? 'primary avatar').toLowerCase();
  let result = raw.replace(PRIMARY_AVATAR_BRACKET_RE, `[${primaryPhotoText}]`);
  result = result.replace(PEDIGREE_BRACKET_RE, (_, inner: string) => {
    const parts = inner.split('/').map((part) => {
      const norm = normalizePedigreeType(part.trim());
      return resolveKey(t, `domain.pedigree.${norm}`) ?? part.trim();
    });
    return `(${parts.join('/')})`;
  });
  const trimmed = result.trim();
  const noteMatch = trimmed.match(/^Note\s+[—-]\s+(.+)$/i);
  if (noteMatch) {
    const noteLabel = resolveKey(t, 'history.entityTypes.note') ?? 'Note';
    return `${noteLabel} — ${noteMatch[1].trim()}`;
  }
  const sepMatch = trimmed.match(COMPOSITE_SEP_RE);
  if (sepMatch) {
    const translatedEvent = tryTranslateEventPrefix(sepMatch[1], t);
    if (translatedEvent) return `${translatedEvent} — ${sepMatch[2].trim()}`;
  }
  return result;
}

export function formatTxLabel(txLabel: string | undefined | null, t: HistoryTranslateFn): string {
  if (!txLabel) return t('history.txLabels.default');
  const trimmed = txLabel.trim();
  const directKey = TX_LABEL_MAP[trimmed.toLowerCase()];
  if (directKey) return resolveKey(t, directKey) ?? toTitleCase(trimmed);

  const mergedMatch = trimmed.match(/^Merged\s+(.+?)\s+into\s+(.+)$/i);
  if (mergedMatch) {
    return resolveKey(t, 'history.txLabels.mergedInto', { source: mergedMatch[1].trim(), target: mergedMatch[2].trim() }) ?? trimmed;
  }
  const batchMatch = trimmed.match(/^Batch merge\s*\((\d+)\s*groups?\)$/i);
  if (batchMatch) {
    return resolveKey(t, 'history.txLabels.batchMergeGroups', { count: Number(batchMatch[1]) }) ?? trimmed;
  }

  const sepMatch = trimmed.match(COMPOSITE_SEP_RE);
  if (sepMatch) {
    const left = sepMatch[1].trim();
    const right = sepMatch[2].trim();
    const leftKey = TX_LABEL_MAP[left.toLowerCase()];
    if (leftKey || /^[a-z0-9_]+\.[a-z0-9_.]+$/i.test(left)) {
      const translatedPrefix = leftKey ? (resolveKey(t, leftKey) ?? toTitleCase(left)) : toTitleCase(left);
      const rightKey = TX_LABEL_MAP[right.toLowerCase()];
      const formattedRight = rightKey
        ? (resolveKey(t, rightKey) ?? toTitleCase(right))
        : localizeEmbeddedDisplayString(right, t);
      return `${translatedPrefix} — ${formattedRight}`;
    }
  }
  return toTitleCase(trimmed);
}

export function formatUndoneTxRef(val: string, t: HistoryTranslateFn): string {
  const trimmed = val.trim();
  if (!trimmed) return val;
  if (TX_LABEL_MAP[trimmed.toLowerCase()] || COMPOSITE_SEP_RE.test(trimmed)) {
    return formatTxLabel(trimmed, t);
  }
  return formatCompactRef(trimmed);
}

export function formatEntityLabel(
  rawLabel: string | null | undefined,
  entityType: string | undefined | null,
  entityGid: string | null | undefined,
  t: HistoryTranslateFn,
  options: { stripGid?: boolean } = { stripGid: true }
): string {
  if (!rawLabel || !rawLabel.trim()) return formatEntityTypeLabel(entityType, t);
  let label = rawLabel.trim();
  if (options.stripGid !== false && entityGid) {
    const escaped = entityGid.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const stripped = label.replace(new RegExp(`\\s*\\(${escaped}\\)$`, 'i'), '').trim();
    if (stripped) label = stripped;
  }
  if (/^Undo transaction$/i.test(label)) {
    return resolveKey(t, 'history.txLabels.historyUndoTransaction') ?? label;
  }
  if (/^Family Tree$/i.test(label)) {
    return resolveKey(t, 'history.entityTypes.tree') ?? label;
  }
  if (TX_LABEL_MAP[label.toLowerCase()]) {
    return formatTxLabel(label, t);
  }
  const sepMatch = label.match(COMPOSITE_SEP_RE);
  if (sepMatch && TX_LABEL_MAP[sepMatch[1].trim().toLowerCase()]) {
    return formatTxLabel(label, t);
  }
  return localizeEmbeddedDisplayString(label, t);
}

export function formatRecordCounts(
  val: unknown,
  emptyLabel: string,
  t?: HistoryTranslateFn
): string {
  if (val && typeof val === 'object' && !Array.isArray(val)) {
    const parts = Object.entries(val as Record<string, unknown>)
      .map(([k, v]) => {
        const num = typeof v === 'number' ? v : Number(v);
        if (v === null || v === undefined || v === '' || Number.isNaN(num)) return null;
        const mapped = RECORD_COUNT_KEY_MAP[toSnakeCaseKey(k)];
        return (mapped && resolveKey(t, `history.recordCounts.${mapped}`, { count: num })) ?? `${num} ${toTitleCase(k)}`;
      })
      .filter((s): s is string => Boolean(s));
    return parts.length > 0 ? parts.join(', ') : emptyLabel;
  }
  if (typeof val === 'string') {
    const tokens = val.split(',').map((s) => s.trim()).filter(Boolean);
    let matchedAny = false;
    const parts = tokens.map((token) => {
      const m = token.match(/^(\d+)\s+([a-zA-Z_ ]+)$/);
      if (!m) return token;
      const mapped = RECORD_COUNT_KEY_MAP[toSnakeCaseKey(m[2])];
      if (!mapped) return token;
      matchedAny = true;
      return resolveKey(t, `history.recordCounts.${mapped}`, { count: Number(m[1]) }) ?? token;
    });
    return matchedAny ? parts.join(', ') : localizeEmbeddedDisplayString(val, t);
  }
  return emptyLabel;
}
