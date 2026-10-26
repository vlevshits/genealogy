import React from 'react';
import { Loader2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import { ControlSize } from '../../types/common';
import { useI18n } from '../../i18n';

export interface SpinnerProps extends React.HTMLAttributes<HTMLSpanElement> {
  size?: ControlSize;
  label?: string;
  ref?: React.Ref<HTMLSpanElement>;
}

export const Spinner: React.FC<SpinnerProps> = ({
  size = 'md',
  label,
  className,
  ref,
  ...props
}) => {
  const { t } = useI18n();
  const sizeClass: Record<ControlSize, string> = {
    sm: 'h-3.5 w-3.5',
    md: 'h-5 w-5',
    lg: 'h-7 w-7',
  };

  return (
    <span
      ref={ref}
      role="status"
      aria-label={label || t('common.loading')}
      className={cn('inline-flex items-center justify-center text-current', className)}
      {...props}
    >
      <Loader2 className={cn('animate-spin', sizeClass[size])} />
    </span>
  );
};
