# UX Review — Change History Visualization & Localization (Feature 14) — 2026-11-14

## Verdict: CHANGES REQUESTED

Screens reviewed:
- `http://localhost:5173/history` (Populated feed, filters, Undo & Restore error/success toasts, Empty `200 OK`, Viewer/Editor `AccessDeniedCard` + mobile `BottomNav` More sheet)
- `http://localhost:5173/persons/550e8400-e29b-41d4-a716-446655440001` -> Profile `⋯` overflow menu -> `RecordHistoryDrawer` (Populated feed with `can_revert: true`/`false`, Restore confirmation `Dialog` + error `Alert`, Empty `200 OK []`, HTTP 500 error state)
- `http://localhost:5173/persons/550e8400-e29b-41d4-a716-446655440001` -> Edit Person (`PersonEditDialog`) -> `ConflictResolutionModal` (`409 Conflict` initial comparison, `Merge field by field` radio selection, retry error `conflict-error-alert`, stacked `Escape` handling)
- `http://localhost:5173/` (`/dashboard` `RecentActivityWidget` + Undo error toast) and `http://localhost:5173/settings/backups` (`TreeSnapshotsCard`)

Matrix inspected: Desktop `1440x900` & Mobile `390x844` · Light & Dark themes · EN & RU locales (real Chromium via Playwright with API fixtures; zero edits made to production code).

Recordings / captures inspected (stored in temporary scratch folder and cleaned up):
- `/history`: `hist_desktop_dark_ru_full`, `hist_empty_desktop_light_en`, `hist_rbac_editor_mobile_ru`, `ux_hist_desktop_undo_toast`, `ux_hist_mobile_dark_ru_top`
- `RecordHistoryDrawer` & overflow menu: `drawer_desktop_light_en`, `drawer_desktop_light_en_menu`, `drawer_desktop_dark_ru_restore_error`, `drawer_mobile_light_en`, `drawer_mobile_dark_ru`, `drawer_mobile_dark_ru_menu`, `drawer_empty_en`, `drawer_error_ru`
- `ConflictResolutionModal`: `conflict_desktop_light_en_initial`, `conflict_desktop_light_en_merge`, `conflict_desktop_light_en_error`, `conflict_desktop_dark_ru_merge`, `conflict_mobile_light_en_merge`, `conflict_mobile_dark_ru_merge`
- `/dashboard` & `/settings/backups`: `dash_desktop_light_en`, `dash_mobile_dark_ru`, `dash_mobile_dark_ru_undoerr`, `backups_desktop_light_en`, `backups_mobile_dark_ru`

---

## Status of Previous Tasks

Baseline reference: [`docs/ux-reviews/2026-11-14-duplicate-events-ux-review.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/2026-11-14-duplicate-events-ux-review.md) (highest existing task ID: `UX-137`).

| ID | Title | Status |
| :--- | :--- | :--- |
| `UX-121` | Add `touch-target-expand` to `Button size="xs"` and localize `SideNav` default `aria-label` (completing `UX-111` & `UX-112`) | **still open (partial)** (`touch-target-expand` is present on `Button size="xs"` and `sm`, `SideNav` outside this scope; `Switch`, `ConflictResolutionModal` radio cards/buttons, `Drawer` close button, and standalone history CTAs remain undersized — tracked in `UX-141`, `UX-142`, `UX-146`) |
| `UX-125` | Complete remaining fixes for `UX-115`, `UX-116`, and `UX-117` (raw codes in `/history` & `/quality`, offline fallbacks, Active Sessions empty state, raw `<button>` in Workbench) | **fixed on `/history` field labels** (`HistoryPage.tsx:597` now renders localized `formatFieldLabel(d.field, t)` without raw snake_case `(d.field)` suffixes; remaining exposed `tx-*`/`snap-*` IDs raised separately in `UX-147` and `UX-148`) |
| `UX-127`–`UX-136` | Duplicate Events `Dialog` focus trap, conflict cues, `Undo` toast wrapping, target sizing, `AppShell` `Appearance` translation | **fixed** (verified in previous audit; `Undo` toast multi-line wrap and mobile `BottomNav` clearance confirmed live on `/history` and `/dashboard`; related stacked-overlay `Escape` propagation bug and `Drawer` missing focus trap raised in `UX-138` and `UX-139`) |
| `UX-137` | Convert `< 768px` `Dialog` layout to docked bottom-sheet with safe-area padding | **still open (deferred to `TD-QUAL-01`)** ([`docs/technical-debt.md:38`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/technical-debt.md#L38)) |

---

## Tasks for frontend-developer

### UX-138 [major] Prevent `Escape` on stacked overlays (`ConflictResolutionModal` over `PersonEditDialog`, Restore `Dialog` over `RecordHistoryDrawer`) from closing all layers simultaneously

- **Screen / viewport / theme / lang:** `/persons/:id` -> `PersonEditDialog` -> `ConflictResolutionModal`; `/persons/:id` -> `RecordHistoryDrawer` -> Restore Confirmation `Dialog` (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** Both `Dialog` and `Drawer` register independent `document.addEventListener('keydown', ...)` `Escape` handlers without coordinating an overlay stack or restricting dismissal to the topmost overlay. Pressing `Escape` once while `ConflictResolutionModal` is open over `PersonEditDialog` closes **both** modals in the same event tick (`dialogsBeforeEscape: 2` -> `dialogsAfterOneEscape: 0`), wiping all unsaved profile edits silently. Likewise, pressing `Escape` inside the Restore Confirmation `Dialog` over `RecordHistoryDrawer` closes **both** the confirmation modal and the entire drawer (`confirmDialogsCount: 2` -> `0`). Additionally, `Dialog` restores `inert`/`aria-hidden` on outer siblings without checking if an underlying overlay remains open.
- **Evidence:**
  - `frontend/src/components/ui/dialog.tsx:135-171` (unconditional `onOpenChangeRef.current(false)` on `document` `keydown`, plus unconditional cleanup of `aria-hidden` / `inert`)
  - `frontend/src/components/ui/drawer.tsx:33-42` (unconditional `document.addEventListener('keydown', ...)` closing `Drawer` on `Escape`)
  - `frontend/src/components/person/PersonEditDialog.tsx:513-520` and `frontend/src/components/history/RecordHistoryDrawer.tsx:424-454`
  - Live browser check (`http://localhost:5173/persons/550e8400-e29b-41d4-a716-446655440001`): `stackedEscape` in both EN and RU showed `dialogsBeforeEscape: 2`, `dialogsAfterOneEscape: 0`, `conflictStillVisible: false`, `editStillVisible: false`; Restore confirmation over `RecordHistoryDrawer` showed `confirmDialogsCount: 2` -> `0` after one `Escape`.
- **Proposed fix:** Maintain a lightweight shared overlay stack (an ordered array of active modal/drawer symbols/IDs registered by both `Dialog` and `Drawer`). Only handle `Escape` when the current instance is the **topmost** entry in the stack (calling `e.stopPropagation()`), and only remove outer `inert`/`aria-hidden` markers when exiting the last modal layer while keeping the parent overlay accessible and restoring focus back into the parent overlay when a child modal closes.
- **Acceptance criteria:**
  - Pressing `Escape` while `ConflictResolutionModal` is open over `PersonEditDialog` closes only `ConflictResolutionModal` and leaves `PersonEditDialog` open with unsaved edits preserved.
  - Pressing `Escape` while the Restore confirmation `Dialog` is open over `RecordHistoryDrawer` closes only the confirmation `Dialog`, leaves `RecordHistoryDrawer` open, and returns focus to the `"Restore this version"` button.
  - Background document nodes remain `inert` whenever at least one modal overlay is still open.
- **Tags:** `a11y`, `states`, `consistency`

---

### UX-139 [major] Add initial focus placement, `Tab`/`Shift+Tab` focus trap, background `inert`, and focus restoration to `Drawer` (`RecordHistoryDrawer`)

- **Screen / viewport / theme / lang:** `/persons/:id` -> `RecordHistoryDrawer` and shared `Drawer` primitive (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** Opening `RecordHistoryDrawer` leaves `document.activeElement` on `<body>` (`activeInsideDrawer: false`). Pressing `Tab` cycles through **7 background page controls** behind the dimmed scrim (`Chronology & Map`, `Family & Relations`, `Media & Documents`, `Notes & Sources`, `Add Event`, `Edit`, etc.) before reaching the drawer's Close button on the 8th `Tab`. Closing the drawer also drops focus instead of returning it to the trigger. Violates WCAG 2.2 SC `2.4.3` (Focus Order), SC `2.1.2` / `1.3.1`, and [`docs/design-system/components.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md#L209-L211).
- **Evidence:**
  - `frontend/src/components/ui/drawer.tsx:19-42, 61-72` (no `drawerRef`, no `tabIndex={-1}`, no `previousActiveRef`, no `inert`/`aria-hidden` on background siblings when `modal === true`, no `Tab` wrap handler)
  - Live keyboard check (`http://localhost:5173/persons/550e8400-e29b-41d4-a716-446655440001`): in both Light EN and Dark RU, `activeInsideDrawer` was `false` immediately after opening `RecordHistoryDrawer`, and the first 7 `Tab` stops stayed `inside: false` on background `PersonProfilePage` controls.
- **Proposed fix:** Extract the focus-trap and `inert` sibling logic from `frontend/src/components/ui/dialog.tsx:16-66, 113-172` into a shared hook/helper and apply it to `Drawer` whenever `open && modal`: move initial focus into `drawerRef` (`tabIndex={-1}` fallback), trap `Tab`/`Shift+Tab` inside the drawer, mark background siblings `inert` / `aria-hidden="true"` (skipping `[data-toast-region]`), and restore focus to `previousActiveRef` on close.
- **Acceptance criteria:**
  - Opening `RecordHistoryDrawer` immediately moves `document.activeElement` inside `<aside role="dialog">`.
  - `Tab` and `Shift+Tab` loop strictly within the drawer's controls (`Close`, `Hide/Show field changes`, `Restore this version`); background tabs and buttons on `/persons/:id` cannot receive focus while open.
  - Closing `RecordHistoryDrawer` restores keyboard focus to the element that opened it.
- **Tags:** `a11y`, `consistency`

---

### UX-140 [major] Fix `ProfileHeroHeader` overflow `DropdownMenu` left-edge clipping on `390px` mobile and add ARIA / keyboard menu focus management

- **Screen / viewport / theme / lang:** `/persons/:id` -> `ProfileHeroHeader` overflow `⋯` menu (`390x844` & `1440x900`; Light & Dark; EN & RU)
- **Problem:**
  1. On `390x844` mobile, the `ProfileHeroHeader` action bar wraps in the center (`x ≈ 166px` EN / `206px` RU). Because `DropdownMenu` uses `align="end"` (`right-0`) with `min-w-52` and no viewport edge clamping, the open menu shifts **58 px off the left edge** in EN (`x: -58.08`, width `264px`) and **47 px off the left edge** in RU (`x: -47.11`, width `293px`), physically slicing off the leading icons and text (`"Set as My Focus Person"` / `"Сделать моей ключевой персоной"` and `"Change History"` / `"История изменений"`).
  2. `DropdownMenu` (`dropdown-menu.tsx:66-130`) wraps the trigger in `<div onClick>` with no `aria-haspopup="menu"` or `aria-expanded={open}`, uses raw `<button role="menuitem">` (`line 91`), and handles `ArrowDown`/`ArrowUp` only in state (`activeIndex`) without moving DOM focus between menu items or supporting `Home`/`End`/return-focus on `Escape` (WCAG `1.4.10` Reflow, `2.1.1` Keyboard, `4.1.2` Name, Role, Value).
- **Evidence:**
  - `frontend/src/components/profile/ProfileHeroHeader.tsx:504-517`
  - `frontend/src/components/ui/dropdown-menu.tsx:39-130`
  - Screenshot `drawer_mobile_dark_ru_menu.png` + telemetry (`menuBox: { x: -58.078125, width: 264.203125 }` EN, `{ x: -47.109375, width: 292.765625 }` RU; `overflowAttrs: { ariaHaspopup: null, ariaExpanded: null }`).
- **Proposed fix:**
  - Clamp `DropdownMenu` horizontal positioning against viewport edges (`max-w-[calc(100vw-2rem)]`, measuring trigger/menu rects on open or switching to a `Drawer side="bottom"` action sheet on `< 640px` mobile) so menus never clip off the left or right edge.
  - Inject `aria-haspopup="menu"` and `aria-expanded={open}` onto the trigger element (or interactive trigger button), move real DOM `.focus()` to the corresponding `[role="menuitem"]` on open and on `ArrowDown`/`ArrowUp`/`Home`/`End`, and return focus to the trigger on `Escape`.
- **Acceptance criteria:**
  - At `390x844` in EN and RU, opening the `ProfileHeroHeader` overflow menu displays 100% of every icon and label inside the viewport (`menuRect.left >= 8 && menuRect.right <= 382`).
  - The trigger exposes `aria-haspopup="menu"` and `aria-expanded`; `ArrowDown`/`ArrowUp` move real keyboard focus through items, and `Escape` closes the menu and restores trigger focus.
- **Tags:** `a11y`, `layout`, `consistency`

---

### UX-141 [major] Fix `ConflictResolutionModal` dual `primary` button hierarchy and pin actions in sticky `Dialog` footer on mobile

- **Screen / viewport / theme / lang:** `/persons/:id` -> `PersonEditDialog` -> `ConflictResolutionModal` (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:**
  1. When `"Merge field by field"` is clicked (`mergeMode === true`), `ConflictResolutionModal.tsx:370-392` renders **two simultaneous `variant="primary"` buttons** (`"Save Merged Record"` and `"Keep my changes"` / `"Сохранить объединённую запись"` and `"Оставить мои изменения"`), violating [`docs/design-system/components.md:94`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md#L94) (*"one `primary` button per view region"*) and obscuring which blue CTA applies the user's per-field selection.
  2. All action buttons use `size="sm"` (`32px` height instead of standard `size="md"` `44px` modal actions) and sit inside the scrollable `children` body rather than `<Dialog footer={...}>`. On `390x844` mobile (`conflict_mobile_dark_ru_merge.png`, `conflict_mobile_light_en_merge.png`), long comparison cards push the action buttons below the scroll fold / behind `dialog-scroll-fade`, completely hiding `"Save Merged Record"` and `"Keep my changes"` until scrolled.
- **Evidence:**
  - `frontend/src/components/history/ConflictResolutionModal.tsx:187-194, 348-393`
  - `frontend/src/components/ui/dialog.tsx:270-282`
  - Screenshots `conflict_desktop_light_en_merge.png`, `conflict_desktop_dark_ru_merge.png`, `conflict_mobile_dark_ru_merge.png`, `conflict_mobile_light_en_merge.png` + live DOM telemetry (`actionBtns` reporting two primary `rgb(45, 85, 132)` / `rgb(104, 153, 211)` buttons, all `height: 32`).
- **Proposed fix:**
  - Move the resolution action bar into `<Dialog footer={...}>` so action controls remain pinned at the bottom of the dialog (`pb-[max(0.875rem,env(safe-area-inset-bottom,0px))]`, stacked full-width or wrapped cleanly on `< sm`) above the scroll region.
  - Upgrade action buttons to `size="md"` (`min-h-11`, `44px`).
  - Enforce a single `variant="primary"` button at all times: in `mergeMode === true`, keep `"Save Merged Record"` (`applyMergedBtn`) as `variant="primary"` and demote `"Keep my changes"` (`keepMineBtn`) to `variant="secondary"`.
- **Acceptance criteria:**
  - Only one `variant="primary"` button renders in `ConflictResolutionModal` in both initial and merge states.
  - On `390x844` mobile in EN and RU, `"Save Merged Record"` / `"Сохранить объединённую запись"` stays pinned and visible in the sticky footer without scrolling past the body or clipping behind `dialog-scroll-fade`, and all footer buttons measure `>= 44px` tall.
- **Tags:** `hierarchy`, `layout`, `space`, `consistency`

---

### UX-142 [major] Remove premature active radio selection before `"Merge field by field"` and fix WAI-ARIA `radiogroup` keyboard arrows in `ConflictResolutionModal`

- **Screen / viewport / theme / lang:** `/persons/:id` -> `PersonEditDialog` -> `ConflictResolutionModal` (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:**
  1. In the initial `409 Conflict` screen (`mergeMode === false`), `ConflictResolutionModal.tsx:244-340` already assigns `role="radiogroup"` and `role="radio"` with `aria-checked="true"`, blue `border-brand bg-brand/10`, and a `<Check />` icon on every `"Your Unsaved Changes"` cell (`radioCountBeforeMerge: 6`). Clicking cells selects choices even though `"Save Merged Record"` is not yet shown, miscommunicating the difference between overview comparison and per-field merge mode.
  2. When `mergeMode === true`, the custom clickable `<div role="radio">` cards fail WAI-ARIA `radiogroup` semantics and [`docs/design-system/components.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md#L139-L141) (`RadioCardGroup`): `ArrowRight`/`ArrowDown`/`ArrowLeft`/`ArrowUp` do **not** move checked selection between `"Your Unsaved Changes"` and `"Latest Saved Version"` (`afterArrowRight: "false"`, `afterArrowDown: "false"`), every card sets `tabIndex={0}` instead of roving `tabIndex`, and desktop cards measure `38px` tall (`< 40px`) with no `.touch-target-expand`.
- **Evidence:**
  - `frontend/src/components/history/ConflictResolutionModal.tsx:241-343`
  - Screenshot `conflict_desktop_light_en_initial.png` + telemetry (`initialMetrics.radioCountBeforeMerge: 6`, `radioArrowCheck: { beforeArrow: "false", afterArrowRight: "false", afterArrowDown: "false" }`, desktop radio height `38px`).
- **Proposed fix:**
  - While `mergeMode === false`, render the comparison table (`ComparisonTable` pattern) as non-interactive diff rows without `role="radio"`, `tabIndex`, blue active borders, or checkmarks until `"Merge field by field"` is chosen.
  - When `mergeMode === true`, use semantic native `<input type="radio">` cards / `RadioCardGroup` with working `ArrowLeft`/`ArrowRight`/`ArrowUp`/`ArrowDown` selection & focus movement, roving `tabIndex`, and `min-h-11` (`44px`) target height.
- **Acceptance criteria:**
  - Before clicking `"Merge field by field"`, `ConflictResolutionModal` exposes `0` `role="radio"` elements and renders neutral side-by-side comparison styling.
  - In merge mode, pressing `ArrowRight` or `ArrowDown` moves focus and toggles selection from `"Your Unsaved Changes"` (`aria-checked="false"`) to `"Latest Saved Version"` (`aria-checked="true"`), and each radio card measures `>= 44px` tall.
- **Tags:** `a11y`, `states`, `hierarchy`, `consistency`

---

### UX-143 [major] Make `RecordHistoryDrawer` HTTP 500 error `Alert` mutually exclusive with `"No Changes Recorded"` `EmptyState`

- **Screen / viewport / theme / lang:** `/persons/:id` -> `RecordHistoryDrawer` error state (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** On HTTP 500 from `GET /api/v1/history/:type/:id`, `RecordHistoryDrawer.tsx:165-167` sets `entries` to `[]` alongside `loadError`, and `lines 272-286` render `{loadError && <Alert variant="danger">` **above** `} : entries.length === 0 ? ( <EmptyState ... /> )`. Users simultaneously see `"An internal server error occurred"` and `"No Changes Recorded — No revision entries were found for this record yet"` (`drawer_error_ru.png`), creating contradictory feedback with no `Retry` button to reload.
- **Evidence:**
  - `frontend/src/components/history/RecordHistoryDrawer.tsx:165-168, 272-286`
  - Screenshot `drawer_error_ru.png`
- **Proposed fix:** Branch mutually exclusively (`loading ? <Spinner /> : loadError ? <Alert variant="danger">...<Button onClick={loadHistory}>{t('common.retry')}</Button></Alert> : entries.length === 0 ? <EmptyState ... /> : <ol>...</ol>`) so failed fetches never display the empty-history graphic and message.
- **Acceptance criteria:**
  - When `GET /api/v1/history/:type/:id` returns HTTP 500, `RecordHistoryDrawer` renders the localized error `Alert` plus a `Retry` action and does **not** render `EmptyState` (`"No Changes Recorded"` / `"Изменений пока нет"`).
  - Clicking `Retry` re-runs `loadHistory()` and displays entries on recovery.
- **Tags:** `states`, `copy`

---

### UX-144 [major] Fix `AccessDeniedCard` horizontal overflow on `390px` RU mobile and hide admin-only routes in mobile `BottomNav` More sheet

- **Screen / viewport / theme / lang:** `/history` non-admin RBAC state & `BottomNav` `"More"` drawer (`390x844`; Light & Dark; EN & RU)
- **Problem:**
  1. On `390x844` mobile in RU, `<Badge variant="warning" size="sm">Только для администраторов</Badge>` inside `<CardHeader className="flex items-start justify-between gap-3">` (`access-denied-card.tsx:32-46`) refuses to wrap and protrudes outside the card border and viewport (`documentElement.scrollWidth: 401px` vs `innerWidth: 390px`, `badge.right: 401px`), failing WCAG 2.2 SC `1.4.10` (Reflow).
  2. Desktop header `More` (`app-shell.tsx:173-184`) and the tablet/mobile hamburger menu (`app-shell.tsx:329-338`) hide admin-only links from `editor` and `viewer` accounts, whereas `bottom-nav.tsx:48, 128-150` omits any `isAdmin` check and shows `/history`, `/quality`, `/settings`, and `/export/print` in the mobile `"More"` sheet to all authenticated roles, funneling non-admins straight into an `AccessDeniedCard` dead end (violates [`docs/design-system/components.md:222`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md#L222) and [`docs/features/14-change-history-concurrency.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md#L57)).
- **Evidence:**
  - `frontend/src/components/ui/access-denied-card.tsx:32-46`
  - `frontend/src/components/ui/bottom-nav.tsx:48, 128-150` and `frontend/src/App.tsx:415-420` (contrast `app-shell.tsx:173-184, 329-338`)
  - Screenshot `hist_rbac_editor_mobile_ru.png` + telemetry (`vw: 390, sw: 401, horizontalOverflow: true`, `badge: { right: 401, w: 179 }`, `moreSheetLinks` showing `"История изменений"` and `"Настройки архива"` to an `editor`).
- **Proposed fix:**
  - Update `access-denied-card.tsx:32-46` to `flex-wrap items-start justify-between gap-2 sm:gap-3` with `min-w-0` on the heading container so long badges wrap cleanly below or inside the card at `390px`.
  - In `bottom-nav.tsx`, derive `const isAdmin = user?.role === 'admin';` and gate admin-only items (`/history`, `/quality`, `/settings`, and `/export/print` per `App.tsx:415-420`) behind `isAdmin`.
- **Acceptance criteria:**
  - At `390x844` in RU and EN, non-admin `/history` exhibits `0px` horizontal overflow (`scrollWidth <= 390`) with the warning badge fully bounded inside `AccessDeniedCard`.
  - Non-admin users (`editor`, `viewer`) do not see `/history`, `/quality`, `/settings`, or `/export/print` inside the mobile `BottomNav` `"More"` drawer.
- **Tags:** `layout`, `a11y`, `consistency`, `states`

---

### UX-145 [major] Localize `GenDate` values (`ABT 1912`, `19 Mar 1912`, `abt 1914`) in RU history diffs and unify child pedigree delimiter formatting

- **Screen / viewport / theme / lang:** `/history`, `/persons/:id` -> `RecordHistoryDrawer`, `ConflictResolutionModal` (`1440x900` & `390x844`; Light & Dark; RU & EN)
- **Problem:**
  1. In Russian UI (`drawer_mobile_dark_ru.png`, `/history` RU feed), historical dates in diff cells (`birth_date`, `death_date`, `date`) output untranslated English GEDCOM strings (`"ABT 1912"`, `"19 Mar 1912"`, `"abt 1914"` in `HistoryPage.tsx:105`) instead of localized Russian genealogical dates (`"около 1912 г."` / `"ок. 1912 г."`, `"19 мар. 1912 г."`, `"около 1914 г."`). `formatObjectValue` (`history-diff-utils.ts:88-94`) returns `obj.text || obj.gedcom` verbatim without calling `formatGenDate(genDate, locale)`, and `formatDiffValue` does not parse/format date-field strings via `parseGenDate` + `formatGenDate`.
  2. Within `/history` `children` diffs, `FamilyChildDto` objects render pedigree in parentheses `"(Усыновлён(а))"` (`history-diff-utils.ts:102`) while formatted strings render in square brackets `"[Усыновлён(а)]"` (`history-label-formatters.ts:136`) on adjacent entries.
- **Evidence:**
  - `frontend/src/components/history/history-diff-utils.ts:88-105, 175-188`
  - `frontend/src/lib/gendate-parser.ts:228, 379-447`
  - `frontend/src/components/history/history-label-formatters.ts:130-137`
  - `frontend/src/pages/HistoryPage.tsx:105`
  - Screenshot `drawer_mobile_dark_ru.png` + telemetry (`Дата рождения`: `"ABT 1912"` -> `"19 Mar 1912"` in RU; `/history` mixing `[Усыновлён(а)]` and `(Усыновлён(а))`).
- **Proposed fix:**
  - Pass `locale` into `formatDiffValue` from `HistoryPage`, `RecordHistoryDrawer`, and `ConflictResolutionModal`.
  - In `history-diff-utils.ts`, format `GenDate` objects and strings under date fields (`birth_date`, `death_date`, `date`) through `parseGenDate(raw, locale)` + `formatGenDate(genDate, locale)` (preserving unknown phrase fallback if unparseable).
  - Unify pedigree delimiters (e.g., consistent parentheses `(...)` across both `FamilyChildDto` formatting and `localizeEmbeddedDisplayString`).
- **Acceptance criteria:**
  - In RU locale across `/history`, `RecordHistoryDrawer`, and `ConflictResolutionModal`, dates such as `"ABT 1912"`, `"19 Mar 1912"`, and `"abt 1914"` render in localized Russian (`"ок. 1912 г."` / `"около 1912 г."`, `"19 мар. 1912 г."`) with zero English month abbreviations or raw `ABT` tokens.
  - Child relationship labels use identical delimiter formatting across object diffs and formatted strings.
- **Tags:** `copy`, `consistency`, `a11y`

---

### UX-146 [major] Fix Dark-mode native date picker icon contrast (`color-scheme: dark`) and enlarge undersized (`24px`–`32px`) interactive controls

- **Screen / viewport / theme / lang:** `/history` (`HistoryFilterBar`), `RecordHistoryDrawer`, `AccessDeniedCard`, `/dashboard` (`RecentActivityWidget`), `/settings/backups` (`TreeSnapshotsCard`) (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:**
  1. **Dark-mode date input icon contrast (WCAG `1.4.11` Non-text Contrast):** `frontend/src/index.css:3-76` omits `color-scheme: light` on `:root` and `color-scheme: dark` on `.dark`. Native Chromium `<TextInput type="date">` controls (`HistoryFilterBar.tsx:98-119`) render black `#000000` calendar icons on top of `#171B22` dark surfaces, making `From Date` / `To Date` (`Дата с` / `Дата по`) icons nearly invisible in Dark Mode (`hist_desktop_dark_ru_full.png`, `ux_hist_mobile_dark_ru_top.png`).
  2. **Undersized visual targets (`< 40px`) and inconsistent primary CTAs:**
     - `Drawer` close button (`drawer.tsx:87-93`) uses `<IconButton size="sm">` (`32x32` px), unlike `Dialog` (`dialog.tsx:253`) which uses `h-10 w-10 min-h-10 min-w-10` (`40x40`).
     - `Switch` (`switch.tsx:60-64`) renders `h-6 w-11` (`44x24px`) with no `touch-target-expand`.
     - Standalone actions and CTAs — `/history` `Undo` & `Restore Record` (`HistoryPage.tsx:471-491`, `32px`), entity `<Button variant="link" className="h-auto px-0 py-0">` (`HistoryPage.tsx:572`, collapsing link padding/height), `RecordHistoryDrawer` `Restore this version` (`RecordHistoryDrawer.tsx:405-413`, `32px`), `AccessDeniedCard` footer CTAs (`access-denied-card.tsx:57-75`, `32px`), `TreeSnapshotsCard` `Create Checkpoint` & `Roll Back to Checkpoint` (`TreeSnapshotsCard.tsx:270-278, 357-368`, `32px`), and mobile `RecentActivityWidget` `Undo` (`RecentActivityWidget.tsx:132-140`, `32px`) — measure `32px` tall even on `390px` mobile and touch laptops where `@media (pointer: coarse)` does not apply.
- **Evidence:**
  - `frontend/src/index.css:3-76`
  - `frontend/src/components/ui/drawer.tsx:87-93`, `frontend/src/components/ui/switch.tsx:60-64`, `frontend/src/components/ui/access-denied-card.tsx:57-75`
  - `frontend/src/pages/HistoryPage.tsx:471-491, 568-576`, `frontend/src/components/history/RecordHistoryDrawer.tsx:405-413`, `frontend/src/components/settings/TreeSnapshotsCard.tsx:270-278, 357-368`, `frontend/src/components/dashboard/RecentActivityWidget.tsx:132-140`
  - Screenshots `ux_hist_mobile_dark_ru_top.png`, `hist_desktop_dark_ru_full.png` + bounding-box telemetry (`32px` heights across those buttons, `44x24` `Switch`).
- **Proposed fix:**
  - Add `color-scheme: light;` to `:root` and `color-scheme: dark;` to `.dark` in `frontend/src/index.css`.
  - Upgrade `Drawer` close button (`drawer.tsx:88`) to `className="h-10 w-10 min-h-10 min-w-10"` matching `Dialog`.
  - Add `touch-target-expand` on `Switch` (`switch.tsx:61`), restore `min-h-8`/`min-h-10` hit padding on `/history` entity links, and use `size="md"` (`44px`) or `min-h-10 sm:min-h-8` responsive sizing on standalone/primary actions (`Restore Record`, `Restore this version`, `AccessDeniedCard` CTAs, `Create Checkpoint`, `Roll Back to Checkpoint`, and mobile `Undo`).
- **Acceptance criteria:**
  - In Dark Mode, native date filter calendar icons in `HistoryFilterBar` render with light foreground glyphs (`>= 3:1` contrast against `#171B22`).
  - `Drawer` close button measures `>= 40x40px`; `Switch` carries `touch-target-expand`; primary/standalone CTAs and mobile `390px` action buttons measure `>= 40px` tall.
- **Tags:** `a11y`, `consistency`, `layout`

---

### UX-147 [minor] Deduplicate `(GID)` and repeated entity names, add affordance to entity links, sanitize `tx-*` IDs, and add `Before`/`After` labels on `/history`

- **Screen / viewport / theme / lang:** `/history` (`HistoryPage.tsx`, `history-diff-utils.ts`, `RecordHistoryDrawer.tsx`) (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:**
  1. Card headers pass `{ stripGid: false }` to `formatEntityLabel` (`HistoryPage.tsx:438-444`), while expanded entry rows render `${rawFormattedLabel} (${displayEntityType})` alongside a separate monospace `{gid}` badge (`lines 555, 583`). Consequently, `"Alexander Levshits"` repeats 3 times and `(I0001)` appears in both the header and the pill below.
  2. In `history-diff-utils.ts:49-51`, `undone_tx_id` passes raw transaction ID strings (`tx-profile-201`) into `formatTxLabel`, whose fallback runs `toTitleCase(trimmed)` (`history-label-formatters.ts:181`), turning internal IDs into confusing text (`Tx Profile 201`) for family members.
  3. Navigable entity `<Button variant="link">` (`HistoryPage.tsx:568-576`) sets `text-primary`, making clickable links visually identical to non-clickable `<span>` titles (`line 578`).
  4. The 3-column diff grid (`HistoryPage.tsx:594-613`) lacks `Field` / `Before` / `After` labels (`ConflictResolutionModal.tsx:229` includes them), and stacked mobile rows rely on strikethrough + tint alone to distinguish previous vs new values.
- **Evidence:**
  - `frontend/src/pages/HistoryPage.tsx:436-446, 544-616`
  - `frontend/src/components/history/history-diff-utils.ts:49-51` and `frontend/src/components/history/history-label-formatters.ts:181`
  - Screenshots `ux_hist_desktop_undo_toast.png`, `hist_desktop_dark_ru_full.png`
- **Proposed fix:**
  - Pass `{ stripGid: true }` in the `/history` card header (`HistoryPage.tsx:443`) so `(GID)` is omitted from the headline and shown once in the monospace GID badge.
  - In `history-diff-utils.ts:49-51`, only run `formatTxLabel` when `val` matches a known `tx_label` (`TX_LABEL_MAP` or composite label); format opaque IDs with `formatCompactRef` rather than `toTitleCase`.
  - Keep `text-brand hover:underline` plus a trailing `ArrowUpRight` / `ChevronRight` icon on clickable entity links (`HistoryPage.tsx:568-576`).
  - Add a `hidden sm:grid sm:grid-cols-3` localized header (`Field` / `Before` / `After`) to `/history` diff lists and subtle `Before:`/`After:` labels on mobile (`sm:hidden`) in both `HistoryPage` and `RecordHistoryDrawer`.
- **Acceptance criteria:**
  - Single-entity cards on `/history` show each person's `GID` at most once (in the monospace pill) and never display title-cased internal IDs like `Tx Profile 201`.
  - Clickable entity links clearly render in brand link color with underline/icon affordance distinct from non-link deleted records.
  - Diff grids expose `Before` / `After` text context visually and for screen readers across desktop and mobile.
- **Tags:** `hierarchy`, `consistency`, `a11y`, `copy`

---

### UX-148 [minor] Expand cramped `w-56` checkpoint label input and remove exposed `(snap-1)` IDs in `TreeSnapshotsCard`

- **Screen / viewport / theme / lang:** `/settings/backups` -> `TreeSnapshotsCard` (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:**
  1. `TreeSnapshotsCard.tsx:261-269` wraps the checkpoint label `<TextInput>` in a fixed `w-56` (`224px`) container, truncating the example placeholder (`"Checkpoint label (e.g., Befo..."`) even on `1440px` desktop while leaving dead horizontal space on `390px` mobile.
  2. `TreeSnapshotsCard.tsx:311-313` prints raw internal snapshot IDs `({snap.id})` (`(snap-1)`, `(snap-2)`) beside every checkpoint title.
- **Evidence:**
  - `frontend/src/components/settings/TreeSnapshotsCard.tsx:257-280, 307-314`
  - Screenshots `backups_desktop_light_en.png`, `backups_mobile_dark_ru.png`
- **Proposed fix:**
  - Change the form container (`lines 257-269`) to allow the label `<TextInput>` to span `w-full sm:w-72 md:w-80` so placeholders remain readable on desktop and full-width on `390px` mobile.
  - Remove the `({snap.id})` span (`lines 311-313`) whenever `snap.label` is populated.
- **Acceptance criteria:**
  - At `390x844`, the checkpoint input stretches across the card width above `"Create Checkpoint"`; on `1440x900`, the placeholder is not prematurely clipped at `224px`.
  - Checkpoint rows display human labels, reason badges, timestamps, sizes, and counts without `(snap-1)` / UUID suffixes.
- **Tags:** `layout`, `space`, `copy`

---

### UX-149 [minor] Replace `(s)` count strings with ICU plurals, align `/history` EN breadcrumb/title, and clean up wrapped `"· "` bullets on mobile

- **Screen / viewport / theme / lang:** `/history`, `/persons/:id` -> `RecordHistoryDrawer`, `/dashboard` -> `RecentActivityWidget` (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:**
  1. `frontend/src/i18n/en.json:1897, 1950` uses `(s)` strings (`"{count} field(s) modified"`, `"{count} transaction(s)"`) instead of ICU plurals used elsewhere (`en.json:1891`).
  2. In EN, the breadcrumb and navigation label say `"Recent Changes"` (`en.json:43`, `App.tsx:453`) whereas the `/history` `<h1>` heading says `"Change History & Audit Log"` (`en.json:1948`); Russian aligns cleanly on `"История изменений"`.
  3. At `390x844` mobile, inline `"· {formatTxLabel(...)}"` in `RecordHistoryDrawer.tsx:327-329` and `<span>·</span>` in `RecentActivityWidget.tsx:96` wrap awkwardly onto the start of line 2 (`"· Edited Profile Details"`) or dangle at the end of line 1 (`"Vera Levshits ·"`).
- **Evidence:**
  - `frontend/src/i18n/en.json:43, 1897, 1948, 1950` and `frontend/src/App.tsx:453`
  - `frontend/src/components/history/RecordHistoryDrawer.tsx:323-331`
  - `frontend/src/components/dashboard/RecentActivityWidget.tsx:92-102`
  - Screenshots `drawer_mobile_light_en.png`, `drawer_mobile_dark_ru.png`, `dash_mobile_dark_ru_undoerr.png`
- **Proposed fix:**
  - Replace `history.drawer.fieldsChangedCount` and `history.page.totalTransactions` in `en.json` with ICU plurals (`"{count, plural, one {# field modified} other {# fields modified}}"` and `"{count, plural, one {# transaction} other {# transactions}}"`).
  - Unify EN nav/breadcrumb (`nav.history`) and page heading (`history.page.title`) terminology (e.g., `"Change History"`).
  - Remove standalone wrapped `"·"` separators on `< sm` mobile (render the timestamp and action description on stacked blocks or hide `"·"` on narrow rows with `hidden sm:inline`).
- **Acceptance criteria:**
  - Zero `(s)` strings render on `/history` or `RecordHistoryDrawer`; EN breadcrumb matches the page title/nav term (`Change History`).
  - No wrapped line in `RecordHistoryDrawer` or `RecentActivityWidget` begins or ends with an orphaned `"·"` bullet at `390px` width.
- **Tags:** `copy`, `consistency`, `layout`

---

### UX-150 [suggestion] Add collapsible mobile filter drawer/toggle and active `FilterChip` pills on `/history`

- **Screen / viewport / theme / lang:** `/history` (`HistoryFilterBar.tsx`) (`390x844`; Light & Dark; EN & RU)
- **Problem:** On `390x844` mobile (`ux_hist_mobile_dark_ru_top.png`), the always-expanded 4-field `HistoryFilterBar` + `"Only deleted records"` switch consumes ~`430px`+ below the header, pushing **100% of transaction cards below the fold**. Collapsing filters behind a `"Filters"` toggle (showing active `FilterChip` pills when filtering, matching [`docs/design-system/components.md:272`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md#L272)) lets older relatives read recent activity right away on mobile.
- **Evidence:**
  - `frontend/src/components/history/HistoryFilterBar.tsx:60-149`
  - Screenshot `ux_hist_mobile_dark_ru_top.png`
- **Proposed fix:** Keep the filter grid always visible on `sm:` / `md:` desktop and collapse below `sm:` into a toggle button with active filter badge + removable `FilterChip` summary pills.
- **Acceptance criteria:**
  - At `390x844` with default filters, the first `/history` transaction card is visible above the fold without scrolling past 4 filter controls.
- **Tags:** `space`, `hierarchy`

---

## Positive Notes

- **WCAG AA contrast on diff pills across Light & Dark themes:** Canvas-composited measurements on `/history` and `RecordHistoryDrawer` confirmed all `bg-danger/10` (`6.09:1`–`6.61:1`) and `bg-success/10` (`14.55:1`–`15.04:1`) diff cells pass WCAG 2.2 AA (`>= 4.5:1`) cleanly in both themes.
- **Lossless bundle & payload humanization:** Composite transactions (`person.add_parent`), `person_merge_bundle`, `event_merge_bundle`, `DeletedPersonSnapshotPayload`, `[primary avatar]`, `"Note — ..."`, `"<event_type> — ..."`, and ICU plural `record_counts` (`1 персона, 1 семья, 2 события...`) all render as readable human summaries with **zero** raw UUIDs or JSON object dumps (`[object Object]`).
- **Toast clearance & error surfacing:** On `/history` and `/dashboard` at `390x844`, the shared `Toast` region sits cleanly above `BottomNav` with full multi-line wrapped copy, and `ConflictResolutionModal`'s `conflict-error-alert` (`409` / retry failure) renders clearly inline without closing the modal or losing user merge choices.
- **Empty & safe-revert semantics:** Empty states on `/history` (`200 OK []`) and `RecordHistoryDrawer` (`200 OK []`) display clean centered `EmptyState` cards, and `create` / `can_revert: false` entries cleanly omit the `"Restore this version"` action.

