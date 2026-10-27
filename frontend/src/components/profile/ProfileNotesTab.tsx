import React from 'react';
import {
  BookOpen,
  ExternalLink,
  FileText,
  Lock,
  PenTool,
  Plus,
  Trash2,
} from 'lucide-react';
import {
  Badge,
  Button,
  Card,
  EmptyState,
  IconButton,
  MarkdownView,
} from '../ui';
import { NoteDto, ProfileSourceSummaryDto } from '../../api/client';
import { useI18n } from '../../i18n';

export interface ProfileNotesTabProps {
  notes: NoteDto[];
  sourcesSummary: ProfileSourceSummaryDto[];
  canEdit: boolean;
  isMaskedLiving?: boolean;
  onAddNote: () => void;
  onEditNote: (note: NoteDto) => void;
  onDeleteNote: (note: NoteDto) => void;
  onSelectSourceEvent?: (eventId: string) => void;
}

export const ProfileNotesTab: React.FC<ProfileNotesTabProps> = ({
  notes,
  sourcesSummary,
  canEdit,
  isMaskedLiving = false,
  onAddNote,
  onEditNote,
  onDeleteNote,
  onSelectSourceEvent,
}) => {
  const { t } = useI18n();

  if (isMaskedLiving) {
    return (
      <Card className="p-8">
        <EmptyState
          icon={<Lock className="h-8 w-8 text-warning" />}
          title={t('profile.notes.maskedTitle')}
          description={t('profile.notes.maskedDesc')}
        />
      </Card>
    );
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
      {/* Left Column (7 cols): Research Notes & Family Stories */}
      <div className="lg:col-span-7 space-y-4">
        <Card className="p-5 sm:p-6 space-y-5">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              <FileText className="h-4 w-4 text-brand shrink-0" />
              <h2 className="font-sans text-base font-semibold text-primary">
                {t('profile.notes.title')}
              </h2>
              <Badge variant="neutral" size="sm">
                {notes.length}
              </Badge>
            </div>

            {canEdit && (
              <Button
                size="sm"
                variant="primary"
                leftIcon={<Plus className="h-3.5 w-3.5" />}
                onClick={onAddNote}
              >
                {t('profile.notes.addNoteBtn')}
              </Button>
            )}
          </div>

          {notes.length === 0 ? (
            <EmptyState
              icon={<FileText className="h-7 w-7 text-tertiary" />}
              title={t('profile.notes.emptyNotesTitle')}
              description={t('profile.notes.emptyNotesDesc')}
              action={
                canEdit ? (
                  <Button
                    size="sm"
                    variant="primary"
                    leftIcon={<Plus className="h-3.5 w-3.5" />}
                    onClick={onAddNote}
                  >
                    {t('profile.notes.addNoteBtn')}
                  </Button>
                ) : undefined
              }
            />
          ) : (
            <div className="space-y-4">
              {notes.map((note) => (
                <div
                  key={note.id}
                  className="rounded-2xl border border-border bg-surface p-4 space-y-3"
                >
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                      {note.is_private ? (
                        <Badge
                          variant="warning"
                          size="sm"
                          icon={<Lock className="h-2.5 w-2.5" />}
                        >
                          {t('domain.privateBadge')}
                        </Badge>
                      ) : (
                        <Badge variant="neutral" size="sm">
                          {t('profile.notes.researchNoteBadge')}
                        </Badge>
                      )}
                      <span className="font-mono text-[11px] text-tertiary">
                        v{note.version}
                      </span>
                    </div>

                    {canEdit && (
                      <div className="flex items-center gap-1">
                        <IconButton
                          size="sm"
                          variant="ghost"
                          label={t('profile.notes.editNoteAria')}
                          onClick={() => onEditNote(note)}
                        >
                          <PenTool className="h-3.5 w-3.5" />
                        </IconButton>
                        <IconButton
                          size="sm"
                          variant="ghost"
                          label={t('profile.notes.deleteNoteAria')}
                          onClick={() => onDeleteNote(note)}
                        >
                          <Trash2 className="h-3.5 w-3.5 text-danger" />
                        </IconButton>
                      </div>
                    )}
                  </div>

                  <MarkdownView content={note.body} />
                </div>
              ))}
            </div>
          )}
        </Card>
      </div>

      {/* Right Column (5 cols): Sources Used Summary (03 §3.2 Tab 4) */}
      <div className="lg:col-span-5">
        <Card className="p-5 sm:p-6 space-y-4">
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <BookOpen className="h-4 w-4 text-brand shrink-0" />
              <h2 className="font-sans text-base font-semibold text-primary">
                {t('profile.sources.title')}
              </h2>
            </div>
            <Badge variant="neutral" size="sm">
              {sourcesSummary.length}
            </Badge>
          </div>

          <p className="text-xs text-secondary">
            {t('profile.sources.subtitle')}
          </p>

          {sourcesSummary.length === 0 ? (
            <EmptyState
              icon={<BookOpen className="h-7 w-7 text-tertiary" />}
              title={t('profile.sources.emptyTitle')}
              description={t('profile.sources.emptyDesc')}
            />
          ) : (
            <div className="space-y-2.5">
              {sourcesSummary.map((src, idx) => {
                const count = src.events_count ?? src.count ?? src.event_ids.length;
                const firstEventId = src.event_ids[0];
                return (
                  <div
                    key={`${src.source_text}-${idx}`}
                    className="flex items-start justify-between gap-3 rounded-xl border border-border bg-surface-muted/40 p-3"
                  >
                    <div className="space-y-1 min-w-0">
                      <div className="text-xs sm:text-sm font-medium text-primary break-words">
                        {src.source_text}
                      </div>
                      <div className="text-[11px] text-tertiary">
                        {t('profile.sources.eventsReferenced', { count })}
                      </div>
                    </div>

                    {firstEventId && onSelectSourceEvent && (
                      <Button
                        size="sm"
                        variant="ghost"
                        leftIcon={<ExternalLink className="h-3.5 w-3.5" />}
                        onClick={() => onSelectSourceEvent(firstEventId)}
                        className="shrink-0"
                      >
                        {t('profile.sources.jumpToEventBtn')}
                      </Button>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </Card>
      </div>
    </div>
  );
};
