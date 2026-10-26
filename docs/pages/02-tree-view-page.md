# Page Specification: Interactive Family Tree Canvas (`/tree`)

## 1. Overview & Objective
The Tree Canvas is the primary landing view of the application. It occupies **100% of the viewport height and width below the topbar**, providing a clutter-free, hardware-accelerated SVG infinite canvas with floating control overlays.

---

## 2. Page Wireframe & Component Layout

```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]      |
+------------------------------------------------------------------------------------+
| [Floating View Switcher]                                    [Floating Depth & Fit] |
| [Pedigree | Hourglass | Descendants | Fan | Kinship]        [Generations: 4 ▼] [⛶] |
|                                                                                    |
|                                                                                    |
|                     [ SVG INFINITE PAN / ZOOM CANVAS ]                             |
|                                                                                    |
|                 +---------------+        +---------------+                         |
|                 | Grandfather   |        | Grandmother   |                         |
|                 +-------+-------+        +-------+-------+                         |
|                         \               /                                          |
|                          +------+------+                                           |
|                          |   Father    |                                           |
|                          +------+------+                                           |
|                                 |                                                  |
|                        +--------+--------+                                         |
|                        |  Focal Person   | <--- (Selected Node)                    |
|                        +-----------------+                                         |
|                                                                                    |
| [Floating Zoom Controls]                                       [Collapsible Radar] |
| [ + | 100% | - | ⌂ ]                                          [     Mini-Map    ] |
+------------------------------------------------------------------------------------+
| SLIDE-OVER QUICK DRAWER (Opens when a person card is clicked)                      |
| [X] Aleksandr Ivanovich Petrov • 1912 - 1989                                       |
| [Avatar] [Key Milestones] [Spouses & Children] [View Full Profile ->]             |
+------------------------------------------------------------------------------------+
```

---

## 3. Structural Components

### 3.1 Top Floating View Mode Switcher
Positioned top-left (`absolute top-4 left-6 z-20`):
- Segmented control pill (`bg-surface/90 backdrop-blur-md border border-border shadow-sm rounded-lg p-1`):
  - **Pedigree** (Horizontal direct ancestors)
  - **Hourglass** (Parents above, children below)
  - **Descendants** (Generational cascade)
  - **Fan Chart** (Concentric circular chart)
  - **Kinship Finder** (Relationship path between two persons)

### 3.2 Top Floating Generation & Framing Controls
Positioned top-right (`absolute top-4 right-6 z-20`):
- **Generation Depth Dropdown:** Quick selector for `3`, `4`, `5`, `6`, or `All` generations.
- **Center on Focal Person Button (`⌂`):** Recenter and reset zoom.
- **Fit View to Screen Button (`⛶`):** Computes bounding box of all visible nodes and scales smoothly.

### 3.3 Bottom-Left Floating Zoom Bar
Positioned bottom-left (`absolute bottom-6 left-6 z-20`):
- Vertical or horizontal button pill:
  - Zoom In (`+`)
  - Current Zoom Percentage Badge (e.g. `100%`, clicking resets to 100%)
  - Zoom Out (`-`)

### 3.4 Bottom-Right Interactive Mini-Map (Radar View)
Positioned bottom-right (`absolute bottom-6 right-6 z-20`):
- Compact thumbnail viewport (`w-48 h-36 bg-surface/80 backdrop-blur border border-border rounded-lg shadow-sm`).
- Shows dense dot matrix of the entire family graph with an outline rectangle indicating the current camera viewport.
- Dragging the rectangle translates the main canvas camera in real-time.
- Can be collapsed into a small icon to maximize canvas real estate.

### 3.5 Selected Person Slide-Over Drawer
- Clicking any node opens a right slide-over panel (`w-96` or `w-[440px]`):
  - **Header:** Person's photo avatar, full name with patronymic, sex badge, life dates.
  - **Core Facts:** Birth, baptism, marriage, death dates and geocoded places.
  - **Immediate Family Links:** Clickable list of parents, spouses, and children to jump directly.
  - **Media Preview:** 3 recent tagged photos.
  - **Actions:**
    - *"View Full Profile"* (navigates to `/persons/:id`).
    - *"Make Focal Person"* (re-roots active tree around this individual).
    - *"Add Relative"* (opens fast in-canvas drawer: Parent, Spouse, Child).

---

## 4. Definition of Done (DoD)

- [ ] **Full Viewport Canvas:** Canvas stretches to full screen height with no scrollbars on the parent container.
- [ ] **Smooth Mode Switching:** Changing visualization mode transitions smoothly without page reload or lost selection.
- [ ] **Quick Drawer Inspection:** Clicking any person node opens the slide-over drawer in <150ms while keeping the tree interactive in the background.
- [ ] **Responsive Overlays:** On mobile devices, floating toolbars condense into a floating bottom sheet to avoid covering the canvas.
