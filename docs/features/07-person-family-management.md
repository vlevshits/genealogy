# Feature Specification: Person & Family Management (Tree Editing)

## 1. Overview & Objective
Provide an intuitive, friction-free editing interface for modifying individuals, unions, and life milestones. The interface features fast **in-canvas quick-add actions** ("Add Father", "Add Child", "Add Spouse") and a comprehensive **Person Profile Page** supporting complex life events, flexible genealogy dates, places, and notes.

> Schema, date storage, notes, living rule, and delete semantics: [10-core-data-model.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md). Places: [12](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/12-places.md).

---

## 2. Person Profile Structure & Capabilities

### 2.1 Identity & Names
- **Structured Name Components:**
  - Given Name (Имя), Patronymic (Отчество), Surname (Фамилия), Maiden / Birth Surname (Девичья фамилия), Nickname, Prefix (e.g. *Dr.*, *Sir*), Suffix (e.g. *Jr.*, *III*).
- **Sex / Gender:** Male, Female, Other, Unknown.
- **Living Status:** Computed automatically ([10 §5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#5-living-status-rule-canonical-algorithm)); Editors may override (Living / Deceased / Automatic).
- **Alternate Names:** Optional list (married name, transliteration, alias) — searchable, shown under the main name.

### 2.2 Flexible Genealogy Date Parser (Bilingual EN / RU)
Genealogical research frequently involves uncertain or approximate dates. The date input component must parse and standardize natural text expressions according to GEDCOM conventions in both English and Russian:
- **Exact:** `14 Jul 1923` / `14 июля 1923 г.`
- **Approximate:** `about 1880` (`ABT 1880`) / `около 1880 г.` (`ок. 1880`)
- **Range:** `between 1914 and 1918` / `между 1914 и 1918 гг.`
- **Before / After:** `before May 1860` (`BEF MAY 1860`) / `до мая 1860` / `после 1945`
- **Estimated / Calculated:** `calculated 1910` (`CAL 1910`) / `вычислено 1910`
- **Historical Calendars:** Toggle for Old Style (Julian, ст. ст.) vs New Style (Gregorian, нов. ст.) for pre-1918 records.

### 2.3 Life Events & Milestones
Each person can have an ordered list of life events:
- **Core Events:** Birth, Baptism / Christening, Marriage, Divorce, Death, Burial / Cremation.
- **Biographical Events:** Residence, Education, Occupation, Military Service, Emigration, Naturalization, Religious Milestones (Bar/Bat Mitzvah, Confirmation).
- **Event Attributes:** Date, Place, Description, Age at Event (auto-calculated), **Source** (optional free text, e.g. *"Метрическая книга, 1912, л. 34"*, autocompleted from earlier entries), Notes, and Photos / attached scans.

### 2.4 Hierarchical Places & Mapping
- Places adhere to hierarchical structure: `[City/Town], [County/District], [State/Province], [Country]`.
- Support latitude/longitude coordinates.
- **Interactive Life Map:** Person profile renders an interactive Leaflet/MapLibre map showing the geographic journey across their lifetime (birthplace $\to$ migrations $\to$ burial).

---

## 3. Quick-Add In-Canvas Actions

To streamline tree expansion while browsing:
1. Hovering over a person card displays directional plus (`+`) buttons:
   - **`+ Top`:** Add Parents (Father / Mother).
   - **`+ Right`:** Add Spouse / Partner.
   - **`+ Bottom`:** Add Child.
   - **`+ Left/Right`:** Add Sibling.
2. Clicking any plus button opens a lightweight modal drawer requesting only:
   - First Name & Surname
   - Sex
   - Birth Date / Birth Place (optional)
   - Living Status
   - **Link existing person instead** toggle (search picker) — needed to connect two already-existing branches.
3. **Duplicate guard:** while typing, a hint suggests matching existing people (*"Пётр Щеглов (1868–1931) already exists — use?"*), see [15 §3.1](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md#31-finding-duplicates).
4. Smart defaults: child surname prefilled from father; patronymic suggested from father's given name (*Иван → Иванович / Ивановна* by sex); when adding a second parent, the existing family is reused.
5. Submitting instantly updates the graph layout and focuses the newly created relative without losing view context.

---

## 4. Families & Relationships
- A `Family` entity links:
  - Partner 1 and Partner 2 (role-neutral; supports single parents and same-sex couples)
  - Relationship Type: Married, Civil Union, Partners, Unknown. **Divorce is recorded as an event** (with date/place), not a type.
  - Family events: Engagement, Marriage, Divorce, Residence.
  - Children: Ordered list (drag to reorder, or *"Sort by birth date"*) with per-parent relation (`Birth`, `Adopted`, `Foster`, `Step`).
- **Editing relationships** (from profile *Family* tab): add/remove child, change child relation, reorder spouses, unlink a partner, and **change parents** (move a child to another family) — every action goes through cycle validation and is undoable ([14](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md)).
- **Deleting** a person/family shows an impact summary and follows [10 §6.2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#62-delete-behavior).

---

## 5. Definition of Done (DoD)

- [ ] **Quick Add from Canvas:** Editors can add parents, spouses, and children directly from the interactive tree view with immediate graph updates.
- [ ] **Date Parser:** Natural expressions like "about 1920", "bef 1880", and "between 1910 and 1915" are correctly parsed into structured ranges and exported cleanly to GEDCOM.
- [ ] **Full Profile Editing:** Editors can add and update custom life events (birth, marriage, death, residence, military).
- [ ] **Interactive Life Map:** A person's profile displays a chronological map showing markers and paths for all geocoded life events.
- [ ] **Relationship Validation:** Prevents circular parent-child loops (e.g. A cannot be their own ancestor).
- [ ] **Multiple Marriages:** Correctly represents and visually displays individuals with multiple consecutive or concurrent marriages and half-siblings.
- [ ] **Connect Branches:** Quick-add can link an existing person as parent/spouse/child instead of creating a new one.
- [ ] **Patronymic Suggestion:** Adding a child of a father with a known given name pre-fills a correctly gendered patronymic (editable).
- [ ] **Event Source & Scan:** Any event can store a free-text source and an attached scan, both shown on the timeline.
