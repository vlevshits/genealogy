import React from 'react';
import { cn } from '../../lib/utils';

export interface TimelineItem {
  id: string;
  title: React.ReactNode;
  meta?: React.ReactNode;
  body?: React.ReactNode;
  actions?: React.ReactNode;
  icon?: React.ReactNode;
  tone?: 'brand' | 'accent' | 'success' | 'neutral';
}

export interface TimelineProps extends React.HTMLAttributes<HTMLOListElement> {
  items: TimelineItem[];
  ref?: React.Ref<HTMLOListElement>;
}

export const Timeline: React.FC<TimelineProps> = ({ items, className, ref, ...props }) => {
  const toneClasses = {
    brand: 'border-brand bg-brand/15 text-brand',
    accent: 'border-accent bg-accent/15 text-accent',
    success: 'border-success bg-success/15 text-success',
    neutral: 'border-border bg-surface-muted text-secondary',
  };

  return (
    <ol ref={ref} className={cn('relative border-l border-border ml-3 space-y-6', className)} {...props}>
      {items.map((item) => {
        const tone = item.tone ?? 'brand';
        return (
          <li key={item.id} className="ml-6">
            <span
              className={cn(
                'absolute -left-3.5 flex h-7 w-7 items-center justify-center rounded-full border',
                toneClasses[tone]
              )}
            >
              {item.icon ?? <span className="h-2 w-2 rounded-full bg-current" />}
            </span>
            <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-2 rounded-xl border border-border bg-surface p-4 shadow-2xs">
              <div className="space-y-1">
                <div className="text-sm font-semibold text-primary">{item.title}</div>
                {item.meta && <div className="text-xs text-secondary">{item.meta}</div>}
                {item.body && <div className="pt-1 text-xs sm:text-sm text-primary">{item.body}</div>}
              </div>
              {item.actions && <div className="shrink-0">{item.actions}</div>}
            </div>
          </li>
        );
      })}
    </ol>
  );
};
