import React from 'react';
import { cn } from '../../lib/utils';
import { GenDate } from '../../types/gendate';
import { formatGenDate } from '../../lib/gendate-parser';
import { useI18n } from '../../i18n';

export interface GenDateTextProps extends React.HTMLAttributes<HTMLSpanElement> {
  date?: GenDate | null;
  showDualCalendar?: boolean;
  short?: boolean;
  ref?: React.Ref<HTMLSpanElement>;
}

export const GenDateText: React.FC<GenDateTextProps> = ({
  date,
  showDualCalendar = false,
  short = false,
  className,
  ref,
  ...props
}) => {
  const { locale, t } = useI18n();
  if (!date) {
    return (
      <span ref={ref} className={cn('text-tertiary', className)} {...props}>
        {t('domain.unknownYear')}
      </span>
    );
  }
  const formatted = formatGenDate(date, locale, { showDualCalendar, short });
  return (
    <span ref={ref} className={cn('font-mono text-xs text-secondary', className)} {...props}>
      {formatted}
    </span>
  );
};
