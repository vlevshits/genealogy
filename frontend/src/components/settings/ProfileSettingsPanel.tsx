import React, { useState, useEffect } from 'react';
import { Home, Save, X } from 'lucide-react';
import {
  Badge,
  Button,
  Divider,
  RoleBadge,
  Select,
  Switch,
  TextInput,
  useToast,
} from '../ui';
import { FormField, FormSection } from '../form';
import {
  fetchTreeData,
  setHomePerson,
  TreePersonNodeDto,
  updateCurrentUserProfile,
  UserDto,
} from '../../api/client';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';
import { useTheme } from '../../lib/theme';
import { ThemeMode } from '../../types/common';

export interface ProfileSettingsPanelProps {
  initialUser?: UserDto | null;
  availablePeople?: TreePersonNodeDto[];
}

const FALLBACK_USER: UserDto = {
  id: 'u-1',
  email: 'admin@levshits.local',
  first_name: 'Vera',
  last_name: 'Levshits',
  display_name: 'Vera Levshits',
  role: 'admin',
  person_id: 'I0001',
  is_active: true,
  has_password: true,
  passkeys_count: 1,
  remaining_recovery_codes: 10,
  locale: 'en',
  theme: 'system',
  home_person_id: 'I0001',
  start_page: 'dashboard',
  digest_opt_in: false,
  created_at: '2026-10-26T00:00:00Z',
  last_login_at: '2026-10-26T12:00:00Z',
};

export const ProfileSettingsPanel: React.FC<ProfileSettingsPanelProps> = ({
  initialUser,
  availablePeople,
}) => {
  const { t, locale, setLocale } = useI18n();
  const { theme, setTheme } = useTheme();
  const { user, updateUser } = useAuth();
  const { addToast } = useToast();

  const currentUser = initialUser ?? user ?? FALLBACK_USER;

  const [firstName, setFirstName] = useState(currentUser.first_name);
  const [lastName, setLastName] = useState(currentUser.last_name);
  const [prefLocale, setPrefLocale] = useState<string>(
    currentUser.locale || locale
  );
  const [prefTheme, setPrefTheme] = useState<ThemeMode>(
    (currentUser.theme as ThemeMode) || theme
  );
  const [startPage, setStartPage] = useState<'tree' | 'dashboard'>(
    currentUser.start_page || 'dashboard'
  );
  const [homePersonId, setHomePersonId] = useState<string | null>(
    currentUser.home_person_id ?? currentUser.person_id ?? null
  );
  const [digestOptIn, setDigestOptIn] = useState<boolean>(
    Boolean(currentUser.digest_opt_in)
  );
  const [people, setPeople] = useState<TreePersonNodeDto[]>(
    availablePeople ?? []
  );
  const [saving, setSaving] = useState(false);
  const [updatingHome, setUpdatingHome] = useState(false);

  useEffect(() => {
    setFirstName(currentUser.first_name);
    setLastName(currentUser.last_name);
    setStartPage(currentUser.start_page || 'dashboard');
    setHomePersonId(currentUser.home_person_id ?? currentUser.person_id ?? null);
    setDigestOptIn(Boolean(currentUser.digest_opt_in));
  }, [
    currentUser.first_name,
    currentUser.last_name,
    currentUser.start_page,
    currentUser.home_person_id,
    currentUser.person_id,
    currentUser.digest_opt_in,
  ]);

  useEffect(() => {
    if (availablePeople) {
      setPeople(availablePeople);
      return;
    }
    let cancelled = false;
    fetchTreeData()
      .then((res) => {
        if (!cancelled) setPeople(res.nodes);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [availablePeople]);

  const matchedHomePerson = people.find(
    (p) => p.id === homePersonId || p.gid === homePersonId
  );
  const homePersonDisplay = matchedHomePerson
    ? `${[
        matchedHomePerson.primary_name.given_name,
        matchedHomePerson.primary_name.patronymic,
        matchedHomePerson.primary_name.surname,
      ]
        .filter(Boolean)
        .join(' ')} (${matchedHomePerson.gid})`
    : homePersonId;

  const handleSetOrClearHomePerson = async (nextPersonId: string | null) => {
    if (updatingHome) return;
    setUpdatingHome(true);
    try {
      const updated = await setHomePerson(nextPersonId);
      setHomePersonId(updated.home_person_id);
      updateUser(updated);
      addToast({
        type: 'success',
        title: nextPersonId
          ? t('tree.homePersonUpdatedToast')
          : t('settings.profile.homePersonClearedToast'),
      });
    } catch {
      // Allow local state update in offline/preview mode
      setHomePersonId(nextPersonId);
    } finally {
      setUpdatingHome(false);
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (saving) return;
    setSaving(true);
    try {
      const updated = await updateCurrentUserProfile({
        first_name: firstName.trim(),
        last_name: lastName.trim(),
        locale: prefLocale,
        theme: prefTheme,
        start_page: startPage,
        home_person_id: homePersonId,
        digest_opt_in: digestOptIn,
      });
      updateUser(updated);
      if (prefLocale === 'en' || prefLocale === 'ru') {
        setLocale(prefLocale);
      }
      setTheme(prefTheme);
      addToast({
        type: 'success',
        title: t('settings.profile.savedToast'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('common.errorLoading'),
        description: err instanceof Error ? err.message : undefined,
      });
    } finally {
      setSaving(false);
    }
  };

  const personOptions = [
    { value: '', label: t('settings.profile.homePersonNoneOption') },
    ...people.map((p) => ({
      value: p.id,
      label: `${[
        p.primary_name.given_name,
        p.primary_name.patronymic,
        p.primary_name.surname,
      ]
        .filter(Boolean)
        .join(' ')} (${p.gid})`,
    })),
  ];

  return (
    <form onSubmit={handleSave} className="space-y-6">
      <FormSection
        title={t('settings.profile.title')}
        description={t('settings.profile.subtitle')}
      >
        <div className="flex items-center justify-between rounded-xl border border-border bg-canvas px-4 py-3">
          <div>
            <div className="text-xs text-secondary">
              {t('settings.profile.emailLabel')}
            </div>
            <div className="font-mono text-sm font-medium text-primary">
              {currentUser.email}
            </div>
          </div>
          <RoleBadge role={currentUser.role} />
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <FormField
            name="first_name"
            label={t('settings.profile.firstNameLabel')}
            required
          >
            <TextInput
              value={firstName}
              onChange={(e) => setFirstName(e.target.value)}
              required
            />
          </FormField>

          <FormField
            name="last_name"
            label={t('settings.profile.lastNameLabel')}
            required
          >
            <TextInput
              value={lastName}
              onChange={(e) => setLastName(e.target.value)}
              required
            />
          </FormField>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <FormField
            name="locale"
            label={t('settings.profile.localeLabel')}
          >
            <Select
              ariaLabel={t('settings.profile.localeLabel')}
              value={prefLocale}
              onChange={(val) => setPrefLocale(val)}
              options={[
                { value: 'en', label: t('settings.general.localeEn') },
                { value: 'ru', label: t('settings.general.localeRu') },
              ]}
            />
          </FormField>

          <FormField
            name="theme"
            label={t('settings.profile.themeLabel')}
          >
            <Select
              ariaLabel={t('settings.profile.themeLabel')}
              value={prefTheme}
              onChange={(val) => setPrefTheme(val as ThemeMode)}
              options={[
                { value: 'light', label: t('common.lightMode') },
                { value: 'dark', label: t('common.darkMode') },
                { value: 'system', label: t('common.systemMode') },
              ]}
            />
          </FormField>

          <FormField
            name="start_page"
            label={t('settings.profile.startPageLabel')}
            hint={t('settings.profile.startPageHint')}
          >
            <Select
              ariaLabel={t('settings.profile.startPageLabel')}
              value={startPage}
              onChange={(val) => setStartPage(val as 'tree' | 'dashboard')}
              options={[
                {
                  value: 'dashboard',
                  label: t('settings.profile.startPageDashboard'),
                },
                {
                  value: 'tree',
                  label: t('settings.profile.startPageTree'),
                },
              ]}
            />
          </FormField>
        </div>

        <Divider />

        {/* Home Person Indicator & Selector (16-dashboard-home.md §2) */}
        <div className="rounded-xl border border-border bg-canvas p-4 space-y-3">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
            <div className="space-y-0.5">
              <div className="flex items-center gap-2">
                <Home className="h-4 w-4 text-brand shrink-0" />
                <span className="text-sm font-semibold text-primary">
                  {t('settings.profile.homePersonLabel')}
                </span>
                {homePersonDisplay ? (
                  <Badge variant="brand" size="sm">
                    {homePersonDisplay}
                  </Badge>
                ) : (
                  <Badge variant="neutral" size="sm">
                    {t('settings.profile.homePersonNotSet')}
                  </Badge>
                )}
              </div>
              <p className="text-xs text-secondary">
                {t('settings.profile.homePersonHint')}
              </p>
            </div>

            {homePersonId && (
              <Button
                type="button"
                size="sm"
                variant="secondary"
                disabled={updatingHome}
                leftIcon={<X className="h-3.5 w-3.5" />}
                onClick={() => void handleSetOrClearHomePerson(null)}
              >
                {t('settings.profile.clearHomePersonBtn')}
              </Button>
            )}
          </div>

          {people.length > 0 && (
            <FormField
              name="home_person_id"
              label={t('settings.profile.selectHomePersonLabel')}
            >
              <Select
                ariaLabel={t('settings.profile.selectHomePersonLabel')}
                value={homePersonId ?? ''}
                onChange={(val) =>
                  void handleSetOrClearHomePerson(val || null)
                }
                options={personOptions}
              />
            </FormField>
          )}
        </div>

        <Switch
          checked={digestOptIn}
          onCheckedChange={setDigestOptIn}
          label={t('settings.profile.digestOptInLabel')}
          description={t('settings.profile.digestOptInDesc')}
        />
      </FormSection>

      <div className="flex justify-end">
        <Button
          type="submit"
          variant="primary"
          loading={saving}
          disabled={saving || !firstName.trim() || !lastName.trim()}
          leftIcon={<Save className="h-4 w-4" />}
        >
          {t('settings.profile.saveBtn')}
        </Button>
      </div>
    </form>
  );
};
