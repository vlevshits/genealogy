import React from 'react';
import { cn } from '../../lib/utils';
import { Badge } from './badge';

export interface PageHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  title: string;
  count?: number;
  subtitle?: string;
  primaryAction?: React.ReactNode;
  secondaryActions?: React.ReactNode;
  ref?: React.Ref<HTMLDivElement>;
}

export const PageHeader: React.FC<PageHeaderProps> = ({
  title,
  count,
  subtitle,
  primaryAction,
  secondaryActions,
  className,
  ref,
  ...props
}) => {
  return (
    <div
      ref={ref}
      className={cn(
        'flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between border-b border-border pb-4',
        className
      )}
      {...props}
    >
      <div className="space-y-1">
        <div className="flex items-center gap-2.5">
          <h1 className="font-sans text-xl sm:text-2xl font-semibold tracking-tight text-primary">
            {title}
          </h1>
          {count !== undefined && (
            <Badge variant="neutral" size="sm">
              {count.toLocaleString()}
            </Badge>
          )}
        </div>
        {subtitle && <p className="text-xs sm:text-sm text-secondary">{subtitle}</p>}
      </div>
      {(primaryAction || secondaryActions) && (
        <div className="flex flex-wrap items-center gap-2 shrink-0">
          {secondaryActions}
          {primaryAction}
        </div>
      )}
    </div>
  );
};
