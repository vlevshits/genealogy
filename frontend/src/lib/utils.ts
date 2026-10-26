import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

const CYR_TO_LAT: Record<string, string> = {
  а: 'a', б: 'b', в: 'v', г: 'g', д: 'd', е: 'e', ё: 'yo', ж: 'zh',
  з: 'z', и: 'i', й: 'y', к: 'k', л: 'l', м: 'm', н: 'n', о: 'o',
  п: 'p', р: 'r', с: 's', т: 't', у: 'u', ф: 'f', х: 'kh', ц: 'ts',
  ч: 'ch', ш: 'sh', щ: 'shch', ъ: '', ы: 'y', ь: '', э: 'e', ю: 'yu', я: 'ya',
};

export function transliterateToLatin(input: string): string {
  return input
    .toLowerCase()
    .split('')
    .map((ch) => (ch in CYR_TO_LAT ? CYR_TO_LAT[ch] : ch))
    .join('');
}

export function normalizeForSearch(input: string): string {
  const lower = input.toLowerCase().trim().replace(/ё/g, 'е');
  const latin = transliterateToLatin(lower);
  return `${lower} ${latin}`;
}

export function matchesCrossScript(target: string, query: string): boolean {
  if (!query.trim()) return true;
  const normTarget = normalizeForSearch(target);
  const normQueryTokens = transliterateToLatin(query.toLowerCase().trim().replace(/ё/g, 'е'))
    .split(/\s+/)
    .filter(Boolean);
  return normQueryTokens.every((token) => normTarget.includes(token));
}
