import React from 'react';
import { cn } from '../../lib/utils';
import { Individual } from '../../types/person';
import { useI18n } from '../../i18n';

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

  const given = person.given_name?.trim() ?? '';
  const patr = person.patronymic?.trim() ?? '';
  const sur = person.surname?.trim() ?? '';
  const birthSur =
    person.birth_surname && person.birth_surname !== sur
      ? `(${person.birth_surname})`
      : '';

  let formatted = '';
  if (variant === 'short') {
    formatted = [given, sur].filter(Boolean).join(' ');
  } else if (variant === 'card') {
    formatted =
      locale === 'ru'
        ? [given, patr, sur, birthSur].filter(Boolean).join(' ')
        : [given, patr, sur, birthSur].filter(Boolean).join(' ');
  } else {
    // 'full'
    formatted =
      locale === 'ru'
        ? [sur, birthSur, given, patr].filter(Boolean).join(' ')
        : [person.name_prefix, given, patr, sur, birthSur, person.name_suffix]
            .filter(Boolean)
            .join(' ');
  }

  return (
    <span ref={ref} className={cn('font-medium text-primary', className)} {...props}>
      {formatted || t('domain.unknownYear')}
    </span>
  );
};
