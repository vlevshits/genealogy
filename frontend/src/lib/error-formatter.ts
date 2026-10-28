import { ApiError } from '../api/client';
import type { TranslateParams } from '../i18n';

export type TranslateFn = (path: string, params?: TranslateParams) => string;

const STATUS_TO_ERROR_KEY: Record<number, string> = {
  0: 'network_error',
  401: 'not_authenticated',
  403: 'forbidden',
  404: 'not_found',
  409: 'version_conflict',
  429: 'rate_limited',
  500: 'internal_error',
};

function tryTranslateErrorCode(code: string, t: TranslateFn): string | null {
  const normalized = code.trim();
  if (!normalized) return null;
  const fullKey = normalized.startsWith('errors.')
    ? normalized
    : `errors.${normalized}`;
  const translated = t(fullKey);
  if (translated && translated !== fullKey) {
    return translated;
  }
  return null;
}

export function getLocalizedErrorMessage(
  err: unknown,
  t: TranslateFn
): string {
  if (err instanceof ApiError) {
    if (err.code) {
      const fromCode = tryTranslateErrorCode(err.code, t);
      if (fromCode) return fromCode;
    }

    const mappedKey = STATUS_TO_ERROR_KEY[err.status];
    if (mappedKey) {
      const fromStatus = tryTranslateErrorCode(mappedKey, t);
      if (fromStatus) return fromStatus;
    }
  } else if (err && typeof err === 'object') {
    const maybeCode = (err as { code?: unknown }).code;
    if (typeof maybeCode === 'string' && maybeCode) {
      const fromCode = tryTranslateErrorCode(maybeCode, t);
      if (fromCode) return fromCode;
    }
    const maybeStatus = (err as { status?: unknown }).status;
    if (typeof maybeStatus === 'number' && maybeStatus in STATUS_TO_ERROR_KEY) {
      const fromStatus = tryTranslateErrorCode(
        STATUS_TO_ERROR_KEY[maybeStatus],
        t
      );
      if (fromStatus) return fromStatus;
    }
  }

  if (err instanceof Error) {
    const msg = err.message || '';
    const lower = msg.toLowerCase();
    if (
      lower.includes('fetch') ||
      lower.includes('network') ||
      lower.includes('load failed')
    ) {
      const netMsg = tryTranslateErrorCode('network_error', t);
      if (netMsg) return netMsg;
    }

    const fromMessageAsCode = tryTranslateErrorCode(msg, t);
    if (fromMessageAsCode) return fromMessageAsCode;
  }

  const unknownTranslated = tryTranslateErrorCode('unknown', t);
  if (unknownTranslated) return unknownTranslated;

  return t('common.errorLoading');
}
