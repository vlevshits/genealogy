# UX Review — Comprehensive Core Screens & Flows Audit — 2026-10-28

## Verdict: OK WITH ISSUES
Screens reviewed:
- Navigation Shell & Mobile Layout: `AppShell.tsx`, `BottomNav.tsx`, `Drawer.tsx`, `OmniboxDialog.tsx`
- Public & Authenticated Dashboard: `/` (`DashboardPage.tsx`, `AnniversariesWidget.tsx`, `TreeGlanceWidget.tsx`, `RecentActivityWidget.tsx`, `LatestPhotosWidget.tsx`)
- Interactive Family Tree Canvas: `/tree` (`TreeCanvas.tsx`, `TreeToolbar.tsx`, `PersonNodeCard.tsx`, `PersonDrawer.tsx`, `PersonBottomSheet.tsx`, `FanChartCanvas.tsx`, `KinshipFinderDialog.tsx`)
- People Directory: `/people` (`PeoplePage.tsx`, `DirectoryTable.tsx`, `DirectoryCardList.tsx`, `DirectoryFilterBar.tsx`, `DirectoryBulkBar.tsx`)
- Person Profile: `/persons/:id` (`PersonProfilePage.tsx`, `ProfileHeroHeader.tsx`, `ProfileChronologyTab.tsx`, `ProfileFamilyTab.tsx`, `ProfileMediaTab.tsx`, `ProfileNotesTab.tsx`)
- Places Map & Surnames: `/places`, `/surnames` (`PlacesPage.tsx`, `PlacesMapView.tsx`, `PlacesTreeListCard.tsx`, `SurnamesPage.tsx`)
- Data Quality & Merge: `/quality`, `/people/merge` (`DataQualityPage.tsx`, `PersonMergePage.tsx`)
- History & Audit Log: `/history` (`HistoryPage.tsx`, `RecordHistoryDrawer.tsx`, `HistoryFilterBar.tsx`)
- Settings & Administration: `/settings` (`SettingsPage.tsx`, `SideNav.tsx`, `GeneralSettingsPanel.tsx`, `ProfileSettingsPanel.tsx`, `SecurityPasskeysPanel.tsx`, `FamilyMembersInvitesPanel.tsx`, `BackupsSettingsPanel.tsx`, `GedcomSettingsPanel.tsx`)
- Auth & Error States: `/login`, `/register`, `/setup`, `LoginCard.tsx`, `RegisterInviteCard.tsx`, `AccessDeniedCard.tsx`, `ErrorBoundary.tsx`

Viewports & Locales Verified:
- Desktop (1440px / 1280px) and Mobile (390px / 360px), Tablet (768px)
- Light & Dark themes
- English (`en`) and Russian (`ru`)

Recordings / Verification Suites:
- `ux_audit_navigation_shell_desktop_and_mobile`
- `ux_audit_dashboard_widgets_en_and_ru`
- `ux_audit_tree_canvas_and_bottom_sheet`
- `ux_audit_people_directory_and_cards`
- `ux_audit_person_profile_and_hero_header`
- `ux_audit_places_map_and_surnames`
- `ux_audit_data_quality_and_merge_wizard`
- `ux_audit_history_feed_and_diffs`
- `ux_audit_settings_panels_and_side_nav`
- `ux_audit_auth_cards_and_login_recovery`
- Frontend test suite: 16 test suites (116 tests) passing

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
| **UX-009** | Elevate toast container above mobile bottom navigation with safe-area clearance | **fixed** |
| **UX-010** | Add missing `role="radiogroup"`, visible focus rings, and mobile layout to conflict modal | **fixed** |
| **UX-011** | Fix untranslated `{count}` token in diff toggle buttons | **fixed** |
| **UX-012** | Prevent diff values from overflowing card boundaries on mobile viewports | **fixed** |
| **UX-013** | Format raw database transaction labels with human-friendly descriptions | **fixed** |
| **UX-014** | Optimize dialog sizing and instruction placement in revision & conflict modals | **fixed** |
| **UX-015** | Keyboard inaccessible stat cards in `TreeGlanceWidget` | **fixed** |
| **UX-016** | Missing visible focus ring on interactive SVG slices in `FanChartCanvas` | **fixed** |
| **UX-017** | Keyboard inaccessible map cluster pins in `PlacesMapView` | **fixed** |
| **UX-018** | Missing column/source identifiers on mobile stacked layout in `PersonMergePage` | **fixed** |
| **UX-019** | Suffix dagger notation `(1989†)` in `AnniversariesWidget` | **fixed** |
| **UX-020** | Screen reader list navigation distortion from `aria-hidden` intermediate `<li>` arrows in `KinshipFinderDialog` | **fixed** |
| **UX-021** | Keyboard zoom shortcuts or visible hint in `PlacesMapView` and `FanChartCanvas` | suggestion (open) |

---

## Tasks for frontend-developer

### UX-022 [major] DropdownMenu triggers (`AppShell` nav more, +add, user profile) inaccessible to keyboard navigation
- **Screen / viewport / theme / lang:** Desktop & tablet header (≥ 768px), `AppShell.tsx` (`/`, `/tree`, `/people`, etc.), all themes, EN & RU.
- **Problem:** In `AppShell.tsx` (lines 140, 189, and 219), the `trigger` prop passed to `DropdownMenu` is a non-interactive `<span>` (`<span className="...">...</span>`). In `DropdownMenu.tsx` (lines 67-72), this trigger is wrapped in a plain `<div onClick={() => setOpen((o) => !o)}>`. Because neither the wrapper `<div>` nor the inner `<span>` is a focusable element (no `tabIndex`, no `<button>`, no `role="button"`, no `onKeyDown` listener), users navigating solely via keyboard cannot reach or open:
  1. The "More" navigation menu (housing links to Places, Print, History, Quality, and GEDCOM).
  2. The "+ Add" authoring menu (housing "Add Person" and "Upload Photos").
  3. The User Profile avatar menu (housing Profile, Passkeys, Admin Settings, and Sign Out).
  Screen readers also fail to perceive them as interactive popup menus (`aria-haspopup="menu"`, `aria-expanded`). This violates WCAG 2.1.1 (Keyboard) and WCAG 4.1.2 (Name, Role, Value).
- **Evidence:** `frontend/src/components/ui/app-shell.tsx:140`, `frontend/src/components/ui/app-shell.tsx:189`, `frontend/src/components/ui/app-shell.tsx:219`, `frontend/src/components/ui/dropdown-menu.tsx:67-72`.
- **Proposed fix:**
  1. In `DropdownMenu.tsx`, render the trigger as a semantic `<button type="button">` with `aria-haspopup="menu"`, `aria-expanded={open}`, `onKeyDown` support (Enter, Space, ArrowDown to open), and a visible focus ring.
  2. In `AppShell.tsx`, use design-system buttons (`Button` with `variant="ghost"` or `IconButton`) with accessible labels for the "More", "+ Add", and user profile avatar triggers.
- **Acceptance criteria:**
  - Tabbing through `AppShell` header places focus on "More", "+ Add", and the user profile avatar with a visible focus ring.
  - Pressing Enter, Space, or ArrowDown opens the menu and shifts focus to the first menu item.
  - Pressing Escape closes the menu and returns focus to the trigger button.
  - Screen readers announce "expanded" / "collapsed" state and menu role.
- **Tags:** a11y, keyboard, hierarchy

---

### UX-023 [major] Modal Dialogs and Drawers lack keyboard Tab focus trapping and initial focus management
- **Screen / viewport / theme / lang:** All screens opening modal dialogs (`Dialog.tsx`, `Drawer.tsx`, `ConfirmDialog.tsx`, `PersonEditDialog.tsx`, `KinshipFinderDialog.tsx`, `DeleteImpactDialog.tsx`, `MediaUploadDialog.tsx`), all viewports, all themes/locales.
- **Problem:** `Dialog.tsx` (`frontend/src/components/ui/dialog.tsx:38-54`) and `Drawer.tsx` (`frontend/src/components/ui/drawer.tsx:33-42`) handle the Escape key and restore focus to `previousActiveRef` on close, but they do NOT trap focus inside the modal when users press Tab or Shift+Tab. Additionally, when opened, initial focus is not programmatically placed on the dialog or its first interactive element. Keyboard and screen reader users can Tab out of the active modal dialog into background page elements behind the backdrop, violating WCAG 2.4.3 (Focus Order), WCAG 2.1.2 (No Keyboard Trap), and the WAI-ARIA Modal Dialog specification.
- **Evidence:** `frontend/src/components/ui/dialog.tsx:42-54`, `frontend/src/components/ui/drawer.tsx:33-42`.
- **Proposed fix:**
  1. Implement a focus-trap effect in `Dialog.tsx` and `Drawer.tsx` (when `modal !== false`):
     - On mount/open: find focusable elements (`button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])`); focus the first element (or close button/container).
     - On `Tab`: if on the last focusable element, loop back to the first.
     - On `Shift+Tab`: if on the first focusable element, loop to the last.
  2. Ensure focus is reliably restored to the trigger element when the dialog unmounts.
- **Acceptance criteria:**
  - Opening any dialog automatically focuses the first interactive element.
  - Pressing Tab repeatedly never leaves the modal container while open.
  - Pressing Shift+Tab from the first element wraps around to the last element.
- **Tags:** a11y, keyboard

---

### UX-024 [major] Sub-40px touch targets and raw `<button>` elements in inputs and media controls
- **Screen / viewport / theme / lang:** Mobile (390px) & touch devices, `PasswordInput` (`input.tsx:216`), `SearchInput` (`input.tsx:170`), `MediaLightboxStudio.tsx:364, 375, 383`, `FaceTagCanvas.tsx:409`.
- **Problem:**
  1. `PasswordInput` show/hide password toggle is a raw `<button>` with `p-1` (24×24px visual and hit area), and `SearchInput` clear button is a raw `<button>` with `p-0.5` (~18×18px hit area). Neither includes the `.touch-target-expand` class or satisfies the 40×40px minimum touch target required by the design system and WCAG 2.5.8 (Target Size).
  2. In `MediaLightboxStudio.tsx`, the Prev, Next, and Close buttons use `Button` with `size="sm"` and `className="h-8 w-8 min-h-0 p-0"`, which overrides the minimum height to 0 and locks them to 32×32px on touch viewports without hit expansion.
  3. In `FaceTagCanvas.tsx`, the "Open Profile" button uses `className="h-6 min-h-0 px-2"` (24px high), making it difficult to tap on mobile.
  4. Using raw `<button>` elements in `input.tsx` violates `.agents/rules/ui-and-i18n-conventions.md` ("Never use raw `<button>`, `<input>`, or `<select>` HTML elements in pages or feature panels; always use the design-system components").
- **Evidence:** `frontend/src/components/ui/input.tsx:170`, `frontend/src/components/ui/input.tsx:216`, `frontend/src/components/media/MediaLightboxStudio.tsx:364-385`, `frontend/src/components/media/FaceTagCanvas.tsx:409`.
- **Proposed fix:**
  1. In `input.tsx`, replace the raw `<button>` elements with `IconButton` or add `touch-target-expand` class to satisfy the 44px touch bounding box.
  2. In `MediaLightboxStudio.tsx`, replace the custom-styled `Button` with `IconButton size="md"` or `IconButton size="sm"` with accessible `label`s.
  3. In `FaceTagCanvas.tsx`, replace `h-6 min-h-0` with minimum 32px height and `touch-target-expand`.
- **Acceptance criteria:**
  - All interactive buttons have an effective touch target ≥ 40×40px on touch devices.
  - Zero raw `<button>` elements exist in form inputs or media controls.
- **Tags:** a11y, consistency

---

### UX-025 [minor] Misleading "Person record not found" alert on network or server failure
- **Screen / viewport / theme / lang:** `/persons/:id` (`PersonProfilePage.tsx:428-430`, `516-521`), all viewports, EN & RU.
- **Problem:** When `fetchPersonProfile` fails due to a network outage, backend offline, or server error (HTTP 500/502), the `.catch` handler indiscriminately sets `setNotFoundError(t('profile.notFoundDesc'))`, and the page displays an alert titled "Person record not found" / "Запись о персоне не найдена" with text stating the person does not exist or is restricted by privacy settings. This misleads users and relatives into thinking an ancestor's record was permanently deleted or access was revoked, rather than recognizing a connection failure.
- **Evidence:** `frontend/src/pages/PersonProfilePage.tsx:428-430`, `frontend/src/pages/PersonProfilePage.tsx:516-521`.
- **Proposed fix:**
  1. In `PersonProfilePage.tsx`, inspect `err`. If `err instanceof ApiError && err.status === 404`, set the "not found" state.
  2. For other error statuses or network failures, route the error through `getLocalizedErrorMessage(err, t)` and display a "Retry" button (`t('common.retry')`) instead of "View in Tree".
  3. Set the alert title and icon appropriately: "Person record not found" for 404, "Unable to load person profile" for network/server errors.
- **Acceptance criteria:**
  - 404 responses display "Person record not found".
  - Network and server errors display a localized connection error alert with a "Retry" button.
- **Tags:** states, copy

---

### UX-026 [minor] Header title truncation and right action crowding on narrow mobile viewports
- **Screen / viewport / theme / lang:** Mobile (390px and below), `AppShell.tsx:75-101`, light & dark themes, especially in Russian.
- **Problem:** On 390px mobile screens, the header row contains the hamburger button, archive title, Language toggle, Theme toggle, and a "Sign In" button ("Войти в семейный архив" / "Sign in" with icon). The title container does not have `truncate` or `min-w-0`, and when the archive title is long (e.g., "Семейный архив Левшиц"), the right action buttons are compressed against the screen edge. Furthermore, "Sign In" is redundant in the header on mobile because the fixed `BottomNav` already contains a prominent "Sign In" tab.
- **Evidence:** `frontend/src/components/ui/app-shell.tsx:75-101`, `frontend/src/components/ui/bottom-nav.tsx:97-101`.
- **Proposed fix:**
  1. Add `min-w-0` to the brand link container and `truncate` to the title `<span>`.
  2. On mobile screens (`< sm`), hide the top header "Sign In" button (`hidden sm:inline-flex`), relying on the `BottomNav` tab and the mobile drawer, leaving adequate room for the title, Language, and Theme toggles.
- **Acceptance criteria:**
  - On a 390px mobile viewport in both EN and RU, the header never overflows horizontally.
  - Long archive titles truncate cleanly with an ellipsis (`…`).
  - Action icons have comfortable spacing without touching screen edges.
- **Tags:** layout, space, responsive

---

### UX-027 [minor] Missing keyboard navigation (ArrowLeft / ArrowRight) in Media Lightbox Studio
- **Screen / viewport / theme / lang:** `/media` (`MediaLightboxStudio.tsx:82-91`), desktop & laptop viewports, all themes and locales.
- **Problem:** When viewing full-screen photos and document scans in `MediaLightboxStudio`, pressing Escape closes the lightbox, but pressing ArrowLeft or ArrowRight does nothing. In media galleries and archival photograph viewers, arrow key navigation is a standard universal expectation. Keyboard users must repeatedly Tab through the UI to reach the Prev and Next buttons.
- **Evidence:** `frontend/src/components/media/MediaLightboxStudio.tsx:82-91`.
- **Proposed fix:**
  - Add `ArrowLeft` -> `onPrevAsset?.()` and `ArrowRight` -> `onNextAsset?.()` to the `keydown` listener in `MediaLightboxStudio.tsx` (ignoring if an input or textarea is currently focused).
- **Acceptance criteria:**
  - Pressing ArrowLeft in the lightbox studio navigates to the previous photo.
  - Pressing ArrowRight navigates to the next photo.
  - Arrow keys do not trigger navigation when editing metadata fields in the sidebar.
- **Tags:** a11y, keyboard

---

### UX-028 [suggestion] Theme switcher button width crowding in mobile "More" drawer
- **Screen / viewport / theme / lang:** Mobile (390px and below), `BottomNav.tsx:273-316`, Russian locale (`ru`).
- **Problem:** In the mobile "More" drawer (`BottomNav.tsx`), the Theme Switcher renders three buttons side-by-side: `Light` ("Светлая"), `Dark` ("Тёмная"), and `System` ("Системная"), each with an icon and label inside an `inline-flex` group. In Russian, the labels are 7-9 characters long, which can cause label truncation or tight text crowding on narrow 360-390px screens.
- **Evidence:** `frontend/src/components/ui/bottom-nav.tsx:273-316`.
- **Proposed fix:**
  - On mobile screens, adjust the container to `grid grid-cols-3` with `px-1.5` so all three options distribute space evenly without wrapping.
- **Acceptance criteria:**
  - In Russian locale at 360px-390px widths, all three theme options display cleanly without text clipping.
- **Tags:** layout, copy, suggestion

---

## Positive Notes
- **Genealogical Typography & Precision:** The dagger symbol `†` is consistently and tastefully applied across deceased nodes, profile hero headers, vital dates lines, and anniversary items, respecting genealogical conventions while preserving dignity.
- **Living Person Privacy Masking:** Masking for living relatives in public anonymous mode is thorough and airtight across all widgets, tree node cards, profile views, and face tags.
- **Internationalization Quality:** The Russian translations (`ru.json`) are natural, grammatically correct with gender-aware kinship terms, and maintain 100% key parity with English (`en.json`). No emojis or icon glyphs leak into localization strings.
- **Theme Consistency:** Light and Dark mode tokens provide excellent contrast across cards, navigation, dialog backdrops, and interactive tree elements.
- **Responsive Architecture:** Complex data-heavy screens like `PersonMergePage` and `SettingsPage` adapt elegantly between multi-column desktop grids and stacked mobile card layouts without data loss.
