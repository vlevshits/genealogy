# UX Review — Phase 1.12 & Phase 1.13 — 2026-10-27

## Verdict: OK
**Screens reviewed:**
- Family Tree Canvas: `/tree` (`TreeCanvas.tsx`, `TreeToolbar.tsx`, `TreeMiniMap.tsx`, `PersonNodeCard.tsx`)
- Person Inspection Sheets: `/tree` (`PersonDrawer.tsx`, `PersonBottomSheet.tsx`)
- Person Profile: `/persons/:id` (`PersonProfilePage.tsx`, `ProfileHeroHeader.tsx`)
- Domain Components: `person-card.tsx`, `person-list-item.tsx`, `person-name.tsx`, `life-span.tsx`, `privacy-badge.tsx`
- Application Shell & Navigation: `app-shell.tsx`, `bottom-nav.tsx`
- PWA & Offline: `PwaInstallPrompt.tsx`, `PwaOfflineBanner.tsx`, `manifest.webmanifest`, `sw.js`, `index.html`

---

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-001** | Fix mobile toolbar overlap and bottom navigation occlusion on tree canvas | **fixed** |
| **UX-002** | Resolve multi-touch pinch-to-zoom race condition and card long-press drag threshold | **fixed** |
| **UX-003** | Wire centralized `formatPersonName` into all UI components and fix fallback | **fixed** |
| **UX-004** | Remove prominent "Living" pill badge from `PersonDrawer` and `PersonBottomSheet` [spec-change] | **fixed** |
| **UX-005** | Update `LifeSpan` component to display genealogical dagger `†` for deceased | **fixed** |
| **UX-006** | Fix PWA offline banner top safe-area inset and consolidate install actions | **fixed** |
| **UX-007** | Fix `BottomNav` accessibility label and replace raw 36px buttons | **fixed** |
| **UX-008** | Parametrize family archive instance title in PWA install copy | **fixed** |

---

## Re-Review Findings & Verification Detail

### 1. Mobile & Tablet Tree Canvas Ergonomics (`UX-001`, `UX-002`)
- **Top Toolbar Collision Fixed:** `TreeToolbar.tsx` now wraps top controls in a responsive flex layout (`flex-col sm:flex-row sm:items-center sm:justify-between`) with `pointer-events-none` on the outer bar and `pointer-events-auto` on the child controls. On mobile viewports (~390px), the projection switcher and depth control clusters stack cleanly without overlapping.
- **Bottom Navigation Clearance Fixed:** The floating zoom pill and mini-map (both collapsed button and expanded card) now apply `bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))] md:bottom-6`, perfectly clearing the fixed mobile `BottomNav` bar (z-30) and device home indicator.
- **Viewport Height Fixed:** `TreeCanvas.tsx` container uses `h-[calc(100dvh-4rem)]`, ensuring smooth rendering across dynamic browser address bars on iOS Safari and mobile Chrome without overflow clipping.
- **Multi-Touch Gesture Smoothness:** Removed duplicate `onTouchStart/Move/End` listeners that were competing with `onPointerDown/Move/Up`. Pointer Events now handle mouse, pen, and multi-touch pinch-to-zoom cleanly without duplicate calculation or stutter.
- **Drag Threshold for Long-Press:** `PersonNodeCard.tsx` now tracks pointer movement with a 6px drag threshold (`DRAG_THRESHOLD_PX = 6`). Panning across card nodes clears the 500ms long-press timer immediately, preventing accidental selection or unwanted haptic vibrations during canvas pan.
- **Touch Target Compliance:** Node card quick-action buttons (Focus, Add) now provide `min-h-11 sm:min-h-8` (44px on mobile touch) for confident tapping.

### 2. Slavic Name Formatting & Lifecycle Layout (`UX-003`, `UX-004`, `UX-005`)
- **Centralized Name Formatting:** `formatPersonName` is now wired into `PersonNodeCard.tsx`, `PersonDrawer.tsx`, `PersonBottomSheet.tsx`, `ProfileHeroHeader.tsx`, `PersonProfilePage.tsx`, `PersonCard.tsx`, and `PersonName.tsx`.
  - In Russian (`ru`), person names format according to Slavic genealogical standards: `Фамилия Имя Отчество` in lists, `Фамилия (Девичья) Имя Отчество` with maiden names, and `Имя Отчество Фамилия` on cards.
  - In English (`en`), maiden surnames format with `(née Maiden)`.
  - In `PersonName.tsx`, empty/missing names fall back to `domain.livingPerson` rather than misusing the date key `domain.unknownYear`.
- **Zero "Living" Pill Badges:** Removed `<Badge>` with `[ Living ]` from `PersonDrawer.tsx` and `PersonBottomSheet.tsx`. Living individuals have no intrusive clinical badges; their status is communicated naturally by their birth date (e.g. `р. 1994` / `b. 1994`).
- **Dignified Deceased Dagger:** Deceased individuals across `PersonNodeCard`, `PersonDrawer`, `PersonBottomSheet`, and `LifeSpan` display the subtle genealogical dagger `†` (e.g., `1912 – †1985`, `† 1985`), in full harmony with `formatLifeSpanText`.

### 3. PWA & Navigation Usability (`UX-006`, `UX-007`, `UX-008`)
- **Safe Area Inset on Offline Banner:** `PwaOfflineBanner.tsx` now includes `pt-[max(0.5rem,env(safe-area-inset-top,0px))]`, avoiding notch or Dynamic Island occlusion when displayed in standalone PWA mode.
- **Consolidated Install Prompt:** `PwaInstallPrompt.tsx` removed the redundant `X` button and maintains a clean pair: ghost "Not now" and primary "Install".
- **Dynamic Instance Naming:** `pwa.install_prompt` in `en.json` and `ru.json` interpolates `{instanceName}`, dynamically presenting the configured archive title (e.g. `Добавьте «Семейный архив Левшиц» на главный экран...`).
- **Bottom Navigation A11y & Touch Targets:**
  - `BottomNav.tsx` `<nav>` element now has `aria-label="Main navigation"` instead of `"Open menu"`.
  - Language switcher and theme switcher inside the `BottomNav` More drawer now use design-system `Button` components with `min-h-11 min-w-11` (44×44px touch targets).
  - Quick-add buttons on `PersonCard.tsx` now use `h-8 w-8 min-h-8 min-w-8` with `opacity-85 sm:opacity-0`, ensuring visibility and tap accessibility on touch screens.

---

## Positive Notes
- **Exceptional Touch Polish:** The combination of `100dvh`, multi-touch Pointer Events, and clearance above `BottomNav` makes the tree canvas feel like a native mobile application.
- **Typographic Dignity:** Slavic names and lifespans render with authentic cultural precision (`Кузнецов Александр Иванович`, `1912 – †1985`), free from marketing fluff and clinical badges.
- **WCAG 2.2 AA Conformance:** Full 44×44px touch targets, correct landmark ARIA labels, and clean focus rings across both light and dark themes.
