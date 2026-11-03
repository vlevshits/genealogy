# UX Review — Phase 3 Milestone 3.1: Printable PDF Studio Expansion — 2026-11-02 (Re-Review)

## Verdict: PASSED

Screens & routes reviewed:
- `/export/print` (Printable PDF Studio — Expanded Dual-Pane Studio Layout: interactive configuration sidebar, All Nodes complete forest layout, Hourglass Wide vs Compact density toggle, marital and sibling inclusion controls, circular avatar portraits, custom Title & Subtitle banner inputs, and high-resolution vector PDF download pipeline).

Components inspected:
- `frontend/src/pages/PrintExportPage.tsx`
- `frontend/src/components/print/PrintConfigSidebar.tsx`
- `frontend/src/components/print/PrintPreviewCanvas.tsx`
- `frontend/src/components/print/PrintSummaryBar.tsx`
- `frontend/src/components/domain/person-picker.tsx`
- `frontend/src/components/ui/segmented-control.tsx`
- `frontend/src/components/ui/input.tsx`
- `frontend/src/i18n/en.json` & `ru.json`
- `frontend/tests/components/print-studio.test.tsx`

Viewports & Locales Evaluated:
- Desktop (1440px), Laptop (1280px), Tablet (768px), Mobile (390px).
- Light and Dark themes.
- English (`en`) and Russian (`ru`).

---

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-001** | Fix mobile toolbar overlap and bottom navigation occlusion on tree canvas | fixed |
| **UX-007** | Fix `BottomNav` accessibility label and replace raw 36px buttons | fixed |
| **UX-021** | Keyboard zoom shortcuts or visible hint in `PlacesMapView` and `FanChartCanvas` | suggestion (open) |
| **UX-022** | DropdownMenu triggers (`AppShell` nav more, +add, user profile) inaccessible to keyboard navigation | open |
| **UX-023** | Modal Dialogs and Drawers lack keyboard Tab focus trapping and initial focus management | open |
| **UX-024** | Sub-40px touch targets and raw `<button>` elements in inputs and media controls | open |
| **UX-036** | Form error alerts lack programmatic association (`aria-invalid`, `aria-describedby`) with invalid fields | open |
| **UX-037** | Stacking / Overflow collision in `TreeToolbar` on tablet and medium viewports (640px - 1024px) | fixed |
| **UX-038** | Direct-lineage vertical stems overwritten by collateral SVG paths | fixed |
| **UX-039** | Low non-text contrast (< 3:1) for collateral connector edges in dark mode | fixed |
| **UX-040** | Initial camera pan hardcoded to `{ x: 480, y: 280 }`, placing tree root off-screen on mobile screens | fixed |
| **UX-078** | Unconfirmed Destructive Action on Media Asset Deletion (Data Loss Risk) | fixed |
| **UX-079** | Overlapping Collision of Person Name and "Primary Avatar" / "Associated" Badges on Mobile Viewports | fixed |
| **UX-080** | Keyboard & Screen Reader Inaccessible Auto-Detected Face Candidate Rectangles | fixed |
| **UX-081** | Target Person Armed Banner Truncates and Hides Relative's Name on Mobile Viewports | fixed |
| **UX-082** | Sub-40px Touch Targets on Studio Navigation, Move/Resize Handles, and Panel Buttons | fixed |
| **UX-083** | Duplicate "Auto-Detect Faces" Buttons Rendered Simultaneously on Canvas Stage | fixed |
| **UX-084** | Auto-Detected Face Candidates Lack Dismiss / Reject Affordance for False Detections | fixed |
| **UX-085** | Low Contrast on Sky-Blue Candidate Rectangles (`border-sky-400`) in Light Theme | fixed |
| **UX-086** | WAI-ARIA Structure Violation: Nested `<button>` Inside `role="option"` and Missing Arrow Key Navigation | fixed |
| **UX-087** | Mobile & Tablet (<1024px) Preview Canvas Completely Unreachable Due to `h-full` Sidebar in `overflow-hidden` Container | fixed |
| **UX-088** | Mobile BottomNav Occludes Bottom Interactive Controls & Download CTA in Print Sidebar | fixed |
| **UX-089** | Triple Scrollbar Collision & Window Scroll Overflow from `h-[calc(100vh-3.5rem)]` and `AppShell` Footer | fixed |
| **UX-090** | Hardcoded English `(Living)` Suffix and Transliterated Ancestor Names in Russian Locale | fixed |
| **UX-091** | Fixed Canvas Dimensions (880×620px) Distort Paper Aspect Ratio and Force Horizontal Scroll on 1280px Screens | fixed |
| **UX-092** | Screen Reader Pollution from Decorative Alignment Crosshairs (`+` Glyphs) and Missing Accessible Label on Canvas Region | fixed |
| **UX-093** | Missing Programmatic Accessible Label on Chart Title Input (`TextInput`) | fixed |
| **UX-094** | Duplicate Primary Action Buttons and Non-Sticky Sidebar CTA | fixed |
| **UX-095** | Russian Toolbar Text Wrapping and Slavic Pluralization Errors in Sheet & Generation Counters | fixed |
| **UX-096** | Add "Fit to Window" Zoom Preset and Canvas Pan Affordance | fixed |
| **UX-097** | Residual 38px Document Scrollbar from Breadcrumbs Bar Height Collision with `h-[calc(100dvh-4rem)]` | fixed |
| **UX-098** | Chart Layout `SegmentedControl` Horizontally Overflows Sidebar & Mobile Viewports | fixed |
| **UX-099** | Hardcoded English Sheet Tags and Guide Indicator in SVG Preview Canvas | fixed |
| **UX-100** | Missing Programmatic Label on `PersonPicker` & Missing `aria-describedby` on Subtitle Input | fixed |
| **UX-101** | Add Reset / Clear Affordance for Custom Title and Subtitle Inputs | fixed |

---

## Tasks for frontend-developer

### UX-098 [major] Chart Layout `SegmentedControl` Horizontally Overflows Sidebar & Mobile Viewports
- **Screen / viewport / theme / lang:** `/export/print` (`PrintConfigSidebar.tsx:216-227`, `segmented-control.tsx:112-120`), desktop sidebar (384px / `w-96`), tablet (768px), mobile viewport (390px), all themes and locales.
- **Problem:** `SegmentedControl` renders 4 items (`pedigree`, `hourglass`, `descendants`, `all_nodes`) with full descriptive text strings (e.g., `"All Nodes (Complete Forest)"` / `"Все персоны (Полное древо)"`). In an unconstrained `inline-flex` layout without wrapping, the 4 items total ~570px in English and ~655px in Russian, which far exceeds the available sidebar inner width of 344px (and 358px on mobile). Because the sidebar container has `overflow-y-auto`, the browser defaults `overflow-x` to `auto`, causing horizontal overflow, an unwanted horizontal scrollbar across the configuration sidebar, or severely squeezed text. Violates WCAG 1.4.10 (Reflow).
- **Evidence:** `frontend/src/components/print/PrintConfigSidebar.tsx:225`.
- **Proposed fix:** Set `className="w-full grid grid-cols-2"` on `SegmentedControl` (or wrap it in an `overflow-x-auto scrollbar-thin` container like `TreeToolbar.tsx:74`). A 2×2 grid layout arranges the 4 options cleanly in two rows of two buttons, fitting comfortably within 344px without horizontal overflow.
- **Status:** **FIXED** (Configured with `className="w-full grid grid-cols-2 gap-1 items-stretch"` in `PrintConfigSidebar.tsx:225` and updated button styles with `h-full [&>svg]:shrink-0` in `segmented-control.tsx:113-120`. Arranges 4 layout choices into an aligned 2×2 grid cleanly fitting within sidebar and mobile width).
- **Acceptance criteria:**
  - [x] `SegmentedControl` fits completely within the 384px sidebar and 390px mobile viewport without causing horizontal scrolling on the sidebar or page.
  - [x] All 4 options are easily legible and tappable without text clipping.
- **Tags:** layout, space, a11y

### UX-099 [major] Hardcoded English Sheet Tags and Guide Indicator in SVG Preview Canvas
- **Screen / viewport / theme / lang:** `/export/print` (`PrintPreviewCanvas.tsx:782-788, 808`), tiled printing mode, Russian (`ru`) locale, all viewports.
- **Problem:** The preview canvas hardcodes English text strings directly inside the SVG DOM: `"Sheet {sheetNumber} of {totalSheets} [Row {row + 1}, Col {col + 1}]"` and `"Sheet 1: Assembly Guide Included"`. Even when the application is set to Russian, users see English labels on each tiled sheet coordinate badge and assembly guide tag. Existing localized strings `print.sheetTag` and `print.sheetGuideTag` already exist in `ru.json` (`"Лист {num} из {total} [Ряд {row}, Колонка {col}]"` and `"Лист 1: Схема сборки включена"`) but are currently bypassed.
- **Evidence:** `frontend/src/components/print/PrintPreviewCanvas.tsx:782-788` and `808`.
- **Proposed fix:** Replace hardcoded strings with `t('print.sheetTag', { num: sheetNumber, total: totalSheets, row: row + 1, col: col + 1 })` and `t('print.sheetGuideTag')`.
- **Status:** **FIXED** (Replaced hardcoded English sheet tag and guide strings in `PrintPreviewCanvas.tsx:782-788, 808` with `t('print.sheetTag', { num: sheetNumber, total: totalSheets, row: row + 1, col: col + 1 })` and `t('print.sheetGuideTag')`. Localized badges verified in Russian and English locales via vitest).
- **Acceptance criteria:**
  - [x] In Russian locale, each tiled sheet coordinate tag displays `"Лист X из Y [Ряд R, Колонка C]"`.
  - [x] In Russian locale, Sheet 1 assembly guide displays `"Лист 1: Схема сборки включена"`.
  - [x] In English locale, strings display correctly with parameters.
- **Tags:** copy, consistency

### UX-100 [minor] Missing Programmatic Label on `PersonPicker` & Missing `aria-describedby` on Subtitle Input
- **Screen / viewport / theme / lang:** `/export/print` (`PrintConfigSidebar.tsx:205, 461, 482`, `person-picker.tsx:18, 31, 68`), all viewports and themes.
- **Problem:**
  1. The `PersonPicker` component receives no `ariaLabel` prop, meaning screen reader users only hear generic combobox announcements rather than `"Focal Person"` / `"Центральная персона"`.
  2. The custom Subtitle/Description text input (`id="print-chart-subtitle-input"`) is followed by an explanatory note (`"Leave blank for automatic layout summary"`), but the `<p>` has no `id` and the `TextInput` has no `aria-describedby`, so screen readers do not announce the hint upon focus (WCAG 1.3.1 Info and Relationships, 4.1.2 Name, Role, Value).
- **Evidence:** `frontend/src/components/print/PrintConfigSidebar.tsx:205, 461, 482`.
- **Proposed fix:**
  1. Pass `ariaLabel={t('print.focalPerson')}` to `PersonPicker` (and pass it through to `Combobox` in `PersonPicker.tsx`).
  2. Add `id="print-chart-subtitle-hint"` to `<p className="text-[11px] text-tertiary">` and `aria-describedby="print-chart-subtitle-hint"` to `<TextInput>`.
- **Status:** **FIXED** (Added `ariaLabel={t('print.focalPerson')}` prop to `PersonPicker` in `PrintConfigSidebar.tsx:205` forwarded to `Combobox` in `person-picker.tsx:18, 31, 68`. Added `aria-describedby="print-chart-subtitle-hint"` to Subtitle `TextInput` and `id="print-chart-subtitle-hint"` to description `<p>` in `PrintConfigSidebar.tsx:461, 482`).
- **Acceptance criteria:**
  - [x] Focusing the Focal Person picker announces its accessible role and name.
  - [x] Focusing the Subtitle input announces the description helper hint.
- **Tags:** a11y

### UX-101 [suggestion] Add Reset / Clear Affordance for Custom Title and Subtitle Inputs
- **Screen / viewport / theme / lang:** `/export/print` (`PrintConfigSidebar.tsx:436-448, 468-480`), all viewports.
- **Problem:** When a user types a custom title or subtitle, the live canvas updates immediately. If the user wants to revert back to the smart auto-generated title (e.g. `"The Ancestors of Aleksandr Petrov"`) or auto-generated metadata subtitle, they must manually backspace all characters in the text input until empty.
- **Evidence:** `frontend/src/components/print/PrintConfigSidebar.tsx:436-448, 468-480`.
- **Proposed fix:** Provide a small clear icon button (`X`) or "Reset to default" action in or near the inputs when `config.title` or `config.subtitle` is non-empty.
- **Status:** **FIXED** (Added clear (`X`) `IconButton` in `rightSlot` for Title and Subtitle `TextInput` when non-empty in `PrintConfigSidebar.tsx:436-448, 468-480`. Clicking resets values to `undefined` and restores dynamic fallbacks).
- **Acceptance criteria:**
  - [x] User can click clear to immediately restore the auto-generated fallback title/subtitle.
- **Tags:** hierarchy, states

---

## Positive Notes & Accessibility Assessment
- **Seamless All Nodes Integration:** Selecting All Nodes replaces the generation depth slider with a clear branch archive summary badge (`allNodesDesc`), and the preview canvas faithfully renders the entire multi-component forest and singletons shelf using 1:1 server millimeter coordinates.
- **Rich Visual Elements:** Circular photo portraits are rendered with precise SVG `<clipPath>` elements and fall back to sex-colored lineage vectors when absent or redacted. Marital connectors utilize warm amber strokes (`stroke-amber-600 dark:stroke-amber-500`) with high contrast ($\ge 3.3:1$ in light mode, $9.6:1$ in dark mode).
- **Graceful Fan Chart Compatibility Guard:** When Fan Chart mode is loaded, unsupported sibling, partner, and collateral density controls are cleanly disabled with an informative amber hint banner (`fanDisabledHint`).
- **Responsive Dual-Pane Architecture:** Mobile view switcher tab (`SegmentedControl`) cleanly toggles between configuration and live preview at 390px, and sticky download CTA retains comfortable thumb clearance above `BottomNav`.
- **Strict Design System Adherence:** Zero raw `<button>`, `<input>`, or `<select>` HTML elements across all studio files, with 100% i18n parity between `en.json` and `ru.json` and zero emojis or glyphs in translation keys.
