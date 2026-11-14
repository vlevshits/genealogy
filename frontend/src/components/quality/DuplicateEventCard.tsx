import React from 'react';
import {
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  GitMerge,
  SlidersHorizontal,
} from 'lucide-react';
import { Badge, Button, Card } from '../ui';
import type { DuplicateEventGroupDto } from '../../api/client';
import { normalizeEventType } from '../../lib/domain-enums';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export type ConflictFieldKey = 'date' | 'place' | 'description' | 'source';

export interface DuplicateEventCardProps {
  group: DuplicateEventGroupDto;
  busy?: boolean;
  highlighted?: boolean;
  onNavigate?: (path: string) => void;
  onDismiss: (group: DuplicateEventGroupDto) => void;
  onCustomize: (group: DuplicateEventGroupDto) => void;
  onMerge: (group: DuplicateEventGroupDto) => void;
}

export function normalizeConflictFields(
  conflictFields: string[] | undefined
): Set<ConflictFieldKey> {
  const result = new Set<ConflictFieldKey>();
  for (const raw of conflictFields ?? []) {
    const key = raw.trim().toLowerCase();
    if (key === 'date' || key === 'date_text') result.add('date');
    else if (key === 'place' || key === 'place_full_name' || key === 'place_name')
      result.add('place');
    else if (key === 'description') result.add('description');
    else if (key === 'source' || key === 'source_text') result.add('source');
  }
  return result;
}

function formatEventTypeLabel(
  eventType: string,
  customType: string | null | undefined,
  t: (path: string, vars?: Record<string, string | number>) => string
): string {
  const overrideKey = `events.types.${eventType}`;
  const overrideVal = t(overrideKey);
  if (overrideVal !== overrideKey) return overrideVal;

  const normalized = normalizeEventType(eventType);
  if (normalized) {
    const domainKey = `domain.events.${normalized}`;
    const domainVal = t(domainKey);
    if (domainVal !== domainKey) return domainVal;
  }

  return customType?.trim() || eventType;
}

function formatGroupRationale(
  group: DuplicateEventGroupDto,
  eventTypeLabel: string,
  conflictSet: Set<ConflictFieldKey>,
  t: (path: string, vars?: Record<string, string | number>) => string
): string {
  const subject = group.partner
    ? `${group.person.display_name} & ${group.partner.display_name}`
    : group.person.display_name;

  if (group.has_conflict || conflictSet.size > 0) {
    const orderedKeys: ConflictFieldKey[] = [
      'date',
      'place',
      'description',
      'source',
    ];
    const fieldLabels = orderedKeys
      .filter((key) => conflictSet.has(key))
      .map((key) => t(`quality.duplicateEvents.fieldLabels.${key}`));
    if (fieldLabels.length > 0) {
      return t('quality.duplicateEvents.rationaleConflict', {
        count: group.events.length,
        eventType: eventTypeLabel,
        subject,
        fields: fieldLabels.join(', '),
      });
    }
  }

  return t('quality.duplicateEvents.rationaleMatch', {
    count: group.events.length,
    eventType: eventTypeLabel,
    subject,
  });
}

const FieldRow: React.FC<{
  label: string;
  value: string;
  isConflict: boolean;
  conflictTag: string;
  emphasizeValue?: boolean;
}> = ({ label, value, isConflict, conflictTag, emphasizeValue = false }) => (
  <div
    className={cn(
      'flex min-w-0 flex-wrap items-baseline gap-1.5',
      isConflict &&
        'rounded-md border border-warning/40 bg-warning/10 px-2 py-1'
    )}
  >
    <dt className="shrink-0 text-tertiary">{label}:</dt>
    <dd
      className={cn(
        'min-w-0 break-words',
        emphasizeValue ? 'font-medium text-primary' : 'text-primary'
      )}
    >
      {value}
    </dd>
    {isConflict && (
      <span className="inline-flex shrink-0 items-center gap-1 rounded bg-warning/15 px-1.5 py-0.5 text-[10px] font-semibold text-warning">
        <AlertTriangle className="h-3 w-3 shrink-0" aria-hidden="true" />
        <span>{conflictTag}</span>
      </span>
    )}
  </div>
);

export const DuplicateEventCard: React.FC<DuplicateEventCardProps> = ({
  group,
  busy = false,
  highlighted = false,
  onNavigate,
  onDismiss,
  onCustomize,
  onMerge,
}) => {
  const { t } = useI18n();
  const summaryId = `dup-event-summary-${group.id}`;
  const firstCustomType = group.events.find((ev) => ev.custom_type)?.custom_type;
  const eventTypeLabel = formatEventTypeLabel(
    group.event_type,
    firstCustomType,
    t
  );
  const conflictSet = normalizeConflictFields(group.conflict_fields);
  const conflictTag = t('quality.duplicateEvents.conflictTag');
  const rationaleText = formatGroupRationale(
    group,
    eventTypeLabel,
    conflictSet,
    t
  );

  const navigateToPerson = (personId: string) => {
    const target = `/persons/${personId}`;
    if (onNavigate) {
      onNavigate(target);
      return;
    }
    window.location.href = target;
  };

  const dismissBtn = (
    <Button
      size="sm"
      variant="secondary"
      disabled={busy}
      className="flex-1 sm:flex-initial"
      aria-describedby={summaryId}
      data-testid={`dismiss-duplicate-event-btn-${group.id}`}
      leftIcon={<CheckCircle2 className="h-3.5 w-3.5" />}
      onClick={() => onDismiss(group)}
    >
      {t('quality.duplicateEvents.looksCorrectBtn')}
    </Button>
  );

  const customizeBtn = (
    <Button
      size={group.has_conflict ? 'md' : 'sm'}
      variant={group.has_conflict ? 'primary' : 'secondary'}
      disabled={busy}
      className={group.has_conflict ? 'w-full sm:w-auto' : 'flex-1 sm:flex-initial'}
      aria-describedby={summaryId}
      data-testid={`customize-duplicate-event-btn-${group.id}`}
      leftIcon={<SlidersHorizontal className="h-3.5 w-3.5" />}
      onClick={() => onCustomize(group)}
    >
      {t('quality.duplicateEvents.customizeBtn')}
    </Button>
  );

  const mergeBtn = (
    <Button
      size={group.has_conflict ? 'sm' : 'md'}
      variant={group.has_conflict ? 'secondary' : 'primary'}
      loading={busy}
      className={group.has_conflict ? 'flex-1 sm:flex-initial' : 'w-full sm:w-auto'}
      aria-describedby={summaryId}
      data-testid={`merge-duplicate-event-btn-${group.id}`}
      leftIcon={<GitMerge className="h-3.5 w-3.5" />}
      onClick={() => onMerge(group)}
    >
      {t('quality.duplicateEvents.mergeBtn')}
    </Button>
  );

  return (
    <Card
      id={`duplicate-event-card-${group.id}`}
      tabIndex={-1}
      data-testid={`duplicate-event-card-${group.id}`}
      className={cn(
        'p-4 sm:p-5 focus:outline-none',
        highlighted && 'border-brand ring-2 ring-brand/40'
      )}
    >
      <div className="mb-3 flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0 max-w-full space-y-1.5">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="brand">{eventTypeLabel}</Badge>
            <Badge variant="neutral">
              {t('quality.duplicateEvents.eventsCountBadge', {
                count: group.events.length,
              })}
            </Badge>
            {group.has_conflict ? (
              <Badge
                variant="warning"
                icon={<AlertTriangle className="h-3 w-3" />}
              >
                {t('quality.duplicateEvents.reviewConflictBadge')}
              </Badge>
            ) : (
              <Badge
                variant="success"
                icon={<CheckCircle2 className="h-3 w-3" />}
              >
                {t('quality.duplicateEvents.readyToMergeBadge')}
              </Badge>
            )}
          </div>

          <div
            id={summaryId}
            className="flex min-w-0 max-w-full flex-wrap items-center gap-1.5 text-sm font-medium text-primary"
          >
            <Button
              variant="link"
              size="xs"
              className="min-h-8 h-auto max-w-full whitespace-normal text-left px-0 py-1 text-sm font-semibold text-primary"
              onClick={() => navigateToPerson(group.person.id)}
            >
              {group.person.display_name}
            </Button>
            {group.person.gid && (
              <span className="font-mono text-xs text-tertiary">
                ({group.person.gid})
              </span>
            )}
            {group.partner && (
              <>
                <span className="text-tertiary">&amp;</span>
                <Button
                  variant="link"
                  size="xs"
                  className="min-h-8 h-auto max-w-full whitespace-normal text-left px-0 py-1 text-sm font-semibold text-primary"
                  onClick={() => navigateToPerson(group.partner!.id)}
                >
                  {group.partner.display_name}
                </Button>
                {group.partner.gid && (
                  <span className="font-mono text-xs text-tertiary">
                    ({group.partner.gid})
                  </span>
                )}
              </>
            )}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
        <div className="min-w-0 space-y-2 rounded-xl border border-border bg-surface-muted p-3">
          <div className="text-[11px] font-semibold uppercase tracking-wide text-tertiary">
            {t('quality.duplicateEvents.recordedColumnTitle')}
          </div>
          <div className="space-y-2">
            {group.events.map((ev) => {
              const isPrimary = ev.id === group.proposed.keep_event_id;
              return (
                <div
                  key={ev.id}
                  className="space-y-1.5 rounded-lg border border-border bg-surface p-2.5"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex flex-wrap items-center gap-1.5">
                      <Badge variant="neutral" className="font-mono">
                        {ev.gid}
                      </Badge>
                      {isPrimary && (
                        <Badge variant="brand">
                          {t('quality.duplicateEvents.primaryBadge')}
                        </Badge>
                      )}
                    </div>
                    <span className="text-xs text-tertiary">
                      {t('quality.duplicateEvents.mediaCount', {
                        count: ev.media_count,
                      })}
                    </span>
                  </div>

                  <dl className="space-y-1 text-xs text-secondary">
                    <FieldRow
                      label={t('quality.duplicateEvents.dateLabel')}
                      value={ev.date_text || '—'}
                      isConflict={conflictSet.has('date')}
                      conflictTag={conflictTag}
                    />
                    <FieldRow
                      label={t('quality.duplicateEvents.placeLabel')}
                      value={ev.place_full_name || '—'}
                      isConflict={conflictSet.has('place')}
                      conflictTag={conflictTag}
                    />
                    {(ev.description || conflictSet.has('description')) && (
                      <FieldRow
                        label={t('quality.duplicateEvents.descriptionLabel')}
                        value={ev.description || '—'}
                        isConflict={conflictSet.has('description')}
                        conflictTag={conflictTag}
                      />
                    )}
                    {(ev.source_text || conflictSet.has('source')) && (
                      <FieldRow
                        label={t('quality.duplicateEvents.sourceLabel')}
                        value={ev.source_text || '—'}
                        isConflict={conflictSet.has('source')}
                        conflictTag={conflictTag}
                      />
                    )}
                  </dl>
                </div>
              );
            })}
          </div>
        </div>

        <div className="min-w-0 flex flex-col justify-between gap-2 rounded-xl border border-border bg-surface-muted p-3">
          <div className="min-w-0 space-y-2">
            <div className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wide text-tertiary">
              <ArrowRight className="h-3.5 w-3.5 text-brand" />
              <span>{t('quality.duplicateEvents.proposedColumnTitle')}</span>
            </div>
            <div className="space-y-2 rounded-lg border border-border bg-surface p-2.5">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <Badge variant="brand" className="font-mono">
                  {group.proposed.keep_event_gid}
                </Badge>
                <span className="text-xs font-medium text-secondary">
                  {t('quality.duplicateEvents.combinedMediaCount', {
                    count: group.proposed.combined_media_count,
                  })}
                </span>
              </div>

              <dl className="space-y-1 text-xs text-secondary">
                <FieldRow
                  label={t('quality.duplicateEvents.dateLabel')}
                  value={group.proposed.date_text || '—'}
                  isConflict={conflictSet.has('date')}
                  conflictTag={conflictTag}
                  emphasizeValue
                />
                <FieldRow
                  label={t('quality.duplicateEvents.placeLabel')}
                  value={group.proposed.place_full_name || '—'}
                  isConflict={conflictSet.has('place')}
                  conflictTag={conflictTag}
                  emphasizeValue
                />
                <FieldRow
                  label={t('quality.duplicateEvents.descriptionLabel')}
                  value={group.proposed.description || '—'}
                  isConflict={conflictSet.has('description')}
                  conflictTag={conflictTag}
                />
                <FieldRow
                  label={t('quality.duplicateEvents.sourceLabel')}
                  value={group.proposed.source_text || '—'}
                  isConflict={conflictSet.has('source')}
                  conflictTag={conflictTag}
                />
              </dl>
            </div>
          </div>

          <p className="text-xs text-tertiary">
            {t('quality.duplicateEvents.zeroDataLossHint')}
          </p>
        </div>
      </div>

      <div className="mt-3 flex flex-wrap items-center justify-between gap-2 border-t border-border pt-3">
        <p className="min-w-0 max-w-full text-xs text-secondary break-words">
          {rationaleText}
        </p>
        <div className="flex w-full min-w-0 max-w-full flex-wrap items-center justify-end gap-2 sm:w-auto">
          {dismissBtn}
          {group.has_conflict ? (
            <>
              {mergeBtn}
              {customizeBtn}
            </>
          ) : (
            <>
              {customizeBtn}
              {mergeBtn}
            </>
          )}
        </div>
      </div>
    </Card>
  );
};
