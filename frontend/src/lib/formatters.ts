import { Locale } from '../types/common';

export interface PersonNameInput {
  given_name?: string | null;
  givenName?: string | null;
  patronymic?: string | null;
  surname?: string | null;
  birth_surname?: string | null;
  birthSurname?: string | null;
  maiden_name?: string | null;
  maidenName?: string | null;
  name_prefix?: string | null;
  namePrefix?: string | null;
  name_suffix?: string | null;
  nameSuffix?: string | null;
  primary_name?: string | null;
  primaryName?: string | null;
}

export type PersonNameFormatMode = 'full' | 'compact' | 'directory' | 'card';

export interface FormatPersonNameOptions {
  format?: PersonNameFormatMode;
  locale?: Locale;
  showMaiden?: boolean;
  maidenStyle?: 'nee' | 'parens';
}

/**
 * Formats a person's name according to locale-specific genealogical rules:
 * - Russian (`ru`):
 *   - `full` / `directory`: `Фамилия [(Девичья)] Имя Отчество`
 *     (e.g., `Кузнецов Александр Иванович`, `Левшиц (Морозова) Вера Ивановна`)
 *   - `card`: `Имя Отчество Фамилия [(Девичья)]`
 *   - `compact`: `Имя Фамилия`
 * - English (`en`):
 *   - `full` / `card`: `Given [Patronymic] Surname [(née Maiden)]`
 *     (e.g., `Aleksandr Ivanovich Kuznetsov`, `Vera Ivanovna Levshits (née Morozova)`)
 *   - `directory`: `Surname [(née Maiden)], Given [Patronymic]`
 *   - `compact`: `Given Surname`
 */
export function formatPersonName(
  person: PersonNameInput | null | undefined,
  options: FormatPersonNameOptions = {}
): string {
  if (!person) return '';

  const {
    format = 'full',
    locale = 'en',
    showMaiden = true,
    maidenStyle = 'nee',
  } = options;

  const given = (person.given_name ?? person.givenName ?? '').trim();
  const patronymic = (person.patronymic ?? '').trim();
  const surname = (person.surname ?? '').trim();
  const rawMaiden = (
    person.birth_surname ??
    person.birthSurname ??
    person.maiden_name ??
    person.maidenName ??
    ''
  ).trim();
  const prefix = (person.name_prefix ?? person.namePrefix ?? '').trim();
  const suffix = (person.name_suffix ?? person.nameSuffix ?? '').trim();

  const hasDistinctMaiden =
    showMaiden &&
    Boolean(rawMaiden) &&
    rawMaiden.toLowerCase() !== surname.toLowerCase();

  // If no structured components exist, fall back to primary_name
  if (!given && !patronymic && !surname && !rawMaiden) {
    return (person.primary_name ?? person.primaryName ?? '').trim();
  }

  if (locale === 'ru') {
    const maidenPart = hasDistinctMaiden ? `(${rawMaiden})` : '';
    const surnameWithMaiden = [surname, maidenPart].filter(Boolean).join(' ');

    if (format === 'compact') {
      return [given, surname || rawMaiden].filter(Boolean).join(' ').trim();
    }

    if (format === 'card') {
      return [prefix, given, patronymic, surnameWithMaiden, suffix]
        .filter(Boolean)
        .join(' ')
        .trim();
    }

    // 'full' or 'directory': Фамилия [(Девичья)] Имя Отчество
    return [prefix, surnameWithMaiden, given, patronymic, suffix]
      .filter(Boolean)
      .join(' ')
      .trim();
  }

  // English ('en')
  const maidenEnPart = hasDistinctMaiden
    ? maidenStyle === 'parens'
      ? `(${rawMaiden})`
      : `(née ${rawMaiden})`
    : '';

  if (format === 'compact') {
    return [given, surname || rawMaiden].filter(Boolean).join(' ').trim();
  }

  if (format === 'directory') {
    const leadSurname = [surname, maidenEnPart].filter(Boolean).join(' ');
    const givenAndPatronymic = [given, patronymic].filter(Boolean).join(' ');
    if (leadSurname && givenAndPatronymic) {
      return `${leadSurname}, ${givenAndPatronymic}`;
    }
    return [leadSurname, givenAndPatronymic].filter(Boolean).join(' ').trim();
  }

  // 'full' or 'card': Given [Patronymic] Surname [(née Maiden)]
  return [prefix, given, patronymic, surname, maidenEnPart, suffix]
    .filter(Boolean)
    .join(' ')
    .trim();
}

export type VitalEventType =
  | 'birth'
  | 'death'
  | 'baptism'
  | 'burial'
  | 'marriage'
  | 'divorce'
  | string;

export type SexInput =
  | 'M'
  | 'F'
  | 'U'
  | 'male'
  | 'female'
  | 'unknown'
  | string
  | null
  | undefined;

export interface FormatVitalEventOptions {
  locale?: Locale;
  compact?: boolean;
}

function normalizeSexCode(sex: SexInput): 'M' | 'F' | 'U' {
  if (!sex) return 'U';
  const upper = sex.trim().toUpperCase();
  if (upper === 'M' || upper === 'MALE') return 'M';
  if (upper === 'F' || upper === 'FEMALE') return 'F';
  return 'U';
}

/**
 * Returns the grammatically agreed vital event label in Russian (`родился` / `родилась`,
 * `умер` / `умерла`, `крещён` / `крещена`, `похоронен` / `похоронена`) or English.
 */
export function formatVitalEventLabel(
  eventType: VitalEventType,
  sex: SexInput = 'U',
  options: FormatVitalEventOptions = {}
): string {
  const { locale = 'en', compact = false } = options;
  const normalizedType = eventType.trim().toLowerCase();
  const sexCode = normalizeSexCode(sex);

  if (locale === 'ru') {
    switch (normalizedType) {
      case 'birth':
      case 'birt':
        if (compact || sexCode === 'U') return 'род.';
        return sexCode === 'F' ? 'родилась' : 'родился';
      case 'death':
      case 'deat':
        if (compact || sexCode === 'U') return 'ум.';
        return sexCode === 'F' ? 'умерла' : 'умер';
      case 'baptism':
      case 'bapt':
      case 'chr':
        if (compact || sexCode === 'U') return 'крещ.';
        return sexCode === 'F' ? 'крещена' : 'крещён';
      case 'burial':
      case 'buri':
        if (compact || sexCode === 'U') return 'похор.';
        return sexCode === 'F' ? 'похоронена' : 'похоронен';
      case 'marriage':
      case 'marr':
        return 'брак';
      case 'divorce':
      case 'div':
        return 'развод';
      default:
        return eventType;
    }
  }

  // English ('en')
  switch (normalizedType) {
    case 'birth':
    case 'birt':
      return compact ? 'b.' : 'born';
    case 'death':
    case 'deat':
      return compact ? '†' : 'died';
    case 'baptism':
    case 'bapt':
    case 'chr':
      return compact ? 'bapt.' : 'baptized';
    case 'burial':
    case 'buri':
      return compact ? 'bur.' : 'buried';
    case 'marriage':
    case 'marr':
      return compact ? 'm.' : 'married';
    case 'divorce':
    case 'div':
      return compact ? 'div.' : 'divorced';
    default:
      return eventType;
  }
}

/**
 * Formats a deceased indicator or lifespan with the genealogical dagger `†` for deceased individuals
 * and zero "Living" badges/text for living individuals.
 */
export function formatLifeSpanText(params: {
  birthYear?: number | string | null;
  deathYear?: number | string | null;
  isLiving?: boolean;
  isRedacted?: boolean;
  locale?: Locale;
}): string {
  const {
    birthYear,
    deathYear,
    isLiving = false,
    isRedacted = false,
    locale = 'en',
  } = params;

  if (isRedacted) {
    return '';
  }

  const bStr = birthYear !== null && birthYear !== undefined && String(birthYear).trim() !== ''
    ? String(birthYear).trim()
    : '';
  const dStr = deathYear !== null && deathYear !== undefined && String(deathYear).trim() !== ''
    ? String(deathYear).trim()
    : '';

  const bornPrefix = locale === 'ru' ? 'р.' : 'b.';

  if (isLiving && !dStr) {
    return bStr ? `${bornPrefix} ${bStr}` : '';
  }

  if (bStr && dStr) {
    return `${bStr} – †${dStr}`;
  }

  if (bStr && !dStr) {
    return `${bornPrefix} ${bStr}`;
  }

  if (!bStr && dStr) {
    return `† ${dStr}`;
  }

  return '';
}
