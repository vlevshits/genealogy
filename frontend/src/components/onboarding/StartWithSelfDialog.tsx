import React, { useEffect, useState } from 'react';
import { UserCheck } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  Select,
  TextInput,
  useToast,
} from '../ui';
import { FormField, GenDateInput } from '../form';
import {
  IndividualDto,
  SexCode,
  startWithSelf,
  UserDto,
} from '../../api/client';
import { useAuth } from '../../lib/auth-context';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';
import { parseGenDate } from '../../lib/gendate-parser';
import { GenDate } from '../../types/gendate';

export interface StartWithSelfDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCompleted?: (individual: IndividualDto, user: UserDto) => void;
}

export const StartWithSelfDialog: React.FC<StartWithSelfDialogProps> = ({
  open,
  onOpenChange,
  onCompleted,
}) => {
  const { t, locale } = useI18n();
  const { user, updateUser } = useAuth();
  const { addToast } = useToast();

  const [givenName, setGivenName] = useState<string>(user?.first_name ?? '');
  const [patronymic, setPatronymic] = useState<string>('');
  const [surname, setSurname] = useState<string>(user?.last_name ?? '');
  const [sex, setSex] = useState<SexCode>('M');
  const [birthDate, setBirthDate] = useState<GenDate | null>(null);
  const [birthPlace, setBirthPlace] = useState<string>('');
  const [saving, setSaving] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setGivenName(user?.first_name ?? '');
      setSurname(user?.last_name ?? '');
      setPatronymic('');
      setSex('M');
      setBirthDate(null);
      setBirthPlace('');
      setErrorMsg(null);
    }
  }, [open, user?.first_name, user?.last_name]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (saving || !givenName.trim() || !surname.trim()) return;

    setSaving(true);
    setErrorMsg(null);
    try {
      const res = await startWithSelf({
        given_name: givenName.trim(),
        patronymic: patronymic.trim() || null,
        surname: surname.trim(),
        sex,
        birth_date: birthDate?.text?.trim() || null,
        birth_place: birthPlace.trim() || null,
      });
      updateUser(res.user);
      addToast({
        type: 'success',
        title: t('onboarding.selfDialog.createdToast'),
        description: [
          res.individual?.primary_name?.given_name ??
            res.individual?.given_name ??
            givenName.trim(),
          res.individual?.primary_name?.patronymic ??
            res.individual?.patronymic ??
            patronymic.trim(),
          res.individual?.primary_name?.surname ??
            res.individual?.surname ??
            surname.trim(),
        ]
          .filter(Boolean)
          .join(' '),
      });
      onCompleted?.(res.individual, res.user);
      onOpenChange(false);
    } catch (err) {
      setErrorMsg(getLocalizedErrorMessage(err, t));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={t('onboarding.selfDialog.title')}
      description={t('onboarding.selfDialog.subtitle')}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <FormField
            name="self_given_name"
            label={t('person.fields.givenName')}
            required
          >
            <TextInput
              value={givenName}
              onChange={(e) => setGivenName(e.target.value)}
              placeholder={t('person.fields.givenNamePlaceholder')}
              required
            />
          </FormField>

          <FormField
            name="self_patronymic"
            label={t('person.fields.patronymic')}
          >
            <TextInput
              value={patronymic}
              onChange={(e) => setPatronymic(e.target.value)}
              placeholder={t('person.fields.patronymicPlaceholder')}
            />
          </FormField>

          <FormField
            name="self_surname"
            label={t('person.fields.surname')}
            required
          >
            <TextInput
              value={surname}
              onChange={(e) => setSurname(e.target.value)}
              placeholder={t('person.fields.surnamePlaceholder')}
              required
            />
          </FormField>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <FormField name="self_sex" label={t('person.fields.sex')} required>
            <Select
              ariaLabel={t('person.fields.sex')}
              value={sex}
              onChange={(val) => setSex(val as SexCode)}
              options={[
                { value: 'M', label: t('person.sex.male') },
                { value: 'F', label: t('person.sex.female') },
                { value: 'U', label: t('person.sex.unknown') },
              ]}
            />
          </FormField>

          <FormField
            name="self_birth_place"
            label={t('person.fields.birthPlace')}
          >
            <TextInput
              value={birthPlace}
              onChange={(e) => setBirthPlace(e.target.value)}
              placeholder={t('onboarding.selfDialog.birthPlacePlaceholder')}
            />
          </FormField>
        </div>

        <FormField
          name="self_birth_date"
          label={t('person.fields.birthDate')}
          hint={t('onboarding.selfDialog.birthDateHint')}
        >
          <GenDateInput
            value={
              birthDate ??
              parseGenDate('', locale === 'ru' ? 'ru' : 'en').value
            }
            onChange={(res) => setBirthDate(res.value)}
          />
        </FormField>

        <div className="flex items-center justify-end gap-2 pt-2">
          <Button
            type="button"
            variant="secondary"
            onClick={() => onOpenChange(false)}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={saving}
            disabled={saving || !givenName.trim() || !surname.trim()}
            leftIcon={<UserCheck className="h-4 w-4" />}
          >
            {t('onboarding.selfDialog.submitBtn')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
