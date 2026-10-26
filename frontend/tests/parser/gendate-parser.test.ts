import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';
import {
  parseGenDate,
  convertJulianToGregorian,
  formatGenDate,
} from '../../src/lib/gendate-parser';
import { GenDate } from '../../src/types/gendate';
import { Locale } from '../../src/types/common';

interface FixtureCase {
  input: string;
  locale: Locale;
  expected: GenDate;
  sortKey: number | null;
  isValid: boolean;
}

describe('GenDate Bilingual Parser & SortKey Calculator', () => {
  const fixturesPath = path.resolve(__dirname, '../../../fixtures/gendate_cases.json');
  const cases: FixtureCase[] = JSON.parse(fs.readFileSync(fixturesPath, 'utf-8'));

  it('has all 15 canonical cases in fixtures/gendate_cases.json', () => {
    expect(cases).toHaveLength(15);
  });

  for (const tc of cases) {
    it(`parses "${tc.input}" (${tc.locale}) into expected GenDate and sortKey ${tc.sortKey}`, () => {
      const result = parseGenDate(tc.input, tc.locale);
      expect(result.value).toEqual(tc.expected);
      expect(result.sortKey).toBe(tc.sortKey);
      expect(result.isValid).toBe(tc.isValid);
    });
  }

  it('converts Julian dates to Gregorian with month/year rollover', () => {
    // 25 Dec 1905 (Julian) + 13 days = 7 Jan 1906 (Gregorian)
    const converted = convertJulianToGregorian({ y: 1905, m: 12, d: 25 });
    expect(converted).toEqual({ y: 1906, m: 1, d: 7 });

    // 14 Jul 1905 (Julian) + 13 days = 27 Jul 1905 (Gregorian)
    const convertedJul = convertJulianToGregorian({ y: 1905, m: 7, d: 14 });
    expect(convertedJul).toEqual({ y: 1905, m: 7, d: 27 });
  });

  it('formats GenDate in both English and Russian including dual calendar', () => {
    const julianRes = parseGenDate('14.07.1905 (ст. ст.)', 'ru');
    expect(formatGenDate(julianRes.value, 'ru', { showDualCalendar: true })).toContain(
      '27 июля 1905 г.'
    );
    expect(formatGenDate(julianRes.value, 'en', { showDualCalendar: true })).toContain(
      '27 Jul 1905'
    );
  });
});
