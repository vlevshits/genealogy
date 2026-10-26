import { describe, it, expect } from 'vitest';
import en from '../../src/i18n/en.json';
import ru from '../../src/i18n/ru.json';
import { formatKinshipLabel } from '../../src/lib/kinship';
import { matchesCrossScript } from '../../src/lib/utils';

function collectKeys(obj: Record<string, unknown>, prefix = ''): string[] {
  const keys: string[] = [];
  for (const [k, v] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === 'object' && !Array.isArray(v)) {
      keys.push(...collectKeys(v as Record<string, unknown>, path));
    } else {
      keys.push(path);
    }
  }
  return keys.sort();
}

describe('i18n Key Parity & Localization Helpers', () => {
  it('maintains 100% key parity between en.json and ru.json', () => {
    const enKeys = collectKeys(en as Record<string, unknown>);
    const ruKeys = collectKeys(ru as Record<string, unknown>);
    expect(enKeys).toEqual(ruKeys);
    expect(enKeys.length).toBeGreaterThan(100);
  });

  it('has non-empty string values for every key in both en.json and ru.json', () => {
    const checkValues = (obj: Record<string, unknown>, locale: string) => {
      for (const [k, v] of Object.entries(obj)) {
        if (v && typeof v === 'object') {
          checkValues(v as Record<string, unknown>, locale);
        } else {
          expect(typeof v, `${locale}.${k} should be a string`).toBe('string');
          expect((v as string).trim().length, `${locale}.${k} should not be empty`).toBeGreaterThan(0);
        }
      }
    };
    checkValues(en as Record<string, unknown>, 'en');
    checkValues(ru as Record<string, unknown>, 'ru');
  });

  it('formats kinship terms with gender agreement in Russian and English', () => {
    expect(formatKinshipLabel('first_cousin', 'male', 'ru')).toBe('Двоюродный брат');
    expect(formatKinshipLabel('first_cousin', 'female', 'ru')).toBe('Двоюродная сестра');
    expect(formatKinshipLabel('first_cousin', 'female', 'en')).toBe('First Cousin');
    expect(formatKinshipLabel('second_cousin', 'female', 'ru')).toBe('Троюродная сестра');
    expect(formatKinshipLabel('parent_in_law_husband', 'female', 'ru')).toBe('Свекровь');
    expect(formatKinshipLabel('parent_in_law_wife', 'female', 'ru')).toBe('Тёща');
  });

  it('matches cross-script Cyrillic and Latin queries for PersonPicker', () => {
    expect(matchesCrossScript('Левшиц Александр Петрович', 'levshits')).toBe(true);
    expect(matchesCrossScript('Кузнецова Елена Александровна', 'kuznetsova elena')).toBe(true);
    expect(matchesCrossScript('Щеглов Пётр Фёдорович', 'shcheglov')).toBe(true);
  });

  it('contains zero forbidden marketing buzzwords or developer tech-stack names in en.json and ru.json', () => {
    const forbiddenTerms = [
      'Next-Gen',
      'Zero-Cloud',
      'Платформа нового поколения',
      'Платформа семейного архива нового поколения',
      'Нулевая зависимость от облака',
      'Полная автономность без облаков',
      'Axum',
      'React 19',
      'Tailwind',
      'PostgreSQL',
      'pg_trgm',
      'Swagger',
    ];

    const enRaw = JSON.stringify(en);
    const ruRaw = JSON.stringify(ru);

    for (const term of forbiddenTerms) {
      expect(enRaw, `en.json should not contain forbidden term "${term}"`).not.toContain(term);
      expect(ruRaw, `ru.json should not contain forbidden term "${term}"`).not.toContain(term);
    }
  });

  it('forbids leading icon symbols (+, arrows, checkmarks) or decorative emojis in en.json and ru.json that cause duplicated UI icons', () => {
    const leadingIconRegex = /^\s*[+＋←→↑↓↔✓✔✕✖✗✘]\s+/;
    const emojiRegex = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/u;

    const checkNoIconPrefixes = (
      obj: Record<string, unknown>,
      locale: string,
      prefix = ''
    ) => {
      for (const [k, v] of Object.entries(obj)) {
        const keyPath = prefix ? `${prefix}.${k}` : k;
        if (v && typeof v === 'object') {
          checkNoIconPrefixes(v as Record<string, unknown>, locale, keyPath);
        } else if (typeof v === 'string') {
          expect(
            leadingIconRegex.test(v),
            `${locale}.${keyPath} ("${v}") must not start with a leading icon symbol (+, arrow, checkmark); render icons via leftIcon/rightIcon instead`
          ).toBe(false);
          expect(
            emojiRegex.test(v),
            `${locale}.${keyPath} ("${v}") must not contain decorative UI emojis`
          ).toBe(false);
        }
      }
    };

    checkNoIconPrefixes(en as Record<string, unknown>, 'en');
    checkNoIconPrefixes(ru as Record<string, unknown>, 'ru');
  });
});
