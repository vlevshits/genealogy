# UX Review — Duplicate Events Detection & Merge (`/quality`) — 2026-11-14

## Verdict: APPROVED

*(Initial audit on 2026-11-14 returned `NEEDS FIXES` with tasks `UX-127`–`UX-137`. Re-audit on 2026-11-14 verified all `6` major (`UX-127`–`UX-132`) and `4` minor (`UX-133`–`UX-136`) tasks are **fixed** across `1440x900`, `390x844`, and `390x667` in Light/Dark themes and EN/RU locales, with `UX-137` (`suggestion`) **deferred to technical debt**.)*

**Screens reviewed:**
- `http://localhost:5173/quality` (`All`, `Critical`, `Warnings`, `Duplicates` tabs; authenticated Admin mock + offline fallback; populated state & empty state)
- `DuplicateEventCustomizeModal` (opened via `Customize` on safe and conflict Duplicate Event cards on `/quality?tab=duplicates`)
- Viewports / Themes / Locales: Desktop `1440x900` (Light EN, Dark RU), Mobile `390x844` (Light EN, Dark RU), Short Mobile `390x667` (Dark RU modal scroll check)

**Inspection captures (`playwrightmcp` screenshots & DOM/keyboard/contrast telemetry):**
- Initial audit: `desktop_1440_light_en_{overview,duplicates,modal,undo_toast,empty}.png`, `desktop_1440_dark_ru_{duplicates,duplicates_auth,duplicates_full,modal}.png`, `mobile_390_dark_ru_{duplicates,modal,undo_toast}.png`, `mobile_390x667_dark_ru_modal.png`, `mobile_390_light_en_{overview,duplicates_top,conflict_card,modal}.png`, `page-2026-11-14T14-47-32-873Z.yml`
- Re-audit verification: `reaudit_1440_light_en_{overview,duplicates_highlight,modal,undo_toast_live,empty}.png`, `reaudit_1440_dark_ru_{duplicates,modal}.png`, `reaudit_390_dark_ru_{top,conflict,undo_toast,modal}.png`, `reaudit_390x667_dark_ru_modal.png`, `reaudit_390_light_en_{overview,conflict}.png`

---

## Re-Audit Task Resolution (`UX-127` – `UX-137`)

| ID | Severity | Title | Verification Status | Evidence |
| :--- | :--- | :--- | :--- | :--- |
| `UX-127` | `major` | Initial focus placement & `Tab`/`Shift+Tab` trap in `Dialog` | **fixed** | `dialog.tsx:16-66, 113-171, 227`; live test (`activeInsideDialog: true`, `inertCount: 8`, `Shift+Tab`/`Tab` loop, `Escape` restores trigger focus) |
| `UX-128` | `major` | Field-level diff markers & primary `Customize` on conflict cards | **fixed** | `DuplicateEventCard.tsx:98-128, 180-208, 316-397`; `Conflict` / `Расхождение` badges on all differing fields, `0` contrast failures (`6.01:1` Light amber), primary `Customize` CTA |
| `UX-129` | `major` | Localized rationale summary & offline fallback in RU | **fixed** | `DuplicateEventCard.tsx:61-96`, `DuplicateEventsSubView.tsx:32-169`, `en.json`/`ru.json:1946-1954`; `badRuTokens: []`, clean RU fallback cards |
| `UX-130` | `major` | Multi-line `Undo` Toast without `truncate` clipping | **fixed** | `toast.tsx:77-80, 154`; full wrapped text at `1440x900` EN (`416px` wide) & `390x844` RU (`12px` clearance above `BottomNav`) |
| `UX-131` | `major` | Single-line `SegmentedControl` wrapping cleanly at `390px` | **fixed** | `DataQualityPage.tsx:422-460`, `segmented-control.tsx:50-67, 101, 132`; `clips: false`, `segOverflow: 0`, `docOverflow: 0` in EN & RU |
| `UX-132` | `major` | `Review Duplicates` CTA on `DUPLICATE_EVENT` warnings & event-ID matching | **fixed** | `DataQualityPage.tsx:225-235, 494-505, 550-564`; switches tab, focuses card, applies `border-brand ring-2` |
| `UX-133` | `minor` | Touch target spacing (`gap-2.5` / `min-h-8`) & `>= 40px` primary/nav controls | **fixed** | `dialog.tsx:253` (`40x40`), `segmented-control.tsx:101` (`40px`), primary/modal buttons (`44px`), chips `10px` vertical gap |
| `UX-134` | `minor` | Modal bottom scroll fade (`dialog-scroll-fade`) & `"Combine sources"` chip | **fixed** | `dialog.tsx:102-111, 270-276`, `DuplicateEventCustomizeModal.tsx:332-364`; scroll cue clears at bottom, combined citations inserted in one click |
| `UX-135` | `minor` | `Looks Correct` `<CheckCircle2 />`, `390px` footer balance, single-border empty states & `Duplicate Events: 0` neutral badge | **fixed** | `DuplicateEventCard.tsx:166-208, 411-425`, `DataQualityPage.tsx:407, 596-600`; verified in EN & RU at `1440px` and `390px` |
| `UX-136` | `minor` | Localize `settings.tabs.appearance` in mobile `AppShell` drawer | **fixed** | `en.json` & `ru.json:366` (`"Appearance"` / `"Оформление"`); verified live in `390x844` drawer |
| `UX-137` | `suggestion` | Full-screen / bottom-sheet `< 768px` layout for `Dialog` | **deferred to technical debt** | Partial full-viewport width/height landed in `dialog.tsx:208, 216, 231` (verified `390x844` & `390x667`); remaining safe-area bottom inset / bottom-sheet polish tracked in `docs/technical-debt.md` |

---

## Status of Previous Tasks

Baseline reference: [`docs/ux-reviews/2026-11-07-milestone-3-2-and-full-ux-review.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/2026-11-07-milestone-3-2-and-full-ux-review.md) (highest existing task ID: `UX-126`).

| ID | Title | Status |
| :--- | :--- | :--- |
| `UX-119` | Define or replace missing Tailwind v4 surface and border tokens (`bg-surface-subtle`, `border-subtle`, `bg-border-strong`) across Story Intake, MCP, and Quality views | **fixed** (`--color-surface-subtle`, `--color-subtle`, `--color-border-subtle`, and `--color-border-strong` are defined in `frontend/src/index.css:86-91`) |
| `UX-120` | Add AI Story Intake (`/stories/intake`) to mobile `BottomNav` "More" sheet, Dashboard quick actions, and unauthenticated route guard | *not re-audited (outside `/quality` scope)* |
| `UX-121` | Add `touch-target-expand` to `Button size="xs"` and localize `SideNav` default `aria-label` (completing `UX-111` & `UX-112`) | **still open (partial)** (`touch-target-expand` added to `Button size="xs"` in `button.tsx:19` and modal/quality targets fixed in `UX-133`; `SideNav` outside this audit scope) |
| `UX-122` | Fix 3-step progression badge state and localize default fallback speaker options on `AiStoryIntakePage` | *not re-audited (outside `/quality` scope)* |
| `UX-123` | Improve `AudioRecorderWidget` accessibility (`aria-live`) and primary button sizing for older relatives | *not re-audited (outside `/quality` scope)* |
| `UX-124` | Fix `McpTokensPanel` endpoint URL mobile overflow and add confirmation before revoking Personal Access Tokens | *not re-audited (outside `/quality` scope)* |
| `UX-125` | Complete remaining fixes for `UX-115`, `UX-116`, and `UX-117` (raw codes in `/history` & `/quality`, offline fallbacks, Active Sessions empty state, raw `<button>` in Workbench) | **fixed on `/quality`** (raw warning codes, RU offline fallback strings in `DuplicateEventsSubView.tsx`, and raw `settings.tabs.appearance` in `AppShell` drawer all resolved; other routes `/history`, `/settings/security`, `/dev/ui` not re-audited) |
| `UX-126` | Align RU glossary terms (`Песочные часы`) and destructive merge button styling in `PersonMergePage` | *not re-audited (outside `/quality` scope)* |

---

## Tasks for frontend-developer

### UX-127 [major] Implement initial focus placement and `Tab`/`Shift+Tab` focus trap in `Dialog` primitive (affects `DuplicateEventCustomizeModal`) — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `dialog.tsx:16-66, 113-171, 227` moves focus inside `[role="dialog"]` on open, marks 8 outer sibling subtrees `inert` / `aria-hidden="true"`, traps `Tab`/`Shift+Tab` across `Close` <-> `Confirm Merge`, and restores trigger focus on `Escape`.
- **Screen / viewport / theme / lang:** `/quality?tab=duplicates` -> `DuplicateEventCustomizeModal` (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** Opening `DuplicateEventCustomizeModal` leaves keyboard focus on the trigger button (`Customize`) behind the dimmed backdrop instead of moving focus into the dialog. Pressing `Tab` once moves focus to the background `Merge Events` button (`merge-duplicate-event-btn-dup-evt-marr-1`) *behind* the active dialog — so pressing `Enter` or `Space` silently triggers a blind merge behind the modal. Tabbing past `Confirm Merge` also escapes out of the dialog into `document.body` / background controls instead of wrapping inside `[role="dialog"][aria-modal="true"]`. Violates WCAG 2.2 SC `2.4.3` (Focus Order) and `docs/design-system/components.md` §3.3 (`Dialog`: focus trap, returns focus to trigger).
- **Evidence:**
  - `frontend/src/components/ui/dialog.tsx:38-54, 72-83` (handles `Escape` and restoring `previousActiveRef`, but lacks `tabIndex={-1}`, initial focus placement into the dialog, and `Tab`/`Shift+Tab` trapping).
  - Live keyboard test on `http://localhost:5173/quality` (`1440x900` Light EN): immediately after activating `button[data-testid="customize-duplicate-event-btn-dup-evt-marr-1"]`, `activeInsideDialog` was `false`; 1st `Tab` landed on background `button[data-testid="merge-duplicate-event-btn-dup-evt-marr-1"]` (`insideDialog: false`); pressing `Tab` from `confirm-customize-merge-event-btn` moved focus to `BODY` (`insideDialog: false, isBody: true`).
- **Proposed fix:** In `Dialog`, move focus to the first focusable descendant inside `dialogRef` (or `dialogRef` with `tabIndex={-1}` as fallback) when `open` becomes `true`, intercept `Tab`/`Shift+Tab` to cycle within focusable elements inside `dialogRef`, and mark siblings outside the modal portal `inert` / `aria-hidden="true"` while open (keeping existing `Escape` close and `previousActiveRef.current?.focus()` restore).
- **Acceptance criteria:**
  - Opening `DuplicateEventCustomizeModal` immediately places `document.activeElement` inside `[role="dialog"]`.
  - Pressing `Tab` on `Confirm Merge` wraps focus back to the dialog `Close` button; pressing `Shift+Tab` on the `Close` button wraps to `Confirm Merge`; background page controls cannot receive focus while open.
  - Pressing `Escape` continues to close the dialog and restore focus to the `Customize` button that opened it.
- **Tags:** `a11y`, `consistency`

---

### UX-128 [major] Surface explicit field-level diff on conflict cards and elevate `Customize` over blind `Merge Events` when `has_conflict` is true — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `DuplicateEventCard.tsx:98-128, 180-208, 316-397` highlights conflict rows with amber border/tint + `Conflict` / `Расхождение` tag (`6.01:1` Light EN, `0` Dark RU contrast failures) and promotes `Customize` to `variant="primary"`.
- **Screen / viewport / theme / lang:** `/quality?tab=duplicates` (`Review Conflict` / `Требует проверки` cards; `1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** Conflicting duplicate event cards (`group.has_conflict === true`, e.g., `dup-evt-marr-1` between `E2001` and `E2002`) give users no clear diff and nudge them toward an accidental overwrite:
  1. `DuplicateEventCard` never reads `group.conflict_fields` to highlight which values differ (`1974-06-15` vs `1974-07-02`, `Москва, ЗАГС № 4` vs `Казань, Петропавловский собор`, `Гражданская регистрация` vs `Церковное венчание`, `Свидетельство о браке...` vs `Приходская книга 1974`). Older relatives must read both cards line by line to spot discrepancies (`docs/design-system/components.md:274-275` requires differing values highlighted and indicated with text, not color alone).
  2. Both safe (`Ready to Merge`) and conflicting (`Review Conflict`) cards render `Customize` as `variant="secondary"` and quick `Merge Events` as the prominent `variant="primary"` blue CTA. On a conflict card, clicking primary `Merge Events` silently discards differing field values in favor of `group.proposed` without asking the relative to pick between conflicting dates, places, descriptions, or sources.
- **Evidence:**
  - `frontend/src/components/quality/DuplicateEventCard.tsx:169-265, 289-310`
  - `docs/design-system/components.md:94, 274-275`
  - Captures: `desktop_1440_light_en_duplicates.png`, `desktop_1440_dark_ru_duplicates_full.png`, `mobile_390_light_en_conflict_card.png`
- **Proposed fix:**
  - Use `group.conflict_fields` in `DuplicateEventCard` to mark differing rows (`date`, `place`, `description`, `source`) with a subtle warning surface/border plus an explicit text/icon tag (e.g., `"Conflict"` / `"Расхождение"`) so discrepancies are never conveyed by color alone.
  - When `group.has_conflict` is `true`, make `Customize` (or `"Resolve & Merge"` / `"Настроить"`) the `variant="primary"` button and demote blind quick-merge to `variant="secondary"` (or route conflict merge confirmation through `DuplicateEventCustomizeModal`).
- **Acceptance criteria:**
  - Every field listed in `group.conflict_fields` is visually highlighted with a non-color text/icon marker in both Light and Dark themes.
  - Safe cards retain `Merge Events` as `variant="primary"`; conflict cards elevate `Customize` to `variant="primary"` and do not highlight blind merge as the default path.
- **Tags:** `hierarchy`, `a11y`, `states`, `consistency`

---

### UX-129 [major] Localize duplicate-event `reason` summary, conflict fields, and fallback fixture text in RU (eliminate raw English strings & GEDCOM tags) — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `DuplicateEventCard.tsx:61-96` and `DuplicateEventsSubView.tsx:32-169` render localized RU rationale strings and clean RU fallback records (`badRuTokens: []`, zero slash names/GEDCOM months).
- **Screen / viewport / theme / lang:** `/quality?tab=duplicates` (RU locale; `1440x900` & `390x844`; Light & Dark)
- **Problem:** Even when the UI language is switched to Russian, `DuplicateEventCard` renders `group.reason` verbatim from the backend in English (`"2 duplicate birth events for Ivan Petrovich Smirnov"`, `"2 duplicate marriage events for Сергей Николаевич Орлов & Мария Ивановна Орлова (Кузнецова) (date/place conflict)"`), and the Rust generator concatenates English templates + raw `canon_type` + raw `conflict_fields`. Additionally, when `buildDefaultDuplicateEventGroups()` renders as a fallback, it hardcodes bilingual slash names (`"Иван Петров / Ivan Petrov"`), English GEDCOM dates (`"12 MAY 1938"`, `"14 JUN 1962"`), and English `reason` prose directly inside RU screens.
- **Evidence:**
  - `frontend/src/components/quality/DuplicateEventCard.tsx:275-277`
  - `backend/src/services/quality/events_duplicates.rs:91-102`
  - `frontend/src/components/quality/DuplicateEventsSubView.tsx:40, 51, 84, 93, 99, 109, 142`
  - Captures: `desktop_1440_dark_ru_duplicates_full.png` & `desktop_1440_dark_ru_duplicates.png`
- **Proposed fix:**
  - Build the rationale line on the frontend via `i18n` using structured group fields (`group.events.length`, localized event type, person/couple label, and translated `group.conflict_fields` labels), using `group.reason` only if custom/unmapped.
  - Localize `buildDefaultDuplicateEventGroups()` strings and replace hardcoded bilingual slash names / English GEDCOM month abbreviations with locale-appropriate values.
- **Acceptance criteria:**
  - Under `RU`, zero English words (`duplicate`, `events for`, `date/place conflict`, `MAY`, `JUN`) or raw GEDCOM type codes (`BIRT`/`MARR`/`RESI`) appear on API-backed or fallback cards.
  - Under `EN`, group rationale and conflict field lists render naturally in English.
- **Tags:** `copy`, `consistency`

---

### UX-130 [major] Remove `truncate` clipping on `Undo` Toast title and description across Desktop (`1440px`) and Mobile (`390px`) — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `toast.tsx:77-80, 154` uses `break-words leading-snug` inside `sm:max-w-md`; full copy verified on `1440x900` Light EN (`416px` width, `overflowsX: false`) and `390x844` Dark RU (`12px` clearance above `BottomNav`).
- **Screen / viewport / theme / lang:** `/quality?tab=duplicates` (post-merge / post-batch-merge `Undo` Toast; `1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** `Toast` caps the viewport wrapper at `max-w-sm` (`384px`) and applies single-line CSS `truncate` to both `item.title` and `item.description` inside a flex row alongside the `6s` countdown circle, `Undo` button, and `Close` button. As a result, critical post-mutation feedback is cut off with `...` even on `1440px` desktop (`"Merged 2 duplicate ev..."` / `"2 duplicate events merged..."` in EN; `"Объединено: 2 груп..."` / `"Успешно объединен..."` on `390px` mobile RU), making it unclear what was merged during the 6-second undo window.
- **Evidence:**
  - `frontend/src/components/ui/toast.tsx:76-81, 151`
  - `frontend/src/components/quality/DuplicateEventsSubView.tsx:286-300`
  - Captures: `desktop_1440_light_en_undo_toast.png` & `mobile_390_dark_ru_undo_toast.png` (`toastRect.width: 352`)
- **Proposed fix:** Replace single-line `truncate` on toast `title` and `description` with multi-line wrapping (`break-words`, up to 2–3 lines if needed), widen desktop toast max-width (`sm:max-w-md`), and top-align the `6s` countdown / `Undo` / `Close` controls so both lines read in full while maintaining clean `BottomNav` clearance on mobile.
- **Acceptance criteria:**
  - Batch-merge and single-merge toast titles and descriptions display without `...` truncation at `1440x900` and `390x844` in both EN and RU.
  - Mobile toast continues to clear `BottomNav` cleanly (`>= 8px` gap) with zero horizontal viewport overflow.
- **Tags:** `states`, `layout`, `copy`

---

### UX-131 [major] Fix `SegmentedControl` tab clipping and 2-line label wrapping at `390px` mobile width — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `segmented-control.tsx:50-67, 101, 132` and `DataQualityPage.tsx:428` wrap single-line `40px` pill buttons into two clean rows at `390x844` (`clips: false`, `segOverflow: 0`, `docOverflow: 0` in EN & RU).
- **Screen / viewport / theme / lang:** `/quality` filter bar (`All`, `Critical`, `Warnings`, `Duplicates`, `Places`; `390x844`; Light & Dark; EN & RU)
- **Problem:** On `390px` viewports, `SegmentedControl` items wrap their labels and counts awkwardly onto 2 lines (`"All \n (2)"`, `"Critical \n (0)"`, `"Warnings \n (2)"`) and still overflow the `overflow-x-auto` container cleanly without cues: in EN, `"Places (0)"` is sliced mid-letter to `"P"` on the right edge; in RU, `"Дубликаты (4)"` extends past the right edge (`right: 406 > 390`) and `"Места (0)"` sits completely offscreen (`right: 469.4`), with no scroll-into-view when selecting tabs or visual scroll affordance.
- **Evidence:**
  - `frontend/src/pages/DataQualityPage.tsx:429-466`
  - `frontend/src/components/ui/segmented-control.tsx:85, 116`
  - Captures: `mobile_390_light_en_overview.png`, `mobile_390_light_en_duplicates_top.png`, `mobile_390_dark_ru_duplicates.png`
- **Proposed fix:** Add `whitespace-nowrap` to `SegmentedControl` tab buttons so each label + `(N)` stays on one line, scroll the active tab into view on selection (`scrollIntoView({ inline: 'nearest', block: 'nearest' })`), and either add a subtle horizontal scroll fade mask on scrollable overflow or let filter pills wrap cleanly onto 2 aligned rows on `< sm`.
- **Acceptance criteria:**
  - Tab labels and `(N)` counters never split onto two lines per button at `390px`.
  - Selecting `Duplicates` / `Дубликаты` or `Places` / `Места` on `390px` ensures the active tab is fully visible and all 5 filter tabs remain clearly discoverable in EN and RU.
- **Tags:** `layout`, `space`, `a11y`

---

### UX-132 [major] Replace misleading `"Merge Events"` button on `All`/`Warnings` tab `DUPLICATE_EVENT` warning cards and harden group matching — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `DataQualityPage.tsx:225-235, 494-505, 550-564` renders secondary `"Review Duplicates"` / `"Перейти к дубликатам"` with `<ArrowRight />`, matches event IDs (`evt-birth-1a`), switches to `Duplicates`, focuses the card, and highlights it with `border-brand ring-2`.
- **Screen / viewport / theme / lang:** `/quality` (`All` & `Warnings` tabs; `1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** When a `DUPLICATE_EVENT` warning card renders in the `All` or `Warnings` tab, its action button uses the exact same primary blue styling and label `"Merge Events"` (`t('quality.duplicateEvents.mergeBtn')`) as `DuplicateEventCard`, yet clicking it only switches `activeTab` to `'duplicates'` without performing any merge or scrolling to the target card. Additionally, `matchingDupEventGroup` only checks `g.id === (w.entity_id || w.person?.id || w.person_id)`, which misses warnings whose `entity_id` holds a person ID or individual event ID.
- **Evidence:**
  - `frontend/src/pages/DataQualityPage.tsx:500-507, 552-562`
  - Capture: `desktop_1440_light_en_overview.png` (button labeled `"Merge Events"` on warning card directly above the empty/other sections)
- **Proposed fix:** Relabel the warning card navigation action honestly (e.g. `"Review Duplicates"` / `"Перейти к дубликатам"` with `variant="secondary"` and `<ArrowRight />`), expand `matchingDupEventGroup` matching to `g.id === w.entity_id || g.person?.id === (w.person_id || w.entity_id) || g.events.some((e) => e.id === w.entity_id)`, and focus/scroll to that group when switching tabs.
- **Acceptance criteria:**
  - Action buttons on warning cards never claim `"Merge Events"` if they only switch tabs.
  - Clicking the warning CTA switches to the `Duplicates` tab and scrolls/highlights the matching `DuplicateEventCard`.
- **Tags:** `copy`, `hierarchy`, `consistency`

---

### UX-133 [minor] Increase touch target spacing & control sizes in `DuplicateEventCustomizeModal` and `/quality` controls — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — Modal `Close` (`40x40px`), `Cancel`/`Confirm Merge` (`44px`), `Auto-Merge All` (`44px`), primary card buttons (`44px`), and `SegmentedControl` tabs (`40px`) all measure `>= 40px`; modal quick-fill chips increased to `min-h-8` (`32px`) with `gap-2.5` (`10px` vertical/horizontal spacing between rows).
- **Screen / viewport / theme / lang:** `/quality?tab=duplicates` & `DuplicateEventCustomizeModal` (`1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** Several interactive controls fall under `40px` height on non-coarse pointers (hybrid touchscreen laptops / desktop accessibility users) because `.touch-target-expand` in `index.css:176-189` applies solely inside `@media (pointer: coarse)` — and inside `DuplicateEventCustomizeModal`, quick-fill chips (`Button size="xs" className="... h-auto py-1"`, measured `24px–25.7px` high) are separated by only `gap-1.5` (`6px`), meaning `44px` expanded coarse-pointer hitboxes vertically collide across wrapped rows. Measured heights `< 40px` on `1440x900`: `24` controls including modal quick-fill chips (`24px–25.7px`), person/partner link buttons (`24px`), card/modal footer buttons & `Auto-Merge All` (`32px`), `Dialog` close `IconButton size="sm"` (`32x32px`), and `SegmentedControl` tabs (`36px`). (`docs/design-system/components.md:380` DoD specifies `>= 44x44px` touch targets, `48x48px` on primary actions.)
- **Evidence:**
  - `frontend/src/components/quality/DuplicateEventCustomizeModal.tsx:235-243, 267-275, 300-308, 333-341`
  - `frontend/src/components/quality/DuplicateEventCard.tsx:103, 119, 278-310`
  - `frontend/src/components/ui/segmented-control.tsx:85`
  - `frontend/src/components/ui/dialog.tsx:99`
  - `frontend/src/components/ui/button.tsx:19` & `frontend/src/index.css:176-189`
- **Proposed fix:** Increase quick-fill chip spacing (`gap-2.5`) and minimum visual height (`min-h-8`) so `44px` touch targets never overlap; upgrade primary CTAs (`Auto-Merge All`, `Confirm Merge`, card primary CTA) to `size="md"` (`44px`); use `min-h-10` + `touch-target-expand` on `SegmentedControl` tabs and `Dialog` close button; ensure person/partner links in cards have adequate vertical tap clearance.
- **Acceptance criteria:**
  - Wrapped quick-fill chip rows in `DuplicateEventCustomizeModal` have `>= 8px` gap and do not produce overlapping hit boxes.
  - Primary actions (`Auto-Merge All`, `Confirm Merge`, primary card button), `SegmentedControl` tabs, and modal `Close` button measure `>= 40px` (or `44px` touch target without collisions).
- **Tags:** `a11y`, `layout`, `consistency`

---

### UX-134 [minor] Improve `DuplicateEventCustomizeModal` scroll discoverability and offer a `"Combine sources"` quick-fill option — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `dialog.tsx:102-111, 270-276` renders `[data-testid="dialog-scroll-fade"]` when content overflows and clears it cleanly at scroll bottom; `DuplicateEventCustomizeModal.tsx:332-364` provides `[data-testid="combine-sources-chip"]` which joins distinct sources (`Свидетельство о браке I-МЮ № 123456 | Приходская книга 1974`).
- **Screen / viewport / theme / lang:** `DuplicateEventCustomizeModal` (`1440x900`, `390x844`, `390x667`; Light & Dark; EN & RU)
- **Problem:** Two usability snags inside `DuplicateEventCustomizeModal`:
  1. On both `1440x900` (`bodyScrollH: 742 > bodyClientH: 609`) and `390x844` (`bodyScrollH: 858 > bodyClientH: 672`), the `"Source"` / `"Источник"` field label sits half-clipped at the bottom edge of `DialogBody` directly above the sticky footer with no bottom scroll fade/cue, obscuring the final field below the fold.
  2. When duplicate events hold distinct archival citations (e.g., `Свидетельство о браке I-МЮ № 123456` and `Приходская книга 1974`), quick-fill buttons only allow replacing `sourceText` with one or the other rather than offering a one-click `"Combine"` option, even though the card reassurance note stresses keeping all source citations.
- **Evidence:**
  - `frontend/src/components/quality/DuplicateEventCustomizeModal.tsx:333-341`
  - Captures: `desktop_1440_light_en_modal.png`, `desktop_1440_dark_ru_modal.png`, `mobile_390_light_en_modal.png`
- **Proposed fix:** Add a subtle bottom scroll fade/shadow indicator when `DialogBody` overflows vertically (`scrollHeight > clientHeight` and not yet scrolled to bottom), slightly tighten vertical padding between Surviving Event cards and fields on desktop, and add a `"Combine sources"` / `"Объединить источники"` quick-fill chip when `>= 2` duplicate events contain distinct non-empty `source_text` strings.
- **Acceptance criteria:**
  - Overflowing modal content displays a clear visual scroll indicator rather than leaving a mid-sliced `"Source"` label as the only clue.
  - When multiple distinct `source_text` values exist, a single-click `"Combine"` chip populates `sourceText` with both citations joined cleanly.
- **Tags:** `space`, `states`, `a11y`

---

### UX-135 [minor] Standardize `Looks Correct` icon/variant, mobile card footer balance, empty-state framing, and zero-count summary badge — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — `DuplicateEventCard.tsx:166-208, 411-425` adds `<CheckCircle2 />` + `variant="secondary"` on `Looks Correct` and uses `flex-1` row 1 + `w-full` primary CTA row 2 on mobile; `DataQualityPage.tsx:407, 596-600` sets `Duplicate Events: 0` to `neutral` and removes the redundant `<Card>` wrapper on `Candidate Duplicate Pairs`. *(Note: `Duplicate Pairs: 0` badge at `DataQualityPage.tsx:403` uses `variant="brand"` outside Duplicate Events scope.)*
- **Screen / viewport / theme / lang:** `/quality` (`Duplicates` & `All` tabs; `1440x900` & `390x844`; Light & Dark; EN & RU)
- **Problem:** Four visual consistency paper-cuts on `/quality`:
  1. `DuplicateEventCard` renders `Looks Correct` as an icon-less `variant="ghost"` button (`DuplicateEventCard.tsx:279-288`), whereas warning cards in `DataQualityPage.tsx:563-570` render the identical label with `variant="secondary"` + `<CheckCircle2 />`, and `PlaceHierarchyCard.tsx:230-320` uses `<XCircle />`.
  2. On `390px` mobile, `DuplicateEventCard` footer (`flex-wrap justify-end gap-2`) places `Looks Correct` + `Customize` on row 1 and leaves `Merge Events` orphaned right-aligned on row 2 (`mobile_390_light_en_conflict_card.png`, `mobile_390_dark_ru_duplicates.png`).
  3. Summary badge `Duplicate Events: 0` remains bright blue `variant="brand"` (`DataQualityPage.tsx:416-423`) when `duplicateEvents.length === 0` (`desktop_1440_light_en_empty.png`), contrasting with `Place Hierarchy Fixes: 0` (`variant="neutral"`).
  4. `Candidate Duplicate Pairs` wraps `<EmptyState>` inside `<Card className="p-6">` (`DataQualityPage.tsx:594-600`), producing double solid+dashed borders directly above `DuplicateEventsSubView`'s single-bordered `<EmptyState>` (`DuplicateEventsSubView.tsx:368-372`).
- **Evidence:**
  - `frontend/src/components/quality/DuplicateEventCard.tsx:278-310`
  - `frontend/src/pages/DataQualityPage.tsx:416-423, 563-570, 594-600`
  - `frontend/src/components/quality/DuplicateEventsSubView.tsx:368-372`
  - `frontend/src/components/quality/PlaceHierarchyCard.tsx:230-320`
  - Captures: `desktop_1440_light_en_empty.png`, `desktop_1440_dark_ru_duplicates_auth.png`, `mobile_390_light_en_conflict_card.png`
- **Proposed fix:**
  - Add `<CheckCircle2 className="h-3.5 w-3.5" />` (or unified dismiss icon/variant) to `DuplicateEventCard` `Looks Correct`.
  - On `< sm`, make the primary card CTA `w-full sm:w-auto` (or lay out secondary actions cleanly above a full-width primary CTA) so buttons align intentionally at `390px`.
  - Switch `summaryDuplicateEvents` badge variant to `duplicateEvents.length > 0 ? 'brand' : 'neutral'`.
  - Remove the redundant outer `<Card>` wrapper from `Candidate Duplicate Pairs` `<EmptyState>` so both sections on `Duplicates` share the same visual framing.
- **Acceptance criteria:**
  - `Looks Correct` shares consistent icon/styling across `/quality` cards.
  - At `390px`, `DuplicateEventCard` footer actions align symmetrically without an awkward half-width orphan button on row 2.
  - Zero-count `Duplicate Events: 0` badge uses `neutral`, and both empty states on `Duplicates` use consistent single-border framing.
- **Tags:** `consistency`, `layout`

---

### UX-136 [minor] Replace untranslated `settings.tabs.appearance` key in mobile `AppShell` navigation drawer *(incidental shell finding)* — **[FIXED]**
- **Re-Audit (2026-11-14):** **fixed** — Added `"appearance": "Appearance"` / `"Оформление"` in `frontend/src/i18n/{en,ru}.json:366`; verified live in `390x844` mobile drawer for both EN (`Appearance`) and RU (`Оформление`).
- **Screen / viewport / theme / lang:** `/quality` mobile navigation drawer (`AppShell`; `390x844`; Light & Dark; EN & RU)
- **Problem:** Opening the mobile navigation menu renders the literal untranslated i18n key `"settings.tabs.appearance"` above `ThemeToggleButton`.
- **Evidence:**
  - `frontend/src/components/ui/app-shell.tsx:363` (`{t('settings.tabs.appearance')}`); missing in `frontend/src/i18n/en.json` and `frontend/src/i18n/ru.json`.
  - Live snapshot: `page-2026-11-14T14-47-32-873Z.yml`.
- **Proposed fix:** Replace with an existing localized label (or add `settings.tabs.appearance` to both `frontend/src/i18n/en.json` and `frontend/src/i18n/ru.json`).
- **Acceptance criteria:**
  - Opening the mobile menu at `390px` displays localized text in both EN and RU instead of `settings.tabs.appearance`.
- **Tags:** `copy`, `consistency`

---

### UX-137 [suggestion] Convert `DuplicateEventCustomizeModal` to a full-screen / bottom-sheet layout on `< 768px` viewports per Design System — **[DEFERRED TO TECHNICAL DEBT]**
- **Re-Audit (2026-11-14):** **deferred to technical debt** — Edge-to-edge full-screen `< md` dimensions (`w-full h-full md:h-auto`, verified `390x844` and `390x667` with zero horizontal overflow) landed in `dialog.tsx:208, 216, 231`; bottom-sheet variant polish and `pb-[env(safe-area-inset-bottom)]` / `md:rounded-b-2xl` footer treatment remain reserved in `docs/technical-debt.md`.
- **Screen / viewport / theme / lang:** `DuplicateEventCustomizeModal` (`390x844` & `390x667`; Light & Dark; EN & RU)
- **Problem:** On mobile viewports (`390x844` and `390x667`), `Dialog` renders as a centered floating card with outer viewport margins rather than full-width/full-height chrome, shrinking usable width and height on compact mobile screens (`bodyScrollH: 858 > bodyClientH: 495` at `390x667`) instead of matching `docs/design-system/components.md:204` (`On mobile screens (< 768px), md+ dialogs convert to full-screen viewports (inset-0) or slide up as bottom sheets with safe-area padding`).
- **Evidence:**
  - `docs/design-system/components.md:204`
  - Captures: `mobile_390_dark_ru_modal.png`, `mobile_390_light_en_modal.png`, `mobile_390x667_dark_ru_modal.png`
- **Proposed fix:** Render `md`+ `Dialog` instances as full-screen (`inset-0 max-h-dvh rounded-none`) or full-width bottom sheet with `pb-[env(safe-area-inset-bottom)]` below `768px`.
- **Acceptance criteria:**
  - On `< 768px` mobile widths, `DuplicateEventCustomizeModal` expands cleanly edge-to-edge with sticky header/footer and safe-area bottom padding.
- **Tags:** `space`, `layout`, `consistency`

---

## Positive Notes
- **Strong WCAG 2.2 AA contrast in both Light and Dark themes:** Automated canvas alpha-compositing audit across `90` rendered text/background pairs on `/quality?tab=duplicates` (Dark RU) and inside `DuplicateEventCustomizeModal` (Light EN & Dark RU) recorded `0` contrast failures (`Рождение` `4.64:1`, `Готово к объединению` `4.94:1`, `Требует проверки` `6.11:1`, `Объединить события` `5.83:1`).
- **Zero horizontal overflow at `390px` with long Cyrillic names:** The `DuplicateEventCard.tsx:277` fix holds cleanly across all inspected states (`docOverflow: 0`, card width `358px` inside `390px` viewport), wrapping long names (`Александра Константиновна Рождественская (I0115)`) and archival descriptions gracefully.
- **Clear Recorded vs Proposed two-column preview:** Grouping `Recorded Variants` on the left and `Proposed Merged Event` on the right makes the merge structure immediately scannable on desktop (`1440px`) and stacks cleanly at `390px`.
- **Idiomatic Russian event labels & accessible form controls:** RU event type labels and headings (`Рождение`, `Проживание`, `Брак / Венчание`, `Объединение событий`) are natural and consistent with `Event` / `Событие` in `docs/memory-bank/glossary.md`, `DuplicateEventCustomizeModal` inputs carry proper `useId()` `<label>` associations, and `Escape` closes the modal while restoring focus to the invoking `Customize` button.
