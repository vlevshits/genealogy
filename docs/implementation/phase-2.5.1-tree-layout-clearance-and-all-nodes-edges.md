# Implementation Report: Tree Layout Clearance Standardization & All-Nodes Edge Completeness

**Date:** 2026-10-28  
**Status:** Completed, Verified & Approved  
**Scope:** Resolution of line-card collision defects and all-nodes edge omission reported on tree visualizations ([`04-tree-visualizations.md`](../features/04-tree-visualizations.md), [`02-tree-view-page.md`](../pages/02-tree-view-page.md)).

---

## 1. Summary of Completed Deliverables

### 1.1 Card Dimensions & Inter-Row Clearance Standardization
- **Universal Layout Constants (`frontend/src/lib/tree-layouts/types.ts`):**
  - Standardized `CARD_HEIGHT = 120` (from 92px) to accommodate all card content without DOM expansion.
  - Standardized `V_GAP = 60` (from 40px) providing ample vertical breathing room between generational rows.
  - Standardized `ROW_STEP = CARD_HEIGHT + V_GAP = 180px`, removing ad-hoc offsets and keeping row heights mathematically uniform.
  - Standardized `CARD_WIDTH = 240`, `H_GAP = 60`, `SPOUSE_GAP = 32`.
- **Fixed Height Lock (`PersonNodeCard.tsx`):**
  - Bound outer card container to exact `height: 120px` and `maxHeight: 120px` with `overflow-hidden` and `justify-between`.
  - Added `size="xs"` to `Button.tsx` (`min-h-6 h-6 px-2 py-0.5 text-[11px]`) for "Focus" and "Add" actions, fitting all content cleanly within 120px.
  - Re-anchored LOD 2 micro-pill slot to `h-[120px]`, centering the 28px micro-pill with `h-[46px]` top and bottom connector stubs aligning with spouse bars at $y + 60\text{px}$.
  - Added `touch-target-expand` on LOD 2 micro-pill to meet WCAG 2.2 AA ($\ge 44\times 44\text{px}$) on touch devices.
- **Deterministic 30px Bus Clearance:**
  - Distribution bus is pinned at $y_{\text{bus}} = y + \text{CARD\_HEIGHT} + V\_GAP / 2 = y + 150\text{px}$ (or $y_{\text{child}} - 30\text{px}$).
  - Strictly guarantees **30px clear space** between the bottom of parent cards and the horizontal distribution bus, and **30px clear space** between the bus and child card tops, completely preventing any edge from overlapping cards.

### 1.2 Two-Phase All-Nodes Layout Engine (`frontend/src/lib/tree-layouts/all-nodes-layout.ts`)
- **Phase 1 (Node Positioning):**
  - Positions all individuals and couple units across components into `compNodes` and `nodePositionMap = new Map<string, PositionedNode>()` before any edges are generated.
  - Resolves the bug where children positioned later in `unitMap.values()` than their parents had their edges silently dropped.
- **Phase 2 (100% Edge Coverage):**
  - **Spouse Connections:** Ingests both `data.families` and `relativesMap.spouseIds`. Spouses in adjacent units connect horizontally at $y + 60\text{px}$. Remarried or separate-unit spouses route through the upper clearance gap ($y_{\text{bus}} = y - 30\text{px}$). Cross-generational spouses route cleanly via upper inter-row gaps.
  - **Parent-Child Connections:** Children with both parents in a couple unit drop through the spouse gap; single parents or separate-unit parents drop from the card bottom. In both cases, $y_{\text{bus}} = y_{\text{child}} - 30\text{px}$.
- **Coordinate Shift Invariance:**
  - When shifting coordinates for primary centering or secondary component side-by-side assembly, `yBus` is preserved directly on the edge (`e.yBus`), eliminating the prior `midY = (sourceY + targetY) / 2` recalculations that were cutting into cards.

### 1.3 Layout Parity across Modes (`TreeCanvas.tsx`, `hourglass-layout.ts`, `descendants-layout.ts`)
- Updated `hourglass-layout.ts` and `descendants-layout.ts` to use canonical `ROW_STEP = 180px` and $y_{\text{bus}} = y + 150\text{px}$.
- Guarded vertical connector stubs in `TreeCanvas.tsx` with `mode !== 'pedigree'`, preventing inappropriate vertical stems in horizontal pedigree views.

---

## 2. Verification & Automated Test Evidence

1. **Frontend Automated Suite (`vitest run --run`):**
   - **18 test files, 182 passed (100%)** in 1.06s.
   - `assertZeroCardEdgeIntersections`: Validates zero edge-card collisions across `hourglass`, `descendant`, `pedigree`, and `all` modes.
   - `assertZeroIntersectionsEverywhere`: Validates that horizontal and vertical edge segments never penetrate card interiors.
   - Edge completeness verified for multi-marriages (3+ spouses), blended families, reverse node ordering, and disconnected forest components.
2. **Production Build:**
   - `tsc --noEmit && vite build` clean exit code 0.
3. **Backend Integration Suite (`cargo test`):**
   - **53 passed (100%)** across all 14 integration test suites.
4. **Architect Review:**
   - Verdict: **APPROVE WITH COMMENTS** (all 4 comments addressed and verified).
5. **QA Review:**
   - Verdict: **PASS**.
6. **UX Review:**
   - Report: [`docs/ux-reviews/2026-10-28-tree-layout-clearance-and-all-nodes-edges-ux-review.md`](../ux-reviews/2026-10-28-tree-layout-clearance-and-all-nodes-edges-ux-review.md)
   - Verdict: **OK**.
