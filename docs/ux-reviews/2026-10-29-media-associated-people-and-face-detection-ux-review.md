# UX Review — Milestone 2.8: Associated People in Media Panel & Automatic Face Detection — 2026-10-29

## Verdict: PASS (Re-Review of UX-078 through UX-086: All 9 Tasks Verified & Fixed)

Screens reviewed:
- Media Lightbox Studio (`/media` and media lightbox across `/`, `/person/:id`):
  - Dual-pane layout: 7-col Face Tagging Canvas & 5-col Media Metadata / People Panel.
  - "PEOPLE IN THIS MEDIA ({count})" partitioned into Tagged Faces and Associated Relatives.
  - "Profile photo" vs "Associated" badges, "+ Tag Face" quick arming action, and "Unlink" action.
  - `LinkPersonDialog` modal dialog to associate relatives without drawing face bounding boxes.
  - Automatic Face Detection via Shape Detection API and canvas heuristic fallback.
  - Pulsing candidate bounding boxes and 1-click candidate adoption into draft or armed target person.
  - Move (`Move`) and Resize (`Maximize2`) bounding box handles in tagging mode.
  - Target person armed banner (`targetPersonBanner`) and cancellation flow.
- Viewports verified: Desktop (1440px), Mobile (390px, 360px).
- Themes verified: Light and Dark mode.
- Locales verified: English (`en`) and Russian (`ru`).

Evidence & Verification Artifacts:
- Browser viewport screenshots:
  - `.playwright-mcp/page-2026-10-29T09-45-31-335Z.png` (Desktop 1440px Dark Studio)
  - `.playwright-mcp/page-2026-10-29T09-45-47-048Z.png` (Desktop 1440px Dark Tagging Mode with handles)
  - `.playwright-mcp/page-2026-10-29T09-46-06-135Z.png` (Desktop 1440px Dark `LinkPersonDialog` modal)
  - `.playwright-mcp/page-2026-10-29T09-46-34-362Z.png` (Desktop 1440px Dark Associated Relatives section)
  - `.playwright-mcp/page-2026-10-29T09-46-51-166Z.png` (Desktop 1440px Dark Armed Target Person banner)
  - `.playwright-mcp/page-2026-10-29T09-47-04-283Z.png` (Mobile 390px Dark Studio Canvas & Toolbar)
  - `.playwright-mcp/page-2026-10-29T09-48-03-685Z.png` (Mobile 390px Dark Badge Overlap Collision)
  - `.playwright-mcp/page-2026-10-29T09-49-44-545Z.png` (Desktop 1440px Light Studio EN)
- Unit & E2E Test suite: `frontend/tests/components/milestone-2-8-associated-people-face-detection.test.tsx` (11 tests passing), all 20 frontend suites (227 tests) green.

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
| **UX-039** | Low non-text contrast (< 3:1) for connector edges | **fixed** |
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
| **UX-069** | Kinship Trace Fails to Highlight Parent-Child Connector When Path Traverses Maternal or Spouse Partner | **fixed** |
| **UX-070** | Ochre Connector Lines (`stroke-amber-500`) Fail WCAG 2.2 AA Non-Text Contrast ($\ge 3:1$) on Light Canvas | **fixed** |
| **UX-071** | Floating Kinship Legend HUD Collides with and Obscures `TreeToolbar` on Mobile Viewports (< 768px) | **fixed** |
| **UX-072** | Kinship Finder Dialog Footer Action Buttons Overflow Horizontally on Mobile Screens in Russian | **fixed** |
| **UX-073** | "Highlight Path on Tree" Leaves Highlighted Relatives Off-Screen Without Camera Auto-Framing | **fixed** |
| **UX-074** | Multi-Trace Switcher `role="tablist"` Lacks `aria-controls` / `tabpanel` Associations and Arrow Key Navigation | **fixed** |
| **UX-075** | Redundant Identical Error Copy Displayed Twice When Two Individuals Are Not Connected | **fixed** |
| **UX-076** | Add Quick "Swap Persons" Action Between Person A and Person B in Kinship Finder | suggestion (open) |
| **UX-077** | Parent-Child Edges in `all-nodes-layout.ts` Lack `familyId`, Breaking Marriage Details Popover | **fixed** |
| **UX-078** | Unconfirmed Destructive Action on Media Asset Deletion (Data Loss Risk) | **fixed** |
| **UX-079** | Overlapping Collision of Person Name and Status Badges on Mobile Viewports | **fixed** |
| **UX-080** | Keyboard & Screen Reader Inaccessible Auto-Detected Face Candidate Rectangles | **fixed** |
| **UX-081** | Target Person Armed Banner Truncates and Hides Relative's Name on Mobile Viewports | **fixed** |
| **UX-082** | Sub-40px Touch Targets on Studio Navigation, Move/Resize Handles, and Panel Buttons | **fixed** |
| **UX-083** | Duplicate "Auto-Detect Faces" Buttons Rendered Simultaneously on Canvas Stage | **fixed** |
| **UX-084** | Auto-Detected Face Candidates Lack Dismiss / Reject Affordance for False Detections | **fixed** |
| **UX-085** | Low Contrast on Sky-Blue Candidate Rectangles (`border-sky-400`) in Light Theme | **fixed** |
| **UX-086** | WAI-ARIA Structure Violation: Nested `<button>` Inside `role="option"` and Missing Arrow Key Navigation | **fixed** |

---

## Tasks for frontend-developer

### UX-078 [critical] Unconfirmed Destructive Action on Media Asset Deletion (Data Loss Risk)
- **Screen / viewport / theme / lang:** Media Lightbox Studio (`MediaMetadataPanel.tsx:244-253` & `MediaLightboxStudio.tsx:480-492`) — all viewports, themes, locales.
- **Problem:** When an editor or admin clicks the "Delete" button (`Удалить` / `Delete`) in `MediaMetadataPanel`, `handleDeleteAsset` immediately calls `deleteMediaAsset(currentAsset.id)` and permanently deletes the photo, its archival metadata, all associated links, and all face tags without any confirmation prompt. On mobile touchscreens, the red "Delete" button sits directly adjacent to "Save Details" (`Сохранить сведения`), making accidental taps catastrophic.
- **WCAG / Guideline:** WCAG 2.2 Criterion 3.3.4 (Error Prevention - Legal, Financial, Data, Level AA). System rule: "destructive actions visually distinct and confirmed".
- **Evidence:**
  `frontend/src/components/media/MediaMetadataPanel.tsx:244-253`
  `frontend/src/components/media/MediaLightboxStudio.tsx:480-492`
- **Proposed fix:**
  Wrap asset deletion in the global `useConfirm()` dialog before invoking `onDeleteAsset()`:
  ```tsx
  const { confirm } = useConfirm();
  const handleDeleteClick = async () => {
    const ok = await confirm({
      title: t('mediaArchive.deleteConfirmTitle'),
      message: t('mediaArchive.deleteConfirmMessage', { title: asset.title }),
      confirmText: t('common.delete'),
      cancelText: t('common.cancel'),
      variant: 'danger',
    });
    if (ok) {
      void onDeleteAsset();
    }
  };
  ```
- **Acceptance criteria:**
  - Clicking "Delete" opens a modal confirmation dialog explaining that the media item and all face tags will be removed.
  - Deletion only proceeds if explicitly confirmed by the user.
- **Tags:** `states`, `a11y`

---

### UX-079 [major] Overlapping Collision of Person Name and "Primary Avatar" / "Associated" Badges on Mobile Viewports
- **Screen / viewport / theme / lang:** `MediaMetadataPanel.tsx:370-403, 476-513` — Mobile viewports ($\le 390\text{px}$), Russian locale `ru`.
- **Problem:**
  In `MediaMetadataPanel.tsx:370-403`, the tagged person card header uses `flex items-center justify-between gap-2`.
  On mobile viewports (390px), the "Primary Avatar" badge (`Основной портрет`) occupies > 130px. When a relative's name wraps (e.g. Russian patronymic "Александрович"), the `Badge` does not shrink or wrap; instead, it is rendered directly on top of the second line of the person's name ("Александрович"), and the vital dates string below gets prematurely cut off into `род. 1912 · †...`.
  Similarly, in the associated relatives section (`MediaMetadataPanel.tsx:476-513`), the `<ExternalLink className="h-3 w-3" />` icon at the right edge of the name button physically collides with the "Associated" (`В материале`) badge.
- **Evidence:** Browser screenshot `.playwright-mcp/page-2026-10-29T09-48-03-685Z.png`.
- **Proposed fix:**
  1. Add `flex-wrap sm:flex-nowrap` to the card header container with `gap-y-1.5`.
  2. Allow `Badge` to have `shrink-0 self-start`.
  3. Ensure the person name button has `whitespace-normal break-words` so multi-line names wrap cleanly beneath the avatar without bleeding into badges.
  4. In the associated relatives section, move the `Badge` into a clean aligned flex column or place it beside the vitals so it never collides with external link icons.
- **Acceptance criteria:**
  - On 360px and 390px mobile screens in Russian (`ru`), person names never overlap with status badges.
  - Vital dates (birth and death years) are fully legible without premature single-character truncation.
- **Tags:** `layout`, `space`

---

### UX-080 [major] Keyboard & Screen Reader Inaccessible Auto-Detected Face Candidate Rectangles
- **Screen / viewport / theme / lang:** `FaceTagCanvas.tsx:370-393` — Canvas view in Tagging Mode, all viewports and themes.
- **Problem:**
  Candidate face boxes produced by automatic detection are rendered as plain `<div>` elements with `onClick={(e) => handleAdoptCandidate(candBox, idx)}`.
  They lack `role="button"`, have no `tabIndex={0}`, have no keyboard event listeners (`onKeyDown` for Enter or Space), and provide no accessible name (`aria-label`).
  A keyboard-only user or screen-reader user cannot Tab into, discover, or adopt any of the automatically detected face candidates.
- **WCAG:** WCAG 2.2 Criterion 2.1.1 (Keyboard, Level A) and Criterion 4.1.2 (Name, Role, Value, Level A).
- **Evidence:** `frontend/src/components/media/FaceTagCanvas.tsx:370-393`
  ```tsx
  <div
    key={`cand-${idx}-${candBox.x}-${candBox.y}`}
    data-face-tag-interactive="true"
    onClick={(e) => {
      e.stopPropagation();
      handleAdoptCandidate(candBox, idx);
    }}
    style={{ ... }}
    className="absolute rounded-lg border-2 border-dashed border-sky-400 bg-sky-400/15 hover:bg-sky-400/30 transition-all cursor-pointer group z-20 flex flex-col justify-between p-1 shadow-md"
  >
  ```
- **Proposed fix:**
  Make candidate rectangles fully accessible keyboard interactive elements:
  1. Add `role="button"` and `tabIndex={0}`.
  2. Add `aria-label={t('mediaArchive.suggestedFaceAria', { index: idx + 1 })}`.
  3. Add `onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleAdoptCandidate(candBox, idx); } }}`.
  4. Add visible focus ring styles: `focus-visible:ring-2 focus-visible:ring-sky-500 focus-visible:outline-none`.
- **Acceptance criteria:**
  - Tabbing through the canvas in tagging mode moves focus to each suggested candidate box with a high-contrast focus ring.
  - Pressing `Enter` or `Space` on a focused candidate box adopts it into a draft box or links the armed target person.
- **Tags:** `a11y`

---

### UX-081 [major] Target Person Armed Banner Truncates and Hides Relative's Name on Mobile Viewports
- **Screen / viewport / theme / lang:** `FaceTagCanvas.tsx:260-280` — Mobile viewports ($\le 390\text{px}$).
- **Problem:**
  When a user arms an associated relative for 1-click face tagging (via "+ Tag Face" / "+ Отметить лицо"), a banner appears at the top of the canvas:
  `"Выберите найденную область лица или нарисуйте рамку для: {name}"`.
  The text span has `className="truncate"`. On a 390px screen, the text cuts off after 26 characters: `"Выберите найденную облас..."`.
  Because the text truncates before the relative's name is reached, the user has NO visual confirmation of WHICH relative is currently armed. Tapping a candidate box could accidentally assign the wrong relative.
- **Evidence:** Browser screenshot `.playwright-mcp/page-2026-10-29T09-47-04-283Z.png`.
- **Proposed fix:**
  In `FaceTagCanvas.tsx:260-280`:
  1. Make the banner layout responsive: `flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1.5`.
  2. Replace `truncate` with `break-words` or structure the banner into a concise heading with a prominent badge/pill displaying the target person's name:
     ```tsx
     <div className="flex items-center gap-1.5 min-w-0">
       <UserCheck className="h-4 w-4 text-brand shrink-0" />
       <span className="text-secondary">{t('mediaArchive.taggingForPrefix')}:</span>
       <span className="font-semibold text-primary truncate max-w-[180px]">{targetPerson.displayName}</span>
     </div>
     ```
- **Acceptance criteria:**
  - On 360px and 390px mobile screens, the armed relative's name is always clearly visible in the top canvas banner.
- **Tags:** `layout`, `hierarchy`

---

### UX-082 [major] Sub-40px Touch Targets on Studio Navigation, Move/Resize Handles, and Panel Buttons
- **Screen / viewport / theme / lang:** Media Lightbox Studio across touch devices and mobile viewports.
- **Problem:**
  Multiple interactive elements fail the project's $\ge 40\times 40\text{px}$ touch target minimum (`--size-touch-min: 44px`):
  1. **Prev / Next photo chevrons:** `MediaLightboxStudio.tsx:523, 534` specify `className="h-8 w-8 min-h-0 p-0"` ($32\times 32\text{px}$) without `touch-target-expand`.
  2. **Move & Resize handles:** `FaceTagCanvas.tsx:447, 469` specify `className="h-5 w-5"` ($20\times 20\text{px}$) `span` elements. On phones and tablets, tapping a 20px corner handle to adjust a bounding box is frustrating and leads to missed touches.
  3. **PersonFacePicker Close button:** `PersonFacePicker.tsx:106` specifies `className="h-6 w-6 min-h-0 p-0"` ($24\times 24\text{px}$) without expansion.
  4. **Metadata & People Panel Action Buttons:** "Link Person", "Tag Face", "Set as Primary Avatar", "Unlink", "Remove" all specify `h-7 min-h-0 px-2` ($28\text{px}$ height) with zero touch target padding.
- **WCAG / Tokens:** WCAG 2.2 AA Criterion 2.5.8 (Target Size - Minimum). Design system token `--size-touch-min: 44px`.
- **Evidence:**
  `frontend/src/components/media/MediaLightboxStudio.tsx:523-535`
  `frontend/src/components/media/FaceTagCanvas.tsx:447, 469`
  `frontend/src/components/media/MediaMetadataPanel.tsx:313, 323, 417, 433, 536, 552`
  `frontend/src/components/media/PersonFacePicker.tsx:106`
- **Proposed fix:**
  1. Add `touch-target-expand` to all icon-only buttons (`h-8 w-8 touch-target-expand`).
  2. For move and resize handles in `FaceTagCanvas.tsx`, expand their coarse-pointer hit areas using transparent pseudo-elements:
     ```css
     before:absolute before:-inset-2 before:content-['']
     ```
     or increase the handle size to `h-6 w-6 sm:h-5 sm:w-5` with `touch-target-expand`.
  3. Upgrade panel secondary actions from `h-7` to `h-8 min-h-8 px-2.5 text-xs touch-target-expand`.
- **Acceptance criteria:**
  - All interactive buttons and handles in `MediaLightboxStudio` provide coarse-pointer hit areas of at least $40\times 40\text{px}$.
- **Tags:** `a11y`

---

### UX-083 [minor] Duplicate "Auto-Detect Faces" Buttons Rendered Simultaneously on Canvas Stage
- **Screen / viewport / theme / lang:** `FaceTagCanvas.tsx:292-305, 600-614` — Canvas view in Tagging Mode.
- **Problem:**
  When tagging mode is active (`isTaggingMode === true`), the exact same "Auto-Detect Faces" (`Автоопределение лиц`) button with Sparkles icon and identical styling is rendered simultaneously in two locations:
  1. Inside the top tagging banner (`FaceTagCanvas.tsx:292`, `h-7 min-h-0 px-2.5`).
  2. Inside the bottom canvas toolbar (`FaceTagCanvas.tsx:600`, `h-8 min-h-0 px-2.5`).
  On small and mobile viewports, having two identical auto-detect buttons visible at the same time causes visual redundancy and squeezes out zoom controls.
- **Evidence:** Browser screenshots `.playwright-mcp/page-2026-10-29T09-45-47-048Z.png` and `.playwright-mcp/page-2026-10-29T09-47-04-283Z.png`.
- **Proposed fix:**
  When `isTaggingMode` is active, render "Auto-Detect Faces" only once in the top contextual tagging banner, and omit it from the bottom persistent toolbar (where the primary toggle "Tag Faces" / zoom controls live).
- **Acceptance criteria:**
  - "Auto-Detect Faces" is displayed exactly once in the canvas view at any given time.
- **Tags:** `consistency`, `space`

---

### UX-084 [minor] Auto-Detected Face Candidates Lack Dismiss / Reject Affordance for False Detections
- **Screen / viewport / theme / lang:** `FaceTagCanvas.tsx:370-393`
- **Problem:**
  Computer vision and heuristic edge detection inevitably produce false positives on complex archival photos (e.g. framing coats, framed paintings in the background, or fabric textures as faces).
  Currently, candidate rectangles offer no dismiss button. The only way to remove a false positive is to adopt it as a face tag and then delete the face tag, or close and reopen the photo.
- **Evidence:** `frontend/src/components/media/FaceTagCanvas.tsx:370-393`
- **Proposed fix:**
  Add a small dismiss `✕` icon button in the top-right corner of each candidate box:
  ```tsx
  <button
    type="button"
    aria-label={t('mediaArchive.dismissCandidateAria')}
    onClick={(e) => {
      e.stopPropagation();
      onDismissCandidateBox?.(idx);
    }}
    className="rounded-full bg-surface/90 text-secondary hover:text-danger p-0.5 shadow-xs"
  >
    <X className="h-3 w-3" />
  </button>
  ```
- **Acceptance criteria:**
  - Users can click/tap the dismiss button on any suggested face candidate to discard it immediately without affecting other candidates.
- **Tags:** `states`, `hierarchy`

---

### UX-085 [minor] Low Contrast on Sky-Blue Candidate Rectangles (`border-sky-400`) in Light Theme
- **Screen / viewport / theme / lang:** `FaceTagCanvas.tsx:383` — Light theme.
- **Problem:**
  Auto-detect candidate bounding boxes use `border-sky-400` (`#38bdf8`) with `bg-sky-400/15`.
  Against light/sepia archival photographs and white backdrops (`#FFFFFF`), `#38bdf8` provides a contrast ratio of only **1.92:1**, failing WCAG 2.2 Criterion 1.4.11 (Non-text Contrast $\ge 3:1$).
  In dark mode, `#38bdf8` achieves 8.4:1, but in light mode the dashed outline is washed out and difficult to discern.
- **Evidence:** `frontend/src/components/media/FaceTagCanvas.tsx:383`
- **Proposed fix:**
  Use `border-sky-600 dark:border-sky-400` (in light mode, `sky-600` `#0284c7` achieves **4.01:1** contrast against white).
- **Acceptance criteria:**
  - Non-text contrast ratio of suggested face candidate boundaries against light backgrounds is $\ge 3.0:1$.
- **Tags:** `a11y`, `consistency`

---

### UX-086 [minor] WAI-ARIA Structure Violation: Nested `<button>` Inside `role="option"` and Missing Arrow Key Navigation in Person Selection Dialogs
- **Screen / viewport / theme / lang:** `LinkPersonDialog.tsx:128-188` and `PersonFacePicker.tsx:120-170`.
- **Problem:**
  In both `LinkPersonDialog` and `PersonFacePicker`, the list of individuals is structured with a container `role="listbox"`, and each child item has `role="option" aria-selected={isSelected}` wrapping a `<Button variant="ghost">`.
  W3C WAI-ARIA 1.2 prohibits interactive child elements (such as `<button>`) inside elements with `role="option"`. This causes axe-core `nested-interactive` errors and causes screen readers to double-announce ("option ... button").
  Additionally, neither dialog supports keyboard `ArrowDown` / `ArrowUp` to cycle through options or `Enter` to select.
- **Evidence:**
  `frontend/src/components/media/LinkPersonDialog.tsx:143-150`
  `frontend/src/components/media/PersonFacePicker.tsx:134-142`
- **Proposed fix:**
  Either:
  1. Remove `role="listbox"` and `role="option"`, rendering a semantic `<ul role="list">` of `<Button>` elements.
  2. OR move `role="option"` directly onto the `<Button>` component (`<Button role="option" aria-selected={isSelected}>`) and add keyboard arrow key navigation to cycle between options.
- **Acceptance criteria:**
  - Automated accessibility linters report zero `nested-interactive` violations in `LinkPersonDialog` and `PersonFacePicker`.
  - Arrow keys navigate smoothly across selectable relatives.
- **Tags:** `a11y`

---

## Positive Notes
- **Clear Two-Tier Partitioning ("Tagged Faces" vs "Associated Relatives"):** Partitioning media participants into tagged individuals with bounding boxes and untagged depicted relatives provides an intuitive mental model for archival photos where faces cannot be pinpointed or group photos with many non-focal individuals.
- **One-Click Target Arming ("+ Tag Face"):** Arming an already associated relative with "+ Tag Face" highlights the person in brand blue and activates the canvas banner, allowing immediate 1-click association of candidate face boxes without re-opening autocomplete pickers.
- **Profile Photo Synchronization & Badges:** Distinct visual badges for "Primary Avatar" (`Основной портрет`) and "Profile photo" vs "Associated" give clear visibility into which photo represents the individual across the family tree.
- **Shape Detection API Integration with Graceful Canvas Fallback:** Implementing the browser's native `FaceDetector` API with an offscreen chromatic clustering heuristic ensures automatic face detection works offline and across diverse browser environments without heavy external dependencies.
- **Clean Escape Key Cascade:** The nested keyboard Escape hierarchy cleanly dismisses the person link modal first, then cancels target arming, and only then closes the lightbox studio.

---

## Re-Review & Verification Report (Post UX-078 - UX-086)

### Re-Review Date: 2026-10-29
### Final Verdict: PASS (All 9 Tasks Verified & Closed)

| Task ID | Severity | Verified Status | Verification Notes |
| :--- | :--- | :--- | :--- |
| **UX-078** | critical | **FIXED** | In `MediaMetadataPanel.tsx:250-260`, `onDeleteAsset` is wrapped in `confirm({ title, description, confirmLabel, cancelLabel, destructive: true })`. Localized Russian and English dialog strings prevent accidental media loss. |
| **UX-079** | major | **FIXED** | In `MediaMetadataPanel.tsx:382-417, 490-529`, header containers now use `flex flex-wrap sm:flex-nowrap items-start sm:items-center justify-between gap-2` with `break-words` on person names and `shrink-0` on badges. Zero badge collision or awkward truncation on 360px and 390px viewports. |
| **UX-080** | major | **FIXED** | In `FaceTagCanvas.tsx:390-410`, candidate rectangles feature `role="button"`, `tabIndex={0}`, localized `aria-label`, keyboard listeners for `Enter` and Space (`' '`), and a high-contrast focus ring (`focus-visible:ring-2 focus-visible:ring-sky-500`). Tooltip is also announced/visible on keyboard focus via `group-focus-visible:opacity-100`. |
| **UX-081** | major | **FIXED** | In `FaceTagCanvas.tsx:261-297`, the armed target person banner uses `flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2` with `break-words` instead of `truncate`. The target relative's full name remains clearly visible on mobile screens. |
| **UX-082** | major | **FIXED** | Touch target expansions applied across all controls: Move & Resize handles in `FaceTagCanvas.tsx:496, 518` now have `touch-target-expand before:absolute before:-inset-2 before:content-['']`; Prev/Next chevrons in `MediaLightboxStudio.tsx:523, 534` have `touch-target-expand`; picker close buttons and panel buttons upgraded to `h-8 min-h-8 px-2.5 touch-target-expand`. |
| **UX-083** | minor | **FIXED** | Redundant "Auto-Detect Faces" button removed from the bottom canvas toolbar in `FaceTagCanvas.tsx:630-650`. Exactly one auto-detect action appears contextually in the top tagging banner during tagging mode. |
| **UX-084** | minor | **FIXED** | In `FaceTagCanvas.tsx:416-437`, candidate rectangles feature an accessible dismiss `IconButton` (`aria-label`, `touch-target-expand`, Enter/Space keyboard support) invoking `handleDismissCandidateBox(idx)` to discard false positive detections immediately. |
| **UX-085** | minor | **FIXED** | In `FaceTagCanvas.tsx:409`, border color updated to `border-sky-600 dark:border-sky-400`. Sky-600 (`#0284c7`) delivers **4.01:1** contrast against white/light backgrounds, satisfying WCAG 2.2 AA Non-Text Contrast ($\ge 3:1$). |
| **UX-086** | minor | **FIXED** | In `LinkPersonDialog.tsx:156-192` and `PersonFacePicker.tsx:148-202`, `role="listbox"` contains `<Button role="option">` directly, eliminating nested interactive elements. Cyclical keyboard arrow navigation (`ArrowDown` / `ArrowUp`) works seamlessly across both search inputs and option lists. |

### Test & Build Verification Summary
- **Vite Production Build & TypeScript:** `tsc --noEmit && vite build` completed in 1.16s with 0 errors.
- **Frontend Test Suite:** 20 test suites passed (234 of 234 tests passing), including dedicated tests in `milestone-2-8-associated-people-face-detection.test.tsx` verifying UX-078 through UX-086, and full i18n parity check in `tests/i18n/parity.test.ts`.

