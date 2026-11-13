import { useCallback } from 'react';
import {
  detachFamilyChild,
  detachFamilyPartner,
  moveToFamily as moveToFamilyRequest,
  MoveToFamilyInput,
  MoveToFamilyResponse,
  ProfileParentFamilyDto,
  ProfileSpouseFamilyDto,
} from '../api/client';
import { useToast } from '../components/ui';
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';
import { useUndoToast } from './useUndoToast';

interface LinkActionResult {
  txId?: string | null;
  /** Optional override of the success toast i18n key. */
  successKey?: string;
}

/**
 * Detach a spouse or a child from a family union (or move a person to another
 * family) WITHOUT deleting the person profile. Every successful change shows an
 * undo toast; undoing refreshes the profile through `onChanged`.
 */
export function useFamilyLinkActions(personId: string, onChanged: () => void) {
  const { t } = useI18n();
  const { addToast } = useToast();
  const { showUndoToast } = useUndoToast();

  const run = useCallback(
    async (
      action: () => Promise<LinkActionResult | void>,
      successKey: string
    ): Promise<boolean> => {
      try {
        const result = await action();
        const txId = result?.txId ?? null;
        const title = t(result?.successKey ?? successKey);
        if (txId) {
          showUndoToast({ txId, title, onUndone: onChanged });
        } else {
          addToast({ type: 'success', title });
        }
        onChanged();
        return true;
      } catch (err) {
        addToast({
          type: 'error',
          title: t('common.errorLoading'),
          description: getLocalizedErrorMessage(err, t),
        });
        onChanged();
        return false;
      }
    },
    [addToast, onChanged, showUndoToast, t]
  );

  const removeSpouse = useCallback(
    (sf: ProfileSpouseFamilyDto) =>
      run(async () => {
        if (!sf.spouse) return;
        const res = await detachFamilyPartner(sf.family_id, sf.spouse.id, sf.version);
        return {
          txId: res.tx_id,
          successKey: res.dissolved
            ? 'profile.family.spouseRemovedDissolvedToast'
            : undefined,
        };
      }, 'profile.family.spouseRemovedToast'),
    [run]
  );

  const removeChild = useCallback(
    (sf: ProfileSpouseFamilyDto, childId: string) =>
      run(async () => {
        const res = await detachFamilyChild(sf.family_id, childId, sf.version);
        return {
          txId: res.tx_id,
          successKey: res.dissolved
            ? 'profile.family.childRemovedDissolvedToast'
            : undefined,
        };
      }, 'profile.family.childRemovedToast'),
    [run]
  );

  /** Detach the profile person from both parents (whole parent family). */
  const unlinkParent = useCallback(
    (pf: ProfileParentFamilyDto) =>
      run(async () => {
        const res = await detachFamilyChild(pf.family_id, personId, pf.version);
        return {
          txId: res.tx_id,
          successKey: res.dissolved
            ? 'profile.family.parentUnlinkedDissolvedToast'
            : undefined,
        };
      }, 'profile.family.parentUnlinkedToast'),
    [personId, run]
  );

  /** Unlink one parent only: the child stays with the other parent. */
  const unlinkParentOnly = useCallback(
    (pf: ProfileParentFamilyDto, otherParentId: string) =>
      run(async () => {
        const res = await moveToFamilyRequest(personId, {
          partner1_id: otherParentId,
          from_family_id: pf.family_id,
          rel_to_partner1: pf.child_relation || 'birth',
        });
        return {
          txId: res.tx_id,
          successKey: res.source_family_dissolved
            ? 'profile.family.parentUnlinkedOnlyDissolvedToast'
            : undefined,
        };
      }, 'profile.family.parentUnlinkedOnlyToast'),
    [personId, run]
  );

  /**
   * Move the profile person to another parent family. Errors are re-thrown so
   * the dialog can map `field_errors` codes to inline messages.
   */
  const moveToFamily = useCallback(
    async (payload: MoveToFamilyInput): Promise<void> => {
      const res: MoveToFamilyResponse = await moveToFamilyRequest(personId, payload);
      showUndoToast({
        txId: res.tx_id,
        title: t('profile.family.moveSuccessToast'),
        onUndone: onChanged,
      });
      onChanged();
    },
    [onChanged, personId, showUndoToast, t]
  );

  return { removeSpouse, removeChild, unlinkParent, unlinkParentOnly, moveToFamily };
}
