# Page Specification: Home Dashboard (`/`)

## 1. Overview & Objective
The Home Dashboard provides an inviting, informative landing view that welcomes family members and gives them immediate reasons to re-engage with the family archive. It serves as the primary post-authentication landing page and the public landing page when public site sharing is active.

> [!IMPORTANT] Access Segregation & Public Guest Behavior:
> - **Default Private Instance (`ENABLE_PUBLIC_ACCESS=false`):** Unauthenticated visitors cannot view the dashboard and are immediately redirected to `/login`.
> - **Public Guest Mode (`ENABLE_PUBLIC_ACCESS=true`, Unauthenticated):** Public visitors can browse the dashboard in **Strictly Read-Only Mode**:
>   - The hero header displays the archive title (e.g. *"Levshits Family Archive"*) with an `[Open Tree →]` CTA. The `[+ Add Person]` button is **strictly omitted**.
>   - **Zero Edit Controls:** No authoring, mutation, or configuration controls are rendered anywhere.
>   - **Living Person Masking:** Living relatives' names are masked to a single initial with dot (e.g. `И.` / `J.`), and exact birth dates are redacted.
>   - **Recent Activity:** Displays additions and updates with living names masked; `Undo` buttons and internal transaction IDs are completely suppressed.
>   - **Data Quality Link:** Administrative data-quality nudge links are hidden.
> - **Authenticated Viewers (`Viewer`):** Full read-only visibility into living relatives, birthdays, and photos; `+ Add Person` and edit controls are hidden.
> - **Authenticated Editors & Admins (`Editor`, `Admin`):** Full access plus `[+ Add Person]` CTA and operational actions.

---

## 2. Page Wireframe & Component Layout

### 2.1 Authenticated Editor / Admin Wireframe
```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Home | Tree | People | Media | RU/EN | Theme | Avatar]      |
+------------------------------------------------------------------------------------+
| Good evening, Lev                                      [Open Tree →] [+ Add Person]|
+-----------------------------------+------------------------------------------------+
| 🎂 THIS MONTH                      | 🕓 RECENTLY ADDED & CHANGED                    |
| Today   · Vera Petrova turns 34   | Vera added Пётр Щеглов — 2h ago         [Undo] |
| Nov 2   · Ivan Levshits (1931†)   | Lev uploaded 6 photos — yesterday       [Undo] |
|          would be 95 — birthday   | Lev edited Елена Морозова — 2 days ago  [Undo] |
| Nov 7   · Wedding anniversary     |                              [All changes →]   |
|          A. & E. Petrov — 90 yrs  |                                                |
+-----------------------------------+------------------------------------------------+
| 📊 TREE AT A GLANCE                | 🖼 LATEST PHOTOS                                |
| 1,420 people · 512 families       | [thumb][thumb][thumb][thumb][thumb][thumb]     |
| 340 photos · 12 generations       |                                                |
| Oldest ancestor: b. abt 1702      |                                                |
| 48 people missing birth dates →   |                                                |
+-----------------------------------+------------------------------------------------+
| Footer: [Tree Title] © 2026 • Private Family Archive                               |
+------------------------------------------------------------------------------------+
```

### 2.2 Unauthenticated Public Guest Wireframe (`ENABLE_PUBLIC_ACCESS=true`)
```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Home | Tree | RU/EN | Theme | [Sign In]]                    |
+------------------------------------------------------------------------------------+
| Levshits Family Archive                                              [Open Tree →] | <- Zero Add button
+-----------------------------------+------------------------------------------------+
| 🎂 THIS MONTH                      | 📊 TREE AT A GLANCE                            |
| Today   · I. turns 34 (Living)    | 1,420 people · 512 families                    |
| Nov 2   · Ivan Levshits (1931†)   | 340 photos · 12 generations                    |
|          would be 95 — birthday   | Oldest ancestor: b. abt 1702                   |
| Nov 7   · Wedding anniversary     | (Admin nudge link hidden)                      |
|          A. & E. Petrov — 90 yrs  |                                                |
| (Living dates masked)             |                                                |
+-----------------------------------+------------------------------------------------+
| [Activity feed & Photos preview are hidden in public guest mode to preserve        |
|  transaction log privacy and restrict media previews to authenticated family]      |
+------------------------------------------------------------------------------------+
| Footer: [Tree Title] © 2026 • Private Family Archive                               |
+------------------------------------------------------------------------------------+
```

---

## 3. Structural Components

### 3.1 Header Greeting & Top Action Bar
- **Authenticated:** Displays personalized greeting based on time of day (e.g. *"Good evening, Lev"* / *"Добрый вечер, Лев"*), the `[Open Tree →]` navigation button, and `[+ Add Person]` (for Editors and Admins only).
- **Public Guest:** Displays the tree instance title and archival subtitle without user greeting. Displays `[Open Tree →]`. The `[+ Add Person]` button is strictly omitted from rendering.

### 3.2 `AnniversariesWidget`
- Displays upcoming birthdays, wedding anniversaries, and memorial death anniversaries within a rolling 30-day window.
- **Living Person Rules:**
  - Authenticated: Displays full names, relationships, and birth dates.
  - Public Guest: Living individuals display single initial with dot (`И.` / `J.`) and living status indicator without exact dates.
- **Julian-to-Gregorian Offset:** Correctly calculates +13 days shift for pre-1918 Russian historical records.
- **Typography:** Uses genealogical prefix dagger `(†YYYY)` for deceased relatives per design-system standards (`UX-019`).

### 3.3 `RecentActivityWidget`
- Shows recent tree modifications in reverse chronological order.
- **Admin View:** Full transaction descriptions, actor attribution, relative timestamp, and 1-click `Undo` button.
- **Viewer & Editor View:** Read-only list of additions without `Undo` buttons.
- **Public Guest View:** **Strictly hidden / omitted from rendering** on the public dashboard (`effectiveTier === 'public_anonymous'`). The internal transaction log is unsuited for public visitors and would render an empty placeholder.

### 3.4 `TreeGlanceWidget`
- Displays high-level archive statistics: total individuals, families, places, and photos, plus generational span.
- Stat cards declare `role="button"`, keyboard activation (`Enter`/`Space`), and visible focus rings (`UX-015`).
- **Data Quality Nudge:** The administrative link (`X people missing birth dates →`) renders strictly for Administrators; hidden for Viewers and public guests.

### 3.5 `LatestPhotosWidget`
- Grid of 6 recently uploaded family photographs and documents.
- **Authenticated View:** Displays thumbnails of recently added photographs.
- **Public Guest View:** **Strictly hidden / omitted from rendering** on the public dashboard (`effectiveTier === 'public_anonymous'`). Photo browsing is reserved for authenticated family members to safeguard family privacy.

---

## 4. Definition of Done (DoD)

- [ ] **Default Private Redirection:** When `ENABLE_PUBLIC_ACCESS=false`, any unauthenticated visit to `/` immediately redirects to `/login`.
- [ ] **Public Guest Read-Only Mode:** When `ENABLE_PUBLIC_ACCESS=true`, unauthenticated visitors can view `/` with zero edit controls and zero `+ Add Person` button.
- [ ] **Public Guest 2-Widget Grid:** In public guest mode (`effectiveTier === 'public_anonymous'`), `/` renders exclusively `AnniversariesWidget` and `TreeGlanceWidget`; `RecentActivityWidget` and `LatestPhotosWidget` are strictly hidden from the DOM.
- [ ] **Zero Photo/Activity Payload in Public API:** Backend `GET /api/v1/dashboard/stats` (or `dashboard.rs`) for `CallerAccess::PublicAnonymous` returns empty arrays (`[]`) for both `recent_activity` and `latest_photos`.
- [ ] **Living Relative Shielding:** On public `/`, living individuals in `AnniversariesWidget` display solely as single initial with dot (`И.` / `J.`) with exact birth dates and private records hidden.
- [ ] **Role-Segregated Actions:** `+ Add Person` button renders exclusively for Editors and Administrators; hidden for Viewers and public guests.
- [ ] **Accessible Widget Cards:** All widget cards satisfy WCAG 2.2 AA keyboard navigation and focus-visible standards.
- [ ] **Default Clean Slate & Zero Dummy Data:** Production builds never insert or display hardcoded mock individuals or fake dashboard payload; an empty database displays the authentic empty-tree onboarding card.
