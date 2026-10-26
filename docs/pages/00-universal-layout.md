# Page Specification: Universal App Shell & Layout Structure

## 1. Overview & Objective
Every view in the application (with the exception of isolated full-screen auth pages) is hosted within a **Universal App Shell**. The shell provides a consistent top header, immediate access to global omnibox search, language switching (EN/RU), theme switching (Light/Dark), and standardized drawer/modal presentation.

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
- **Primary Navigation Links** (kept to 4 to stay simple):
  - `Home` (`/`): Dashboard ([16](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md)). Hidden for anonymous visitors.
  - `Tree` (`/tree`): Interactive canvas visualizer.
  - `People` (`/people`): People Directory & surnames ([13](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md)).
  - `Media` (`/media`): Archive gallery of photographs and documents.
- **`More ▾` menu** (secondary, role-aware):
  - `Places` (`/places`), `Print` (`/export/print`)
  - Editors+: `Recent Changes` (`/history`), `Data Quality` (`/quality`), `Import / Export` (`/settings/gedcom`)
- **`+ Add` button** (Editors+): *Add person* (unconnected), *Upload photos*.
- **Global Omnibox Search (`Cmd+K` / `Ctrl+K`):**
  - Instant fuzzy search by first name, patronymic, surname, maiden name, or birth year.
  - Shows recent searches, quick jumps to persons, and shortcut navigation.
- **Right Utilities:**
  - **Language Selector:** Toggle between `EN` and `RU`.
  - **Theme Toggle:** Switch between `Light`, `Dark`, and `System` mode.
  - **User Profile Menu:** Shows avatar or initials; links to Profile, Security/Passkeys, Admin Settings (if Admin/Editor), and Logout.
  - **Public Guest Indicator:** If the visitor is unauthenticated in public mode, displays a discreet *"Sign In"* button.

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
| Route | Page | Spec |
| :--- | :--- | :--- |
| `/setup`, `/login`, `/register`, `/magic-link/verify` | Auth | [01 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/01-auth-pages.md) |
| `/` | Home dashboard / empty-tree welcome | [16](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md) |
| `/tree` | Tree canvas | [02 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/02-tree-view-page.md) |
| `/people`, `/people/surnames`, `/people/merge` | People directory, surnames, merge | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/persons/:id` | Person profile (UUID or `gid`) | [03 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/03-person-profile-page.md) |
| `/places`, `/places/:id` | Places | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/media` | Media archive | [04 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/04-media-gallery-page.md) |
| `/export/print` | PDF generator | [05 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/05-printable-pdf-page.md) |
| `/settings/merge` | Tree merge wizard | [06 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/06-tree-merge-page.md) |
| `/history`, `/quality` | Recent changes, data quality | [08 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/08-directory-pages.md) |
| `/settings/*` | Settings & admin | [07 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/07-admin-settings-page.md) |
| `*` | 404 page with search box and link home | — |

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
- **Navigation Tabs (5 fixed items, 48×48px tap targets):**
  1. `Home` (`/`): Dashboard ([16](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md))
  2. `Tree` (`/tree`): Canvas visualizer ([04](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md))
  3. `People` (`/people`): Directory & surnames ([13](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md))
  4. `Media` (`/media`): Photos & documents archive ([06](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/06-media-face-tagging.md))
  5. `More` (`⋯`): Opens a swipeable bottom sheet containing Places, Print PDF, Recent Changes, Data Quality, Admin Settings, Language toggle, Theme switch, and *"Install App"* ([18](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/18-progressive-web-app-mobile.md)).

### 3.9 PWA Standalone Mode & Safe-Area Architecture
- **Standalone Launch:** When running as a standalone PWA, browser address bar and controls are absent.
- **Safe-Area Insets:**
  - Compact Topbar applies `padding-top: max(0.75rem, env(safe-area-inset-top))` to clear camera notches / dynamic islands.
  - BottomNav applies `padding-bottom: max(0.5rem, env(safe-area-inset-bottom))` to avoid colliding with the iOS home indicator bar.
  - Floating action buttons on canvas views clear safe-area left and right margins in landscape orientation.

---

## 4. Definition of Done (DoD)

- [ ] **Universal Consistency:** Topbar, search trigger, theme switcher, and language switcher remain identical across all non-auth pages.
- [ ] **Mobile Bottom Navigation:** On screens `< 768px`, navigation shifts to a fixed, thumb-accessible `BottomNav` with 5 items and safe-area padding.
- [ ] **PWA Standalone & Safe-Area Insets:** In standalone PWA mode, headers, toolbars, bottom sheets, and bottom navigation render clear of device notches and home indicator bars in both portrait and landscape.
- [ ] **Touch Target Standard:** All interactive controls (tabs, buttons, dropdown triggers) maintain a minimum hit area of 44×44px (48×48px for primary mobile actions).
- [ ] **Zero Layout Shift:** Theme and language switches execute instantaneously with zero content jump or layout flicker.
- [ ] **Accessibility:** Full keyboard traversal (`Tab`, `Shift+Tab`, `Cmd+K`, `Esc`) and visible focus indicators on all interactive header elements.
- [ ] **Clean Archival Footer & Zero Stack Mentions:** Footer contains only family archive branding and copyright; tech-stack strings (Axum, React, Tailwind, PostgreSQL) and marketing buzzwords are strictly absent across the entire shell.
