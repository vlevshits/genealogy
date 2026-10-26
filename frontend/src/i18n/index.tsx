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
    (path: string, params?: TranslateParams): string => {
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
      if (!params) {
        return current;
      }
      return current.replace(/\{(\w+)\}/g, (_, key: string) =>
        key in params ? String(params[key]) : `{${key}}`
      );
    },
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
