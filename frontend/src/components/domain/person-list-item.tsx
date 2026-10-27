import React from 'react';
import { cn } from '../../lib/utils';
import { Individual } from '../../types/person';
import { Avatar } from '../ui/avatar';
import { PersonName } from './person-name';
import { LifeSpan } from './life-span';
import { PlaceText } from './place-text';
import { PrivacyBadge } from './privacy-badge';
import { useI18n } from '../../i18n';
import { formatPersonName } from '../../lib/formatters';

export interface PersonListItemProps extends React.HTMLAttributes<HTMLDivElement> {
  person: Individual;
  selected?: boolean;
  ref?: React.Ref<HTMLDivElement>;
}

export const PersonListItem: React.FC<PersonListItemProps> = ({
  person,
  selected = false,
  className,
  ref,
  ...props
}) => {
  const { locale } = useI18n();
  const fullName = formatPersonName(person, { locale, format: 'compact' });

  return (
    <div
      ref={ref}
      className={cn(
        'flex items-center justify-between gap-3 rounded-xl px-3 py-2 transition-colors',
        selected ? 'bg-brand/10' : 'hover:bg-surface-muted/70',
        className
      )}
      {...props}
    >
      <div className="flex items-center gap-3 min-w-0">
        <Avatar
          name={fullName}
          src={person.avatar_url}
          sex={person.sex}
          size="md"
          showGenderRing
          isLivingRedacted={person.is_redacted}
        />
        <div className="min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <PersonName person={person} variant="full" className="text-sm font-semibold truncate" />
            <span className="font-mono text-[11px] text-tertiary">{person.gid}</span>
            {person.is_private && <PrivacyBadge />}
          </div>
          <div className="mt-0.5 flex items-center flex-wrap gap-2 text-xs text-secondary">
            <LifeSpan
              birthDate={person.birth_date}
              deathDate={person.death_date}
              isLiving={person.is_living}
              isRedacted={person.is_redacted}
            />
            {person.birth_place && !person.is_redacted && (
              <>
                <span className="text-tertiary">·</span>
                <PlaceText place={person.birth_place} />
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
