import { useEffect, useId, useRef } from 'react';

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

// Module-scoped stack of currently open overlay IDs in mount order so stacked
// Dialogs and Drawers only handle Escape for the topmost overlay.
const overlayStack: string[] = [];

interface InertRecord {
  count: number;
  prevAriaHidden: string | null;
  hadInert: boolean;
}

const inertRecords = new WeakMap<HTMLElement, InertRecord>();

export function registerOverlay(id: string): void {
  const existingIdx = overlayStack.indexOf(id);
  if (existingIdx !== -1) {
    overlayStack.splice(existingIdx, 1);
  }
  overlayStack.push(id);
}

export function unregisterOverlay(id: string): void {
  const idx = overlayStack.indexOf(id);
  if (idx !== -1) {
    overlayStack.splice(idx, 1);
  }
}

export function isTopOverlay(id: string): boolean {
  return overlayStack.length > 0 && overlayStack[overlayStack.length - 1] === id;
}

export function getActiveOverlayStack(): readonly string[] {
  return [...overlayStack];
}

function retainInert(el: HTMLElement): void {
  const existing = inertRecords.get(el);
  if (existing) {
    existing.count += 1;
    return;
  }
  inertRecords.set(el, {
    count: 1,
    prevAriaHidden: el.getAttribute('aria-hidden'),
    hadInert: el.hasAttribute('inert'),
  });
  el.setAttribute('aria-hidden', 'true');
  el.setAttribute('inert', '');
}

function releaseInert(el: HTMLElement): void {
  const existing = inertRecords.get(el);
  if (!existing) return;
  existing.count -= 1;
  if (existing.count > 0) return;

  inertRecords.delete(el);
  if (existing.prevAriaHidden === null) {
    el.removeAttribute('aria-hidden');
  } else {
    el.setAttribute('aria-hidden', existing.prevAriaHidden);
  }
  if (!existing.hadInert) {
    el.removeAttribute('inert');
  }
}

export interface UseModalOverlayOptions {
  open: boolean;
  modal?: boolean;
  dismissible?: boolean;
  overlayRef: React.RefObject<HTMLElement | null>;
  panelRef: React.RefObject<HTMLElement | null>;
  onOpenChange: (open: boolean) => void;
}

export function useModalOverlay({
  open,
  modal = true,
  dismissible = true,
  overlayRef,
  panelRef,
  onOpenChange,
}: UseModalOverlayOptions): void {
  const overlayId = useId();
  const previousActiveRef = useRef<HTMLElement | null>(null);
  const onOpenChangeRef = useRef(onOpenChange);
  onOpenChangeRef.current = onOpenChange;

  useEffect(() => {
    if (!open || typeof document === 'undefined') return;

    registerOverlay(overlayId);

    const lockedSiblings: HTMLElement[] = [];
    if (modal) {
      previousActiveRef.current = document.activeElement as HTMLElement | null;

      let current: HTMLElement | null = overlayRef.current;
      while (current && current.parentElement && current !== document.body) {
        const parent = current.parentElement;
        for (const child of Array.from(parent.children)) {
          if (!(child instanceof HTMLElement) || child === current) continue;
          if (child.tagName === 'SCRIPT' || child.tagName === 'STYLE') continue;
          if (
            child.hasAttribute('data-toast-region') ||
            child.querySelector('[data-toast-region]')
          ) {
            continue;
          }
          retainInert(child);
          lockedSiblings.push(child);
        }
        current = parent;
      }

      if (panelRef.current) {
        const focusables = getFocusableElements(panelRef.current);
        const target = focusables[0] ?? panelRef.current;
        target.focus();
      }
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isTopOverlay(overlayId)) return;
      if (e.key === 'Escape' && dismissible) {
        e.preventDefault();
        e.stopPropagation();
        onOpenChangeRef.current(false);
        return;
      }
      if (modal) {
        handleDialogFocusTrapKeyDown(e, panelRef.current);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      unregisterOverlay(overlayId);

      // Release inert markers BEFORE restoring focus so returning focus to a parent
      // overlay control succeeds rather than hitting an inert ancestor.
      for (const el of lockedSiblings) {
        releaseInert(el);
      }
      if (modal) {
        previousActiveRef.current?.focus?.();
      }
    };
  }, [open, modal, dismissible, overlayId, overlayRef, panelRef]);
}
