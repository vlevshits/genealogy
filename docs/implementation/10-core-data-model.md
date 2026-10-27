# Implementation Report: Phase 1.4 — Core Data Model, Person/Family/Event Editing, Date Parser & Places

**Date:** 2026-10-27  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 4 of [`docs/roadmap.md`](../roadmap.md) — Core genealogy schema with 13 canonical tables, stable `gid` sequences, versioning and cycle protection ([`10-core-data-model.md`](../features/10-core-data-model.md)), Slavic patronymic suggestion engine and Person/Family/Life Event editing with safe delete impact previews ([`07-person-family-management.md`](../features/07-person-family-management.md)), Places hierarchy with comma-path auto-splitting, alternate-name search, place merging, and delete protection ([`12-places.md`](../features/12-places.md)), and resolution of **TD-004** and **TD-005**.

---

## 1. Summary of What Was Implemented

### 1.1 Database Schema (`backend/migrations/20261026000005_core_genealogy_model.sql`)
- Created migration [`20261026000005_core_genealogy_model.sql`](../../backend/migrations/20261026000005_core_genealogy_model.sql):
  - Created 5 monotonic sequence-backed ID generators (`individual_gid_seq`, `family_gid_seq`, `event_gid_seq`, `place_gid_seq`, `media_gid_seq`).
  - Created 13 canonical relational tables:
    1. `individuals` (UUID, `gid` `I0001`, given/patronymic/surname, sex, living status, optimistic `version`).
    2. `individual_alt_names` (maiden, transliterated, and religious names).
    3. `families` (UUID, `gid` `F0001`, husband/wife FKs, relationship type, `version`).
    4. `family_children` (family-child edge with lineage `birth`, `adopted`, `foster`).
    5. `life_events` (UUID, `gid` `E0001`, individual/family event type, structured `GenDate`, place FK, source citation).
    6. `notes` (polymorphic notes attached to individuals, families, events, places, or media).
    7. `places` (UUID, `gid` `P0001`, name, parent FK, coordinates, jurisdiction type, `version`).
    8. `place_alt_names` (historical names, Soviet/Imperial variants, transliterations).
    9. `media_assets` (UUID, `gid` `M0001`, storage path, mime type, checksum, `version`).
    10. `media_links` (many-to-many associations between media and individuals/events/places).
    11. `media_face_tags` (face bounding boxes on photos).
    12. `change_log` (audit trail, diffs, and undo metadata).
    13. `tree_snapshots` (versioned backup snapshots).
  - Trigram indexes for fast fuzzy search on names and places.
  - Advanced schema version to `5`.

### 1.2 Backend Domain Services & REST Endpoints (`backend/`)
- **Genealogy Services ([`backend/src/services/genealogy.rs`](../../backend/src/services/genealogy.rs)):**
  - `allocate_gid`: Formats `I%04d`, `F%04d`, `E%04d`, `P%04d`, `M%04d` from sequence values, ensuring no re-use.
  - Cycle detection: `ensure_no_ancestor_cycle` (depth-first traversal preventing ancestor-descendant loops) and `ensure_no_place_cycle` (preventing jurisdiction parent cycles).
  - `resolve_or_create_place_chain`: Splits comma-separated strings (e.g. `"Тверь, Тверская губерния, Российская империя"`) right-to-left, deduplicating parent jurisdictions automatically.
  - `recompute_individual_vitals_and_living`: Evaluates events to automatically sync living status, birth, and death dates.
  - Optimistic locking: All `PATCH` and mutation operations verify `version` matching and increment monotonically, returning `409 Conflict` on race conditions.
  - Safe delete impact summary: `compute_person_delete_impact` calculates affected relationships, cascades, and orphaned families prior to deletion.
- **GenDate Extensions ([`backend/src/models/gendate.rs`](../../backend/src/models/gendate.rs)):**
  - Added `sort_key()` computing integer chronological representations for all date modifiers.
  - Added `to_gedcom()` serializing standard GEDCOM date strings (`ABT 1895`, `BET 1910 AND 1915`, etc.).
- **Patronymic Suggestion & Cross-Script Search ([`backend/src/services/privacy.rs`](../../backend/src/services/privacy.rs)):**
  - Cyrillic and Latin Slavic patronymic derivation supporting regular and irregular root endings (`-ович/-овна`, `-евич/-евна`, `-ич/-инична`).
  - Search normalization supporting cross-script transliteration queries.
- **REST Endpoints & OpenAPI Documentation ([`backend/src/routes/rbac.rs`](../../backend/src/routes/rbac.rs), [`backend/src/routes/mod.rs`](../../backend/src/routes/mod.rs)):**
  - Exposed full CRUD, quick-add, impact summary, source autocomplete, and place merge routes.
  - Documented complete OpenAPI 3.1 DTO schemas in `/api/v1/openapi.json`.

### 1.3 Modular Frontend UI & API Client (`frontend/`)
- **TypeScript DTOs & API Client ([`frontend/src/api/openapi-types.ts`](../../frontend/src/api/openapi-types.ts), [`client.ts`](../../frontend/src/api/client.ts)):**
  - Created canonical TypeScript definitions matching backend OpenAPI schemas (**TD-004** resolved).
  - Added safe `204 No Content` handling in `apiFetch` preventing JSON parse errors on empty delete responses.
- **Person, Family & Event Dialogs ([`frontend/src/components/person/`](../../frontend/src/components/person/)):**
  - [`QuickAddRelativeDialog.tsx`](../../frontend/src/components/person/QuickAddRelativeDialog.tsx): Quick-add for parents, spouse, child, sibling with patronymic suggestions and branch linking.
  - [`PersonEditDialog.tsx`](../../frontend/src/components/person/PersonEditDialog.tsx): Tabbed editing for personal details, alternate names, and living status override.
  - [`LifeEventDialog.tsx`](../../frontend/src/components/person/LifeEventDialog.tsx): Event management with structured `GenDateInput` and source citation autocomplete.
  - [`DeleteImpactDialog.tsx`](../../frontend/src/components/person/DeleteImpactDialog.tsx): Detailed impact preview before person deletion.
- **Places Management & Navigation ([`frontend/src/components/places/`](../../frontend/src/components/places/), [`PlacesPage.tsx`](../../frontend/src/pages/PlacesPage.tsx)):**
  - [`PlacesTreeListCard.tsx`](../../frontend/src/components/places/PlacesTreeListCard.tsx): Hierarchical and flat listing with search, filtering, and missing coordinates indicators.
  - [`PlaceDetailPanel.tsx`](../../frontend/src/components/places/PlaceDetailPanel.tsx): Details view, parent breadcrumbs, alternate names list, and map privacy shield.
  - [`PlaceFormDialog.tsx`](../../frontend/src/components/places/PlaceFormDialog.tsx): Creation and editing with comma-path parent resolution.
  - [`PlaceMergeDialog.tsx`](../../frontend/src/components/places/PlaceMergeDialog.tsx): Target place selection, event/child repointing, and toast feedback.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | Zero formatting differences |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (25/25 tests)** | `core_data_model_test.rs` (6 tests), `auth_test.rs` (4 tests), `operations_test.rs` (7 tests), `rbac_test.rs` (4 tests), `gendate_fixtures_test.rs` (2 tests), `health_test.rs` (2 tests) |
| **Frontend Unit & Component Tests** | `npm test -- --run` | **Passed (57/57 tests)** | 7 test suites: `person-and-places.test.tsx`, `auth-and-modular-ui.test.tsx`, `settings-page.test.tsx`, `ui-and-a11y.test.tsx`, `parity.test.ts`, `gendate-parser.test.ts`, `rbac-ui.test.tsx` |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | `tsc --noEmit && vite build` clean in 1.06s |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | Round 1 & Round 2 defect fixes verified and approved |
| **QA & Browser Verification** | `qa-engineer` + Playwright MCP | **PASS (0 open defects)** | All 19 DoD items passed. Round 1 defects (DEF-001..DEF-004) verified resolved in live browser runs |

---

## 3. Technical Debt Resolved & Identified

See [`docs/technical-debt.md`](../technical-debt.md) for details:
- **TD-004 (RESOLVED):** Client-side OpenAPI code generation / TypeScript synchronization (`frontend/src/api/openapi-types.ts`).
- **TD-005 (RESOLVED):** SQLx offline build compatibility (all queries use runtime query binding, 0 live DB requirement at compile time).
- **TD-007 (Added):** Mutation Response Envelope Standardization (standardize merge vs update response structures across domain APIs in Phase 2).

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 5 — Empty-Tree Onboarding & Home Person:** [`docs/features/16-dashboard-home.md`](../features/16-dashboard-home.md) §2–3.
