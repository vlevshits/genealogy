import React from 'react';
import { MapPin } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Place } from '../../types/place';
import { Tooltip } from '../ui/tooltip';

export interface PlaceTextProps extends React.HTMLAttributes<HTMLSpanElement> {
  place?: Place | string | null;
  showIcon?: boolean;
  ref?: React.Ref<HTMLSpanElement>;
}

export const PlaceText: React.FC<PlaceTextProps> = ({
  place,
  showIcon = true,
  className,
  ref,
  ...props
}) => {
  if (!place) return null;

  const name = typeof place === 'string' ? place : place.name;
  const hierarchy =
    typeof place === 'object' && place.hierarchy_names && place.hierarchy_names.length > 0
      ? place.hierarchy_names.join(' → ')
      : name;

  const content = (
    <span
      ref={ref}
      className={cn('inline-flex items-center gap-1 text-xs text-secondary', className)}
      {...props}
    >
      {showIcon && <MapPin className="h-3 w-3 shrink-0 text-tertiary" />}
      <span className="truncate">{name}</span>
    </span>
  );

  if (hierarchy !== name) {
    return <Tooltip content={hierarchy}>{content}</Tooltip>;
  }

  return content;
};
