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

### 4.1 Family Unit & Union Types
A `Family` entity links partners and children:
- **Partner 1 & Partner 2:** Role-neutral references (`partner1_id`, `partner2_id`), supporting heterosexual unions, same-sex couples, and single-parent households (where `partner2_id IS NULL`).
- **Relationship Type (`relationship_type`):**
  - `married` ("Married" / "В браке")
  - `civil_union` ("Civil Union" / "Гражданский союз")
  - `partners` ("Partners" / "Партнёры")
  - `unknown` ("Unknown" / "Не указано")
- **Editing Union Type:** Editors and Admins can switch the union type via a dropdown on the union card header in the profile *Family* tab (`PATCH /api/v1/families/{id}` with `{ "relationship_type": "..." }`).
- **No Empty Marriage Events Required:** Informal partnerships (`partners`) or unions without formal marriage documentation do not require empty or stub `marriage` events.
- **Divorce is NOT a Union Type:** Divorce is modeled strictly as a life event (`event_type = 'divorce'`), see [§4.2](#42-divorce-semantics-vs-mistaken-link-unlink-vs-deceased-partner).

### 4.2 Divorce Semantics vs. Mistaken-Link Unlink vs. Deceased Partner
Genealogical clarity requires distinguishing between legal dissolution, data-entry correction, and widowhood:

| Scenario | Data Model Change | UI & Profile Presentation | Tree Canvas Visual |
| :--- | :--- | :--- | :--- |
| **Divorce (Расторжение брака)** | `life_events` row added: `event_type = 'divorce'`, `family_id = families.id`, date, place, source. Family container and both partners **retained**. | Union header displays a "Divorced" badge (`Badge variant="neutral"`), union date span (e.g. *1934 – 1952 (Divorced)*), and divorce event in timeline. Children remain under this union. | Ochre marriage bar displays double diagonal slash hatch marks (`//`) at center (or dashed ochre line). Children distribution bus remains intact beneath anchor. |
| **Mistaken-Link Unlink (Ошибочная связь)** | Person removed from `family_children` or partner slot cleared. If the family has 0 children and $\le 1$ partner, the family container is **dissolved** (deleted). | Confirmation dialog warns what will remain. No divorce event created. Person profiles are **never** deleted. | Connector line disappears. Tree recalculates without that edge. |
| **Deceased Partner (Овдовение)** | Partner has a `death` event in `life_events`. No change to family row. | Lifespan shows death year (e.g. `† 1989`). Union is not labeled divorced. | Standard ochre bar with deceased partner card (`†` indicator). |

### 4.3 Unlinking Relationships (Parent, Child, Spouse)
Unlinking disconnects relationships created by mistake without deleting any person profiles:

1. **Unlink Parent (Person's own parents):**
   - Available on each parent card in the profile *Family* tab (Parents & Siblings section).
   - **Single-Parent Family:** Removing the child from `family_children`. If the family is left with 0 children and $\le 1$ partner, the family record is automatically deleted.
   - **Two-Parent Couple Family:** Clicking "Unlink" on a parent opens a confirmation modal offering two explicit choices:
     - *Unlink from both parents:* Removes the child from this family container entirely.
     - *Unlink this parent only:* Retains the child with the other parent by atomically moving the child to a single-parent family container for the other parent (reusing or creating `(OtherParent, NULL)`).
2. **Unlink Child:**
   - Available on child cards under each union. Removes `(family_id, child_id)` from `family_children`.
   - If the family is left with 0 children and has only 1 partner (`partner2_id IS NULL`), the empty family container is dissolved.
3. **Unlink Partner / Spouse:**
   - Available on the union header.
   - If the union has children, clearing a partner retains the family container as a single-parent family, preserving the parent-child lineages for all children.
   - If the union has 0 children, detaching the partner dissolves the childless family container.

### 4.4 Moving a Child to Another Family ("Change Parents") & Cycle Validation
When a child was attached to the wrong parents or biological parents are identified:
- **Dedicated Atomic Endpoint:** `POST /api/v1/persons/{id}/move-to-family` accepts:
  ```json
  {
    "source_family_id": "UUID",
    "target_family_id": "UUID",
    "rel_to_partner1": "birth",
    "rel_to_partner2": "birth"
  }
  ```
- **Single Database Transaction:**
  1. Validates that `id` is a child in `source_family_id`.
  2. **Cycle Validation (Hard Guard):** Runs recursive ancestor traversal on `target_family_id` partners. If the child is an ancestor of either partner, or if the child is equal to either partner, the request is rejected with `400 Bad Request` (`code: "relationship_cycle_detected"`).
  3. Removes `(source_family_id, child_id)` from `family_children`.
  4. Dissolves `source_family_id` if left childless and $\le 1$ partner.
  5. Inserts `(target_family_id, child_id)` with `rel_to_partner1`, `rel_to_partner2`, and `sort_order = max + 1`.
  6. Writes a single `change_log` entry (`tx_label: "family.move_child"`), returning `{ "tx_id": "...", "person": ... }`.
- **UI Experience:** "Change parents…" button on the child card or in the Parents section opens a searchable picker dialog for an existing family/couple, displays a dry-run confirmation with partner names, and executes atomically with 1-click Undo.

### 4.5 Child Relationship Types (Birth, Adopted, Foster, Step)
Each parent-child relationship records a distinct lineage type per parent:
- **Values:** `birth` (biological), `adopted`, `foster`, `step`, `unknown`.
- **Per-Parent Specificity:** `family_children.rel_to_partner1` and `rel_to_partner2` are configured independently:
  - Example: Child is biological (`birth`) to Father and `step` to Step-Mother in a blended family.
  - Example: Child is `adopted` by both parents.
- **UI Editor:**
  - Interactive dropdown selector on child cards (under Spouses & Children) and on parent cards (under Parents).
  - Changing relation calls `PATCH /api/v1/families/{id}/children/{child_id}`.
- **Downstream Effects:**
  - **GEDCOM Export:** Maps to `FAMC.PEDI` (`birth`, `adopted`, `foster`).
  - **Tree Canvas Lines:** Adoptive and foster lines render with distinct styling (`strokeDasharray="4 4"` or double line), while step relationships visually isolate stepparent lines from biological distribution buses per [04 §2.7.3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#273-family-partitioned-children-buses--child-to-marriage-association).

### 4.6 Ordering of Spouses and Children
To ensure authentic chronological and preferred presentation:
- **Children Ordering:**
  - Stored in `family_children.sort_order INT`.
  - **Auto-Sort Button:** *"Sort by birth date"* automatically sorts children chronologically by `birth_sort` (null dates at the end), assigning sequential `sort_order` indices.
  - **Manual Drag Reordering:** Drag handle (`GripVertical`) on each child card allows custom reordering, submitting `PUT /api/v1/families/{id}/children/reorder` with `{ "child_ids": [id1, id2, ...] }`.
- **Spouses Ordering:**
  - Display default: Chronological by earliest recorded marriage event date (`marriage.date`); fallback to earliest child birth date; fallback to creation order.
  - Manual override supported via spouse reordering controls, updating `families.sort_order` or partner position.

### 4.7 Single-Parent Families & "Other Parent Unknown" (No Placeholder People)
Single parents and unknown parentage are modeled cleanly without corrupting the tree with fake individuals:
- **Rule:** A single-parent family is a `Family` entity with `partner1_id = <person_id>` and `partner2_id = NULL`.
- **No Placeholder Persons:** Never create dummy "Unknown Father", "N.N.", or "?" person records.
- **UI Presentation:**
  - In Spouses & Children: Rendered with header *"Children with unknown partner"* / *"Дети от неизвестного партнёра"*. Includes an *"Add / Link other parent"* action to connect an existing or new person directly into `partner2_id`.
  - In Parents: If only one parent is recorded, the other slot displays a clean UI placeholder card (*"Mother unknown"* / *"Father unknown"*) with an *"Add Mother"* / *"Add Father"* CTA.
- **Tree Canvas:** Drops a single vertical stem directly from the bottom-center of the single parent's card to the child distribution bus runner with zero dummy cards ([04 §2.7.3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#273-family-partitioned-children-buses--child-to-marriage-association)).

### 4.8 Half-Sibling and Step-Sibling Classification & Kinship Labels
The system dynamically computes and labels sibling relationships from parental links:
- **Full Sibling (Родной брат / Родная сестра):** Shares both biological parents (`rel_to_partner = 'birth'` in the same family or matching parents).
- **Half-Sibling:** Shares exactly one biological parent:
  - **Paternal Half-Sibling:** Shared father, distinct mothers.
    - EN: *Half-brother (paternal)* / *Half-sister (paternal)*
    - RU: *Единокровный брат* / *Единокровная сестра*
  - **Maternal Half-Sibling:** Shared mother, distinct fathers.
    - EN: *Half-brother (maternal)* / *Half-sister (maternal)*
    - RU: *Единоутробный брат* / *Единоутробная сестра*
- **Step-Sibling (Сводный брат / Сводная сестра):** Children of a stepparent from another union where NO biological parent is shared.
  - EN: *Step-brother* / *Step-sister*
  - RU: *Сводный брат* / *Сводная сестра*
- **UI Badge:** Rendered as a neutral badge on each sibling card in the profile *Family* tab.

### 4.9 Merge of Duplicate Unions on Person Merge
When two person records are merged (see [15 §3.2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md#32-merge-flow-peoplemergetargetsource-admin-only)):
- If both individuals possess unions with the same partner (or both are single-parent families with no second partner):
  - Consolidate the two family records into a single family.
  - **Children:** Union of child sets; duplicate child links collapsed (preserving explicit relationship type over `unknown`/`birth` and minimal `sort_order`).
  - **Events:** Re-point all family life events (`marriage`, `divorce`, `residence`) to surviving family; collapse exact duplicates.
  - **Notes & Media:** Re-point to surviving family.
  - Delete redundant family row.
  - Entire merge executed in a single atomic SQL transaction logged in `change_log`.

### 4.10 Atomic History Transactions & 1-Click Undo for All Relationship Actions
Every relationship mutation is logged in `change_log` inside the same database transaction:
- Endpoints return `tx_id` in response payloads.
- UI triggers `useUndoToast` displaying a 6-second toast notification:
  - *"Parent unlinked. [Undo]"*
  - *"Child moved to new family. [Undo]"*
  - *"Children reordered. [Undo]"*
- Clicking *Undo* calls `POST /api/v1/history/tx/{tx_id}/undo`, completely restoring prior family links and child records.

---

## 5. API Endpoints

All mutating endpoints require CSRF header `X-Requested-With: genealogy` and authenticated `Editor` or `Admin` role.

| Method | Endpoint | Description | Error Codes |
| :--- | :--- | :--- | :--- |
| `POST` | `/api/v1/persons/quick-add` | Create relative and link family atomically | `validation_failed`, `person_not_found`, `family_cycle_detected` |
| `POST` | `/api/v1/persons/{id}/move-to-family` | Atomically move child to another family | `person_not_found`, `family_not_found`, `relationship_cycle_detected`, `conflict` |
| `PATCH` | `/api/v1/families/{id}` | Update union type (`relationship_type`), partners | `family_not_found`, `invalid_partner`, `conflict` |
| `DELETE` | `/api/v1/families/{id}/partners/{person_id}` | Detach partner from union (dissolves if childless) | `family_not_found`, `partner_not_found`, `conflict` |
| `DELETE` | `/api/v1/families/{id}/children/{child_id}` | Detach child from family (dissolves if childless single-parent) | `family_not_found`, `child_not_found`, `conflict` |
| `PATCH` | `/api/v1/families/{id}/children/{child_id}` | Update relation types (`rel_to_partner1`, `rel_to_partner2`) | `family_not_found`, `child_not_found`, `invalid_relation`, `conflict` |
| `PUT` | `/api/v1/families/{id}/children/reorder` | Update `sort_order` of all children in family | `family_not_found`, `invalid_child_list`, `conflict` |
| `POST` | `/api/v1/families/{id}/events` | Add family life event (e.g. `marriage`, `divorce`) | `family_not_found`, `invalid_event_type`, `conflict` |

---

## 6. Permissions & Privacy (RBAC Matrix)

| Action | Anonymous (Public) | Viewer | Editor | Admin |
| :--- | :---: | :---: | :---: | :---: |
| View families, spouses, children | Redacted (living shielded) | Full read | Full read | Full read |
| Quick-add relative | ❌ | ❌ | ✅ | ✅ |
| Change union type / child relation | ❌ | ❌ | ✅ | ✅ |
| Unlink parent / child / spouse | ❌ | ❌ | ✅ | ✅ |
| Move child to another family | ❌ | ❌ | ✅ | ✅ |
| Reorder children / spouses | ❌ | ❌ | ✅ | ✅ |
| Record divorce event | ❌ | ❌ | ✅ | ✅ |
| Undo relationship mutation | ❌ | ❌ | ✅ (own tx) | ✅ (all) |

---

## 7. Localization (EN / RU Specifics)

- **Relationship Types:**
  - `married`: Married / В браке
  - `civil_union`: Civil Union / Гражданский союз
  - `partners`: Partners / Партнёры
  - `unknown`: Unknown / Не указано
- **Pedigree / Child Relations:**
  - `birth`: Biological / Родной
  - `adopted`: Adopted / Усыновлённый (Удочерённая)
  - `foster`: Foster / Приёмный (Приёмная)
  - `step`: Stepchild / Пасынок (Падчерица)
- **Kinship Sibling Terms:**
  - Paternal half-sibling: Half-brother (paternal) / Единокровный брат; Half-sister (paternal) / Единокровная сестра
  - Maternal half-sibling: Half-brother (maternal) / Единоутробный брат; Half-sister (maternal) / Единоутробная сестра
  - Step-sibling: Step-brother / Сводный брат; Step-sister / Сводная сестра
- **Slavic Patronymics:** Auto-derived from father's given name using Russian grammatical suffixes (`-ович` / `-евич` for male, `-овна` / `-евна` for female).

---

## 8. Edge Cases & Non-Goals

- **Self-Ancestry Rejection:** Cycle validation recursively prevents any operation that would make an individual their own ancestor or descendant.
- **Multiple Families per Child:** Supported (e.g. birth family and adoptive family). Moving a child requires specifying `source_family_id` when multiple exist.
- **Dissolution Safety:** Families with remaining children are NEVER deleted upon spouse unlinking or divorce; they transition into single-parent families.
- **Non-Goals:**
  - Complex multi-partner communal unions beyond two partners per family record (modeled as successive 2-partner families).
  - Bulk branch detachment to a separate database (Phase 3).

---

## 9. Definition of Done (DoD)

- [ ] **Quick Add from Canvas:** Editors can add parents, spouses, and children directly from the interactive tree view with immediate graph updates.
- [ ] **Parent Unlink:** Parent cards in the Family tab feature an Unlink button; unlinking from a couple family offers unlinking from both parents or moving to a single-parent family; childless families with $\le 1$ partner dissolve cleanly without deleting person profiles.
- [ ] **Atomic Move Child:** `POST /api/v1/persons/{id}/move-to-family` moves a child between families in a single transaction, enforces cycle validation (rejecting descendant/self parentage with `relationship_cycle_detected`), and provides 1-click Undo.
- [ ] **Child Relation Editor:** Dropdown on child and parent cards allows selecting `birth`, `adopted`, `foster`, or `step` per parent, saving via `PATCH /families/{id}/children/{child_id}`.
- [ ] **Union Type Switcher:** Union header in Family tab provides a dropdown to set `married`, `civil_union`, `partners`, or `unknown`.
- [ ] **Children Reordering:** Family card provides a *"Sort by birth date"* button and drag-and-drop handles updating `family_children.sort_order`.
- [ ] **Divorce Semantics:** Adding a `divorce` event marks the union as ended in profile and tree canvas, displays a "Divorced" badge with date/place, and keeps all children intact under the union.
- [ ] **Sibling Kinship Badges:** Sibling cards display localized badges distinguishing full, paternal half (*единокровный*), maternal half (*единоутробный*), and step (*сводный*) siblings.
- [ ] **No Placeholder Persons for Unknown Parents:** Single-parent unions represent unknown partners cleanly; no fake person rows ("Unknown", "N.N.") are generated.
- [ ] **Duplicate Union Merge:** Merging two duplicate persons merges their duplicate unions with the same partner, consolidating children, family events, notes, and media.
- [ ] **Undo for Link Actions:** All relationship mutations return `tx_id` and display an interactive 6-second Undo toast.
- [ ] **Relationship Validation:** Prevents circular parent-child loops (e.g. A cannot be their own ancestor).
- [ ] **Multiple Marriages:** Correctly represents and visually displays individuals with multiple consecutive or concurrent marriages and half-siblings.
- [ ] **Connect Branches:** Quick-add can link an existing person as parent/spouse/child via searchable picker instead of creating a new one.
- [ ] **Patronymic Suggestion:** Adding a child of a father with a known given name pre-fills a correctly gendered patronymic (editable).
- [ ] **Date Parser:** Natural expressions like "about 1920", "bef 1880", and "between 1910 and 1915" parse into structured ranges.
