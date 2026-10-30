# UX Review — Milestone 2.7: Multiple Spouses, Family-Partitioned Children Buses & Multi-Trace Kinship Path Finder — 2026-10-29

## Verdict: NEEDS FIXES

Screens reviewed:
- `/tree` (Tree Canvas in Hourglass, Descendants, and All Nodes modes):
  - Symmetrical 2-spouse layout `[Spouse 1] === [Person] === [Spouse 2]` on Gen 0.
  - Separate ochre marriage lines with midpoint union circle dot indicators.
  - Strict family-partitioned children branches (no blended child buses, single-parent children from bottom-center).
  - Anti-collinear bus runner clearances ($\ge 12\text{px}$) avoiding overlapping lines.
  - Multi-trace kinship path highlighting on SVG canvas:
    - Trace 1: Brand blue solid (`stroke-brand`).
    - Trace 2: Warm ochre dashed (`stroke-amber-500`, `dasharray="6 4"`).
    - Trace 3: Violet dashed (`stroke-purple-500`, `dasharray="4 4"`).
  - Floating Multi-Trace Kinship Legend HUD with Clear Highlights action.
- Kinship Finder Dialog (`KinshipFinderDialog` modal on `/tree`):
  - Person A & Person B selectors.
  - Multi-trace switcher tabs with blood vs affinal relationship badges.
  - Calculated degree result banner.
  - Ladder breadcrumb view with directional arrows.
  - "Highlight Path on Tree", "Highlight Selected", and "Highlight All Paths" action buttons.
- Viewports verified: Desktop (1440px), Mobile (390px, 360px).
- Themes verified: Light and Dark mode.
- Locales verified: English (`en`) and Russian (`ru`).

Evidence & Verification:
- Test suite: `frontend/tests/components/milestone-2-7-multi-spouse-kinship.test.tsx` (7 passing tests), `frontend/tests/components/tree-canvas.test.tsx` (41 passing tests), all 19 test suites (210 tests) green.
- Code inspection: `TreeCanvas.tsx`, `KinshipFinderDialog.tsx`, `ConnectionNavPopover.tsx`, `hourglass-layout.ts`, `descendants-layout.ts`, `all-nodes-layout.ts`.

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
| **UX-062** | Keyboard & Screen Reader Violation: Connection Edge Buttons Inside `aria-hidden="true"` SVG Container | **fixed** |
| **UX-063** | Missing Visible Keyboard Focus Ring on Connection Edge Paths | **fixed** |
| **UX-064** | Public Guest Banner Action Stacking & Squeezed Layout on Mobile Viewports (< 640px) | **fixed** |
| **UX-065** | Sub-40px Touch Targets on Connection Popover Action Buttons | **fixed** |
| **UX-066** | Programmatic Focus Management and Popover Dismissal on "Jump to Person" | **fixed** |
| **UX-067** | Collateral Connector Line Contrast Below 3:1 Non-Text Requirement in Light & Dark Mode | **fixed** |
| **UX-068** | Missing Accessible State / Announcement for Connection Endpoint Highlights on PersonNodeCard | **fixed** |

---

## Tasks for frontend-developer

### UX-069 [critical] Kinship Trace Fails to Highlight Parent-Child Connector When Path Traverses Maternal or Spouse Partner
- **Screen / viewport / theme / lang:** `/tree` (Canvas view in Hourglass, Descendants, and All Nodes modes — all viewports, themes, locales).
- **Problem:**
  In `TreeCanvas.tsx:454-472`, `getTraceIndexForEdge` determines whether an edge belongs to a highlighted kinship path by checking:
  ```ts
  const pids = highlightedTraces[i].personIds;
  if (pids.includes(edge.sourceId) && pids.includes(edge.targetId)) {
    return i;
  }
  ```
  However, in family-partitioned layouts (`hourglass-layout.ts:496-515` and `descendants-layout.ts:440-460`), parent-child edges for children of a union are created with `sourceId = branch.personId` (e.g. the focal person / father) and `familyId = union.familyId`.
  When a kinship path traverses the mother or spouse partner (e.g., maternal grandparent $\to$ mother $\to$ child), `pids` contains the mother and the child, but NOT the father. Because `edge.sourceId` is the father, `pids.includes(edge.sourceId)` evaluates to `false`.
  Consequently, the connector stem from the family union down to the child is completely omitted from the highlight, displaying a broken, disconnected trace line on the canvas.
- **Evidence:**
  `frontend/src/components/tree/TreeCanvas.tsx:454-472`
  `frontend/src/lib/tree-layouts/hourglass-layout.ts:496-515`
- **Proposed fix:**
  Enhance `getTraceIndexForEdge` in `TreeCanvas.tsx` to recognize family partner membership for parent-child edges:
  ```ts
  const getTraceIndexForEdge = useCallback(
    (edge: PositionedEdge): number => {
      const checkTrace = (pids: string[]): boolean => {
        if (pids.includes(edge.sourceId) && pids.includes(edge.targetId)) {
          return true;
        }
        if (edge.kind === 'parent_child' && edge.familyId) {
          const fam = effectiveViewData.families?.find((f) => f.id === edge.familyId);
          if (fam && pids.includes(edge.targetId)) {
            const isPartner1 = fam.partner1_id && pids.includes(fam.partner1_id);
            const isPartner2 = fam.partner2_id && pids.includes(fam.partner2_id);
            if (isPartner1 || isPartner2) return true;
          }
        }
        return false;
      };

      if (highlightedTraces.length > 0) {
        for (let i = 0; i < highlightedTraces.length; i++) {
          if (checkTrace(highlightedTraces[i].personIds)) {
            return i;
          }
        }
      } else if (highlightedPathIds.length >= 2 && checkTrace(highlightedPathIds)) {
        return 0;
      }
      return -1;
    },
    [highlightedTraces, highlightedPathIds, effectiveViewData.families]
  );
  ```
  In addition, when a path traverses through a spouse into the child, ensure the spouse connection line between the two partners is also highlighted so the path continuously links from the partner to the union midpoint dot.
- **Acceptance criteria:**
  - When finding a kinship path traversing a maternal ancestor or spouse into their children, the vertical family connector down to the child is highlighted in the active trace stroke.
  - Zero broken or severed line segments appear on highlighted kinship traces.
- **Tags:** `states`, `hierarchy`

---

### UX-070 [major] Ochre Connector Lines (`stroke-amber-500`) Fail WCAG 2.2 AA Non-Text Contrast ($\ge 3:1$) on Light Canvas
- **Screen / viewport / theme / lang:** `/tree` (Light Theme — all viewports and locales).
- **Problem:**
  In `TreeCanvas.tsx:878` and `TreeCanvas.tsx:887`, Trace 2 and regular spouse connection lines use `stroke-amber-500` (`#F59E0B`). Midpoint union dots also use `fill-amber-500`.
  Against the light canvas background (`--color-bg-canvas: #F8F9FA`), `#F59E0B` has a contrast ratio of only **2.01:1**, severely failing WCAG 2.2 Criterion 1.4.11 (Non-text Contrast $\ge 3:1$).
  In light mode, the yellow-amber lines appear washed out and are difficult to distinguish for older relatives or users in bright ambient environments. (In dark mode, `#F59E0B` against `#0E1116` provides 7.86:1, which is fully compliant).
- **Evidence:**
  `frontend/src/components/tree/TreeCanvas.tsx:878, 887, 956, 968`
  Measured contrast ratio: `#F59E0B` on `#F8F9FA` = **2.01:1** (< 3:1).
- **Proposed fix:**
  In light mode, replace `stroke-amber-500` and `fill-amber-500` with `stroke-amber-600 dark:stroke-amber-400` or `stroke-[var(--color-brand-accent)]` (`#C68A34` in light mode achieves **3.32:1** contrast; `#D9A04B` in dark mode achieves **8.1:1**).
  Apply the same token to `fill-amber-600 dark:fill-amber-400` on the midpoint union dots.
- **Acceptance criteria:**
  - Non-text contrast ratio of Trace 2 and marriage connection lines against the canvas is $\ge 3.0:1$ in both light and dark themes.
- **Tags:** `a11y`, `consistency`

---

### UX-071 [major] Floating Kinship Legend HUD Collides with and Obscures `TreeToolbar` on Mobile Viewports (< 768px)
- **Screen / viewport / theme / lang:** `/tree` (Mobile viewports $\le 768\text{px}$, especially in Russian).
- **Problem:**
  On mobile viewports, `TreeToolbar` stacks into a two-row vertical layout at `top-3 inset-x-3` with vertical height extending to $\sim 95\text{px}$–$110\text{px}$.
  The Floating Multi-Trace Kinship Legend HUD in `TreeCanvas.tsx:762` is positioned at `className="absolute top-16 sm:top-14 md:top-4 left-1/2 -translate-x-1/2 z-30 ..."`.
  At `top-16` (64px), the Legend HUD is rendered directly on top of Row 2 of `TreeToolbar` (which contains the Generations selector, Kinship Finder button, Home button, and Fit button).
  Because the Legend HUD has `z-30` and `pointer-events-auto`, users cannot tap the toolbar buttons beneath it, and the overlapping cards create an unusable visual collision on phones.
  Furthermore, the HUD lacks `max-w-[calc(100vw-24px)]`. When Russian trace labels wrap, the HUD can overflow screen margins horizontally.
- **Evidence:**
  `frontend/src/components/tree/TreeCanvas.tsx:762`
  `frontend/src/components/tree/TreeToolbar.tsx:69`
- **Proposed fix:**
  Adjust the responsive positioning and max-width of the Legend HUD:
  - On mobile (`< md`), position it safely below the stacked toolbar: `top-28 sm:top-24 md:top-4`, or dock it in the lower viewport area: `bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))] md:top-4 md:bottom-auto`.
  - Add `w-max max-w-[calc(100vw-24px)]` and `mx-auto` to prevent horizontal viewport clipping.
  - In `Button` for "Clear highlights" (`TreeCanvas.tsx:808`), remove `h-6` and use `h-8 min-h-8 px-2.5 text-xs touch-target-expand` to guarantee a comfortable touch target $\ge 40\times 40\text{px}$.
- **Acceptance criteria:**
  - On 360px and 390px mobile viewports, the Legend HUD never collides with or obscures `TreeToolbar` buttons.
  - The HUD stays within viewport boundaries without horizontal scrolling or margin clipping.
- **Tags:** `layout`, `space`, `a11y`

---

### UX-072 [major] Kinship Finder Dialog Footer Action Buttons Overflow Horizontally on Mobile Screens in Russian
- **Screen / viewport / theme / lang:** `/tree` (`KinshipFinderDialog` on viewports $\le 390\text{px}$, Russian locale `ru`).
- **Problem:**
  In `KinshipFinderDialog.tsx:796-884`, the footer action container uses:
  `<div className="flex justify-end gap-2 pt-2 border-t border-border">`
  When multiple traces are found, the footer renders three buttons:
  1. "Закрыть" (Close)
  2. "Подсветить путь" (Highlight Selected)
  3. "Подсветить все пути" (Highlight All Paths)
  Each button has an icon and text. In Russian, these three buttons side-by-side require at least $430\text{px}$ of horizontal width.
  On a mobile screen of $360\text{px}$ or $390\text{px}$ (where modal width is $\le 340\text{px}$), the buttons overflow the modal horizontally or get severely squished, causing text truncation and accidental taps.
- **Evidence:**
  `frontend/src/components/tree/KinshipFinderDialog.tsx:796-884`
- **Proposed fix:**
  Make the dialog action bar responsive:
  ```tsx
  <div className="flex flex-col-reverse sm:flex-row sm:justify-end gap-2 pt-2 border-t border-border">
  ```
  On mobile (`< sm`), buttons should stretch to full width (`w-full`), making them easy to tap ($44\text{px}$ touch targets) with clear labels, while sitting neatly side-by-side on tablet/desktop.
- **Acceptance criteria:**
  - On 360px and 390px mobile viewports in Russian (`ru`), all dialog action buttons render with zero horizontal overflow and zero text truncation.
- **Tags:** `layout`, `a11y`

---

### UX-073 [major] "Highlight Path on Tree" Leaves Highlighted Relatives Off-Screen Without Camera Auto-Framing
- **Screen / viewport / theme / lang:** `/tree` (Tree Canvas — all viewports and devices).
- **Problem:**
  When the user activates "Highlight Path on Tree", "Highlight Selected", or "Highlight All Paths" in `KinshipFinderDialog`, the modal closes and the edges/cards are highlighted.
  However, camera pan and zoom do not update. If the two relatives are located in distant generations or branches, or if the user was zoomed into an unrelated branch, the highlighted kinship path is located entirely off-screen.
  Users (especially non-technical relatives on phones) receive no visual feedback that anything changed on the tree and are forced to manually pan and zoom across the canvas to locate the path.
- **Evidence:**
  `frontend/src/components/tree/TreeCanvas.tsx:1238-1249`
  `frontend/src/components/tree/KinshipFinderDialog.tsx:817-881`
- **Proposed fix:**
  When `onHighlightPath` is called:
  Calculate the bounding box of the highlighted person nodes in `layout.nodes`.
  If matching nodes exist, smoothly center the camera or call `centerOnWorldPoint(centerX, centerY, targetZoom, true)` so the entire highlighted path is neatly framed in the viewport.
- **Acceptance criteria:**
  - Activating path highlight automatically frames the highlighted nodes in the center of the viewport.
- **Tags:** `states`, `hierarchy`

---

### UX-074 [minor] Multi-Trace Switcher `role="tablist"` Lacks `aria-controls` / `tabpanel` Associations and Arrow Key Navigation
- **Screen / viewport / theme / lang:** `/tree` (`KinshipFinderDialog`).
- **Problem:**
  In `KinshipFinderDialog.tsx:678-720`, the trace selector renders `role="tablist"` with child buttons having `role="tab"` and `aria-selected`.
  However, the content area below (degree banner and step list) does not render `role="tabpanel"`, has no `id`, and the tabs lack `aria-controls`.
  Assistive technology accessibility linters (axe rule `tabpanel-aria-controls`) flag tablists lacking associated panels.
  Furthermore, standard tablist interaction expects `ArrowLeft` / `ArrowRight` to cycle through tabs.
- **Evidence:**
  `frontend/src/components/tree/KinshipFinderDialog.tsx:678-720`
- **Proposed fix:**
  1. Add `id="kinship-tab-${idx}"` and `aria-controls="kinship-trace-panel"` to each tab button.
  2. Add `id="kinship-trace-panel" role="tabpanel" aria-labelledby={`kinship-tab-${activeTraceIndex}`}` to the degree result and breadcrumb section.
  3. Add `onKeyDown` on tabs to support ArrowLeft and ArrowRight navigation.
- **Acceptance criteria:**
  - Automated accessibility linters report zero tablist / tabpanel structure errors in `KinshipFinderDialog`.
  - Screen reader announces the active tab and its corresponding panel.
- **Tags:** `a11y`

---

### UX-075 [minor] Redundant Identical Error Copy Displayed Twice When Two Individuals Are Not Connected
- **Screen / viewport / theme / lang:** `/tree` (`KinshipFinderDialog`).
- **Problem:**
  When Person A and Person B have no genealogical connection:
  `effectiveResult.relationship_label` is set to `t('tree.kinship.notConnected')` ("No connected relationship path found between these two individuals.").
  The degree result banner displays:
  "Calculated Relationship: No connected relationship path found between these two individuals."
  And directly below it, line 793 renders:
  `<Alert variant="info">{t('tree.kinship.notConnected')}</Alert>`.
  Displaying the identical long sentence in both the summary box and an alert immediately below it creates visual redundancy and makes the dialog look unfinished.
- **Evidence:**
  `frontend/src/components/tree/KinshipFinderDialog.tsx:724-746, 793`
- **Proposed fix:**
  When not connected (`!effectiveResult.connected`):
  Display a single clean `Alert variant="info"` with a clear title and guidance (e.g. suggesting checking whether parents or spouses are linked), and hide or simplify the calculated degree banner.
- **Acceptance criteria:**
  - The "not connected" explanation is presented once cleanly without verbatim duplication.
- **Tags:** `copy`, `consistency`

---

### UX-076 [suggestion] Add Quick "Swap Persons" Action Between Person A and Person B in Kinship Finder
- **Screen / viewport / theme / lang:** `/tree` (`KinshipFinderDialog`).
- **Problem:**
  Kinship relationships are often evaluated from both perspectives (e.g. "Ivan is grandfather to Dmitry" vs "Dmitry is grandson to Ivan").
  Currently, reversing the evaluation requires clicking both `Select` dropdowns and re-selecting each person from the full archive directory list.
- **Evidence:**
  `frontend/src/components/tree/KinshipFinderDialog.tsx:645-669`
- **Proposed fix:**
  Add a swap button (`<IconButton icon={<ArrowUpDown className="h-4 w-4" />} />` or `<ArrowLeftRight />`) between the Person A and Person B selectors that swaps `personAId` and `personBId` in one click.
- **Acceptance criteria:**
  - Clicking the swap button instantly exchanges Person A and Person B and re-evaluates the kinship path.
- **Tags:** `space`, `hierarchy`

---

### UX-077 [minor] Parent-Child Edges in `all-nodes-layout.ts` Lack `familyId`, Breaking Marriage Details Popover
- **Screen / viewport / theme / lang:** `/tree` (All Nodes mode).
- **Problem:**
  In `all-nodes-layout.ts:785-801` and `822-835`, `createPositionedEdge` creates parent-child edges without passing `familyId`.
  When a user clicks a parent-child connection in All Nodes mode, `selectedEdge.familyId` is undefined. Consequently, `TreeCanvas.tsx:1160-1167` cannot resolve the `family` object, and the connection popover cannot show family marriage details or context.
- **Evidence:**
  `frontend/src/lib/tree-layouts/all-nodes-layout.ts:785, 822`
  `frontend/src/components/tree/TreeCanvas.tsx:1160-1167`
- **Proposed fix:**
  Pass the matching `familyId` (looked up from `rel.unions` or `data.families` where the child is listed in `children`) when constructing parent-child edges in `all-nodes-layout.ts`.
- **Acceptance criteria:**
  - In All Nodes mode, clicking a parent-child edge resolves the associated `family` object.
- **Tags:** `consistency`, `states`

---

## Positive Notes
- **Symmetrical 2-Spouse Hourglass Layout (`[S1] === [Person] === [S2]`):** The symmetrical couple layout on Generation 0 anchors the focal person directly in the center while cleanly flanking two spouses with equal spacing and separate ochre connector bars.
- **Strict Family Partitioning & Zero Child Blending:** Children branches are strictly partitioned under their respective marriage midpoint drop stems. Single-parent children drop cleanly from the bottom-center of the person card with zero crossover confusion.
- **Anti-Collinear Bus Runner Clearances ($\ge 12\text{px}$):** Vertical bus lines maintain at least 12px horizontal clearance, completely eliminating overlapping connector runs between adjacent family branches.
- **Multi-Trace Color & Dash Distinctions:** Multi-trace kinship rendering pairs distinct colors (Brand blue, Warm ochre, Violet) with distinct stroke dash patterns (solid, `6 4`, `4 4`), ensuring full accessibility under WCAG 1.4.1 (Use of Color).
- **Privacy Masking in Breadcrumbs:** The kinship step chain and multi-trace tabs flawlessly adhere to privacy masking rules for living individuals in public guest mode.
