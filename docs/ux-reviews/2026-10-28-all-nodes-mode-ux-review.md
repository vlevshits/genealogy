# UX Review — Milestone 2.5: All Nodes & Complete Graph View Mode (`TASK-TREE-ALL-NODES-MODE`) — 2026-10-28

## Verdict: OK

Screens reviewed:
- `/tree?mode=all` (`TreeCanvas.tsx`, `TreeToolbar.tsx`, `PersonNodeCard.tsx`, `all-nodes-layout.ts`):
  - Toolbar experience in `mode === 'all'`: SegmentedControl 5-way switcher with `Network` icon, dynamic depth selector suppression, and total individuals count badge.
  - Multi-component forest visual hierarchy: $\ge 300\text{px}$ spatial separation between Primary component ($C_{\text{primary}}$) and secondary branches ($C_2 \dots C_k$).
  - Component header origin badges ("Main Family Branch", "Branch 2", "Isolated Records").
  - 4-column shelf grid for isolated singletons below family branches.
  - Level of Detail (LOD) transitions across zoom levels: LOD 0 ($\ge 0.5x$), LOD 1 ($0.25x-0.5x$), and LOD 2 ($< 0.25x$ micro-pill).
  - Privacy masking across all components in public guest mode (`public_anonymous`).
- Viewports verified: Desktop (1440px), Tablet (768px), Mobile (390px, 360px).
- Themes verified: Light and Dark mode.
- Locales verified: English (`en`) and Russian (`ru`).

Evidence & Verification Suite:
- Automated tests: `frontend/tests/components/tree-canvas.test.tsx` (all 23 tests passing including 9 all-nodes and LOD 2 tests), `frontend/tests/i18n/parity.test.ts` (10 tests passing). All 18 frontend test suites (177 tests) green.
- Browser captures & snapshots: `.playwright-mcp/page-2026-10-28T16-15-03-676Z.yml`.

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
| **UX-038** | Direct-lineage vertical stems overwritten by collateral SVG paths | open |
| **UX-039** | Low non-text contrast (< 3:1) for collateral connector edges in dark mode | open |
| **UX-040** | Initial camera pan hardcoded to `{ x: 480, y: 280 }`, placing tree root off-screen on mobile | open |
| **UX-041** | Missing tooltip / explanatory subtitle for "Wide tree" / "Compact tree" toggle | open |
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

---

## Tasks for frontend-developer

### UX-048 [fixed] Disconnected / floating connector lines in LOD 2 zoom mode (< 0.25x)
- **Screen / viewport / theme / lang:** `/tree` when zoomed below $0.25x$ across all themes and locales.
- **Problem:** When zooming out below $0.25x$, `PersonNodeCard` switches to LOD 2 micro-pill rendering with `height: 28px` positioned at `top: y`. However, the layout engine (`all-nodes-layout.ts` and `hourglass-layout.ts`) computes static SVG edge coordinates assuming standard card dimensions (`CARD_HEIGHT = 92px`):
  1. The horizontal spouse connector edge is routed at `y + CARD_HEIGHT / 2 = y + 46px`. Because the micro-pill only spans `y` to `y + 28px`, the spouse line floats $18\text{px}$ below both spouse pills in empty air.
  2. The outgoing vertical parent stem departs at `stemSourceY = y + CARD_HEIGHT = y + 92px`. The micro-pill ends at `y + 28px`, leaving a $64\text{px}$ blank gap between the bottom of the parent card and the origin of the distribution bus.
  Incoming parent-to-child edges arrive at `childY = y_child`, hitting the top border of the child pill, so outgoing and incoming lines appear completely disjointed at bird's-eye zoom.
- **Evidence:** `frontend/src/components/tree/PersonNodeCard.tsx:230-272`
- **Resolution:**
  - `PersonNodeCard` anchors its outer slot container to full `CARD_HEIGHT` (`minHeight: 92px`) and centers the 28px micro-pill vertically at `y + 32px`. The horizontal spouse connector edge at `y + 46px` aligns with the vertical midpoint (`32 + 14 = 46px`).
  - Top connection stub (`top-0 left-1/2 w-0.5 h-8`) bridges incoming parent stems from `top: 0` to the top of the pill (`top: 32px`).
  - Bottom connection stub (`bottom-0 left-1/2 w-0.5 h-8`) bridges outgoing child stems from the bottom of the pill (`top: 60px`) to `bottom: 0` (`top: 92px`).
- **Tags:** `layout`, `hierarchy`, `consistency`

---

### UX-049 [fixed] Semantic exclusion and scaling illegibility of component headers in ARIA tree (WCAG 1.3.1)
- **Screen / viewport / theme / lang:** `/tree?mode=all` across all viewports and themes.
- **Problem:**
  1. **ARIA structure failure (WCAG 1.3.1):** Visual header badges ("Main Family Branch", "Branch 2", "Isolated Records") rendered as plain `<div>`s directly inside `<div role="tree">`, missing semantic role/level.
  2. **Zoom illegibility:** Component headers shrank down to $\approx 1.8\text{px} - 2.4\text{px}$ at full overview zoom ($0.15x - 0.2x$), making branch names impossible to read.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:752-793`
- **Resolution:**
  - Component header badges now have `role="heading"` and `aria-level={3}` with descriptive `aria-label={`${title} (${comp.nodeCount})`}`.
  - Inverse scale clamp (`badgeScale = Math.max(1, Math.min(2.5, 0.6 / zoom))`) with `transformOrigin: 'left center'` maintains badge legibility without covering adjacent nodes across bird's-eye zoom factors.
- **Tags:** `a11y`, `hierarchy`, `space`

---

### UX-050 [fixed] Shelf grid start X coordinate clamped to 0, misaligning isolated records when primary branch extends left
- **Screen / viewport / theme / lang:** `/tree?mode=all` on archives where the focal person has ancestors or collaterals placed at negative coordinates.
- **Problem:** Starting X coordinate of the isolated records shelf was computed as `const shelfStartX = Math.max(0, minAllX);`, which artificially pushed the shelf to the right when branches expanded into negative X space.
- **Evidence:** `frontend/src/lib/tree-layouts/all-nodes-layout.ts:705-716`
- **Resolution:**
  - Changed `shelfStartX` to `minAllX` directly. Shelf column 0 aligns flush with the visual left margin of all tree components.
- **Tags:** `layout`, `consistency`

---

### UX-051 [fixed] LOD 2 micro-pill renders full name instead of concise surname/initials
- **Screen / viewport / theme / lang:** `/tree` at `zoom < 0.25` in English and Russian.
- **Problem:** LOD 2 rendered `{displayName}` (full tripartite name), truncating prematurely into an uninformative given name snippet (e.g. "Alexander Mikh...") hiding the surname.
- **Evidence:** `frontend/src/components/tree/PersonNodeCard.tsx:39-104, 245-247, 315-318`
- **Resolution:**
  - Implemented `formatConciseSurnameInitials`: formats names as `Surname GivenInitial. PatronymicInitial.` (e.g. "Левшиц А. М.", "Smith M. A.").
  - Retains single initial privacy masking for living persons under `public_anonymous` tier ("S.", "L."). Full name remains available on the container's accessible `aria-label`.
- **Tags:** `hierarchy`, `copy`

---

### UX-052 [fixed] Missing Home person anchor badge in LOD 2 micro-pill
- **Screen / viewport / theme / lang:** `/tree` at `zoom < 0.25` across all viewports.
- **Problem:** In LOD 2, the focal/home person indicator was omitted, making it hard to locate the user's root person in large trees.
- **Evidence:** `frontend/src/components/tree/PersonNodeCard.tsx:304-306, 320-326`
- **Resolution:**
  - Added brand `Home` icon (`data-testid="lod2-home-icon"`) with localized `aria-label={t('tree.homePersonShort')}` and subtle `border-brand/70` highlight on the micro-pill.
- **Tags:** `discoverability`, `hierarchy`

---

### UX-053 [fixed] Singular English pluralization for All Nodes count badge (`1 individuals`)
- **Screen / viewport / theme / lang:** `/tree?mode=all` in English locale (`en.json`).
- **Problem:** `en.json` string `"allNodesCount": "{count} individuals"` rendered `"1 individuals"` in single-person archives.
- **Evidence:** `frontend/src/i18n/en.json:651`, `frontend/src/i18n/ru.json:651`
- **Resolution:**
  - Adopted neutral label `"Individuals: {count}"` in `en.json`, matching Russian `"Персон: {count}"`.
- **Tags:** `copy`

---

### UX-054 [suggestion] SegmentedControl horizontal overflow on mobile viewports (< 480px) lacks scroll affordance
- **Screen / viewport / theme / lang:** `/tree` on mobile screens (390px, 360px viewport).
- **Problem:** With 5 projection mode options ("Hourglass", "Pedigree", "Descendants", "Fan Chart", "All Nodes"), the SegmentedControl requires $> 440\text{px}$ of horizontal width in English and $> 490\text{px}$ in Russian. On 390px mobile screens, the control scrolls horizontally (`overflow-x-auto`) without any scrollbar or gradient edge fade. Users on mobile devices may not discover the "All Nodes" option because it is clipped off the right edge of the screen without visual indication that more options exist.
- **Evidence:** `frontend/src/components/tree/TreeToolbar.tsx:71-74`.
- **Proposed fix:** Add a subtle gradient mask/fade on the scroll container's trailing edge or condense option labels on mobile screens $< 480\text{px}$ (e.g. icon-only with accessible `aria-label` or condensed text).
- **Acceptance criteria:**
  - Mobile users receive a visual cue (fade or scroll indicator) indicating scrollable mode options, or all options fit comfortably on screen.
- **Tags:** `discoverability`, `space`

---

## Positive Notes
- **Dynamic Toolbar Adaptability:** Seamless suppression of the `generations` depth dropdown and Wide/Compact toggle when entering `mode === 'all'`, replaced cleanly by the `allNodesCount` badge.
- **Spatial Separation:** Clean $\ge 300\text{px}$ separation between disconnected family branches, completely avoiding overlapping branches or tangled spaghetti connectors.
- **Isolated Records Shelf:** Well-structured 4-column shelf grid for isolated singletons, ensuring 100% of archive records are accessible without scattering random unattached nodes across the canvas.
- **Privacy Shielding:** Strict, consistent privacy masking of living relatives in public guest mode across all components (primary branch, secondary branches, and isolated records) with zero information leakage.
- **Performance Architecture:** Frustum culling in `TreeCanvas.tsx` skipping off-screen cards to ensure high frame rates during pan/zoom in large archives.
- **i18n & Design Token Conformance:** 100% bilingual parity between EN and RU with zero icon glyphs or emojis baked into translation strings, and design-system components used throughout.
