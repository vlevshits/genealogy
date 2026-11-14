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
import { ConflictResolutionModal } from '../history';
import { useLiveVersion } from '../../hooks/useLiveVersion';
import { useUndoToast } from '../../hooks/useUndoToast';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
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
  const { showUndoToast } = useUndoToast();

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
  const [conflictState, setConflictState] = useState<{
    open: boolean;
    myChanges: Record<string, unknown>;
    theirChanges: Record<string, unknown>;
    remoteVersion: number;
    updatedByName?: string | null;
    updatedAt?: string | null;
  }>({
    open: false,
    myChanges: {},
    theirChanges: {},
    remoteVersion: 2,
  });

  const { hasRemoteUpdate, remoteVersion, updatedByName } = useLiveVersion({
    entityType: 'persons',
    entityId: person?.id,
    initialVersion: person?.version ?? 1,
    enabled: open && Boolean(person?.id),
  });

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

  const buildPayload = (versionToUse: number) => {
    const livingOverride =
      livingStatus === 'living'
        ? true
        : livingStatus === 'deceased'
        ? false
        : null;
    return {
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
      version: versionToUse,
    };
  };

  const openConflictFromError = (
    err: ApiError,
    submittedPayload: Record<string, unknown>
  ) => {
    const details = (err.details ?? {}) as {
      current?: Record<string, unknown>;
      their_data?: Record<string, unknown>;
      version?: number;
      current_version?: number;
      expected_version?: number;
      updated_by_name?: string;
      updated_at?: string;
    };
    const theirData = (details.their_data ??
      details.current ??
      {}) as Record<string, unknown>;
    const theirPrimaryName = (theirData.primary_name ?? {}) as Record<
      string,
      unknown
    >;
    const currentRemote: Record<string, unknown> = {
      given_name:
        theirPrimaryName.given_name ??
        theirData.given_name ??
        person.primary_name.given_name,
      patronymic:
        theirPrimaryName.patronymic ??
        theirData.patronymic ??
        person.primary_name.patronymic,
      surname:
        theirPrimaryName.surname ??
        theirData.surname ??
        person.primary_name.surname,
      birth_surname: theirData.birth_surname ?? person.birth_surname,
      nickname: theirData.nickname ?? person.nickname,
      sex: theirData.sex ?? person.sex,
      biography:
        theirData.biography ??
        theirData.notes ??
        person.biography ??
        person.notes ??
        null,
    };
    const resolvedRemoteVersion =
      details.current_version ??
      details.version ??
      (typeof theirData.version === 'number'
        ? theirData.version
        : undefined) ??
      remoteVersion ??
      (person.version ?? 1) + 1;
    setConflictState({
      open: true,
      myChanges: {
        given_name: submittedPayload.given_name,
        patronymic: submittedPayload.patronymic,
        surname: submittedPayload.surname,
        birth_surname: submittedPayload.birth_surname,
        nickname: submittedPayload.nickname,
        sex: submittedPayload.sex,
        biography: submittedPayload.biography,
      },
      theirChanges: currentRemote,
      remoteVersion: resolvedRemoteVersion,
      updatedByName:
        details.updated_by_name ??
        (typeof theirData.updated_by_name === 'string'
          ? theirData.updated_by_name
          : undefined) ??
        updatedByName,
      updatedAt:
        details.updated_at ??
        (typeof theirData.updated_at === 'string'
          ? theirData.updated_at
          : undefined),
    });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (saving || !givenName.trim()) return;
    setSaving(true);
    setErrorMsg(null);
    const payload = buildPayload(person.version ?? 1);
    try {
      const updated = await updatePerson(person.id, payload);
      const txId = (updated as unknown as { tx_id?: string }).tx_id;
      if (txId) {
        showUndoToast({
          txId,
          title: t('history.undo_toast_label'),
          description: t('person.edit.savedToast'),
          onUndone: () => onPersonUpdated?.(person as PersonDetailDto),
        });
      } else {
        addToast({
          type: 'success',
          title: t('person.edit.savedToast'),
        });
      }
      onPersonUpdated?.(updated);
      onOpenChange(false);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        openConflictFromError(err, payload);
      } else {
        setErrorMsg(getLocalizedErrorMessage(err, t));
      }
    } finally {
      setSaving(false);
    }
  };

  return (
    <>
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={t('person.edit.title')}
      description={t('person.edit.subtitle')}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {hasRemoteUpdate && (
          <Alert variant="warning" data-testid="live-version-warning-banner">
            {t('history.liveVersion.warningBanner', {
              name: updatedByName || t('history.conflict.anotherEditor'),
            })}
          </Alert>
        )}
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

    <ConflictResolutionModal
      open={conflictState.open}
      onOpenChange={(nextOpen) =>
        setConflictState((prev) => ({ ...prev, open: nextOpen }))
      }
      updatedByName={conflictState.updatedByName}
      updatedAt={conflictState.updatedAt}
      remoteVersion={conflictState.remoteVersion}
      myChanges={conflictState.myChanges}
      theirChanges={conflictState.theirChanges}
      onKeepMine={async (resolvedVersion) => {
        const targetVersion =
          typeof resolvedVersion === 'number'
            ? resolvedVersion
            : conflictState.remoteVersion;
        const retryPayload = buildPayload(targetVersion);
        try {
          const updated = await updatePerson(person.id, retryPayload);
          onPersonUpdated?.(updated);
          setConflictState((prev) => ({ ...prev, open: false }));
          onOpenChange(false);
        } catch (err) {
          if (err instanceof ApiError && err.status === 409) {
            openConflictFromError(err, retryPayload);
            return;
          }
          throw err;
        }
      }}
      onUseTheirs={() => {
        setConflictState((prev) => ({ ...prev, open: false }));
        onOpenChange(false);
      }}
      onMerge={async (mergedFields, resolvedVersion) => {
        const targetVersion =
          typeof resolvedVersion === 'number'
            ? resolvedVersion
            : conflictState.remoteVersion;
        const base = buildPayload(targetVersion);
        const mergedPayload = {
          ...base,
          ...(mergedFields as Partial<typeof base>),
          version: targetVersion,
        };
        try {
          const updated = await updatePerson(person.id, mergedPayload);
          onPersonUpdated?.(updated);
          setConflictState((prev) => ({ ...prev, open: false }));
          onOpenChange(false);
        } catch (err) {
          if (err instanceof ApiError && err.status === 409) {
            openConflictFromError(err, mergedPayload);
            return;
          }
          throw err;
        }
      }}
    />
    </>
  );
};
