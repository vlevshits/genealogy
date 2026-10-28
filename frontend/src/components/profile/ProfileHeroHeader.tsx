import React, { useState } from 'react';
import {
  AlertTriangle,
  Calendar,
  ChevronDown,
  Copy,
  GitBranch,
  GitMerge,
  History,
  Home,
  Lock,
  MoreHorizontal,
  PenTool,
  Plus,
  Save,
  Sparkles,
  Trash2,
  Unlock,
  UserPlus,
  X,
} from 'lucide-react';
import {
  Alert,
  Avatar,
  Badge,
  Button,
  DropdownMenu,
  Select,
  Switch,
  TextInput,
} from '../ui';
import { FormField } from '../form';
import {
  ProfileCapabilitiesDto,
  ProfileKinshipDto,
  QuickAddRelation,
  SexCode,
  TreePersonNodeDto,
  UpdatePersonInput,
} from '../../api/client';
import { formatTreeCardName } from '../tree/PersonNodeCard';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { useI18n } from '../../i18n';

export interface ProfileHeroHeaderProps {
  person: TreePersonNodeDto;
  kinship: ProfileKinshipDto | null;
  capabilities: ProfileCapabilitiesDto;
  accessTier?: string;
  mediaCount?: number;
  onViewInTree: () => void;
  onOpenEditModal: () => void;
  onSaveInlineIdentity?: (input: UpdatePersonInput) => Promise<void>;
  onAddEvent: () => void;
  onQuickAdd: (relation: QuickAddRelation) => void;
  onSetHomePerson: () => void;
  onTogglePrivate: () => void;
  onDeletePerson: () => void;
  onOpenHistory?: () => void;
  onMergePerson?: () => void;
  onCopyLink?: () => void;
}

export const ProfileHeroHeader: React.FC<ProfileHeroHeaderProps> = ({
  person,
  kinship,
  capabilities,
  accessTier = 'admin',
  mediaCount = 0,
  onViewInTree,
  onOpenEditModal,
  onSaveInlineIdentity,
  onAddEvent,
  onQuickAdd,
  onSetHomePerson,
  onTogglePrivate,
  onDeletePerson,
  onOpenHistory,
  onMergePerson,
  onCopyLink,
}) => {
  const { t, locale } = useI18n();
  const { displayName, isMaskedLiving } = formatTreeCardName(
    person,
    accessTier,
    locale,
    'card'
  );

  // In-place identity editing state (03 §3.3)
  const [inlineEditing, setInlineEditing] = useState(false);
  const [givenName, setGivenName] = useState(
    person.primary_name.given_name || ''
  );
  const [patronymic, setPatronymic] = useState(
    person.primary_name.patronymic || ''
  );
  const [surname, setSurname] = useState(person.primary_name.surname || '');
  const [birthSurname, setBirthSurname] = useState(person.birth_surname || '');
  const [sex, setSex] = useState<SexCode>((person.sex as SexCode) || 'U');
  const [isPrivate, setIsPrivate] = useState(Boolean(person.is_private));
  const [savingInline, setSavingInline] = useState(false);
  const [inlineError, setInlineError] = useState<string | null>(null);

  const startInlineEdit = () => {
    setGivenName(person.primary_name.given_name || '');
    setPatronymic(person.primary_name.patronymic || '');
    setSurname(person.primary_name.surname || '');
    setBirthSurname(person.birth_surname || '');
    setSex((person.sex as SexCode) || 'U');
    setIsPrivate(Boolean(person.is_private));
    setInlineError(null);
    setInlineEditing(true);
  };

  const handleInlineSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!onSaveInlineIdentity) {
      setInlineEditing(false);
      return;
    }
    setSavingInline(true);
    setInlineError(null);
    try {
      await onSaveInlineIdentity({
        given_name: givenName.trim(),
        patronymic: patronymic.trim() || null,
        surname: surname.trim() || null,
        birth_surname: birthSurname.trim() || null,
        sex,
        is_private: isPrivate,
        version: person.version ?? 1,
      });
      setInlineEditing(false);
    } catch (err) {
      setInlineError(getLocalizedErrorMessage(err, t));
    } finally {
      setSavingInline(false);
    }
  };

  // Compute age for deceased or living
  const birthYear =
    person.birth_sort
      ? Math.floor(person.birth_sort / 10000)
      : person.birth?.date
      ? Number((person.birth.date.match(/\b(\d{4})\b/) || [])[1]) || null
      : null;

  const deathYear =
    person.death_sort
      ? Math.floor(person.death_sort / 10000)
      : person.death?.date
      ? Number((person.death.date.match(/\b(\d{4})\b/) || [])[1]) || null
      : null;

  const lifespanAge =
    birthYear && deathYear && deathYear >= birthYear
      ? deathYear - birthYear
      : null;

  // Dynamic kinship label
  const kinshipLabel = kinship
    ? locale === 'ru' && kinship.label_ru
      ? kinship.label_ru
      : kinship.label_en ||
        t(
          `profile.kinship.${kinship.relation_key || kinship.code || 'relative'}`
        )
    : null;

  // Build overflow menu items based on role capabilities (03 §3.1)
  const overflowItems = [
    ...(accessTier !== 'public_anonymous'
      ? [
          {
            id: 'set-home',
            label: t('profile.actions.setHomePerson'),
            icon: <Home className="h-3.5 w-3.5" />,
            onSelect: onSetHomePerson,
          },
        ]
      : []),
    ...(capabilities.can_edit
      ? [
          {
            id: 'edit-modal',
            label: t('profile.actions.fullIdentityDialog'),
            icon: <PenTool className="h-3.5 w-3.5" />,
            onSelect: onOpenEditModal,
          },
          {
            id: 'toggle-private',
            label: person.is_private
              ? t('profile.actions.unmarkPrivate')
              : t('profile.actions.markPrivate'),
            icon: person.is_private ? (
              <Unlock className="h-3.5 w-3.5" />
            ) : (
              <Lock className="h-3.5 w-3.5" />
            ),
            onSelect: onTogglePrivate,
          },
        ]
      : []),
    ...(capabilities.can_merge
      ? [
          {
            id: 'history',
            label: t('profile.actions.viewHistory'),
            icon: <History className="h-3.5 w-3.5" />,
            separatorBefore: true,
            onSelect: () => onOpenHistory?.(),
          },
          {
            id: 'merge',
            label: t('profile.actions.mergeWith'),
            icon: <GitMerge className="h-3.5 w-3.5" />,
            onSelect: () => onMergePerson?.(),
          },
        ]
      : []),
    {
      id: 'copy-link',
      label: t('profile.actions.copyLink'),
      icon: <Copy className="h-3.5 w-3.5" />,
      separatorBefore: accessTier !== 'public_anonymous',
      onSelect: () => onCopyLink?.(),
    },
    ...(capabilities.can_delete
      ? [
          {
            id: 'delete',
            label: t('profile.actions.deletePerson'),
            icon: <Trash2 className="h-3.5 w-3.5" />,
            danger: true,
            separatorBefore: true,
            onSelect: onDeletePerson,
          },
        ]
      : []),
  ];

  const stripeColorClass =
    person.sex === 'M'
      ? 'bg-male'
      : person.sex === 'F'
      ? 'bg-female'
      : 'bg-neutral';

  return (
    <section
      aria-label={t('profile.hero.ariaLabel')}
      className="relative overflow-hidden rounded-3xl border border-border bg-surface p-6 sm:p-8 shadow-xs"
    >
      {/* Top lineage accent bar */}
      <span
        aria-hidden="true"
        className={`absolute inset-x-0 top-0 h-1.5 ${stripeColorClass}`}
      />

      <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-6">
        {/* Left: Portrait Avatar + Biographical Identity */}
        <div className="flex flex-col sm:flex-row items-start gap-5 min-w-[280px] flex-1">
          <div className="relative shrink-0">
            <Avatar
              name={isMaskedLiving ? '?' : displayName}
              src={isMaskedLiving ? undefined : person.avatar_url ?? undefined}
              size="xl"
              className="h-24 w-24 sm:h-28 sm:w-28 rounded-2xl text-2xl shadow-xs border border-border"
            />
            {!isMaskedLiving && mediaCount > 1 && (
              <div className="mt-1.5 text-center font-mono text-[11px] text-tertiary">
                {t('profile.hero.photoCountIndicator', {
                  current: 1,
                  total: mediaCount,
                })}
              </div>
            )}
          </div>

          <div className="space-y-2.5 min-w-[280px] flex-1">
            {/* Badges row: Home Person, Kinship Pill, Private Badge (NO Living pill!) */}
            <div className="flex flex-wrap items-center gap-2">
              {capabilities.is_home_person && (
                <Badge
                  variant="brand"
                  size="sm"
                  icon={<Home className="h-3 w-3" />}
                >
                  {t('tree.homePersonBadge')}
                </Badge>
              )}

              {kinshipLabel && !isMaskedLiving && (
                <Badge
                  variant="brand"
                  size="sm"
                  icon={<Sparkles className="h-3 w-3" />}
                >
                  {t('profile.hero.kinshipBadge', { relation: kinshipLabel })}
                </Badge>
              )}

              {person.is_private && !isMaskedLiving && (
                <Badge
                  variant="warning"
                  size="sm"
                  icon={<Lock className="h-3 w-3" />}
                >
                  {t('domain.privateBadge')}
                </Badge>
              )}
            </div>

            {/* Primary Heading */}
            <div className="space-y-1">
              <h1 className="font-serif text-2xl sm:text-3xl font-semibold tracking-tight text-primary break-normal hyphens-none">
                {displayName}
                {!isMaskedLiving &&
                  person.birth_surname &&
                  person.birth_surname !== person.primary_name.surname && (
                    <span className="ml-2 text-lg font-normal text-secondary">
                      ({t('profile.hero.neeLabel')} {person.birth_surname})
                    </span>
                  )}
              </h1>

              {/* Alternate names */}
              {!isMaskedLiving &&
                person.alt_names &&
                person.alt_names.length > 0 && (
                  <div className="flex flex-wrap items-center gap-1.5 text-xs text-secondary">
                    <span className="text-tertiary">
                      {t('profile.hero.alsoKnownAs')}:
                    </span>
                    {person.alt_names.map((alt, idx) => {
                      const altFull = [
                        alt.given_name,
                        alt.patronymic,
                        alt.surname,
                      ]
                        .filter(Boolean)
                        .join(' ');
                      if (!altFull) return null;
                      return (
                        <Badge key={alt.id || idx} variant="neutral" size="sm">
                          {altFull}
                        </Badge>
                      );
                    })}
                  </div>
                )}
            </div>

            {/* Vital Dates Line: Living shows birth only; Deceased shows subtle † cross */}
            {isMaskedLiving ? (
              <div className="flex items-center gap-2 text-xs sm:text-sm text-secondary">
                <Lock className="h-4 w-4 text-warning shrink-0" />
                <span>{t('profile.hero.publicLivingNotice')}</span>
              </div>
            ) : (
              <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs sm:text-sm text-secondary">
                {person.birth?.date || person.birth?.place ? (
                  <span className="inline-flex items-center gap-1.5">
                    <Calendar className="h-3.5 w-3.5 text-tertiary shrink-0" />
                    <span>
                      {t('profile.hero.bornPrefix')}{' '}
                      <strong className="font-medium text-primary">
                        {person.birth.date || t('profile.hero.dateUnknown')}
                      </strong>
                      {person.birth.place ? ` (${person.birth.place})` : ''}
                    </span>
                  </span>
                ) : (
                  <span className="text-tertiary">
                    {t('profile.hero.birthNotRecorded')}
                  </span>
                )}

                {!person.is_living && (
                  <span className="inline-flex items-center gap-1.5">
                    <span
                      aria-label={t('person.quickAdd.livingNo')}
                      className="font-serif font-semibold text-secondary"
                    >
                      · †
                    </span>
                    <span>
                      <strong className="font-medium text-primary">
                        {person.death?.date || t('profile.hero.deceasedLabel')}
                      </strong>
                      {person.death?.place ? ` (${person.death.place})` : ''}
                      {lifespanAge !== null
                        ? ` · ${t('profile.hero.ageAtDeath', {
                            age: lifespanAge,
                          })}`
                        : ''}
                    </span>
                  </span>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Right: Action Buttons */}
        <div className="flex flex-wrap items-center gap-2 shrink-0">
          <Button
            size="md"
            variant="secondary"
            leftIcon={<GitBranch className="h-4 w-4" />}
            onClick={onViewInTree}
          >
            {t('profile.actions.viewInTree')}
          </Button>

          {capabilities.can_edit && !isMaskedLiving && (
            <>
              <Button
                size="md"
                variant="secondary"
                leftIcon={<PenTool className="h-4 w-4" />}
                onClick={startInlineEdit}
              >
                {t('profile.actions.editProfile')}
              </Button>

              <DropdownMenu
                align="end"
                ariaLabel={t('profile.actions.addMenuAria')}
                trigger={
                  <Button
                    size="md"
                    variant="primary"
                    leftIcon={<Plus className="h-4 w-4" />}
                    rightIcon={<ChevronDown className="h-3.5 w-3.5" />}
                  >
                    {t('profile.actions.addMenuTrigger')}
                  </Button>
                }
                items={[
                  {
                    id: 'add-event',
                    label: t('profile.actions.addEvent'),
                    icon: <Calendar className="h-3.5 w-3.5" />,
                    onSelect: onAddEvent,
                  },
                  {
                    id: 'add-relative',
                    label: t('profile.actions.addRelative'),
                    icon: <UserPlus className="h-3.5 w-3.5" />,
                    onSelect: () => onQuickAdd('child'),
                  },
                ]}
              />
            </>
          )}

          <DropdownMenu
            align="end"
            ariaLabel={t('profile.actions.moreAria')}
            trigger={
              <span className="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-border bg-surface text-secondary hover:bg-surface-muted hover:text-primary transition-colors">
                <MoreHorizontal className="h-4 w-4" />
              </span>
            }
            items={overflowItems}
          />
        </div>
      </div>

      {/* Inline Identity Editor Form (03 §3.3) */}
      {inlineEditing && capabilities.can_edit && (
        <form
          onSubmit={handleInlineSubmit}
          className="mt-6 space-y-4 rounded-2xl border border-brand/40 bg-surface-muted/50 p-4 sm:p-5"
        >
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-primary">
              {t('profile.inlineEdit.title')}
            </h2>
            <Button
              size="sm"
              variant="ghost"
              onClick={onOpenEditModal}
            >
              {t('profile.actions.fullIdentityDialog')}
            </Button>
          </div>

          {inlineError && <Alert variant="danger">{inlineError}</Alert>}

          <div className="grid grid-cols-1 sm:grid-cols-4 gap-3">
            <FormField
              name="given_name"
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
              name="patronymic"
              label={t('person.edit.patronymicLabel')}
            >
              <TextInput
                value={patronymic}
                onChange={(e) => setPatronymic(e.target.value)}
              />
            </FormField>

            <FormField name="surname" label={t('person.edit.surnameLabel')}>
              <TextInput
                value={surname}
                onChange={(e) => setSurname(e.target.value)}
              />
            </FormField>

            <FormField
              name="birth_surname"
              label={t('person.edit.birthSurnameLabel')}
            >
              <TextInput
                value={birthSurname}
                onChange={(e) => setBirthSurname(e.target.value)}
              />
            </FormField>
          </div>

          <div className="flex flex-wrap items-center justify-between gap-4 pt-1">
            <div className="flex flex-wrap items-center gap-4">
              <div className="w-40">
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
              </div>

              <Switch
                checked={isPrivate}
                onCheckedChange={setIsPrivate}
                label={t('person.edit.privateRecordLabel')}
              />
            </div>

            <div className="flex items-center gap-2">
              <Button
                variant="secondary"
                size="sm"
                leftIcon={<X className="h-3.5 w-3.5" />}
                onClick={() => setInlineEditing(false)}
                disabled={savingInline}
              >
                {t('common.cancel')}
              </Button>
              <Button
                type="submit"
                variant="primary"
                size="sm"
                loading={savingInline}
                leftIcon={<Save className="h-3.5 w-3.5" />}
              >
                {t('common.save')}
              </Button>
            </div>
          </div>
        </form>
      )}

      {/* Footer Metadata Bar: GID, Version & Admin Data Quality check */}
      <div className="mt-5 flex flex-wrap items-center justify-between gap-2 border-t border-border/70 pt-3 text-xs text-tertiary">
        <div className="flex flex-wrap items-center gap-3">
          <span className="font-mono">
            {t('profile.hero.gidLabel')}: <strong className="text-secondary">{person.gid}</strong>
          </span>
          <span className="font-mono">
            v{person.version ?? 1}
          </span>
        </div>

        {capabilities.can_merge &&
          !isMaskedLiving &&
          !person.birth?.date && (
            <div className="inline-flex items-center gap-1.5 text-warning font-medium">
              <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
              <span>{t('profile.hero.qualityMissingBirthHint')}</span>
            </div>
          )}
      </div>
    </section>
  );
};
