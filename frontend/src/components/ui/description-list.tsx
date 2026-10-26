import React from 'react';
import { cn } from '../../lib/utils';

export interface DescriptionItem {
  label: React.ReactNode;
  value: React.ReactNode;
}

export interface DescriptionListProps extends React.HTMLAttributes<HTMLDListElement> {
  items: DescriptionItem[];
  layout?: 'horizontal' | 'stacked';
  ref?: React.Ref<HTMLDListElement>;
}

export const DescriptionList: React.FC<DescriptionListProps> = ({
  items,
  layout = 'horizontal',
  className,
  ref,
  ...props
}) => {
  return (
    <dl
      ref={ref}
      className={cn('divide-y divide-border text-xs sm:text-sm', className)}
      {...props}
    >
      {items.map((item, idx) => (
        <div
          key={idx}
          className={cn(
            'py-2.5',
            layout === 'horizontal'
              ? 'flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1'
              : 'flex flex-col gap-1'
          )}
        >
          <dt className="font-medium text-secondary">{item.label}</dt>
          <dd className="text-primary font-medium sm:text-right">{item.value}</dd>
        </div>
      ))}
    </dl>
  );
};
