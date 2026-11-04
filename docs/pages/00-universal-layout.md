# Page Specification: Universal App Shell & Layout Structure

## 1. Overview & Objective
Every view in the application (with the exception of isolated full-screen auth pages) is hosted within a **Universal App Shell**. The shell provides a consistent top header, immediate access to global omnibox search, language switching (EN/RU), theme switching (Light/Dark), and standardized drawer/modal presentation.

> [!IMPORTANT] Access Segregation & Default Private Posture:
> Out of the box (`ENABLE_PUBLIC_ACCESS=false`), the application operates as a **Default Private Instance**: unauthenticated visitors never enter the universal app shell and are automatically redirected to the Sign-In page (`/login`). When public access is explicitly enabled (`ENABLE_PUBLIC_ACCESS=true`), unauthenticated guests access a stripped, read-only shell with masked living data. Authenticated family members (`Viewer`, `Editor`, `Admin`) access the full shell with role-segregated navigation per [02-rbac-access-control.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md).

---

## 2. Shell Layout Wireframe

```
+----------------------------------------------------------------------------------------------------+
| [Logo] Family Tree | [Home] [Tree] [People] [Media] [More ▾] | [🔍 Cmd+K] | [+ Add] [RU/EN] [🌙] [Avatar] |
+----------------------------------------------------------------------------------------------------+
| Breadcrumbs / Context Bar (e.g. Tree > Levshits Lineage > Aleksandr Petrovich)      [Action Button] |
+----------------------------------------------------------------------------------------------------+
|                                                                                                    |
|                                       PAGE CONTENT VIEWPORT                                        |
|                     (Canvas View / Standard Container / Dual-Pane Layout)                          |
|                                                                                                    |
+----------------------------------------------------------------------------------------------------+
| Global Toast Notifications / Status Bar (Bottom Right / Floating)                                  |
+----------------------------------------------------------------------------------------------------+
| Footer: [Tree Title] © 2026 • Private Family Archive                                               |
+----------------------------------------------------------------------------------------------------+
```

#### Mobile Shell Wireframe (`< 768px` & Standalone PWA)
```
+----------------------------------------------------------------+
| [Logo] Family Archive                      [🔍]  [Avatar/Menu] | <- Compact Header (h-14 + safe-top)
+----------------------------------------------------------------+
| Breadcrumb / Context Sub-Bar (horizontally scrollable)         |
+----------------------------------------------------------------+
|                                                                |
|                     PAGE CONTENT VIEWPORT                      |
|             (Scrollable container or full canvas)              |
|                                                                |
+----------------------------------------------------------------+
| [🏠 Home]   [🌳 Tree]   [👥 People]   [🖼 Media]   [⋯ More]    | <- BottomNav (h-16 + safe-bottom)
+----------------------------------------------------------------+
```

---

## 3. Persistent Shell Elements

### 3.1 Top Navigation Bar (Header)
- **Brand Identity:** Subtle leaf/tree icon + Instance Tree Name (e.g. *"Levshits Family Archive"*). Clicking navigates to `/` (Home).
- **Public Guest Header (`ENABLE_PUBLIC_ACCESS=true`, Unauthenticated):**
  - Renders a minimal read-only bar: Brand Logo/Title, permitted navigation links (`Home` [`/`] and `Tree` [`/tree`]), Language toggle (`RU/EN`), Theme switch, and a prominent **"Sign In"** button (navigating to `/login?returnUrl=<current_path_and_query>` to preserve context).
  - All restricted module links (`People`, `Media`, `Places`, `More ▾`), the `+ Add` creation menu, and global Omnibox search are strictly hidden.
- **Authenticated Primary Navigation Links (Viewers, Editors, Admins):**
  - `Home` (`/`): Dashboard ([16](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md)).
  - `Tree` (`/tree`): Interactive canvas visualizer.
  - `People` (`/people`): People Directory & surnames ([13](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md)).
  - `Media` (`/media`): Archive gallery of photographs and documents.
- **`More ▾` Menu (Role-Aware Dropdown):**
  - **All Authenticated (Viewer+):** `Places` (`/places`).
  - **Administrators Only:** `Data Quality` (`/quality`), `In-Tree Person Merge` (`/people/merge`), `Print / Posters` (`/export/print`), `Recent Changes` (`/history`), `GEDCOM Import & Export` (`/settings/gedcom`), `Tree Merge Wizard` (`/settings/merge`), and `Instance Settings` (`/settings`).
- **`+ Add` Button (Editor & Admin only):**
  - Quick action menu: *Add Person* (unconnected or family node), *Upload Photos*, *Tell Me About My Family* (`/stories/intake`).
  - Hidden completely for unauthenticated guests and Viewers.
- **Global Omnibox Search (`Cmd+K` / `Ctrl+K`):**
  - Accessible to all authenticated users (Viewer, Editor, Admin).
  - Instant fuzzy search by first name, patronymic, surname, maiden name, or birth year.
  - Shows recent searches, quick jumps to persons, and shortcut navigation.
  - Hidden from unauthenticated public visitors.
- **Right Utilities:**
  - **Language Selector:** Toggle between `EN` and `RU`.
  - **Theme Toggle:** Switch between `Light`, `Dark`, and `System` mode.
  - **User Profile Menu:** Shows avatar or initials; links to Profile (`/settings/profile`), Security & Passkeys, Admin Settings (Admins only), and Logout.
  - **Public Guest Indicator:** If the visitor is unauthenticated in public mode, displays a discreet *"Sign In"* button in place of the avatar menu.

### 3.2 Context Sub-Bar (Breadcrumbs & Page Header)
- For content pages (Person Profiles, Settings, Media), a standardized sub-bar renders:
  - **Left:** Hierarchy breadcrumbs (e.g. `Archive > Photos > 1941 Leningrad Group Photo`).
  - **Right:** Primary page CTA (e.g., `+ Add Life Event`, `Edit Details`, `Export PDF`).

### 3.3 Slide-Over Drawer System (Right Drawer)
- Standard width: `480px` on desktop, `100vw` on mobile.
- Used across the app for quick inspection without leaving current context:
  - Clicking a person card in the tree opens the **Quick Info Drawer**.
  - Clicking a photo thumbnail opens the **Media Detail Drawer**.

### 3.4 Modal Dialog Standards
- Max widths: `sm` (400px, confirmations), `md` (560px, simple forms), `xl` (800px, multi-step wizards), `full` (95vw, merge diff review).
- Backdrop: Soft frosted glass (`backdrop-blur-sm bg-black/40`).
- Keyboard controls: `Esc` closes modal; `Enter` submits primary action.

### 3.5 Route Map
| Route | Page | Access Level / Role | Spec |
| :--- | :--- | :--- | :--- |
| `/setup` | First-time instance setup | Unconfigured only (0 users exist) | [01 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/01-auth-pages.md) |
| `/login`, `/register`, `/magic-link/verify` | Auth & invite acceptance | Public / Unauthenticated | [01 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/01-auth-pages.md) |
| `/` | Home dashboard / empty onboarding | Authenticated (Viewer+) or Public Guest (Read-only, masked living data when `ENABLE_PUBLIC_ACCESS=true`; redirects `/login` if private) | [09 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/09-dashboard-page.md) |
| `/tree` | Tree canvas visualizer | Authenticated (Viewer+) or Public Guest (Masked living data when `ENABLE_PUBLIC_ACCESS=true`) | [02 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/02-tree-view-page.md) |
| `/people`, `/people/surnames` | People directory & surnames | Authenticated (Viewer+) | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/people/merge` | In-tree duplicate merge | Administrators Only | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/persons/:id` | Person profile (UUID or `gid`) | Authenticated (Viewer+); Public Guest if public enabled (Masked living data) | [03 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/03-person-profile-page.md) |
| `/places`, `/places/:id` | Places directory & detail | Authenticated (Viewer+) | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/media` | Media archive & face tagging | Authenticated (Viewer+) | [04 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/04-media-gallery-page.md) |
| `/stories/intake` | AI Story Intake & Subtree Staging Studio | Editors & Admins | [10 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/10-ai-story-intake-page.md) |
| `/export/print` | Printable PDF studio | Administrators Only | [05 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/05-printable-pdf-page.md) |
| `/settings/merge` | Tree import & merge wizard | Administrators Only | [06 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/06-tree-merge-page.md) |
| `/history` | Global audit & change history feed | Administrators Only | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/quality` | Data quality & soft warnings | Administrators Only | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/settings/profile` | User profile, language, start page | Authenticated (All users) | [07 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/07-admin-settings-page.md) |
| `/settings/*` | Instance settings, users, backups | Administrators Only | [07 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/07-admin-settings-page.md) |
| `*` | 404 page with search box and link | All | — |

### 3.6 Global States
- **Loading:** skeleton placeholders matching final layout (no spinners on full pages).
- **Empty:** every list has a friendly empty state with the primary action (*"No places yet — add one"*).
- **Error:** inline retry card; global toast for failed saves with *Retry*; `version_conflict` opens the conflict dialog ([14 §4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md#4-optimistic-concurrency)).
- **Offline:** banner *"You're offline — changes can't be saved"*; forms keep their unsaved input.
- **Unsaved changes guard:** navigating away from a dirty form asks for confirmation.

### 3.7 Persistent Footer & Anti-Fluff Presentation Rules
- **Footer Scope:** Rendered at the bottom of container-based views (`/`, `/people`, `/media`, `/settings`, etc.). On full-viewport canvas views (`/tree`), the footer is omitted to maximize vertical tree space.
- **Footer Content:**
  - Left: Tree instance title + copyright year (e.g. *"Levshits Family Archive © 2026"*).
  - Right: Discreet archival statement (e.g. *"Private Family Archive"* / *"Частный семейный архив"*).
- **Strictly Prohibited in Shell & Footer:**
  - **No tech-stack strings:** Never display strings like `"Axum + React 19 + Tailwind v4 + PostgreSQL 16"`.
  - **No marketing hype or badges:** No promotional slogans like `"Next-Gen Family Archive Platform"` or `"Zero-Cloud Dependency"`.
  - **No developer links in user UI:** Internal tools like `/swagger-ui` (OpenAPI) and `/dev/ui` (workbench) must never appear in end-user navigation, footers, or cards.

### 3.8 Mobile Bottom Navigation (`BottomNav`)
- **Activation:** Displayed exclusively on mobile screens (`< 768px`). Replaces desktop top-navigation links to bring primary navigation within thumb reach.
- **Dimensions & Safe Area:** `h-16` base height + `padding-bottom: max(0.5rem, env(safe-area-inset-bottom))`.
- **Public Guest Mode (`ENABLE_PUBLIC_ACCESS=true`, Unauthenticated):**
  - Renders a minimal 3-action thumb bar: `[🏠 Home]`, `[🌳 Tree]`, and `[🔐 Sign In]`. All restricted modules (`People`, `Media`, `Places`, `More`) are completely omitted.
- **Authenticated Navigation Tabs (5 fixed items, 48×48px tap targets):**
  1. `Home` (`/`): Dashboard ([16](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md))
  2. `Tree` (`/tree`): Canvas visualizer ([04](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md))
  3. `People` (`/people`): Directory & surnames ([13](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md))
  4. `Media` (`/media`): Photos & documents archive ([06](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/06-media-face-tagging.md))
  5. `More` (`⋯`): Opens a swipeable bottom sheet containing role-filtered navigation items:
     - All Authenticated: Places (`/places`), Language toggle, Theme switch, User Profile (`/settings/profile`), *"Install App"* ([18](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/18-progressive-web-app-mobile.md)).
     - Admins Only: Data Quality (`/quality`), Duplicate Person Merge (`/people/merge`), Print PDF (`/export/print`), Recent Changes (`/history`), Admin Settings (`/settings`).

### 3.9 PWA Standalone Mode & Safe-Area Architecture
- **Standalone Launch:** When running as a standalone PWA, browser address bar and controls are absent.
- **Safe-Area Insets:**
  - Compact Topbar applies `padding-top: max(0.75rem, env(safe-area-inset-top))` to clear camera notches / dynamic islands.
  - BottomNav applies `padding-bottom: max(0.5rem, env(safe-area-inset-bottom))` to avoid colliding with the iOS home indicator bar.
  - Floating action buttons on canvas views clear safe-area left and right margins in landscape orientation.

---

## 4. Definition of Done (DoD)

- [ ] **Universal Consistency:** Topbar, search trigger, theme switcher, and language switcher remain identical across all non-auth pages.
- [ ] **Default Private Redirection:** Out of the box (`ENABLE_PUBLIC_ACCESS=false`), unauthenticated requests to `/` or any internal route immediately redirect to `/login`; no shell chrome or genealogical data is exposed.
- [ ] **Role-Segregated Navigation:** The `+ Add` button is hidden for Viewers; `Data Quality` (`/quality`), `Duplicate Merge` (`/people/merge`), `Print / Posters` (`/export/print`), `Recent Changes` (`/history`), `Import / Export`, and `Instance Settings` are rendered exclusively for Administrators.
- [ ] **Public Mode Minimalist Shell:** When `ENABLE_PUBLIC_ACCESS=true`, unauthenticated guests receive a stripped header showing only `Tree`, language/theme toggles, and a *"Sign In"* button, with living data masked.
- [ ] **Mobile Bottom Navigation:** On screens `< 768px`, navigation shifts to a fixed, thumb-accessible `BottomNav` with safe-area padding and role-filtered `More` sheet.
- [ ] **PWA Standalone & Safe-Area Insets:** In standalone PWA mode, headers, toolbars, bottom sheets, and bottom navigation render clear of device notches and home indicator bars in both portrait and landscape.
- [ ] **Touch Target Standard:** All interactive controls (tabs, buttons, dropdown triggers) maintain a minimum hit area of 44×44px (48×48px for primary mobile actions).
- [ ] **Zero Layout Shift:** Theme and language switches execute instantaneously with zero content jump or layout flicker.
- [ ] **Accessibility:** Full keyboard traversal (`Tab`, `Shift+Tab`, `Cmd+K`, `Esc`) and visible focus indicators on all interactive header elements.
- [ ] **Clean Archival Footer & Zero Stack Mentions:** Footer contains only family archive branding and copyright; tech-stack strings (Axum, React, Tailwind, PostgreSQL) and marketing buzzwords are strictly absent across the entire shell.
