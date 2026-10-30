# Implementation Report: Milestone 2.5 — All Nodes / Complete Graph View Mode

**Date:** 2026-10-28  
**Status:** Completed, Verified & Approved  
**Scope:** Milestone 2.5 of [`docs/roadmap.md`](../roadmap.md) — `TASK-TREE-ALL-NODES-MODE`: Complete Forest & Graph View Canvas, Multi-Component Partitioning, Isolated Records Shelf, Multi-Tier LOD, Toolbar Count Badge, and Kinship Finder Fix ([`04-tree-visualizations.md`](../features/04-tree-visualizations.md#26-all-nodes-complete-graph-view), [`02-tree-view-page.md`](../pages/02-tree-view-page.md#38-all-nodes-canvas-layout--multi-component-forest-rendering)).

---

## 1. Summary of Completed Deliverables

### 1.1 Unpruned Forest Traversal & Component Partitioning (`backend/src/routes/rbac.rs`)
- **Complete Forest Query (`mode = "all"`):**
  - Updated `TreeViewQuery` to accept `mode = "all"`, bypassing focal BFS pruning.
  - Returns all visible persons and families in the database matching caller RBAC access tier.
  - Partitioned graph into connected components using Disjoint Set Union (DSU).
  - Evaluated primary component containing focal person centered symmetrically at $X \approx 0$.
  - Assigned secondary components side-by-side with $\ge 300\text{px}$ gap, leveled chronologically by median birth year.
  - Grouped isolated singletons into Gen 0 baseline records shelf.
- **RBAC Privacy Shielding:**
  - Enforced strict privacy rules in unauthenticated public guest mode (`public_anonymous`): living individuals in all branches are masked with single initial with dot (`И.` / `J.`), silhouette avatars, and zero vital dates/places.

### 1.2 Multi-Component Forest Layout & LOD Routing (`frontend/src/lib/tree-layouts/all-nodes-layout.ts`)
- **Forest Layout Engine:**
  - Implemented `layoutAllNodesTree` computing bottom-up widths ($W_{\text{branch}} = \max(W_{\text{couple}}, W_{\text{children}})$) per component with zero card overlap.
  - Arranged primary component at canvas center and secondary components horizontally offset with clean visual hierarchy.
  - Organized isolated singletons into a responsive 4-column shelf grid positioned below the lowest generational tier, aligned with `minAllX` (`UX-050`).
  - Added continuous orthogonal SVG edge routing for spouse connectors and generational distribution buses.
- **Multi-Tier Level of Detail (LOD) in `PersonNodeCard.tsx`:**
  - **LOD 0 ($\ge 0.5x$):** Full interactive card with avatar, vital dates, places, key indicators, Focus and Add action buttons.
  - **LOD 1 ($0.25x - 0.5x$):** Compact card omitting action footer and secondary metadata for clean medium-zoom overview.
  - **LOD 2 ($< 0.25x$):** Ultra-compact $28\text{px}$ micro-pill centered within standard $92\text{px}$ bounding slot (`h-[92px] flex items-center justify-center`) with vertical connector stubs (`UX-048`), concise surname + initials formatting (`UX-051`), and home person anchor badge (`UX-052`).

### 1.3 Canvas & Toolbar Adaptation (`TreeCanvas.tsx` & `TreeToolbar.tsx`)
- **Toolbar Experience:**
  - Added "All Nodes" option to `TreeToolbar` SegmentedControl with `Network` Lucide icon.
  - Dynamically replaced generations depth selector and Wide/Compact toggle with total individuals count badge (`tree.toolbar.allNodesCount` with correct EN/RU pluralization, `UX-053`).
  - Synchronized selection with URL query parameter (`?mode=all`).
- **Canvas Scaling & Landmarks:**
  - Extended canvas zoom range down to `MIN_ZOOM = 0.1` for bird's-eye forest navigation.
  - Added semantic component landmark headers (`role="heading"`, `aria-level={3}`, `UX-049`) with inverse scale clamp (`scale(clamp(1.0, 0.6 / zoom, 2.5))`) to maintain legibility without layout explosion.
  - Enforced viewport frustum culling maintaining 60fps pan/zoom.

### 1.4 Kinship Finder Dropdown Selection Lock Fix (`KinshipFinderDialog.tsx`)
- **Bug Fix:**
  - Resolved regression where selecting a different relative for Person B in `KinshipFinderDialog` was immediately overwritten back to `initialPersonBId`.
  - Removed state variables `personAId` and `personBId` from the initialization `useEffect` dependency array (`[open, initialPersonAId, initialPersonBId]`), allowing user selections to persist cleanly.

---

## 2. Verification & Automated Test Evidence

1. **Backend Integration Tests:**
   - `backend/tests/tree_view_test.rs`: `test_all_nodes_mode_unpruned_forest_and_living_masking` passing. All 51 backend tests green.
2. **Frontend Component & Integration Tests:**
   - `frontend/tests/components/tree-canvas.test.tsx` (23 tests passing):
     - Verified all visible persons and families rendered on canvas without pruning.
     - Verified multi-component forest layout with $\ge 300\text{px}$ spacing and isolated records shelf.
     - Verified dynamic toolbar adaptation: mode switcher, depth dropdown suppression, total count badge.
     - Verified LOD 0, LOD 1, and LOD 2 micro-pill rendering with connector stubs and home anchor.
   - `frontend/tests/i18n/parity.test.ts` (10 tests passing): 100% parity between EN and RU with zero icon glyphs or emojis in strings.
   - Entire frontend test suite: 18 test files, 177 tests passing.

---

## 3. UX Review Verdict

- **UX Review Report:** [`docs/ux-reviews/2026-10-28-all-nodes-mode-ux-review.md`](../ux-reviews/2026-10-28-all-nodes-mode-ux-review.md)
- **Verdict:** **OK**
- **Tasks Resolved:**
  - `UX-048`: Micro-pill vertical centering in 92px slot and edge connector stubs.
  - `UX-049`: Semantic component headers with inverse scale clamp.
  - `UX-050`: Shelf grid alignment with `minAllX`.
  - `UX-051`: Concise surname + initials formatting in LOD 2.
  - `UX-052`: Home person icon in LOD 2 micro-pill.
  - `UX-053`: English count badge pluralization grammar.
- **Tracked Technical Debt:**
  - `UX-054` recorded in `docs/technical-debt.md` for mobile SegmentedControl scroll affordance.

