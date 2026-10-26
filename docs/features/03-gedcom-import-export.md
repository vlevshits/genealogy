# Feature Specification: GEDCOM Import & Export

## 1. Overview & Objective
Enable seamless data portability and migration with industry-standard genealogy tools (Ancestry, MyHeritage, FamilySearch, RootsMagic, Gramps, Gramps Web, and others). The system must support importing and exporting **GEDCOM 5.5.1** and **GEDCOM 7.0**, including media assets (via **GEDZIP** or bundled ZIP archives), while providing first-class handling of real-world vendor dialects (such as Gramps Web 6.0.1 and MyHeritage 5.5.1 exports) with zero data loss.

---

## 2. Supported Formats & Standards

### 2.1 File Standards
1. **GEDCOM 5.5.1:** Legacy standard used by 95% of existing genealogy tools. Character encodings: UTF-8, ANSI/Windows-1252, ANSEL.
2. **GEDCOM 7.0:** Modern standard with native UTF-8 enforcement, standardized event structures, and official media handling.
3. **GEDZIP / Bundled Archive (.gdz / .zip):** A single ZIP archive containing:
   - `tree.ged`: The primary GEDCOM text file.
   - Media folder containing referenced photos and documents.
4. **Streaming Parser & Writer:** Uses the `ged_io` crate for streaming tokenization and serialization with minimal memory footprint.

### 2.2 Dialect & Vendor Extensions (Gramps Web 6.0.1 & MyHeritage 5.5.1)
Real-world exports from platforms like Gramps Web 6.0.1 (identified via `1 SOUR Gramps 2 VERS 6.0.1`) and MyHeritage contain vendor-specific constructs, path conventions, and tag combinations. The importer natively understands and reconciles these patterns:

1. **Multiple `NAME` Blocks (Maiden vs. Married Surnames):**
   Gramps Web exports married women with two distinct `1 NAME` blocks:
   ```gedcom
   1 NAME Мария /Петрова/
   2 TYPE birth
   2 GIVN Мария
   2 SURN Петрова
   1 NAME Мария /Иванова/
   2 TYPE married
   2 GIVN Мария
   2 SURN Иванова
   ```
   - **Import:** The record with `2 TYPE birth` maps to `individuals.birth_surname` (`Петрова`) and initial `individuals.given_name` (`Мария`). The record with `2 TYPE married` maps to `individuals.surname` (`Иванова`, representing the current married surname) and creates a row in `individual_alt_names` (`name_type = 'married'`, `given_name = 'Мария'`, `surname = 'Иванова'`). If only a birth name is present, `surname` and `birth_surname` are both set to that surname. Russian patronymics within `GIVN` are split into `individuals.patronymic` per [00 §2.2](00-internationalization-localization.md#22-slavic-genealogy-name-structure-patronymics--отчество).
   - **Export:** When `birth_surname` is present and differs from `surname`, export emits primary `1 NAME Given /BirthSurname/` with `2 TYPE birth` followed by `1 NAME Given /Surname/` with `2 TYPE married`, preserving 100% round-trip fidelity.

2. **Geographic Coordinates (`PLAC.MAP`):**
   Gramps Web and MyHeritage export coordinates using directional hemisphere letters:
   ```gedcom
   2 PLAC Тверь, Россия
   3 MAP
   4 LATI N56.858721
   4 LONG E35.917596
   ```
   - **Import:** The parser extracts directional letters (`N`/`S` for latitude, `E`/`W` for longitude). It converts `N` → positive decimal float (`+56.858721`), `S` → negative decimal float (`-56.858721`), `E` → positive decimal float (`+35.917596`), and `W` → negative decimal float (`-35.917596`), storing the result in `places.latitude` and `places.longitude` (`DOUBLE PRECISION`). Raw numeric floats without directional prefix are parsed directly.
   - **Export:** Coordinates are formatted back with standard directional prefixes (`N`/`S` and `E`/`W`) per GEDCOM 5.5.1 specification.

3. **Parental & Social Associations (`1 ASSO` + `2 RELA`):**
   Gramps Web frequently emits explicit `ASSO` tags for parents on individual records alongside `FAMC` family linkage:
   ```gedcom
   1 ASSO @I0013@
   2 RELA Отец
   1 ASSO @I0014@
   2 RELA Мать
   ```
   - **Parental Deduplication:** When an `ASSO` record references a father or mother (`RELA` = `Отец`, `Мать`, `Father`, `Mother`, `Parent`) that already matches a resolved `FAM` parent-child relationship, the importer marks it as redundant and avoids creating duplicate links.
   - **Missing Family Synthesis:** If a parental `ASSO` exists without an accompanying `FAM` record, the system synthesizes a single-parent family structure.
   - **Social / Non-Parental Associations:** Other relationships (e.g. `Godparent` / `Крёстный`, `Witness` / `Свидетель`, `Guardian` / `Опекун`, `Neighbor`) are converted into research notes attached to the individual (`"Связь: [Роль] с [Имя]"`, e.g. `"Связь: Крёстный с @I0042@ Иван Иванов"`), ensuring no social context is lost.

4. **Server Media Paths & Missing Binaries (`0 @O...@ OBJE`):**
   Gramps Web exports server-internal Linux/Docker filesystem paths:
   ```gedcom
   0 @O0000@ OBJE
   1 FILE /root/gramps/000085_971532825e5c5dtfb56015_Y.jpg
   2 FORM jpg
   2 TITL Портрет дедушки
   ```
   - **Bundled Archive / GEDZIP:** The importer strips absolute directory prefixes (`/root/gramps/`, `C:\...`) and matches media entries by base filename (`000085_971532825e5c5dtfb56015_Y.jpg`) within the uploaded archive.
   - **Standalone `.ged` Import:** When media files are not uploaded with the text file, the parser creates media stubs in `media_assets` with `original_file_name` = base filename, `external_path` = raw path, `title` = `TITL`, `mime_type` inferred from `FORM`, and `is_missing = TRUE`, linking them to the appropriate individuals via `media_links`.
   - **Post-Import Media Reconciliation:** Administrators can upload a ZIP or folder of photos at any later time; the system reconciles incoming files by base filename, populates binaries, clears `is_missing`, and generates WebP thumbnails without breaking links.

5. **External Provenance Identifiers (`_UID`, `RIN`):**
   Gramps and MyHeritage attach persistent identifiers:
   ```gedcom
   1 FACT MH:I10
   2 TYPE RIN
   1 _UID CF665D02-7132-46C3-868D-7BF37FD03AC5
   ```
   - `1 _UID` (128-bit hex UUID) and `FACT ... TYPE RIN` (MyHeritage Record Identification Number) are preserved in `individuals.external_ids` and `families.external_ids` JSONB (`{"uid": "CF665D02-7132-46C3-868D-7BF37FD03AC5", "mh_rin": "MH:I10"}`).
   - On GEDCOM export, both `_UID` and `RIN` tags are emitted, enabling seamless bi-directional synchronization with external software.

6. **Vendor System Timestamps (`1 EVEN ... 2 TYPE _UPD`):**
   MyHeritage and Gramps Web export software record modification timestamps as:
   ```gedcom
   1 EVEN 16 MAY 2011 16:14:00 GMT -0500 
   2 TYPE _UPD
   ```
   - These represent database sync timestamps rather than personal biographical milestones.
   - The parser filters `_UPD` events out of the individual's life events timeline, preventing artificial events from appearing on the profile, while recording the timestamp in the import log and metadata.

7. **Civil Marriage Registration (`1 MARL`):**
   Gramps Web exports civil marriage registration / licensing (`1 MARL`) alongside ceremonial marriages (`1 MARR`):
   ```gedcom
   1 MARR
   2 DATE 22 AUG 2020
   1 MARL
   2 DATE 1 AUG 2020
   ```
   - Imported into `life_events` linked to the `family_id` with `event_type = 'marriage_license'` (or `custom` with `custom_type = 'marriage_license'`), displaying in Russian UI as *"Регистрация брака / ЗАГС"*.

8. **Deceased Status Flag (`1 DEAT Y`):**
   Individuals marked with `1 DEAT Y` (without explicit date or place) are flagged as deceased (`has_death_event = TRUE`, `living_override = FALSE`), ensuring privacy algorithms never inadvertently treat deceased individuals as living.

9. **Cause of Death & Age (`2 CAUS`, `2 AGE`):**
   Subtags under `DEAT` (e.g. `2 CAUS общая сердечная недостаточность`, `2 AGE 72`) are mapped directly to `life_events.description` (`"общая сердечная недостаточность (возраст: 72)"`), preserving medical and biographical context.

10. **Text Concatenation (`1 CONC` vs. `1 CONT`):**
    Per GEDCOM 5.5.1 specification:
    - `1 CONC` lines are appended to the preceding line *without* whitespace or newline.
    - `1 CONT` lines are appended with a newline (`\n`).
    This prevents mid-word splitting in multi-line notes.

11. **Escaped Characters in Contact Data (`1 RESI` with `2 EMAIL`):**
    GEDCOM standard escapes the `@` symbol as `@@` (e.g. `user@@example.com`). The parser unescapes `@@` → `@` and preserves contact information in notes or residence event details.

---

## 3. Import Workflow

### 3.1 Step 1: Upload & Initial Validation
- Administrator uploads a `.ged`, `.zip`, or `.gdz` file (supports drag-and-drop, up to 500MB; restricted to Admin role).
- Server performs stream parsing without loading the full tree into memory:
  - Detects character encoding (with automatic ANSEL to UTF-8 transcoding if needed).
  - Validates syntax and header tags (`HEAD`, `CHAR`, `GEDC.VERS`, `SOUR`).
  - Scans for referenced media files, identifies external directory prefixes (e.g. `/root/gramps/`), and verifies presence of local files in the uploaded bundle.

### 3.2 Step 2: Pre-Import Summary & Dry Run
Before modifying the database, the UI displays a detailed breakdown:
- Total Individuals found (e.g. `1,420`)
- Total Families found (e.g. `512`)
- Total Media objects found / matched (e.g. `340 files found, 12 missing binaries`)
- Total Source citations found (e.g. `210`) — will be kept as plain-text event sources
- Dialect & Vendor Tag Highlights (e.g. `Identified Gramps Web 6.0.1 export; normalized 57 coordinate pairs, preserved 339 _UIDs, filtered 338 internal _UPD timestamps`).

### 3.3 Step 3: Import Mode Selection
- **Clean Slate (Full Replace):** Erases previous tree data and populates fresh records from the GEDCOM file. Requires typed confirmation when the tree is not empty; an automatic `tree_snapshot` is taken first ([14](14-change-history-concurrency.md)).
- **Append & Intelligent Merge:** Compares incoming tree against existing records using candidate matching, offers side-by-side reconciliation, deduplicates media, and generates an automatic rollback checkpoint. See full specification in [09-tree-import-merge-reconciliation.md](09-tree-import-merge-reconciliation.md).

### 3.4 Step 4: Background Execution & Progress Reporting
- Since trees can contain tens of thousands of records, the import executes as an asynchronous background worker job.
- Real-time progress is streamed to the user via **Server-Sent Events (SSE)** or **WebSockets**:
  - `Parsing records... 45%`
  - `Importing media files... 78%`
  - `Building relationship indices... 95%`
  - `Completed in 4.2 seconds.`
- Jobs run on the Postgres-backed job queue ([17 §5](17-api-security-operations.md#5-background-jobs)); the user can leave the page and return.

### 3.5 Step 5: Import Report & Post-Import Media Reconciliation
- Final summary: counts imported, warnings (unparseable dates kept as phrases, unknown tags skipped, missing media), downloadable as a text log.
- On first import into an empty tree, prompt the admin to pick their own person → sets `person_id` and `home_person_id` ([16 §3](16-dashboard-home.md#3-empty-tree-onboarding-first-run-after-setup)).
- **Post-Import Media Upload:** If the import referenced media files that were missing from the upload (e.g. standalone `.ged` with external server paths), the UI presents a reconciliation prompt: *"129 media files referenced in this tree are missing images. Upload your photos folder or ZIP archive to link them."* Uploading the folder or archive triggers automatic reconciliation matching incoming filenames to `media_assets.original_file_name`, storing the binaries in persistent storage, clearing `is_missing = FALSE`, and generating WebP thumbnails without breaking any existing links.

---

## 4. Export Workflow

### 4.1 Export Configuration Modal
When an **Administrator** triggers **"Export Tree"** (restricted exclusively to the Admin role per [02 §2](02-rbac-access-control.md#2-master-segregation--rbac-permission-matrix); Editors, Viewers, and Public visitors receive 403 Forbidden):
1. **Format Selection:**
   - GEDCOM 7.0 (Modern UTF-8 standard)
   - GEDCOM 5.5.1 (Maximum backward compatibility)
2. **Media Inclusion:**
   - **Standalone `.ged`:** Only text records and internal references.
   - **Full Package (.zip / .gdz):** Bundles all photos, scanned documents, and relative media paths into an archive ready for importing into desktop tools.
3. **Privacy Filter Checkboxes:**
   - [x] Redact living individuals (replaces living names with single initial e.g. `И.` / `J.`, completely suppresses surnames, patronymics, and private birth dates).
   - [ ] Include private notes and internal research logs.
4. **Subtree Export (Optional):**
   - Export entire tree.
   - Export ancestors of focal individual (N generations).
   - Export descendants of focal individual (N generations).

---

## 5. Data Mapping & Schema Considerations

| GEDCOM Tag | Database Entity / Field | Notes |
| :--- | :--- | :--- |
| `INDI` | `individuals` table | Maps `SEX`, `NAME`, `TITL`, `NPFX`, `NSFX`. |
| `INDI.BIRT`, `DEAT`, `BAPM` | `life_events` table | Maps event type, `DATE`, `PLAC`, `ADDR`. |
| `1 DEAT Y` | `individuals` living flags | Sets `has_death_event = TRUE`, `living_override = FALSE` even without date/place. |
| `2 CAUS`, `2 AGE` | `life_events.description` | Cause of death and age at death mapped into description (e.g. `"Рак (возраст: 56)"`). |
| `FAM` | `families` table | Maps `HUSB`, `WIFE`, `CHIL`, `MARR`, `DIV`. |
| `1 MARL` | `life_events` table | Family event with `event_type = 'marriage_license'` (civil registration / ЗАГС). |
| `OBJE` / `0 @O...@ OBJE` | `media_assets` table | Maps file paths, MIME types, titles. |
| `1 FILE /path/file.ext` | `media_assets.original_file_name` & `external_path` | Extracts base filename, strips external server paths (`/root/gramps/`), flags `is_missing = TRUE` if binary not present. |
| `1 NAME` `/Surname/`, `GIVN`, `SURN`, `_PATR`, `_MARNM` | `individuals` name fields + `individual_alt_names` | Russian 3-part given names split into given + patronymic when second token ends in `-вич/-вна/-ич/-ична`. |
| `1 NAME ... 2 TYPE birth` vs `2 TYPE married` | `individuals.birth_surname`, `surname` & `individual_alt_names` | `TYPE birth` surname stored in `birth_surname`; `TYPE married` surname stored in `surname` and `individual_alt_names` (`name_type = 'married'`). Export emits both blocks. |
| `1 ASSO @I...@` + `2 RELA` | `families` / `notes` | Parental `ASSO` deduplicated against `FAM` or synthesized into family structure; social associations saved as attached research notes (`"Связь: [Роль] с [Имя]"`). |
| `1 _UID` | `individuals.external_ids`, `families.external_ids` | 128-bit hex UUID preserved in JSONB (`{"uid": "..."}`) and round-tripped on export. |
| `1 FACT ... 2 TYPE RIN` | `individuals.external_ids`, `families.external_ids` | MyHeritage RIN preserved in JSONB (`{"mh_rin": "..."}`) and round-tripped on export. |
| `1 EVEN ... 2 TYPE _UPD` | Import audit log / metadata | Vendor record modification timestamp; silently filtered from life events timeline. |
| `PLAC` | `places` hierarchy | Comma-split + dedup ([12 §5](12-places.md#5-gedcom-mapping)). |
| `PLAC.MAP.LATI`, `LONG` | `places.latitude`, `places.longitude` | Converts directional `N`/`S`/`E`/`W` prefixes to decimal floats (`DOUBLE PRECISION`); re-exports with directional prefixes. |
| `DATE` (incl. `@#DJULIAN@`, phrases) | `GenDate` JSONB | Unparseable → phrase, never dropped. |
| `1 CONC` / `1 CONT` | `notes.body` | `CONC` appended without newline/whitespace; `CONT` appended with newline (`\n`). |
| `1 RESI` with `2 EMAIL` | `notes` / `life_events` | Unescapes `@@` → `@`; preserves contact details in attached notes or residence event description. |
| `CHIL` + `PEDI` / `_FREL` / `_MREL` | `family_children.rel_to_partner*` | Maps relationship pedigree (`birth`, `adopted`, `foster`). |
| `SOUR` / `REPO` / citation `PAGE` | `life_events.source_text` | **Flattened** to text `"<Source title>, <PAGE>"` on each cited event; citations on a person/family (not an event) become a note. Repositories and confidence (`QUAY`) dropped. **Export:** each distinct `source_text` becomes one `0 @S1@ SOUR` record with `TITL` = text, cited from its events. |
| `NOTE` | `notes` table | Maps research notes and markdown/plain text. |
| xref `@I42@` | `individuals.gid` | Preserved as `I42` when unique ([10 §3.1](10-core-data-model.md#31-human-readable-ids-gid)). |
| `RESN privacy` / `_PRIV` | `is_private = TRUE` | Privacy flag preserved. |
| Unknown / vendor tags | — | Counted in import report; not stored (keeps model simple). |

---

## 6. Definition of Done (DoD)

- [ ] **Gramps Web 6.0.1 Test Fixture Validation:** Successfully parses and imports real-world Gramps Web 6.0.1 exports (`test-data/gramps-export/gramps-web-export-20261026130902.ged` locally; anonymized synthetic equivalent in CI) without crashing, preserving all 284 individuals, 84 families, and 129 media references.
- [ ] **Parser Compatibility:** Successfully parses standard GEDCOM 5.5.1 sample datasets (e.g. Torture Test GEDCOM, 551test.ged) and GEDCOM 7.0 specifications without crashing.
- [ ] **Encoding Handling:** Correctly decodes UTF-8 with/without BOM, UTF-16, ANSI, and ANSEL legacy genealogical character sets.
- [ ] **Multiple Name Blocks (Birth & Married):** For individuals with both `TYPE birth` and `TYPE married` name records, sets `birth_surname` and `surname` accordingly, stores the married name variant in `individual_alt_names`, and round-trips both on GEDCOM export.
- [ ] **Directional Coordinate Parsing:** Accurately converts directional coordinate strings (e.g. `N52.31724` → `52.31724`, `E26.86921` → `26.86921`, `S...` and `W...` to negative floats) into `places.latitude` and `places.longitude`, and formats back on export.
- [ ] **Parental Association Deduplication:** Deduplicates `1 ASSO` tags with `RELA Отец` / `RELA Мать` that match existing `FAM` relationships without creating duplicate graph edges; preserves non-parental associations in notes.
- [ ] **External Provenance Preservation:** Preserves `1 _UID` and `FACT ... TYPE RIN` into `external_ids` JSONB and re-emits them on export.
- [ ] **Vendor Event Filtering:** Filters `1 EVEN ... 2 TYPE _UPD` timestamps out of the biographical life events timeline.
- [ ] **Deceased Status Integrity:** `1 DEAT Y` records without explicit date or place correctly set `has_death_event = TRUE` and `living_override = FALSE`.
- [ ] **Media Bundle Handling & Server Path Stripping:** Uploading a `.zip` or `.gdz` extracts images and matches files by base filename, stripping external server prefixes (e.g. `/root/gramps/`).
- [ ] **Post-Import Media Reconciliation:** Standalone `.ged` imports create media asset stubs (`is_missing = TRUE`); subsequent bulk photo upload automatically reconciles assets by base filename, stores files, and clears missing flags.
- [ ] **Text Concatenation (`CONC`/`CONT`):** Assembles multi-line notes without breaking words on `CONC` lines, and preserves newlines on `CONT` lines.
- [ ] **Pre-Import Review Screen:** Displays accurate counts and warning highlights before writing changes to the database.
- [ ] **Progress Streaming:** Long imports display a responsive, live progress bar with percentage and step descriptions.
- [ ] **Round-Trip Fidelity:** Exporting a tree to GEDCOM and re-importing it into a clean instance yields identical counts of individuals, parent-child relationships, dates, places, and media links.
- [ ] **Privacy Filter Enforcement:** Exporting with "Redact Living Individuals" produces a file free of sensitive birthdates, surnames, or personal notes of living persons, masking names as single initials (e.g. `И.` / `J.`).
- [ ] **Admin-Only Isolation:** GEDCOM import and export endpoints reject non-Admin requests with 403 Forbidden.
