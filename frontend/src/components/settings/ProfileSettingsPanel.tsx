import React, { useState, useEffect } from 'react';
import { Save } from 'lucide-react';
import {
  Button,
  RoleBadge,
  Select,
  TextInput,
  useToast,
} from '../ui';
import { FormField, FormSection } from '../form';
import { updateCurrentUserProfile, UserDto } from '../../api/client';
import { useAuth } from '../../lib/auth-context';
import { useI18n } from '../../i18n';
import { useTheme } from '../../lib/theme';
import { ThemeMode } from '../../types/common';

export interface ProfileSettingsPanelProps {
  initialUser?: UserDto | null;
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
  created_at: '2026-10-26T00:00:00Z',
  last_login_at: '2026-10-26T12:00:00Z',
};

export const ProfileSettingsPanel: React.FC<ProfileSettingsPanelProps> = ({
  initialUser,
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
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setFirstName(currentUser.first_name);
    setLastName(currentUser.last_name);
  }, [currentUser.first_name, currentUser.last_name]);

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

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
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
        </div>
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
