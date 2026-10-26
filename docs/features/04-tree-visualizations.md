# Feature Specification: Interactive Tree Visualizations

## 1. Overview & Objective
Transform the family tree browsing experience from slow, static tables into a fast, fluid, and delightful visual canvas inspired by modern creative tools (Miro, Figma) and premium genealogy apps (MyHeritage, MacFamilyTree). The interface must support multiple layout projections and smooth 60fps pan/zoom interactions across desktop, tablet, and mobile devices.

---

## 2. Visualization Projections & Modes

### 2.1 Pedigree (Horizontal Ancestor) View
- **Structure:** Focal individual on the left; father above, mother below, expanding horizontally to the right across generational tiers.
- **Interactions:**
  - Branch expansion badges (`+` / `-`) indicating further ancestors.
  - Clicking any node can set it as the new focal center with smooth camera lerp (pan/zoom animation).

### 2.2 Hourglass (Family) View
- **Structure:** Focal individual and their spouse(s) placed in the center. Direct ancestors branch upwards; children and descendants branch downwards.
- **Use Case:** Best all-around overview for family gatherings, showing parents, grandparents, children, and grandchildren simultaneously.

### 2.3 Descendants Tree View
- **Structure:** Top-down vertical tree showing all descendants originating from a selected matriarch or patriarch ancestor.
- **Interactions:** Color-coded generational bands, collapsible sibling branches to focus on specific family lines.

### 2.4 Fan Chart (Radial Sunburst)
- **Structure:** Semi-circular or 360-degree circular concentric rings radiating from the focal individual (supports 4 to 8 generations).
- **Aesthetics:** Distinct color palettes for Paternal Grandfather (Blue), Paternal Grandmother (Green), Maternal Grandfather (Yellow/Amber), and Maternal Grandmother (Red/Coral).
- **Density:** Automatically scales font size and text truncation based on ring radius.

### 2.5 Kinship & Relationship Path Calculator
- **Function:** Solves "How are John Doe and Mary Smith related?".
- **Visualization:** Highlights the shortest genealogical path connecting the two individuals through their Most Recent Common Ancestor(s) (MRCA).
- **Text Summary:** Computes the official English/localized kinship degree (e.g. *"First cousin twice removed"*, *"Great-great-aunt"*).

---

## 3. Visual Design & Card Anatomy

```
+-----------------------------------------------------------+
| [Avatar]  Johnathan Alexander DOE                         |
|  (Photo)  b. 14 Mar 1912 - Boston, MA                     |
|           d. 22 Oct 1989 - Chicago, IL (Age 77)           |
|                                                           |
| [Quick Toolbar: 🔍 Profile | 🎯 Focus | ➕ Add | ✏️ Edit]  |
+-----------------------------------------------------------+
```

### 3.1 Card States & Styling
- **Default State:** Clean card with subtle border, subtle drop shadow, gender accent pill (soft slate/indigo for male, rose/coral for female, neutral grey for unspecified).
- **Hover / Active State:** Elevated card with glowing border, displaying the quick action toolbar.
- **Living Individual State:** Distinct badge (`Living`) with privacy-masked details (display name masked as single initial character with dot, e.g. `И.` / `J.`, with surnames, patronymics, dates, places, and media completely stripped) when viewed by unauthenticated public users.
- **Direct Lineage Highlighter:** Glowing connector lines tracing the direct biological ancestry path.

---

## 4. Canvas Navigation & Controls

### 4.1 Fluid Navigation
- **Pan & Zoom:** Smooth hardware-accelerated movement via pointer drag, mouse wheel, or mobile multi-touch pinch-to-zoom with natural velocity dampening (see [18 §3.6](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/18-progressive-web-app-mobile.md#36-interactive-tree-canvas-touch-gestures) for touch gesture specifications).
- **Mini-Map (Radar View):** Collapsible bottom-right viewport thumbnail displaying tree density and the current camera viewport rectangle (collapsed by default on mobile `< 768px`).
- **Camera Jump (Focal Lerp):** Selecting an individual smoothly glides the camera and centers them at optimal zoom level.

### 4.2 Toolbar Controls
- **Generation Depth Slider:** Quick filter to display 3, 4, 5, 6, or unlimited generations.
- **Search & Quick Jump:** Omnibox (`Cmd+K` / `Ctrl+K`) with instant fuzzy search by first name, surname, maiden name, or birth year; selecting a result flies directly to the node.
- **Reset View / Fit to Screen:** Instantly calculates bounding box of active nodes and fits within screen bounds.
- **Fullscreen Mode:** Maximizes canvas area for immersive exploration.

---

## 5. Technical Implementation Details

### 5.1 Rendering Technology
- **Frontend Engine:** SVG + React with `d3-hierarchy` / `dagre` layout algorithms for deterministic, non-overlapping tree layout calculation.
- **Styling & Design Tokens:** Pure Tailwind CSS v4 with custom theme tokens, modern card gradients, accessible DOM nodes, and responsive SVG connector curves.
- **Pan/Zoom:** `d3-zoom` with CSS `transform: translate3d(...)` for 60fps hardware-accelerated movement.
- **Virtualization / Culling:** For large trees (>1,000 visible nodes), off-screen cards are culled from DOM rendering to maintain smooth 60fps frame rates.

### 5.2 API Data Contract
- Endpoint: `GET /api/v1/tree/view?focal_id={uuid}&mode={pedigree|hourglass|fan|descendant}&generations={n}`
- Returns a normalized graph payload containing nodes (individuals with truncated preview fields) and edges (parent-child and spouse relationships).

---

## 6. Definition of Done (DoD)

- [ ] **Canvas Responsiveness:** Pan and zoom maintain consistent 60fps on modern desktop browsers and mobile touch screens.
- [ ] **All 5 View Modes Implemented:** Users can switch between Pedigree, Hourglass, Descendant, Fan Chart, and Kinship Path without page reloads.
- [ ] **Dynamic Layout Calculation:** Tree nodes never overlap; multiple marriages and complex sibling lines are routed with clean orthogonal or bezier connector curves.
- [ ] **Camera Transitions:** Changing focal individual smoothly animates camera position to the target node.
- [ ] **Omnibox Search:** `Ctrl+K` search finds individuals within <100ms and smoothly centers the target on selection.
- [ ] **Touch Gesture Support:** Native pinch-to-zoom and two-finger panning work smoothly on iOS and Android devices; mobile card inspection opens a swipeable bottom sheet preserving tree context ([18](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/18-progressive-web-app-mobile.md)).
