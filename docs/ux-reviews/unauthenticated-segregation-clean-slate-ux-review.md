# UX Review — Unauthenticated Segregation & Clean Slate — 2026-10-28

## Verdict: OK
Screens reviewed:
- Public Home Dashboard: `/` (`DashboardPage.tsx`, `AnniversariesWidget.tsx`, `TreeGlanceWidget.tsx`, `RecentActivityWidget.tsx`, `LatestPhotosWidget.tsx`)
- Public Tree Visualizer: `/tree` (`TreeCanvas.tsx`, `TreeToolbar.tsx`, `PersonNodeCard.tsx`, `PersonDrawer.tsx`, `PersonBottomSheet.tsx`, `PublicGuestBanner.tsx`, `EmptyTreeOnboardingCard.tsx`)
- Public Person Profile: `/persons/:id` (`PersonProfilePage.tsx`, `ProfileHeroHeader.tsx`, `ProfileChronologyTab.tsx`, `ProfileFamilyTab.tsx`, `ProfileMediaTab.tsx`)
- Navigation Shell & Mobile Layout: `AppShell.tsx`, `BottomNav.tsx`, `NavLink.tsx`

Viewports & Locales Verified:
- Desktop (1280px+) and Mobile (390px)
- Light & Dark themes
- English (`en`) and Russian (`ru`)

Recordings / Verification Suites:
- `ux_unauthenticated_dashboard_audit`
- `ux_unauthenticated_tree_canvas_audit`
- `ux_unauthenticated_person_profile_audit`
- `ux_unauthenticated_navigation_shell_audit`
- `ux_clean_slate_empty_tree_audit`
- Automated test suite: `tests/components/phase2-delight.test.tsx` (all 9 delight/unauthenticated tests pass; 114 total frontend tests pass)

---

## Status of Previous Tasks
*From Phase 1.11, 1.12, 1.13, and Phase 2 reviews:*
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

## Detailed Review Findings

### 1. Navigation Shell & Mobile Bottom Navigation
- **Desktop Header (`AppShell.tsx`):**
  - Left navigation renders exactly two primary links: `Home` (`/`) and `Tree` (`/tree`). All member-only links (`People`, `Media`, `Places`, `Print`, `History`, `Quality`, `Settings`) and dropdown menus are cleanly omitted.
  - Right controls display the `LanguageToggleButton`, `ThemeToggleButton`, and a prominent primary `Sign In` button (`variant="primary" size="md" leftIcon={<LogIn />}`).
  - Omnibox `⌘K` search trigger, `+ Add` authoring dropdown, and user profile avatar menu are strictly suppressed when unauthenticated.
- **Mobile Bottom Navigation (`BottomNav.tsx`):**
  - Grid cleanly dynamically switches from 5 columns to 3 columns (`grid-cols-3`): `Home`, `Tree`, and `Sign In`.
  - All touch targets satisfy WCAG 2.2 AA tap target sizing (`min-h-12 min-w-12`, 48×48px).
  - The "More" drawer trigger button (`MoreHorizontal`) is completely suppressed for unauthenticated guests, preventing access to restricted member links.
- **Mobile Hamburger Drawer (`AppShell.tsx`):**
  - Renders only `Home`, `Tree`, and a dedicated full-width `Sign In` button. Zero restricted routes are exposed.

### 2. Public Home Dashboard (`/`)
- **Header & Hero Banner:**
  - Displays archive title and subtitle with the single primary action: `t('app.openWorkbench')` ("Explore Family Tree" / "Открыть древо").
  - The `+ Add Person` button is completely omitted (`canEdit = false`).
- **Widgets in Public Mode:**
  - **AnniversariesWidget:** Shows public anniversary events; living relatives are masked with single-initial format (`E.`) with zero privacy badge clutter, while deceased relatives format with dignified genealogical prefix dagger `(†1989)`.
  - **TreeGlanceWidget:** Renders real archive counts. Interactive stat cards ("Persons", "Places") are accessible via keyboard (`role="button" tabIndex={0}`) with focus rings and Enter/Space handlers.
  - **RecentActivityWidget:** Shows public activity feed. The admin "All changes" button and individual transaction `Undo` buttons are completely hidden (`isAdmin = false`).
  - **LatestPhotosWidget:** Read-only thumbnail grid. Clicking opens the lightbox studio with `canEdit={false}`, hiding all face tagging and delete actions.

### 3. Public Tree Canvas Visualizer (`/tree`)
- **Privacy Notice Banner (`PublicGuestBanner.tsx`):**
  - Informative banner with `variant="info"` and `Shield` icon explains living member masking ("Living Family Members Are Protected" / "Данные живущих родственников защищены") and includes a direct `Sign In to Family Archive` button.
- **Tree Canvas Toolbar (`TreeToolbar.tsx`):**
  - Clean projection switcher (Hourglass, Pedigree, Descendants, Fan Chart), generation depth slider, zoom pill, focal reset, fit-to-screen, and Kinship Finder.
  - Zero authoring buttons (`+ Add Relative`, `Edit Node`).
- **Node Cards (`PersonNodeCard.tsx`):**
  - Living relatives render masked with single initial (e.g. `E.`) and private notice (`* [Private]` / `* [Личные данные скрыты]`).
  - Deceased relatives render authentic lifespan with prefix dagger `(†1989)`.
  - Node quick actions: ONLY the `Focus` button (`tree.card.focusBtn`) is rendered. The `+ Add` button (`tree.card.addBtn`) is strictly omitted (`canEdit && !isMaskedLiving`).
  - Zero placeholder slots (`layout.placeholders` are omitted when `canEdit = false`).
  - Zero hover handles or edit triggers.
- **Inspection Sheets (`PersonDrawer.tsx` / `PersonBottomSheet.tsx`):**
  - Renders identity header, vital facts, and immediate family links.
  - Footer buttons provide clean read-only navigation: `Make Focal` and `View Full Profile`.
  - All authoring actions (Add father/mother/spouse/child, Edit profile, Add event, Delete person, Set as Home Person) are strictly hidden (`canEdit && isAuthenticated`).

### 4. Public Person Profile (`/persons/:id`)
- **Hero Header (`ProfileHeroHeader.tsx`):**
  - Clean identity presentation with avatar, name, and lifespan.
  - Action buttons: ONLY `View in tree` and `More` (with `Copy Link`) are displayed.
  - The `Edit Profile` button and `+ Add` dropdown menu (Add event / Add relative) are strictly hidden (`capabilities.can_edit = false`).
  - Inline identity editor form is completely disabled.
- **Tab Content:**
  - **Chronology Tab:** Historical events timeline. Zero "Add event", "Edit event", "Delete event", or "Edit biography" buttons (`canEdit = false`).
  - **Family Tab:** Parents, Spouses, Children cards with clickable navigation links. Zero "+ Add" relative buttons (`canEdit = false`).
  - **Media Tab:** Read-only gallery. Zero "Upload photos" or "Tag face" buttons (`canEdit = false`).
  - **Sources & Notes Tabs:** Read-only citations and research summaries.

### 5. Clean Slate Experience
- **Empty Tree State (`EmptyTreeOnboardingCard.tsx`):**
  - When the archive has zero records (fresh install or post-clean slate), public visitors see a dignified waiting card:
    - Title: `t('onboarding.emptyViewerTitle')` ("Family Tree In Preparation" / "Семейное древо подготавливается")
    - Description: `t('onboarding.viewerWaitingNotice')` ("This family tree is still being prepared by archive editors...")
    - Zero `Start with self` or `Import GEDCOM` authoring buttons.
  - Zero mock records or synthetic demo names are injected into the DOM.

---

## Positive Notes
- **Total Architectural Cleanliness:** Complete separation of public and member surfaces; no editing or administrative affordances leak into the unauthenticated guest view.
- **Flawless Mobile Ergonomics:** The 3-tab BottomNav (`Home`, `Tree`, `Sign In`) is intuitive, uncluttered, and complies with 48×48px touch targets.
- **Strict Privacy Guarantees:** Living persons are masked safely on server/client boundary without breaking visual tree continuity or leaking personal details.
- **Zero Technical Regressions:** All 114 frontend tests pass cleanly.
