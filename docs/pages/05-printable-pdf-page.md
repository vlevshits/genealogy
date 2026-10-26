# Page Specification: Printable PDF Studio (`/export/print`)

## 1. Overview & Objective
The Printable PDF Studio allows users to configure, preview, and generate wall charts and home-printable tiled posters. It uses a **Dual-Pane Studio Layout** with an interactive configuration sidebar on the left and a live real-time preview canvas on the right.

> [!IMPORTANT] Administrator-Only Isolation:
> Generating printable PDF posters and wall charts (`/export/print` and `POST /api/v1/export/pdf`) is an **Administrator-only administrative tool** (`RequireRole::<Admin>`). To prevent heavy background rendering abuse and bulk data harvesting, this page is inaccessible to Viewers and Editors (`403 Forbidden`) as well as unauthenticated visitors (redirected to `/login`).

---

## 2. Page Wireframe & Layout

```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]      |
+------------------------------------------------------------------------------------+
| PRINTABLE PDF STUDIO                                                               |
+------------------------------------+-----------------------------------------------+
| CONFIGURATION SIDEBAR (Left)       | LIVE PDF PREVIEW CANVAS (Right)               |
|                                    |                                               |
| Focal Individual:                  | +-------------------------------------------+ |
| [ Aleksandr Petrov           🔍 ]  | |  Page 1 (R1,C1)   :   Page 2 (R1,C2)      | |
|                                    | |   + - - - - - - - - - - - - - - - - - - + | |
| Chart Layout:                      | |   |                 :                   | | |
| (•) Pedigree  ( ) Hourglass        | |   |   Levshits Family Tree              | | |
| ( ) Descendants ( ) Fan Chart      | |   |                 :                   | | |
|                                    | |   + - - - - - - - - - - - - - - - - - - + | |
| Generation Depth: [ 5 Generations] | |  Page 3 (R2,C1)   :   Page 4 (R2,C2)      | |
|                                    | +-------------------------------------------+ |
| Printing Format:                   |                                               |
| (•) Multi-Page Tiled (Home Print)  | ℹ️ Dimensions: 594 × 420 mm (A2 equivalent)   |
| ( ) Single Poster (Print Shop)     | 📄 Output: 4 Sheets (2×2 Grid) of A4 Paper    |
|                                    | ✂️ Overlap: 10 mm with alignment crop marks   |
| Target Sheet Size: [ A4 Paper   ▼] |                                               |
| Overlap Bleed: [ 10 mm          ▼] |                                               |
|                                    |                                               |
| Visual Theme:                      |                                               |
| (•) Modern Clean  ( ) Heritage     |                                               |
|                                    |                                               |
| Privacy:                           |                                               |
| [✓] Redact Living Persons          |                                               |
|                                    |                                               |
| Language: [ Русский (RU)        ▼] |                                               |
|                                    |                                               |
| [ 📥 Download Printable PDF ]      |                                               |
+------------------------------------+-----------------------------------------------+
```

---

## 3. Structural Sections

### 3.1 Configuration Sidebar (Left Pane, `w-96`)
- **Focal Person Picker:** Searchable autocomplete input; defaults to the active root ancestor or current user.
- **Chart Style Segmented Buttons:**
  - Pedigree (Horizontal ancestors)
  - Hourglass (Ancestors + Descendants)
  - Descendant Tree
  - Colorful Radial Fan Chart
- **Printing Format Radio Group:**
  - **Multi-Page Tiled:** Prints on standard home desktop printers (A4 or US Letter); engine adds cut marks and assembly coordinates.
  - **Single Poster:** Creates a continuous high-res vector file for professional wide-format print shops (A0, A1, A2, A3).
- **Home Tiling Options (Shown when Tiled is active):**
  - Target Paper Size: `A4` or `US Letter`.
  - Overlap Margin: Slider from `5mm` to `20mm` (default: `10mm`).
  - [x] Print dashed cut lines and corner crosshairs.
  - [x] Include Assembly Overview Guide on Sheet 1.
- **Styling Theme Selector:**
  - `Modern Clean`: Minimalist sans-serif, crisp slate borders, soft white background.
  - `Heritage Antique`: Warm parchment background, classical serif titles, sepia flourishes.
- **Privacy & Language:**
  - Checkbox: *"Mask living individuals' dates & photos"*.
  - Language Dropdown: `English` or `Русский` (ensures localized kinship labels and Cyrillic Unicode fonts).

### 3.2 Live Interactive Preview Canvas (Right Pane)
- Renders an exact scaled preview of the PDF layout.
- In Tiled Mode, displays subtle dashed boundary lines between pages with page coordinate labels (*"Sheet 1 of 4: Top-Left"*).
- Summary Badge: Displays total sheets, grid layout ($N \times M$), total physical dimensions in centimeters/inches, and estimated print time.
- **Download Action Button:** Prominent floating CTA triggers instant native streaming generation from the backend.

---

## 4. Definition of Done (DoD)

- [ ] **Administrator-Only Isolation:** Access to `/export/print` and `POST /api/v1/export/pdf` is restricted exclusively to Administrators; unauthenticated visitors are redirected to `/login`, and Viewers/Editors receive `403 Forbidden`.
- [ ] **Dual-Pane Responsiveness:** Configuration on left, live interactive preview on right; stacks cleanly on small screens.
- [ ] **Accurate Sheet Estimation:** The UI dynamically calculates and displays the exact sheet count and grid layout before downloading.
- [ ] **Theme Switching in Preview:** Switching between Modern Clean and Heritage Antique instantly updates the preview styling.
- [ ] **Fast Streaming Download:** Clicking "Download Printable PDF" streams the generated vector file with valid cut marks in <2 seconds.
