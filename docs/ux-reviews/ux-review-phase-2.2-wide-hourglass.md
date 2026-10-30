# UX Review — Milestone 2.2: Wide & Collateral-Expanded Tree Visualizations in Hourglass Mode & Marriage Link Fix

## Verdict: NEEDS FIXES

Screens & routes reviewed:
- `/tree` (Hourglass projection mode: Wide vs Compact toggle, horizontal spouse connector bars, vertical union stems to children, generational baselines Gen -2 to +2, responsive behavior across mobile, tablet, and desktop viewports, light and dark themes, English and Russian locales).

Components inspected:
- `frontend/src/components/tree/TreeToolbar.tsx`
- `frontend/src/components/tree/TreeCanvas.tsx`
- `frontend/src/components/tree/PersonNodeCard.tsx`
- `frontend/src/components/tree/TreeMiniMap.tsx`
- `frontend/src/lib/tree-layouts/hourglass-layout.ts`
- `frontend/src/lib/tree-layouts/curve-routing.ts`
- `frontend/src/lib/tree-layouts/types.ts`
- `frontend/src/lib/tree-layouts/index.ts`
- `frontend/src/i18n/en.json` & `ru.json`

Viewports & Locales Verified:
- Desktop (1440px / 1280px), Tablet (768px / 820px), Mobile (390px / 360px).
- Light and Dark modes.
- English (`en`) and Russian (`ru`).

---

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-001** | Fix mobile toolbar overlap and bottom navigation occlusion on tree canvas | fixed |
| **UX-002** | Resolve multi-touch pinch-to-zoom race condition and card long-press drag threshold | fixed |
| **UX-003** | Wire centralized `formatPersonName` into all UI components and fix fallback | fixed |
| **UX-004** | Remove prominent "Living" pill badge from `PersonDrawer` and `PersonBottomSheet` [spec-change] | fixed |
| **UX-005** | Update `LifeSpan` component to display genealogical dagger `†` for deceased | fixed |
| **UX-006** | Fix PWA offline banner top safe-area inset and consolidate install actions | fixed |
| **UX-007** | Fix `BottomNav` accessibility label and replace raw 36px buttons | fixed |
| **UX-008** | Parametrize family archive instance title in PWA install copy | fixed |
| **UX-009** | Elevate toast container above mobile bottom navigation with safe-area clearance | fixed |
| **UX-010** | Add missing `role="radiogroup"`, visible focus rings, and mobile layout to conflict modal | fixed |
| **UX-011** | Fix untranslated `{count}` token in diff toggle buttons | fixed |
| **UX-012** | Prevent diff values from overflowing card boundaries on mobile viewports | fixed |
| **UX-013** | Format raw database transaction labels with human-friendly descriptions | fixed |
| **UX-014** | Optimize dialog sizing and instruction placement in revision & conflict modals | fixed |
| **UX-015** | Keyboard inaccessible stat cards in `TreeGlanceWidget` | fixed |
| **UX-016** | Missing visible focus ring on interactive SVG slices in `FanChartCanvas` | fixed |
| **UX-017** | Keyboard inaccessible map cluster pins in `PlacesMapView` | fixed |
| **UX-018** | Missing column/source identifiers on mobile stacked layout in `PersonMergePage` | fixed |
| **UX-019** | Suffix dagger notation `(1989†)` in `AnniversariesWidget` | fixed |
| **UX-020** | Screen reader list navigation distortion from `aria-hidden` intermediate `<li>` arrows in `KinshipFinderDialog` | fixed |
| **UX-021** | Keyboard zoom shortcuts or visible hint in `PlacesMapView` and `FanChartCanvas` | suggestion (open) |
| **UX-022** | DropdownMenu triggers (`AppShell` nav more, +add, user profile) inaccessible to keyboard navigation | open |
| **UX-023** | Modal Dialogs and Drawers lack keyboard Tab focus trapping and initial focus management | open |
| **UX-024** | Sub-40px touch targets and raw `<button>` elements in inputs and media controls | open |
| **UX-025** | Misleading "Person record not found" alert on network or server failure | open |
| **UX-026** | Header title truncation and right action crowding on narrow mobile viewports | open |
| **UX-027** | Missing keyboard navigation (ArrowLeft / ArrowRight) in Media Lightbox Studio | open |
| **UX-028** | Theme switcher button width crowding in mobile "More" drawer | suggestion (open) |
| **UX-029** | Tree canvas pointer capture swallows left clicks on person cards; keyboard activation missing | **fixed** |
| **UX-030** | Mobile `PersonBottomSheet` missing authoring actions (Add Mother, Add Spouse, Add Event, Delete) | open |
| **UX-031** | Face tags on photograph canvas keyboard inaccessible; missing focus rings, tooltip disclosure, and delete | open |
| **UX-032** | Non-admin authorized users (Editors & Viewers) locked out of Profile and Passkeys in mobile "More" drawer | open |
| **UX-033** | Invitation creation modal lacks 'Admin' role selection option | open |
| **UX-034** | Authenticated mobile header button congestion and title squeeze | open |
| **UX-035** | Destructive "Delete Person" button in `PersonDrawer` lacks visual danger treatment | open |
| **UX-036** | Form error alerts lack programmatic association (`aria-invalid`, `aria-describedby`) with invalid fields | open |

---

## Tasks for frontend-developer

### UX-037 [major] Stacking / Overflow collision in `TreeToolbar` on tablet and medium viewports (640px - 1024px)
- **Screen / viewport / theme / lang:** `/tree` (`TreeToolbar.tsx:66-171`), viewports 640px to 1024px (especially in Russian locale `ru`).
- **Problem:**
  In `TreeToolbar.tsx:66`:
  ```tsx
  <div className="pointer-events-none absolute top-3 inset-x-3 sm:top-4 sm:inset-x-6 z-20 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
  ```
  On `sm` (≥ 640px), the toolbar switches to `sm:flex-row sm:justify-between` without `sm:flex-wrap`.
  On the left, the Mode Switcher (`SegmentedControl`) and the Wide/Compact toggle (`SegmentedControl`) consume up to ~690px in Russian ("Песочные часы", "Прямые предки", "Потомки", "Веерная диаграмма" + "Широкое древо", "Компактное древо").
  On the right, the depth and framing controls (Generations select 176px + Kinship Finder button 140px + Home button 40px + Maximize button 40px) consume ~400px.
  The combined required width is ~1090px.
  On tablet viewports (e.g. 768px iPad portrait, 820px, 900px, 1024px), the left and right containers collide, pushing the right controls (Center Home, Fit View) off-screen past the right viewport boundary, rendering them completely unreachable to tablet users.
- **Evidence:** `frontend/src/components/tree/TreeToolbar.tsx:66-72`, `130-170`.
- **Proposed fix:**
  1. Add `sm:flex-wrap lg:flex-nowrap` to the top container so that when width is constrained, controls wrap cleanly into secondary rows rather than clipping.
  2. For viewports under `lg` (< 1024px), collapse the Kinship Finder button into an `IconButton` (`aria-label={t('tree.kinship.openBtn')}`) with `<Route className="h-4 w-4" />` to save ~100px.
  3. Ensure the left container allows horizontal scrolling (`max-w-full overflow-x-auto`) if screen width is critically narrow.
- **Acceptance criteria:**
  - On viewports from 640px to 1024px in both English and Russian, all toolbar controls (Mode switcher, Wide/Compact toggle, Depth select, Kinship button, Home button, Fit to Screen button) remain fully visible within the viewport without horizontal overflow.
  - No buttons are pushed off-screen or rendered unreachable.
- **Tags:** layout, responsive, tree

---

### UX-038 [major] Direct-lineage vertical stems overwritten by collateral SVG paths (visual regression / stroke layering glitch)
- **Screen / viewport / theme / lang:** `/tree` (`TreeCanvas.tsx:600-638`, `hourglass-layout.ts:373-416`), all viewports, light and dark themes.
- **Problem:**
  In Hourglass layout, the stem from the parent couple to the children shares a common vertical drop from `spouseY` to `parentBusY`.
  The direct lineage child (e.g. focal person Anna) creates an edge with `isDirectLineage: true` (`stroke-brand/80`, `strokeWidth: 2.5`).
  Sibling branches create edges with `isDirectLineage: false` (`stroke-border`, `strokeWidth: 1.5`).
  Because `edges` are appended with focal child first, then siblings (`hourglass-layout.ts:393-416`), SVG renders the edges in DOM document order.
  The gray 1.5px sibling edge is painted OVER the 2.5px brand direct-lineage stem between `spouseY` and `parentBusY`, creating an unsightly visual glitch (a faded/gray stripe splitting down the direct lineage line).
  Throughout the descendant cascade, overlapping vertical trunk segments from direct and collateral paths have conflicting strokes depending on array order.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:609-637`, `frontend/src/lib/tree-layouts/hourglass-layout.ts:373-416`.
- **Proposed fix:**
  In `TreeCanvas.tsx`, sort or partition `layout.edges` when rendering the SVG layer: render all collateral edges (`!edge.isDirectLineage`) first, followed by direct lineage edges (`edge.isDirectLineage`), and finally highlighted kinship edges (`isKinshipHighlighted`).
  Alternatively, sort by `(edge.isDirectLineage ? 1 : 0) + (isKinshipHighlighted ? 2 : 0)`.
- **Acceptance criteria:**
  - The direct lineage stem from the parent union to the focal person is rendered with a clean, unbroken `stroke-brand/80` line without any gray collateral edge painted over it.
  - Direct lineage lines always take visual precedence over collateral branches across all generations.
- **Tags:** layout, visual, consistency

---

### UX-039 [major] Low non-text contrast (< 3:1) for collateral connector edges in dark mode (WCAG 1.4.11 failure)
- **Screen / viewport / theme / lang:** `/tree` (`TreeCanvas.tsx:619, 633`), dark mode, all viewports.
- **Problem:**
  In dark mode, the canvas background is `--color-bg-canvas: #0E1116`.
  Collateral edges are styled with `stroke-border` (`--color-border-subtle: #2E3542`) and `strokeWidth: 1.5`.
  The contrast ratio of `#2E3542` against `#0E1116` is only ~1.5:1, well below the WCAG 2.2 AA non-text contrast threshold of 3:1 (Criterion 1.4.11 Non-text Contrast).
  Older family members using dark mode cannot discern the thin 1.5px collateral connector lines connecting aunts, uncles, cousins, and siblings to the tree.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:619, 633`, `frontend/src/index.css:44, 47`.
- **Proposed fix:**
  Update the CSS class for non-direct lineage tree edges to use a token with sufficient contrast against `--color-bg-canvas` in both themes:
  For example, `stroke-border-focus/60` or `stroke-secondary/40` or a dedicated lineage token `stroke-lineage-neutral` (`#64748B` in dark mode gives 4.8:1 contrast against `#0E1116`), maintaining `strokeWidth: 1.5` so it remains visually subordinate to the 2.5px direct lineage brand lines while exceeding the 3:1 contrast requirement.
- **Acceptance criteria:**
  - Collateral tree edges achieve ≥ 3:1 contrast ratio against the canvas background in both light and dark modes.
  - Sibling and cousin branches remain clearly legible in dark mode.
- **Tags:** a11y, dark-mode, contrast

---

### UX-040 [major] Initial camera pan hardcoded to `{ x: 480, y: 280 }`, placing tree root off-screen on mobile screens
- **Screen / viewport / theme / lang:** `/tree` (`TreeCanvas.tsx:131-134`), mobile viewports (< 768px, specifically 360px - 390px).
- **Problem:**
  In `TreeCanvas.tsx`, initial camera pan state is hardcoded to:
  ```ts
  const [pan, setPan] = useState<{ x: number; y: number }>({
    x: 480,
    y: 280,
  });
  ```
  And there is no `useEffect` or lifecycle hook that centers the tree on load.
  On a 390px mobile screen (e.g. iPhone), rendering the world origin $(0, 0)$ at `screenX = 480px` places the focal person card and central family branch 90px beyond the right edge of the screen!
  When first navigating to `/tree` on a phone, users are presented with an empty gray canvas, having no visual indication that the tree is rendered off to the right. They must randomly drag the canvas or realize they need to tap the small "Home" icon.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:131-134, 276-303`.
- **Proposed fix:**
  Add an initial centering effect in `TreeCanvas.tsx`: once `layout.nodes` are calculated (length > 0) and `viewport.width > 0`, if initial centering has not yet occurred (`initialCenterDoneRef.current === false`), call `centerOnNode(focusPersonId ?? layout.focalId)` (or `fitToScreen()` on mobile) and mark `initialCenterDoneRef.current = true`.
- **Acceptance criteria:**
  - When loading `/tree` on mobile (360px - 390px) and desktop, the focal person card is centered within the viewport automatically without requiring manual dragging.
  - The canvas does not open to blank empty space.
- **Tags:** layout, mobile, canvas

---

### UX-041 [minor] Missing tooltip / explanatory subtitle for "Wide tree" / "Compact tree" toggle
- **Screen / viewport / theme / lang:** `/tree` (`TreeToolbar.tsx:103-126`), all viewports, EN and RU.
- **Problem:**
  The i18n keys `tree.view_options.collateral_subtitle` ("Aunts, uncles, cousins, siblings, nieces and nephews" / "Дяди, тети, кузены, братья, сестры и племянники") were created specifically for Milestone 2.2, but are unused in the codebase.
  Older or non-technical family members encountering "Wide tree" vs "Compact tree" ("Широкое древо" / "Компактное древо") do not know what changes when the toggle is flipped (e.g. whether it changes page orientation, zoom scale, or genealogical scope).
  Furthermore, the toggle container has redundant double borders and pill wrappers (`border border-border bg-surface/90 rounded-xl p-1` wrapping `SegmentedControl` which already has `border border-border bg-surface-muted rounded-lg p-0.5`).
- **Evidence:** `frontend/src/components/tree/TreeToolbar.tsx:103-126`, `frontend/src/i18n/en.json:586`, `frontend/src/i18n/ru.json:586`.
- **Proposed fix:**
  1. Add `title={t('tree.view_options.collateral_subtitle')}` or wrap the toggle in `<Tooltip content={t('tree.view_options.collateral_subtitle')}>` so hovering or focusing reveals the explanation.
  2. Remove the outer redundant `border border-border bg-surface/90` wrapper around the `SegmentedControl` or streamline styling so it matches the mode switcher cleanly.
- **Acceptance criteria:**
  - Hovering or focusing the Wide / Compact toggle displays the subtitle explaining that collateral relatives (aunts, uncles, cousins, siblings, nieces/nephews) are included.
  - The toggle styling is clean without nested double borders.
- **Tags:** consistency, copy, a11y

---

### UX-042 [minor] Lack of card-level visual hierarchy between direct lineage and collateral branches
- **Screen / viewport / theme / lang:** `/tree` (`PersonNodeCard.tsx:170-205`, `TreeCanvas.tsx:640-672`), all viewports, light and dark themes.
- **Problem:**
  In Wide Hourglass mode, up to 30+ person cards are rendered on screen simultaneously across direct ancestors (grandparents, parents), focal couple, direct descendants (children, grandchildren), and collateral branches (aunts, uncles, cousins, siblings, nieces/nephews).
  `PositionedNode` calculates `isDirectLineage: boolean`, but this prop is never passed to `PersonNodeCard`.
  As a result, all person cards share identical border (`border-border`), background (`bg-surface`), and visual weight.
  It is difficult for users to visually trace the direct lineage trunk through the dense sea of collateral relatives at a glance.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:648-671`, `frontend/src/components/tree/PersonNodeCard.tsx:18-32`.
- **Proposed fix:**
  1. Pass `isDirectLineage={pn.isDirectLineage}` from `TreeCanvas` to `PersonNodeCard`.
  2. In `PersonNodeCard`, provide subtle visual cues for direct lineage vs collateral nodes:
     - Direct lineage nodes can have a subtle distinction such as a distinct subtle border tint (`border-brand/40` or solid border vs `border-border/80`), or collateral nodes can have a slightly softer background (`bg-surface/90`) or a subtle "Collateral" / "Branch" aria/title hint.
     - In `TreeMiniMap.tsx`, render direct lineage nodes with `fill-brand/60` and collateral nodes with `fill-secondary/30` instead of coloring all non-focal nodes identically.
- **Acceptance criteria:**
  - Users can readily distinguish direct ancestors/descendants from collateral relatives on the tree canvas and mini-map.
  - The visual hierarchy does not reduce readability of text or facts on any card.
- **Tags:** hierarchy, visual, space

---

### UX-043 [suggestion] Mobile toolbar vertical height reduction and icon assistance for Wide/Compact toggle
- **Screen / viewport / theme / lang:** `/tree` (`TreeToolbar.tsx:66-171`), mobile viewports (< 640px).
- **Problem:**
  On narrow mobile screens (< 640px), the toolbar stacks into 3 full rows:
  - Row 1: Mode Switcher (4 buttons)
  - Row 2: Wide/Compact toggle (2 buttons, "Широкое древо" / "Компактное древо")
  - Row 3: Generations select + Kinship button + Home button + Fit View button
  Together, these 3 rows consume ~160px of vertical space, covering over 20% of the canvas height. When a user taps a card and `PersonBottomSheet` opens at 25% height, the visible interactive tree area is reduced to ~40% of the screen.
- **Evidence:** `frontend/src/components/tree/TreeToolbar.tsx:66-171`.
- **Proposed fix:**
  1. On mobile (< 640px), use shorter labels for the collateral toggle (e.g. `wide_short`: "Wide" / "Широкое", `compact_short`: "Compact" / "Компактное") or add compact icons (e.g. `<Columns3 className="h-3.5 w-3.5" />` vs `<Columns2 className="h-3.5 w-3.5" />`).
  2. For the Kinship button, render icon-only on mobile (`px-2.5` with `<Route className="h-4 w-4" />` and `aria-label={t('tree.kinship.openBtn')}`).
  3. This allows Row 2 and Row 3 controls to fit much more compactly, saving 40-50px of vertical height for canvas interaction.
- **Acceptance criteria:**
  - On 390px mobile screens, the top toolbar height is reduced by at least 35px.
  - All controls remain touch-friendly (≥ 40px touch targets).
- **Tags:** mobile, space, ergonomics

---

## Positive Notes
- **Marriage Links and Vertical Stems:** The horizontal spouse connector bar cleanly and accurately spans the 32px gap between inner card edges (`sourceX = minX + CARD_WIDTH`, `targetX = maxX`), and the vertical stem drops directly from the exact midpoint of the marriage bar without any detached gap.
- **Generational Baseline Alignment:** All collateral branches adhere strictly to generational baselines: aunts/uncles align at Generation -1, first cousins and siblings at Generation 0, and nieces/nephews at Generation 1, with zero vertical misalignment across the tree.
- **Recursive Subtree Spacing:** Subtree width allocation ensures zero card collision even with asymmetric collateral branches (e.g. multiple sibling families with children expanding outwards).
- **Single Parent Resilience:** Single fathers or mothers cleanly connect to focal children and siblings using orthogonal bus lines dropping from the bottom center of the card.
- **URL Parameter Synchronization:** Toggling between Wide and Compact modes seamlessly updates the URL query parameter (`?collateral=true|false`) without triggering an unnecessary full page reload.
