import React, { useState } from 'react';
import {
  BookOpen,
  Calendar,
  Eye,
  FileText,
  Heart,
  Lock,
  MapPin,
  PenTool,
  Plus,
  Save,
  Trash2,
  X,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  EmptyState,
  IconButton,
  MarkdownView,
  Textarea,
} from '../ui';
import {
  ProfileLifeEventDto,
  TreePersonNodeDto,
} from '../../api/client';
import { ProfileLifeMap } from './ProfileLifeMap';
import { useI18n } from '../../i18n';
import { cn } from '../../lib/utils';

export interface ProfileChronologyTabProps {
  person: TreePersonNodeDto;
  events: ProfileLifeEventDto[];
  canEdit: boolean;
  isMaskedLiving?: boolean;
  highlightedEventId?: string | null;
  onSelectEvent?: (eventId: string | null) => void;
  onAddEvent: () => void;
  onEditEvent: (event: ProfileLifeEventDto) => void;
  onDeleteEvent?: (event: ProfileLifeEventDto) => void;
  onSaveBiography?: (nextBio: string) => Promise<void>;
  onSelectRelativeId?: (relativeId: string) => void;
}

export const ProfileChronologyTab: React.FC<ProfileChronologyTabProps> = ({
  person,
  events,
  canEdit,
  isMaskedLiving = false,
  highlightedEventId = null,
  onSelectEvent,
  onAddEvent,
  onEditEvent,
  onDeleteEvent,
  onSaveBiography,
  onSelectRelativeId,
}) => {
  const { t } = useI18n();

  const initialBio = person.biography ?? person.notes ?? '';
  const [editingBio, setEditingBio] = useState(false);
  const [bioDraft, setBioDraft] = useState(initialBio);
  const [bioPreview, setBioPreview] = useState(false);
  const [savingBio, setSavingBio] = useState(false);

  const startBioEdit = () => {
    setBioDraft(person.biography ?? person.notes ?? '');
    setBioPreview(false);
    setEditingBio(true);
  };

  const handleSaveBio = async () => {
    if (!onSaveBiography) {
      setEditingBio(false);
      return;
    }
    setSavingBio(true);
    try {
      await onSaveBiography(bioDraft);
      setEditingBio(false);
    } finally {
      setSavingBio(false);
    }
  };

  const getEventLabel = (ev: ProfileLifeEventDto): string => {
    if (ev.event_type === 'custom' && ev.custom_type) {
      return ev.custom_type;
    }
    return t(`domain.events.${ev.event_type}`);
  };

  if (isMaskedLiving) {
    return (
      <Card className="p-8">
        <EmptyState
          icon={<Lock className="h-8 w-8 text-warning" />}
          title={t('profile.chronology.maskedTitle')}
          description={t('profile.chronology.maskedDesc')}
        />
      </Card>
    );
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
      {/* Left Pane (7 cols): Chronological Life Events Stream */}
      <div className="lg:col-span-7 space-y-4">
        <Card className="p-5 sm:p-6 space-y-5">
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              <Calendar className="h-4 w-4 text-brand shrink-0" />
              <h2 className="font-sans text-base font-semibold text-primary">
                {t('profile.chronology.timelineTitle')}
              </h2>
              <Badge variant="neutral" size="sm">
                {events.length}
              </Badge>
            </div>

            {canEdit && (
              <Button
                size="sm"
                variant="secondary"
                leftIcon={<Plus className="h-3.5 w-3.5" />}
                onClick={onAddEvent}
              >
                {t('profile.actions.addEvent')}
              </Button>
            )}
          </div>

          {events.length === 0 ? (
            <EmptyState
              icon={<Calendar className="h-7 w-7 text-tertiary" />}
              title={t('profile.chronology.emptyEventsTitle')}
              description={t('profile.chronology.emptyEventsDesc')}
              action={
                canEdit ? (
                  <Button
                    size="sm"
                    variant="primary"
                    leftIcon={<Plus className="h-3.5 w-3.5" />}
                    onClick={onAddEvent}
                  >
                    {t('profile.actions.addEvent')}
                  </Button>
                ) : undefined
              }
            />
          ) : (
            <div className="relative pl-5 space-y-4 before:absolute before:left-2 before:top-2 before:bottom-2 before:w-0.5 before:bg-border">
              {events.map((ev) => {
                const year = ev.date_sort
                  ? Math.floor(ev.date_sort / 10000)
                  : null;
                const dateDisplay =
                  ev.date?.text ||
                  ev.gedcom_date ||
                  (year ? String(year) : t('profile.hero.dateUnknown'));
                const scansCount = ev.scans_count ?? ev.media_ids?.length ?? 0;
                const isHighlighted = highlightedEventId === ev.id;

                const dotColor =
                  ev.event_type === 'birth' || ev.event_type === 'baptism'
                    ? 'bg-brand border-brand'
                    : ev.event_type === 'marriage'
                    ? 'bg-accent border-accent'
                    : ev.event_type === 'death' || ev.event_type === 'burial'
                    ? 'bg-secondary border-secondary'
                    : 'bg-surface border-brand';

                return (
                  <div
                    key={ev.id}
                    data-event-id={ev.id}
                    onClick={() =>
                      onSelectEvent?.(isHighlighted ? null : ev.id)
                    }
                    className={cn(
                      'relative rounded-2xl border p-4 transition-all cursor-pointer',
                      isHighlighted
                        ? 'border-brand bg-brand/5 ring-2 ring-brand/25'
                        : 'border-border bg-surface hover:border-brand/50'
                    )}
                  >
                    {/* Timeline Node Dot */}
                    <span
                      aria-hidden="true"
                      className={cn(
                        'absolute -left-[1.45rem] top-5 h-3 w-3 rounded-full border-2',
                        dotColor
                      )}
                    />

                    <div className="flex items-start justify-between gap-3">
                      <div className="space-y-1.5 min-w-0 flex-1">
                        {/* Event Type, Year & Age at Event */}
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="font-mono text-xs font-bold text-brand">
                            {year ?? '—'}
                          </span>
                          <span className="font-sans text-sm font-semibold text-primary">
                            {getEventLabel(ev)}
                          </span>

                          {ev.age_at_event !== null &&
                            ev.age_at_event !== undefined && (
                              <Badge variant="neutral" size="sm">
                                {t('profile.chronology.ageAtEvent', {
                                  age: ev.age_at_event,
                                })}
                              </Badge>
                            )}

                          {ev.is_private && (
                            <Badge
                              variant="warning"
                              size="sm"
                              icon={<Lock className="h-2.5 w-2.5" />}
                            >
                              {t('domain.privateBadge')}
                            </Badge>
                          )}

                          {scansCount > 0 && (
                            <Badge
                              variant="brand"
                              size="sm"
                              icon={<FileText className="h-2.5 w-2.5" />}
                            >
                              {t('profile.chronology.scansAttached', {
                                count: scansCount,
                              })}
                            </Badge>
                          )}
                        </div>

                        {/* Date & Location */}
                        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-secondary">
                          <span className="inline-flex items-center gap-1">
                            <Calendar className="h-3 w-3 text-tertiary shrink-0" />
                            <span>{dateDisplay}</span>
                          </span>

                          {ev.place_full_name && (
                            <span className="inline-flex items-center gap-1">
                              <MapPin className="h-3 w-3 text-tertiary shrink-0" />
                              <span>{ev.place_full_name}</span>
                            </span>
                          )}
                        </div>

                        {/* Spouse Link for Family Events */}
                        {ev.spouse_name && (
                          <div className="flex items-center gap-1.5 text-xs text-secondary">
                            <Heart className="h-3 w-3 text-accent shrink-0" />
                            <span>{t('profile.chronology.withSpouse')}:</span>
                            {ev.spouse_id && onSelectRelativeId ? (
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  onSelectRelativeId(ev.spouse_id!);
                                }}
                                className="h-6 px-1.5 text-xs text-brand font-medium"
                              >
                                {ev.spouse_name}
                              </Button>
                            ) : (
                              <strong className="text-primary">
                                {ev.spouse_name}
                              </strong>
                            )}
                          </div>
                        )}

                        {/* Description */}
                        {ev.description && (
                          <p className="text-xs sm:text-sm text-primary pt-0.5">
                            {ev.description}
                          </p>
                        )}

                        {/* Archival Source Citation */}
                        {ev.source_text && (
                          <div className="flex items-start gap-1.5 rounded-lg bg-surface-muted/70 px-2.5 py-1.5 text-[11px] text-secondary">
                            <FileText className="h-3.5 w-3.5 text-tertiary shrink-0 mt-0.5" />
                            <span>
                              <strong className="font-medium text-primary">
                                {t('profile.chronology.sourceLabel')}:
                              </strong>{' '}
                              {ev.source_text}
                            </span>
                          </div>
                        )}
                      </div>

                      {/* Inline Edit / Delete Controls for Editors & Admins */}
                      {canEdit && (
                        <div
                          className="flex items-center gap-1 shrink-0"
                          onClick={(e) => e.stopPropagation()}
                        >
                          <IconButton
                            size="sm"
                            variant="ghost"
                            label={t('profile.chronology.editEventAria')}
                            onClick={() => onEditEvent(ev)}
                          >
                            <PenTool className="h-3.5 w-3.5" />
                          </IconButton>
                          {onDeleteEvent && (
                            <IconButton
                              size="sm"
                              variant="ghost"
                              label={t('profile.chronology.deleteEventAria')}
                              onClick={() => onDeleteEvent(ev)}
                            >
                              <Trash2 className="h-3.5 w-3.5 text-danger" />
                            </IconButton>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </Card>
      </div>

      {/* Right Pane (5 cols): Interactive Life Migration Map & Narrative Biography */}
      <div className="lg:col-span-5 space-y-6">
        <ProfileLifeMap
          events={events}
          highlightedEventId={highlightedEventId}
          onSelectEvent={(id) => onSelectEvent?.(id)}
        />

        {/* Narrative Biography Card */}
        <Card className="p-5 space-y-4">
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <BookOpen className="h-4 w-4 text-brand shrink-0" />
              <h3 className="font-sans text-sm font-semibold text-primary">
                {t('profile.biography.title')}
              </h3>
            </div>

            {canEdit && !editingBio && (
              <Button
                size="sm"
                variant="ghost"
                leftIcon={<PenTool className="h-3.5 w-3.5" />}
                onClick={startBioEdit}
              >
                {t('common.edit')}
              </Button>
            )}
          </div>

          {editingBio && canEdit ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs text-secondary">
                  {t('profile.biography.markdownHint')}
                </span>
                <Button
                  size="sm"
                  variant="ghost"
                  leftIcon={<Eye className="h-3.5 w-3.5" />}
                  onClick={() => setBioPreview((p) => !p)}
                >
                  {bioPreview
                    ? t('profile.biography.editModeBtn')
                    : t('profile.biography.previewBtn')}
                </Button>
              </div>

              {bioPreview ? (
                <div className="rounded-xl border border-border bg-surface-muted/40 p-3 min-h-28">
                  <MarkdownView
                    content={bioDraft || t('profile.biography.emptyText')}
                  />
                </div>
              ) : (
                <Textarea
                  value={bioDraft}
                  onChange={(e) => setBioDraft(e.target.value)}
                  rows={5}
                  placeholder={t('profile.biography.placeholder')}
                />
              )}

              <div className="flex justify-end gap-2">
                <Button
                  size="sm"
                  variant="secondary"
                  leftIcon={<X className="h-3.5 w-3.5" />}
                  onClick={() => setEditingBio(false)}
                  disabled={savingBio}
                >
                  {t('common.cancel')}
                </Button>
                <Button
                  size="sm"
                  variant="primary"
                  loading={savingBio}
                  leftIcon={<Save className="h-3.5 w-3.5" />}
                  onClick={() => void handleSaveBio()}
                >
                  {t('common.save')}
                </Button>
              </div>
            </div>
          ) : initialBio.trim() ? (
            <MarkdownView content={initialBio} />
          ) : (
            <p className="text-xs text-tertiary italic">
              {t('profile.biography.emptyText')}
            </p>
          )}
        </Card>
      </div>
    </div>
  );
};
