import React from 'react';
import { z } from 'zod';
import { Form, useForm } from '../form/form';
import { FormField } from '../form/form-field';
import { FormActions } from '../form/form-actions';
import { GenDateInput } from '../form/gen-date-input';
import { Select } from '../ui/select';
import { TextInput } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { Combobox } from '../ui/combobox';
import { Switch } from '../ui/switch';
import { FileDropzone } from '../ui/file-dropzone';
import { PlacePicker } from './place-picker';
import { EventType } from '../../types/common';
import { GenDate } from '../../types/gendate';
import { useI18n } from '../../i18n';

const EVENT_TYPES: EventType[] = [
  'birth',
  'baptism',
  'education',
  'occupation',
  'military',
  'marriage',
  'residence',
  'emigration',
  'immigration',
  'census',
  'death',
  'burial',
  'custom',
];

const SUGGESTED_SOURCES = [
  {
    value: 'Метрическая книга Витебской синагоги, 1912 г., л. 34',
    label: 'Метрическая книга Витебской синагоги, 1912 г., л. 34',
  },
  {
    value: 'ЦАМО РФ, ф. 33, оп. 686044, д. 1912',
    label: 'ЦАМО РФ, ф. 33, оп. 686044, д. 1912',
  },
  {
    value: 'Воспоминания бабушки Веры Ивановны Левшиц',
    label: 'Воспоминания бабушки Веры Ивановны Левшиц',
  },
];

export interface EventFormValues extends Record<string, unknown> {
  event_type: EventType;
  custom_type: string;
  date: GenDate | null;
  place_id: string | null;
  description: string;
  source_text: string;
  is_private: boolean;
}

export interface EventFormProps {
  initialValues?: Partial<EventFormValues>;
  onSubmit: (values: EventFormValues) => void;
  onCancel?: () => void;
  externalServerErrors?: Record<string, string>;
  externalSoftWarning?: string | null;
  className?: string;
}

export const EventForm: React.FC<EventFormProps> = ({
  initialValues,
  onSubmit,
  onCancel,
  externalServerErrors,
  externalSoftWarning,
  className,
}) => {
  const { t } = useI18n();

  const schema = z.object({
    event_type: z.string().min(1),
    description: z.string().min(2, { message: t('common.required') }),
  });

  const form = useForm<EventFormValues>({
    defaultValues: {
      event_type: initialValues?.event_type ?? 'military',
      custom_type: initialValues?.custom_type ?? '',
      date: initialValues?.date ?? {
        mod: 'between',
        start: { y: 1941, m: null, d: null },
        end: { y: 1945, m: null, d: null },
        cal: 'gregorian',
        text: 'между 1941 и 1945 гг.',
      },
      place_id: initialValues?.place_id ?? 'p-1',
      description:
        initialValues?.description ??
        'Служба в 42-м отдельном инженерно-сапёрном батальоне',
      source_text:
        initialValues?.source_text ?? 'ЦАМО РФ, ф. 33, оп. 686044, д. 1912',
      is_private: initialValues?.is_private ?? false,
    },
    schema,
  });

  React.useEffect(() => {
    if (externalServerErrors) {
      form.setServerErrors(externalServerErrors);
    }
  }, [externalServerErrors]);

  React.useEffect(() => {
    if (externalSoftWarning !== undefined) {
      form.setFieldWarning('date', externalSoftWarning);
    }
  }, [externalSoftWarning]);

  return (
    <Form form={form} onSubmit={onSubmit} className={className}>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <FormField name="event_type" label={t('domain.eventForm.typeLabel')} required>
          {({ id }) => (
            <Select<EventType>
              id={id}
              value={form.values.event_type}
              onChange={(v) => form.setValue('event_type', v)}
              options={EVENT_TYPES.map((et) => ({
                value: et,
                label: t(`domain.events.${et}`),
              }))}
            />
          )}
        </FormField>

        {form.values.event_type === 'custom' && (
          <FormField name="custom_type" label={t('domain.eventForm.customTypeLabel')} required>
            {({ id }) => (
              <TextInput
                id={id}
                value={form.values.custom_type}
                onChange={(e) => form.setValue('custom_type', e.target.value)}
              />
            )}
          </FormField>
        )}
      </div>

      <FormField name="date" label={t('domain.eventForm.dateLabel')} optionalText>
        {({ id, invalid }) => (
          <GenDateInput
            id={id}
            invalid={invalid}
            value={form.values.date}
            onChange={(res) => {
              form.setValue('date', res.value);
              if (res.value.start?.y && res.value.start.y < 1912) {
                form.setFieldWarning('date', t('domain.eventForm.softWarningChronology'));
              } else {
                form.setFieldWarning('date', null);
              }
            }}
          />
        )}
      </FormField>

      <FormField name="place_id" label={t('domain.eventForm.placeLabel')} optionalText>
        {() => (
          <PlacePicker
            value={form.values.place_id}
            onChange={(pid) => form.setValue('place_id', pid)}
          />
        )}
      </FormField>

      <FormField name="description" label={t('domain.eventForm.descriptionLabel')} required>
        {({ id, invalid }) => (
          <Textarea
            id={id}
            invalid={invalid}
            value={form.values.description}
            placeholder={t('domain.eventForm.descriptionPlaceholder')}
            onChange={(e) => form.setValue('description', e.target.value)}
          />
        )}
      </FormField>

      <FormField name="source_text" label={t('domain.eventForm.sourceLabel')} optionalText>
        {() => (
          <Combobox
            allowFreeText
            value={form.values.source_text}
            options={SUGGESTED_SOURCES}
            placeholder={t('domain.eventForm.sourcePlaceholder')}
            onChange={(val) => form.setValue('source_text', val ?? '')}
          />
        )}
      </FormField>

      <FormField name="scan" label={t('domain.eventForm.scanLabel')} optionalText>
        {() => <FileDropzone maxFiles={3} maxSizeMb={15} />}
      </FormField>

      <Switch
        checked={form.values.is_private}
        onCheckedChange={(c) => form.setValue('is_private', c)}
        label={t('domain.eventForm.privateLabel')}
        description={t('domain.eventForm.privateDescription')}
      />

      <FormActions onCancel={onCancel} />
    </Form>
  );
};
