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
- **Overview:** The focal individual and their primary spouse(s) are placed at the vertical center (Generation 0). Ancestral generations extend upwards ($Y < 0$), while descendant generations cascade downwards ($Y > 0$).
- **Use Case:** The premier family gathering overview, giving users an immediate visual context of their generation, their parents' generation, their grandparents, and their children.
- **Dual Projection Modes:**
  The Hourglass projection supports two operating modes controlled by the `collateral` query parameter and toolbar toggle:
  1. **Compact Hourglass (`collateral=false`):**
     - Confines the upward tree strictly to direct lineal ascendants (parents, grandparents, great-grandparents) and downward tree strictly to direct lineal descendants (children, grandchildren).
     - Minimal horizontal width; best for quick direct ancestry checks or narrow mobile viewports.
  2. **Wide / Collateral-Expanded Hourglass (`collateral=true`, Default):**
     - Expands the ancestral tree to encompass **collateral branches** originating from direct ancestors: grandparents' children (aunts and uncles) and their families (spouses, first cousins); parents' children (siblings) and their families (spouses, nieces, nephews); and great-grandparents' collateral lines (great-aunts/uncles, second cousins).
     - Retains clean horizontal generational tiers so all relatives of the same generational distance align on identical horizontal baselines.

#### 2.2.1 Scope of Included Relatives in Wide Hourglass
When `mode=hourglass` and `collateral=true` (up to `generations = n`, default 4):
- **Generation 0 (Focal Center Baseline, $Y = 0$):**
  - **Focal Individual ($F$):** Anchored at canvas center.
  - **Spouses of $F$:** All recorded spouses $S_1, S_2, \dots, S_k$ positioned on Gen 0 flanking $F$ (e.g. Spouse 1 on left, $F$ in middle, Spouse 2 on right for 2 spouses per [§2.7](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#27-multiple-spouses-successive-marriages--family-partitioned-children-buses)), each connected via an explicit warm ochre marriage connector bar (`--color-brand-accent`).
  - **Full & Half Siblings of $F$:** Children of $F$'s parents (excluding $F$), flanking the focal group. Half-siblings from successive marriages of either parent hang under their respective parental union bus.
  - **Spouses of Siblings:** Placed beside their respective partner (including multiple spouses per [§2.7](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#27-multiple-spouses-successive-marriages--family-partitioned-children-buses)).
  - **First Cousins of $F$:** Children of $F$'s aunts and uncles (from Gen -1), placed on the Gen 0 baseline flanking the sibling clusters.
  - **Spouses of First Cousins:** Placed beside their respective cousin partner.
  - **Second Cousins of $F$:** (When $n \ge 4$) Grandchildren of great-aunts/uncles, positioned at Gen 0.
- **Generation -1 (Parents & Aunts/Uncles Tier, $Y = -1 \times \text{ROW\_STEP}$):**
  - **Parents and Stepparents of $F$:** Biological parents leading directly to $F$; if either parent has successive marriages (e.g. father's second wife, mother's second husband), additional spouses are placed on Gen -1 with explicit marriage bars and separate union anchors.
  - **Aunts and Uncles of $F$:** Non-parent children of $F$'s grandparents.
  - **Spouses of Aunts and Uncles:** In-law partners of aunts/uncles (including successive spouses).
  - **First Cousins Once Removed (Older):** (When $n \ge 4$) Children of great-aunts/uncles.
- **Generation -2 (Grandparents Tier, $Y = -2 \times \text{ROW\_STEP}$):**
  - **Grandparents of $F$:** Paternal and maternal grandparent couples (and any remarriage partners in wide mode).
  - **Great-Aunts and Great-Uncles:** (When $n \ge 4$) Non-grandparent children of great-grandparents + their spouses.
- **Generation -3 (Great-Grandparents Tier, $Y = -3 \times \text{ROW\_STEP}$):**
  - **Great-Grandparents of $F$:** Up to 4 couples (8 direct ancestors) at the ancestor limit boundary.
- **Generation +1 (Children, Nieces & Nephews Tier, $Y = +1 \times \text{ROW\_STEP}$):**
  - **Direct Children of $F$:** Children of $F$ and partner(s) + their spouses, partitioned strictly under the specific union bus of the marriage they belong to (Union 1 children under Union 1 bus, Union 2 children under Union 2 bus).
  - **Nieces and Nephews of $F$:** Children of $F$'s siblings + their spouses, partitioned by sibling union.
  - **First Cousins Once Removed (Younger):** Children of $F$'s first cousins + their spouses.
- **Generation +2 (Grandchildren & Grand-Nieces/Nephews Tier, $Y = +2 \times \text{ROW\_STEP}$):**
  - **Direct Grandchildren of $F$:** Children of $F$'s children + their spouses, partitioned by child union.
  - **Grand-Nieces and Grand-Nephews:** Children of $F$'s nieces/nephews.

#### 2.2.2 Generational Alignment & Baseline Mapping Table
All individuals are placed strictly according to their genealogical distance relative to the focal person ($F$):

| Tier Index | Relative Baseline ($Y$) | Primary Kinship Relative to Focal Person ($F$) | Included Relatives & Spouses |
| :--- | :--- | :--- | :--- |
| **Gen -3** | $-3 \times \text{ROW\_STEP}$ | Great-Grandparents | 4 Great-grandparent couples |
| **Gen -2** | $-2 \times \text{ROW\_STEP}$ | Grandparents | Paternal & maternal grandparents; Great-aunts & great-uncles + spouses |
| **Gen -1** | $-1 \times \text{ROW\_STEP}$ | Parents, Aunts & Uncles | Biological parents; Stepparents (successive marriages); Paternal & maternal aunts & uncles + spouses |
| **Gen 0** | $0$ (Canvas Center) | **Focal Person, Siblings & First Cousins** | **Focal person + all spouses**; Full/half siblings + spouses; Paternal & maternal first cousins + spouses |
| **Gen +1** | $+1 \times \text{ROW\_STEP}$ | Children, Nieces & Nephews | Children of focal person (partitioned by union bus) + spouses; Nieces & nephews + spouses; First cousins' kids + spouses |
| **Gen +2** | $+2 \times \text{ROW\_STEP}$ | Grandchildren & Grand-Nieces/Nephews | Grandchildren (partitioned by union bus) + spouses; Grand-nieces & grand-nephews |
| **Gen +3** | $+3 \times \text{ROW\_STEP}$ | Great-Grandchildren | Great-grandchildren + spouses |

#### 2.2.3 Layout Geometry, Subtree Clustering & Bus Routing
1. **Deterministic Bottom-Up Width Allocation:**
   - Every individual card has fixed dimensions: $W_{\text{card}} = 240\text{px}$, $H_{\text{card}} = 92\text{px}$.
   - A single person card has base width $W_{\text{unit}} = W_{\text{card}}$.
   - A single-couple unit has base width $W_{\text{couple}} = 2 \times W_{\text{card}} + \text{SPOUSE\_GAP}$ (where $\text{SPOUSE\_GAP} = 24\text{px}$).
   - A multi-spouse unit with $k$ spouses has base width:
     $$W_{\text{multi}} = (k + 1) \times W_{\text{card}} + k \times \text{SPOUSE\_GAP}$$
     (For $k = 2$, $W_{\text{multi}} = 3 \times 240\text{px} + 2 \times 24\text{px} = 768\text{px}$).
   - For each family union $U_j$ with $k_j$ children, the children bounding width is:
     $$W_{\text{children}}(U_j) = \sum_{i=1}^{k_j} W_{\text{subtree}}(C_{j, i}) + \max(0, k_j - 1) \times H_{\text{GAP}}$$
   - The total bounding width allocated to a family branch with multiple unions is computed per [§2.7.4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#274-recursive-subtree-width-allocation--collision-prevention), ensuring **zero node overlap** regardless of spouse count or branch cardinality.
2. **Horizontal Centering & Axis Balancing:**
   - The focal group (focal person + spouses) is positioned symmetrically around $X = 0$.
   - Paternal ancestral and collateral subtrees expand towards $X < 0$ (left).
   - Maternal ancestral and collateral subtrees expand towards $X > 0$ (right).
   - Sibling clusters flank the focal multi-spouse group directly on Gen 0.
3. **Orthogonal Distribution Bus Routing & Anti-Collinear Clearance:**
   - **Spouse Connectors:** Clean horizontal lines joining the inner edge of each partner to the individual at vertical center ($Y + H_{\text{card}} / 2$). Rendered in warm ochre/amber (`--color-brand-accent`) to immediately distinguish partnerships from parentage.
   - **Union Midpoint Anchors:** Each distinct marriage union between person and spouse has its own circular anchor junction dot ($r = 3\text{px}$, warm ochre fill) at $(X_{\text{union}, j}, Y_{\text{parent}} + H_{\text{card}} / 2)$, visually rooting the vertical child stem to that specific marriage bar.
   - **Orthogonal Family Buses & Clearance Lanes:**
     1. Vertical drop from each union anchor: $(X_{\text{union}, j}, Y_{\text{parent}} + H_{\text{card}} / 2) \to (X_{\text{union}, j}, Y_{\text{bus}, j})$.
     2. Horizontal runner spanning from the first child to the last child belonging strictly to that union along $Y_{\text{bus}, j}$.
     3. Vertical drop lines branching from $Y_{\text{bus}, j}$ into the top-center $(X_{C_{j, i}}, Y_{\text{child}})$ of each child card.
   - **Anti-Collinear Disambiguation & Staggered Bus Lanes:**
     - Horizontal bus runners from distinct unions or adjacent family branches sharing the same vertical gap are assigned staggered lanes with $\ge 12\text{px}$ clearance (e.g. Lane A at $Y + 144\text{px}$, Lane B at $Y + 156\text{px}$) so horizontal paths never merge into ambiguous single lines.
     - Collinear vertical drop lines from different branches or unions are offset horizontally by $\ge 12\text{px}$ to guarantee visual independence.
   - **Direct Lineage Highlighter:**
     - Edges along the direct ancestry line (grandparent $\to$ parent $\to$ focal person $\to$ child $\to$ grandchild) are visually distinguished with a prominent accent color (`var(--color-brand-primary)`), thicker stroke width ($2.5\text{px}$ vs $1.5\text{px}$), and higher z-index over collateral edges.

### 2.3 Descendants Tree View
- **Structure:** Top-down vertical tree showing all descendants originating from a selected matriarch or patriarch ancestor.
- **Multiple Spouses & Successive Marriages:** Fully supports multiple marriages at the root patriarch/matriarch and recursively across all descendant generations per [§2.7](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md#27-multiple-spouses-successive-marriages--family-partitioned-children-buses):
  - Spouses are placed on the same generational baseline flanking the descendant node.
  - Children from each marriage hang strictly from their specific union bus.
- **Interactions:** Color-coded generational bands, collapsible sibling branches to focus on specific family lines.

### 2.4 Fan Chart (Radial Sunburst)
- **Structure:** Semi-circular or 360-degree circular concentric rings radiating from the focal individual (supports 4 to 8 generations).
- **Aesthetics:** Distinct color palettes for Paternal Grandfather (Blue), Paternal Grandmother (Green), Maternal Grandfather (Yellow/Amber), and Maternal Grandmother (Red/Coral).
- **Density:** Automatically scales font size and text truncation based on ring radius.

### 2.5 Kinship & Relationship Path Calculator (Multi-Trace Support)

#### 2.5.1 Multi-Trace Discovery & Complex Kinship Resolution
In families with multiple marriages, successive spouses, remarriages, or consanguineous ties (inter-marriage between related branches), two individuals are frequently related through **more than one distinct genealogical path**:
- **Bloodline vs. Affinal/Step Traces:**
  - *Example 1 (Half-Siblings & Remarriage):* Two individuals share a common biological father ($Fa$), while one is also connected to the other's mother ($Mo2$) via a step-relationship or subsequent marriage.
  - *Example 2 (Double First Cousins / Inter-Marriage):* Two brothers marry two sisters, or cousins marry across branches, creating two distinct biological kinship lineages through both paternal and maternal common ancestors.
  - *Example 3 (Multi-Marriage In-Laws):* Person A is connected to Person B through Marriage 1, and also through a separate family branch via Marriage 2.

To resolve these cases with genealogical fidelity, the Kinship engine identifies **all valid minimal genealogical traces** connecting Person A and Person B (up to $K = 5$ distinct traces), rather than arbitrarily picking a single shortest path and obscuring other ties:
1. **Trace Discovery Engine:**
   - Traverses the family graph using multi-source bidirectional traversal, searching for:
     - Biological Most Recent Common Ancestors (MRCA) via parent-child directed paths.
     - Affinal / marital connections via spouse links.
   - Computes distinct path sequences $T_1, T_2, \dots, T_m$ (where each trace consists of a series of person nodes and relationship steps).
2. **Trace Ranking & Categorization:**
   - **Trace 1 (Primary):** The shortest biological path through common biological ancestors; if no blood relationship exists, the shortest overall connection.
   - **Traces 2..K (Alternate Traces):** Ranked by step length and degree of kinship.
   - Each trace is tagged with its nature:
     - `is_blood_relation: true` (direct biological lineage through shared ancestors).
     - `is_blood_relation: false` (affinal/step relationship involving marriage unions).
   - Natural localized kinship labels for each trace (e.g. *"Half-brother (paternal)"*, *"Step-brother (via father's second marriage)"*, *"Second cousin (maternal line)"*).

#### 2.5.2 Multi-Trace UI & Inspection in Kinship Dialog (`KinshipFinderDialog`)
When multiple traces exist between the selected individuals:
1. **Trace Switcher Tabs / Dropdown:**
   - If more than 1 trace is found, the dialog renders a segmented trace selector or pill tabs:
     - `[Trace 1: Half-brother (via Father)]`
     - `[Trace 2: Step-brother (via Mother's Marriage)]`
   - Each tab displays:
     - Trace index (`1`, `2`, ...).
     - Relationship title with gender agreement.
     - Kinship degree badge (`Degree 2`, `Blood relation` / `Affinal`).
     - Common ancestor names pill.
2. **Step-by-Step Relationship Ladder:**
   - Selecting a trace displays its specific breadcrumb ladder: Person A $\to$ parent $\to$ common ancestor $\to$ spouse $\to$ Person B.
3. **Canvas Highlight Actions:**
   - **"Highlight Selected Trace" Button (`Route` icon):** Closes dialog and highlights strictly the active trace on the canvas.
   - **"Highlight All Traces" Button (`Network` icon):** Highlights all discovered traces simultaneously on the canvas, assigning distinct colors to each trace for side-by-side comparison.

#### 2.5.3 Multi-Trace Canvas Visualization (`TreeCanvas`)
When one or more kinship traces are highlighted on the tree canvas:
1. **Multi-Trace Visual Hierarchy & Color Coding:**
   - **Trace 1 (Primary / Active Selected):**
     - **Color:** Brand Accent Indigo/Blue (`--color-brand-primary`: Light `#2D5584`, Dark `#4F7EB5`, $3.5\text{px}$ solid stroke with glow filter `drop-shadow(0 0 4px rgba(45, 85, 132, 0.45))`).
     - **Z-Index:** Topmost edge layer.
     - **Card Rings:** Person cards along Trace 1 receive active focus rings (`ring-2 ring-brand-primary`).
   - **Trace 2 (Alternate Trace):**
     - **Color:** Archival Warm Ochre / Amber (`--color-fan-3`: Light `#C68A34`, Dark `#D9A04B`, $3.0\text{px}$ stroke, dashed `strokeDasharray="6 3"` or solid with amber glow).
     - **Card Rings:** Cards unique to Trace 2 receive amber focus rings (`ring-2 ring-amber-500`).
   - **Trace 3 (Alternate Trace):**
     - **Color:** Regal Violet / Purple (`--color-fan-4`: Light `#7C3AED`, Dark `#9333EA`, $3.0\text{px}$ stroke, dashed `strokeDasharray="4 4"`).
   - **Shared Nodes & Edges:** If two traces share common intermediate ancestors or edges, the stroke renders in the primary trace's styling with a dual-stripe or elevated accent glow, and the node card displays a multi-trace badge.
2. **Floating Multi-Trace Legend Chip:**
   - Positioned top-center of the canvas (`absolute top-4 left-1/2 -translate-x-1/2 z-30`):
     - Displays compact pills for each visible trace with its color dot (e.g. `[● Trace 1: Paternal] [● Trace 2: Maternal]`).
     - Clicking a pill switches focus/lerp to that trace.
     - **"Clear Highlights" Button (`✕`):** Clears all highlighted kinship paths and restores standard tree styling with 1 click.

### 2.6 All Nodes (Complete Graph) View
- **Overview & Objective:** An unpruned, full-archive visualization canvas that renders all visible individuals and families in the database on a unified coordinate space without focal-tree pruning.
- **Use Case:** Family historians and curators exploring the complete scope of their database simultaneously—inspecting disconnected collateral branches, isolated family lines, in-law ancestry trees, and unlinked archival research records that are normally omitted in single-focal tree projections.
- **Projection Identifier:** `mode="all"`.

#### 2.6.1 Multi-Component Forest Partitioning & Shelf Assembly
A real-world family archive often contains multiple disconnected trees (a mathematical forest) rather than a single connected component (e.g., the primary family tree, in-laws' independent ancestral lines, step-families, or unlinked archival research singletons).
1. **Connected Component Partitioning:**
   - The traversal engine partitions visible persons and families into disjoint connected components $C_1, C_2, \dots, C_k$ using undirected graph connectivity (edges formed by partner unions and parent-child links).
   - **Primary Component ($C_{\text{primary}}$):** The component containing `focal_id` (or `resolved_home_person_id`). If neither is specified, the component containing the largest number of nodes is designated as primary.
   - **Secondary Components ($C_2 \dots C_k$):** Ordered deterministically: first by descending node count, then by earliest recorded birth year, then by lexicographical GID/ID.
2. **Horizontal Side-by-Side Placement:**
   - The primary component $C_{\text{primary}}$ is positioned symmetrically around $X \approx 0$ (or left-anchored at $X = 0$).
   - Secondary multi-person components are positioned to the right of the preceding component with a distinct visual separation gap:
     $$X_{\text{start}}(C_{i}) = X_{\text{max}}(C_{i-1}) + \text{COMPONENT\_GAP}$$
     where $\text{COMPONENT\_GAP} = 300\text{px}$.
3. **Isolated Records Shelf Grid:**
   - Singletons (individuals with no recorded parents, spouses, or children) and single-couple fragments without ancestors/descendants are aggregated into an organized **"Isolated Records" Shelf Grid**.
   - Positioned cleanly alongside or below secondary components with uniform card grid spacing ($W_{\text{card}} + 24\text{px}$, $H_{\text{card}} + 24\text{px}$), preventing scattered orphan cards from cluttering the main lineage canvas while guaranteeing that 100% of archive records remain visible, searchable, and interactive.

#### 2.6.2 Generational Tier Alignment & Chronological Leveling
1. **Primary Component Baselines:**
   - The focal or home person is anchored at Generation 0 ($Y = 0$).
   - Ascending generational tiers extend upwards ($Y < 0$, Gen -1, Gen -2, Gen -3...).
   - Descending generational tiers extend downwards ($Y > 0$, Gen +1, Gen +2, Gen +3...).
   - Spouses within each family union align on the identical horizontal baseline.
2. **Disconnected Secondary Component Chronological Leveling:**
   - For disconnected components lacking a direct link to the focal person, topological roots (individuals with in-degree 0 in child links) are identified.
   - **Chronological Alignment:** If birth years are recorded in both the secondary component and the primary tree, the component's root generation is anchored to match historical generational epochs:
     $$\Delta \text{Gen} = \text{round}\left(\frac{\text{median\_birth\_year}(C_j) - \text{birth\_year}(\text{focal})}{30}\right)$$
     This ensures that 19th-century ancestors in an in-law tree align on the same horizontal plane as 19th-century ancestors in the primary tree.
   - **Fallback Alignment:** If birth dates are unavailable, the secondary component's root individuals are placed at Generation 0, cascading downwards with child links.

#### 2.6.3 Subtree Clustering, Orthogonal Bus Routing & Lineage Highlighting
1. **Deterministic Bottom-Up Width Allocation:**
   - Every individual card has fixed dimensions: $W_{\text{card}} = 240\text{px}$, $H_{\text{card}} = 92\text{px}$.
   - Subtree widths are computed recursively bottom-up: $W_{\text{branch}} = \max(W_{\text{couple}}, W_{\text{children}})$, ensuring **zero card collision** across all components.
2. **Orthogonal Distribution Bus Routing:**
   - Clean horizontal distribution bus runners connect parent unions to children via vertical drop lines, maintaining visual harmony with the wide hourglass layout.
3. **Direct Lineage Highlighter:**
   - If a focal person is active, their direct ancestral and descendant lineage spine within the primary component is visually accentuated (`var(--color-primary)` $2.5\text{px}$ stroke), providing an immediate visual anchor while all collateral, in-law, and secondary components render in standard neutral styling.

#### 2.6.4 Level of Detail (LOD) & Canvas Performance (100–1,500 Nodes)
Single-family self-hosted archives typically contain between 100 and 1,500 individuals across 5 to 10 generations. To maintain a steady 60fps pan/zoom interaction across all devices:
1. **Multi-Tier Level of Detail (LOD):**
   - **LOD 0 (Full Detail, $\text{Zoom} \ge 0.5$):** Renders complete person cards (photo avatar, full name with patronymic, vital birth/death dates and places, interactive action handles on hover).
   - **LOD 1 (Simplified Card, $0.25 \le \text{Zoom} < 0.5$):** Renders lightweight cards (photo/initial avatar, name, birth/death years only); suppresses hover action bars and secondary text to drastically reduce DOM complexity.
   - **LOD 2 (Micro-Pill / Dot Mode, $\text{Zoom} < 0.25$):** Renders compact pills displaying gender color accent and initials/surname, providing a clean birds-eye overview of the entire archive forest.
2. **DOM Frustum Culling:**
   - Off-screen nodes located outside the visible camera viewport (with safety padding) skip detailed SVG/DOM layout passes during continuous panning and multi-touch zooming.

### 2.7 Multiple Spouses, Successive Marriages & Family-Partitioned Children Buses

#### 2.7.1 The Successive Marriages Challenge & Genealogical Authenticity
In real-world family archives, individuals frequently have multiple marriages across their lifetime due to widowhood, remarriage, or divorce. A naive single-couple tree layout engine (such as taking `rel.spouseIds.find(...)`) introduces severe historical and visual defects:
1. **Omission of Partners:** Second and subsequent spouses are completely dropped from the visualization or orphaned into floating disconnected fragments.
2. **Falsification of Biological Parentage:** Children belonging to separate unions are lumped together under a single partner's distribution bus. Half-siblings are visually rendered as full biological siblings sharing the same parents, misleading relatives and curators.

To preserve genealogical authenticity, the tree visualization engine across **Hourglass (`mode="hourglass"`)**, **Descendants (`mode="descendant"`)**, and **All Nodes (`mode="all"`)** treats every marriage as a distinct genealogical union ($U_j = (P, S_j)$) with its own explicit marriage line, anchor junction, and dedicated family-specific children distribution bus.

#### 2.7.2 Multi-Spouse Placement & Baseline Ordering
1. **Strict Generational Baseline Alignment:**
   - All spouses $S_1, S_2, \dots, S_k$ and the partner individual $P$ align on the exact same horizontal generational baseline ($Y = \text{tier} \times \text{ROW\_STEP}$, $H_{\text{card}} = 92\text{px}$).
2. **Deterministic Chronological Ordering:**
   - Spouses are ordered deterministically by:
     1. Recorded marriage event date (`marriage.date`) in chronological order (earliest first).
     2. If marriage dates are absent, by birth date of the earliest child belonging to that family union.
     3. If neither dates exist, by stable deterministic fallback: partner birth date, then partner GID / UUID lexicographical order.
3. **Spatial Placement for $k = 2$ Spouses (Symmetrical Flanking):**
   - Symmetrical flanking places the multi-married individual $P$ at the center, flanked by their partners:
     ```
     [ Spouse 1 (Earlier) ] ======= [ Person P ] ======= [ Spouse 2 (Later) ]
              |         (Union 1)        |         (Union 2)        |
              +------------*------------+------------*------------+
                           |                         |
                    [ Union 1 Bus ]           [ Union 2 Bus ]
     ```
   - **Spouse 1 ($S_1$):** Placed to the left of $P$.
   - **Person ($P$):** Placed in the center.
   - **Spouse 2 ($S_2$):** Placed to the right of $P$.
   - **Union 1 Marriage Line:** Connects the right edge of $S_1$ to the left edge of $P$ at vertical center ($Y + H_{\text{card}} / 2$). Rendered in warm ochre/amber (`--color-brand-accent`, $2.5\text{px}$).
   - **Union 2 Marriage Line:** Connects the right edge of $P$ to the left edge of $S_2$ at vertical center ($Y + H_{\text{card}} / 2$). Rendered in warm ochre/amber (`--color-brand-accent`, $2.5\text{px}$).
   - **Zero Card Crossings:** Because $P$ is immediately adjacent to both $S_1$ and $S_2$, both ochre marriage lines connect directly with zero line-card intersections.
4. **Spatial Placement for $k \ge 3$ Spouses:**
   > [!NOTE] Assumption:
   > For $k \ge 3$ spouses (accounting for $< 2\%$ of archival unions), spouses are arranged in a contiguous horizontal row on the same generational baseline: $S_1$ on the far left, $P$ adjacent to $S_1$ and $S_2$, and subsequent spouses $S_3 \dots S_k$ placed consecutively to the right. Union lines for $S_3 \dots S_k$ connect to $P$ via an upper arched connector runner passing with $\ge 12\text{px}$ clearance above card borders, ensuring zero card occlusion and deterministic layout stability.

#### 2.7.3 Family-Partitioned Children Buses & Child-to-Marriage Association
In family history, descendant children are **biologically and genealogically associated with a specific marriage/union**, rather than being an undifferentiated pool of children belonging to an individual. A tree that combines children of different marriages under a single shared bus corrupts parentage records and misleads users.

1. **Strict Child-to-Marriage Binding:**
   - Every child in the tree is linked to their parents via their canonical `family_id` (`family_children.family_id` from [10-core-data-model.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md)):
     - $\text{Children}(U_1)$: Children whose birth family is $U_1 = (P, S_1)$.
     - $\text{Children}(U_2)$: Children whose birth family is $U_2 = (P, S_2)$.
     - $\text{Children}(U_{\text{single}})$: Children whose parent record contains only $P$ without a recorded second spouse.
   - **Zero Child Blending:** Under no circumstances may children of $U_1$ and children of $U_2$ share a horizontal bus runner, nor may children of $U_1$ hang beneath the anchor of $U_2$.
   - **Stepparent Edge Isolation:** Spouse $S_2$ is a legal or social stepparent to children of $U_1$. Connector lines from $U_2$'s marriage bar or $S_2$'s card must NEVER branch into children of $U_1$.
2. **Frontend Index Structure (`TreeGraphIndex` Enhancement):**
   - In `frontend/src/lib/tree-layouts/types.ts`, `buildTreeGraphIndex` must preserve explicit family structures rather than flattening all children into a single monolithic `rel.childIds` array:
     ```typescript
     export interface FamilyUnion {
       familyId: string;
       partner1Id: string | null;
       partner2Id: string | null;
       childIds: string[];
       marriageDate?: string | null;
       marriagePlace?: string | null;
     }
     ```
   - Layout builders (`hourglass-layout.ts`, `descendants-layout.ts`, `all-nodes-layout.ts`) must iterate over `person.unions` and query children per union, rather than aggregating all children from the individual.
3. **Dedicated Union Distribution Buses:**
   - **Union 1 Anchor & Bus:**
     - Midpoint anchor junction dot ($r = 3\text{px}$, warm ochre fill) at $(X_{U1}, Y + H_{\text{card}} / 2)$, where $X_{U1} = \frac{X_{S1} + W_{\text{card}} + X_P}{2}$.
     - Vertical drop stem runs from $(X_{U1}, Y + H_{\text{card}} / 2)$ down to $Y_{\text{bus1}} = Y + H_{\text{card}} + V_{\text{GAP}} / 2$.
     - Horizontal bus runner spans strictly from the top-center of the first child of $U_1$ to the top-center of the last child of $U_1$.
     - Individual vertical drop lines connect $Y_{\text{bus1}}$ into the top-center of each child in $\text{Children}(U_1)$.
   - **Union 2 Anchor & Bus:**
     - Midpoint anchor junction dot ($r = 3\text{px}$, warm ochre fill) at $(X_{U2}, Y + H_{\text{card}} / 2)$, where $X_{U2} = \frac{X_P + W_{\text{card}} + X_{S2}}{2}$.
     - Vertical drop stem runs from $(X_{U2}, Y + H_{\text{card}} / 2)$ down to $Y_{\text{bus2}} = Y + H_{\text{card}} + V_{\text{GAP}} / 2$.
     - Horizontal bus runner spans strictly from the top-center of the first child of $U_2$ to the top-center of the last child of $U_2$.
     - Individual vertical drop lines connect $Y_{\text{bus2}}$ into the top-center of each child in $\text{Children}(U_2)$.
   - **Single-Parent Fallback Bus:**
     - For children where no spouse/partner was recorded, a single vertical stem drops directly from the bottom-center of $P$'s card ($(X_P + W_{\text{card}} / 2, Y_P + H_{\text{card}})$) down to a dedicated bus lane at $Y_{\text{bus, single}}$, feeding strictly into the top centers of single-parent children.
4. **Kinship Disambiguation (Full Siblings vs. Half-Siblings):**
   - Full biological siblings share a common horizontal distribution runner and union anchor.
   - Half-siblings are physically and visually partitioned into distinct clusters under their respective union anchors with $\ge 264\text{px}$ horizontal anchor separation, providing an unmistakable visual distinction between maternal and paternal half-sibling lines.
5. **Ancestral Pedigree Traceability:**
   - In Pedigree View (`mode="pedigree"`), ascending along a child node traces strictly back to the biological mother and father belonging to that child's birth family ($U_j$), ensuring stepparents from other marriages are not erroneously rendered as biological ancestors.

#### 2.7.4 Recursive Subtree Width Allocation & Collision Prevention
Bottom-up width calculation recursively accounts for all unions and partitioned children subtrees:
1. **Union Children Width:**
   For each union $U_j$:
   $$W_{\text{children}}(U_j) = \sum_{i=1}^{k_j} W_{\text{subtree}}(C_{j, i}) + \max(0, k_j - 1) \times H_{\text{GAP}}$$
   (If $U_j$ has 0 children, $W_{\text{children}}(U_j) = 0$).
2. **Symmetrical Wing Widths (for $k = 2$ Spouses):**
   - **Left Wing Width ($W_{\text{left}}$):** Encompasses $S_1$ and its children subtree:
     $$W_{\text{left}} = \max\left(W_{\text{card}} + \frac{\text{SPOUSE\_GAP}}{2}, \; W_{\text{children}}(U_1)\right)$$
   - **Right Wing Width ($W_{\text{right}}$):** Encompasses $S_2$ and its children subtree:
     $$W_{\text{right}} = \max\left(W_{\text{card}} + \frac{\text{SPOUSE\_GAP}}{2}, \; W_{\text{children}}(U_2)\right)$$
   - **Total Cluster Subtree Width:**
     $$W_{\text{subtree}}(P) = W_{\text{left}} + W_{\text{card}} + W_{\text{right}}$$
3. **Guaranteed Clearance:**
   - $S_1$ and its children cluster are centered within the left wing $[X_{\text{start}}, X_{\text{start}} + W_{\text{left}}]$.
   - $P$ is placed at $X_{\text{start}} + W_{\text{left}}$.
   - $S_2$ and its children cluster are centered within the right wing $[X_P + W_{\text{card}}, X_P + W_{\text{card}} + W_{\text{right}}]$.
   - This bottom-up formulation guarantees **zero node collision** between the children of Union 1 and the children of Union 2, while maintaining consistent $\ge H_{\text{GAP}}$ ($24\text{px}$) spacing between all cards.

#### 2.7.5 Intersection & Edge Clearance Rules
1. **Horizontal Stem Separation:**
   - Midpoint anchors $X_{U1}$ and $X_{U2}$ are separated horizontally by at least:
     $$\Delta X_{\text{anchors}} \ge W_{\text{card}} + \text{SPOUSE\_GAP} = 240\text{px} + 24\text{px} = 264\text{px}$$
     This wide separation ensures vertical stems descending to Gen +1 buses never collide or merge.
2. **Staggered Bus Runner Lanes:**
   - When multiple unions exist in the same generational gap, their horizontal bus lines are assigned staggered vertical lanes ($\ge 12\text{px}$ clearance):
     - Union 1 Bus: $Y_{\text{bus1}} = Y + H_{\text{card}} + 24\text{px}$ (Lane A).
     - Union 2 Bus: $Y_{\text{bus2}} = Y + H_{\text{card}} + 36\text{px}$ (Lane B).
     - Single Parent Bus: $Y_{\text{bus, single}} = Y + H_{\text{card}} + 48\text{px}$ (Lane C).
   - This prevents horizontal runners from overlapping into ambiguous single lines even during wide-angle zoom levels.
3. **Interactive Connection HUD Integration:**
   - Each marriage line maintains its independent interactive hit area ($\ge 14\text{px}$ desktop / $\ge 20\text{px}$ mobile).
   - Clicking the Union 1 line opens `ConnectionNavPopover` displaying $P$ and $S_1$ with their specific marriage facts.
   - Clicking the Union 2 line opens `ConnectionNavPopover` displaying $P$ and $S_2$ with their specific marriage facts.

#### 2.7.6 Multiple Spouses in Ancestor Generational Tiers (Gen -1, Gen -2)
1. **Wide Hourglass Mode (`collateral=true`):**
   - If a direct ancestor (such as the Father or Mother of the focal person) had multiple marriages:
     - The focal couple's biological parents are placed at the center of Gen -1.
     - Stepmother / second wife of the father is placed to the left/outer flank of Father with an ochre marriage line.
     - Stepfather / second husband of the mother is placed to the right/outer flank of Mother with an ochre marriage line.
     - Half-siblings from ancestral remarriages hang under their respective parental union bus and sit on Gen 0 flanking the sibling group.
2. **Compact Hourglass Mode (`collateral=false`):**
   - Remarriage partners of direct ancestors and collateral half-siblings are pruned, keeping the ancestral cascade strictly confined to biological direct lineage.
   - The focal person's own multiple spouses and direct children remain fully rendered.

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
- **Collateral & Secondary Node Identification:** Cards across all branches share identical card dimensions and typography, seamlessly integrating into the canvas.
- **Direct Lineage Highlighter:** Glowing connector lines tracing the direct biological ancestry path.
- **Multi-Tier Level of Detail (LOD) States (All Nodes & Large Graphs):**
  - **LOD 0 (Full Detail, $\text{Zoom} \ge 0.5$):** Full card with avatar, complete name with patronymic, vital dates and places, and interactive hover toolbar (`Profile`, `Focus`, `Add`, `Edit`).
  - **LOD 1 (Simplified Card, $0.25 \le \text{Zoom} < 0.5$):** Lightweight card displaying avatar, full name, and birth/death years only; action buttons suppressed to maximize pan/zoom throughput.
  - **LOD 2 (Micro-Pill Mode, $\text{Zoom} < 0.25$):** Minimalist colored pill showing gender indicator and surname/initials, providing a clean bird's-eye overview of the full forest.

### 3.2 Living Individual Privacy Masking in Collateral Branches & Complete Graph
Per [02-rbac-access-control.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md), living-person privacy shielding applies unconditionally across all nodes in the tree, across every projection mode (`hourglass`, `pedigree`, `descendant`, `fan`, and `all`):
- **Default Private Instance (`ENABLE_PUBLIC_ACCESS=false`):** Unauthenticated visitors cannot access `/tree` (redirects to `/login`).
- **Public Tree Mode (`ENABLE_PUBLIC_ACCESS=true`, Unauthenticated):** Living relatives in any component or branch (living cousins, aunts/uncles, siblings, nieces/nephews, in-laws) display with single initial character placeholders (e.g. `И.` / `J.`) and neutral silhouette avatars; surnames, patronymics, dates, places, notes, and photos are completely stripped from API response payloads and DOM nodes.
- **Authenticated Family Viewers (`Viewer`):** Full visibility of living relatives across all branches (names, patronymics, dates, places, media).
- **Authenticated Editors & Admins (`Editor`, `Admin`):** Full living visibility plus node authoring handles and quick-add actions across all components.

### 3.3 Connection Line Styling & Relationship Color Palette
All tree connector lines are rendered as crisp vector paths with semantic color tokens, explicit stroke weights, and clear visual differentiation:
1. **Marriages & Spouse Unions (`kind === 'spouse'`):**
   - **Color:** Warm Archival Ochre / Amber (`--color-brand-accent` / `--color-fan-3`: Light `#C68A34`, Dark `#D9A04B`).
   - **Stroke Weight:** $2.5\text{px}$ solid line.
   - **Union Anchor Dot:** Midpoint anchor circle ($r = 3\text{px}$, warm ochre fill) at $(X_{\text{union}}, Y + H_{\text{card}} / 2)$ marking the partnership root.
   - **Visual Character:** The warm amber tone creates an immediate, welcoming distinction between marital partnerships and genealogical descent lines, eliminating visual ambiguity in crowded clusters.
2. **Direct Lineage Parent-Child (`kind === 'parent_child' && is_direct_lineage`):**
   - **Color:** Primary Brand Blue (`--color-brand-primary`: Light `#2D5584`, Dark `#4F7EB5`).
   - **Stroke Weight:** $2.5\text{px}$ solid line.
   - **Visual Character:** Prominent, confident spine tracing biological direct ancestry and descent through the focal person.
3. **Collateral Parent-Child (`kind === 'parent_child' && !is_direct_lineage`):**
   - **Color:** Neutral Slate Gray (`--color-lineage-neutral`: Light `#64748B`, Dark `#8896AB`).
   - **Stroke Weight:** $1.5\text{px}$ solid line.
   - **Visual Character:** Provides stable, clear structural scaffolding for collateral branches (aunts, uncles, cousins, nieces, nephews) without competing with the primary lineage spine.
4. **Placeholder / Ghost Parent Link (`kind === 'placeholder'`):**
   - **Color:** Muted Slate (`--color-lineage-neutral` / `stroke-secondary` at $1.5\text{px}$, dashed `strokeDasharray="5 5"`).
5. **Active / Selected Connection:**
   - **Color:** High-visibility glowing accent (`--color-brand-primary` / `--color-brand-accent` with CSS filter `drop-shadow(0 0 4px rgba(45, 85, 132, 0.45))` or dark `rgba(79, 126, 181, 0.65)`).
6. **Kinship Path Highlighter (Multi-Trace Color Palette):**
   - **Trace 1 (Primary / Active Selected):** High-contrast Primary Brand Blue (`--color-brand-primary`: Light `#2D5584`, Dark `#4F7EB5`, $3.5\text{px}$ solid stroke with glow filter `drop-shadow(0 0 4px rgba(45, 85, 132, 0.45))`).
   - **Trace 2 (Alternate Trace):** Warm Archival Ochre / Amber (`--color-fan-3`: Light `#C68A34`, Dark `#D9A04B`, $3.0\text{px}$ stroke, dashed `strokeDasharray="6 3"` or luminous amber glow).
   - **Trace 3 (Alternate Trace):** Regal Violet / Purple (`--color-fan-4`: Light `#7C3AED`, Dark `#9333EA`, $3.0\text{px}$ stroke, dashed `strokeDasharray="4 4"`).
   - **Shared Edges:** When multiple traces share common edges (e.g. a shared ancestral couple), the stroke renders in primary glowing styling, and the edge tooltip indicates the shared traces.
7. **SVG Stacking Order (Z-Index):**
   - Level 1 (Bottom): Collateral edges.
   - Level 2: Direct lineage edges.
   - Level 3: Active selected edge.
   - Level 4: Secondary kinship traces (Trace 2, Trace 3).
   - Level 5 (Top): Primary kinship trace (Trace 1).

### 3.4 Visual Contrast, Sharpness & Subpixel Anti-Aliasing (WCAG 2.2 AA Compliance)
To prevent fuzzy or blurry rendering across both light and dark themes:
1. **WCAG 2.2 AA Contrast Standards:**
   - **Text Contrast:** Minimum $4.5:1$ contrast ratio against card surface backgrounds for all text elements (names, patronymics, birth/death dates, locations, living masked initials). Secondary metadata text (`--color-text-secondary`) must maintain $\ge 4.5:1$ (Light mode: `#4B5563` on `#FFFFFF` = $7.0:1$; Dark mode: `#9CA3AF` on `#171B22` = $6.5:1$). Washed-out or low-opacity text multipliers (such as `opacity-40` on small text) are strictly forbidden.
   - **Line & Border Contrast:** Minimum $3.0:1$ contrast ratio against canvas background (`--color-bg-canvas`) for all connector lines, card borders, and badges. Washed-out fractional opacities on connectors (e.g. `stroke-secondary/40`) are replaced with solid, fully opaque semantic tokens (`--color-lineage-neutral`, `--color-brand-primary`, `--color-brand-accent`).
2. **Subpixel Blurring Prevention & Integer Pixel Snapping:**
   - **Pan/Zoom Coordinate Snapping:** During idle canvas state and at the end of camera animations, canvas translate coordinates are rounded to integer device pixels (`Math.round(pan.x)`, `Math.round(pan.y)`). Non-integer CSS translate values (e.g. `translate3d(142.37px, 281.82px, 0)`) trigger GPU subpixel interpolation that blurs text, SVG paths, and card borders.
   - **SVG Path Rendering:** All orthogonal connector lines specify `shapeRendering="geometricPrecision"` (or `crispEdges` on 1px/1.5px orthogonal straight stems), preventing fractional anti-aliasing fuzziness.
   - **Card Borders:** Node cards feature solid, crisp borders (`1px solid var(--color-border-subtle)`: Light mode `#CBD5E1`, Dark mode `#334155`) with contrast $\ge 3:1$ against the canvas background, avoiding semi-transparent, indistinct card outlines.
   - **Font Smoothing:** Enforce `-webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; text-rendering: optimizeLegibility;` across all canvas card typography.

---

## 4. Canvas Navigation & Controls

### 4.1 Fluid Navigation
- **Pan & Zoom:** Smooth hardware-accelerated movement via pointer drag, mouse wheel, or mobile multi-touch pinch-to-zoom with natural velocity dampening (see [18 §3.6](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/18-progressive-web-app-mobile.md#36-interactive-tree-canvas-touch-gestures) for touch gesture specifications).
- **Mini-Map (Radar View):** Collapsible bottom-right viewport thumbnail displaying tree density and the current camera viewport rectangle (collapsed by default on mobile `< 768px`). In `mode="all"`, the mini-map encompasses the entire multi-component forest bounding box.
- **Camera Jump (Focal Lerp):** Selecting an individual smoothly glides the camera and centers them at optimal zoom level.

### 4.2 Toolbar Controls
- **Projection Mode Switcher (`SegmentedControl`):**
  - **Hourglass (`GitFork`):** Ancestors above, descendants below, with collateral expansion option.
  - **Pedigree (`GitBranch`):** Horizontal direct ancestors.
  - **Descendants (`Workflow`):** Generational cascade.
  - **Fan Chart (`Disc`):** Concentric circular sunburst.
  - **All Nodes (`Network`):** Full unpruned archive forest displaying all visible persons and families on a unified canvas.
- **Dynamic Control Visibility in All Nodes Mode (`mode === 'all'`):**
  - **Generation Depth Selector:** Suppressed/hidden (or rendered as an informational count badge, e.g. `{count} individuals` / `{count} персон`), because traversal in `all` mode encompasses all records without depth pruning.
  - **Hourglass Collateral Branches Toggle ("Wide / Compact"):** Suppressed/hidden (exclusive to `hourglass` mode).
  - **Search & Quick Jump:** Omnibox (`Cmd+K` / `Ctrl+K`) with instant fuzzy search; selecting a result smoothly glides the camera across components to center on the target node.
  - **Center on Focal Person (`⌂`):** Recenter camera on the focal individual in the primary component and restore 100% zoom.
  - **Reset View / Fit to Screen (`⛶`):** Instantly calculates the combined bounding box of all connected components and shelf cards, smoothly framing the entire archive forest on screen.
- **Hourglass Collateral Branches Toggle ("Wide / Compact"):**
  - Rendered dynamically in the top toolbar whenever `mode === 'hourglass'`.
  - Segmented pill or toggle button:
    - **Wide (`collateral=true`, default):** Displays collateral branches (aunts, uncles, cousins, siblings, nieces/nephews) alongside direct ancestors and descendants.
    - **Compact (`collateral=false`):** Hides collateral branches, displaying only direct lineal ascendants and descendants.
  - Toggling recalculates layout in-memory and smoothly transitions camera bounds without full page reload.
- **Search & Quick Jump:** Omnibox (`Cmd+K` / `Ctrl+K`) with instant fuzzy search by first name, surname, maiden name, or birth year; selecting a result flies directly to the node.
- **Fullscreen Mode:** Maximizes canvas area for immersive exploration.

### 4.3 Interactive Connection Selection & Connection Navigation HUD (`ConnectionNavPopover`)
Users can interact directly with tree connection lines to inspect relationships and navigate smoothly between endpoints:
1. **Transparent Hit Area Ergonomics (14px Desktop / 20px Mobile):**
   - Standard 1.5px–2.5px SVG paths are too thin for reliable clicking or tapping.
   - Dual-path rendering architecture:
     - **Visible Path:** Renders the styled line with `pointer-events: none`.
     - **Interactive Hit Path:** Renders an underlying duplicate path with `fill="none"`, `stroke="transparent"`, `strokeWidth={14}` (desktop) or `strokeWidth={20}` (mobile/touch devices), `pointerEvents="stroke"`, and `cursor="pointer"`.
   - **Hover Affordance:** Hovering over the hit path elevates the visible line's stroke width by $+1\text{px}$ and displays a cursor pointer indicator.
2. **Connection Selection State:**
   - Clicking or tapping the hit path selects the edge (`selectedEdgeId = edge.id`).
   - The selected edge illuminates with the active glowing accent.
   - Both connected endpoint cards receive an active focus ring (`ring-2 ring-brand-primary`) and elevated shadow, creating an unmistakable visual trace between the relatives.
   - Selecting an edge opens the **Connection Navigation HUD** (`ConnectionNavPopover`).
   - Clicking canvas background or pressing `Esc` clears edge selection and closes the HUD.
3. **Connection Navigation HUD / Popover Anatomy:**
   A floating, frosted overlay panel (`bg-surface/95 backdrop-blur-md border border-border shadow-elevation-2 rounded-xl p-3 z-30`) positioned dynamically near the clicked edge midpoint (or docked top-center / bottom sheet on mobile `< 768px`):
   - **Relationship Header Badge:** Localized relationship type pill:
     - `"Marriage"` / `"Брак"` (displays marriage date and place if recorded in the family union).
     - `"Parent → Child"` / `"Родитель → Ребёнок"`.
   - **Endpoint Relatives List:** Two interactive participant chips for Person A (Source) and Person B (Target):
     - Participant avatar (with living silhouette fallback in public guest mode).
     - Full localized name with patronymic (masked to single initial with dot in public guest mode).
     - Kinship / Role label (e.g. *"Father"*, *"Mother"*, *"Spouse"*, *"Son"*, *"Daughter"*).
     - **"Jump to Person" Action Button (`Navigation` / `Focus` icon):** Clicking triggers a smooth camera glide (`camera.lerp`) centering the viewport directly on that relative's card and briefly pulsing the card border.
   - **Action Bar:**
     - **"Find Kinship Path" Button (`Route` icon):** Pre-populates the Kinship Finder dialog with the two endpoints and opens the calculation modal.
     - **"View Profile" Link:** Opens `/persons/:id` in a new tab or drawer.
     - **Close Button (`✕`):** Dismisses the HUD.
4. **Living Person Privacy in Connection HUD:**
   Per [02-rbac-access-control.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md), when unauthenticated public guest mode is active, living endpoints in the HUD display with single initial character placeholders (e.g. `И.` / `J.`), silhouette avatars, and zero vital dates.

---

## 5. Technical Implementation Details

### 5.1 Graph Traversal & Data Model Reference
The tree visualization engine operates directly over the canonical schema in [10-core-data-model.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md) without schema alterations:
- **Upward Ancestor Traversal:** Starting from the focal person ($F$), queue-based BFS traverses parental unions upwards up to `generations` limit, collecting direct ancestor IDs and family unions.
- **Collateral Branch Expansion (when `collateral=true`):**
  For each ancestor union discovered:
  1. Inspect `family_children` to identify all sibling children.
  2. For non-direct children (aunts, uncles, siblings, great-aunts/uncles), enqueue them for collateral expansion.
  3. Traverse collateral marriages (`families` where collateral person is partner1 or partner2) and collect spouses.
  4. Cascade downwards through collateral children (cousins, nieces/nephews) up to the generational boundary:
     - Children of aunts/uncles $\to$ First cousins (Gen 0).
     - Children of first cousins $\to$ First cousins once removed (Gen +1).
     - Children of siblings $\to$ Nieces & nephews (Gen +1).
     - Children of nieces/nephews $\to$ Grand-nieces & grand-nephews (Gen +2).
- **Downward Descendant Traversal:** From focal couple, cascades downwards collecting direct children, grandchildren, and their spouses.
- **All Nodes Full Graph Traversal (`mode="all"`):**
  Bypasses focal-tree BFS pruning entirely to encompass the complete database forest:
  1. **Visibility Filtering:** Collects all visible persons (`store.persons`) matching caller RBAC access tier (`!p.is_private || can_view_private`).
  2. **Family Ingestion:** Collects all visible families where partners and children belong to visible persons.
  3. **Connected Component Partitioning:**
     - Builds an undirected connectivity graph $G = (V, E)$ over visible persons via spouse unions and parent-child edges.
     - Partitions $V$ into connected components: $[C_{\text{primary}}, C_2, \dots, C_k]$.
     - $C_{\text{primary}}$ is assigned to the component containing `focal_id` (or `resolved_home_person_id`). If neither is specified, the largest component by node count is selected.
  4. **Topological Tier Leveling:**
     - In $C_{\text{primary}}$, anchors `focal_id` at Generation 0 ($Y = 0$). Propagates relative tiers upwards to direct ancestors (Gen -1, Gen -2...) and downwards to descendants (Gen +1, Gen +2...). Spouses share identical generational baselines.
     - In secondary components $C_j$, identifies root patriarchs/matriarchs (in-degree 0 in parentage). If birth dates exist, computes chronological generational delta $\Delta \text{Gen} = \text{round}\left(\frac{\text{median\_birth\_year}(C_j) - \text{birth\_year}(\text{focal})}{30}\right)$ to align generational epochs horizontally with the primary tree; falls back to Gen 0 if birth dates are unknown.
  5. **Direct Lineage Flagging:**
     - Direct biological ancestry and descendant path of `focal_id` is flagged with `is_direct_lineage = true`. All lateral, in-law, and secondary component nodes are flagged with `is_direct_lineage = false`.
  6. **Living Person Shielding:**
     - Every node across all components is passed through `redact_person_for_tree_view(&p, &access)` to shield living persons in public guest mode.

### 5.2 API Data Contract
- **Endpoint:** `GET /api/v1/tree/view`
- **Query Parameters:**
  | Parameter | Type | Required | Default | Description |
  | :--- | :--- | :--- | :--- | :--- |
  | `focal_id` | String | No | Resolved home person | UUID or GID of the focal person (used as centering anchor and Gen 0 reference) |
  | `mode` | String | No | `"hourglass"` | Projection mode: `"hourglass"`, `"pedigree"`, `"descendant"`, `"fan"`, `"all"` |
  | `generations` | Integer | No | `4` | Generation depth limit (`1..=10`). Ignored when `mode="all"` |
  | `collateral` | Boolean | No | `true` | When `mode=hourglass`, whether to include collateral branches. Ignored when `mode="all"` |

- **Response Payload (`TreeViewResponse`):**
  ```json
  {
    "access_tier": "authenticated_viewer",
    "tree_title": "Petrov Family Tree",
    "focal_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "mode": "all",
    "generations": 10,
    "collateral": true,
    "nodes": [
      {
        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "gid": "I0001",
        "display_name": "Aleksandr Ivanovich Petrov",
        "sex": "M",
        "generation": 0,
        "is_direct_lineage": true,
        "is_living": false,
        "birth_date": "14 Mar 1912",
        "birth_place": "Boston, MA",
        "death_date": "22 Oct 1989",
        "death_place": "Chicago, IL",
        "primary_avatar_url": "/api/v1/media/..."
      }
    ],
    "families": [
      {
        "id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        "partner1_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "partner2_id": "e2a1b3c4-d5e6-4f7a-8b9c-0d1e2f3a4b5c",
        "children": [
          { "child_id": "b1c2d3e4-f5a6-4b7c-8d9e-0f1a2b3c4d5e" }
        ]
      }
    ],
    "edges": [
      {
        "id": "edge-p1-child",
        "source": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "target": "b1c2d3e4-f5a6-4b7c-8d9e-0f1a2b3c4d5e",
        "kind": "parent_child",
        "family_id": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        "is_direct_lineage": true
      }
    ],
    "resolved_home_person_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "total_visible": 184
  }
  ```
- **Error Responses:**
  - `401 Unauthorized`: Instance in default private posture (`ENABLE_PUBLIC_ACCESS=false`) without authentication.
  - `404 Not Found` (`code: "person_not_found"`): `focal_id` does not match any existing visible person.
  - `422 Unprocessable Entity` (`code: "validation_failed"`): Invalid `mode` (must be `hourglass`, `pedigree`, `descendant`, `fan`, or `all`) or `generations` outside `1..=10`.

- **Endpoint:** `GET /api/v1/kinship`
- **Query Parameters:**
  | Parameter | Type | Required | Default | Description |
  | :--- | :--- | :--- | :--- | :--- |
  | `from` | String | Yes | — | UUID or GID of the source individual |
  | `to` | String | Yes | — | UUID or GID of the target individual |
  | `max_traces` | Integer | No | `5` | Maximum number of distinct genealogical traces to compute (`1..=10`) |

- **Response Payload (`KinshipResponse` with Multi-Trace Support):**
  ```json
  {
    "connected": true,
    "from_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "to_id": "e2a1b3c4-d5e6-4f7a-8b9c-0d1e2f3a4b5c",
    "traces_count": 2,
    "relationship_key": "half_brother_paternal",
    "label_en": "Half-brother (paternal)",
    "label_ru": "Единокровный брат (по отцу)",
    "degree_of_kinship": 2,
    "steps_count": 2,
    "is_blood_relation": true,
    "path_person_ids": ["3fa85f64...", "father-uuid...", "e2a1b3c4..."],
    "traces": [
      {
        "id": "trace-1",
        "relationship_key": "half_brother_paternal",
        "label_en": "Half-brother (paternal)",
        "label_ru": "Единокровный брат (по отцу)",
        "degree_of_kinship": 2,
        "is_blood_relation": true,
        "common_ancestor_ids": ["father-uuid..."],
        "path_person_ids": ["3fa85f64...", "father-uuid...", "e2a1b3c4..."],
        "steps": [
          { "person_id": "3fa85f64...", "relation_to_next": "Father" },
          { "person_id": "father-uuid...", "relation_to_next": "Son" },
          { "person_id": "e2a1b3c4...", "relation_to_next": null }
        ]
      },
      {
        "id": "trace-2",
        "relationship_key": "step_brother_maternal_marriage",
        "label_en": "Step-brother (via mother's marriage)",
        "label_ru": "Сводный брат (по браку матери)",
        "degree_of_kinship": 3,
        "is_blood_relation": false,
        "common_ancestor_ids": [],
        "path_person_ids": ["3fa85f64...", "mother-uuid...", "stepfather-uuid...", "e2a1b3c4..."],
        "steps": [
          { "person_id": "3fa85f64...", "relation_to_next": "Mother" },
          { "person_id": "mother-uuid...", "relation_to_next": "Spouse" },
          { "person_id": "stepfather-uuid...", "relation_to_next": "Son" },
          { "person_id": "e2a1b3c4...", "relation_to_next": null }
        ]
      }
    ]
  }
  ```

### 5.3 Permissions & Access Control Matrix
| Access Tier | Direct Ancestors / Descendants | Collateral Relatives (Aunts, Uncles, Cousins) | All Nodes Complete Graph (`mode="all"`) | Living Relative Privacy Masking |
| :--- | :--- | :--- | :--- | :--- |
| **Default Private** | `401 Unauthorized` | `401 Unauthorized` | `401 Unauthorized` | Not applicable (access blocked) |
| **Public Guest** | Masked living (`И.` / `J.`) | Masked living (`И.` / `J.`) | Masked living across all components | Strict masking: silhouettes, zero dates/places |
| **Viewer** | Full visibility | Full visibility | Full visibility of all archive components | Full visibility of all living relatives |
| **Editor** | Full visibility + edit handles | Full visibility + edit handles | Full visibility + edit handles across all nodes | Full visibility + node quick-add triggers |
| **Admin** | Full visibility + management | Full visibility + management | Full visibility + full tree management | Full visibility + full tree management |

### 5.4 Localization (i18n)
All UI labels respect bilingual parity (English / Russian) with zero icon glyphs or emojis in translation files (`frontend/src/i18n/en.json`, `frontend/src/i18n/ru.json`):
- `tree.modes.all`: "All Nodes" / "Весь архив"
- `tree.view_options.wide_hourglass`: "Wide tree" / "Широкое древо"
- `tree.view_options.compact_hourglass`: "Compact tree" / "Компактное древо"
- `tree.view_options.collateral_toggle`: "Include collateral branches" / "Включать боковые ветви"
- `tree.view_options.collateral_subtitle`: "Aunts, uncles, cousins, siblings, nieces and nephews" / "Дяди, тети, кузены, братья, сестры и племянники"
- `tree.toolbar.allNodesBadge`: "All archive nodes" / "Все персоны архива"
- `tree.toolbar.allNodesCount`: "{count} individuals" / "{count} персон"
- `tree.allNodes.componentCount`: "{count} family branches" / "{count} ветвей древа"
- `tree.allNodes.isolatedShelfTitle`: "Additional Individuals & Families" / "Отдельные персоны и ветви"
- `tree.allNodes.primaryBranchBadge`: "Main Family Branch" / "Основная ветвь"
- `tree.allNodes.secondaryBranchBadge`: "Branch {index}" / "Ветвь {index}"
- `tree.connection.marriage`: "Marriage" / "Брак"
- `tree.connection.parent_child`: "Parent to child" / "Связь с родителем"
- `tree.connection.jump_to`: "Jump to {name}" / "Перейти к: {name}"
- `tree.connection.find_kinship`: "Find kinship path" / "Найти степень родства"
- `tree.connection.close`: "Close connection details" / "Закрыть сведения о связи"
- `tree.connection.select_hint`: "Click connection to inspect relatives" / "Нажмите на связь для перехода"
- `tree.connection.relationship_type`: "Relationship" / "Тип связи"
- `tree.connection.view_person`: "View profile" / "Открыть профиль"
- `tree.kinship.tracesCount`: "{count} paths found" / "Найдено путей: {count}"
- `tree.kinship.traceTab`: "Path {index}: {relationship}" / "Путь {index}: {relationship}"
- `tree.kinship.highlightSelected`: "Highlight Path" / "Подсветить путь"
- `tree.kinship.highlightAll`: "Highlight All Paths" / "Подсветить все пути"
- `tree.kinship.clearHighlights`: "Clear Highlights" / "Сбросить подсветку"
- `tree.kinship.legendTitle`: "Relationship Paths" / "Пути родства"
- `tree.kinship.primaryTrace`: "Primary Trace" / "Основной путь"
- `tree.kinship.alternateTrace`: "Alternate Trace" / "Дополнительный путь"
- `tree.kinship.bloodRelation`: "Blood relation" / "Кровное родство"
- `tree.kinship.affinalRelation`: "Step / In-law relation" / "Свойство / сводное родство"

### 5.5 Edge Cases & Non-Goals
- **Multiple Spouses & Successive Marriages:** Individuals with 2 or more spouses are rendered on identical generational baselines; 2 spouses are placed symmetrically flanking the person (`[Spouse 1] === [Person] === [Spouse 2]`). Each marriage union displays an explicit ochre connector bar and anchor dot.
- **Family-Partitioned Children Buses:** Children belonging to different marriages are partitioned strictly by `family_id` and hang under their respective union bus, preventing half-siblings from being lumped together under a single partner.
- **Children with Unrecorded / Unknown Spouse:** Single-parent children hang directly from the bottom-center of the parent's card via a dedicated single-parent distribution bus.
- **Stepchildren & Ancestral Remarriage:** In Wide Hourglass mode, remarriage partners of direct ancestors (e.g. father's second wife, mother's second husband) are placed on Gen -1 with half-siblings rendered on Gen 0 under that specific union bus. In Compact Hourglass mode, ancestor remarriages and collateral half-siblings are cleanly pruned.
- **Multi-Component Forest Partitioning:** When an archive contains disconnected family trees (e.g. spouse's independent ancestral line with no known marriage node to the primary tree), each component is cleanly isolated and arranged side-by-side with $\ge 300\text{px}$ horizontal spacing, preventing inter-tree node collision.
- **Isolated Singletons:** Persons without any parent, spouse, or child links are aggregated into an organized "Isolated Records" shelf grid, ensuring they remain searchable and editable without scattering across the canvas.
- **Pedigree Collapse / Consanguinity:** When relatives marry within the tree, each person card is rendered once at their primary generational slot; secondary marriage links are routed with clean curved lines without node duplication.
- **Collinear Bus Overlap & Multi-Lane Offset:** In complex sibling clusters with multiple parent unions, bus lines sharing the same inter-row gap are offset into distinct horizontal lanes ($\ge 12\text{px}$ clearance) so lines never merge into single ambiguous cuts.
- **Subpixel Anti-Aliasing & Sharpness:** Non-integer translate coordinates on canvas panning are snapped to integer pixels during idle state, and straight connector segments use `shapeRendering="geometricPrecision"` to guarantee crisp line rendering without blur.
- **Connection Hit Target Overlap:** When two connector lines pass close to each other, the top hit area in DOM order receives the pointer event. If both are selected, the active edge is elevated to top z-index.
- **Large Graph Performance (100–1,500 Nodes):** Multi-tier Level of Detail (LOD) and DOM frustum culling ensure 60fps pan/zoom performance on desktop, tablet, and mobile devices even when viewing 1,500 individuals.
- **Non-Goals:**
  - Manual drag-and-drop node positioning. Layout positioning is 100% deterministic and automatic.
  - Three-dimensional (3D) or force-directed physics graph simulations (which produce floating, unreadable genealogy baselines).

---

## 6. Definition of Done (DoD)

- [ ] **Multiple Spouses Visualization (Hourglass & Descendants):** In Hourglass, Descendants, and All Nodes modes, an individual with multiple spouses displays each spouse on the same generational baseline with an explicit warm ochre marriage connector bar (`--color-brand-accent`, $2.5\text{px}$) and union anchor dot.
- [ ] **Symmetrical 2-Spouse Placement:** An individual with 2 spouses is positioned symmetrically in the center (`[Spouse 1] === [Person] === [Spouse 2]`), ordered chronologically by marriage date / earliest child birth date, with zero line-card crossing.
- [ ] **Family-Partitioned Children Buses:** Children are partitioned strictly by family union (`family_id`); children of Spouse 1 hang under Union 1 bus, and children of Spouse 2 hang under Union 2 bus, with zero lumping of half-siblings under a shared bus.
- [ ] **Single-Parent Children Bus Fallback:** Children with no recorded second parent hang directly from the bottom-center of the parent's card via a dedicated single-parent distribution bus.
- [ ] **Multi-Spouse Recursive Width Allocation:** Symmetrical wing width calculation ($W_{\text{left}} + W_{\text{card}} + W_{\text{right}}$) guarantees zero node collision between children clusters of different marriages.
- [ ] **Staggered Multi-Union Bus Lanes:** Horizontal bus runners from distinct unions or adjacent families sharing the same generational gap maintain $\ge 12\text{px}$ vertical clearance, preventing merged or overlapping horizontal lines.
- [ ] **Multiple Spouses in Connection HUD:** Clicking either marriage line opens `ConnectionNavPopover` displaying the specific union's participants and wedding facts.
- [ ] **Ancestor Remarriage in Wide Hourglass:** In Wide Hourglass mode (`collateral=true`), direct ancestors with multiple marriages render additional spouses on Gen -1 and half-siblings on Gen 0 under their respective parental union bus.
- [ ] **Compact Hourglass Lineal Pruning:** In Compact Hourglass mode (`collateral=false`), secondary spouses of ancestors and collateral half-siblings are pruned, retaining strictly direct lineal ascendants and descendants.
- [ ] **All Nodes Full Traversal:** `GET /api/v1/tree/view?mode=all` returns all visible persons and families in the database matching caller RBAC tier without depth or focal-branch pruning.
- [ ] **Multi-Component Forest Partitioning:** The graph is partitioned into connected components; the primary component (containing focal/home person) is centered, while secondary components are positioned side-by-side with $\ge 300\text{px}$ gaps.
- [ ] **Topological Generational Baselines in All Nodes:** Nodes in the primary and secondary components align on strict horizontal generational baselines ($Y = \text{tier} \times \text{ROW\_STEP}$), with chronological leveling for disconnected components with known birth years.
- [ ] **Isolated Records Shelf Grid:** Isolated singletons and couple fragments are organized in a clean shelf grid with uniform spacing rather than cluttering the canvas.
- [ ] **Zero Overlapping Cards:** Recursive bounding width allocation ensures zero card collisions across all components and branches.
- [ ] **Continuous Orthogonal Bus Routing:** Spouses and parent-child unions across all components connect via clean orthogonal bus runners and vertical drop lines without crossing card bodies.
- [ ] **Direct Lineage Accentuation:** The direct lineage path from ancestors through focal person to descendants is visually accentuated with distinct primary stroke styling.
- [ ] **Connection Line Relationship Differentiation:** Marriage lines render in warm ochre/amber (`--color-brand-accent`, $2.5\text{px}$), direct lineage parent-child lines in primary brand (`--color-brand-primary`, $2.5\text{px}$), and collateral lines in neutral slate (`--color-lineage-neutral`, $1.5\text{px}$) with zero washed-out low-contrast opacities.
- [ ] **WCAG 2.2 AA Contrast Compliance:** Text on person cards meets $\ge 4.5:1$ contrast against surface backgrounds; connector lines, card borders, and badges meet $\ge 3:1$ contrast against canvas backgrounds in both light and dark modes.
- [ ] **Crisp Vector & Transform Rendering:** Canvas pan translation snaps to integer pixels upon gesture completion, preventing subpixel text and border blurriness; connector SVG paths utilize `shapeRendering="geometricPrecision"`.
- [ ] **Interactive Connection Hit Target:** Every connector line provides an underlying transparent interactive hit path ($\ge 14\text{px}$ desktop, $\ge 20\text{px}$ touch) with `pointer-events: stroke` and pointer cursor on hover.
- [ ] **Edge Selection & Endpoint Highlight:** Clicking a connector selects the edge, highlights the path with a glowing accent stroke, and adds active focus rings (`ring-2 ring-brand-primary`) to both connected endpoint cards.
- [ ] **Connection Navigation HUD:** Selecting an edge opens a floating popover / HUD (`ConnectionNavPopover`) displaying relationship type, avatar badges for both endpoints, 1-click "Jump to [Person]" button, and "Find Kinship Path" trigger.
- [ ] **Smooth Camera Glide on Jump:** Clicking "Jump to [Person]" in the Connection HUD smoothly glides the camera (`camera.lerp`) to center the target person's card in the viewport.
- [ ] **Anti-Collinear Bus Routing & Clearance:** Horizontal and vertical distribution bus segments from adjacent unions maintain $\ge 12\text{px}$ clearance, preventing merged or overlapping collinear lines.
- [ ] **All Nodes Toolbar Controls:** `TreeToolbar` includes "All Nodes" mode switcher option with `Network` Lucide icon; when active, the `generations` depth selector and wide/compact toggle are hidden or replaced by an informational individuals count badge.
- [ ] **URL Query Synchronization:** Toggling to All Nodes synchronizes `?mode=all` to URL search params and restores on page reload.
- [ ] **Fit to Screen for Full Forest:** "Fit to Screen" calculates the collective bounding box of all components and isolated cards, cleanly framing the full tree archive.
- [ ] **Public Mode Living Masking Across All Components & HUD:** When unauthenticated public mode is enabled, living individuals in every component and within the Connection HUD display single-initial masking (`И.` / `J.`), silhouette avatars, and zero vital dates or places.
- [ ] **60fps Pan/Zoom up to 1,500 Nodes:** Canvas maintains fluid 60fps pan/zoom interactions for archives up to 1,500 nodes through multi-tier Level of Detail (LOD) and viewport frustum culling.
- [ ] **Radar Mini-Map Coverage:** The radar mini-map SVG reflects all components and renders the camera viewport relative to the full forest bounds.
- [ ] **Wide Hourglass Traversal:** `GET /api/v1/tree/view?mode=hourglass&collateral=true` returns direct ancestors, focal couple, direct descendants, AND all collateral branches (grandparents' children + families, parents' children + families) up to the specified `generations` limit.
- [ ] **Compact Hourglass Fallback:** `GET /api/v1/tree/view?mode=hourglass&collateral=false` returns strictly direct lineal ancestors and descendants.
- [ ] **Multi-Trace Kinship Calculation:** `GET /api/v1/kinship` and local graph calculator discover up to 5 distinct genealogical paths connecting two individuals across multiple marriages, remarriages, and collateral branches, ranking direct biological traces first followed by affinal/step traces.
- [ ] **Multi-Trace Selection in Kinship Dialog:** When multiple relationship paths exist, `KinshipFinderDialog` renders segmented trace switcher tabs displaying the specific relationship title, degree, common ancestor(s), step-by-step breadcrumb ladder, and a "Highlight All Paths" trigger.
- [ ] **Multi-Trace Canvas Visualization:** The tree canvas renders multiple kinship traces simultaneously with distinct palette colors (Trace 1: Primary Brand Blue $3.5\text{px}$ solid glow; Trace 2: Warm Archival Ochre $3.0\text{px}$; Trace 3: Regal Violet $3.0\text{px}$) with matching node card rings.
- [ ] **Floating Multi-Trace Legend HUD:** Canvas displays a compact floating legend chip showing active traces with color badges, 1-click camera lerp to each trace, and a "Clear Highlights" (`✕`) button.
- [ ] **Localization Parity:** English and Russian strings are fully provided for all new controls with zero raw English leakage and zero icon glyphs in JSON string values.
