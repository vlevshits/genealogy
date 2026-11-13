import { useCallback } from 'react';
import {
  deleteFamily,
  ProfileSpouseFamilyDto,
  updateFamily,
} from '../api/client';
import { useToast } from '../components/ui';
import { getLocalizedErrorMessage } from '../lib/error-formatter';
import { useI18n } from '../i18n';

/**
 * Detach a spouse or a child from a family union WITHOUT deleting the person
 * profile, so the existing profile can be re-attached elsewhere. A union left
 * with no spouse and no children is removed (with its marriage event).
 */
export function useFamilyLinkActions(personId: string, onChanged: () => void) {
  const { t } = useI18n();
  const { addToast } = useToast();

  const run = useCallback(
    async (action: () => Promise<unknown>, successKey: string) => {
      try {
        await action();
        addToast({ type: 'success', title: t(successKey) });
        onChanged();
      } catch (err) {
        addToast({
          type: 'error',
          title: t('common.errorLoading'),
          description: getLocalizedErrorMessage(err, t),
        });
        onChanged();
      }
    },
    [addToast, onChanged, t]
  );

  const removeSpouse = useCallback(
    (sf: ProfileSpouseFamilyDto) =>
      run(async () => {
        if (sf.children.length === 0) {
          await deleteFamily(sf.family_id);
          return;
        }
        // Keep the profile owner as partner1 and clear the other slot.
        await updateFamily(sf.family_id, {
          version: sf.version ?? 1,
          partner1_id: personId,
          partner2_id: '',
        });
      }, 'profile.family.spouseRemovedToast'),
    [personId, run]
  );

  const removeChild = useCallback(
    (sf: ProfileSpouseFamilyDto, childId: string) =>
      run(async () => {
        const remaining = sf.children
          .filter((ch) => (ch.person ?? ch.relative)?.id !== childId)
          .map((ch, index) => ({
            child_id: (ch.person ?? ch.relative)!.id,
            sort_order: index,
            rel_to_partner1: ch.rel_to_partner1 || 'birth',
            rel_to_partner2: ch.rel_to_partner2 || 'birth',
          }));
        if (remaining.length === 0 && !sf.spouse) {
          await deleteFamily(sf.family_id);
          return;
        }
        await updateFamily(sf.family_id, {
          version: sf.version ?? 1,
          children: remaining,
        });
      }, 'profile.family.childRemovedToast'),
    [run]
  );

  return { removeSpouse, removeChild };
}

