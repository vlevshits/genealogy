import React, { useEffect, useId, useMemo, useState } from 'react';
import { GitMerge } from 'lucide-react';
import {
  Button,
  Dialog,
  RadioGroup,
  TextInput,
  Textarea,
} from '../ui';
import { FormField } from '../form';
import type {
  DuplicateEventGroupDto,
  MergeDuplicateEventsRequest,
} from '../../api/client';
import { useI18n } from '../../i18n';

export interface DuplicateEventCustomizeModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  group: DuplicateEventGroupDto | null;
  saving?: boolean;
  onSubmit: (payload: MergeDuplicateEventsRequest) => void;
}

export interface CustomEventMergeFormValues {
  keepEventId: string;
  dateText: string;
  placeName: string;
  description: string;
  sourceText: string;
}

export function buildCustomEventMergePayload(
  group: DuplicateEventGroupDto,
  values: CustomEventMergeFormValues
): MergeDuplicateEventsRequest {
  const trimmedDate = values.dateText.trim();
  const trimmedPlace = values.placeName.trim();
  const trimmedDescription = values.description.trim();
  const trimmedSource = values.sourceText.trim();

  let resolvedPlaceId: string | undefined;
  if (trimmedPlace) {
    const matchingEventPlace = group.events.find(
      (ev) =>
        (ev.place_full_name ?? '').trim() === trimmedPlace &&
        Boolean(ev.place_id)
    );
    if (matchingEventPlace?.place_id) {
      resolvedPlaceId = matchingEventPlace.place_id;
    } else if (
      (group.proposed.place_full_name ?? '').trim() === trimmedPlace &&
      group.proposed.place_id
    ) {
      resolvedPlaceId = group.proposed.place_id;
    }
  }

  return {
    group_id: group.id,
    keep_event_id: values.keepEventId,
    merge_event_ids: group.events
      .filter((ev) => ev.id !== values.keepEventId)
      .map((ev) => ev.id),
    date_text: trimmedDate,
    ...(resolvedPlaceId ? { place_id: resolvedPlaceId } : {}),
    place_name: trimmedPlace,
    description: trimmedDescription,
    source_text: trimmedSource,
  };
}

function collectDistinctValues(
  values: Array<string | null | undefined>
): string[] {
  const seen = new Set<string>();
  const result: string[] = [];
  for (const val of values) {
    const trimmed = (val ?? '').trim();
    if (trimmed && !seen.has(trimmed)) {
      seen.add(trimmed);
      result.push(trimmed);
    }
  }
  return result;
}

export const DuplicateEventCustomizeModal: React.FC<
  DuplicateEventCustomizeModalProps
> = ({ open, onOpenChange, group, saving = false, onSubmit }) => {
  const { t } = useI18n();
  const formId = useId();
  const [keepEventId, setKeepEventId] = useState('');
  const [dateText, setDateText] = useState('');
  const [placeName, setPlaceName] = useState('');
  const [description, setDescription] = useState('');
  const [sourceText, setSourceText] = useState('');

  useEffect(() => {
    if (!group) return;
    setKeepEventId(group.proposed.keep_event_id || group.events[0]?.id || '');
    setDateText(group.proposed.date_text || '');
    setPlaceName(group.proposed.place_full_name || '');
    setDescription(group.proposed.description || '');
    setSourceText(group.proposed.source_text || '');
  }, [group]);

  const dateChoices = useMemo(
    () =>
      group
        ? collectDistinctValues(group.events.map((ev) => ev.date_text))
        : [],
    [group]
  );

  const placeChoices = useMemo(
    () =>
      group
        ? collectDistinctValues(group.events.map((ev) => ev.place_full_name))
        : [],
    [group]
  );

  const descriptionChoices = useMemo(
    () =>
      group
        ? collectDistinctValues(group.events.map((ev) => ev.description))
        : [],
    [group]
  );

  const sourceChoices = useMemo(
    () =>
      group
        ? collectDistinctValues(group.events.map((ev) => ev.source_text))
        : [],
    [group]
  );

  const combinedSourcesValue = useMemo(
    () => (sourceChoices.length >= 2 ? sourceChoices.join(' | ') : ''),
    [sourceChoices]
  );

  if (!group) return null;

  const eventOptions = group.events.map((ev) => {
    const summaryParts = [
      ev.place_full_name,
      ev.description,
      ev.source_text,
      t('quality.duplicateEvents.mediaCount', { count: ev.media_count }),
    ].filter(Boolean);
    return {
      value: ev.id,
      label: `${ev.gid}${ev.date_text ? ` · ${ev.date_text}` : ''}`,
      description: summaryParts.join(' · '),
      dataTestId: `customize-keep-event-${ev.id}`,
    };
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!keepEventId) return;

    onSubmit(
      buildCustomEventMergePayload(group, {
        keepEventId,
        dateText,
        placeName,
        description,
        sourceText,
      })
    );
  };

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={t('quality.duplicateEvents.customizeModalTitle')}
      description={t('quality.duplicateEvents.customizeModalDesc')}
      size="md"
      footer={
        <div className="flex flex-wrap items-center justify-end gap-2 min-w-0 max-w-full">
          <Button
            variant="ghost"
            size="md"
            onClick={() => onOpenChange(false)}
            disabled={saving}
          >
            {t('common.cancel')}
          </Button>
          <Button
            type="submit"
            form={formId}
            variant="primary"
            size="md"
            loading={saving}
            disabled={!keepEventId}
            data-testid="confirm-customize-merge-event-btn"
            leftIcon={<GitMerge className="h-4 w-4" />}
          >
            {t('quality.duplicateEvents.confirmMergeBtn')}
          </Button>
        </div>
      }
    >
      <form id={formId} onSubmit={handleSubmit} className="space-y-3">
        <fieldset className="space-y-2 rounded-xl border border-border bg-surface-muted p-2.5">
          <legend className="px-1 text-xs sm:text-sm font-medium text-primary">
            {t('quality.duplicateEvents.primaryEventSelectLabel')}
          </legend>
          <RadioGroup
            value={keepEventId}
            onChange={setKeepEventId}
            ariaLabel={t('quality.duplicateEvents.primaryEventSelectLabel')}
            options={eventOptions}
          />
        </fieldset>

        <div className="space-y-1.5">
          <FormField
            name="date_text"
            label={t('quality.duplicateEvents.dateLabel')}
          >
            <TextInput
              value={dateText}
              onChange={(e) => setDateText(e.target.value)}
              data-testid="customize-event-date-input"
              aria-label={t('quality.duplicateEvents.dateLabel')}
            />
          </FormField>
          {dateChoices.length > 1 && (
            <div className="flex flex-wrap items-center gap-2.5 pt-0.5">
              <span className="text-[11px] text-tertiary">
                {t('quality.duplicateEvents.quickFillHint')}
              </span>
              {dateChoices.map((val) => (
                <Button
                  key={val}
                  size="xs"
                  className="min-h-8 px-2.5 py-1.5 max-w-full whitespace-normal text-left h-auto"
                  variant={dateText.trim() === val ? 'primary' : 'secondary'}
                  onClick={() => setDateText(val)}
                >
                  {val}
                </Button>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-1.5">
          <FormField
            name="place_name"
            label={t('quality.duplicateEvents.placeLabel')}
          >
            <TextInput
              value={placeName}
              onChange={(e) => setPlaceName(e.target.value)}
              data-testid="customize-event-place-input"
              aria-label={t('quality.duplicateEvents.placeLabel')}
            />
          </FormField>
          {placeChoices.length > 1 && (
            <div className="flex flex-wrap items-center gap-2.5 pt-0.5">
              <span className="text-[11px] text-tertiary">
                {t('quality.duplicateEvents.quickFillHint')}
              </span>
              {placeChoices.map((val) => (
                <Button
                  key={val}
                  size="xs"
                  className="min-h-8 px-2.5 py-1.5 max-w-full whitespace-normal text-left h-auto"
                  variant={placeName.trim() === val ? 'primary' : 'secondary'}
                  onClick={() => setPlaceName(val)}
                >
                  {val}
                </Button>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-1.5">
          <FormField
            name="description"
            label={t('quality.duplicateEvents.descriptionLabel')}
          >
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              minRows={2}
              data-testid="customize-event-description-input"
              aria-label={t('quality.duplicateEvents.descriptionLabel')}
            />
          </FormField>
          {descriptionChoices.length > 1 && (
            <div className="flex flex-wrap items-center gap-2.5 pt-0.5">
              <span className="text-[11px] text-tertiary">
                {t('quality.duplicateEvents.quickFillHint')}
              </span>
              {descriptionChoices.map((val) => (
                <Button
                  key={val}
                  size="xs"
                  className="min-h-8 px-2.5 py-1.5 max-w-full whitespace-normal text-left h-auto"
                  variant={description.trim() === val ? 'primary' : 'secondary'}
                  onClick={() => setDescription(val)}
                >
                  {val}
                </Button>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-1.5">
          <FormField
            name="source_text"
            label={t('quality.duplicateEvents.sourceLabel')}
          >
            <Textarea
              value={sourceText}
              onChange={(e) => setSourceText(e.target.value)}
              minRows={2}
              data-testid="customize-event-source-input"
              aria-label={t('quality.duplicateEvents.sourceLabel')}
            />
          </FormField>
          {(sourceChoices.length > 1 || Boolean(combinedSourcesValue)) && (
            <div className="flex flex-wrap items-center gap-2.5 pt-0.5">
              <span className="text-[11px] text-tertiary">
                {t('quality.duplicateEvents.quickFillHint')}
              </span>
              {sourceChoices.map((val) => (
                <Button
                  key={val}
                  size="xs"
                  className="min-h-8 px-2.5 py-1.5 max-w-full whitespace-normal text-left h-auto"
                  variant={sourceText.trim() === val ? 'primary' : 'secondary'}
                  onClick={() => setSourceText(val)}
                >
                  {val}
                </Button>
              ))}
              {combinedSourcesValue && (
                <Button
                  size="xs"
                  className="min-h-8 px-2.5 py-1.5 max-w-full whitespace-normal text-left h-auto"
                  variant={
                    sourceText.trim() === combinedSourcesValue
                      ? 'primary'
                      : 'secondary'
                  }
                  data-testid="combine-sources-chip"
                  onClick={() => setSourceText(combinedSourcesValue)}
                >
                  {t('quality.duplicateEvents.combineSourcesBtn')}
                </Button>
              )}
            </div>
          )}
        </div>
      </form>
    </Dialog>
  );
};
