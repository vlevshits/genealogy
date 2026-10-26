import React from 'react';
import { cn } from '../../lib/utils';

export interface KbdProps extends React.HTMLAttributes<HTMLElement> {
  ref?: React.Ref<HTMLElement>;
}

export const Kbd: React.FC<KbdProps> = ({ className, children, ref, ...props }) => {
  return (
    <kbd
      ref={ref}
      className={cn(
        'inline-flex items-center justify-center rounded border border-border bg-surface-muted px-1.5 py-0.5 font-mono text-[11px] font-medium text-tertiary shadow-2xs',
        className
      )}
      {...props}
    >
      {children}
    </kbd>
  );
};
