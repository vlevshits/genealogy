# Page Specification: Interactive Family Tree Canvas (`/tree`)

## 1. Overview & Objective
The Tree Canvas is the interactive visual heart of the application. It occupies **100% of the viewport height and width below the topbar**, providing a clutter-free, hardware-accelerated SVG infinite canvas with floating control overlays.

> [!IMPORTANT] Tier-Based Access & Privacy Behavior:
> - **Default Private Instance (`ENABLE_PUBLIC_ACCESS=false`):** Unauthenticated visitors cannot access `/tree` and are immediately redirected to `/login`.
> - **Public Tree Mode (`ENABLE_PUBLIC_ACCESS=true`, Unauthenticated):** Public visitors see the tree canvas in **Strictly Masked Mode**. Living persons display as `Living` / `Живущий(ая)` with silhouette avatars, and zero dates, places, notes, or photos. Canvas editing controls, add buttons, and search omnibox are hidden.
> - **Authenticated Viewers (`Viewer`):** Trusted family members see the complete tree with all living relatives' data (names, patronymics, dates, places, photos) fully visible. Canvas is strictly read-only (node add handles and edit triggers are hidden).
> - **Authenticated Editors & Admins (`Editor`, `Admin`):** Full living person visibility plus interactive node edit handles, quick-add relative triggers (`+`), and contextual menus.

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
|                          |                                                  |
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

### 3.5 Selected Person Inspection (Drawer vs. Mobile Bottom Sheet)
- **Desktop (`> 1024px`):** Right slide-over panel (`w-[440px]`):
  - **Header:**
    - Public Mode (Living): Shows silhouette and `Living` / `Живущий`.
    - Authenticated / Deceased: Photo avatar, full name with patronymic, sex badge, life dates.
  - **Core Facts:**
    - Public Mode (Living): Suppressed completely (empty state: *"Private details hidden"*).
    - Authenticated / Deceased: Birth, baptism, marriage, death dates and geocoded places.
  - **Immediate Family Links:** Clickable list of parents, spouses, and children to jump directly.
  - **Media Preview:**
    - Public Mode: 3 recent tagged photos of deceased individuals only; photos of living persons return silhouette placeholders.
    - Authenticated: 3 recent tagged photos.
  - **Actions:**
    - Public Mode: *"View Full Profile"* (available for deceased only; living links redirect to login).
    - Authenticated Viewers: *"View Full Profile"* (`/persons/:id`), *"Make Focal Person"*. Write actions (`+ Add Relative`, `Edit Details`) are hidden.
    - Editors & Admins: *"View Full Profile"*, *"Make Focal Person"*, plus `+ Add Relative` and `Edit Details`.
- **Mobile (`< 768px`) & Tablet:** Converts into a swipeable **Quick-View Bottom Sheet** (`BottomSheet`) per [18 §3.7](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/18-progressive-web-app-mobile.md#37-mobile-quick-view-bottom-sheet-bottomsheet):
  - **Peek state (25% height):** Displays avatar, name, birth/death span, kinship badge, and *"View Profile"* CTA, keeping 75% of the tree canvas interactive in the background.
  - **Expanded state (75% height):** Shows full facts, immediate relatives, and quick actions (filtered by role).
  - **Swipe-to-dismiss:** Dragging down past threshold smoothly closes the sheet.

### 3.6 Mobile & Tablet Touch Gesture Ergonomics
- **Pinch-to-Zoom:** Natural two-finger pinch gesture scaling between `0.25x` and `3.0x` with logarithmic damping. Zoom focal point anchors dynamically to the geometric midpoint between the two touches.
- **Canvas Panning:** Single-finger drag on canvas background or two-finger drag anywhere translates the SVG viewport with inertia dampening.
- **Tap vs. Hover:** Hover states and tooltips are omitted on touch screens (`@media (pointer: coarse)`). A short tap (< 250ms, < 10px drift) selects the card and opens the bottom sheet.
- **Long-Press Menu:** Holding a card for > 500ms triggers a haptic pulse (`navigator.vibrate(15)` where supported) and opens a quick context menu:
  - Viewers & Public: *"Make Focal"*, *"View Profile"*.
  - Editors & Admins: *"Make Focal"*, *"View Profile"*, *"Add Parent"*, *"Add Child"*, *"Edit Person"*.
- **Control Compression:** On screens `< 768px`, floating zoom and view switchers condense into a unified top-right pill; the radar mini-map is collapsed by default into an icon button.

---

## 4. Definition of Done (DoD)

- [ ] **Full Viewport Canvas:** Canvas stretches to full screen height with no scrollbars on the parent container.
- [ ] **Default Private Redirection:** In default private posture (`ENABLE_PUBLIC_ACCESS=false`), unauthenticated access to `/tree` redirects directly to `/login`.
- [ ] **Public Mode Living Masking:** When accessed unauthenticated under `ENABLE_PUBLIC_ACCESS=true`, cards of living persons display `Living` placeholders and neutral silhouettes; facts and photos are completely stripped.
- [ ] **Viewer Living Visibility & Read-Only:** Authenticated Viewers see complete personal records (full names, patronymics, dates, places, media) for living relatives, while write handles and edit buttons remain hidden.
- [ ] **Editor & Admin Node Authoring:** Node hover handles (`+`), quick-add controls, and drawer edit buttons are accessible to Editors and Admins.
- [ ] **Smooth Mode Switching:** Changing visualization mode transitions smoothly without page reload or lost selection.
- [ ] **Multi-Touch Gestures:** Native pinch-to-zoom and two-finger panning operate at 60fps on iOS and Android touch screens without viewport-zoom glitching.
- [ ] **Mobile Quick-View Bottom Sheet:** On mobile screens `< 768px`, tapping a person card opens a swipeable bottom sheet with 25% peek and 75% expanded states rather than an intrusive desktop drawer.
- [ ] **Touch Target Standards:** Floating canvas controls, zoom buttons, and interactive node elements have a minimum hit area of 44×44px.
- [ ] **Responsive Overlays:** On mobile devices, floating toolbars condense to clear safe-area margins and prevent obstructing the canvas.
