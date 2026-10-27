import React from 'react';
import { Plus } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Individual } from '../../types/person';
import { Avatar } from '../ui/avatar';
import { IconButton } from '../ui/icon-button';
import { PersonName } from './person-name';
import { LifeSpan } from './life-span';
import { PlaceText } from './place-text';
import { PrivacyBadge } from './privacy-badge';
import { useI18n } from '../../i18n';
import { formatPersonName } from '../../lib/formatters';

export type QuickAddDirection = 'parent' | 'spouse' | 'child';

export interface PersonCardProps {
  person: Individual;
  selected?: boolean;
  onSelect?: (person: Individual) => void;
  onQuickAdd?: (person: Individual, direction: QuickAddDirection) => void;
  className?: string;
}

export const PersonCard: React.FC<PersonCardProps> = ({
  person,
  selected = false,
  onSelect,
  onQuickAdd,
  className,
}) => {
  const { t, locale } = useI18n();

  const accentBorder =
    person.sex === 'male'
      ? 'border-l-4 border-l-male'
      : person.sex === 'female'
        ? 'border-l-4 border-l-female'
        : 'border-l-4 border-l-neutral';

  const fullName = formatPersonName(person, { locale, format: 'compact' });

  return (
    <div className={cn('group relative inline-block p-3', className)}>
      {/* Quick-add directional buttons — visible on mobile touch and revealed on desktop hover/focus */}
      {onQuickAdd && (
        <>
          <IconButton
            size="sm"
            label={t('domain.addFather')}
            title={t('domain.addFather')}
            onClick={(e) => {
              e.stopPropagation();
              onQuickAdd(person, 'parent');
            }}
            className="absolute -top-1 left-1/2 -translate-x-1/2 z-10 flex h-8 w-8 min-h-8 min-w-8 items-center justify-center rounded-full border border-border bg-surface text-secondary opacity-85 sm:opacity-0 shadow-sm transition-opacity group-hover:opacity-100 focus-visible:opacity-100 hover:bg-brand hover:text-surface cursor-pointer"
          >
            <Plus className="h-3.5 w-3.5" />
          </IconButton>
          <IconButton
            size="sm"
            label={t('domain.addSpouse')}
            title={t('domain.addSpouse')}
            onClick={(e) => {
              e.stopPropagation();
              onQuickAdd(person, 'spouse');
            }}
            className="absolute -right-1 top-1/2 -translate-y-1/2 z-10 flex h-8 w-8 min-h-8 min-w-8 items-center justify-center rounded-full border border-border bg-surface text-secondary opacity-85 sm:opacity-0 shadow-sm transition-opacity group-hover:opacity-100 focus-visible:opacity-100 hover:bg-brand hover:text-surface cursor-pointer"
          >
            <Plus className="h-3.5 w-3.5" />
          </IconButton>
          <IconButton
            size="sm"
            label={t('domain.addChild')}
            title={t('domain.addChild')}
            onClick={(e) => {
              e.stopPropagation();
              onQuickAdd(person, 'child');
            }}
            className="absolute -bottom-1 left-1/2 -translate-x-1/2 z-10 flex h-8 w-8 min-h-8 min-w-8 items-center justify-center rounded-full border border-border bg-surface text-secondary opacity-85 sm:opacity-0 shadow-sm transition-opacity group-hover:opacity-100 focus-visible:opacity-100 hover:bg-brand hover:text-surface cursor-pointer"
          >
            <Plus className="h-3.5 w-3.5" />
          </IconButton>
        </>
      )}

      <div
        role="button"
        tabIndex={0}
        onClick={() => onSelect?.(person)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onSelect?.(person);
          }
        }}
        className={cn(
          'flex w-72 items-start gap-3.5 rounded-xl border border-border bg-surface p-3.5 text-left shadow-xs transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
          accentBorder,
          selected ? 'ring-2 ring-focus shadow-md' : 'hover:shadow-md'
        )}
      >
        <Avatar
          name={fullName}
          src={person.avatar_url}
          sex={person.sex}
          size="md"
          isLivingRedacted={person.is_redacted}
        />
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex items-start justify-between gap-1.5">
            <PersonName
              person={person}
              variant="card"
              className="text-sm font-semibold leading-snug"
            />
            <span className="font-mono text-[10px] text-tertiary shrink-0">{person.gid}</span>
          </div>

          <LifeSpan
            birthDate={person.birth_date}
            deathDate={person.death_date}
            isLiving={person.is_living}
            isRedacted={person.is_redacted}
            showAge={false}
          />

          {person.birth_place && !person.is_redacted && (
            <div className="truncate">
              <PlaceText place={person.birth_place} />
            </div>
          )}

          {person.is_private && (
            <div className="flex items-center gap-1.5 pt-1">
              <PrivacyBadge />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
