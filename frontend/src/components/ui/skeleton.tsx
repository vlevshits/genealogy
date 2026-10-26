import React from 'react';
import { cn } from '../../lib/utils';

export interface SkeletonProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'text' | 'circle' | 'rect';
  ref?: React.Ref<HTMLDivElement>;
}

export const Skeleton: React.FC<SkeletonProps> = ({
  variant = 'rect',
  className,
  ref,
  ...props
}) => {
  return (
    <div
      ref={ref}
      aria-hidden="true"
      className={cn(
        'animate-pulse bg-surface-muted border border-border/50',
        variant === 'circle' && 'rounded-full',
        variant === 'text' && 'h-4 rounded-md',
        variant === 'rect' && 'rounded-xl',
        className
      )}
      {...props}
    />
  );
};
