# Page Specification: Printable PDF Studio (`/export/print`)

## 1. Overview & Objective
The Printable PDF Studio allows administrators to configure, preview, and generate wall charts and home-printable tiled posters with complete feature parity to the interactive Tree View. It uses a **Dual-Pane Studio Layout** with an interactive configuration sidebar on the left and a live real-time preview canvas on the right.

> [!IMPORTANT] Administrator-Only Isolation:
> Generating printable PDF posters and wall charts (`/export/print` and `POST /api/v1/export/pdf`) is an **Administrator-only administrative tool** (`RequireRole::<Admin>`). To prevent heavy background rendering abuse and bulk data harvesting, this page is inaccessible to Viewers and Editors (`403 Forbidden`) as well as unauthenticated visitors (redirected to `/login`).

---

## 2. Page Wireframe & Component Layout

```
+----------------------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]                      |
+----------------------------------------------------------------------------------------------------+
| PRINTABLE PDF STUDIO                                                                               |
+---------------------------------------+------------------------------------------------------------+
| CONFIGURATION SIDEBAR (Left, w-96)    | LIVE PDF PREVIEW CANVAS (Right)                            |
|                                       |                                                            |
| Focal Individual:                     | +-- TOOLBAR: [Layout Map | PDF View] [-] [ 100% ] [+] [⛶] +|
| [ Aleksandr Petrov              🔍 ]  | |                                                          |
|                                       | | SUMMARY BAR:                                             |
| Chart Layout:                         | | [📄 4 Sheets (2×2 Grid)] [📐 594 × 420 mm]               |
| [ Pedigree | Hourglass | Descendants ]| | [✂️ 10 mm Overlap] [👥 18 Persons] [🛡️ Living Masked]     |
| [ All Nodes | Fan Chart             ] | +----------------------------------------------------------+
|                                       | | PREVIEW CANVAS                                           |
| Tree Density (Hourglass):             | |  Page 1 (R1,C1)             :   Page 2 (R1,C2)           |
| (•) Wide (Collateral)  ( ) Compact    | |   + - - - - - - - - - - - - - - - - - - - - - - - - +    |
|                                       | |   |               The Levshits Family Tree          |    |
| Generation Depth:                     | |   |       Commemorative reunion edition • Aug 2026  |    |
| [ 5 Generations                     ] | |   |                                                 |    |
|                                       | |   |                 [Grandparents]                  |    |
| Printing Format:                      | |   |                     |     |                     |    |
| (•) Multi-Page Tiled (Home Print)     | |   |                 [ Parents ]                     |    |
| ( ) Single Poster (Print Shop)        | |   |             +-------+-----+-------+             |    |
|                                       | |   |             |                     |             |    |
| Target Sheet Size: [ A4 Paper      ▼] | |   | [Sibling]   | [Focal] === [Spouse]|   [Sibling] |    |
| Overlap Bleed: [ 10 mm             ▼] | |   |   (photo)   |  (photo)     (photo)|    (photo)  |    |
| [✓] Show crop marks & crosshairs      | |   |             +-------+-----+-------+             |    |
| [ ] Include assembly guide on Sheet 1 | |   |                     |                           |    |
|                                       | |   |                 [Children]                      |    |
| Visual Theme:                         | |   + - - - - - - - - - - - - - - - - - - - - - - - - +    |
| (•) Modern Clean  ( ) Heritage        | |  Page 3 (R2,C1)             :   Page 4 (R2,C2)           |
|                                       | +----------------------------------------------------------+
| Relatives & Content Density:          |                                                            |
| [✓] Include spouse / partner          | ℹ️ Dimensions: 594 × 420 mm (A2 equivalent)                |
| [✓] Include siblings                  | 📄 Output: 4 Sheets (2×2 Grid) of A4 Paper                 |
| [✓] Show portrait photos              | ✂️ Overlap: 10 mm with alignment crop marks                |
| [ ] Redact living individuals         | 👥 Persons: 18 individuals included on canvas              |
|                                       |                                                            |
| Chart Title:                          |                                                            |
| [ The Levshits Family Tree          ] |                                                            |
|                                       |                                                            |
| Description / Subtitle:               |                                                            |
| [ Commemorative reunion edition...  ] |                                                            |
| ℹ️ Leave blank for automatic summary  |                                                            |
|                                       |                                                            |
| [ 📥 Download Printable PDF         ] |                                                            |
+---------------------------------------+------------------------------------------------------------+
```

### Components Used:
- `AppHeader` ([design-system/components.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md)): Standard site header with navigation, user avatar, and global search.
- `PersonPicker` ([pages/03-person-profile-page.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/03-person-profile-page.md)): Autocomplete search input to select the focal center individual.
- `SegmentedControl` ([design-system/components.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md)): Layout projection picker (`pedigree`, `hourglass`, `descendants`, `all_nodes`, `fan`), density selector (`wide` vs `compact`), and theme selector (`modern` vs `heritage`).
- `RadioGroup` & `RadioOption`: Format selection (Tiled Home Print vs Single Poster).
- `Select` & `Slider`: Paper sheet size (`A4`, `Letter`, `A0`–`A3`) and overlap bleed margin ($5\text{--}20\text{ mm}$).
- `Checkbox`: Toggles for crop marks, assembly guide, partner inclusion, sibling inclusion, photos, and living person redaction.
- `TextInput`: Dedicated input fields for custom Headline Title and custom Description / Subtitle.
- `PrintSummaryBar`: Responsive strip of badges (`Badge`) summarizing output sheets, dimensions, overlap, person count, and privacy status.
- `PrintPreviewCanvas`: High-fidelity SVG canvas with zoom/pan controls, tile boundary guides, card avatars, and connector lines.
- `Button`: Sticky primary action CTA for streaming PDF download.

---

## 3. Structural Components

### 3.1 Configuration Sidebar (Left Pane, `w-96`)

#### 1. Focal Individual Picker
- Searchable autocomplete input defaulting to the authenticated administrator's profile person or the tree root.
- Optional in `all_nodes` mode (acts as the primary highlighted landmark anchor).

#### 2. Chart Layout Projection
- Segmented control options:
  - **Pedigree:** Horizontal ancestor tree (Left-to-Right).
  - **Hourglass:** Bilateral center-outward tree (Ancestors above, focal in center, descendants below).
  - **Descendants:** Top-down descendant waterfall tree.
  - **All Nodes (`all_nodes`):** Complete family forest graph view with multi-component partitioning and isolated records shelf.
  - **Fan Chart:** Concentric radial ancestry chart (180° / 360°).

#### 3. Tree Density / Collateral Mode (Shown in Hourglass Mode)
- Segmented control:
  - **Wide (Collateral) (`collateral = true`, Default):** Expands ancestral tree to encompass collateral lines (aunts/uncles on Gen -1, first cousins and siblings on Gen 0, nieces/nephews on Gen +1).
  - **Compact (`collateral = false`):** Confines tree strictly to direct lineal ascendants and direct lineal descendants.

#### 4. Generation Depth
- Segmented control or numeric stepper from 1 to 8 generations (default: 5).
- **In All Nodes mode:** Generation depth is hidden, replaced by a static information badge indicating total graph individuals.

#### 5. Printing Format
- **Multi-Page Tiled (Home Print):**
  - Target Sheet Size: `A4` or `US Letter`.
  - Overlap Margin Slider: $5\text{ mm}$ to $20\text{ mm}$ (default: $10\text{ mm}$).
  - Checkbox: `[x] Show dashed cut lines and corner crosshairs`.
  - Checkbox: `[ ] Include Assembly Overview Guide on Sheet 1`.
- **Single Poster (Print Shop):**
  - Target Poster Size: `A0`, `A1`, `A2`, `A3`, or `Custom`.

#### 6. Visual Theme
- `Modern Clean`: Crisp sans-serif fonts, clean 1px slate borders, soft white / subtle cream background.
- `Heritage Antique`: Warm ivory parchment texture, classical serif typography, sepia flourish connectors.

#### 7. Relatives & Content Density
- **`[✓] Include spouse / partner` (`include_partner`)**:
  - **Default:** Checked (`true`).
  - **Behavior:** Includes the focal person's recorded spouse/partner(s) on the generational baseline with an ochre marital link. In Descendant mode, also displays married partners of descendants.
  - **Fan Chart Guard:** Disabled with tooltip when Fan Chart mode is active.
- **`[ ] Include siblings` (`include_siblings`)**:
  - **Default:** Unchecked (`false`).
  - **Behavior:** Includes full and half siblings of the focal person on Generation 0, ordered chronologically and branching from the common parental union distribution bus runner.
  - **Fan Chart Guard:** Disabled with tooltip when Fan Chart mode is active.
- **`[✓] Show portrait photos` (`show_photos`)**:
  - **Default:** Checked (`true`). Embeds 300 DPI circular profile portraits on cards in both preview canvas and PDF export.
- **`[ ] Redact living individuals` (`redact_living`)**:
  - **Default:** Unchecked (`false`). When enabled, masks birth dates to `"Living"` / `"Живущий"` and replaces photos with neutral vector silhouettes.

#### 8. Customizable Title & Subtitle Banner
- **Chart Title (`title: Option<String>`):**
  - Text input allowing custom headline override (e.g. *"The Levshits Family Tree"*).
  - Placeholder: localized default e.g. `"The Petrov Family Tree"`.
  - Smart Fallback: When left blank or cleared, defaults to `"{instance_tree_title}: {focal_name}"` or `"Family Tree of {focal_name}"`.
- **Description / Subtitle (`subtitle: Option<String>`):**
  - Text input for custom description or commemorative subtitle (e.g. *"Family reunion keepsake • August 2026"*).
  - Placeholder: `"e.g. Commemorative family reunion keepsake"`.
  - Helper note: *"Leave blank to use the automatic layout summary"*.
  - Smart Fallback: When left blank, falls back to the system metadata banner:
    - EN: `"Layout: {mode} • Generations: {gens} • Persons: {count} • {date}"`
    - RU: `"Режим: {mode} • Поколений: {gens} • Персон: {count} • {date}"`

#### 9. Sticky Primary Action Button
- Fixed bottom container with prominent `Download Printable PDF` (`Button variant="primary"`).
- Displays spinning loader and text `"Generating PDF..."` during native binary streaming.

---

### 3.2 Live Interactive Preview Canvas (Right Pane)

#### 3.2.1 Real-Time Canvas Rendering
- Connects to `POST /api/v1/export/pdf/preview-info` via debounced query ($200\text{ ms}$).
- Dynamically renders exact vector cards, photos, and connectors:
  - **Custom Title & Subtitle Banner:** Displayed prominently in the top banner area with active font styles (Modern Clean vs Heritage Antique). Updates live as the user types in the sidebar fields.
  - **Partner Cards:** Rendered directly adjacent to the focal person on the baseline ($Y = 0$), connected with a warm ochre marital connector (`--color-brand-accent`). Multiple spouses render symmetrically (`[Spouse 1] === [Focal] === [Spouse 2]`).
  - **Sibling Cards:** Rendered along Generation 0 flanking the focal couple, hanging from the common parental distribution bus runner.
  - **Profile Photos:** When `show_photos = true`, renders circular cropped avatars on cards; unassigned avatars display vector monogram initials.
  - **All Nodes Forest View:** In `all_nodes` mode, renders all connected components spaced by $\ge 30\text{ mm}$ and isolated singletons in a 4-column shelf grid below.
  - **Tiling Grid Boundaries:** In Tiled mode, renders subtle dashed sheet outlines with coordinate labels (`Sheet 1 of 4: Top-Left [R1, C1]`) and shaded overlap margins with alignment crop marks.
- **Canvas Toolbar:**
  - View switcher: `Layout Map` (interactive SVG preview) vs `PDF View` (embedded browser PDF viewer).
  - Zoom controls: Zoom In (`+`), Zoom Out (`-`), Fit to Viewport (`⛶`), and Reset Scale (`100%`).

#### 3.2.2 Live Summary Bar (`PrintSummaryBar`)
A responsive horizontal badge strip at the top of the canvas:
1. **Total Sheets Badge (`Badge variant="brand"`):** Displays sheet count and grid layout (e.g., `📄 4 Sheets (2×2 Grid) of A4 Paper` or `📄 1 Single Poster Sheet (A1)`).
2. **Physical Dimensions Badge (`Badge variant="neutral"`):** Shows computed millimeters and standard paper size equivalent (e.g., `📐 594 × 420 mm (A2 equivalent)`).
3. **Overlap Bleed Badge (`Badge variant="neutral"`):** Indicates active overlap (e.g., `✂️ 10 mm Overlap with Cut Lines`).
4. **Persons Count Badge (`Badge variant="neutral"`):** Displays total individual cards on the canvas: `👥 18 Persons`. Increases immediately when partners, siblings, or collateral branches are enabled.
5. **Privacy Badge (`Badge variant="warning" | "neutral"`):** Indicates `🛡️ Full Details` or `🛡️ Living Persons Masked`.
6. **Live Loading Indicator:** Subtle spinner with text *"Calculating preview..."* during layout recomputation.

---

### 3.3 State Management & Interactions

| State | User Action | System Response |
| :--- | :--- | :--- |
| **Initial Mount** | Admin opens `/export/print` | Resolves home person or tree root; queries `POST .../preview-info`; renders initial preview with default headline banner and metadata subtitle. |
| **Toggle Density (Wide/Compact)** | Admin switches between Wide and Compact in Hourglass | Updates `collateral` parameter; debounced preview expands/contracts collateral branches (aunts/uncles/cousins); Persons badge and sheet grid recalculate. |
| **Select All Nodes** | Admin clicks `All Nodes` layout | Bypasses generation depth; calculates complete multi-component forest; canvas displays all archive individuals and isolated records shelf. |
| **Toggle Partner** | Admin checks/unchecks `Include spouse / partner` | Sidebar updates config state; debounced preview adds/removes partner card and ochre connector; Persons badge and sheet grid recalculate. |
| **Toggle Siblings** | Admin checks/unchecks `Include siblings` | Sidebar updates config state; debounced preview adds/removes sibling cards along Gen 0 branching from parental bus runner. |
| **Toggle Photos** | Admin checks/unchecks `Show portrait photos` | Toggles avatar rendering on cards across preview canvas and generated PDF. |
| **Type Custom Title / Subtitle** | Admin types custom headline or description | Banner on canvas updates in real time; clearing field immediately restores localized smart fallback default. |
| **Switch to Fan Chart** | Admin clicks `Fan Chart` layout | Sidebar disables partner, sibling, and collateral density controls with helper tooltip; canvas recalculates to concentric radial fan. |
| **Click Download** | Admin clicks `Download Printable PDF` | Triggers `POST /api/v1/export/pdf`; browser initiates native file streaming download of the high-res vector PDF with cut marks, photos, and custom banner included. |

---

## 4. Definition of Done (DoD)

- [ ] **Administrator-Only Isolation:** Access to `/export/print` and `POST /api/v1/export/pdf` is restricted exclusively to Administrators; unauthenticated visitors are redirected to `/login`, and Viewers/Editors receive `403 Forbidden`.
- [ ] **Dual-Pane Responsiveness:** Configuration on left (`w-96`), live interactive preview on right; stacks cleanly on small screens with a tab switcher (`Config` / `Preview`).
- [ ] **All Nodes Mode Support:** Selecting `All Nodes` layout renders the entire unpruned genealogical forest with connected components and isolated records shelf.
- [ ] **Hourglass Density Toggle:** Switching between `Wide (Collateral)` and `Compact` dynamically adds/removes aunts, uncles, cousins, and nieces/nephews.
- [ ] **Partner & Sibling UI Controls:** Checkboxes for "Include spouse / partner" (default `true`) and "Include siblings" (default `false`) are functional under "Relatives & Content Density".
- [ ] **Custom Title & Subtitle Inputs:** Sidebar provides dedicated inputs for "Chart Title" and "Description / Subtitle"; live canvas banner updates dynamically, with localized fallback defaults when left empty.
- [ ] **Embedded Photos in Preview & PDF:** Profile photos are rendered as 300 DPI circular crops on cards in both preview canvas and exported PDF when `show_photos` is enabled.
- [ ] **Fan Chart Compatibility Guard:** When Fan Chart layout is selected, partner, sibling, and density controls are visually disabled with an explanatory tooltip.
- [ ] **Live Preview Partner Rendering:** Partner cards are rendered adjacent to the focal person on the generational baseline with warm ochre marital connectors (`#d97706`).
- [ ] **Live Preview Sibling Rendering:** Sibling cards align along Generation 0 with stems branching from the common parental distribution bus runner.
- [ ] **Accurate Dynamic Badges:** Summary bar instantly updates the Persons Count badge, total dimensions ($W \times H\text{ mm}$), and sheet grid matrix ($N \times M$) when layout options change.
- [ ] **Theme Switching in Preview:** Switching between Modern Clean and Heritage Antique instantly updates canvas fonts, colors, and border styling.
- [ ] **Fast Streaming Download:** Clicking "Download Printable PDF" streams the generated vector file with valid cut marks, photos, and custom headers in <2 seconds.
- [ ] **UI & i18n Quality:** All copy and labels are bilingual (EN and RU) with zero hardcoded emojis or icons in translation bundles (`en.json` / `ru.json`).


