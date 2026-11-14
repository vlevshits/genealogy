import React, { useCallback, useEffect, useId, useRef, useState } from 'react';
import { X } from 'lucide-react';
import { cn } from '../../lib/utils';
import { IconButton } from './icon-button';
import { useI18n } from '../../i18n';

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(', ');

export function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const candidates = Array.from(
    container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)
  );
  return candidates.filter((el) => {
    if (el.hasAttribute('disabled') || el.getAttribute('aria-hidden') === 'true') {
      return false;
    }
    const hiddenAncestor = el.closest('[hidden], [inert], [aria-hidden="true"]');
    if (hiddenAncestor && hiddenAncestor !== container && container.contains(hiddenAncestor)) {
      return false;
    }
    if (typeof window !== 'undefined' && typeof window.getComputedStyle === 'function') {
      const style = window.getComputedStyle(el);
      if (style.display === 'none' || style.visibility === 'hidden') {
        return false;
      }
    }
    return true;
  });
}

export function handleDialogFocusTrapKeyDown(
  e: Pick<KeyboardEvent, 'key' | 'shiftKey' | 'preventDefault'>,
  container: HTMLElement | null,
  activeElement: Element | null = typeof document !== 'undefined'
    ? document.activeElement
    : null
): boolean {
  if (e.key !== 'Tab' || !container) return false;
  const focusables = getFocusableElements(container);
  if (focusables.length === 0) {
    e.preventDefault();
    container.focus();
    return true;
  }
  const first = focusables[0];
  const last = focusables[focusables.length - 1];
  if (e.shiftKey) {
    if (!activeElement || activeElement === first || !container.contains(activeElement)) {
      e.preventDefault();
      last.focus();
      return true;
    }
  } else if (!activeElement || activeElement === last || !container.contains(activeElement)) {
    e.preventDefault();
    first.focus();
    return true;
  }
  return false;
}

export type DialogSize = 'sm' | 'md' | 'xl' | 'full';

export interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  size?: DialogSize;
  dismissible?: boolean;
  title?: string;
  description?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  className?: string;
}

export const Dialog: React.FC<DialogProps> = ({
  open,
  onOpenChange,
  size = 'md',
  dismissible = true,
  title,
  description,
  children,
  footer,
  className,
}) => {
  const { t } = useI18n();
  const titleId = useId();
  const descId = useId();
  const overlayRef = useRef<HTMLDivElement | null>(null);
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const bodyRef = useRef<HTMLDivElement | null>(null);
  const previousActiveRef = useRef<HTMLElement | null>(null);
  const onOpenChangeRef = useRef(onOpenChange);
  onOpenChangeRef.current = onOpenChange;
  const [canScrollDown, setCanScrollDown] = useState(false);

  const updateScrollCue = useCallback(() => {
    const el = bodyRef.current;
    if (!el) {
      setCanScrollDown(false);
      return;
    }
    setCanScrollDown(el.scrollHeight - el.scrollTop - el.clientHeight > 6);
  }, []);

  useEffect(() => {
    if (!open || typeof document === 'undefined') return;
    previousActiveRef.current = document.activeElement as HTMLElement | null;

    // Mark background sibling subtrees outside this dialog overlay as inert & aria-hidden
    // so neither keyboard Tab nor assistive technologies escape behind the active modal.
    const modifiedSiblings: Array<{
      el: HTMLElement;
      prevAriaHidden: string | null;
      hadInert: boolean;
    }> = [];
    let current: HTMLElement | null = overlayRef.current;
    while (current && current.parentElement && current !== document.body) {
      const parent = current.parentElement;
      for (const child of Array.from(parent.children)) {
        if (!(child instanceof HTMLElement) || child === current) continue;
        if (child.tagName === 'SCRIPT' || child.tagName === 'STYLE') continue;
        if (child.hasAttribute('data-toast-region')) continue;
        modifiedSiblings.push({
          el: child,
          prevAriaHidden: child.getAttribute('aria-hidden'),
          hadInert: child.hasAttribute('inert'),
        });
        child.setAttribute('aria-hidden', 'true');
        child.setAttribute('inert', '');
      }
      current = parent;
    }

    if (dialogRef.current) {
      const focusables = getFocusableElements(dialogRef.current);
      const target = focusables[0] ?? dialogRef.current;
      target.focus();
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && dismissible) {
        e.preventDefault();
        onOpenChangeRef.current(false);
        return;
      }
      handleDialogFocusTrapKeyDown(e, dialogRef.current);
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      for (const { el, prevAriaHidden, hadInert } of modifiedSiblings) {
        if (prevAriaHidden === null) {
          el.removeAttribute('aria-hidden');
        } else {
          el.setAttribute('aria-hidden', prevAriaHidden);
        }
        if (!hadInert) {
          el.removeAttribute('inert');
        }
      }
      previousActiveRef.current?.focus?.();
    };
  }, [open, dismissible]);

  useEffect(() => {
    if (!open) {
      setCanScrollDown(false);
      return;
    }
    const raf =
      typeof requestAnimationFrame === 'function'
        ? requestAnimationFrame(updateScrollCue)
        : 0;
    updateScrollCue();
    const el = bodyRef.current;
    let observer: ResizeObserver | null = null;
    if (el && typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(updateScrollCue);
      observer.observe(el);
    }
    if (typeof window !== 'undefined') {
      window.addEventListener('resize', updateScrollCue);
    }
    return () => {
      if (raf && typeof cancelAnimationFrame === 'function') {
        cancelAnimationFrame(raf);
      }
      observer?.disconnect();
      if (typeof window !== 'undefined') {
        window.removeEventListener('resize', updateScrollCue);
      }
    };
  }, [open, children, updateScrollCue]);

  if (!open) return null;

  const sizeClasses: Record<DialogSize, string> = {
    sm: 'max-w-[400px] w-full rounded-2xl',
    md: 'w-full h-full md:h-auto md:max-w-[560px] md:rounded-2xl',
    xl: 'w-full h-full md:h-auto md:max-w-[800px] md:rounded-2xl',
    full: 'w-[95vw] h-[90vh] rounded-2xl',
  };

  return (
    <div
      ref={overlayRef}
      className="fixed inset-0 z-50 flex items-center justify-center p-0 md:p-4"
    >
      <div
        aria-hidden="true"
        onClick={() => dismissible && onOpenChange(false)}
        className="fixed inset-0 bg-primary/40 backdrop-blur-xs transition-opacity"
      />
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
        aria-labelledby={title ? titleId : undefined}
        aria-describedby={description ? descId : undefined}
        className={cn(
          'relative z-10 flex flex-col border border-border bg-surface text-primary shadow-xl max-h-screen md:max-h-[85vh] focus:outline-none',
          sizeClasses[size],
          className
        )}
      >
        {(title || dismissible) && (
          <div className="flex items-start justify-between gap-4 border-b border-border px-6 py-4">
            <div className="min-w-0">
              {title && (
                <h2 id={titleId} className="text-base font-semibold text-primary">
                  {title}
                </h2>
              )}
              {description && (
                <p id={descId} className="mt-1 text-xs text-secondary">
                  {description}
                </p>
              )}
            </div>
            {dismissible && (
              <IconButton
                size="sm"
                className="h-10 w-10 min-h-10 min-w-10"
                label={t('common.close')}
                onClick={() => onOpenChange(false)}
              >
                <X className="h-4 w-4" />
              </IconButton>
            )}
          </div>
        )}
        <div className="relative flex flex-1 min-h-0 flex-col">
          <div
            ref={bodyRef}
            onScroll={updateScrollCue}
            className="flex-1 overflow-y-auto px-6 py-4 text-sm"
          >
            {children}
          </div>
          {canScrollDown && (
            <div
              aria-hidden="true"
              data-testid="dialog-scroll-fade"
              className="pointer-events-none absolute inset-x-0 bottom-0 h-8 bg-gradient-to-t from-surface via-surface/85 to-transparent"
            />
          )}
        </div>
        {footer && (
          <div className="flex items-center justify-end gap-2.5 border-t border-border bg-surface-muted/40 px-6 py-3.5 rounded-b-2xl">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
};

export const DialogHeader: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('space-y-1', className)} {...props}>
    {children}
  </div>
);

export const DialogBody: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('space-y-4', className)} {...props}>
    {children}
  </div>
);

export const DialogFooter: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({
  className,
  children,
  ...props
}) => (
  <div className={cn('flex items-center justify-end gap-2', className)} {...props}>
    {children}
  </div>
);
