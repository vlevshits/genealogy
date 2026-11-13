import { ApiError } from '../api/client';
import { getLocalizedErrorMessage, TranslateFn } from './error-formatter';

/** Validation codes returned by `POST /persons/{id}/move-to-family`. */
export const MOVE_TO_FAMILY_ERROR_CODES = [
  'target_required',
  'duplicate_partner',
  'not_a_child_of_family',
  'ambiguous_source_family',
  'same_family',
  'already_child_of_family',
] as const;

export type MoveToFamilyErrorCode = (typeof MOVE_TO_FAMILY_ERROR_CODES)[number];

function isMoveCode(value: string): value is MoveToFamilyErrorCode {
  return (MOVE_TO_FAMILY_ERROR_CODES as readonly string[]).includes(value);
}

/**
 * Maps a move-to-family failure to a localized message. Field error codes
 * (`field_errors` values, or keys as a fallback) take precedence over the
 * generic envelope code (`validation_failed`, `cycle_detected`, ...).
 */
export function getMoveToFamilyErrorMessage(err: unknown, t: TranslateFn): string {
  if (err instanceof ApiError && err.field_errors) {
    for (const [field, code] of Object.entries(err.field_errors)) {
      const found = [code, field].find(
        (c): c is MoveToFamilyErrorCode => typeof c === 'string' && isMoveCode(c)
      );
      if (found) return t(`profile.family.moveErrors.${found}`);
    }
  }
  if (err instanceof ApiError && err.status === 404 && err.code !== 'person_not_found') {
    return t('profile.family.moveErrors.family_not_found');
  }
  return getLocalizedErrorMessage(err, t);
}
