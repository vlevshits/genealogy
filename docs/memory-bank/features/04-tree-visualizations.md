# Tree Visualizations — Implementation Memory

**Spec:** [docs/features/04-tree-visualizations.md](../../features/04-tree-visualizations.md) · **Status:** implemented

## 1. Summary
High-performance, interactive graphical tree canvas providing multiple genealogical perspectives: Hourglass (ancestors and descendants), Pedigree, Descendants, Radial Fan Chart, and All Nodes (complete forest graph view). Features multi-spouse family partitioning, anti-collinear line clearance, pan/zoom gesture physics, and a Multi-Trace Kinship Path Finder.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Routes | `backend/src/routes/rbac/mod.rs` | `get_tree`, `get_tree_view`, `get_fan_chart`, `get_kinship` |
| Service | `backend/src/services/genealogy/mod.rs` | Tree graph algorithms, `find_kinship_paths` (multi-trace BFS with detour pruning) |
| Canvas component | `frontend/src/components/tree/` | `TreeCanvas.tsx` (SVG/HTML canvas with pan/zoom and viewport virtualization) |
| Node cards | `frontend/src/components/tree/` | `PersonNodeCard.tsx` (compact card with photo, dates, lineage colors) |
| Layout engines | `frontend/src/lib/tree-layouts/` | `hourglass-layout.ts`, `all-nodes-layout.ts`, `pedigree-layout.ts`, `descendants-layout.ts` |
| Kinship Finder | `frontend/src/components/tree/` | `KinshipFinderDialog.tsx` and `frontend/src/lib/kinship.ts` |
| Fan Chart | `frontend/src/components/tree/` | `FanChartCanvas.tsx` (SVG arc calculations for 3-7 generations) |
| Tests | `backend/tests/tree_view_test.rs` | Graph traversal, cycle prevention, kinship distance calculation |
| Frontend tests | `frontend/tests/components/` | `tree-canvas.test.tsx`, `milestone-2-7-multi-spouse-kinship.test.tsx` |

## 3. Key Flows
1. **Interactive Tree Navigation:**
   - User navigates to tree view with focused individual `?focus=I0001`.
   - Layout engine computes coordinate grids for ancestors (ascending generations) and descendants (descending generations).
   - Lines are routed through family connector nodes using anti-collinear clearance offsets (`GOT-004`).
2. **Multi-Trace Kinship Calculation:**
   - User opens Kinship Finder and selects Person A and Person B.
   - `GET /api/v1/kinship?person_a=...&person_b=...` executes pruned bidirectional BFS in `services::genealogy`.
   - All shortest and relevant collateral paths are returned (e.g. biological kinship + spouse link) and highlighted on the canvas.

## 4. Invariants & Business Rules
- **Cycle Immunity:** Traversal algorithms must detect and break graph cycles (consanguinity or looped parent links) without infinite recursion.
- **Multiple Spouses Separation:** Children of successive marriages are partitioned strictly into their respective family connector buses.

## 5. Decisions
- **Hybrid SVG/HTML Canvas:** Render complex node cards with rich HTML/CSS for accessibility and theme styling, while rendering connection lines and HUD overlays via an underlying SVG layer.

## 6. Gotchas & Known Limitations
- `GOT-004`: Overlapping orthogonal connection lines; resolved via anti-collinear lane clearance offsets.

_Last curated: 2026-11-04 (memory bank bootstrap)_
