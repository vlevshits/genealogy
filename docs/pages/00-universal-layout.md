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

---

## 4. Definition of Done (DoD)

- [ ] **Universal Consistency:** Topbar, search trigger, theme switcher, and language switcher remain identical across all non-auth pages.
- [ ] **Mobile Responsiveness:** Navigation smoothly collapses into an accessible mobile hamburger drawer on screens `< 768px`.
- [ ] **Zero Layout Shift:** Theme and language switches execute instantaneously with zero content jump or layout flicker.
- [ ] **Accessibility:** Full keyboard traversal (`Tab`, `Shift+Tab`, `Cmd+K`, `Esc`) and visible focus indicators on all interactive header elements.
