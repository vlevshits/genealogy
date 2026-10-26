import React from 'react';
import { cn } from '../../lib/utils';

export interface DividerProps extends React.HTMLAttributes<HTMLDivElement> {
  orientation?: 'horizontal' | 'vertical';
  label?: React.ReactNode;
  ref?: React.Ref<HTMLDivElement>;
}

export const Divider: React.FC<DividerProps> = ({
  orientation = 'horizontal',
  label,
  className,
  ref,
  ...props
}) => {
  if (orientation === 'vertical') {
    return (
      <div
        ref={ref}
        role="separator"
        aria-orientation="vertical"
        className={cn('h-full w-px bg-border shrink-0', className)}
        {...props}
      />
    );
  }

  if (label) {
    return (
      <div
        ref={ref}
        role="separator"
        aria-orientation="horizontal"
        className={cn('relative flex items-center py-2', className)}
        {...props}
      >
        <div className="flex-grow border-t border-border" />
        <span className="mx-3 shrink-0 text-xs font-medium text-tertiary">{label}</span>
        <div className="flex-grow border-t border-border" />
      </div>
    );
  }

  return (
    <div
      ref={ref}
      role="separator"
      aria-orientation="horizontal"
      className={cn('w-full border-t border-border', className)}
      {...props}
    />
  );
};
