import { useLayoutEffect, useState, type CSSProperties, type RefObject } from 'react';

const GAP = 6;
const VIEWPORT_MARGIN = 8;
const MAX_MENU_HEIGHT = 288;
const MIN_MENU_HEIGHT = 160;

/**
 * Position a menu rendered in a portal (document.body) relative to an anchor,
 * so it is never clipped by an `overflow` ancestor such as a dialog body.
 * Flips above the anchor when there is more room there.
 */
export function useFloatingMenu(
  anchorRef: RefObject<HTMLElement | null>,
  open: boolean
): CSSProperties | undefined {
  const [style, setStyle] = useState<CSSProperties | undefined>(undefined);

  useLayoutEffect(() => {
    if (!open) {
      setStyle(undefined);
      return;
    }
    const update = () => {
      const el = anchorRef.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const below = window.innerHeight - rect.bottom - GAP - VIEWPORT_MARGIN;
      const above = rect.top - GAP - VIEWPORT_MARGIN;
      const flip = below < MIN_MENU_HEIGHT && above > below;
      const maxHeight = Math.max(
        120,
        Math.min(MAX_MENU_HEIGHT, flip ? above : below)
      );
      setStyle({
        position: 'fixed',
        left: rect.left,
        width: Math.max(rect.width, 180),
        maxHeight,
        ...(flip
          ? { bottom: window.innerHeight - rect.top + GAP }
          : { top: rect.bottom + GAP }),
      });
    };
    update();
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    return () => {
      window.removeEventListener('resize', update);
      window.removeEventListener('scroll', update, true);
    };
  }, [anchorRef, open]);

  return style;
}

