# Feature Specification: Printable PDF Tree Export (Single & Multi-Page Tiled)

## 1. Overview & Objective
Enable administrators to transform digital family trees into high-resolution, physical keepsakes. Generating and exporting printable PDFs is restricted exclusively to the **Administrator** role per [02 §2](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md#2-master-segregation--rbac-permission-matrix) (to prevent bulk extraction of family data by viewers or unauthenticated users).

The system generates professional-grade, high-resolution vector PDF documents in two distinct printing formats:
1. **Single-Page Poster:** Standard large-format engineering/poster sizes (A0, A1, A2, A3) suitable for professional print shops.
2. **Multi-Page Tiled PDF:** Automatically sliced into a grid of standard home-printer pages (A4 or US Letter) complete with **cut lines, overlap margins, and assembly coordinates** so family members can easily trim and glue them together.

---

## 2. Print Formats & Page Tiling Engine

### 2.1 Single-Page Poster Mode
- **Supported Dimensions:** A0 (841 x 1189 mm), A1 (594 x 841 mm), A2 (420 x 594 mm), A3 (297 x 420 mm), or Custom Width/Height.
- **Resolution:** 100% vector typography and connector curves; photos downsampled to 300 DPI for crisp print quality without astronomical file sizes.
- **Framing & Margins:** Configurable decorative border, title banner ("The Ancestors of [Focal Person]"), generation counter, and creation date.

### 2.2 Multi-Page Tiled Mode (Print & Assemble at Home)
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

#### Tiling & Assembly Features:
1. **Overlap Margins (Bleed):** Each page includes a configurable overlap (default: 10mm / 0.4 inches) where adjacent artwork duplicates slightly.
2. **Crop Marks / Cut Lines:** Thin dashed line and scissors icon indicating the exact line to slice with a hobby knife or paper trimmer.
3. **Alignment Crosshairs (`+`):** Corner crosshair marks to ensure pinpoint alignment when pasting sheet over sheet.
4. **Assembly Header / Footer:** Subtle text outside the printable area:
   - Example: `Sheet 5 of 9 [Row 2, Column 2] — Paste Right of Sheet 4, Below Sheet 2`.
5. **Assembly Guide Page (Sheet 1):** Optional first page showing a mini bird's-eye map of the entire grid with instructions on how to glue the sheets together.

---

## 3. Visual Themes & Customization

Users can customize the visual aesthetic before generating the PDF:

| Theme Name | Background | Typography | Connectors | Best For |
| :--- | :--- | :--- | :--- | :--- |
| **Modern Clean** | Pure White / Subtle Cream | Sans-serif (Inter / Roboto) | Sleek 1px slate lines | Modern family reunions, minimalist frames |
| **Heritage Antique** | Aged parchment texture / Warm ivory | Classical Serif (Playfair / Merriweather) | Warm sepia / flourishes | Formal genealogy records, vintage gifts |
| **Botanical Fan** | Radial fan chart palette | Elegant modern condensed | Multi-color lineage rings | Colorful 5-generation ancestry displays |

### Configurable Options:
- **Tree Style:** Pedigree, Descendants, Hourglass, or Fan Chart.
- **Generation Depth:** 3 to 8 generations.
- **Content Density:** Show/hide dates, show/hide places, show/hide portrait photos.
- **Living Person Masking:** Checkbox to redact living individuals' exact birth years.

---

## 4. Technical Architecture & PDF Engine

### 4.1 Generation Strategy (Rust Engine)
- **Rust Layout & Slicing Pipeline (`printpdf` / `typst` / Headless SVG):**
  - The tree hierarchy coordinates, connector splines, and typography are computed geometrically in millimeter units.
  - The vector tree is rendered onto an internal vector canvas with crisp vector text (TrueType font embedding) and 300 DPI embedded images.
  - **Single Poster:** The entire canvas is wrapped in an A0/A1/A2/A3 page container with decorative margins.
  - **Tiled Multi-Page Mode:** The layout engine projects a bounding box clipping window of size $(W_{page} - 2 \cdot M_{margin}) \times (H_{page} - 2 \cdot M_{margin})$ across the $N \times M$ grid matrix with overlap bleed ($10\text{mm}$).
  - In each grid tile cell, the engine injects:
    1. Clipped tree content.
    2. Overlap shaded margin with alignment crosshairs.
    3. Dashed crop lines (`stroke-dasharray="4 2"`).
    4. Coordinate identification footer (`Sheet X of Y [Row R, Col C]`).
- **Performance:** Native Rust streaming writes directly to the HTTP response stream; a 6-generation tree exports in under 800ms with negligible RAM overhead.

### 4.2 API Data Contract
- `POST /api/v1/export/pdf`
- Request Payload:
```json
{
  "focal_person_id": "uuid",
  "view_mode": "pedigree",
  "generations": 5,
  "theme": "modern",
  "format": "tiled",
  "target_sheet_size": "A4",
  "overlap_mm": 10,
  "show_crop_marks": true,
  "show_photos": true,
  "redact_living": true,
  "title": "The Levshits Family Tree"
}
```
- Response: `application/pdf` binary stream with `Content-Disposition: attachment; filename="family-tree-tiled.pdf"`.

---

## 5. Definition of Done (DoD)

- [ ] **Single-Page PDF Generation:** Generates valid, printable A0, A1, A2, and A3 PDF files with sharp vector lines and fonts at 300 DPI.
- [ ] **Multi-Page Tiling Math:** Large tree is cleanly partitioned across an $N \times M$ grid of A4 / Letter pages without dropped content.
- [ ] **Assembly Markers:** Every page in tiled mode features dashed cut lines, corner crosshair marks, and clear coordinates (e.g. `Row 2, Column 3`).
- [ ] **Overlap Margins:** Adjacent pages share the specified overlap margin (e.g. 10mm) allowing clean seamless gluing.
- [ ] **Theme Support:** User can switch between at least two distinct visual themes (Modern Clean and Heritage Antique).
- [ ] **Photo Inclusion:** Profile photos are properly cropped into circles/rounded rectangles and embedded in the PDF without pixelation.
- [ ] **Preview Modal:** Before downloading, user sees a visual preview displaying sheet counts (e.g., *"This tree will print across 6 sheets (3x2 grid) of A4 paper"*).
- [ ] **Admin-Only Isolation:** Access to `/export/print` and `POST /api/v1/export/pdf` is restricted exclusively to Administrators; non-Admins receive 403 Forbidden.
