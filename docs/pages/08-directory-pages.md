# Page Specification: Directory & Maintenance Pages (`/people`, `/places`, `/history`, `/quality`)

## 1. Overview & Objective
Four secondary pages that share one layout. Families, events, and notes are always reached through a person or place.

Feature behavior lives in the feature specs; this document defines the shared page layout.

| Route | Purpose | Access Level / Role | Feature spec |
| :--- | :--- | :--- | :--- |
| `/people` | People Directory | Authenticated (Viewer+) | [13 §4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md#4-people-directory-people) |
| `/people/surnames` | Surname index | Authenticated (Viewer+) | [13 §4.3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md#43-surnames-index-peoplesurnames) |
| `/people/merge?a=&b=` | Merge two duplicate people | Administrators Only | [15 §3.2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md#32-merge-flow-peoplemergeab) |
| `/places`, `/places/:id` | Places list / tree / map, place detail | Authenticated (Viewer+) | [12 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/12-places.md#3-user-experience) |
| `/history` | Global audit & change history feed | Administrators Only | [14 §3.1](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md#31-recent-changes-history) |
| `/quality` | Data quality & soft warnings | Administrators Only | [15 §4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md#4-data-quality-page-quality-editor) |

---

## 2. Shared List Layout

```
+------------------------------------------------------------------------------------+
| [Global Header]                                                                    |
+------------------------------------------------------------------------------------+
| Title (count)      [Sub-tabs: List | Tree | Map]          [🔍 Filter...] [+ Add]  |
| [Filter chips ............................................................] [Clear]|
+------------------------------------------------------------------------------------+
| Virtualized table / feed                                                            |
|  - sticky header, sortable columns, row hover actions (⋯)                          |
|  - click row → detail page; Cmd/Ctrl+click → new tab                               |
|  +------------------------------------------------------------------------------------+
| Bulk action bar (appears when rows are selected; Editors+ only)                    |
+------------------------------------------------------------------------------------+
```

- **Max content width** `max-w-7xl`, comfortable row height 56 px (avatars 32 px), compact density toggle.
- **Mobile (<768 px):** tables collapse to cards (name + key dates + chevron); filters move into a bottom sheet.
- **Filter state in URL** on every list page.

## 3. Detail Layout (place)

```
+------------------------------------------------------------------------------------+
| Breadcrumbs: Places > Самарская губ. > Новоузенский уезд > Покровка   [Edit] [⋯]   |
+------------------------------------------------------------------------------------+
| HEADER: name, type, key metadata, alt names                                        |
+--------------------------------------------+---------------------------------------+
| MAIN: linked records list                  | SIDE: map / scans / notes             |
| (people & events here, sub-places)         |                                       |
+--------------------------------------------+---------------------------------------+
```
`⋯` menu: *History*, *Merge into…*, *Delete* (role-aware; disabled with tooltip when blocked, e.g. place in use).

## 4. History & Quality Pages
- `/history` (Administrators Only): day-grouped feed (avatar of author, sentence, time, *Restore/Undo* action), filters as chips. Accessible exclusively to Admins; hidden from navigation and returns `403 Forbidden` for Viewers and Editors.
- `/quality` (Administrators Only): Centralized data maintenance hub accessible exclusively to Admins (`403 Forbidden` for Viewers and Editors).
  - **Summary Chips:** `⚠️ Warnings ({count})`, `👥 Duplicates ({count})`, `📍 Place Hierarchy ({count})`, `ℹ️ Incomplete ({count})`.
  - **Navigation Tabs:**
    1. *All Warnings* / *Critical* / *Warnings* / *Info*: soft validation checks with *Dismiss* ("This is correct") and direct profile links.
    2. *Duplicates*: relationship-centric duplicate pairs with similarity badge, kinship context indicators (shared parents/spouses/children, detached subtree tags), *Not a Duplicate* button, and *Compare & Merge* CTA.
    3. *Place Hierarchy* (`/quality?tab=places_hierarchy`): automated gazetteer normalization queue:
       - Sub-filters: *All issues*, *Unparsed paths*, *Orphan places*, *Redundant duplicates*.
       - Batch Action Bar: *"⚡ Auto-fix All High Confidence ({count})"* with dry-run confirmation dialog.
       - Place Fix Cards: side-by-side comparison displaying current flat place record (raw name, type, coordinates, event count) vs. proposed decomposed hierarchy tree (`[Country] → [Region] → [District] → [Clean Leaf]`), confidence pill badge, rule explanation, and actions (`Apply Fix`, `Customize / Pick Parent`, `Dismiss`).
       - Customization Dialog: allows manual parent selection via Place Picker combobox, clean leaf name editing, and place type override before applying.
       - Undo Toast: immediate toast notification with 1-click Undo button reversing the batch or single fix transaction.

### 4.1 Places Page Administrative Banner (`/places`)
- When the backend detects unparsed place strings, orphan localities, or administrative type mismatches, `/places` renders a top notification card for Administrators:
  *"14 place hierarchy issues detected in your tree. Clean up jurisdictions, link orphan localities, and enhance map accuracy."* with action button `[ Review & Fix in Data Quality ]` routing to `/quality?tab=places_hierarchy`.

---

## 5. Definition of Done (DoD)

- [ ] **One Layout:** All list pages share header, filter bar, table, and bulk bar components.
- [ ] **Mobile Cards:** Lists are fully usable on a phone without horizontal scrolling.
- [ ] **Role Segregation & Route Guards:** `/history`, `/quality`, and `/people/merge` are accessible exclusively to Administrators (`403 Forbidden` for Viewers and Editors); `/people` and `/places` are restricted to authenticated family members (`401 Unauthorized` / redirect to `/login` for unauthenticated visitors).
- [ ] **Place Hierarchy Quality Tab:** `/quality` features a dedicated "Place Hierarchy" tab with summary chips, before/after cards, confidence badges, and customizable parent selection.
- [ ] **Batch Auto-Fix Workflow:** Clicking "Auto-fix High Confidence" displays a confirmation summary and applies all high-confidence recommendations with an instant Undo toast.
- [ ] **Places Page Quality Banner:** `/places` displays a notification banner for Administrators when place hierarchy issues exist, linking directly to `/quality?tab=places_hierarchy`.
