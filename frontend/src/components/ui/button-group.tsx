import React from 'react';
import { cn } from '../../lib/utils';

export interface ButtonGroupProps extends React.HTMLAttributes<HTMLDivElement> {
  orientation?: 'horizontal' | 'vertical';
  ref?: React.Ref<HTMLDivElement>;
}

export const ButtonGroup: React.FC<ButtonGroupProps> = ({
  orientation = 'horizontal',
  className,
  children,
  ref,
  ...props
}) => {
  return (
    <div
      ref={ref}
      role="group"
      className={cn(
        'inline-flex rounded-lg border border-border bg-surface p-0.5 shadow-xs',
        orientation === 'vertical' ? 'flex-col' : 'flex-row items-center',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
};
