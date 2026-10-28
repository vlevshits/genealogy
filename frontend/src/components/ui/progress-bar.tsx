import React from 'react';
import { cn } from '../../lib/utils';

export interface ProgressBarProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number;
  max?: number;
  indeterminate?: boolean;
  label?: string;
  hideLabel?: boolean;
  stepText?: string;
  showPercentage?: boolean;
  variant?: 'brand' | 'success' | 'warning' | 'danger';
  ref?: React.Ref<HTMLDivElement>;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value = 0,
  max = 100,
  indeterminate = false,
  label,
  hideLabel = false,
  stepText,
  showPercentage = true,
  variant = 'brand',
  className,
  ref,
  'aria-label': ariaLabel,
  ...props
}) => {
  const clamped = Math.min(max, Math.max(0, value));
  const percent = max > 0 ? Math.round((clamped / max) * 100) : 0;

  const barColors = {
    brand: 'bg-brand',
    success: 'bg-success',
    warning: 'bg-warning',
    danger: 'bg-danger',
  };

  return (
    <div ref={ref} className={cn('w-full space-y-1.5', className)} {...props}>
      {!hideLabel && (label || showPercentage || stepText) && (
        <div className="flex items-center justify-between text-xs">
          <div className="font-medium text-primary">
            {label}
            {stepText && <span className="ml-2 text-secondary font-normal">· {stepText}</span>}
          </div>
          {showPercentage && !indeterminate && (
            <span className="font-mono text-secondary">{percent}%</span>
          )}
        </div>
      )}
      <div
        role="progressbar"
        aria-label={ariaLabel || label || stepText || 'Progress'}
        aria-valuemin={0}
        aria-valuemax={max}
        aria-valuenow={indeterminate ? undefined : clamped}
        className="h-2 w-full overflow-hidden rounded-full bg-surface-muted border border-border/50"
      >
        <div
          className={cn(
            'h-full rounded-full transition-all duration-200',
            barColors[variant],
            indeterminate && 'w-1/2 animate-pulse'
          )}
          style={indeterminate ? undefined : { width: `${percent}%` }}
        />
      </div>
    </div>
  );
};
