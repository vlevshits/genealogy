import React, { useState, useEffect } from 'react';
import { Lock, Save } from 'lucide-react';
import {
  Alert,
  Button,
  Divider,
  Select,
  Switch,
  TextInput,
  useToast,
} from '../ui';
import { FormField, FormSection } from '../form';
import {
  AdminSystemResponse,
  InstanceSettingsDto,
  updateAdminSettings,
} from '../../api/client';
import { useI18n } from '../../i18n';
import { SystemDiagnosticsCard } from './SystemDiagnosticsCard';

export interface GeneralSettingsPanelProps {
  settings: InstanceSettingsDto;
  system: AdminSystemResponse;
  onSettingsUpdated?: (updated: InstanceSettingsDto) => void;
  onInstanceTitleChange?: (nextTitle: string) => void;
}

export const GeneralSettingsPanel: React.FC<GeneralSettingsPanelProps> = ({
  settings,
  system,
  onSettingsUpdated,
  onInstanceTitleChange,
}) => {
  const { t, setLocale } = useI18n();
  const { addToast } = useToast();

  const [treeTitle, setTreeTitle] = useState<string>(settings.tree_title);
  const [defaultLocale, setDefaultLocale] = useState<string>(
    settings.default_locale
  );
  const [enablePublicAccess, setEnablePublicAccess] = useState<boolean>(
    settings.enable_public_access
  );
  const [livingThreshold, setLivingThreshold] = useState<number>(
    settings.living_person_age_threshold
  );
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setTreeTitle(settings.tree_title);
    setDefaultLocale(settings.default_locale);
    setEnablePublicAccess(settings.enable_public_access);
    setLivingThreshold(settings.living_person_age_threshold);
  }, [settings]);

  const isLocked = settings.settings_locked_from_env;

  const thresholdOptions = [90, 100, 105, 110, 115, 120].map((yrs) => ({
    value: String(yrs),
    label: t('settings.general.yearsOption', { years: yrs }),
  }));

  const handleSaveSettings = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isLocked || saving) return;

    setSaving(true);
    try {
      const updated = await updateAdminSettings({
        tree_title: treeTitle.trim(),
        default_locale: defaultLocale,
        enable_public_access: enablePublicAccess,
        living_person_age_threshold: livingThreshold,
      });
      onSettingsUpdated?.(updated);
      onInstanceTitleChange?.(updated.tree_title);
      if (updated.default_locale === 'en' || updated.default_locale === 'ru') {
        setLocale(updated.default_locale);
      }
      addToast({
        type: 'success',
        title: t('settings.general.savedSuccess'),
        description: t('settings.general.savedDesc'),
      });
    } catch (err) {
      addToast({
        type: 'error',
        title: t('settings.general.saveError'),
        description: err instanceof Error ? err.message : undefined,
      });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      {isLocked && (
        <Alert
          variant="info"
          icon={<Lock className="h-4 w-4 text-brand shrink-0 mt-0.5" />}
          title={t('settings.general.lockedTitle')}
        >
          {t('settings.general.lockedDesc')}
        </Alert>
      )}

      <form onSubmit={handleSaveSettings} className="space-y-6">
        <FormSection
          title={t('settings.general.sectionTitle')}
          description={t('settings.general.sectionDesc')}
        >
          <FormField
            name="tree_title"
            label={t('settings.general.treeTitleLabel')}
            required
            hint={
              isLocked
                ? t('settings.general.lockedTitle')
                : t('settings.general.treeTitleHint')
            }
          >
            <TextInput
              value={treeTitle}
              onChange={(e) => setTreeTitle(e.target.value)}
              disabled={isLocked}
              readOnly={isLocked}
              maxLength={120}
            />
          </FormField>

          <FormField
            name="default_locale"
            label={t('settings.general.defaultLocaleLabel')}
            hint={
              isLocked
                ? t('settings.general.lockedTitle')
                : t('settings.general.defaultLocaleHint')
            }
          >
            <Select
              ariaLabel={t('settings.general.defaultLocaleLabel')}
              value={defaultLocale}
              onChange={(val) => setDefaultLocale(val)}
              disabled={isLocked}
              options={[
                { value: 'en', label: t('settings.general.localeEn') },
                { value: 'ru', label: t('settings.general.localeRu') },
              ]}
            />
          </FormField>
        </FormSection>

        <FormSection
          title={t('settings.general.privacySectionTitle')}
          description={t('settings.general.privacySectionDesc')}
        >
          <Switch
            checked={enablePublicAccess}
            onCheckedChange={setEnablePublicAccess}
            disabled={isLocked}
            label={t('settings.general.enablePublicLabel')}
            description={t('settings.general.enablePublicDesc')}
          />

          <Divider />

          <FormField
            name="living_person_age_threshold"
            label={t('settings.general.livingThresholdLabel')}
            hint={
              isLocked
                ? t('settings.general.lockedTitle')
                : t('settings.general.livingThresholdHint')
            }
          >
            <Select
              ariaLabel={t('settings.general.livingThresholdLabel')}
              value={String(livingThreshold)}
              onChange={(val) => setLivingThreshold(Number(val))}
              disabled={isLocked}
              options={thresholdOptions}
            />
          </FormField>
        </FormSection>

        <div className="flex items-center justify-end gap-3">
          <Button
            type="submit"
            variant="primary"
            loading={saving}
            disabled={isLocked || saving || treeTitle.trim().length === 0}
            leftIcon={<Save className="h-4 w-4" />}
          >
            {t('settings.general.saveButton')}
          </Button>
        </div>
      </form>

      <SystemDiagnosticsCard system={system} />
    </div>
  );
};
