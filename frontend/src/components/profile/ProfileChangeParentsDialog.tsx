import React, { useState } from 'react';
import { Alert, Button, Combobox, ComboboxOption, Dialog, SegmentedControl, Select, TextInput } from '../ui';
import { FormField } from '../form';
import {
  fetchPeopleDirectory,
  MoveToFamilyInput,
  ProfileParentFamilyDto,
} from '../../api/client';
import { getMoveToFamilyErrorMessage } from '../../lib/family-move-errors';
import { useI18n } from '../../i18n';

type TargetMode = 'partners' | 'family';

const PEDIGREE_VALUES = ['birth', 'adopted', 'foster', 'step'] as const;

export interface ProfileChangeParentsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  personId: string;
  parentFamilies: ProfileParentFamilyDto[];
  /** Performs the move; rejects with the API error on failure. */
  onSubmit: (payload: MoveToFamilyInput) => Promise<void>;
}

export const ProfileChangeParentsDialog: React.FC<ProfileChangeParentsDialogProps> = ({
  open,
  onOpenChange,
  personId,
  parentFamilies,
  onSubmit,
}) => {
  const { t } = useI18n();
  const [mode, setMode] = useState<TargetMode>('partners');
  const [partner1, setPartner1] = useState<string | null>(null);
  const [partner2, setPartner2] = useState<string | null>(null);
  const [familyRef, setFamilyRef] = useState('');
  const [fromFamily, setFromFamily] = useState('');
  const [rel1, setRel1] = useState('birth');
  const [rel2, setRel2] = useState('birth');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const needsSource = parentFamilies.length > 1;
  const pedigreeOptions = PEDIGREE_VALUES.map((v) => ({
    value: v,
    label: t(`domain.pedigree.${v}`),
  }));

  const loadPeople = async (
    query: string,
    signal: AbortSignal
  ): Promise<ComboboxOption[]> => {
    const res = await fetchPeopleDirectory({ q: query, limit: 20 });
    if (signal.aborted) return [];
    return res.items
      .filter((p) => p.id !== personId)
      .map((p) => ({
        value: p.id,
        label: `${p.display_name} (${p.gid})`,
        description: p.birth_year ? String(p.birth_year) : undefined,
      }));
  };

  const hasTarget =
    mode === 'family' ? familyRef.trim() !== '' : Boolean(partner1 || partner2);
  const canSubmit = hasTarget && (!needsSource || fromFamily !== '') && !submitting;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit) return;
    const payload: MoveToFamilyInput = {
      rel_to_partner1: rel1,
      rel_to_partner2: rel2,
    };
    if (mode === 'family') {
      payload.family_id = familyRef.trim();
    } else {
      if (partner1) payload.partner1_id = partner1;
      if (partner2) payload.partner2_id = partner2;
    }
    if (needsSource) payload.from_family_id = fromFamily;
    setSubmitting(true);
    setError(null);
    try {
      await onSubmit(payload);
      onOpenChange(false);
    } catch (err) {
      setError(getMoveToFamilyErrorMessage(err, t));
    } finally {
      setSubmitting(false);
    }
  };

  const showRel2 = mode === 'family' || Boolean(partner2);

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={t('profile.family.changeParentsTitle')}
      description={t('profile.family.changeParentsDesc')}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <SegmentedControl<TargetMode>
          ariaLabel={t('profile.family.changeParentsTitle')}
          value={mode}
          onChange={setMode}
          options={[
            { value: 'partners', label: t('profile.family.modePartners') },
            { value: 'family', label: t('profile.family.modeFamily') },
          ]}
        />

        {error && <Alert variant="danger">{error}</Alert>}

        {needsSource && (
          <FormField name="from_family_id" label={t('profile.family.fromFamilyLabel')} required>
            <Select
              ariaLabel={t('profile.family.fromFamilyLabel')}
              value={fromFamily || undefined}
              placeholder={t('profile.family.fromFamilyPlaceholder')}
              onChange={setFromFamily}
              options={parentFamilies.map((pf) => ({
                value: pf.family_id,
                label: pf.gid,
              }))}
            />
          </FormField>
        )}

        {mode === 'partners' ? (
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <FormField name="partner1_id" label={t('profile.family.parent1Label')} required>
              <Combobox
                ariaLabel={t('profile.family.parent1Label')}
                value={partner1}
                loadOptions={loadPeople}
                placeholder={t('profile.family.parentPlaceholder')}
                onChange={setPartner1}
              />
            </FormField>
            <FormField
              name="partner2_id"
              label={t('profile.family.parent2Label')}
              optionalText
            >
              <Combobox
                ariaLabel={t('profile.family.parent2Label')}
                value={partner2}
                loadOptions={loadPeople}
                placeholder={t('profile.family.parentPlaceholder')}
                onChange={setPartner2}
              />
            </FormField>
          </div>
        ) : (
          <FormField
            name="family_id"
            label={t('profile.family.familyRefLabel')}
            hint={t('profile.family.familyRefHint')}
            required
          >
            <TextInput
              value={familyRef}
              placeholder={t('profile.family.familyRefPlaceholder')}
              onChange={(e) => setFamilyRef(e.target.value)}
            />
          </FormField>
        )}

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <FormField name="rel_to_partner1" label={t('profile.family.rel1Label')}>
            <Select
              ariaLabel={t('profile.family.rel1Label')}
              value={rel1}
              onChange={setRel1}
              options={pedigreeOptions}
            />
          </FormField>
          {showRel2 && (
            <FormField name="rel_to_partner2" label={t('profile.family.rel2Label')}>
              <Select
                ariaLabel={t('profile.family.rel2Label')}
                value={rel2}
                onChange={setRel2}
                options={pedigreeOptions}
              />
            </FormField>
          )}
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button type="submit" variant="primary" loading={submitting} disabled={!canSubmit}>
            {t('profile.family.changeParentsSubmit')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
