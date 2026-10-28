# UX Review — Authorized User Experience (Editor & Admin Roles) — 2026-10-28

## Verdict: OK WITH ISSUES
Screens & authoring workflows reviewed:
- **Authenticated Navigation & Shell:** `AppShell.tsx`, `BottomNav.tsx`, `DropdownMenu.tsx`, user profile menu, "+ Add" authoring trigger.
- **Person Profile & Authoring:** `/persons/:id` (`PersonProfilePage.tsx`, `ProfileHeroHeader.tsx`, `PersonEditDialog.tsx`, `QuickAddRelativeDialog.tsx`, `DeleteImpactDialog.tsx`).
- **Interactive Tree Canvas Authoring:** `/tree` (`TreeCanvas.tsx`, `PersonNodeCard.tsx`, `PersonDrawer.tsx`, `PersonBottomSheet.tsx`).
- **Media Management & Face Tagging:** `/media` (`MediaPage.tsx`, `FaceTagCanvas.tsx`, `MediaLightboxStudio.tsx`, `MediaUploadDialog.tsx`).
- **Data Quality & Merge Wizard:** `/quality`, `/people/merge` (`DataQualityPage.tsx`, `PersonMergePage.tsx`).
- **History, Concurrency & Revisions:** `/history` (`HistoryPage.tsx`, `RecordHistoryDrawer.tsx`, `ConflictResolveModal.tsx`).
- **Settings & Administration:** `/settings` (`SettingsPage.tsx`, `GeneralSettingsPanel.tsx`, `ProfileSettingsPanel.tsx`, `FamilyMembersInvitesPanel.tsx`, `SecurityPasskeysPanel.tsx`, `BackupsSettingsPanel.tsx`, `TreeSnapshotsCard.tsx`, `GedcomSettingsPanel.tsx`).

Viewports & Locales Verified:
- Desktop (1440px / 1280px), Tablet (768px), and Mobile (390px / 360px).
- Light and Dark themes.
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

---

## Tasks for frontend-developer

### UX-029 [critical] Tree canvas pointer capture swallows left clicks on person cards; keyboard activation missing
- **Screen / viewport / theme / lang:** `/tree` (`TreeCanvas.tsx:284-299`, `PersonNodeCard.tsx:171-198`), all viewports, all themes, EN & RU.
- **Problem:**
  1. Users cannot reliably open the person drawer/bottom sheet by left-clicking on person cards. In `TreeCanvas.tsx`, `handlePointerDown` calls `e.currentTarget.setPointerCapture(e.pointerId)` on the canvas element for left-clicks because the exemption selector `button, [role="button"], [role="radio"], select, input` does not include `[role="treeitem"]` or `[data-node-id]`. When a user left-clicks a card, pointer capture routes events to the canvas. Any minute pointer jitter (>= 1px) marks the gesture as a drag pan (`isDraggingRef.current = true`), cancelling or suppressing the card's `onClick`.
  2. In contrast, right-clicking (`e.button !== 0`) hit line 283 (`if (e.button !== 0 && e.pointerType === 'mouse') return;`), bypassing pointer capture, which led users to discover that right-click opened the card while left-click failed.
  3. `PersonNodeCard` lacks `tabIndex={0}` and an `onKeyDown` handler (Enter / Space) to trigger `onSelect(node)`, preventing keyboard users from selecting cards or opening details.
- **Evidence:** `frontend/src/components/tree/TreeCanvas.tsx:282-299`, `frontend/src/components/tree/PersonNodeCard.tsx:171-198`.
- **Proposed fix:**
  1. In `TreeCanvas.tsx`, add `[role="treeitem"], [data-node-id]` to the exemption check in `handlePointerDown`:
     ```ts
     targetEl.closest('button, [role="button"], [role="radio"], [role="treeitem"], [data-node-id], select, input')
     ```
     This prevents `setPointerCapture` from capturing left-clicks on person cards.
  2. In `PersonNodeCard.tsx`, add `tabIndex={0}` and `onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect?.(node); } }}` to allow keyboard selection.
- **Acceptance criteria:**
  - Left-clicking anywhere on a person card immediately and reliably opens `PersonDrawer` (desktop) or `PersonBottomSheet` (mobile).
  - Panning the tree canvas only occurs when clicking/dragging the canvas background or dragging outside interactive cards.
  - Tabbing through the tree reaches person cards and pressing Enter or Space opens the drawer.
- **Tags:** a11y, keyboard, canvas

---

### UX-030 [major] Mobile `PersonBottomSheet` missing authoring actions (Add Mother, Add Spouse, Add Event, Delete)
- **Screen / viewport / theme / lang:** Mobile viewports (390px), `/tree` (`PersonBottomSheet.tsx:206-234`), all themes, EN & RU.
- **Problem:** When an Editor or Admin opens the `PersonBottomSheet` on mobile, the quick-actions bar in expanded state only contains "Add Father" and "Add Child" (`onQuickAdd?.(person, 'father')`, `onQuickAdd?.(person, 'child')`), completely omitting "Add Mother" and "Add Spouse", as well as "Add Event" and "Delete Person" which are present on desktop `PersonDrawer.tsx:307-366`. Authorized users on phones cannot build maternal or spousal connections, record vital events, or delete records from the tree canvas.
- **Evidence:** `frontend/src/components/tree/PersonBottomSheet.tsx:206-234` vs `frontend/src/components/tree/PersonDrawer.tsx:307-366`.
- **Proposed fix:**
  1. Add "Add Mother" (`onQuickAdd(person, 'mother')`) and "Add Spouse" (`onQuickAdd(person, 'spouse')`) buttons alongside Father and Child.
  2. Add "Add Event" (`onAddEvent?.(person)`) and "Delete Person" (`onDeletePerson?.(person)`) action buttons.
  3. Ensure all buttons use minimum 40px touch targets (`min-h-11 sm:min-h-8`).
- **Acceptance criteria:**
  - On mobile tree view, expanded bottom sheet displays full authoring parity with desktop drawer: Add Father, Add Mother, Add Spouse, Add Child, Edit Profile, Add Event, and Delete Person.
  - All authoring action buttons trigger their respective modals.
- **Tags:** consistency, mobile, authoring

---

### UX-031 [major] Face tags on photograph canvas keyboard inaccessible; missing focus rings, tooltip disclosure, and delete
- **Screen / viewport / theme / lang:** `/media` (`FaceTagCanvas.tsx:290-415`), all viewports, all themes, EN & RU.
- **Problem:**
  1. Face tags drawn on photos in `FaceTagCanvas.tsx` are rendered as non-interactive `<div>` containers without `tabIndex={0}`, `role="button"`, or accessible names (`aria-label`). Keyboard users navigating through the media studio cannot focus on face tags.
  2. The person metadata card (display name, birth/death dates †, and "Open Profile" button) only appears on mouse hover (`onMouseEnter`/`onMouseLeave`). When navigating via keyboard, this information is completely hidden.
  3. There is no keyboard-operable delete trigger directly on the face tag for editors.
- **Evidence:** `frontend/src/components/media/FaceTagCanvas.tsx:297-320`, `372-414`.
- **Proposed fix:**
  1. Add `tabIndex={0}`, `role="button"`, and `aria-label={t('mediaArchive.faceTagAria', { name: tag.person_name })}` to the bounding box container.
  2. Add `onFocus` and `onBlur` handlers so keyboard focus reveals the person card just like hover does.
  3. Provide visible focus ring styling (`focus-visible:ring-2 focus-visible:ring-focus`).
  4. Ensure the "Open Profile" button in the revealed card is keyboard reachable.
- **Acceptance criteria:**
  - Tabbing through the photo in `FaceTagCanvas` places focus on each face tag sequentially with a visible focus ring.
  - Focusing a tag displays the person's info tooltip card.
  - Screen readers announce the person's name and vital dates.
- **Tags:** a11y, keyboard, media

---

### UX-032 [major] Non-admin authorized users (Editors & Viewers) locked out of Profile and Passkeys in mobile "More" drawer
- **Screen / viewport / theme / lang:** Mobile viewports (< 768px), `BottomNav.tsx:104-135`, all themes, EN & RU.
- **Problem:** In the mobile navigation `BottomNav.tsx`, the "More" drawer lists Places, Print, History, Quality, and "Admin Settings" (`/settings`).
  1. For Editors and Viewers, `/settings` is an admin-only route that returns `403 Forbidden` (`AccessDeniedCard`).
  2. There is no link in the mobile drawer to "My Profile" (`/settings/profile`) or "Security & Passkeys" (`/settings/security`), which are the only self-service settings accessible to non-admins.
  3. In addition, the desktop header avatar menu is hidden or crowded on mobile. Consequently, an Editor using a smartphone cannot access their account settings, change their display name/language, enroll a new WebAuthn passkey, or sign out from the mobile drawer.
- **Evidence:** `frontend/src/components/ui/bottom-nav.tsx:104-135`, `frontend/src/docs/pages/07-admin-settings-page.md:6-8`.
- **Proposed fix:**
  1. In `BottomNav.tsx`, conditionally render links based on user role:
     - For all authenticated users: add "My Profile" (`/settings/profile`, `User` icon) and "Security & Passkeys" (`/settings/security`, `KeyRound` icon).
     - Add a "Sign Out" button at the bottom of the drawer.
     - For Admins only: display "Admin Settings" (`/settings`), "History", "Quality", and "GEDCOM Import/Export" (`/settings/gedcom`).
     - For non-admins: hide or omit the admin-only settings links so they don't encounter 403 Forbidden pages.
- **Acceptance criteria:**
  - Editors and Viewers on mobile see "My Profile" and "Passkeys" in the "More" drawer, with zero 403 links.
  - Admins on mobile see Profile, Passkeys, and Admin Settings with GEDCOM.
  - Tapping "Sign Out" in mobile drawer signs out the user cleanly.
- **Tags:** rbac, navigation, mobile

---

### UX-033 [major] Invitation creation modal lacks 'Admin' role selection option
- **Screen / viewport / theme / lang:** `/settings` (`FamilyMembersInvitesPanel.tsx:366-378`), desktop & mobile, all themes, EN & RU.
- **Problem:** In the "Invite Family Member" dialog, the role selector dropdown options only include "Editor" (`roleEditor`) and "Viewer" (`roleViewer`). It omits "Admin" (`roleAdmin`), even though `RoleChangeDialog.tsx` allows assigning the Admin role and `docs/pages/07-admin-settings-page.md:48-56` specifies role management across Admin, Editor, and Viewer. An administrator setting up the family archive cannot invite a co-administrator or family archivist successor directly via invite link.
- **Evidence:** `frontend/src/components/settings/FamilyMembersInvitesPanel.tsx:374-377`.
- **Proposed fix:**
  - Add `{ value: 'admin', label: t('settings.users.roleAdmin') }` to the `options` array of the `Select` component in `FamilyMembersInvitesPanel.tsx`.
  - Update `inviteRole` state type to `'admin' | 'editor' | 'viewer'`.
- **Acceptance criteria:**
  - The invitation dialog role dropdown offers Admin, Editor, and Viewer.
  - Generated invitation link successfully creates an invite with the Admin role when selected.
- **Tags:** rbac, settings, authoring

---

### UX-034 [minor] Authenticated mobile header button congestion and title squeeze
- **Screen / viewport / theme / lang:** Mobile viewports (360px - 390px), `AppShell.tsx:168-248`, light & dark themes, especially in Russian (`ru`).
- **Problem:** In authenticated mode on 390px screens, `AppShell.tsx` renders 5 buttons in the right header container: Search trigger, `+ Add` authoring dropdown, Language toggle, Theme toggle, and User profile avatar. Together with container padding and gaps, these controls consume > 220px of width. On 360-390px screens, this leaves under 140px for the brand logo and title. In Russian, "Семейный архив Левшиц" causes severe title crowding or pushes actions against the screen edge.
- **Evidence:** `frontend/src/components/ui/app-shell.tsx:168-248`.
- **Proposed fix:**
  1. On mobile screens (`< sm`), hide the top header `+ Add` button (`hidden sm:inline-flex`), because "+ Add" is readily accessible via the tree canvas and people/media directory buttons.
  2. On mobile screens (`< sm`), collapse search to an icon-only button without extra margins.
  3. Ensure `min-w-0` and `truncate` are applied to the title heading so it elides gracefully.
- **Acceptance criteria:**
  - On a 360px and 390px viewport in Russian and English, the header has comfortable breathing room (>= 12px between title and right controls).
  - No horizontal scrolling or button overlapping occurs.
- **Tags:** layout, space, responsive

---

### UX-035 [minor] Destructive "Delete Person" button in `PersonDrawer` lacks visual danger treatment
- **Screen / viewport / theme / lang:** `/tree` (`PersonDrawer.tsx:359-366`), desktop viewports, all themes, EN & RU.
- **Problem:** In `PersonDrawer.tsx`, the "Delete Person" button is rendered with `variant="ghost"`. Only the small `Trash2` icon has `className="h-3 w-3 text-danger"`, while the button text itself ("Delete Person" / "Удалить персону") uses standard neutral `text-secondary` with no danger hover styling. According to visual hierarchy and destructive action conventions, destructive mutations must be immediately distinguishable from non-destructive actions ("Edit Profile", "Add Event").
- **Evidence:** `frontend/src/components/tree/PersonDrawer.tsx:359-366`.
- **Proposed fix:**
  - Style the button with distinct danger cues: `variant="ghost" className="text-danger hover:bg-danger/10 hover:text-danger"`.
- **Acceptance criteria:**
  - The Delete Person button in `PersonDrawer` clearly presents danger text and hover state.
- **Tags:** hierarchy, consistency

---

### UX-036 [minor] Form error alerts lack programmatic association (`aria-invalid`, `aria-describedby`) with invalid fields
- **Screen / viewport / theme / lang:** All authoring dialogs (`PersonEditDialog.tsx`, `GeneralSettingsPanel.tsx`, `ProfileHeroHeader.tsx`, `QuickAddRelativeDialog.tsx`), all viewports, all themes.
- **Problem:** When an API error or validation error occurs (e.g. invalid date format, missing required name, conflict), the error is rendered in a top-level `<Alert variant="danger">`, but the associated `TextInput` or `Select` field does not receive `aria-invalid="true"` or `aria-describedby="[error-id]"`. Screen reader users who navigate to the input are not notified that the field is invalid or why.
- **Evidence:** `frontend/src/components/person/PersonEditDialog.tsx:288-320`, `frontend/src/components/profile/ProfileHeroHeader.tsx:492-520`.
- **Proposed fix:**
  - In `FormField.tsx` and authoring dialogs, when an error message is present, attach `aria-invalid={true}` and `aria-describedby="form-field-error-[name]"` to the child input.
- **Acceptance criteria:**
  - When form validation fails, the offending inputs have `aria-invalid="true"`.
  - Screen readers announce the error message when focusing the invalid input.
- **Tags:** a11y, forms

---

## Positive Notes
- **Slavic Patronymic Auto-Calculation:** The `QuickAddRelativeDialog` automatically and accurately derives Slavic patronymics for children when adding to a father (e.g., Mikhail -> Mikhailovich / Mikhailovna), significantly expediting repetitive tree authoring for East Slavic genealogy.
- **Multi-Touch & Pinch-to-Zoom Precision:** Panning and zooming on the canvas feel smooth and robust, properly anchoring around cursor and centroid coordinates without jitter.
- **Undo Toast Coverage on Mutations:** Edits across `PersonEditDialog`, `PersonMergePage`, and `DeleteImpactDialog` consistently provide a non-destructive 10-second undo toast backed by transaction IDs, giving archivist families peace of mind.
- **Safety Safeguards on Sensitive Operations:** Snapshot rollback, clean-slate GEDCOM import, and sole-admin deactivation all enforce typed confirmations ("ROLLBACK", "CLEAN_SLATE") and disable buttons until verified, preventing accidental family data loss.
- **Localization & Slavic Orthography:** Cyrillic genealogical labels, complex declensions, and dagger notations (`†`) are rendered consistently with zero icon glyph pollution in translation files.

