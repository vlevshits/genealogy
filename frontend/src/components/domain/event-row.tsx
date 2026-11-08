import React from 'react';
import {
  Award,
  Baby,
  BookOpen,
  Briefcase,
  Cross,
  FileImage,
  GraduationCap,
  Heart,
  Home,
  Pencil,
  Shield,
} from 'lucide-react';
import { cn } from '../../lib/utils';
import { normalizeEventType } from '../../lib/domain-enums';
import { LifeEvent } from '../../types/event';
import { EventType } from '../../types/common';
import { GenDateText } from './gen-date-text';
import { PlaceText } from './place-text';
import { PrivacyBadge } from './privacy-badge';
import { Badge } from '../ui/badge';
import { IconButton } from '../ui/icon-button';
import { useI18n } from '../../i18n';

export interface EventRowProps {
  event: LifeEvent;
  onEdit?: (event: LifeEvent) => void;
  className?: string;
}

function getEventIcon(type: EventType): React.ReactNode {
  switch (type) {
    case 'birth':
    case 'baptism':
      return <Baby className="h-4 w-4 text-brand" />;
    case 'death':
    case 'burial':
    case 'cremation':
      return <Cross className="h-4 w-4 text-secondary" />;
    case 'marriage':
    case 'engagement':
      return <Heart className="h-4 w-4 text-female" />;
    case 'education':
      return <GraduationCap className="h-4 w-4 text-brand" />;
    case 'occupation':
      return <Briefcase className="h-4 w-4 text-accent" />;
    case 'military':
      return <Shield className="h-4 w-4 text-male" />;
    case 'residence':
      return <Home className="h-4 w-4 text-brand" />;
    default:
      return <Award className="h-4 w-4 text-accent" />;
  }
}

export const EventRow: React.FC<EventRowProps> = ({ event, onEdit, className }) => {
  const { t } = useI18n();
  const normType = normalizeEventType(event.event_type);
  const typeLabel =
    normType === 'custom' && event.custom_type
      ? event.custom_type
      : t(`domain.events.${normType}`);

  return (
    <div
      className={cn(
        'group flex items-start justify-between gap-4 rounded-2xl border border-border bg-surface p-4 shadow-2xs transition-colors hover:border-secondary/40',
        className
      )}
    >
      <div className="flex items-start gap-3.5 min-w-0 flex-1">
        <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-surface-muted border border-border">
          {getEventIcon(normType)}
        </div>

        <div className="min-w-0 flex-1 space-y-1.5">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-semibold text-primary">{typeLabel}</span>
            <GenDateText date={event.date} showDualCalendar />
            {event.age_at_event !== undefined && event.age_at_event !== null && (
              <Badge variant="neutral" size="sm">
                {t('domain.ageAtEvent', { age: event.age_at_event })}
              </Badge>
            )}
            {event.is_private && <PrivacyBadge />}
          </div>

          {event.place && <PlaceText place={event.place} />}

          {event.description && (
            <p className="text-xs sm:text-sm text-primary leading-relaxed">{event.description}</p>
          )}

          {(event.source_text || event.has_scan) && (
            <div className="flex flex-wrap items-center gap-2 pt-1 text-xs text-secondary">
              {event.source_text && (
                <span className="inline-flex items-center gap-1 rounded-md bg-surface-muted px-2 py-0.5 text-[11px]">
                  <BookOpen className="h-3 w-3 text-brand" />
                  <span>{event.source_text}</span>
                </span>
              )}
              {event.has_scan && (
                <Badge
                  variant="brand"
                  size="sm"
                  icon={<FileImage className="h-3 w-3" />}
                >
                  {t('domain.scanAttached', { count: event.scan_count ?? 1 })}
                </Badge>
              )}
            </div>
          )}
        </div>
      </div>

      {onEdit && (
        <IconButton
          size="sm"
          label={t('common.edit')}
          onClick={() => onEdit(event)}
          className="opacity-80 group-hover:opacity-100"
        >
          <Pencil className="h-3.5 w-3.5" />
        </IconButton>
      )}
    </div>
  );
};
