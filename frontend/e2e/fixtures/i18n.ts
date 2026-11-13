import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export type Locale = 'en' | 'ru';

type Dict = { [key: string]: string | Dict };

const cache = new Map<Locale, Dict>();

function load(locale: Locale): Dict {
  let dict = cache.get(locale);
  if (!dict) {
    const file = fileURLToPath(new URL(`../../src/i18n/${locale}.json`, import.meta.url));
    dict = JSON.parse(readFileSync(file, 'utf-8')) as Dict;
    cache.set(locale, dict);
  }
  return dict;
}

/**
 * Resolves a dotted translation key from the real `en.json` / `ru.json`
 * bundles (single source of truth, so copy edits do not break the E2E suite)
 * and interpolates `{param}` placeholders.
 */
export function tr(locale: Locale, key: string, params: Record<string, string> = {}): string {
  let node: string | Dict | undefined = load(locale);
  for (const part of key.split('.')) {
    node = typeof node === 'object' ? node[part] : undefined;
  }
  if (typeof node !== 'string') {
    throw new Error(`Missing i18n key "${key}" for locale "${locale}"`);
  }
  return node.replace(/\{(\w+)\}/g, (_m, name: string) => params[name] ?? `{${name}}`);
}

/** Case-sensitive exact-name matcher usable in `getByRole(..., { name })`. */
export function exact(text: string): RegExp {
  return new RegExp(`^${text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}$`);
}
