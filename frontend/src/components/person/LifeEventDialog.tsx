import React, { useEffect, useState } from 'react';
import { Save } from 'lucide-react';
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
import { FormField, GenDateInput } from '../form';
import {
  createLifeEvent,
  fetchEventSources,
  LifeEventDto,
  LifeEventType,
  updateLifeEvent,
} from '../../api/client';
import { GenDate } from '../../types/gendate';
import { getLocalizedErrorMessage } from '../../lib/error-formatter';
import { normalizeEventType } from '../../lib/domain-enums';
import { useI18n } from '../../i18n';

export interface LifeEventDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  individualId?: string | null;
  familyId?: string | null;
  initialEvent?: LifeEventDto | null;
  initialSourceSuggestions?: string[];
  onEventSaved?: (saved: LifeEventDto) => void;
}

const DEFAULT_SOURCES = [
  'Метрическая книга, 1912, л. 34',
  'ГАРФ, ф. 102, оп. 1, д. 18',
  'Parish register, 1898, entry 42',
];

export const LifeEventDialog: React.FC<LifeEventDialogProps> = ({
  open,
  onOpenChange,
  individualId,
  familyId,
  initialEvent,
  initialSourceSuggestions,
  onEventSaved,
}) => {
  const { t } = useI18n();
  const { addToast } = useToast();

  const [eventType, setEventType] = useState<LifeEventType | string>(
    initialEvent?.event_type ? normalizeEventType(initialEvent.event_type) : 'birth'
  );
  const [customType, setCustomType] = useState<string>(
    initialEvent?.custom_type ?? ''
  );
  const [dateRaw, setDateRaw] = useState<string>(
    initialEvent?.date_text ?? initialEvent?.date?.text ?? ''
  );
  const [parsedDate, setParsedDate] = useState<GenDate | null>(
    initialEvent?.date ?? null
  );
  const [placePath, setPlacePath] = useState<string>(
    initialEvent?.place_full_name ?? ''
  );
  const [description, setDescription] = useState<string>(
    initialEvent?.description ?? ''
  );
  const [sourceText, setSourceText] = useState<string>(
    initialEvent?.source_text ?? ''
  );
  const [isPrivate, setIsPrivate] = useState<boolean>(
    initialEvent?.is_private ?? false
  );
  const [sourceSuggestions, setSourceSuggestions] = useState<string[]>(
    initialSourceSuggestions ?? DEFAULT_SOURCES
  );
  const [saving, setSaving] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    if (initialSourceSuggestions) {
      setSourceSuggestions(initialSourceSuggestions);
      return;
    }
    if (!open) return;
    let cancelled = false;
    fetchEventSources()
      .then((res) => {
        if (!cancelled && res.items.length > 0) {
          setSourceSuggestions(res.items);
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [open, initialSourceSuggestions]);

  useEffect(() => {
    setEventType(
      initialEvent?.event_type
        ? normalizeEventType(initialEvent.event_type)
        : 'birth'
    );
    setCustomType(initialEvent?.custom_type ?? '');
    setDateRaw(initialEvent?.date_text ?? initialEvent?.date?.text ?? '');
    setParsedDate(initialEvent?.date ?? null);
    setPlacePath(initialEvent?.place_full_name ?? '');
    setDescription(initialEvent?.description ?? '');
    setSourceText(initialEvent?.source_text ?? '');
    setIsPrivate(Boolean(initialEvent?.is_private));
    setErrorMsg(null);
  }, [initialEvent, open]);

  const eventTypeOptions = [
    { value: 'birth', label: t('domain.events.birth') },
    { value: 'baptism', label: t('domain.events.baptism') },
    { value: 'marriage', label: t('domain.events.marriage') },
    { value: 'marriage_license', label: t('domain.events.marriage_license') },
    { value: 'engagement', label: t('domain.events.engagement') },
    { value: 'divorce', label: t('domain.events.divorce') },
    { value: 'death', label: t('domain.events.death') },
    { value: 'burial', label: t('domain.events.burial') },
    { value: 'cremation', label: t('domain.events.cremation') },
    { value: 'residence', label: t('domain.events.residence') },
    { value: 'occupation', label: t('domain.events.occupation') },
    { value: 'education', label: t('domain.events.education') },
    { value: 'graduation', label: t('domain.events.graduation') },
    { value: 'military', label: t('domain.events.military') },
    { value: 'emigration', label: t('domain.events.emigration') },
    { value: 'immigration', label: t('domain.events.immigration') },
    { value: 'naturalization', label: t('domain.events.naturalization') },
    { value: 'census', label: t('domain.events.census') },
    { value: 'religion', label: t('domain.events.religion') },
    { value: 'adoption', label: t('domain.events.adoption') },
    { value: 'retirement', label: t('domain.events.retirement') },
    { value: 'custom', label: t('domain.events.custom') },
  ];

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (saving) return;
    setSaving(true);
    setErrorMsg(null);
    try {
      const payload = {
        individual_id: individualId ?? initialEvent?.individual_id ?? null,
        family_id: familyId ?? initialEvent?.family_id ?? null,
        event_type: eventType,
        custom_type: eventType === 'custom' ? customType.trim() || null : null,
        date: parsedDate,
        date_text: dateRaw.trim() || null,
        place_path: placePath.trim() || null,
        description: description.trim() || null,
        source_text: sourceText.trim() || null,
        is_private: isPrivate,
      };

      const saved = initialEvent
        ? await updateLifeEvent(initialEvent.id, {
            ...payload,
            version: initialEvent.version ?? 1,
          })
        : await createLifeEvent(payload);

      addToast({
        type: 'success',
        title: t('events.savedToast'),
      });
      onEventSaved?.(saved);
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
      title={
        initialEvent
          ? t('events.dialogTitleEdit')
          : t('events.dialogTitleAdd')
      }
      description={t('events.dialogSubtitle')}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {errorMsg && <Alert variant="danger">{errorMsg}</Alert>}

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <FormField
            name="event_type"
            label={t('events.eventTypeLabel')}
          >
            <Select
              ariaLabel={t('events.eventTypeLabel')}
              value={eventType}
              onChange={(val) => setEventType(val)}
              options={eventTypeOptions}
            />
          </FormField>

          {eventType === 'custom' && (
            <FormField
              name="custom_type"
              label={t('events.customTypeLabel')}
              required
            >
              <TextInput
                value={customType}
                onChange={(e) => setCustomType(e.target.value)}
                required
              />
            </FormField>
          )}
        </div>

        <FormField name="event_date" label={t('events.dateLabel')}>
          <GenDateInput
            value={parsedDate}
            onChange={(result) => {
              setDateRaw(result.value.text);
              setParsedDate(result.value);
            }}
          />
        </FormField>

        <FormField name="event_place" label={t('events.placeLabel')}>
          <TextInput
            value={placePath}
            onChange={(e) => setPlacePath(e.target.value)}
            placeholder={t('events.placePlaceholder')}
          />
        </FormField>

        <FormField
          name="event_description"
          label={t('events.descriptionLabel')}
          optionalText
        >
          <Textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={2}
          />
        </FormField>

        <div className="space-y-2">
          <FormField
            name="event_source"
            label={t('events.sourceLabel')}
            optionalText
          >
            <TextInput
              value={sourceText}
              onChange={(e) => setSourceText(e.target.value)}
              placeholder={t('events.sourcePlaceholder')}
            />
          </FormField>

          {sourceSuggestions.length > 0 && (
            <div className="space-y-1">
              <div className="text-[11px] font-medium text-tertiary">
                {t('events.sourceSuggestionsLabel')}:
              </div>
              <div className="flex flex-wrap gap-1.5">
                {sourceSuggestions.slice(0, 4).map((src) => (
                  <Button
                    key={src}
                    size="sm"
                    variant="secondary"
                    onClick={() => setSourceText(src)}
                  >
                    {src}
                  </Button>
                ))}
              </div>
            </div>
          )}
        </div>

        <Switch
          checked={isPrivate}
          onCheckedChange={setIsPrivate}
          label={t('events.privateEventLabel')}
        />

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
            leftIcon={<Save className="h-4 w-4" />}
          >
            {t('events.saveBtn')}
          </Button>
        </div>
      </form>
    </Dialog>
  );
};
