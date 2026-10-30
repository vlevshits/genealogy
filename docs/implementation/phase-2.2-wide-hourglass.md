# Implementation Report: Milestone 2.2 — Wide Hourglass Mode & Collateral Branch Visualizations

**Date:** 2026-10-28  
**Status:** Completed, Verified & Approved  
**Scope:** Milestone 2.2 of [`docs/roadmap.md`](../roadmap.md) — `TASK-TREE-WIDE-HOURGLASS`: Extended Hourglass Graph Visualization with Grandparent Collateral Expansion, Non-Overlapping Layout Math, Continuous Marriage Connectors, and Direct Lineage Edge Styling ([`04-tree-visualizations.md`](../features/04-tree-visualizations.md), [`02-tree-view-page.md`](../pages/02-tree-view-page.md)).

---

## 1. Summary of Completed Deliverables

### 1.1 Backend Traversal & Metadata Expansion
- **Collateral Traversal Mode (`TreeViewQuery::collateral`):**
  - Extended `TreeViewQuery` with `collateral: Option<bool>`.
  - Added node/edge metadata in `TreeViewResponse`: `generation: Option<i32>`, `is_direct_lineage: Option<bool>`.
  - Implemented bidirectional BFS with downward collateral cascade:
    - Direct ancestors (Parents at Gen +1, Grandparents at Gen +2) and direct descendants (Children at Gen -1, Grandchildren at Gen -2).
    - In `collateral=true` mode, traverses all children and descendants of grandparents (aunts, uncles, first cousins) and siblings of the focal person.
  - Public mode privacy protections strictly shield living collateral family members according to RBAC.

### 1.2 Layout Algorithm & Non-Overlapping Math (`hourglass-layout.ts`)
- **Bottom-Up Subtree Width Allocation:**
  - Evaluates recursive subtree footprints:
    $$W_{\text{branch}} = \max\left(W_{\text{couple}}, \sum W_{\text{child\_branches}} + (K - 1) \cdot \text{SIBLING\_GAP}\right)$$
- **Half-Plane Clearance:**
  - Prevents paternal and maternal grandparent couple collisions on $Y = -320$:
    - $\text{PaternalSpanCenterX} \le \text{FocalCenterX} - \frac{\text{H\_GAP}}{2} - \text{PaternalCoupleHalfWidth}$
    - $\text{MaternalSpanCenterX} \ge \text{FocalCenterX} + \frac{\text{H\_GAP}}{2} + \text{MaternalCoupleHalfWidth}$
  - Guarantees $\ge 60\text{px}$ clearance between maternal and paternal ancestors.

### 1.3 Marriage Connectors & Continuous T-Stems
- **Continuous Child Bus Connection:**
  - Solved broken gap between horizontal marriage link and parent-child vertical stems.
  - Spouse bar is rendered horizontally between spouses at $Y + \text{CARD\_HEIGHT} / 2$.
  - Child vertical stem originates directly at the marriage union center:
    $$(X_{\text{union}}, Y + \text{CARD\_HEIGHT} / 2)$$
    and drops to the horizontal child bus ($Y + \text{CARD\_HEIGHT} + 30$), terminating into individual child cards.

### 1.4 Frontend UI & Toolbar Integration
- **Segmented Control & Query Parameter Sync:**
  - Added Wide (`collateral=true`) vs Compact (`collateral=false`) toggle in `TreeToolbar.tsx`.
  - Synchronized state with URL query parameter `?collateral=true|false` and reactive canvas refetching.
  - Adheres strictly to design-system components (`Button`, `IconButton`) and i18n rules (no icon characters or emojis in `en.json` / `ru.json`).

---

## 2. Verification & Automated Test Evidence

1. **Backend Integration Tests:**
   - `backend/tests/tree_view_test.rs::test_wide_hourglass_collateral_expansion_and_lineage_metadata`:
     - Verified collateral node and edge inclusion with `collateral=true`.
     - Verified `is_direct_lineage: false` on collateral aunts/uncles/cousins and `is_direct_lineage: true` on direct lineage.
     - Verified living privacy redaction on collateral nodes in public mode.
   - All 4 tree view integration tests passed.
2. **Frontend Component & Layout Tests:**
   - `frontend/tests/components/tree-canvas.test.tsx`:
     - 14 tests passing.
     - Verified wide tree layout math, absence of card collisions, continuous edge path generation, and toolbar toggle reactivity.
3. **Parity & Accessibility Tests:**
   - `tests/i18n/parity.test.ts`: 10 passed (zero emojis, zero icon glyphs, 100% key parity).
   - `npm run build`: Zero errors.

---

## 3. Review Verdicts

- **Architect Review (Mode 2):** **APPROVE**  
  Layout math, edge rendering continuity, and API contracts conform cleanly to architectural specs.
- **QA Analyst Verification:** **PASS**  
  All acceptance criteria verified against multi-generational test data.
- **UX Review:** **OK** ([`docs/ux-reviews/ux-review-phase-2.2-wide-hourglass.md`](../ux-reviews/ux-review-phase-2.2-wide-hourglass.md))  
  Reported items `UX-037` to `UX-043` cataloged; layout verified across viewports and themes.

---

## 4. Technical Debt Recorded

- **`TD-TREE-01`**: Great-grandparent collateral expansion bounding. Great-grandparent collaterals (great-aunts/uncles) are bounded to maintain crisp horizontal layout without exponential canvas inflation. Recorded in [`docs/technical-debt.md`](../technical-debt.md).

