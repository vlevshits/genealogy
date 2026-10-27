import React from 'react';
import { cn } from '../../lib/utils';
import { Individual } from '../../types/person';
import { useI18n } from '../../i18n';
import { formatPersonName } from '../../lib/formatters';

export type PersonNameVariant = 'full' | 'short' | 'card';

export interface PersonNameProps extends React.HTMLAttributes<HTMLSpanElement> {
  person: Pick<
    Individual,
    | 'given_name'
    | 'patronymic'
    | 'surname'
    | 'birth_surname'
    | 'name_prefix'
    | 'name_suffix'
    | 'is_redacted'
  >;
  variant?: PersonNameVariant;
  ref?: React.Ref<HTMLSpanElement>;
}

export const PersonName: React.FC<PersonNameProps> = ({
  person,
  variant = 'full',
  className,
  ref,
  ...props
}) => {
  const { locale, t } = useI18n();

  if (person.is_redacted) {
    const masked = person.surname
      ? t('domain.livingMasked', { surname: person.surname })
      : t('domain.livingPerson');
    return (
      <span ref={ref} className={cn('italic text-secondary', className)} {...props}>
        {masked}
      </span>
    );
  }

  const formatted = formatPersonName(person, {
    locale,
    format: variant === 'short' ? 'compact' : variant,
    showMaiden: variant !== 'short',
    maidenStyle: 'parens',
  });

  return (
    <span ref={ref} className={cn('font-medium text-primary', className)} {...props}>
      {formatted || t('domain.livingPerson')}
    </span>
  );
};
