import React from 'react';
import { cn } from '../../lib/utils';
import { GenDate } from '../../types/gendate';
import { useI18n } from '../../i18n';

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
  const { t } = useI18n();

  if (isRedacted) {
    return (
      <span ref={ref} className={cn('text-xs text-tertiary', className)} {...props}>
        {t('domain.living')}
      </span>
    );
  }

  const bYear = birthDate?.start?.y ?? null;
  const dYear = deathDate?.start?.y ?? null;

  const left = bYear !== null ? String(bYear) : t('domain.unknownYear');
  const right = isLiving
    ? t('domain.living')
    : dYear !== null
      ? String(dYear)
      : t('domain.unknownYear');

  const age =
    showAge && bYear !== null && dYear !== null && dYear >= bYear ? dYear - bYear : null;

  return (
    <span ref={ref} className={cn('font-mono text-xs text-secondary', className)} {...props}>
      {left} – {right}
      {age !== null && <span className="ml-1.5 text-tertiary">· {t('domain.yearsOld', { age })}</span>}
    </span>
  );
};
