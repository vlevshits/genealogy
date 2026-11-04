import React from 'react';
import { User } from 'lucide-react';
import { cn } from '../../lib/utils';
import { Sex } from '../../types/common';

export type AvatarSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

export interface AvatarProps extends React.HTMLAttributes<HTMLSpanElement> {
  src?: string | null;
  alt?: string;
  name?: string;
  size?: AvatarSize;
  sex?: Sex;
  showGenderRing?: boolean;
  isLivingRedacted?: boolean;
  ref?: React.Ref<HTMLSpanElement>;
}

const SIZE_CLASSES: Record<AvatarSize, string> = {
  xs: 'h-6 w-6 text-[10px]',
  sm: 'h-8 w-8 text-xs',
  md: 'h-10 w-10 text-sm',
  lg: 'h-16 w-16 text-lg',
  xl: 'h-28 w-28 text-2xl',
};

function getInitials(name?: string): string {
  if (!name) return '';
  const cleaned = name.replace(/\([^)]*\)/g, '').trim();
  const parts = cleaned
    .split(/\s+/)
    .map((p) => p.replace(/^[^\p{L}\p{N}]+|[^\p{L}\p{N}]+$/gu, ''))
    .filter(Boolean);
  if (parts.length === 0) return '';
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
}

export const Avatar: React.FC<AvatarProps> = ({
  src,
  alt,
  name,
  size = 'md',
  sex = 'unknown',
  showGenderRing = false,
  isLivingRedacted = false,
  className,
  ref,
  ...props
}) => {
  const [imgError, setImgError] = React.useState(false);

  React.useEffect(() => {
    setImgError(false);
  }, [src]);

  const ringClass = showGenderRing
    ? sex === 'male'
      ? 'ring-2 ring-male ring-offset-2 ring-offset-surface'
      : sex === 'female'
        ? 'ring-2 ring-female ring-offset-2 ring-offset-surface'
        : 'ring-2 ring-neutral ring-offset-2 ring-offset-surface'
    : '';

  const bgAccent =
    sex === 'male'
      ? 'bg-male/15 text-male'
      : sex === 'female'
        ? 'bg-female/15 text-female'
        : 'bg-surface-muted text-secondary';

  const initials = getInitials(name);

  return (
    <span
      ref={ref}
      aria-label={alt || name || 'Avatar'}
      className={cn(
        'relative inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full border border-border font-semibold select-none',
        SIZE_CLASSES[size],
        bgAccent,
        ringClass,
        className
      )}
      {...props}
    >
      {isLivingRedacted ? (
        <User className="h-1/2 w-1/2 opacity-70" />
      ) : src && !imgError ? (
        <img
          src={src}
          alt={alt || name || ''}
          onError={() => setImgError(true)}
          className="h-full w-full object-cover"
        />
      ) : initials ? (
        <span>{initials}</span>
      ) : (
        <User className="h-1/2 w-1/2 opacity-70" />
      )}
    </span>
  );
};

export interface AvatarGroupProps {
  items: Array<{ id: string; name?: string; src?: string | null; sex?: Sex }>;
  max?: number;
  size?: AvatarSize;
  className?: string;
}

export const AvatarGroup: React.FC<AvatarGroupProps> = ({
  items,
  max = 4,
  size = 'sm',
  className,
}) => {
  const visible = items.slice(0, max);
  const overflow = items.length - visible.length;

  return (
    <div className={cn('flex items-center -space-x-2', className)}>
      {visible.map((item) => (
        <Avatar
          key={item.id}
          name={item.name}
          src={item.src}
          sex={item.sex}
          size={size}
          className="ring-2 ring-surface"
        />
      ))}
      {overflow > 0 && (
        <span
          className={cn(
            'inline-flex items-center justify-center rounded-full border border-border bg-surface-muted font-mono font-semibold text-secondary ring-2 ring-surface',
            SIZE_CLASSES[size]
          )}
        >
          +{overflow}
        </span>
      )}
    </div>
  );
};
