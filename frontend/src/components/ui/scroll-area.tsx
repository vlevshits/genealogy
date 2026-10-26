import React from 'react';
import { cn } from '../../lib/utils';

export interface ScrollAreaProps extends React.HTMLAttributes<HTMLDivElement> {
  maxHeight?: string | number;
  ref?: React.Ref<HTMLDivElement>;
}

export const ScrollArea: React.FC<ScrollAreaProps> = ({
  maxHeight = 320,
  style,
  className,
  children,
  ref,
  ...props
}) => {
  return (
    <div
      ref={ref}
      style={{ maxHeight, ...style }}
      className={cn('overflow-y-auto overscroll-contain pr-1', className)}
      {...props}
    >
      {children}
    </div>
  );
};
