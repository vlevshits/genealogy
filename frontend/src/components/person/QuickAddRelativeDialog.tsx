import React, { useEffect, useState } from 'react';
import { Link2, UserPlus } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  SegmentedControl,
  Select,
  TextInput,
  useToast,
} from '../ui';
import { FormField, GenDateInput } from '../form';
import {
  createPerson,
  PersonDetailDto,
  QuickAddRelation,
  quickAddRelative,
  SexCode,
  TreePersonNodeDto,
} from '../../api/client';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { computeSlavicPatronymic } from '../../lib/patronymic';
import { QuickAddDuplicateHint } from '../quality/QuickAddDuplicateHint';
import { useI18n } from '../../i18n';

export interface QuickAddRelativeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  anchorPerson?: TreePersonNodeDto | null;
  relation?: QuickAddRelation;
  existingPeople?: TreePersonNodeDto[];
  initialMode?: 'create' | 'link';
  onRelativeAdded?: (createdOrLinked?: PersonDetailDto) => void;
}

export const QuickAddRelativeDialog: React.FC<QuickAddRelativeDialogProps> = ({
  open,
  onOpenChange,
  anchorPerson = null,
  relation = 'child',
  existingPeople = [],
  initialMode = 'create',
  onRelativeAdded,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const defaultSexForRelation = (rel: QuickAddRelation): SexCode => {
    if (rel === 'father') return 'M';
    if (rel === 'mother') return 'F';
    if (rel === 'spouse' && anchorPerson) {
      return anchorPerson.sex === 'M'
        ? 'F'
        : anchorPerson.sex === 'F'
        ? 'M'
        : 'U';
    }
    return 'M';
  };

  const fatherGivenNameForChild =
    relation === 'child' && anchorPerson?.sex === 'M'
      ? anchorPerson.primary_name.given_name
      : '';

  const [mode, setMode] = useState<'create' | 'link'>(initialMode);
  const [selectedExistingId, setSelectedExistingId] = useState<string>('');
  const [sex, setSex] = useState<SexCode>(() =>
    defaultSexForRelation(relation)
  );
  const [givenName, setGivenName] = useState<string>('');
  const [patronymic, setPatronymic] = useState<string>(() =>
    fatherGivenNameForChild
      ? computeSlavicPatronymic(
          fatherGivenNameForChild,
          defaultSexForRelation(relation)
        )
      : ''
  );
  const [patronymicTouched, setPatronymicTouched] = useState<boolean>(false);
  const [surname, setSurname] = useState<string>(() =>
    anchorPerson?.primary_name.surname ?? ''
  );
  const [birthDate, setBirthDate] = useState<string>('');
  const [parsedBirthDate, setParsedBirthDate] = useState<import('../../types/gendate').GenDate | null>(null);
  const [birthPlace, setBirthPlace] = useState<string>('');
  const [livingStatus, setLivingStatus] = useState<'auto' | 'living' | 'deceased'>(
    'auto'
  );
  const [submitting, setSubmitting] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    const nextSex = defaultSexForRelation(relation);
    setMode(anchorPerson ? initialMode : 'create');
    setSelectedExistingId('');
    setGivenName('');
    setBirthDate('');
    setParsedBirthDate(null);
    setBirthPlace('');
    setLivingStatus('auto');
    setSex(nextSex);
    setSurname(anchorPerson?.primary_name.surname || '');
    setPatronymicTouched(false);
    if (anchorPerson && relation === 'child' && anchorPerson.sex === 'M') {
      setPatronymic(
        computeSlavicPatronymic(anchorPerson.primary_name.given_name, nextSex)
      );
    } else {
      setPatronymic('');
    }
    setErrorMsg(null);
  }, [anchorPerson, relation, open, initialMode]);

  if (!anchorPerson && !open) return null;

  const anchorDisplayName = anchorPerson
    ? [
        anchorPerson.primary_name.given_name,
        anchorPerson.primary_name.patronymic,
        anchorPerson.primary_name.surname,
      ]
        .filter(Boolean)
        .join(' ')
    : '';

  const dialogTitle = !anchorPerson
    ? t('nav.addPerson')
    : relation === 'father'
    ? t('person.quickAdd.titleFather', { name: anchorDisplayName })
    : relation === 'mother'
    ? t('person.quickAdd.titleMother', { name: anchorDisplayName })
    : relation === 'spouse'
    ? t('person.quickAdd.titleSpouse', { name: anchorDisplayName })
    : relation === 'child'
    ? t('person.quickAdd.titleChild', { name: anchorDisplayName })
    : t('person.quickAdd.titleSibling', { name: anchorDisplayName });

  const handleSexChange = (nextSex: SexCode) => {
    setSex(nextSex);
    if (
      anchorPerson &&
      !patronymicTouched &&
      relation === 'child' &&
      anchorPerson.sex === 'M'
    ) {
      setPatronymic(
        computeSlavicPatronymic(anchorPerson.primary_name.given_name, nextSex)
      );
    }
  };

  const candidateOptions = existingPeople
    .filter((p) => p.id !== anchorPerson?.id)
    .map((p) => ({
      value: p.id,
      label: `${[
        p.primary_name.given_name,
        p.primary_name.patronymic,
        p.primary_name.surname,
      ]
        .filter(Boolean)
        .join(' ')} (${p.gid})`,
    }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (submitting) return;
    setSubmitting(true);
    setErrorMsg(null);
    try {
      const livingOverride =
        livingStatus === 'living'
          ? true
          : livingStatus === 'deceased'
          ? false
          : null;

      if (!anchorPerson) {
        const created = await createPerson({
          given_name: givenName.trim(),
          patronymic: patronymic.trim() || null,
          surname: surname.trim() || null,
          sex,
          living_override: livingOverride,
        });
        addToast({
          type: 'success',
          title: t('person.quickAdd.addedToast'),
        });
        onRelativeAdded?.(created);
        onOpenChange(false);
        return;
      }

      const res = await quickAddRelative(
        anchorPerson.id,
        mode === 'link'
          ? {
              relation,
              existing_person_id: selectedExistingId,
            }
          : {
              relation,
              given_name: givenName.trim(),
              patronymic: patronymic.trim() || null,
              surname: surname.trim() || null,
              sex,
              birth_date: birthDate.trim() || null,
              birth_place: birthPlace.trim() || null,
              living_override: livingOverride,
            }
      );
      addToast({
        type: 'success',
        title: t('person.quickAdd.addedToast'),
      });
      onRelativeAdded?.(res.person);
      onOpenChange(false);
    } catch (err) {
      setErrorMsg(getLocalizedErrorMessage(err, t));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={dialogTitle}
      description={t('person.quickAdd.subtitle')}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {anchorPerson && (
          <SegmentedControl
            ariaLabel={t('person.quickAdd.subtitle')}
            value={mode}
            onChange={(val) => setMode(val as 'create' | 'link')}
            options={[
              {
                value: 'create',
                label: t('person.quickAdd.modeCreateNew'),
                icon: <UserPlus className="h-3.5 w-3.5" />,
              },
              {
                value: 'link',
                label: t('person.quickAdd.modeLinkExisting'),
                icon: <Link2 className="h-3.5 w-3.5" />,
              },
            ]}
          />
        )}

        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        {mode === 'link' ? (
          <FormField
            name="existing_person_id"
            label={t('person.quickAdd.selectExistingLabel')}
            required
          >
            <Select
              ariaLabel={t('person.quickAdd.selectExistingLabel')}
              value={selectedExistingId}
              placeholder={t('person.quickAdd.selectExistingPlaceholder')}
              onChange={(val) => setSelectedExistingId(val)}
              options={candidateOptions}
            />
          </FormField>
        ) : (
          <div className="space-y-4">
            <QuickAddDuplicateHint
              givenName={givenName}
              surname={surname}
              birthDate={birthDate}
              sex={sex}
              anchorPersonId={anchorPerson?.id}
              existingPeople={existingPeople}
              onUseExisting={(personId) => {
                setMode('link');
                setSelectedExistingId(personId);
              }}
              onOpenProfile={(personId) => {
                if (typeof window !== 'undefined') {
                  window.history.pushState({}, '', `/persons/${personId}`);
                  window.dispatchEvent(new PopStateEvent('popstate'));
                }
                onOpenChange(false);
              }}
            />

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              <FormField
                name="given_name"
                label={t('person.quickAdd.givenNameLabel')}
                required
              >
                <TextInput
                  value={givenName}
                  onChange={(e) => setGivenName(e.target.value)}
                  required
                />
              </FormField>

              <FormField
                name="patronymic"
                label={t('person.quickAdd.patronymicLabel')}
                hint={
                  fatherGivenNameForChild
                    ? t('person.quickAdd.patronymicAutoHint', {
                        father: fatherGivenNameForChild,
                      })
                    : undefined
                }
              >
                <TextInput
                  value={patronymic}
                  onChange={(e) => {
                    setPatronymicTouched(true);
                    setPatronymic(e.target.value);
                  }}
                />
              </FormField>

              <FormField
                name="surname"
                label={t('person.quickAdd.surnameLabel')}
              >
                <TextInput
                  value={surname}
                  onChange={(e) => setSurname(e.target.value)}
                />
              </FormField>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <FormField
                name="sex"
                label={t('person.quickAdd.sexLabel')}
              >
                <Select
                  ariaLabel={t('person.quickAdd.sexLabel')}
                  value={sex}
                  onChange={(val) => handleSexChange(val as SexCode)}
                  options={[
                    { value: 'M', label: t('domain.sex.male') },
                    { value: 'F', label: t('domain.sex.female') },
                    { value: 'U', label: t('domain.sex.unknown') },
                  ]}
                />
              </FormField>

              <FormField
                name="living_status"
                label={t('person.quickAdd.livingStatusLabel')}
              >
                <Select
                  ariaLabel={t('person.quickAdd.livingStatusLabel')}
                  value={livingStatus}
                  onChange={(val) =>
                    setLivingStatus(val as 'auto' | 'living' | 'deceased')
                  }
                  options={[
                    {
                      value: 'auto',
                      label: t('person.quickAdd.livingAuto'),
                    },
                    {
                      value: 'living',
                      label: t('person.quickAdd.livingYes'),
                    },
                    {
                      value: 'deceased',
                      label: t('person.quickAdd.livingNo'),
                    },
                  ]}
                />
              </FormField>
            </div>

            <FormField
              name="birth_date"
              label={t('person.quickAdd.birthDateLabel')}
              optionalText
            >
              <GenDateInput
                value={parsedBirthDate}
                onChange={(result) => {
                  setParsedBirthDate(result.value);
                  setBirthDate(result.value.text);
                }}
              />
            </FormField>

            <FormField
              name="birth_place"
              label={t('person.quickAdd.birthPlaceLabel')}
              optionalText
            >
              <TextInput
                value={birthPlace}
                onChange={(e) => setBirthPlace(e.target.value)}
                placeholder={t('person.quickAdd.birthPlacePlaceholder')}
              />
            </FormField>
          </div>
        )}

        <div className="flex justify-end gap-2 pt-2">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={submitting}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={submitting}
            disabled={
              submitting ||
              (mode === 'create'
                ? !givenName.trim()
                : !selectedExistingId)
            }
          >
            {mode === 'create'
              ? t('person.quickAdd.submitCreateBtn')
              : t('person.quickAdd.submitLinkBtn')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
