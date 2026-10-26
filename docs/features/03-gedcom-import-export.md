# Feature Specification: GEDCOM Import & Export

## 1. Overview & Objective
Enable seamless data portability and migration with industry-standard genealogy tools (Ancestry, MyHeritage, FamilySearch, RootsMagic, and others). The system must support importing and exporting **GEDCOM 5.5.1** and **GEDCOM 7.0**, including media assets (via **GEDZIP** or bundled ZIP archives).

---

## 2. Supported Formats & Standards

### 2.1 File Standards
1. **GEDCOM 5.5.1:** Legacy standard used by 95% of existing genealogy tools. Character encodings: UTF-8, ANSI/Windows-1252, ANSEL.
2. **GEDCOM 7.0:** Modern standard with native UTF-8 enforcement, standardized event structures, and official media handling.
3. **GEDZIP / Bundled Archive (.gdz / .zip):** A single ZIP archive containing:
   - `tree.ged`: The primary GEDCOM text file.
   - `media/`: Directory containing photos, documents, and certificates referenced by relative paths in the GEDCOM `FILE` tags.
4. **Rust Parser & Writer Engine:** Powered by the `ged_io` crate for high-speed streaming tokenization and serialization with minimal memory footprint.

---

## 3. Import Workflow

### 3.1 Step 1: Upload & Initial Validation
- User uploads a `.ged`, `.zip`, or `.gdz` file (supports drag-and-drop, up to 500MB).
- Server performs stream parsing without loading the full tree into memory:
  - Detects character encoding (with automatic ANSEL to UTF-8 transcoding if needed).
  - Validates syntax and header tags (`HEAD`, `CHAR`, `GEDC.VERS`).
  - Scans for referenced media files and verifies their presence in the uploaded bundle.

### 3.2 Step 2: Pre-Import Summary & Dry Run
Before modifying the database, the UI displays a detailed breakdown:
- Total Individuals found (e.g. `1,420`)
- Total Families found (e.g. `512`)
- Total Media objects found / matched (e.g. `340 files found, 12 missing`)
- Total Source citations found (e.g. `210`) — will be kept as plain-text event sources
- Warnings & Custom Tag Inspection (e.g. `Found 14 vendor-specific tags like _UID, _MARNM`).

### 3.3 Step 3: Import Mode Selection
- **Clean Slate (Full Replace):** Erases previous tree data and populates fresh records from the GEDCOM file. Requires typed confirmation when the tree is not empty; an automatic `tree_snapshot` is taken first ([14](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md)).
- **Append & Intelligent Merge:** Compares incoming tree against existing records using candidate matching, offers side-by-side reconciliation, deduplicates media, and generates an automatic rollback checkpoint. See full specification in [09-tree-import-merge-reconciliation.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/09-tree-import-merge-reconciliation.md).

### 3.4 Step 4: Background Execution & Progress Reporting
- Since trees can contain tens of thousands of records, the import executes as an asynchronous background worker job.
- Real-time progress is streamed to the user via **Server-Sent Events (SSE)** or **WebSockets**:
  - `Parsing records... 45%`
  - `Importing media files... 78%`
  - `Building relationship indices... 95%`
  - `Completed in 4.2 seconds.`
- Jobs run on the Postgres-backed job queue ([17 §5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/17-api-security-operations.md#5-background-jobs)); the user can leave the page and return.

### 3.5 Step 5: Import Report & "Which person are you?"
- Final summary: counts imported, warnings (unparseable dates kept as phrases, unknown tags skipped, missing media), downloadable as a text log.
- On first import into an empty tree, prompt the admin to pick their own person → sets `person_id` and `home_person_id` ([16 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md#3-empty-tree-onboarding-first-run-after-setup)).

---

## 4. Export Workflow

### 4.1 Export Configuration Modal
When an Admin or Editor triggers **"Export Tree"**, an interactive dialog offers:
1. **Format Selection:**
   - GEDCOM 7.0 (Modern UTF-8 standard)
   - GEDCOM 5.5.1 (Maximum backward compatibility)
2. **Media Inclusion:**
   - **Standalone `.ged`:** Only text records and internal references.
   - **Full Package (.zip / .gdz):** Bundles all photos, scanned documents, and relative media paths into an archive ready for importing into desktop tools.
3. **Privacy Filter Checkboxes:**
   - [x] Redact living individuals (replaces living names with `Living`, suppresses private birth dates).
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
| `FAM` | `families` table | Maps `HUSB`, `WIFE`, `CHIL`, `MARR`, `DIV`. |
| `OBJE` | `media_assets` table | Maps file paths, MIME types, titles. |
| `SOUR` / `REPO` / citation `PAGE` | `life_events.source_text` | **Flattened** to text `"<Source title>, <PAGE>"` on each cited event; citations on a person/family (not an event) become a note. Repositories and confidence (`QUAY`) dropped. **Export:** each distinct `source_text` becomes one `0 @S1@ SOUR` record with `TITL` = text, cited from its events (valid in both 5.5.1 and 7.0). |
| `NOTE` | `notes` table | Maps research notes and markdown/plain text. |
| xref `@I42@` | `individuals.gid` | Preserved as `I42` when unique ([10 §3.1](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#31-human-readable-ids-gid)). |
| `NAME` `/Surname/`, `GIVN`, `SURN`, `_PATR`, `_MARNM`, extra `NAME` records | `individuals` name fields + `individual_alt_names` | Russian 3-part given names (`Иван Петрович /Сидоров/`) split into given + patronymic when the second token ends in `-вич/-вна/-ич/-ична`. |
| `PLAC` | `places` hierarchy | Comma-split + dedup ([12 §5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/12-places.md#5-gedcom-mapping)). |
| `DATE` (incl. `@#DJULIAN@`, phrases) | `GenDate` JSONB | Unparseable → phrase, never dropped. |
| `CHIL` + `PEDI` / `_FREL` / `_MREL` | `family_children.rel_to_partner*` | |
| `RESN privacy` / `_PRIV` | `is_private = TRUE` | |
| Unknown / vendor tags | — | Counted in import report; not stored (keeps model simple). |

---

## 6. Definition of Done (DoD)

- [ ] **Parser Compatibility:** Successfully parses standard GEDCOM 5.5.1 sample datasets (e.g. Torture Test GEDCOM, 551test.ged) and GEDCOM 7.0 specifications without crashing.
- [ ] **Encoding Handling:** Correctly decodes UTF-8 with/without BOM, UTF-16, ANSI, and ANSEL legacy genealogical character sets.
- [ ] **Media Bundle Handling:** Uploading a `.zip` or `.gdz` automatically extracts images and establishes links between `individuals` and their corresponding media files.
- [ ] **Pre-Import Review Screen:** Displays accurate counts and warning highlights before writing changes to the database.
- [ ] **Progress Streaming:** Long imports display a responsive, live progress bar with percentage and step descriptions.
- [ ] **Round-Trip Fidelity:** Exporting a tree to GEDCOM and re-importing it into a clean instance yields identical counts of individuals, parent-child relationships, dates, places, and media links.
- [ ] **Privacy Filter Enforcement:** Exporting with "Redact Living Individuals" produces a file free of sensitive birthdates or personal notes of living persons.
