import {
  CalendarSystem,
  GenDate,
  GenDateModifier,
  ParsedGenDateResult,
  PartialDate,
} from '../types/gendate';
import { Locale } from '../types/common';

const EN_MONTHS: Record<string, number> = {
  jan: 1, january: 1,
  feb: 2, february: 2,
  mar: 3, march: 3,
  apr: 4, april: 4,
  may: 5,
  jun: 6, june: 6,
  jul: 7, july: 7,
  aug: 8, august: 8,
  sep: 9, sept: 9, september: 9,
  oct: 10, october: 10,
  nov: 11, november: 11,
  dec: 12, december: 12,
};

const RU_MONTHS: Record<string, number> = {
  янв: 1, январь: 1, января: 1, январе: 1, январем: 1, январём: 1,
  фев: 2, февраль: 2, февраля: 2, феврале: 2, февралем: 2, февралём: 2,
  мар: 3, март: 3, марта: 3, марте: 3, мартом: 3,
  апр: 4, апрель: 4, апреля: 4, апреле: 4, апрелем: 4,
  май: 5, мая: 5, мае: 5, маем: 5,
  июн: 6, июнь: 6, июня: 6, июне: 6, июнем: 6,
  июл: 7, июль: 7, июля: 7, июле: 7, июлем: 7,
  авг: 8, август: 8, августа: 8, августе: 8, августом: 8,
  сен: 9, сент: 9, сентябрь: 9, сентября: 9, сентябре: 9, сентябрем: 9, сентябрём: 9,
  окт: 10, октябрь: 10, октября: 10, октябре: 10, октябрем: 10, октябрём: 10,
  ноя: 11, ноябрь: 11, ноября: 11, ноябре: 11, ноябрем: 11, ноябрём: 11,
  дек: 12, декабрь: 12, декабря: 12, декабре: 12, декабрем: 12, декабрём: 12,
};

const EN_MONTH_NAMES_SHORT = [
  '', 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
];

const RU_MONTH_NAMES_GENITIVE = [
  '', 'января', 'февраля', 'марта', 'апреля', 'мая', 'июня',
  'июля', 'августа', 'сентября', 'октября', 'ноября', 'декабря',
];

const RU_MONTH_NAMES_NOMINATIVE = [
  '', 'январь', 'февраль', 'март', 'апрель', 'май', 'июнь',
  'июль', 'август', 'сентябрь', 'октябрь', 'ноябрь', 'декабрь',
];

export function isLeapYearGregorian(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
}

export function daysInMonthGregorian(year: number, month: number): number {
  switch (month) {
    case 1:
    case 3:
    case 5:
    case 7:
    case 8:
    case 10:
    case 12:
      return 31;
    case 4:
    case 6:
    case 9:
    case 11:
      return 30;
    case 2:
      return isLeapYearGregorian(year) ? 29 : 28;
    default:
      return 30;
  }
}

export function julianOffsetDays(year: number): number {
  if (year >= 1900 && year <= 2099) return 13;
  if (year >= 1800 && year <= 1899) return 12;
  if (year >= 1700 && year <= 1799) return 11;
  if (year >= 1582 && year <= 1699) return 10;
  return 13;
}

export function convertJulianToGregorian(date: PartialDate): PartialDate {
  if (date.m == null || date.d == null) {
    return { ...date };
  }
  const offset = julianOffsetDays(date.y);
  let y = date.y;
  let m = date.m;
  let d = date.d + offset;

  while (true) {
    const dim = daysInMonthGregorian(y, m);
    if (d <= dim) break;
    d -= dim;
    m += 1;
    if (m > 12) {
      m = 1;
      y += 1;
    }
  }
  return { y, m, d };
}

export function computeGenDateSortKey(genDate: GenDate): number | null {
  if (genDate.mod === 'phrase' || !genDate.start) {
    return null;
  }
  const normalizedStart =
    genDate.cal === 'julian' && genDate.start.m != null && genDate.start.d != null
      ? convertJulianToGregorian(genDate.start)
      : genDate.start;

  const base =
    normalizedStart.y * 10000 +
    (normalizedStart.m ?? 0) * 100 +
    (normalizedStart.d ?? 0);

  if (genDate.mod === 'before') {
    return base - 1;
  }
  if (genDate.mod === 'after') {
    return base + 1;
  }
  return base;
}

function parseSingleDateAtom(raw: string): PartialDate | null {
  const cleaned = raw
    .trim()
    .replace(/(?:^|\s)(?:гг?\.?|года|год)\.?(?=\s|$)/gi, ' ')
    .replace(/\s+/g, ' ')
    .trim();

  if (!cleaned) return null;

  // 1. DD.MM.YYYY or DD/MM/YYYY
  const dmyDots = cleaned.match(/^(\d{1,2})[./](\d{1,2})[./](\d{3,4})$/);
  if (dmyDots) {
    const d = parseInt(dmyDots[1], 10);
    const m = parseInt(dmyDots[2], 10);
    const y = parseInt(dmyDots[3], 10);
    if (m >= 1 && m <= 12 && d >= 1 && d <= 31 && y >= 100 && y <= 2999) {
      return { y, m, d };
    }
    return null;
  }

  // 2. MM.YYYY or MM/YYYY
  const myDots = cleaned.match(/^(\d{1,2})[./](\d{3,4})$/);
  if (myDots) {
    const m = parseInt(myDots[1], 10);
    const y = parseInt(myDots[2], 10);
    if (m >= 1 && m <= 12 && y >= 100 && y <= 2999) {
      return { y, m, d: null };
    }
    return null;
  }

  // 3. ISO YYYY-MM-DD
  const isoFull = cleaned.match(/^(\d{3,4})-(\d{1,2})-(\d{1,2})$/);
  if (isoFull) {
    const y = parseInt(isoFull[1], 10);
    const m = parseInt(isoFull[2], 10);
    const d = parseInt(isoFull[3], 10);
    if (m >= 1 && m <= 12 && d >= 1 && d <= 31 && y >= 100 && y <= 2999) {
      return { y, m, d };
    }
    return null;
  }

  // 4. YYYY only
  const yearOnly = cleaned.match(/^(\d{3,4})$/);
  if (yearOnly) {
    const y = parseInt(yearOnly[1], 10);
    if (y >= 100 && y <= 2999) {
      return { y, m: null, d: null };
    }
    return null;
  }

  // 5. DD Month YYYY (EN or RU)
  const dMonthY = cleaned.match(/^(\d{1,2})\s+([a-zA-Zа-яА-ЯёЁ.]+)\s+(\d{3,4})$/);
  if (dMonthY) {
    const d = parseInt(dMonthY[1], 10);
    const monthToken = dMonthY[2].toLowerCase().replace(/\.$/, '');
    const y = parseInt(dMonthY[3], 10);
    const m = EN_MONTHS[monthToken] ?? RU_MONTHS[monthToken];
    if (m && d >= 1 && d <= 31 && y >= 100 && y <= 2999) {
      return { y, m, d };
    }
    return null;
  }

  // 6. Month DD, YYYY (EN style e.g. Jul 14, 1923)
  const monthDY = cleaned.match(/^([a-zA-Zа-яА-ЯёЁ.]+)\s+(\d{1,2}),?\s+(\d{3,4})$/);
  if (monthDY) {
    const monthToken = monthDY[1].toLowerCase().replace(/\.$/, '');
    const d = parseInt(monthDY[2], 10);
    const y = parseInt(monthDY[3], 10);
    const m = EN_MONTHS[monthToken] ?? RU_MONTHS[monthToken];
    if (m && d >= 1 && d <= 31 && y >= 100 && y <= 2999) {
      return { y, m, d };
    }
    return null;
  }

  // 7. Month YYYY (EN or RU)
  const monthY = cleaned.match(/^([a-zA-Zа-яА-ЯёЁ.]+)\s+(\d{3,4})$/);
  if (monthY) {
    const monthToken = monthY[1].toLowerCase().replace(/\.$/, '');
    const y = parseInt(monthY[2], 10);
    const m = EN_MONTHS[monthToken] ?? RU_MONTHS[monthToken];
    if (m && y >= 100 && y <= 2999) {
      return { y, m, d: null };
    }
    return null;
  }

  return null;
}

export function parseGenDate(input: string, _locale: Locale = 'en'): ParsedGenDateResult {
  const originalText = input.trim();
  if (!originalText) {
    const emptyVal: GenDate = {
      mod: 'exact',
      start: null,
      end: null,
      cal: 'gregorian',
      text: '',
    };
    return { value: emptyVal, sortKey: null, isValid: true };
  }

  let working = originalText;
  let cal: CalendarSystem = 'gregorian';

  // Detect Julian calendar markers: (ст. ст.), ст. ст., (o.s.), (julian), старый стиль
  const julianPattern = /\(?\s*(?:ст\.\s*ст\.|старый\s+стиль|o\.?\s*s\.?|julian)\s*\)?/i;
  if (julianPattern.test(working)) {
    cal = 'julian';
    working = working.replace(julianPattern, '').trim();
  }

  // Detect Gregorian explicit marker if present: (нов. ст.), (n.s.)
  const gregorianPattern = /\(?\s*(?:нов\.\s*ст\.|новый\s+стиль|n\.?\s*s\.?|gregorian)\s*\)?/i;
  if (gregorianPattern.test(working)) {
    cal = 'gregorian';
    working = working.replace(gregorianPattern, '').trim();
  }

  // Check range: between X and Y / между X и Y
  const betweenEn = working.match(/^(?:between|bet\.?)\s+(.+?)\s+(?:and|[-–—])\s+(.+)$/i);
  const betweenRu = working.match(/^(?:между)\s+(.+?)\s+(?:и|[-–—])\s+(.+)$/i);
  const betweenMatch = betweenEn || betweenRu;
  if (betweenMatch) {
    const start = parseSingleDateAtom(betweenMatch[1]);
    const end = parseSingleDateAtom(betweenMatch[2]);
    if (start && end) {
      const value: GenDate = {
        mod: 'between',
        start,
        end,
        cal,
        text: originalText,
      };
      return {
        value,
        sortKey: computeGenDateSortKey(value),
        isValid: true,
      };
    }
  }

  // Check from_to: from X to Y / с X по Y
  const fromToEn = working.match(/^from\s+(.+?)\s+to\s+(.+)$/i);
  const fromToRu = working.match(/^(?:с|со)\s+(.+?)\s+по\s+(.+)$/i);
  const fromToMatch = fromToEn || fromToRu;
  if (fromToMatch) {
    const start = parseSingleDateAtom(fromToMatch[1]);
    const end = parseSingleDateAtom(fromToMatch[2]);
    if (start && end) {
      const value: GenDate = {
        mod: 'from_to',
        start,
        end,
        cal,
        text: originalText,
      };
      return {
        value,
        sortKey: computeGenDateSortKey(value),
        isValid: true,
      };
    }
  }

  // Check single modifier prefixes
  let mod: GenDateModifier = 'exact';
  let remainder = working;

  const modRules: Array<{ regex: RegExp; mod: GenDateModifier }> = [
    { regex: /^(?:about|abt\.?|circa|ca\.?|c\.|~|около|ок\.?)\s+(.+)$/i, mod: 'about' },
    { regex: /^(?:estimated|est\.?|приблизительно|прибл\.?|оценочно)\s+(.+)$/i, mod: 'estimated' },
    { regex: /^(?:calculated|cal\.?|вычислено|выч\.?)\s+(.+)$/i, mod: 'calculated' },
    { regex: /^(?:before|bef\.?|<|до|ранее)\s+(.+)$/i, mod: 'before' },
    { regex: /^(?:after|aft\.?|>|после|позже)\s+(.+)$/i, mod: 'after' },
  ];

  for (const rule of modRules) {
    const match = working.match(rule.regex);
    if (match) {
      mod = rule.mod;
      remainder = match[1].trim();
      break;
    }
  }

  const parsedStart = parseSingleDateAtom(remainder);
  if (parsedStart) {
    const value: GenDate = {
      mod,
      start: parsedStart,
      end: null,
      cal,
      text: originalText,
    };
    return {
      value,
      sortKey: computeGenDateSortKey(value),
      isValid: true,
    };
  }

  // Fallback: unparseable input is stored as a phrase date (never blocks saving)
  const phraseValue: GenDate = {
    mod: 'phrase',
    start: null,
    end: null,
    cal: 'gregorian',
    text: originalText,
  };
  return {
    value: phraseValue,
    sortKey: null,
    isValid: true,
  };
}

export function formatPartialDate(
  date: PartialDate | null,
  locale: Locale,
  genitiveMonth = true
): string {
  if (!date) return '';
  const { y, m, d } = date;
  if (m && d) {
    if (locale === 'ru') {
      return `${d} ${RU_MONTH_NAMES_GENITIVE[m]} ${y} г.`;
    }
    return `${d} ${EN_MONTH_NAMES_SHORT[m]} ${y}`;
  }
  if (m && !d) {
    if (locale === 'ru') {
      const mName = genitiveMonth ? RU_MONTH_NAMES_GENITIVE[m] : RU_MONTH_NAMES_NOMINATIVE[m];
      return `${mName} ${y} г.`;
    }
    return `${EN_MONTH_NAMES_SHORT[m]} ${y}`;
  }
  return locale === 'ru' ? `${y} г.` : `${y}`;
}

export function formatGenDate(
  genDate: GenDate | null | undefined,
  locale: Locale,
  options?: { showDualCalendar?: boolean; short?: boolean }
): string {
  if (!genDate) return '';
  if (genDate.mod === 'phrase' || !genDate.start) {
    return genDate.text || '';
  }

  const isRu = locale === 'ru';
  const short = options?.short ?? false;
  const startStr = formatPartialDate(
    genDate.start,
    locale,
    genDate.mod === 'before' || genDate.mod === 'after' || genDate.start.d != null
  );
  const endStr = genDate.end ? formatPartialDate(genDate.end, locale, true) : '';

  let base = '';
  switch (genDate.mod) {
    case 'exact':
      base = startStr;
      break;
    case 'about':
      base = isRu
        ? `${short ? 'ок.' : 'около'} ${startStr}`
        : `${short ? 'abt' : 'about'} ${startStr}`;
      break;
    case 'estimated':
      base = isRu
        ? `${short ? 'прибл.' : 'оценочно'} ${startStr}`
        : `${short ? 'est.' : 'estimated'} ${startStr}`;
      break;
    case 'calculated':
      base = isRu
        ? `${short ? 'выч.' : 'вычислено'} ${startStr}`
        : `${short ? 'cal.' : 'calculated'} ${startStr}`;
      break;
    case 'before':
      base = isRu ? `до ${startStr}` : `${short ? 'bef' : 'before'} ${startStr}`;
      break;
    case 'after':
      base = isRu ? `после ${startStr}` : `${short ? 'aft' : 'after'} ${startStr}`;
      break;
    case 'between':
      base = isRu
        ? `между ${ formatPartialDate(genDate.start, locale, false).replace(/ г\.$/, '') } и ${endStr}`
        : `between ${startStr} and ${endStr}`;
      break;
    case 'from_to':
      base = isRu ? `с ${startStr} по ${endStr}` : `from ${startStr} to ${endStr}`;
      break;
    default:
      base = startStr;
  }

  if (genDate.cal === 'julian') {
    const julianSuffix = isRu ? ' (ст. ст.)' : ' (O.S.)';
    if (options?.showDualCalendar && genDate.start.m && genDate.start.d) {
      const greg = convertJulianToGregorian(genDate.start);
      const gregStr = formatPartialDate(greg, locale, true);
      return `${base}${julianSuffix} / ${gregStr}`;
    }
    return `${base}${julianSuffix}`;
  }

  return base;
}
