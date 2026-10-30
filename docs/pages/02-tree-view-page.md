# Page Specification: Interactive Family Tree Canvas (`/tree`)

## 1. Overview & Objective
The Tree Canvas is the interactive visual heart of the application. It occupies **100% of the viewport height and width below the topbar**, providing a clutter-free, hardware-accelerated SVG infinite canvas with floating control overlays.

> [!IMPORTANT] Tier-Based Access & Privacy Behavior:
> - **Default Private Instance (`ENABLE_PUBLIC_ACCESS=false`):** Unauthenticated visitors cannot access `/tree` and are immediately redirected to `/login`.
> - **Public Tree Mode (`ENABLE_PUBLIC_ACCESS=true`, Unauthenticated):** Public visitors see the tree canvas in **Strictly Masked Read-Only Mode**:
>   - Living persons display as a single initial character with dot (e.g. `И.` / `J.`) with silhouette avatars, and zero dates, places, notes, or photos. Surnames, patronymics, and full given names are never exposed.
>   - **Zero Edit Controls:** Absolutely NO authoring or editing controls are rendered on the canvas:
>     - No `+ Add Person` or Quick-Add trigger in the floating toolbar.
>     - No `+ Add parent / child / spouse` placeholder ghost slots, node add handles, or hover plus indicators.
>     - No `Edit`, `Delete`, `+ Add Relative`, `Add Event`, or `Set home person` buttons on node cards, Quick Drawer, or Bottom Sheet.
>     - Context menus (long-press) provide only *"Make Focal"* and *"View Profile"* (for deceased).
>     - Header omnibox search and `+ Add` menu are completely omitted.
> - **Authenticated Viewers (`Viewer`):** Trusted family members see the complete tree with all living relatives' data (names, patronymics, dates, places, photos) fully visible. Canvas is strictly read-only (node add handles and edit triggers are hidden).
> - **Authenticated Editors & Admins (`Editor`, `Admin`):** Full living person visibility plus interactive node edit handles, quick-add relative triggers (`+`), and contextual menus.

---

## 2. Page Wireframe & Component Layout

```
+----------------------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]                      |
+----------------------------------------------------------------------------------------------------+
| [Floating View Switcher]                                    [Floating Mode Option, Depth & Fit]    |
| [Pedigree | Hourglass | Descendants | Fan | All Nodes | ⇄]  [All Nodes ▼] [184 individuals] [⛶]    |
|                                                                                                    |
|                                [ SVG INFINITE PAN / ZOOM CANVAS ]                                  |
|                                                                                                    |
| [PRIMARY FAMILY COMPONENT]                           [SECONDARY IN-LAW BRANCH]                     |
| [Gen -2: Grandparents Tier]                          [Gen -2: Grandparents Tier]                   |
|                 +---------------+       +---------------+            +---------------+             |
|                 | Paternal G-Fa | ===== | Paternal G-Mo |            | In-Law G-Fa   |             |
|                 +-------+-------+       +-------+-------+            +-------+-------+             |
|                         |                       |                            |                     |
|         +---------------+-----------------------+---------------+            |                     |
|         |                                                       |            |                     |
| [Gen -1: Parents & Collaterals]                                 |            |                     |
| +---------------+   +===============+       +===============+   |    +-------+-------+             |
| | Uncle & Wife  |   # Father [Ring] # ===== # Mother [Ring] #   |    | In-Law Uncle  |             |
| +-------+-------+   +=======+=======+   *   +=======+=======+   |    +---------------+             |
|         |                   |           |           |           |                                  |
|         |                   |   [FLOATING CONNECTION HUD]       |    [ISOLATED RECORDS SHELF GRID] |
|         |                   |   [♥ Marriage • 1934 Moscow    ]  |    +---------------+             |
|         |                   |   [(Avatar) Father ➔ [Jump]]      |    | Unlinked 1    |             |
|         |                   |   [(Avatar) Mother ➔ [Jump]]      |    +---------------+             |
|         |                   |   [🔗 Kinship] [Profile] [✕]      |    +---------------+             |
|         |           +-------+-----------------------+-------+   |    | Unlinked 2    |             |
|         |           |                                       |   |    +---------------+             |
| [Gen 0: Focal Baseline]                                     |   |                                  |
| +---------------+   +---------------+       +---------------+   |   +---------------+              |
| | First Cousin  |   | Brother       |       | Spouse 1      |===+===| Focal Person  |===== [Spouse 2]
| +---------------+   +---------------+       +-------+-------+   |   +-------+-------+      |       |
|                                                     |   (Union 1)           |    (Union 2) +---+---+
|                                                     +-------*-------+       +-------*----------+
|                                                             |                       |
| [Gen +1: Children & Nieces]                           [Union 1 Bus]           [Union 2 Bus]
|                                                       +-----+-----+           +-----+-----+
|                                                       |           |           |           |
|                                                  [Child 1]     [Child 2] [Child 3]     [Child 4]
|                                                                                                    |
| [Floating Zoom Controls]                                                       [Collapsible Radar] |
| [ + | 100% | - | ⌂ ]                                                          [     Mini-Map    ] |
+----------------------------------------------------------------------------------------------------+
| SLIDE-OVER QUICK DRAWER (Opens when a person card is clicked)                                      |
| [X] Aleksandr Ivanovich Petrov • 1912 - 1989                                                       |
| [Avatar] [Key Milestones] [Spouses & Children] [View Full Profile ->]                             |
+----------------------------------------------------------------------------------------------------+
```

---

## 3. Structural Components

### 3.1 Top Floating View Mode Switcher
Positioned top-left (`absolute top-4 left-6 z-20`):
- Segmented control pill (`bg-surface/90 backdrop-blur-md border border-border shadow-sm rounded-lg p-1`):
  - **Pedigree** (`GitBranch` — horizontal direct ancestors)
  - **Hourglass** (`GitFork` — ancestors above, descendants below, with collateral expansion option)
  - **Descendants** (`Workflow` — generational cascade)
  - **Fan Chart** (`Disc` — concentric circular chart)
  - **All Nodes** (`Network` — unpruned complete archive forest)
  - **Kinship Finder** (`Route` — relationship path between two persons)

### 3.2 Top Floating Generation, Framing & Projection Controls
Positioned top-right (`absolute top-4 right-6 z-20`):
- **Hourglass Projection Mode Toggle:** Visible exclusively when `mode === 'hourglass'`:
  - Quick dropdown or segmented toggle:
    - **Wide (`collateral=true`, default):** Encompasses collateral branches (grandparents' children + families, parents' children + families).
    - **Compact (`collateral=false`):** Confines display strictly to direct lineal ascendants and descendants.
  - Updates URL parameter without triggering full page reload.
- **Dynamic Controls in All Nodes Mode (`mode === 'all'`):**
  - **Generation Depth Dropdown:** Hidden/suppressed; replaced by an informational badge showing total individual count: `<Badge variant="outline">{t('tree.toolbar.allNodesCount', { count: totalVisible })}</Badge>`.
  - **Wide / Compact Toggle:** Hidden/suppressed.
  - **Fit View to Screen Button (`⛶`):** Computes collective bounding box of all connected components and isolated shelf cards, smoothly scaling to frame the entire forest archive.
  - **Center on Focal Person Button (`⌂`):** Recenter camera on the focal individual in the primary component at optimal zoom.
- **Standard Generation Depth Dropdown (Hourglass / Pedigree / Descendants):** Quick selector for `2`, `3`, `4`, `5`, `6`, or `All` generations.
- **Center on Focal Person Button (`⌂`):** Recenter camera on the focal individual at $(0, 0)$ and reset zoom.
- **Fit View to Screen Button (`⛶`):** Computes bounding box of all visible nodes (including wide collateral branches) and scales smoothly.

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
    - Public Mode (Living): Shows silhouette and single initial character with dot (e.g. `И.` / `J.`) without surname or patronymic.
    - Authenticated / Deceased: Photo avatar, full name with patronymic, sex badge, life dates.
  - **Core Facts:**
    - Public Mode (Living): Suppressed completely (empty state: *"Private details hidden"*).
    - Authenticated / Deceased: Birth, baptism, marriage, death dates and geocoded places.
  - **Immediate Family Links:** Grouped by family union: lists each spouse with their respective children clustered under that marriage, plus any single-parent children in a distinct group, enabling 1-click jumps directly to relatives.
  - **Media Preview:**
    - Public Mode: 3 recent tagged photos of deceased individuals only; photos of living persons return silhouette placeholders.
    - Authenticated: 3 recent tagged photos.
  - **Actions:**
    - Public Mode: *"View Full Profile"* (for deceased ancestors; living profiles render masked placeholder) and *"Make Focal Person"*. All authoring/editing controls (`+ Add Relative`, `Edit Details`, `Set as my home person`, `Upload avatar`, `Delete`) are strictly hidden.
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

### 3.7 Wide Hourglass Generational Layout & Bus Structure
- **Strict Generational Alignment:** All cards on a given generational distance align to identical horizontal baselines ($Y = \text{tier} \times \text{ROW\_STEP}$):
  - Gen -2: Grandparents.
  - Gen -1: Parents, Aunts, Uncles + spouses.
  - Gen 0: Focal Person + spouse, Siblings + spouses, First Cousins + spouses.
  - Gen +1: Children of focal person + spouses, Nieces & Nephews + spouses, First Cousins' children.
- **Deterministic Spacing:** Recursive subtree width calculation ensures zero node overlap across complex collateral sibling groups.
- **Bus Routing:** Orthogonal distribution bus runners connect parent unions to multiple children, avoiding angled spaghetti cross-lines.

### 3.8 All Nodes Canvas Layout & Multi-Component Forest Rendering
- **Forest Partitioning & Spatial Placement:**
  - **Primary Component ($C_{\text{primary}}$):** Centered at $X \approx 0$ with the focal/home person anchored at Generation 0 ($Y = 0$).
  - **Secondary Components ($C_2 \dots C_k$):** Independent family lines (in-laws, adopted lines) are placed side-by-side to the right of preceding components with a clear $\ge 300\text{px}$ horizontal separation gap.
  - **Isolated Records Shelf Grid:** Individuals with no family relationships (singletons) and isolated couple fragments are aggregated into a clean shelf grid positioned alongside or beneath secondary components, ensuring 100% of database records remain accessible without scattering across the main lineage canvas.
- **Component Header Badges:** Subtle pill badges (e.g. `<Badge variant="outline">Main Family Branch</Badge>`, `<Badge variant="outline">Branch 2</Badge>`) mark the origin of each disconnected component.
- **Multi-Tier Level of Detail (LOD) Transitions:**
  - **LOD 0 ($\text{Zoom} \ge 0.5$):** Full card with avatar, complete name with patronymic, vital dates and places, interactive hover toolbar (`Profile`, `Focus`, `Add`, `Edit`).
  - **LOD 1 ($0.25 \le \text{Zoom} < 0.5$):** Simplified card with avatar, full name, and birth/death years only; action buttons suppressed to maximize pan/zoom throughput.
  - **LOD 2 ($\text{Zoom} < 0.25$):** Minimalist colored pill showing gender indicator and surname/initials, providing a clean bird's-eye overview of the full forest.
- **Cross-Component Navigation & Camera Lerp:**
  - Searching for any relative via Omnibox (`Cmd+K`) or clicking a relative in the Quick Drawer smoothly animates the camera (`camera.lerp`) across components to center on the target node.
  - "Fit to Screen" (`⛶`) computes the combined bounding box of all components and isolated cards, smoothly framing the entire archive forest.
- **Full Forest Radar Mini-Map:**
  - The bottom-right radar mini-map scales to the combined forest dimensions, rendering all components as miniature node rectangles with the active camera viewport highlight.

### 3.9 Interactive Connection Lines, Anti-Collinear Clearance & Connection Navigation HUD (`ConnectionNavPopover`)
- **Visual Differentiation of Connection Lines:**
  - **Marriages / Unions (`kind: 'spouse'`):** Warm ochre/amber (`--color-brand-accent`: Light `#C68A34`, Dark `#D9A04B`, $2.5\text{px}$ solid stroke) with a small circular anchor junction dot ($r = 3\text{px}$) at the union midpoint, clearly separating partnerships from bloodline descent.
  - **Direct Lineage Parent-Child (`kind: 'parent_child' && is_direct_lineage`):** Confident primary brand line (`--color-brand-primary`: Light `#2D5584`, Dark `#4F7EB5`, $2.5\text{px}$ solid stroke, elevated z-index) tracing direct ancestry and descent through the focal person.
  - **Collateral Parent-Child (`kind: 'parent_child' && !is_direct_lineage`):** Neutral slate line (`--color-lineage-neutral`: Light `#64748B`, Dark `#8896AB`, $1.5\text{px}$ solid stroke).
  - **Active / Selected Connection:** High-visibility glowing accent line ($3.5\text{px}$ stroke with luminous drop-shadow filter) connecting two actively highlighted person cards.
- **Visual Contrast & Sharpness (WCAG 2.2 AA):**
  - Text on cards maintains $\ge 4.5:1$ contrast against card backgrounds in both themes; line strokes and card borders maintain $\ge 3:1$ contrast against the canvas background (`#F8F9FA` light / `#0E1116` dark) with zero washed-out low-opacity filters.
  - **Pixel Snapping:** Canvas pan translation snaps to integer pixels upon gesture completion (`Math.round(pan.x)`, `Math.round(pan.y)`), preventing fractional subpixel rasterization blur on typography, borders, and vector paths. All SVG orthogonal lines apply `shapeRendering="geometricPrecision"`.
- **Anti-Collinear Routing & Multi-Lane Bus Clearance:**
  - Distribution bus horizontal runners from adjacent unions sharing the same vertical gap are assigned staggered lanes with $\ge 12\text{px}$ vertical clearance (e.g. $Y + 144\text{px}$ vs $Y + 156\text{px}$) so lines never merge into single ambiguous cuts.
  - Collinear vertical drops from separate branches are offset by $\ge 12\text{px}$ horizontally.
- **Dual-Layer SVG Hit Target:**
  - Every connection line renders an underlying invisible hit path: `fill="none"`, `stroke="transparent"`, `strokeWidth={14}` on desktop ($\ge 20\text{px}$ on mobile/touch screens), with `pointerEvents="stroke"` and pointer cursor on hover.
  - Hovering brightens the visible path by $+1\text{px}$ stroke width.
- **Connection Selection & Endpoint Synchronization:**
  - Clicking the hit path sets `selectedEdgeId = edge.id`.
  - The path illuminates with the active glowing accent.
  - The two endpoint cards receive active focus rings (`ring-2 ring-brand-primary` / Level 2 elevation shadow), creating an unambiguous visual trace between the connected relatives.
- **Floating Connection Navigation HUD / Popover (`ConnectionNavPopover`):**
  - **Desktop (`> 768px`):** Positioned near the clicked edge midpoint as a frosted floating popover (`bg-surface/95 backdrop-blur-md border border-border rounded-xl shadow-elevation-2 p-3 z-30`).
  - **Mobile (`< 768px`):** Docks to the top-center safe area or integrates as a swipeable bottom sheet peek.
  - **Content Elements:**
    - Relationship header badge ("Marriage" / "Parent → Child", displaying marriage date/place if recorded).
    - Endpoint Relative A & Endpoint Relative B cards:
      - Photo avatar / gender silhouette (with living-privacy masking in public guest mode).
      - Full name with patronymic (masked to single initial with dot in public guest mode).
      - Kinship / Role label ("Father", "Mother", "Spouse", "Son", "Daughter").
      - One-click "Jump to [Person]" button (`Navigation` icon): Triggers a smooth camera glide (`camera.lerp`) centering the viewport directly on that relative's card and pulsing the card ring.
    - "Find Kinship Path" CTA (`Route` icon): Opens the Kinship Finder pre-populated with Person A and Person B.
    - Close button (`✕`) / `Esc` key dismissal.

### 3.10 Multiple Spouses Placement & Family-Partitioned Bus Visualization
Per [04 §2.7](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#27-multiple-spouses-successive-marriages--family-partitioned-children-buses), the interactive canvas natively visualizes individuals with multiple marriages across Hourglass, Descendants, and All Nodes modes:
- **Symmetrical 2-Spouse Flanking:**
  - When an individual has 2 spouses, they are positioned symmetrically on the same generational baseline:
    `[ Spouse 1 (Earlier) ] ======= [ Individual ] ======= [ Spouse 2 (Later) ]`
  - Ordered chronologically by marriage date / earliest child birth date.
  - Each union displays an independent ochre marriage bar (`--color-brand-accent`, $2.5\text{px}$) with an anchor junction dot ($r = 3\text{px}$) at vertical center.
- **Family-Partitioned Children Buses:**
  - Descendant children are partitioned strictly by `family_id`:
    - Union 1 children hang under Union 1 bus (rooted at Anchor 1, left).
    - Union 2 children hang under Union 2 bus (rooted at Anchor 2, right).
    - Children with unrecorded spouses hang from a single-parent drop stem centered beneath the individual's card.
  - Full siblings share a common bus runner; half-siblings visually originate from separate union anchors.
- **Anti-Collinear Multi-Lane Staggering:**
  - The two union anchors are separated by $\ge 264\text{px}$ horizontally.
  - Horizontal bus lines in shared vertical gaps are assigned staggered lanes ($\ge 12\text{px}$ clearance), ensuring zero line overlaps.
- **Independent Connection Inspection:**
  - Clicking either marriage connector opens `ConnectionNavPopover` displaying the specific union's participants and wedding facts.

### 3.11 Kinship Path Finder Multi-Trace Visualization & Legend HUD
Per [04 §2.5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#25-kinship--relationship-path-calculator-multi-trace-support), when two individuals are related through multiple marriages, successive unions, or collateral ties:
- **`KinshipFinderDialog` Multi-Trace Switcher:**
  - When multiple genealogical paths exist, the modal displays segmented trace tabs:
    - `[Trace 1: Paternal Half-Brother]`
    - `[Trace 2: Maternal Step-Brother]`
  - Each tab indicates the relationship degree, biological vs. affinal tag, and common ancestors.
  - Action buttons: "Highlight This Trace" (active trace only) and "Highlight All Traces" (highlights all discovered paths simultaneously with distinct palette colors).
- **Multi-Trace Visual Color Coding:**
  - **Trace 1 (Primary):** Brand Primary Blue (`--color-brand-primary`: Light `#2D5584`, Dark `#4F7EB5`, $3.5\text{px}$ solid glow).
  - **Trace 2 (Alternate):** Archival Warm Ochre (`--color-fan-3`: Light `#C68A34`, Dark `#D9A04B`, $3.0\text{px}$ stroke, dashed `strokeDasharray="6 3"` or amber glow).
  - **Trace 3 (Alternate):** Regal Violet (`--color-fan-4`: Light `#7C3AED`, Dark `#9333EA`, $3.0\text{px}$ stroke, dashed `strokeDasharray="4 4"`).
  - Person cards on the highlighted traces receive matching accent border rings.
- **Floating Multi-Trace Legend HUD:**
  - Positioned top-center of the canvas (`absolute top-4 left-1/2 -translate-x-1/2 z-30`):
    - Frosted pill container (`bg-surface/90 backdrop-blur-md border border-border shadow-md rounded-full px-3 py-1 flex items-center gap-2`).
    - Color indicator pills for each active trace with 1-click camera lerp.
    - 1-Click "Clear Highlights" (`✕`) button to dismiss paths and restore normal tree styling.

---

## 4. Definition of Done (DoD)

- [ ] **Full Viewport Canvas:** Canvas stretches to full screen height with no scrollbars on the parent container.
- [ ] **Default Private Redirection:** In default private posture (`ENABLE_PUBLIC_ACCESS=false`), unauthenticated access to `/tree` redirects directly to `/login`.
- [ ] **Public Mode Living Masking:** When accessed unauthenticated under `ENABLE_PUBLIC_ACCESS=true`, cards of living persons display single initial character placeholders (e.g. `И.` / `J.`) and neutral silhouettes; surnames, patronymics, facts, and photos are completely stripped across both direct and collateral nodes.
- [ ] **Zero Edit Controls in Public Canvas:** When accessed unauthenticated under `ENABLE_PUBLIC_ACCESS=true`, no toolbar Add buttons, canvas ghost slots, node creation handles (+), drawer/sheet edit buttons, or destructive controls are rendered anywhere.
- [ ] **Viewer Living Visibility & Read-Only:** Authenticated Viewers see complete personal records (full names, patronymics, dates, places, media) for living relatives, while write handles and edit buttons remain hidden.
- [ ] **Editor & Admin Node Authoring:** Node hover handles (`+`), quick-add controls, and drawer edit buttons are accessible to Editors and Admins.
- [ ] **Smooth Mode & Projection Switching:** Changing visualization mode or toggling between Wide and Compact Hourglass transitions smoothly without page reload or lost selection.
- [ ] **Wide Hourglass Layout Accuracy:** In Wide Hourglass mode, grandparents' other children (aunts/uncles), their spouses, first cousins, siblings, and nieces/nephews are displayed on strict generational baselines with zero card collision.
- [ ] **Multiple Spouses Visualization (Hourglass & Descendants):** In Hourglass, Descendants, and All Nodes modes, an individual with multiple spouses displays each spouse on the same generational baseline with an explicit warm ochre marriage connector bar (`--color-brand-accent`, $2.5\text{px}$) and union anchor dot.
- [ ] **Symmetrical 2-Spouse Placement:** An individual with 2 spouses is positioned symmetrically in the center (`[Spouse 1] === [Person] === [Spouse 2]`), ordered chronologically by marriage date / earliest child birth date, with zero line-card crossing.
- [ ] **Family-Partitioned Children Buses:** Children are partitioned strictly by family union (`family_id`); children of Spouse 1 hang under Union 1 bus, and children of Spouse 2 hang under Union 2 bus, with zero lumping of half-siblings under a shared bus.
- [ ] **Single-Parent Children Bus Fallback:** Children with no recorded second parent hang directly from the bottom-center of the parent's card via a dedicated single-parent distribution bus.
- [ ] **Multi-Spouse Recursive Width Allocation:** Symmetrical wing width calculation ($W_{\text{left}} + W_{\text{card}} + W_{\text{right}}$) guarantees zero node collision between children clusters of different marriages.
- [ ] **Multi-Trace Kinship Visualization:** When multiple genealogical paths exist between two individuals, `KinshipFinderDialog` allows switching between traces and provides a "Highlight All Paths" action; the canvas displays multiple traces simultaneously with distinct palette colors (Blue, Ochre, Violet) and matching card focus rings.
- [ ] **Floating Multi-Trace Legend HUD:** Canvas displays a floating top-center legend pill showing active kinship traces with quick-jump pills and a 1-click "Clear Highlights" (`✕`) button.
- [ ] **Connection Line Relationship Differentiation:** Marriage lines render in warm ochre/amber (`--color-brand-accent`, $2.5\text{px}$), direct lineage parent-child lines in primary brand (`--color-brand-primary`, $2.5\text{px}$), and collateral lines in neutral slate (`--color-lineage-neutral`, $1.5\text{px}$) with zero washed-out low-contrast opacities.
- [ ] **WCAG 2.2 AA Contrast Compliance:** Text on person cards meets $\ge 4.5:1$ contrast against surface backgrounds; connector lines, card borders, and badges meet $\ge 3:1$ contrast against canvas backgrounds in both light and dark modes.
- [ ] **Crisp Vector & Transform Rendering:** Canvas pan translation snaps to integer pixels upon gesture completion, preventing subpixel text and border blurriness; connector SVG paths utilize `shapeRendering="geometricPrecision"`.
- [ ] **Interactive Connection Hit Target:** Every connector line provides an underlying transparent interactive hit path ($\ge 14\text{px}$ desktop, $\ge 20\text{px}$ touch) with `pointer-events: stroke` and pointer cursor on hover.
- [ ] **Edge Selection & Endpoint Highlight:** Clicking a connector selects the edge, highlights the path with a glowing accent stroke, and adds active focus rings (`ring-2 ring-brand-primary`) to both connected endpoint cards.
- [ ] **Connection Navigation HUD:** Selecting an edge opens a floating popover / HUD (`ConnectionNavPopover`) displaying relationship type, avatar badges for both endpoints, 1-click "Jump to [Person]" button, and "Find Kinship Path" trigger.
- [ ] **Smooth Camera Glide on Jump:** Clicking "Jump to [Person]" in the Connection HUD smoothly glides the camera (`camera.lerp`) to center the target person's card in the viewport.
- [ ] **Anti-Collinear Bus Routing & Clearance:** Horizontal and vertical distribution bus segments from adjacent unions maintain $\ge 12\text{px}$ clearance, preventing merged or overlapping collinear lines.
- [ ] **Public Mode Living Masking in Connection HUD:** When unauthenticated public mode is enabled, living individuals displayed in the Connection HUD have masked single-initial names (`И.` / `J.`), silhouette avatars, and zero vital dates.
- [ ] **All Nodes Mode Selector:** Toolbar SegmentedControl includes "All Nodes" option with `Network` Lucide icon, switching to `mode="all"` without page reload.
- [ ] **Dynamic Toolbar Controls in All Nodes:** In `mode="all"`, the generation depth dropdown and wide/compact toggle are hidden or replaced by an informational individuals count badge.
- [ ] **Multi-Component Forest Wireframe Conformance:** Disconnected family branches render side-by-side with $\ge 300\text{px}$ horizontal gap; isolated singletons render in an organized shelf grid with zero card collisions.
- [ ] **Full Forest Fit-to-Screen:** Clicking `⛶` in All Nodes mode calculates bounding box across all components and frames the entire canvas within the viewport.
- [ ] **Omnibox Cross-Component Jump:** Searching a relative in a secondary component or isolated shelf smoothly pans and zooms the camera to focus on their card.
- [ ] **Multi-Tier LOD Transitions:** Zooming out below $0.5x$ transitions cards to simplified rendering (avatar + name + life years), and below $0.25x$ to micro-pills, sustaining 60fps pan/zoom.
- [ ] **Radar Mini-Map Forest Coverage:** Mini-map renders all components and allows 1-click camera jumps across the complete forest.
- [ ] **Multi-Touch Gestures:** Native pinch-to-zoom and two-finger panning operate at 60fps on iOS and Android touch screens without viewport-zoom glitching.
- [ ] **Mobile Quick-View Bottom Sheet:** On mobile screens `< 768px`, tapping a person card opens a swipeable bottom sheet with 25% peek and 75% expanded states rather than an intrusive desktop drawer.
- [ ] **Touch Target Standards:** Floating canvas controls, zoom buttons, and interactive node elements have a minimum hit area of 44×44px.
- [ ] **Responsive Overlays:** On mobile devices, floating toolbars condense to clear safe-area margins and prevent obstructing the canvas.
