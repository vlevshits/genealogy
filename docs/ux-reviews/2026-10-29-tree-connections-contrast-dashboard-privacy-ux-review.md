# UX Review — Interactive Tree Connections, Visual Contrast & Clarity, and Public Dashboard Privacy — 2026-10-29

## Verdict: NEEDS FIXES

Screens reviewed:
- `/tree` (Tree Canvas in Light and Dark themes, authenticated and public guest modes):
  - Dual-layer SVG hit targets (`stroke-width="14"` desktop / `stroke-width="20"` mobile).
  - Ochre marriage lines (`stroke-amber-500`) with midpoint union circle dot indicators.
  - Direct lineage (`stroke-brand/80`) vs collateral (`stroke-secondary/40`) connector paths.
  - Edge selection HUD (`ConnectionNavPopover`) with jump-to-node action, kinship finder prefill, living privacy masking.
  - Endpoint highlight rings (`ring-2 ring-brand`) and camera pulse animation.
  - Subpixel integer translation clamping on camera idle.
- `/` (Dashboard in public guest mode vs authenticated mode):
  - Public guest 2-widget grid: `AnniversariesWidget` & `TreeGlanceWidget`, omitting `RecentActivityWidget` and `LatestPhotosWidget`.
  - `PublicGuestBanner` with living member privacy notice and sign-in CTA.
- Viewports verified: Desktop (1440px), Mobile (390px, 360px).
- Themes verified: Light and Dark mode.
- Locales verified: English (`en`) and Russian (`ru`).

Evidence & Verification Artifacts:
- Browser viewport screenshots:
  - `.playwright-mcp/page-2026-10-29T08-30-33-462Z.png` (Desktop 1440px Light Dashboard)
  - `.playwright-mcp/page-2026-10-29T08-31-00-939Z.png` (Desktop 1440px Dark Dashboard)
  - `.playwright-mcp/page-2026-10-29T08-31-12-428Z.png` (Mobile 390px Dark Dashboard EN)
  - `.playwright-mcp/page-2026-10-29T08-31-40-646Z.png` (Mobile 390px Dark Dashboard RU)
- Test suite: `frontend/tests/components/tree-canvas.test.tsx` (34 tests passing), `frontend/tests/i18n/parity.test.ts` (12 tests passing). All 18 frontend suites (195 unit tests) passing green.

---

## Status of Previous Tasks

| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
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
| **UX-037** | Stacking / Overflow collision in `TreeToolbar` on tablet and medium viewports (640px - 1024px) | open |
| **UX-038** | Direct-lineage vertical stems overwritten by collateral SVG paths | **fixed** |
| **UX-039** | Low non-text contrast (< 3:1) for connector edges | **fixed in M2.5 / re-audited in UX-067** |
| **UX-040** | Initial camera pan hardcoded to `{ x: 480, y: 280 }`, placing tree root off-screen on mobile | **fixed** |
| **UX-041** | Missing tooltip / explanatory subtitle for "Wide tree" / "Compact tree" toggle | **fixed** |
| **UX-042** | Lack of card-level visual hierarchy between direct lineage and collateral branches | open |
| **UX-043** | Mobile toolbar vertical height reduction and icon assistance for Wide/Compact toggle | suggestion (open) |
| **UX-044** | Misleading divider label and dead action button when SMTP is disabled | **fixed** |
| **UX-045** | Inaccessible and skipped SMTP disabled hint for screen reader and keyboard users | **fixed** |
| **UX-046** | Missing keyboard focus management and long email text wrapping in `magic_sent` confirmation view | **fixed** |
| **UX-047** | Multi-line Emergency Recovery button icon alignment on mobile viewports | **fixed** |
| **UX-048** | Disconnected / floating connector lines in LOD 2 zoom mode (< 0.25x) | **fixed** |
| **UX-049** | Semantic exclusion and scaling illegibility of component headers in ARIA tree (WCAG 1.3.1) | **fixed** |
| **UX-050** | Shelf grid start X coordinate clamped to 0, misaligning isolated records | **fixed** |
| **UX-051** | LOD 2 micro-pill renders full name instead of concise surname/initials | **fixed** |
| **UX-052** | Missing Home person anchor badge in LOD 2 micro-pill | **fixed** |
| **UX-053** | Singular English pluralization for All Nodes count badge (`1 individuals`) | **fixed** |
| **UX-054** | SegmentedControl horizontal overflow on mobile viewports (< 480px) lacks scroll affordance | suggestion (open) |
| **UX-055** | Cross-generational multi-tier spouse connectors in all-nodes mode route via outer corridors | suggestion (open) |
| **UX-056** | Hardware Security Key (YubiKey) Registration Blocked by Enforced `authenticatorAttachment` | **fixed** |
| **UX-057** | Misleading Error Toast Title on Passkey Enrollment / Management Failures | **fixed** |
| **UX-058** | Silent Failure and Premature Navigation on Passkey Enrollment in `MagicLinkVerifyCard` | **fixed** |
| **UX-059** | Missing WebAuthn `TimeoutError` and `AbortError` Mapping in `getLocalizedErrorMessage` | **fixed** |
| **UX-060** | Jargon-Heavy Copy for Passkey Replay Attack Error | **fixed** |
| **UX-061** | Off-Screen Error Placement on Long Mobile Viewports in `SetupWizardCard` | **fixed** |

---

## Tasks for frontend-developer

### UX-062 [critical] Keyboard & Screen Reader Violation: Connection Edge Buttons Inside `aria-hidden="true"` SVG Container
- **Screen / viewport / theme / lang:** `/tree` — all viewports, themes, locales.
- **Problem:** In `TreeCanvas.tsx:751-815`, the container `<svg>` element has `aria-hidden="true"`, while inside it every connection edge renders an interactive, keyboard-focusable path with `role="button"`, `tabIndex={0}`, and `aria-label`. WCAG 2.2 AA (Criterion 4.1.2 Name, Role, Value and Criterion 1.3.1 Info and Relationships) and W3C WAI-ARIA 1.2 specify that focusable elements must never be descendants of an `aria-hidden="true"` container (Axe rule: `aria-hidden-focus`). When a keyboard or screen reader user tabs into the tree, the focus indicator enters an invisible element that assistive technologies are explicitly instructed to ignore, rendering the connection navigation HUD completely broken and confusing for screen reader users.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:751-815`
  ```tsx
  <svg
    aria-hidden="true"
    className="overflow-visible pointer-events-none absolute left-0 top-0"
    width={1}
    height={1}
  >
    {sortedEdges.map((edge) => (
      <g key={edge.id} className="group">
        {/* Lower transparent hit path (wide clickable target) */}
        <path
          d={edge.pathD}
          fill="none"
          stroke="transparent"
          strokeWidth={isMobile ? 20 : 14}
          style={{ pointerEvents: 'stroke' }}
          className="cursor-pointer focus:outline-hidden"
          role="button"
          tabIndex={0}
          aria-label={`${edgeAriaLabel}: ${t('tree.connection.select_hint')}`}
  ```
- **Proposed fix:**
  1. Remove `aria-hidden="true"` from the parent `<svg>` layer. Give the `<svg>` `role="group"` and `aria-label={t('tree.connections_layer')}`.
  2. Mark only decorative visual paths (the upper visual stroke `<path aria-hidden="true">` and `<circle aria-hidden="true">`) with `aria-hidden="true"`.
- **Acceptance criteria:**
  - Automated accessibility linters / axe-core audits report zero `aria-hidden-focus` violations on `/tree`.
  - When tabbing onto connection edges with VoiceOver/NVDA, the screen reader announces the edge role and accessible label.
- **Tags:** `a11y`

---

### UX-063 [major] Missing Visible Keyboard Focus Ring on Connection Edge Paths
- **Screen / viewport / theme / lang:** `/tree` — all viewports, themes, locales.
- **Problem:** The transparent hit path has `stroke="transparent"` and `className="cursor-pointer focus:outline-hidden"`. When a keyboard user tabs onto an edge, there is no outline, no focus ring (`focus-visible:ring-*`), and no visible stroke change on `:focus-visible`. The visual styling only changes when `isSelected` is true (after activation). This directly violates WCAG 2.2 Criterion 2.4.7 (Focus Visible, Level AA) and Criterion 2.4.11 (Focus Appearance, Level AA).
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:799-805`
  ```tsx
  <path
    d={edge.pathD}
    fill="none"
    stroke="transparent"
    strokeWidth={isMobile ? 20 : 14}
    style={{ pointerEvents: 'stroke' }}
    className="cursor-pointer focus:outline-hidden"
    role="button"
    tabIndex={0}
  ```
- **Proposed fix:**
  Provide visible keyboard focus feedback on the hit path or linked visual path when the edge is focused:
  - Add `focus-visible:stroke-brand/50 focus-visible:outline-none` to the hit path, or use a CSS selector / group state so that when the hit path has `:focus-visible`, the visual path activates a high-contrast focus glow/ring.
- **Acceptance criteria:**
  - When navigating edges via the `Tab` key, a prominent visible indicator (contrast $\ge 3:1$ against canvas) clearly highlights the focused edge before activation.
- **Tags:** `a11y`

---

### UX-064 [major] Public Guest Banner Action Stacking & Squeezed Layout on Mobile Viewports (< 640px)
- **Screen / viewport / theme / lang:** `/` (Dashboard) and `/tree` (Public Guest View) on viewports $\le 390\text{px}$, especially in Russian (`ru`).
- **Problem:** `Alert` renders content and action in an unyielding horizontal flex row (`flex items-start justify-between gap-3`). On mobile viewports (~390px), the "Sign In to Family Archive" button and dismiss button occupy > 200px, leaving only ~120px for the text column. This forces the heading, badge, and description to wrap into a tall, 1-word-per-line column in Russian ("Вы / просматриваете / публичную / версию..."). Furthermore, in Russian, the "Войти в семейный архив" button squeezes the dismiss "X" button off the right edge of the card, causing horizontal overflow/clipping.
- **Evidence:** Browser screenshots `.playwright-mcp/page-2026-10-29T08-31-12-428Z.png` (EN mobile) and `.playwright-mcp/page-2026-10-29T08-31-40-646Z.png` (RU mobile).
- **Proposed fix:**
  In `PublicGuestBanner.tsx`, restructure the alert layout so that on small screens (`< sm`), action buttons stack below or wrap cleanly under the text:
  ```tsx
  <Alert
    variant="info"
    className={cn("flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3", className)}
    ...
  ```
  On mobile viewports, the "Sign in" button should stretch to full width or sit beneath the description text, and the dismiss icon button should remain accessible in the top-right corner.
- **Acceptance criteria:**
  - On 360px and 390px mobile screens in both English and Russian, the description text occupies full container width with normal sentence wrapping.
  - The dismiss "X" button is never clipped or pushed past the right viewport margin.
- **Tags:** `layout`, `space`

---

### UX-065 [major] Sub-40px Touch Targets on Connection Popover Action Buttons
- **Screen / viewport / theme / lang:** `/tree` (Connection HUD Popover) on mobile and touch devices.
- **Problem:** The Close button (`<X>`), Jump to Person buttons (`<Crosshair>`), and View Person buttons (`<ExternalLink>`) all specify `className="h-7 w-7"` ($28\times 28\text{px}$) without `.touch-target-expand`. WCAG 2.2 AA Criterion 2.5.8 (Target Size - Minimum) requires at least $24\times 24\text{px}$ spacing or $40\times 40\text{px}$ minimum target size, and the project design system mandates touch targets $\ge 40\times 40\text{px}$ (`--size-touch-min: 44px`). On mobile devices, 28px icon buttons are frustrating to tap accurately and cause mis-taps.
- **Evidence:** `frontend/src/components/tree/ConnectionNavPopover.tsx:125-133, 156-176, 197-217`
  ```tsx
  <IconButton
    variant="ghost"
    size="sm"
    onClick={onClose}
    label={t('tree.connection.close')}
    className="h-7 w-7 text-secondary hover:text-primary"
  >
    <X className="h-4 w-4" />
  </IconButton>
  ```
- **Proposed fix:**
  Add `touch-target-expand` to all `IconButton` components in `ConnectionNavPopover.tsx`, or use `h-8 w-8 min-h-8 min-w-8 touch-target-expand` so coarse-pointer devices receive a hit area $\ge 44\times 44\text{px}$.
- **Acceptance criteria:**
  - All interactive buttons in `ConnectionNavPopover` have coarse-pointer hit areas of at least $44\times 44\text{px}$.
- **Tags:** `a11y`

---

### UX-066 [major] Programmatic Focus Management and Popover Dismissal on "Jump to Person"
- **Screen / viewport / theme / lang:** `/tree`
- **Problem:** When the user clicks the "Jump to person" button inside `ConnectionNavPopover`, the camera centers on the node and pulses, but:
  1. Focus is NOT programmatically shifted to the target `PersonNodeCard` in the DOM. For keyboard and screen reader users, focus remains stuck on the "Jump to" button in the popover.
  2. The popover remains open and flies across the screen following camera pan coordinates, obscuring the newly focused card.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:1037-1048`
- **Proposed fix:**
  On "Jump to person":
  1. Dismiss or close the popover (`setSelectedEdge(null)`).
  2. Programmatically focus the target person node card in the DOM:
     ```ts
     const cardEl = containerRef.current?.querySelector<HTMLElement>(`[data-node-id="${personId}"]`);
     cardEl?.focus();
     ```
  3. When `onFindKinship` is invoked, also dismiss the popover so it does not remain under the dialog.
- **Acceptance criteria:**
  - Activating "Jump to person" shifts keyboard focus to the target node card and dismisses the connection popover.
  - Activating "Find kinship path" closes the popover before opening the `KinshipFinderDialog`.
- **Tags:** `a11y`, `states`

---

### UX-067 [minor] Collateral Connector Line Contrast Below 3:1 Non-Text Requirement in Light & Dark Mode
- **Screen / viewport / theme / lang:** `/tree` — canvas view, light and dark themes.
- **Problem:** Collateral and placeholder branches use `strokeClass = 'stroke-secondary/40'`. On the light canvas background `#F8F9FA`, 40% opacity of `#4B5563` yields a contrast ratio of only **1.92:1** (and ~2.1:1 on dark canvas `#0E1116`), failing WCAG 2.2 Criterion 1.4.11 (Non-text Contrast $\ge 3:1$). Because connector lines are essential graphical objects for tracing pedigree and parent-child lines, low-contrast washed out lines cause severe strain for older family members on dim or standard laptop screens.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:765, 771`
  ```tsx
  let strokeClass = 'stroke-secondary/40';
  ```
- **Proposed fix:**
  Increase the stroke opacity of collateral branches to `stroke-secondary/70` in light mode and `stroke-secondary/70` in dark mode (or `stroke-slate-500 dark:stroke-slate-400`), achieving $\ge 3.2:1$ contrast against the canvas while remaining visually subordinate to direct-lineage `stroke-brand/80`.
- **Acceptance criteria:**
  - Non-text contrast ratio of collateral connector lines against both light and dark canvas backgrounds is $\ge 3.0:1$.
- **Tags:** `consistency`, `a11y`

---

### UX-068 [minor] Missing Accessible State / Announcement for Connection Endpoint Highlights on PersonNodeCard
- **Screen / viewport / theme / lang:** `/tree`
- **Problem:** When an edge is selected, `isEdgeEndpointHighlighted` applies a visual ring (`ring-2 ring-brand shadow-elevation-2`) and `isPulsing` applies an animation. However, `aria-selected` only reflects `isSelected || isFocal`. Screen reader users navigating the tree canvas have no programmatic indication that the node card is an endpoint of the currently inspected connection.
- **Evidence:** `frontend/src/components/tree/PersonNodeCard.tsx:280, 381, 411`
- **Proposed fix:**
  When `isEdgeEndpointHighlighted` is true, expose this relationship via `aria-describedby` or an accessible label/badge (e.g. `t('tree.connection.endpoint_badge')`) so screen reader users are aware of the highlight.
- **Acceptance criteria:**
  - Screen reader users receive an indication when a card is one of the active endpoints of the currently selected connection.
- **Tags:** `a11y`, `states`

---

## Positive Notes
- **WCAG 2.2 AA Contrast Achievement on Tertiary Text:** Updating `--color-text-tertiary` to `#6B7280` in light mode (4.73:1) and `#9CA3AF` in dark mode (6.66:1) successfully satisfies the $\ge 4.5:1$ contrast requirement for subtle metadata (dates, places, GIDs).
- **Subpixel Translation Clamping on Idle:** Rounding `pan.x` and `pan.y` to integer pixels via `Math.round(pan.x)` whenever camera animation and dragging are idle prevents subpixel blur and chromatic distortion on non-Retina displays.
- **Midpoint Union Ochre Styling & Visual Distinction:** Ochre marriage lines (`stroke-amber-500`) with circular midpoint dot indicators provide immediate, intuitive visual differentiation from parent-child stems.
- **Privacy Masking in Connection HUD:** `ConnectionNavPopover` flawlessly respects privacy masking for living individuals in public guest mode (displaying initials "S." and omitting sensitive dates and avatars).
- **Public Dashboard Privacy Segregation:** Public anonymous guests are presented with a focused, privacy-respecting 2-widget grid (Anniversaries and Tree at a Glance), completely hiding sensitive transaction audit logs and photos.
