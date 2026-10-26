import React from 'react';
import { cn } from '../../lib/utils';

export interface NavLinkProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  active?: boolean;
  icon?: React.ReactNode;
  onNavigate?: (href: string) => void;
  ref?: React.Ref<HTMLAnchorElement>;
}

export const NavLink: React.FC<NavLinkProps> = ({
  href = '#',
  active = false,
  icon,
  onNavigate,
  onClick,
  className,
  children,
  ref,
  ...props
}) => {
  return (
    <a
      ref={ref}
      href={href}
      aria-current={active ? 'page' : undefined}
      onClick={(e) => {
        onClick?.(e);
        if (!e.defaultPrevented && onNavigate && !e.metaKey && !e.ctrlKey) {
          e.preventDefault();
          onNavigate(href);
        }
      }}
      className={cn(
        'inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
        active
          ? 'bg-surface-muted text-primary font-semibold'
          : 'text-secondary hover:bg-surface-muted/70 hover:text-primary',
        className
      )}
      {...props}
    >
      {icon}
      <span>{children}</span>
    </a>
  );
};
