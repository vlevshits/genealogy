import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';
import { cn } from '../../lib/utils';

export interface CollapsibleProps {
  title: React.ReactNode;
  defaultOpen?: boolean;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  persistKey?: string;
  children: React.ReactNode;
  className?: string;
}

export const Collapsible: React.FC<CollapsibleProps> = ({
  title,
  defaultOpen = false,
  open,
  onOpenChange,
  persistKey,
  children,
  className,
}) => {
  const [internalOpen, setInternalOpen] = useState<boolean>(() => {
    if (persistKey && typeof window !== 'undefined') {
      const saved = localStorage.getItem(`collapsible_${persistKey}`);
      if (saved === 'true') return true;
      if (saved === 'false') return false;
    }
    return defaultOpen;
  });

  const isOpen = open !== undefined ? open : internalOpen;

  const toggle = () => {
    const next = !isOpen;
    if (open === undefined) setInternalOpen(next);
    if (persistKey && typeof window !== 'undefined') {
      localStorage.setItem(`collapsible_${persistKey}`, String(next));
    }
    onOpenChange?.(next);
  };

  return (
    <div className={cn('rounded-xl border border-border bg-surface', className)}>
      <button
        type="button"
        aria-expanded={isOpen}
        onClick={toggle}
        className="flex w-full items-center justify-between gap-3 px-4 py-3 text-left text-xs sm:text-sm font-semibold text-primary hover:bg-surface-muted/50 rounded-xl transition-colors cursor-pointer"
      >
        <span>{title}</span>
        <ChevronDown
          className={cn(
            'h-4 w-4 text-secondary transition-transform duration-200',
            isOpen && 'rotate-180'
          )}
        />
      </button>
      {isOpen && <div className="border-t border-border px-4 py-3 text-xs sm:text-sm">{children}</div>}
    </div>
  );
};
