import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import en from './en.json';
import ru from './ru.json';
import { Locale } from '../types/common';

const translations: Record<Locale, Record<string, unknown>> = { en, ru };

export type TranslateParams = Record<string, string | number>;

export interface I18nContextType {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (path: string, params?: TranslateParams) => string;
}

function getPluralCategory(locale: Locale, count: number): 'one' | 'few' | 'many' | 'other' {
  const abs = Math.abs(count);
  if (locale === 'ru') {
    const mod10 = abs % 10;
    const mod100 = abs % 100;
    if (mod10 === 1 && mod100 !== 11) return 'one';
    if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return 'few';
    return 'many';
  }
  return abs === 1 ? 'one' : 'other';
}

function formatIcuPlural(
  locale: Locale,
  template: string,
  params?: TranslateParams
): string {
  if (!params) return template;

  // Handle ICU plural blocks: {varName, plural, one {...} few {...} many {...} other {...}}
  const pluralRegex = /\{(\w+),\s*plural,\s*((?:[a-z]+\s*\{[^}]*\}\s*)+)\}/g;
  const resolved = template.replace(pluralRegex, (_, varName: string, body: string) => {
    const rawVal = params[varName];
    const numVal = typeof rawVal === 'number' ? rawVal : Number(rawVal ?? 0);
    const category = getPluralCategory(locale, numVal);

    const branchRegex = /([a-z]+)\s*\{([^}]*)\}/g;
    const branches: Record<string, string> = {};
    let match: RegExpExecArray | null = branchRegex.exec(body);
    while (match) {
      branches[match[1]] = match[2];
      match = branchRegex.exec(body);
    }

    const chosen = branches[category] ?? branches.other ?? branches.many ?? '';
    return chosen.replace(/#/g, String(numVal));
  });

  return resolved.replace(/\{(\w+)\}/g, (_, key: string) =>
    key in params ? String(params[key]) : `{${key}}`
  );
}

export function translate(
  locale: Locale,
  path: string,
  params?: TranslateParams
): string {
  const keys = path.split('.');
  let current: unknown = translations[locale];
  for (const key of keys) {
    if (current && typeof current === 'object' && key in (current as Record<string, unknown>)) {
      current = (current as Record<string, unknown>)[key];
    } else {
      return path;
    }
  }
  if (typeof current !== 'string') {
    return path;
  }
  return formatIcuPlural(locale, current, params);
}

const I18nContext = createContext<I18nContextType | null>(null);

export const I18nProvider: React.FC<{ children: React.ReactNode; initialLocale?: Locale }> = ({
  children,
  initialLocale,
}) => {
  const [locale, setLocaleState] = useState<Locale>(() => {
    if (initialLocale) return initialLocale;
    if (typeof window === 'undefined') return 'en';
    const saved = localStorage.getItem('genealogy_locale');
    if (saved === 'en' || saved === 'ru') return saved;
    return typeof navigator !== 'undefined' && navigator.language?.startsWith('ru') ? 'ru' : 'en';
  });

  const setLocale = useCallback((newLocale: Locale) => {
    setLocaleState(newLocale);
    if (typeof window !== 'undefined') {
      localStorage.setItem('genealogy_locale', newLocale);
      document.documentElement.lang = newLocale;
    }
  }, []);

  useEffect(() => {
    if (typeof document !== 'undefined') {
      document.documentElement.lang = locale;
    }
  }, [locale]);

  const t = useCallback(
    (path: string, params?: TranslateParams): string => translate(locale, path, params),
    [locale]
  );

  return (
    <I18nContext.Provider value={{ locale, setLocale, t }}>
      {children}
    </I18nContext.Provider>
  );
};

export const useI18n = (): I18nContextType => {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error('useI18n must be used within an I18nProvider');
  }
  return context;
};

export const useTranslation = useI18n;
