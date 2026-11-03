# Feature Specification: Printable PDF Tree Export (Single & Multi-Page Tiled)

## 1. Overview & Objective
Enable administrators to transform digital family trees into high-resolution, physical keepsakes. Generating and exporting printable PDFs is restricted exclusively to the **Administrator** role per [02 §2](02-rbac-access-control.md#2-master-segregation--rbac-permission-matrix) (to prevent bulk extraction of family data by viewers or unauthenticated users).

The system generates professional-grade, high-resolution vector PDF documents in two distinct printing formats:
1. **Single-Page Poster:** Standard large-format engineering/poster sizes (A0, A1, A2, A3) suitable for professional print shops.
2. **Multi-Page Tiled PDF:** Automatically sliced into a grid of standard home-printer pages (A4 or US Letter) complete with **cut lines, overlap margins, and assembly coordinates** so family members can easily trim and assemble them together.

The export studio provides **complete feature parity with interactive Tree View** ([04-tree-visualizations.md](04-tree-visualizations.md)):
- **Comprehensive Layout Modes:** Pedigree (horizontal ancestry), Hourglass (ancestors + descendants), Descendants (waterfall), **All Nodes** (`all_nodes` — complete family forest graph view), and Fan Chart (radial ancestry).
- **Layout Density & Collateral Modes:** Toggle between **Compact** (direct lineal ascendants/descendants) and **Wide** (collateral expansion including aunts, uncles, first cousins, nieces, and nephews).
- **Customizable Relatives:** Independent checkboxes for **"Include spouse / partner"** and **"Include siblings"** with strict biological/marital connection routing.
- **Embedded Profile Photos:** 300 DPI circular/rounded vector portraits embedded on cards in both live preview and generated PDF output.
- **Customizable Title & Subtitle Banner:** Full sidebar customization of the main headline title and subtitle/description with intelligent localized fallback defaults.

---

## 2. Data Model & Request Parameters

The PDF export subsystem does not alter the underlying persistence schema defined in [10-core-data-model.md](10-core-data-model.md). It consumes entities from the in-memory `GenealogyPrivacyStore` snapshot (`individuals`, `families`, `family_children`, `events`, `media`) and computes a pure vector layout tree.

### 2.1 API Request Schema (`PdfExportRequest`)
The export request DTO supports fine-grained control over tree generation and relative inclusion:

```json
{
  "focal_person_id": "uuid | string",
  "view_mode": "pedigree | hourglass | descendants | all_nodes | fan",
  "collateral": true,
  "generations": 5,
  "theme": "modern | heritage",
  "format": "tiled | poster",
  "target_sheet_size": "A4 | Letter | A0 | A1 | A2 | A3",
  "overlap_mm": 10.0,
  "show_crop_marks": true,
  "show_photos": true,
  "redact_living": false,
  "include_assembly_guide": false,
  "include_partner": true,
  "include_siblings": false,
  "title": "The Levshits Family Tree",
  "subtitle": "Commemorative reunion edition • August 2026"
}
```

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `focal_person_id` | `Option<String>` | `None` (resolves home person or tree root) | ID or GID of the center/root individual (optional in `all_nodes` mode). |
| `view_mode` | `Option<String>` | `"pedigree"` | Tree projection: `"pedigree"`, `"hourglass"`, `"descendants"`, `"all_nodes"` (or `"all"`), or `"fan"`. |
| `collateral` | `Option<bool>` | `true` (in hourglass) | Collateral expansion toggle: `true` (Wide: aunts/uncles, cousins, nieces/nephews) vs `false` (Compact: direct lineal line). |
| `generations` | `Option<u32>` | `5` | Depth of generations (clamped 1–8). Ignored in `all_nodes` mode. |
| `theme` | `Option<String>` | `"modern"` | Visual theme: `"modern"` (Modern Clean) or `"heritage"` (Heritage Antique). |
| `format` | `Option<String>` | `"tiled"` | Printing format: `"tiled"` (multi-sheet home print) or `"poster"` (single continuous sheet). |
| `target_sheet_size` | `Option<String>` | `"A4"` | Sheet dimensions: `"A4"`, `"Letter"`, `"A0"`, `"A1"`, `"A2"`, `"A3"`. |
| `overlap_mm` | `Option<f64>` | `10.0` | Overlap bleed margin for tiled printing (5.0 mm to 20.0 mm). |
| `show_crop_marks` | `Option<bool>` | `true` | When true, renders dashed cut lines, scissor marks, and corner alignment crosshairs. |
| `show_photos` | `Option<bool>` | `true` | When true, embeds 300 DPI circular/rounded avatar portraits on person cards. |
| `redact_living` | `Option<bool>` | `false` | When true, redacts exact birth dates and portraits of living individuals. |
| `include_assembly_guide` | `Option<bool>` | `false` | When true, prepends Sheet 1 with an assembly grid guide. |
| `include_partner` | `Option<bool>` | `true` | When true, includes the focal person's spouse/partner(s) (and married spouses of descendants in Descendants mode). |
| `include_siblings` | `Option<bool>` | `false` | When true, includes full and half siblings of the focal person branching from common parents. |
| `title` | `Option<String>` | `None` (auto-generated) | Custom headline banner title override. When omitted, falls back to `"{tree_title}: {focal_name}"` or `"Family Tree of {focal_name}"`. |
| `subtitle` | `Option<String>` | `None` (auto-generated) | Custom subtitle / description text override. When omitted, falls back to metadata string (e.g. `"Layout: {mode} • Generations: {gens} • Persons: {count} • {date}"`). |

### 2.2 Preview Response Schema (`PdfExportPreviewInfo`)
Returned by `POST /api/v1/export/pdf/preview-info` to feed the client-side studio canvas:
- `total_sheets: u32` — Total physical sheets required.
- `grid_rows: u32` / `grid_cols: u32` — Grid dimensions for tiled format ($N \times M$).
- `canvas_width_mm: f64` / `canvas_height_mm: f64` — Calculated overall canvas dimensions in millimeters.
- `sheet_width_mm: f64` / `sheet_height_mm: f64` — Selected target paper size.
- `overlap_mm: f64` — Active overlap bleed margin.
- `format: String` & `target_sheet_size: String` — Active format and paper size identifier.
- `total_persons: usize` — Total number of rendered individual cards.
- `summary_text: String` — Localized summary badge string (e.g. *"4 Sheets (2×2 Grid) of A4 Paper"*).
- `title: String` — Active resolved headline title.
- `subtitle: String` — Active resolved subtitle / description.
- `cards: Vec<PdfPreviewCardDto>` — Geometric card positions ($x, y, w, h$, generation, lifespan, focal flag, avatar flag).
- `connectors: Vec<PdfPreviewConnectorDto>` — Polyline connector paths and dash styles.
- `poster_layout: Option<PdfPosterLayoutDto>` — Scaling and centering transform for single poster mode.
- `tiles: Vec<PdfTileSheetDto>` — Slicing windows for multi-page tiled mode.

---

## 3. User Experience, Tree Layouts & Positioning Rules

### 3.1 Print Formats & Page Tiling Engine

#### 3.1.1 Single-Page Poster Mode
- **Supported Dimensions:** A0 ($841 \times 1189\text{ mm}$), A1 ($594 \times 841\text{ mm}$), A2 ($420 \times 594\text{ mm}$), A3 ($297 \times 420\text{ mm}$), or Custom Width/Height.
- **Resolution:** 100% vector typography and connector curves; photos downsampled to 300 DPI for crisp print quality without astronomical file sizes.
- **Framing & Margins:** Configurable decorative border, title banner, generation counter, and creation date.

#### 3.1.2 Multi-Page Tiled Mode (Print & Assemble at Home)
Home printers cannot print large formats directly. The tiling engine slices the large tree canvas into an $N \times M$ grid of standard sheets (A4 or US Letter):

```
+----------------+----------------+----------------+
| Page 1 (R1,C1) | Page 2 (R1,C2) | Page 3 (R1,C3) |
| [Crop Margin]  | [Crop Margin]  | [Crop Margin]  |
+----------------+----------------+----------------+
| Page 4 (R2,C1) | Page 5 (R2,C2) | Page 6 (R2,C3) |
|                |                |                |
+----------------+----------------+----------------+
```

1. **Overlap Margins (Bleed):** Each page includes a configurable overlap (default: $10\text{ mm}$ / 0.4 inches) where adjacent artwork duplicates slightly.
2. **Crop Marks / Cut Lines:** Thin dashed line and scissors icon indicating the exact line to slice with a hobby knife or paper trimmer.
3. **Alignment Crosshairs (`+`):** Corner crosshair marks to ensure pinpoint alignment when pasting sheet over sheet.
4. **Assembly Header / Footer:** Subtle text outside the printable area:
   - Example: `Sheet 5 of 9 [Row 2, Column 2] — Paste Right of Sheet 4, Below Sheet 2`.
5. **Assembly Guide Page (Sheet 1):** Optional first page showing a mini bird's-eye map of the entire grid with instructions on how to glue the sheets together.

### 3.2 Visual Themes & Styling

| Theme Name | Background | Typography | Connectors | Best For |
| :--- | :--- | :--- | :--- | :--- |
| **Modern Clean** | Pure White / Subtle Cream | Sans-serif (Inter / Roboto) | Sleek 1px slate lines (`#64748b`) | Modern family reunions, minimalist frames |
| **Heritage Antique** | Aged parchment texture / Warm ivory | Classical Serif (Playfair / Merriweather) | Warm sepia flourishes (`#78350f`) | Formal genealogy records, vintage gifts |

### 3.3 Layout Projections & Density Modes

Standard person card dimensions: $W_{\text{card}} = 56.0\text{ mm}$, $H_{\text{card}} = 26.0\text{ mm}$, horizontal gap $G_x = 16.0\text{ mm}$, vertical gap $G_y = 12.0\text{ mm}$.

```
+--------------------------------------------------------------------------------------------------------------------------------+
|                                             SUMMARY OF LAYOUT MODES & DENSITY                                                  |
+-------------------+--------------------+---------------------------------------+-----------------------------------------------+
| Mode              | Density Options    | Partner Placement                     | Sibling & Collateral Scope                    |
+-------------------+--------------------+---------------------------------------+-----------------------------------------------+
| **Pedigree**      | Fixed              | Paired vertically at Gen 0 below focal| Siblings stacked vertically on Gen 0 column;  |
| (Left-to-Right)   |                    | ($X = \text{MARGIN}$), ochre bar      | direct ancestors expand rightward             |
+-------------------+--------------------+---------------------------------------+-----------------------------------------------+
| **Hourglass**     | **Compact** vs     | Flanks focal horizontally on Gen 0;   | Compact: direct lineal ancestors/descendants. |
| (Center-Outward)  | **Wide**           | children descend from union bus       | Wide: collateral aunts/uncles/cousins/nieces  |
+-------------------+--------------------+---------------------------------------+-----------------------------------------------+
| **Descendants**   | Fixed              | Paired horizontally with focal on Gen | Siblings placed on Gen 0 flanking focal with  |
| (Top-Down)        |                    | 0; married descendants paired beside  | top fraternal bracket; descent follows focal  |
+-------------------+--------------------+---------------------------------------+-----------------------------------------------+
| **All Nodes**     | Complete Forest    | Partners paired beside nodes across   | Entire family graph partitioned by connected  |
| (Graph View)      |                    | all components on generational baselines| components + isolated records shelf grid      |
+-------------------+--------------------+---------------------------------------+-----------------------------------------------+
| **Fan Chart**     | Radial Sectors     | Disabled (not supported geometrically)| Disabled (not supported geometrically)        |
+-------------------+--------------------+---------------------------------------+-----------------------------------------------+
```

#### 3.3.1 Hourglass: Compact vs Wide Collateral Mode
Controlled by the `collateral` query/request parameter:
1. **Compact Mode (`collateral = false`):**
   - Direct biological ascendants upwards (parents, grandparents) and direct descendants downwards (children, grandchildren).
   - Horizontal width is minimal; ideal for standard multi-sheet poster sizes.
   - When `include_partner = true`, focal person's spouse(s) are placed on Gen 0.
   - When `include_siblings = true`, focal person's direct siblings are added along Gen 0.
2. **Wide Collateral Mode (`collateral = true`, Default for Hourglass):**
   - Encompasses full collateral family branches originating from direct ancestors:
     - **Generation -1:** Parents + aunts and uncles (grandparents' children) + spouses of aunts/uncles.
     - **Generation 0:** Focal person + spouses + siblings (and spouses) + first cousins (aunts/uncles' children) + first cousin spouses.
     - **Generation +1:** Children of focal couple + nieces and nephews (siblings' children) + first cousins once removed.
     - **Generation +2:** Grandchildren + grand-nieces/nephews.
   - Strict horizontal baseline alignment: all relatives of the same generational distance align on identical horizontal baselines ($Y = \text{tier} \times \text{ROW\_STEP}$).

#### 3.3.2 All Nodes Mode (`all_nodes` / Complete Forest Graph View)
- **Objective:** Export the complete genealogical archive onto a comprehensive wall poster or multi-page tiled booklet.
- **Traversal & Partitioning:**
  - Bypasses BFS depth pruning; discovers all individuals and families in the active privacy snapshot.
  - Partitions graph into connected components using Disjoint Set Union (DSU).
  - Primary component (containing focal person or tree root) is placed centrally at top.
  - Secondary components are arranged horizontally side-by-side or in a multi-row grid with $\ge 30.0\text{ mm}$ spacing between component bounding boxes.
  - Isolated singletons (individuals with zero parents, spouses, or children) are arranged in a neat 4-column shelf grid below the lowest generational tier.
- **Controls Adaptation:**
  - Generation Depth selector is hidden/ignored in `all_nodes` mode.
  - Summary badge displays: `Total Individuals: {count}`.

#### 3.3.3 Partners / Spouses Placement & Marital Connectors
- **Default State:** `include_partner = true`.
- **Positioning:**
  - Vertical layouts (Hourglass, Descendant, All Nodes): Positioned on the same horizontal baseline as the individual, spaced by $12.0\text{ mm}$.
  - Symmetrical multi-spouse placement per [04 §2.7](04-tree-visualizations.md#27-multiple-spouses-successive-marriages--family-partitioned-children-buses):
    - 1 Spouse: `[Focal] === [Spouse]`.
    - 2 Spouses: Symmetrically flanking the person in chronological marriage order: `[Spouse 1] === [Focal] === [Spouse 2]`.
    - 3+ Spouses: Sequenced chronologically with clear union anchors.
  - Horizontal layouts (Pedigree): Positioned at $X = \text{MARGIN\_X}$, directly below the focal person (`Y = focal.y + CARD_HEIGHT + 8.0mm`), keeping the horizontal corridor open for ancestors.
- **Marital Connector Bar:**
  - Warm ochre connector line (`--color-brand-accent`, `#d97706` / `#b45309`, stroke width 1.5 pt).
  - In vertical layouts, runs horizontally between card side centers. In Pedigree, runs vertically between bottom of focal and top of partner.

#### 3.3.4 Siblings Placement & Parental Union Distribution Bus
- **Default State:** `include_siblings = false`.
- **Baseline Alignment:** Full and half siblings are placed on Generation 0, ordered chronologically by birth date from left to right (or top to bottom in Pedigree).
- **Parental Union Distribution Bus:**
  - Siblings **never** connect directly to the focal person card.
  - Stems descend from the **common parents' distribution runner**:
    1. Parent couple's union at Generation -1 drops a vertical stem into the inter-generational zone.
    2. Horizontal distribution bus spans across all children of that union (focal person + siblings).
    3. Vertical drop stems descend into the top-center of each child card.
  - Half-siblings: Children from a parent's prior or subsequent marriage hang strictly from that specific parental union bus.
  - Unknown parents fallback: When parents are not recorded, siblings share a top dashed fraternal bracket runner.

#### 3.3.5 Photos & Avatar Rendering Pipeline
- When `show_photos = true`:
  - **In Live Preview Canvas:** Resolves avatar thumbnail (`/api/v1/media/{id}/crop` or data URL) and renders a crisp circular portrait on the card. If unassigned, displays a clean vector monogram initials circle.
  - **In Generated Vector PDF:** Rust engine resolves avatar bytes from storage, crops to a 300 DPI circular vector mask with a 0.5 pt subtle ring border, and embeds directly in the PDF stream.
  - When `redact_living = true` and individual is living, photo is suppressed and replaced by a neutral vector silhouette.

#### 3.3.6 Customizable Title & Subtitle Banner
The header banner of the canvas and exported PDF contains two customizable text elements:
1. **Chart Title (`title`):**
   - Main headline typography ($16\text{ pt}$ bold serif in Heritage, $15\text{ pt}$ semi-bold sans-serif in Modern).
   - Custom text provided in `title` input is rendered verbatim.
   - Smart fallback: When omitted or blank, defaults to `"{instance_tree_title}: {focal_name}"` or `"Family Tree of {focal_name}"`.
2. **Description / Subtitle (`subtitle`):**
   - Secondary explanatory line ($9\text{ pt}$ medium text).
   - Custom text provided in `subtitle` input is rendered verbatim (e.g. *"Commemorative family reunion keepsake • August 2026"*).
   - Smart fallback: When omitted or blank, falls back to the system metadata banner:
     - EN: `"Layout: {mode} • Generations: {gens} • Persons: {count} • {date}"`
     - RU: `"Режим: {mode} • Поколений: {gens} • Персон: {count} • {date}"`

---

## 4. Permissions & Privacy

### 4.1 Master RBAC Access Control
Per [02 §2](02-rbac-access-control.md#2-master-segregation--rbac-permission-matrix), generating high-resolution PDF exports is an **Administrator-only** operation:

| Access Tier | UI Studio (`/export/print`) | Preview API (`POST .../preview-info`) | Export API (`POST .../export/pdf`) |
| :--- | :--- | :--- | :--- |
| **Unauthenticated** | Redirect to `/login` | `401 Unauthorized` | `401 Unauthorized` |
| **Viewer** | `403 Forbidden` | `403 Forbidden` | `403 Forbidden` |
| **Editor** | `403 Forbidden` | `403 Forbidden` | `403 Forbidden` |
| **Administrator** | Full Access | Full Access | Full Access |

### 4.2 Living Person Privacy Masking
- When `redact_living = true`:
  - Any living person has their vital dates masked: birth year is replaced with `"Living"` (or `"Живущий"` in RU).
  - Portrait photos are omitted and replaced by a neutral vector silhouette.

---

## 5. API Endpoints

### 5.1 Preview Info Calculation
- **Route:** `POST /api/v1/export/pdf/preview-info`
- **Role:** Administrator (`RequireRole::<Admin>`)
- **Payload:** `PdfExportRequest` (JSON)
- **Response:** `200 OK` with `PdfExportPreviewInfo` (JSON)
- **Error Codes:**
  - `400 Bad Request`: `invalid_generations`, `invalid_dimensions`, `invalid_view_mode`.
  - `403 Forbidden`: `insufficient_permissions`.
  - `404 Not Found`: `person_not_found`.

### 5.2 PDF Document Generation
- **Route:** `POST /api/v1/export/pdf`
- **Role:** Administrator (`RequireRole::<Admin>`)
- **Payload:** `PdfExportRequest` (JSON)
- **Response:** `200 OK` with binary `application/pdf` stream:
  - Header: `Content-Disposition: attachment; filename="family-tree-tiled.pdf"` (or `poster.pdf`).
  - Header: `Content-Type: application/pdf`.
- **Error Codes:**
  - `400 Bad Request`: `invalid_export_parameters`.
  - `403 Forbidden`: `insufficient_permissions`.
  - `404 Not Found`: `person_not_found`.
  - `500 Internal Server Error`: `pdf_render_error`.

---

## 6. Localization (EN & RU)

The PDF export subsystem adheres strictly to [00-internationalization-localization.md](00-internationalization-localization.md):
- **Copywriting Tone:** Zero marketing buzzwords and zero developer tech-stack branding in all studio UI controls and generated PDF text.
- **No Embedded Icons in Localization Strings:** Translation keys contain purely clean strings without embedded icon characters or emojis per `ui-and-i18n-conventions.md`.
- **Cyrillic Typography in Rust Engine:**
  - Embedded TrueType fonts in the Rust PDF renderer include full Unicode Cyrillic glyph sets (Inter / PT Serif / Roboto).
  - Patronymics (отчество) are rendered accurately on Russian-language cards: `Иван Иванович Иванов`.
- **Bilingual Fallback Strings:**
  - Fallback Subtitle EN: `"Layout: {mode} • Generations: {gens} • Persons: {count} • {date}"`
  - Fallback Subtitle RU: `"Режим: {mode} • Поколений: {gens} • Персон: {count} • {date}"`
  - Vital Dates EN: `"b. 1920 – d. 1985"`, `"Living"`
  - Vital Dates RU: `"р. 1920 – ум. 1985"`, `"Живущий"`

---

## 7. Edge Cases & Non-Goals

### 7.1 Edge Cases
1. **All Nodes Mode with Large Trees (500+ Persons):** Tiling engine partitions the canvas across large grids (e.g. $6 \times 4$ sheets); preview displays sheet warning if grid exceeds 30 sheets.
2. **Focal Person with No Recorded Partner:** Rendered singly without placeholder blanks.
3. **Multiple Partners:** Rendered symmetrically in Hourglass/Descendants without card overlap; canvas dynamically widens.
4. **Blank Title or Subtitle:** Automatically falls back to localized default title and metadata subtitle.
5. **Fan Chart Selection:** `include_partner`, `include_siblings`, and `collateral` toggles are disabled in the sidebar with an informative tooltip.

### 7.2 Non-Goals
- **WYSIWYG Drag-and-Drop Card Repositioning:** PDF layout is computed algorithmically; manual card dragging on the preview canvas is out of scope.
- **Direct PDF Vector In-Browser Editing:** The studio is a layout configuration and preview tool, not a vector drawing editor.

---

## 8. Definition of Done (DoD)

- [ ] **DTO & API Support:** `PdfExportRequest` accepts `view_mode: "all_nodes"`, `collateral: Option<bool>`, `include_partner: Option<bool>`, `include_siblings: Option<bool>`, `title: Option<String>`, and `subtitle: Option<String>`.
- [ ] **Hourglass Compact vs Wide Density:** Toggling between Compact and Wide modes dynamically adjusts collateral relative scope (aunts/uncles, cousins, nieces/nephews) on the canvas.
- [ ] **All Nodes Mode Generation:** Selecting `all_nodes` mode partitions the entire family tree into connected components and an isolated records shelf, calculating correct canvas bounds and sheet grids.
- [ ] **Partner Rendering with Ochre Link:** With `include_partner: true`, partners are positioned adjacent to focal individual on the generational baseline connected by a warm ochre marital bar.
- [ ] **Multiple Spouses Symmetry:** Individuals with 2 spouses render symmetrically (`[Spouse 1] === [Person] === [Spouse 2]`) with zero card collisions.
- [ ] **Sibling Rendering on Generation Baseline:** With `include_siblings: true`, siblings align on Generation 0, branching from the common parental distribution bus runner.
- [ ] **Portrait Photos Embedded at 300 DPI:** When `show_photos: true`, profile photos are rendered in circular crops on cards in both preview canvas and exported PDF.
- [ ] **Custom Title & Subtitle Banner:** Custom `title` and `subtitle` text entered in sidebar overrides the header banner; smart localized fallbacks render when fields are empty.
- [ ] **Fan Chart Compatibility Guard:** When `view_mode == "fan"`, partner, sibling, and collateral options are disabled in the UI and ignored in the backend.
- [ ] **Dynamic Sheet & Person Count Badges:** Summary bar instantly updates the Persons Count badge, total dimensions ($W \times H\text{ mm}$), and sheet grid matrix ($N \times M$) when layout options change.
- [ ] **Living Person Privacy Redaction:** When `redact_living: true`, living individuals' vital dates are replaced by `"Living"` / `"Живущий"` and photos replaced by neutral silhouettes.
- [ ] **Admin-Only Isolation:** Access to `/export/print` and `POST /api/v1/export/pdf` remains strictly isolated to Administrators; non-Admins receive `403 Forbidden`.
- [ ] **Localization Parity:** Studio controls, badges, and tooltips are fully localized in EN and RU with zero icon glyphs or emojis in translation files.

