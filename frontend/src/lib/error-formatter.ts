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
      if (err.code === 'user_verification_required') {
        return t('errors.user_verification_required');
      }
      if (err.code === 'passkey_verification_failed') {
        return t('errors.passkey_verification_failed');
      }
      const fromCode = tryTranslateErrorCode(err.code, t);
      if (fromCode) return fromCode;
    }

    const mappedKey = STATUS_TO_ERROR_KEY[err.status];
    if (mappedKey) {
      const fromStatus = tryTranslateErrorCode(mappedKey, t);
      if (fromStatus) return fromStatus;
    }
  } else if (
    (typeof DOMException !== 'undefined' && err instanceof DOMException) ||
    (err && typeof err === 'object' && 'name' in err)
  ) {
    const domErr = err as { name?: string };
    if (
      domErr.name === 'NotAllowedError' ||
      domErr.name === 'TimeoutError' ||
      domErr.name === 'AbortError'
    ) {
      return t('errors.webauthn_not_allowed');
    }
    if (domErr.name === 'InvalidStateError') {
      return t('errors.webauthn_already_registered');
    }
    if (domErr.name === 'SecurityError') {
      return t('errors.webauthn_security_error');
    }
    if (domErr.name === 'NotSupportedError') {
      return t('errors.webauthn_not_supported');
    }
  }

  if (err && typeof err === 'object' && !(err instanceof ApiError)) {
    const maybeCode = (err as { code?: unknown }).code;
    if (typeof maybeCode === 'string' && maybeCode) {
      if (maybeCode === 'user_verification_required') {
        return t('errors.user_verification_required');
      }
      if (maybeCode === 'passkey_verification_failed') {
        return t('errors.passkey_verification_failed');
      }
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

/**
 * Localizes backend and parser warning strings for GEDCOM imports and analyses.
 */
export function formatGedcomWarning(rawWarning: string, t: TranslateFn): string {
  if (!rawWarning || typeof rawWarning !== 'string') return '';
  const trimmed = rawWarning.trim();

  // 1. External media paths warning (e.g. "129 external media paths (/root/...) have no bundled binary...")
  const mediaMissingMatch = trimmed.match(
    /^(\d+)\s+external media paths(?:\s+\(([^)]+)\))?\s+have no bundled binary;\s+media asset stubs (?:will be|were) created for post-import reconciliation\.?$/i
  );
  if (mediaMissingMatch) {
    const count = parseInt(mediaMissingMatch[1], 10);
    const sample = mediaMissingMatch[2];
    if (sample) {
      return t('gedcom.warnings.missingMediaStubsWithSample', { count, sample });
    }
    return t('gedcom.warnings.missingMediaStubs', { count });
  }

  // 2. Unresolved ASSO target (e.g. "Unresolved ASSO target @I0012@")
  const assoMatch = trimmed.match(/^Unresolved ASSO target @([^@]+)@\.?$/i);
  if (assoMatch) {
    return t('gedcom.warnings.unresolvedAsso', { xref: assoMatch[1] });
  }

  // 3. Fallback to raw warning string
  return rawWarning;
}
