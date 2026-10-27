import React, { useEffect, useState } from 'react';
import { Plus, Save, Trash2 } from 'lucide-react';
import {
  Alert,
  Button,
  Dialog,
  Select,
  Switch,
  Textarea,
  TextInput,
  useToast,
} from '../ui';
import { FormField } from '../form';
import {
  AltNameType,
  ApiError,
  IndividualAltNameDto,
  PersonDetailDto,
  SexCode,
  TreePersonNodeDto,
  updatePerson,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface PersonEditDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  person: (TreePersonNodeDto & Partial<PersonDetailDto>) | null;
  onPersonUpdated?: (updated: PersonDetailDto) => void;
}

export const PersonEditDialog: React.FC<PersonEditDialogProps> = ({
  open,
  onOpenChange,
  person,
  onPersonUpdated,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [givenName, setGivenName] = useState(
    person?.primary_name.given_name ?? ''
  );
  const [patronymic, setPatronymic] = useState(
    person?.primary_name.patronymic ?? ''
  );
  const [surname, setSurname] = useState(person?.primary_name.surname ?? '');
  const [birthSurname, setBirthSurname] = useState(person?.birth_surname ?? '');
  const [nickname, setNickname] = useState(person?.nickname ?? '');
  const [namePrefix, setNamePrefix] = useState(person?.name_prefix ?? '');
  const [nameSuffix, setNameSuffix] = useState(person?.name_suffix ?? '');
  const [sex, setSex] = useState<SexCode>(person?.sex ?? 'U');
  const [livingStatus, setLivingStatus] = useState<
    'auto' | 'living' | 'deceased'
  >(
    person?.living_override === true
      ? 'living'
      : person?.living_override === false
      ? 'deceased'
      : 'auto'
  );
  const [isPrivate, setIsPrivate] = useState<boolean>(
    person?.is_private ?? false
  );
  const [biography, setBiography] = useState<string>(
    person?.biography ?? person?.notes ?? ''
  );
  const [altNames, setAltNames] = useState<IndividualAltNameDto[]>(
    person?.alt_names ?? []
  );
  const [saving, setSaving] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    if (!person) return;
    setGivenName(person.primary_name.given_name ?? '');
    setPatronymic(person.primary_name.patronymic ?? '');
    setSurname(person.primary_name.surname ?? '');
    setBirthSurname(person.birth_surname ?? '');
    setNickname(person.nickname ?? '');
    setNamePrefix(person.name_prefix ?? '');
    setNameSuffix(person.name_suffix ?? '');
    setSex(person.sex ?? 'U');
    setLivingStatus(
      person.living_override === true
        ? 'living'
        : person.living_override === false
        ? 'deceased'
        : 'auto'
    );
    setIsPrivate(Boolean(person.is_private));
    setBiography(person.biography ?? person.notes ?? '');
    setAltNames(person.alt_names ?? []);
    setErrorMsg(null);
  }, [person, open]);

  if (!person) return null;

  const handleAddAltName = () => {
    setAltNames((prev) => [
      ...prev,
      {
        name_type: 'married',
        given_name: givenName,
        patronymic: patronymic || null,
        surname: '',
      },
    ]);
  };

  const handleRemoveAltName = (index: number) => {
    setAltNames((prev) => prev.filter((_, idx) => idx !== index));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (saving || !givenName.trim()) return;
    setSaving(true);
    setErrorMsg(null);
    try {
      const livingOverride =
        livingStatus === 'living'
          ? true
          : livingStatus === 'deceased'
          ? false
          : null;

      const updated = await updatePerson(person.id, {
        given_name: givenName.trim(),
        patronymic: patronymic.trim() || null,
        surname: surname.trim() || null,
        birth_surname: birthSurname.trim() || null,
        nickname: nickname.trim() || null,
        name_prefix: namePrefix.trim() || null,
        name_suffix: nameSuffix.trim() || null,
        sex,
        living_override: livingOverride,
        is_private: isPrivate,
        biography: biography.trim() || null,
        alt_names: altNames,
        version: person.version ?? 1,
      });

      addToast({
        type: 'success',
        title: t('person.edit.savedToast'),
      });
      onPersonUpdated?.(updated);
      onOpenChange(false);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        setErrorMsg(t('person.edit.conflictError'));
      } else {
        setErrorMsg(
          err instanceof Error ? err.message : t('common.errorLoading')
        );
      }
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={t('person.edit.title')}
      description={t('person.edit.subtitle')}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <FormField
            name="edit_given_name"
            label={t('person.edit.givenNameLabel')}
            required
          >
            <TextInput
              value={givenName}
              onChange={(e) => setGivenName(e.target.value)}
              required
            />
          </FormField>

          <FormField
            name="edit_patronymic"
            label={t('person.edit.patronymicLabel')}
          >
            <TextInput
              value={patronymic}
              onChange={(e) => setPatronymic(e.target.value)}
            />
          </FormField>

          <FormField
            name="edit_surname"
            label={t('person.edit.surnameLabel')}
          >
            <TextInput
              value={surname}
              onChange={(e) => setSurname(e.target.value)}
            />
          </FormField>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-4 gap-3">
          <FormField
            name="edit_birth_surname"
            label={t('person.edit.birthSurnameLabel')}
          >
            <TextInput
              value={birthSurname}
              onChange={(e) => setBirthSurname(e.target.value)}
            />
          </FormField>

          <FormField
            name="edit_nickname"
            label={t('person.edit.nicknameLabel')}
          >
            <TextInput
              value={nickname}
              onChange={(e) => setNickname(e.target.value)}
            />
          </FormField>

          <FormField
            name="edit_prefix"
            label={t('person.edit.prefixLabel')}
          >
            <TextInput
              value={namePrefix}
              onChange={(e) => setNamePrefix(e.target.value)}
            />
          </FormField>

          <FormField
            name="edit_suffix"
            label={t('person.edit.suffixLabel')}
          >
            <TextInput
              value={nameSuffix}
              onChange={(e) => setNameSuffix(e.target.value)}
            />
          </FormField>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <FormField name="edit_sex" label={t('person.edit.sexLabel')}>
            <Select
              ariaLabel={t('person.edit.sexLabel')}
              value={sex}
              onChange={(val) => setSex(val as SexCode)}
              options={[
                { value: 'M', label: t('domain.sex.male') },
                { value: 'F', label: t('domain.sex.female') },
                { value: 'U', label: t('domain.sex.unknown') },
              ]}
            />
          </FormField>

          <FormField
            name="edit_living_override"
            label={t('person.edit.livingStatusLabel')}
          >
            <Select
              ariaLabel={t('person.edit.livingStatusLabel')}
              value={livingStatus}
              onChange={(val) =>
                setLivingStatus(val as 'auto' | 'living' | 'deceased')
              }
              options={[
                { value: 'auto', label: t('person.quickAdd.livingAuto') },
                { value: 'living', label: t('person.quickAdd.livingYes') },
                { value: 'deceased', label: t('person.quickAdd.livingNo') },
              ]}
            />
          </FormField>
        </div>

        <Switch
          checked={isPrivate}
          onCheckedChange={setIsPrivate}
          label={t('person.edit.privateRecordLabel')}
        />

        {/* Alternate Names Section */}
        <div className="rounded-xl border border-border bg-surface-muted/40 p-3 space-y-3">
          <div className="flex items-center justify-between gap-2">
            <span className="text-xs font-semibold text-primary">
              {t('person.edit.altNamesTitle')}
            </span>
            <Button
              size="sm"
              variant="secondary"
              leftIcon={<Plus className="h-3.5 w-3.5" />}
              onClick={handleAddAltName}
            >
              {t('person.edit.addAltNameBtn')}
            </Button>
          </div>

          {altNames.map((alt, idx) => (
            <div
              key={idx}
              className="grid grid-cols-1 sm:grid-cols-3 gap-2 items-center"
            >
              <Select
                ariaLabel={t('person.edit.altNameTypeLabel')}
                value={alt.name_type}
                onChange={(val) => {
                  const next = [...altNames];
                  next[idx] = { ...alt, name_type: val as AltNameType };
                  setAltNames(next);
                }}
                options={[
                  {
                    value: 'married',
                    label: t('person.edit.altTypeMarried'),
                  },
                  { value: 'alias', label: t('person.edit.altTypeAlias') },
                  {
                    value: 'transliteration',
                    label: t('person.edit.altTypeTranslit'),
                  },
                  {
                    value: 'religious',
                    label: t('person.edit.altTypeReligious'),
                  },
                  { value: 'other', label: t('person.edit.altTypeOther') },
                ]}
              />
              <TextInput
                size="sm"
                value={alt.surname ?? ''}
                placeholder={t('person.edit.surnameLabel')}
                aria-label={t('person.edit.surnameLabel')}
                onChange={(e) => {
                  const next = [...altNames];
                  next[idx] = { ...alt, surname: e.target.value };
                  setAltNames(next);
                }}
              />
              <div className="flex justify-end">
                <Button
                  size="sm"
                  variant="ghost"
                  leftIcon={<Trash2 className="h-3.5 w-3.5 text-danger" />}
                  onClick={() => handleRemoveAltName(idx)}
                >
                  {t('common.delete')}
                </Button>
              </div>
            </div>
          ))}
        </div>

        <FormField
          name="edit_biography"
          label={t('person.edit.biographyLabel')}
          optionalText
        >
          <Textarea
            value={biography}
            onChange={(e) => setBiography(e.target.value)}
            rows={3}
          />
        </FormField>

        <div className="flex justify-end gap-2 pt-2">
          <Button
            variant="secondary"
            onClick={() => onOpenChange(false)}
            disabled={saving}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={saving}
            disabled={saving || !givenName.trim()}
            leftIcon={<Save className="h-4 w-4" />}
          >
            {t('person.edit.saveBtn')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
