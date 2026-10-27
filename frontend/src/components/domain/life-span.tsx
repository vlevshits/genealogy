import React from 'react';
import { cn } from '../../lib/utils';
import { GenDate } from '../../types/gendate';
import { useI18n } from '../../i18n';
import { formatLifeSpanText } from '../../lib/formatters';

export interface LifeSpanProps extends React.HTMLAttributes<HTMLSpanElement> {
  birthDate?: GenDate | null;
  deathDate?: GenDate | null;
  isLiving?: boolean;
  isRedacted?: boolean;
  showAge?: boolean;
  ref?: React.Ref<HTMLSpanElement>;
}

export const LifeSpan: React.FC<LifeSpanProps> = ({
  birthDate,
  deathDate,
  isLiving = false,
  isRedacted = false,
  showAge = true,
  className,
  ref,
  ...props
}) => {
  const { t, locale } = useI18n();

  if (isRedacted) {
    return null;
  }

  const bYear = birthDate?.start?.y ?? null;
  const dYear = deathDate?.start?.y ?? null;

  const spanText = formatLifeSpanText({
    birthYear: bYear,
    deathYear: dYear,
    isLiving,
    isRedacted,
    locale,
  });

  if (!spanText) {
    return null;
  }

  const age =
    showAge && bYear !== null && dYear !== null && dYear >= bYear ? dYear - bYear : null;

  return (
    <span ref={ref} className={cn('font-mono text-xs text-secondary', className)} {...props}>
      {spanText}
      {age !== null && <span className="ml-1.5 text-tertiary">· {t('domain.yearsOld', { age })}</span>}
    </span>
  );
};
