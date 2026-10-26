# Page Specification: Person Profile & Life Story (`/persons/:id`)

## 1. Overview & Objective
The Person Profile page provides an immersive, biographical deep-dive into an individual ancestor's life. It balances historical storytelling with structured genealogical data, featuring a **Hero Identity Header**, an **Interactive Life Map**, a **Chronological Timeline of Life Events**, and dedicated tabs for family relations, photos, and notes.

---

## 2. Page Wireframe & Layout

```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]      |
+------------------------------------------------------------------------------------+
| HERO BANNER                                                                        |
| +------------+  Aleksandr Ivanovich Petrov (1912 – 1989)       [Edit Profile]      |
| |  [Avatar]  |  b. 14 Mar 1912 (Tver, Russia) • d. 22 Oct 1989 [View in Tree ⎘]   |
| |   Photo    |  Kinship: Great-Grandfather (Прадедушка)       [+ Add Event]        |
| +------------+                                                                     |
+------------------------------------------------------------------------------------+
| TABS: [ Chronology & Map ]  [ Family & Relations ]  [ Media (12) ]  [ Notes (5) ]  |
+------------------------------------------------------------------------------------+
| DUAL PANE CONTENT:                                                                 |
|                                                                                    |
| LEFT PANE: Life Events Timeline          | RIGHT PANE: Interactive Life Map        |
|                                          |                                         |
| 🟢 1912 (Age 0) - Birth                  | +-------------------------------------+ |
|    14 Mar 1912 • Tver, Russia            | |  [ Leaflet / MapLibre World Map ]   | |
|    Source: Tver Church Metrical Book     | |                                     | |
|                                          | |  📍 Tver (1912)                     | |
| 🔵 1934 (Age 22) - Marriage              | |   \                                 | |
|    To Elena Vasilieva • Moscow           | |    \---> 📍 Moscow (1934)           | |
|                                          | |           \                         | |
| 🔴 1941-1945 (Age 29-33) - Military      | |            v                        | |
|    Red Army Infantry • Baltic Front      | |          📍 Leningrad (1944)        | |
|                                          | +-------------------------------------+ |
| ⚫ 1989 (Age 77) - Death & Burial        | BIOGRAPHICAL NOTES                      |
|    22 Oct 1989 • Vaganovskoe Cemetery    | "Engineer and veteran of the Great      |
|                                          | Patriotic War..."                       |
+------------------------------------------------------------------------------------+
```

---

## 3. Structural Sections

### 3.1 Hero Identity Header
- **Portrait Avatar:** Large rounded-2xl avatar (`w-28 h-28`). If multiple photos exist, an indicator shows *"1 of 4 photos"*, allowing one-click avatar changes.
- **Name Typography:** Large Serif heading (`font-serif text-3xl font-semibold`): Given Name, Patronymic (*Отчество*), and Surname.
- **Life Span & Vital Dates:**
  - Formatted exact or approximate dates (`b. 14 Mar 1912 – d. 22 Oct 1989 • Age 77`).
  - Living person badge (`Living / Живущий`) with privacy masking when viewed without permissions.
- **Dynamic Kinship Pill:** Computes relationship relative to the viewer's **home person** ([16 §2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md#2-home-person)) (e.g., *"Your Great-Grandfather"* / *"Ваш Прадедушка"*). Hidden when no home person is set or no relationship exists.
- **Header Actions:**
  - `Edit Profile` (Editors/Admins only)
  - `View in Tree` (smoothly navigates to `/tree`, centering on this node)
  - `+ Add Event / Relative` quick buttons.
  - `⋯` menu: *Set as my home person*, *History* ([14](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md)), *Merge with…* ([15](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md)), *Mark private*, *Delete* (Editors+), *Copy link*.
  - Footer: `gid` (e.g. `I0042`), last modified by/when; ⚠️ data-quality badge for Editors.

### 3.2 Standard Tabbed Navigation
- **Tab 1: Chronology & Map (Default):**
  - **Left Column (Timeline):** Chronological stream of life milestones (Birth, Christening, Marriages, Residences, Military, Death, Burial) with age calculations; each event shows its free-text source (e.g. *"Source: Tver Church Metrical Book"*) and a 📄 icon if a scan is attached.
  - **Right Column (Life Map & Bio):**
    - Geographic migration map connecting life event coordinates with chronological path arrows.
    - Narrative biography markdown text block.
- **Tab 2: Family & Relations:**
  - Structured card groups:
    - **Parents & Siblings**
    - **Spouses & Children** (distinguishing biological, adopted, or step relationships).
- **Tab 3: Media & Documents:**
  - Photo gallery showing all images this person is tagged in.
  - Scanned documents, certificates, and censuses.
  - "Add Photo" / "Tag in Existing Photo" buttons.
- **Tab 4: Notes:**
  - Family stories, research remarks, transcriptions (Markdown); private notes marked 🔒.
  - *"Sources used"* summary: distinct event source texts for this person, each linking to its event.

### 3.3 Edit Mode (Editors / Admins)
Editing happens **in place** on the profile — no separate editor page, no nested modal stacks:
- **Identity:** `Edit Profile` turns the hero into a form (name parts, alt names, sex, living override, privacy). Save / Cancel sticky bar at the bottom; dirty-state guard on navigation.
- **Events:** each timeline row has an inline edit pencil → row expands into a form (type, date with live parse preview *"→ about 1890"*, place picker, description, source (free text with autocomplete), attach scan / photos). `+ Add Event` inserts a new expanded row at the correct chronological position.
- **Biography:** Markdown editor with preview toggle.
- **Family tab:** add/remove/reorder relationships as in [07 §4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/07-person-family-management.md#4-families--relationships).
- Every save sends `version`; conflicts open the comparison dialog; every save offers an *Undo* toast.

---

## 4. Definition of Done (DoD)

- [ ] **Unified Information Architecture:** All person profiles share the identical header, tabs, and dual-pane layout structure.
- [ ] **Interactive Geocoded Life Map:** Life events with coordinates automatically render numbered pins and connected migration polylines on the map.
- [ ] **Living Privacy Guard:** When an anonymous visitor accesses a living person's profile, sensitive dates, maps, and photos are masked with privacy placeholders.
- [ ] **Fast Navigation to Tree:** Clicking "View in Tree" opens the tree canvas and centers the camera on the person in <300ms.
- [ ] **Inline Editing:** All person data (identity, events, biography, relationships) can be edited from the profile without leaving the page.
- [ ] **Single Request Load:** Profile renders from one API call using `?include=` expansion.
