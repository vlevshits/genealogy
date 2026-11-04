# Feature Specification: Data Quality Checks, In-Tree Person Merge & Place Hierarchy Auto-Fix

## 1. Overview & Objective
Family trees accumulate mistakes: typos in years, impossible ages, duplicates created when relatives enter the same branch independently, and flat, unparsed place records from legacy GEDCOM imports. The application provides a calm, unified data maintenance environment:

1. **Inline soft warnings** while editing (never blocking).
2. A centralized **Data Quality** page (`/quality`, Admin-only) with deterministic validation checks.
3. **Relationship-centric duplicate detection & merge** for people within the tree, prioritizing relational evidence (parents, spouses, siblings, children) and focusing on detached subtrees / unconnected singletons while strictly disqualifying biologically related kin (father/son, siblings, spouses).
4. **Automated place hierarchy resolution & gazetteer quality**, automatically converting unparsed comma-separated strings, orphan localities, and coordinate-proximate places into clean administrative trees (`Country` → `Region` → `District` → `City`/`Village`) with dry-run previews, batch auto-fixing, and atomic undo.

---

## 2. Validation Rules

### 2.1 Hard Errors (block save)
Defined in [10-core-data-model.md §6.1](10-core-data-model.md#61-hard-constraints-save-is-rejected): ancestor cycles, self-partnering, place hierarchy cycles (`ensure_no_place_cycle`), malformed events.

### 2.2 Soft Warnings (shown inline + on Data Quality page)
Fixed set; thresholds configurable via environment variables:

| Code | Entity | Category | Rule Description | Default threshold | Severity |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `DEATH_BEFORE_BIRTH` | Person | Chronology | Death or burial date precedes birth date | — | Critical |
| `TOO_OLD` | Person | Plausibility | Age at death or (if living) current age above limit | 110 years | Warning |
| `PARENT_TOO_YOUNG` | Person | Family age | Parent younger than N at child's birth | 13 years | Warning |
| `MOTHER_TOO_OLD` | Person | Family age | Mother older than N at child's birth | 55 years | Warning |
| `BORN_AFTER_PARENT_DEATH` | Person | Chronology | Child born >1 year after father's death or after mother's death | — | Critical |
| `MARRIED_TOO_YOUNG` | Person | Family age | Partner younger than N at marriage | 14 years | Warning |
| `EVENT_AFTER_DEATH` | Person | Chronology | Non-burial/non-cremation life event dated after death | — | Warning |
| `POSSIBLE_DUPLICATE` | Person | Duplicates | Relationship-weighted candidate score ≥ 80% (see §3.1) | 80% | Warning |
| `MISSING_BIRTH` | Person | Completeness | Deceased person with no birth date and no death date | — | Info |
| `UNCONNECTED` | Person | Completeness | Person with no family links at all (isolated node) | — | Info |
| `PLACE_UNPARSED_PATH` | Place | Places | Place name contains multi-part comma-separated jurisdictions with missing parent hierarchy | — | Warning |
| `PLACE_ORPHAN_HIERARCHY` | Place | Places | Locality or district place missing `parent_id` linkable to an existing parent by name/coordinates | — | Warning |
| `PLACE_TYPE_MISMATCH` | Place | Places | Place type is `other`, but name contains clear administrative keyword (`губерния`, `область`, `уезд`, `район`, etc.) | — | Info |
| `PLACE_DUPLICATE_CANDIDATE` | Place | Places | Duplicate place nodes under same parent or equivalent jurisdiction paths | — | Warning |

- Approximate dates use their widest plausible range — `abt 1880` never triggers a warning that a ±5 year window would avoid.
- Each warning can be **dismissed per record** ("This is correct") and stays dismissed (`dismissed_warnings(entity_id, code, user_id)`).

### 2.3 Inline Presentation
- In edit forms, a yellow hint under the field: *"Mother would be 9 years old at this birth. Check the date?"* — save is still permitted.
- On profiles, a discreet ⚠️ badge in the header opens the list of warnings (Admins only).

---

## 3. Duplicate Detection & Relationship-Centric Person Merge

### 3.1 Relationship-Centric Candidate Matching Engine
In family trees, names naturally recur across generations (e.g. sons named after fathers or grandfathers: *"John Smith Sr."* and *"John Smith Jr."*, or Russian naming patterns where patronymics repeat). Name similarity alone is insufficient and produces false positives.

The duplicate detection engine prioritizes **structural genealogical context** and **detached subtree isolation**:

#### 3.1.1 Structural Non-Duplicate Disqualification (Kinship & Graph Guards)
Two records in the live tree are **immediately disqualified** from duplicate consideration (`score = 0`) if any of the following hold:
1. **Direct Ancestor or Descendant:** Person A is an ancestor or descendant of Person B (parent, child, grandparent, grandchild).
2. **Spouses / Partners:** Person A and Person B share or have shared a union/marriage.
3. **Distinct Siblings in the Same Family:** Person A and Person B are confirmed children of the same family with distinct birth dates, different sexes, or distinct given names/patronymics.
4. **Distinct Confirmed Biological Parents:** Both Person A and Person B have confirmed, non-matching biological parents in the same connected tree component (where the parents themselves are distinct individuals).
5. **Non-Overlapping Lifespans in Connected Component:** Both records have confirmed vital date ranges that do not overlap (e.g. Person A died in 1890, Person B was born in 1915).

#### 3.1.2 Relational Scoring & Weighting
Within a cleanly created or imported single connected tree component, duplicate candidates are suppressed unless high-weight relational overlap is present. Duplicate detection specifically targets:
- **Detached subtrees and unconnected singletons** (individuals added without complete parental connections that match existing tree nodes).
- **Secondary tree imports / branch reconciliations** (covered in [09](09-tree-import-merge-reconciliation.md)).

| Factor | Weight | Scoring Condition |
| :--- | :---: | :--- |
| **Parents Alignment** | 35% | Matching Father and/or Mother (exact or candidate match), or child's patronymic matches father's given name. |
| **Spouse(s) Alignment** | 25% | Shared spouse name, marriage date, or marriage place. |
| **Children Set Overlap** | 20% | Shared children with matching given names and birth years. |
| **Vital Dates & Places** | 20% | Overlapping birth/death years ($\pm 2$ years tolerance) and matching birth/death place. |
| **Name Similarity** | Capped at 40% | Levenshtein $\le 2$, Cyrillic orthography / Metaphone phonetic match, Slavic stem folding. **Name similarity alone CANNOT exceed 40%** without relational or vital date confirmation, preventing father/son false matches. |

- **Detached Subtree Boost:** When at least one candidate belongs to an unconnected singleton or a disconnected tree component, the engine relaxes parent requirements and evaluates spouse, sibling, and generational vital date windows.
- Background job runs nightly and on-demand; cached in `duplicate_candidates(person_a, person_b, score, computed_at)`.
- **Real-time check on Quick-Add:** When an Editor or Admin adds a person, if candidate score $\ge 80\%$, the drawer prompts: *"Possible match: Пётр Щеглов (1868–1931) — Use existing?"*.

### 3.2 Merge Flow (`/people/merge?target=…&source=…`, Admin only)
Entry points: Data Quality page, People Directory (*Compare / Merge* action), person profile menu (`⋯ → Merge with…`).

```
+-----------------------------------------------------------------------------------+
| Merge People                                                                      |
+----------------------+------------------------------+-----------------------------+
| Field                | Пётр Щеглов (I0012)          | Петр Щеглов (I0388)         |
+----------------------+------------------------------+-----------------------------+
| Name                 | (•) Пётр Фёдорович Щеглов    | ( ) Петр Щеглов  → alt name |
| Birth                | ( ) abt 1868                 | (•) 3 Feb 1868, Тверь       |
| Death                | (•) 1931                     | ( ) —                       |
| Events               | 4  ─────── all kept, exact duplicates collapsed ─────── 2  |
| Parents / Families   | 1 family                     | 1 family → merged if same   |
| Photos / Notes       | 3 / 2                        | 1 / 1  → all kept           |
+----------------------+------------------------------+-----------------------------+
| Result keeps ID I0012.   [ Cancel ]                         [ Merge people ]      |
+-----------------------------------------------------------------------------------+
```

Merge execution rules:
- **Scalar fields:** User picks per field; the unselected name is automatically preserved in `individual_alt_names`.
- **Events, notes, media links, face tags:** Union of all items; exact duplicates (same type + date + place) collapsed.
- **Families:** If both are children in families with the same parents $\to$ families merged; if both are partners with the same spouse $\to$ families merged (children unioned). Otherwise families re-point to the surviving person.
- `users.person_id` and `home_person_id` re-pointed.
- The losing `gid` is recorded as an alias so old links and bookmarks redirect cleanly.
- Executed in a single atomic SQL transaction, logged as one `change_log` entry $\to$ fully undoable ([14](14-change-history-concurrency.md)).
- Cycle check runs on the merged result; merge is rejected if an ancestor cycle would be introduced.

---

## 4. Place Hierarchy Quality Engine & Automated Restructuring

### 4.1 Problem & Failure Modes
In genealogy datasets, places often arrive via legacy GEDCOM imports or rapid manual entry in unparsed or flat states:
1. **Unparsed Comma-Separated Names:** A place's leaf `name` contains the full jurisdiction string (e.g. `name = "Тверь, Тверская губерния, Российская империя"` or `"г. Тверь, Тверской уезд"`) with `parent_id = NULL`.
2. **Orphan / Flat Places:** Locality or district records have `parent_id = NULL` despite existing parent regions (e.g. `"Тверской уезд"` not linked to `"Тверская губерния"`).
3. **Unclassified Place Types:** Places imported with `place_type = "other"`, despite explicit administrative terms (`губерния`, `область`, `уезд`, `район`, `волость`, `город`, `село`, `деревня`, `Governorate`, `Oblast`, `District`, `County`, `City`, `Village`).
4. **Duplicate Place Nodes:** Redundant records representing the same jurisdiction at different hierarchy levels (e.g. `"Тверская губ."` and `"Тверская губерния"`).
5. **Unused Spatial Clues:** Places possessing coordinates (`latitude`, `longitude`) that fall within known parent boundaries or near cluster centroids but lack parent hierarchy links.

### 4.2 Local Deterministic Heuristics & Spatial Containment
The hierarchy engine runs **100% locally on the self-hosted backend** with zero external network requests, preserving complete privacy (`GEOCODING_ENABLED=false` friendly).

#### 4.2.1 Right-to-Left Path Decomposition & Prefix Stripping
- Splits multi-part strings by commas, semicolons, or slashes.
- Strips administrative abbreviation prefixes before matching:
  - Russian: `г. `, `гор. `, `с. `, `д. `, `дер. `, `пос. `, `п. `, `ст. `, `х. `, `хут. `, `вол. `, `волость `, `уезд `, `г-я `, `губ. `, `губерния `, `обл. `, `область `, `р-н `, `район `.
  - English: `City of `, `Town of `, `Village of `, `Co. `, `County`, `Gov. `, `Governorate`, `Dist. `, `District`, `Twp. `, `Township`.
- Decomposes top-down from right-most jurisdiction (Country/Region) down to leaf locality (City/Village).

#### 4.2.2 Keyword-Based Place Type Inference
Infers canonical `place_type` from administrative keywords:
- **`country`:** Empire, Kingdom, Republic, Federation, State(s) of, Россия, Российская империя, СССР, Russian Empire, USSR, Poland.
- **`region`:** губерния, область, воеводство, governorate, oblast, province, state, krai, край, voivodeship.
- **`district`:** уезд, район, округ, волость, uyezd, raion, district, county, parish, okrug.
- **`city`:** город, г., city, town, borough.
- **`village`:** село, деревня, хутор, местечко, посёлок, станица, с., д., пос., village, hamlet, shtetl, settlement.
- **`cemetery`:** кладбище, погост, cemetery, graveyard, burial ground.
- **`parish`:** приход, parish, churchyard.

#### 4.2.3 Spatial Proximity & Bounding Containment (Haversine Formula)
When an orphan place has coordinates (`lat_a`, `lon_a`):
1. **Parent Centroid Distance:** Computes great-circle distance $d$ to candidate parent jurisdictions:
   - Candidate District: $d \le 40\text{ km}$.
   - Candidate Region/Governorate: $d \le 200\text{ km}$.
2. **Child Cluster Bounding Envelope:** If an existing parent region has other child localities with coordinates, the engine computes their geographic bounding box $[lat_{min}, lat_{max}] \times [lon_{min}, lon_{max}]$ with a 15% margin. If the orphan locality falls inside this envelope, confidence increases.
3. **Combined Text & Spatial Evidence:** An orphan place with matching region stem (e.g. *"Новоузенский"*) AND coordinates inside the candidate parent's bounding box receives **High Confidence** ($\ge 90\%$).

#### 4.2.4 Duplicate Node Consolidation
When decomposing paths or linking orphans, if a matching place node already exists under the target parent:
- The engine proposes **Consolidate into Existing Node** rather than creating a duplicate.
- Re-points all associated `life_events` and child places to the surviving node.
- Preserves the unparsed or alternate string in `place_alt_names` with `note: "Consolidated during hierarchy auto-fix"`.

### 4.3 Confidence Tiers & Recommendations

| Confidence Tier | Score Range | Criteria | Recommended Action |
| :--- | :---: | :--- | :--- |
| **High Confidence** | $\ge 85\%$ | Unambiguous comma-separated path matching existing tree parents, or orphan place with matching jurisdiction stem + coordinate containment. | Eligible for **"Auto-fix High Confidence"** batch execution or 1-click single card fix. |
| **Medium Confidence** | $60\% - 84\%$ | Candidate parent found by coordinate proximity alone (without name stem), or multiple parent candidates exist (e.g. duplicate historical governorates). | Presented in card list; requires Admin review and parent selection before applying. |
| **Needs Review / Low** | $< 60\%$ | Unrecognized locality name without coordinates and without clear administrative suffix. | Admin manual selection via Place Picker combobox. |

### 4.4 Automated Restructuring & Undo Semantics
- Executed inside a single atomic database transaction.
- Intermediate parent places (`Region`, `District`) are created automatically if they do not yet exist, reusing existing nodes whenever possible.
- Leaf place `name` is updated to the clean locality name.
- The original unparsed path is archived in `place_alt_names` (e.g. `note: "Original unparsed path"`), ensuring zero loss of raw imported strings and 100% search compatibility.
- `full_name` is recomputed down the entire subtree.
- All `life_events` remain attached to the clean leaf place.
- Logged in `change_log` under a single transaction ID $\to$ generates an instant Undo toast in the UI (*"12 place hierarchies updated · [Undo]"*) and can be fully reverted from `/history`.

---

## 5. Data Quality Page Layout (`/quality`, Admin only)
Access to `/quality` is restricted strictly to **Administrators** (`RequireRole::<Admin>`). Viewers and Editors receive `403 Forbidden`.

### 5.1 Page Header & Summary Chips
- Header: *Data Quality & Tree Maintenance*.
- Subtitle: *Automated chronological validation, relationship-centric duplicate detection, and place hierarchy normalization.*
- Summary Chips:
  - `⚠️ 8 warnings` (chronology/family age)
  - `👥 2 duplicate candidates`
  - `📍 14 place hierarchy fixes`
  - `ℹ️ 42 missing birth dates`

### 5.2 Filter Tabs
1. **All Warnings** (chronological and structural plausibility rules).
2. **Critical** (`DEATH_BEFORE_BIRTH`, `BORN_AFTER_PARENT_DEATH`).
3. **Warnings** (`TOO_OLD`, `PARENT_TOO_YOUNG`, `MOTHER_TOO_OLD`, `MARRIED_TOO_YOUNG`, `EVENT_AFTER_DEATH`).
4. **Duplicates** (Candidate duplicate person pairs with relationship scores and *Compare & Merge* button).
5. **Place Hierarchy** (Dedicated place hierarchy normalization queue).

### 5.3 Place Hierarchy Sub-View (`/quality?tab=places_hierarchy`)
```
+---------------------------------------------------------------------------------------------------------+
| Data Quality & Tree Maintenance                                                                          |
| [ 8 warnings ] [ 2 duplicate candidates ] [ 14 place hierarchy fixes ] [ 42 missing dates ]             |
+---------------------------------------------------------------------------------------------------------+
| [ All Warnings ] [ Critical ] [ Warnings ] [ Duplicates (2) ] [ Place Hierarchy (14) ]                 |
+---------------------------------------------------------------------------------------------------------+
| Filter: [ All issues ] [ Unparsed paths (9) ] [ Orphan places (4) ] [ Redundant duplicates (1) ]        |
|                                                     [ ⚡ Auto-fix All High Confidence (12) ]            |
+---------------------------------------------------------------------------------------------------------+
| CARD 1: High Confidence (95%) — Unparsed Comma-Separated Path                                           |
| Current Place:                                     Proposed Hierarchy:                                  |
| "Тверь, Тверская губерния, Российская империя"     Российская империя (country)                         |
| Type: other · Events: 6 · Coords: 56.85, 35.91      └── Тверская губерния (region)                      |
|                                                          └── Тверь (city) [Clean Leaf]                  |
| Rule: Right-to-left jurisdiction decomposition. Original raw string archived to alt names.             |
| [ Dismiss ] [ Customize / Pick Parent ]                                                    [ Apply Fix ]|
+---------------------------------------------------------------------------------------------------------+
| CARD 2: High Confidence (90%) — Orphan Locality Linked by Coordinate Proximity & Stem                   |
| Current Place:                                     Proposed Hierarchy:                                  |
| "Покровка" (village, P0005)                         Самарская губерния (region)                         |
| Parent: None · Events: 4 · Coords: 53.22, 50.18      └── Новоузенский уезд (district)                    |
|                                                              └── Покровка (village)                     |
| Rule: Coordinates inside Новоузенский уезд cluster envelope (d = 12 km).                                 |
| [ Dismiss ] [ Customize / Pick Parent ]                                                    [ Apply Fix ]|
+---------------------------------------------------------------------------------------------------------+
| CARD 3: Medium Confidence (75%) — Ambiguous Parent Region                                               |
| Current Place:                                     Proposed Candidates:                                 |
| "Ивановка" (village) · No coordinates               (•) Тверской уезд, Тверская губерния                 |
| Parent: None · Events: 1                            ( ) Самарский уезд, Самарская губерния               |
| [ Dismiss ] [ Select Candidate & Apply ]                                                                |
+---------------------------------------------------------------------------------------------------------+
```

---

## 6. Permissions & Privacy
- **Strict Role Isolation:** `/quality` and all associated endpoints (`/api/v1/quality/*`, `/api/v1/people/merge*`) are restricted strictly to **Administrators** (`RequireAdmin`). Viewers and Editors attempting access receive `403 Forbidden`.
- **Public Masking Parity:** No data quality metrics, warnings, unparsed place strings, or merge candidates are accessible in unauthenticated public guest mode.
- **Zero External Egress:** Place hierarchy normalization and coordinate proximity calculations run completely on-device without outbound API calls.

---

## 7. API Endpoints

| Method | Endpoint | Access Level | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/v1/quality/summary` | Admin only | Summary counts per rule, severity, duplicate count, and place hierarchy issue count. |
| `GET` | `/api/v1/quality/warnings` | Admin only | Paginated soft warnings list (`?rule=&category=&severity=&cursor=`). |
| `POST` | `/api/v1/quality/warnings/dismiss` | Admin only | Dismiss warning (`{ entity_id, code }`). |
| `GET` | `/api/v1/quality/duplicates` | Admin only | Candidate duplicate pairs with relationship scores and reason breakdown. |
| `POST` | `/api/v1/quality/duplicates/not-duplicate`| Admin only | Mark pair as distinct individuals (`{ person_a, person_b }`). |
| `GET` | `/api/v1/people/match?given=&surname=&birth=` | Editors & Admins | Quick-add real-time candidate duplicate check. |
| `GET` | `/api/v1/people/merge/preview?a=&b=` | Admin only | Side-by-side field comparison payload for person merge. |
| `POST` | `/api/v1/people/merge` | Admin only | Execute atomic person merge (`{ keep_id, merge_id, field_choices }`). |
| `GET` | `/api/v1/quality/places/analysis` | Admin only | Inspects all places; returns categorized hierarchy issues (`unparsed_paths`, `orphans`, `duplicates`) with confidence scores. |
| `GET` | `/api/v1/quality/places/preview?id=` | Admin only | Detailed single-place preview showing current record, proposed parent chain, and affected events count. |
| `POST` | `/api/v1/quality/places/fix` | Admin only | Apply hierarchy restructuring for a single place (`{ place_id, parent_id?, clean_name?, place_type? }`). |
| `POST` | `/api/v1/quality/places/batch-fix` | Admin only | Batch apply all high-confidence fixes (`{ min_confidence: 85, place_ids?: Vec<String> }`). |
| `POST` | `/api/v1/quality/places/dismiss` | Admin only | Dismiss place hierarchy recommendation (`{ place_id }`). |

---

## 8. Localization (EN/RU)
- Administrative historical terms translated cleanly without technical developer jargon:
  - English: *Governorate*, *Region*, *District*, *Uyezd*, *County*, *City*, *Town*, *Village*, *Parish*, *Cemetery*.
  - Russian: *Губерния*, *Область*, *Уезд*, *Район*, *Волость*, *Город*, *Село*, *Деревня*, *Приход*, *Кладбище*.
- Translation strings adhere strictly to anti-fluff rules: zero emojis, zero leading/trailing icon glyphs in `en.json` and `ru.json`.
- Toast notifications:
  - EN: *"Place hierarchy updated"*, *"12 place hierarchies updated successfully"*.
  - RU: *"Иерархия места обновлена"*, *"Иерархия успешно обновлена для 12 мест"*.

---

## 9. Edge Cases & Non-Goals

### 9.1 Cycle Protection
Before applying any parent assignment (`parent_id = target_parent`), the engine executes `ensure_no_place_cycle`. If assigning a parent would create an ancestor loop (e.g. Place A $\to$ Place B $\to$ Place A), the operation is rejected with `409 Conflict` (`cycle_detected`).

### 9.2 Preserving Historical Accuracy
In historical genealogy, jurisdictions changed across time (e.g. *Тверская губерния* in 1900 vs *Тверская область* today). The automated engine structures the hierarchy based on the existing tree context and recorded historical documents. Modern alternative names are added to `place_alt_names` rather than overwriting historical names.

### 9.3 Non-Goals
- Automatic external bulk web scraping of geonames or OpenStreetMap: All heuristics use deterministic local rules and existing tree coordinates.
- Destructive deletion of events or places: Events are re-pointed; original unparsed strings are preserved as alternate names.

---

## 10. Definition of Done (DoD)

- [ ] **Administrator-Only Access:** Access to `/quality`, `/people/merge`, and `/api/v1/quality/*` is strictly restricted to Administrators (`RequireAdmin`); Viewers and Editors receive `403 Forbidden`.
- [ ] **Kinship & Relationship Guards:** Relatives who are direct ancestors/descendants, spouses, confirmed siblings with distinct birth dates, or individuals with distinct confirmed parents are strictly disqualified from duplicate detection (`score = 0`).
- [ ] **Name Similarity Cap:** Name similarity alone without relational or vital date corroboration cannot exceed 40% and never triggers a duplicate warning by itself.
- [ ] **Detached Subtree & Singleton Prioritization:** Duplicate detection focuses on detached subtrees, unconnected singletons, and multi-tree imports.
- [ ] **Unparsed Path Decomposition:** Right-to-left comma-separated paths (e.g. *"Тверь, Тверская губерния, Российская империя"*) are split into proper parent-child hierarchy nodes with clean leaf names.
- [ ] **Alt-Name Preservation:** The original unparsed place string is archived into `place_alt_names` so zero textual history is lost.
- [ ] **Place Type Inference:** Keywords (`губерния`, `область`, `уезд`, `район`, `город`, `село`, `деревня`) correctly infer `country`, `region`, `district`, `city`, or `village`.
- [ ] **Coordinate Proximity Matching:** Orphan places with coordinates matching parent child cluster envelopes ($\le 40\text{ km}$ district / $\le 200\text{ km}$ region) receive high-confidence hierarchy recommendations.
- [ ] **Redundant Node Consolidation:** Resolving a place hierarchy into an existing node merges life events and media links without creating duplicate place entities.
- [ ] **Batch Auto-Fix:** Clicking "Auto-fix High Confidence" applies all $\ge 85\%$ recommendations in a single atomic SQL transaction.
- [ ] **1-Click Undo:** Any single or batch place hierarchy fix can be undone immediately from the toast or `/history`.
- [ ] **Zero External Egress:** All place parsing, matching, and spatial calculations operate offline without external network calls.
- [ ] **Bilingual Parity & Icon Discipline:** All UI elements and notifications have complete EN/RU translations with zero icon glyphs or emojis in JSON string values.
