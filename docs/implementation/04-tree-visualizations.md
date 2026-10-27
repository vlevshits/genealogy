# Implementation Report: Phase 1.6 — Tree Canvas: Hourglass, Pedigree, Descendants, Quick-Add & Drawer

**Date:** 2026-10-27  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 6 of [`docs/roadmap.md`](../roadmap.md) — Interactive SVG + HTML Tree Canvas below topbar, 3 core tree projections (Hourglass, Pedigree, Descendants), pan/zoom canvas navigation, person card anatomy with gender lineage strip, public mode living person masking, quick-add relative triggers (`+`), placeholder parent slots, right-hand inspection drawer on desktop, mobile quick-view bottom sheet, and radar mini-map ([`04-tree-visualizations.md`](../features/04-tree-visualizations.md), [`02-tree-view-page.md`](../pages/02-tree-view-page.md), [`02-rbac-access-control.md`](../features/02-rbac-access-control.md), [`00-internationalization-localization.md §3`](../features/00-internationalization-localization.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Backend Projections & Graph API (`backend/`)
- **Extended Full Tree Endpoint (`GET /api/v1/tree`):**
  - Extended `TreeResponse` with `pub families: Vec<FamilyDto>` containing caller-visible family unions and child edges.
- **Dedicated Projection View Endpoint (`GET /api/v1/tree/view`):**
  - Supported query parameters: `focal_id` (UUID or GID), `mode` (`"hourglass"`, `"pedigree"`, `"descendant"`), `generations` (`1..=10`, default `4`).
  - Implemented graph traversals in `backend/src/routes/rbac.rs`:
    - `hourglass`: anchors focal couple at center tier 0, traverses direct ancestors upwards, and traverses children/descendants downwards.
    - `pedigree`: traverses direct biological ancestors upwards/rightwards through parent unions.
    - `descendant`: cascades downwards from ancestor through child links.
  - Synthesized normalized `edges: Vec<TreeEdgeDto>` (`kind: "spouse"` and `kind: "parent_child"`).
  - Enforced RBAC Privacy: unauthenticated public mode callers see living persons masked to single initial with dot (e.g. `И.` / `J.`) and silhouette, with all private facts stripped. Default private instances enforce `401 Unauthorized`.
- **Integration Tests ([`backend/tests/tree_view_test.rs`](../../backend/tests/tree_view_test.rs)):**
  - Verified `hourglass`, `pedigree`, and `descendant` graph traversals, generation limits, `404 person_not_found`, and public living masking across all 3 modes.

### 1.2 Frontend Layout Engines (`frontend/src/lib/tree-layouts/`)
- Implemented pure TypeScript, deterministic layout algorithms without external heavy dependencies:
  - [`pedigree-layout.ts`](../../frontend/src/lib/tree-layouts/pedigree-layout.ts) (`computePedigreeLayout`): horizontal direct ancestor layout ($X = 0$ expanding right) with symmetric child centering ($Y_{\text{child}} = (Y_{\text{father}} + Y_{\text{mother}}) / 2$) and placeholder parent slots.
  - [`hourglass-layout.ts`](../../frontend/src/lib/tree-layouts/hourglass-layout.ts) (`computeHourglassLayout`): focal couple at center ($Y = 0$), ancestors branching upwards ($Y < 0$), descendants branching downwards ($Y > 0$).
  - [`descendants-layout.ts`](../../frontend/src/lib/tree-layouts/descendants-layout.ts) (`computeDescendantsLayout`): top-down cascade ($Y \ge 0$) with recursive subtree width allocation to guarantee zero node collisions.
  - [`curve-routing.ts`](../../frontend/src/lib/tree-layouts/curve-routing.ts): horizontal cubic bezier curves and vertical orthogonal distribution buses.

### 1.3 Interactive Canvas & Modular Tree Components (`frontend/src/components/tree/`)
- [`TreeCanvas.tsx`](../../frontend/src/components/tree/TreeCanvas.tsx):
  - Hardware-accelerated 60fps pan/zoom (`translate3d(...) scale(...)`) filling 100% of viewport below topbar with zero outer page scrollbars.
  - Pointer drag panning with pointer capture protection, cursor-anchored wheel zooming with native non-passive listener, and touch pinch-to-zoom.
  - Camera animations: `fitToScreen` (`⛶`) and `centerOnFocal` (`⌂`).
  - Empty-tree onboarding fallback card when 0 individuals exist.
- [`PersonNodeCard.tsx`](../../frontend/src/components/tree/PersonNodeCard.tsx):
  - 240×92px dimensions, gender accent stripe (`bg-male` / `bg-female` / `bg-neutral`), 🏠 Home Person badge, living status badge, and hover quick toolbar (`Profile`, `Focus`, `Add`, `Edit`).
  - Public masked mode: renders single initial with dot and silhouette avatar.
- [`TreeToolbar.tsx`](../../frontend/src/components/tree/TreeToolbar.tsx):
  - Top-left floating mode switcher: `Hourglass`, `Pedigree`, `Descendants`.
  - Top-right floating controls: generation depth selector (`3`, `4`, `5`, `6`, `All`), center on focal/home button, fit to screen.
  - Bottom-left zoom pill: Zoom In, Zoom Percentage reset, Zoom Out.
- [`PersonDrawer.tsx`](../../frontend/src/components/tree/PersonDrawer.tsx) & [`PersonBottomSheet.tsx`](../../frontend/src/components/tree/PersonBottomSheet.tsx):
  - Slide-over right drawer (`w-[440px]`) on desktop, and swipeable bottom sheet (25% peek / 75% expand) on mobile (<768px).
  - Displays full identity, vital dates, places, immediate family links to jump or re-center, and profile CTA.
- [`TreeMiniMap.tsx`](../../frontend/src/components/tree/TreeMiniMap.tsx):
  - Bottom-right radar overview thumbnail with interactive camera viewport rectangle and collapse button.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | 0 formatting diffs |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (31/31 tests)** | 8 test suites: `tree_view_test.rs` (3 tests), `onboarding_home_person_test.rs` (3 tests), `core_data_model_test.rs` (6 tests), `auth_test.rs` (4 tests), `operations_test.rs` (7 tests), `rbac_test.rs` (4 tests), `gendate_fixtures_test.rs` (2 tests), `health_test.rs` (2 tests) |
| **Frontend Unit & Component Tests** | `npm test -- --run` | **Passed (68/68 tests)** | 9 test suites: `tree-canvas.test.tsx` (6 tests), `onboarding-and-home-person.test.tsx` (5 tests), `parity.test.ts` (6 tests), `person-and-places.test.tsx` (5 tests), `rbac-ui.test.tsx` (4 tests), `auth-and-modular-ui.test.tsx` (9 tests), `settings-page.test.tsx` (5 tests), `ui-and-a11y.test.tsx` (10 tests), `gendate-parser.test.ts` (18 tests) |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | Clean production bundle in 1.06s |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | Round 1 & Round 2 defect fixes verified and approved |
| **QA & Browser Verification** | `qa-engineer` + Playwright MCP | **PASS (0 open defects)** | All 9 DoD items and live browser scenarios verified (mode switching, pan/zoom, drawer, quick-add, public living masking, zero console errors) |

---

## 3. Technical Debt & Tradeoffs Identified

See [`docs/technical-debt.md`](../technical-debt.md) for details:
- **No new technical debt introduced.**
- **Existing active items:**
  - **TD-006** (Optional): Bundle `@fontsource/*` WOFF2 assets in Docker image when network access is enabled.
  - **TD-007** (Phase 2): Mutation Response Envelope Standardization across domain APIs.

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 7 — Person Profile:** Timeline, family, media, notes tabs ([`docs/pages/03-person-profile-page.md`](../pages/03-person-profile-page.md)).
