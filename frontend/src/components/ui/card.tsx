import React from 'react';
import { cn } from '../../lib/utils';

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  interactive?: boolean;
  ref?: React.Ref<HTMLDivElement>;
}

export const Card: React.FC<CardProps> = ({
  interactive = false,
  className,
  children,
  ref,
  ...props
}) => {
  return (
    <div
      ref={ref}
      className={cn(
        'rounded-2xl border border-border bg-surface text-primary shadow-xs transition-all duration-150',
        interactive && 'cursor-pointer hover:shadow-md hover:border-secondary/40',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
};

export const CardHeader: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('border-b border-border px-5 py-4', className)} {...props}>
    {children}
  </div>
);

export const CardBody: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('p-5', className)} {...props}>
    {children}
  </div>
);

export const CardFooter: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div
    className={cn(
      'flex items-center justify-between border-t border-border bg-surface-muted/40 px-5 py-3 rounded-b-2xl',
      className
    )}
    {...props}
  >
    {children}
  </div>
);
