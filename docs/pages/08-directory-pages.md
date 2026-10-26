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
- `/quality` (Administrators Only): summary chips on top, three tabs (*Warnings*, *Possible duplicates*, *Incomplete*), each row with a primary fix action (*Open*, *Compare*, *Dismiss*). Accessible exclusively to Admins; hidden from navigation and returns `403 Forbidden` for Viewers and Editors.

---

## 5. Definition of Done (DoD)

- [ ] **One Layout:** All list pages share header, filter bar, table, and bulk bar components.
- [ ] **Mobile Cards:** Lists are fully usable on a phone without horizontal scrolling.
- [ ] **Role Segregation & Route Guards:** `/history`, `/quality`, and `/people/merge` are accessible exclusively to Administrators (`403 Forbidden` for Viewers and Editors); `/people` and `/places` are restricted to authenticated family members (`401 Unauthorized` / redirect to `/login` for unauthenticated visitors).
