# Implementation Report: Phase 1.7 — Person Profile: Timeline, Family, Media & Notes Tabs

**Date:** 2026-10-27  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 7 of [`docs/roadmap.md`](../roadmap.md) — Comprehensive Person Profile Page (`/persons/:id`), Hero identity header with deceased `†` cross and zero Living pill, dynamic kinship pill relative to caller's home person, dual-pane Chronology & Life Map, Family & Relations tab, Media Gallery tab, Notes & Sources tab with Markdown CRUD, and fast tree canvas navigation ([`03-person-profile-page.md`](../pages/03-person-profile-page.md), [`10-core-data-model.md`](../features/10-core-data-model.md), [`07-person-family-management.md`](../features/07-person-family-management.md), [`02-rbac-access-control.md`](../features/02-rbac-access-control.md), [`16-dashboard-home.md`](../features/16-dashboard-home.md), [`00-internationalization-localization.md §3`](../features/00-internationalization-localization.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Backend Aggregated Profile Endpoint & BFS Kinship Solver (`backend/`)
- **Single-Request Profile Aggregation (`GET /api/v1/persons/{id}/profile`):**
  - Resolves person by UUID or case-insensitive GID (`I0100`).
  - Supports `?include=kinship,events,families,media,notes,sources` (defaults to all sections).
  - Returns `PersonProfileResponseDto` containing:
    - `person`: `TreePersonNodeDto` (redacted per caller access tier).
    - `kinship`: `Option<ProfileKinshipDto>` computed relative to caller's `resolved_home_person_id`.
    - `events`: `Vec<ProfileLifeEventDto>` combining individual and family events, sorted chronologically with computed `age_at_event`, geocoded place coordinates, and source text.
    - `families`: `ProfileFamiliesDto` (`parent_families` with parents and siblings; `spouse_families` with partners, marriage date/place, and children).
    - `media`: `Vec<MediaAssetSummaryDto>`.
    - `notes`: `Vec<NoteDto>` (Markdown notes filtered by caller RBAC).
    - `sources_summary`: `Vec<ProfileSourceSummaryDto>` deduplicating archival sources.
    - `capabilities`: `ProfileCapabilitiesDto` (`can_edit`, `can_delete`, `can_merge`, `is_home_person`).
- **Cycle-Safe BFS Kinship Solver ([`backend/src/services/genealogy.rs`](../../backend/src/services/genealogy.rs)):**
  - Traverses the family DAG up to 6 generations with cycle detection, computing shortest kinship paths:
    `self`, `parent`, `child`, `grandparent`, `grandchild`, `great_grandparent`, `great_grandchild`, `spouse`, `sibling`, `pibling` (uncle/aunt), `nibling` (nephew/niece), `first_cousin`, `parent_in_law_husband`, `parent_in_law_wife`.
  - Returns localized labels (`label_en`, `label_ru`) matching target gender.
- **Living-Person Public Shield:**
  - When accessed anonymously in public mode (`ENABLE_PUBLIC_ACCESS=true`), living individuals are masked to a single initial character with dot (`"A."` / `"И."`), with silhouette avatar, and stripped dates, places, notes, maps, and media.
  - In default private mode (`ENABLE_PUBLIC_ACCESS=false`), unauthenticated access yields `401 Unauthorized`.
  - Private records (`is_private = true`) return `404 person_not_found` indistinguishable from non-existent IDs for non-editors.

### 1.2 Frontend Person Profile Page & Modular Architecture (`frontend/`)
- **Profile Page ([`frontend/src/pages/PersonProfilePage.tsx`](../../frontend/src/pages/PersonProfilePage.tsx)):**
  - Mounted at `/persons/:id` in `App.tsx`.
  - Synchronizes active tab via URL search param (`?tab=chronology|family|media|notes`).
- **Hero Identity Header ([`ProfileHeroHeader.tsx`](../../frontend/src/components/profile/ProfileHeroHeader.tsx)):**
  - Large portrait avatar (`w-28 h-28`) with photo count badge.
  - Serif typography with alternate names and maiden name display.
  - **No "Living" badge pill:** living individuals display birth details only, preserving horizontal room and preventing name truncation.
  - **Subtle `†` cross:** deceased individuals display death date/place and lifespan age preceded by `†`.
  - **Dynamic Kinship Pill:** displays calculated relationship relative to home person (e.g. `You (Home Person)`, `Father`, `Great-Grandmother`), linking to kinship finder or home person settings.
  - Actions: `View in Tree` (`/tree?focus={id}`), `Edit Profile`, `+ Add Event`, `+ Add Relative`, and `⋯` overflow menu (*Set as Home Person*, *Mark Private*, *Delete*, *Merge with…*).
- **Tab 1: Chronology & Biography ([`ProfileChronologyTab.tsx`](../../frontend/src/components/profile/ProfileChronologyTab.tsx) & [`ProfileLifeMap.tsx`](../../frontend/src/components/profile/ProfileLifeMap.tsx)):**
  - 12-column dual-pane layout: chronological life events timeline with age calculations, source citations, scan indicators, and inline edit pencil.
  - Interactive geocoded Life Map plotting migration waypoints with directional vectors, with clean fallback notice when map tiles are disabled.
  - Markdown narrative biography with inline Editor/Admin editing.
- **Tab 2: Family & Relationships ([`ProfileFamilyTab.tsx`](../../frontend/src/components/profile/ProfileFamilyTab.tsx)):**
  - Parents & Siblings card group and Spouses & Children card group with relationship badges (`birth`, `adopted`, `step`, `foster`).
  - Clickable relative cards navigating to `/persons/:id` and quick `+ Add Relative` buttons.
- **Tab 3: Media Gallery ([`ProfileMediaTab.tsx`](../../frontend/src/components/profile/ProfileMediaTab.tsx)):**
  - Segmented grid for Portraits/Photographs and Archival Documents/Scans with face tag count badges and full-resolution lightbox viewer.
- **Tab 4: Notes & Sources ([`ProfileNotesTab.tsx`](../../frontend/src/components/profile/ProfileNotesTab.tsx) & [`NoteEditDialog.tsx`](../../frontend/src/components/profile/NoteEditDialog.tsx)):**
  - Full CRUD for Markdown research notes, private note 🔒 lock badge, and deduplicated sources used summary linking to timeline events.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | 0 formatting diffs |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (34/34 tests)** | 9 test suites: `person_profile_test.rs` (3 tests), `tree_view_test.rs` (3 tests), `onboarding_home_person_test.rs` (3 tests), `core_data_model_test.rs` (6 tests), `auth_test.rs` (4 tests), `operations_test.rs` (7 tests), `rbac_test.rs` (4 tests), `gendate_fixtures_test.rs` (2 tests), `health_test.rs` (2 tests) |
| **Frontend Unit & Component Tests** | `npm test -- --run` | **Passed (72/72 tests)** | 10 test suites: `person-profile.test.tsx` (4 tests), `tree-canvas.test.tsx` (6 tests), `onboarding-and-home-person.test.tsx` (5 tests), `parity.test.ts` (6 tests), `person-and-places.test.tsx` (5 tests), `rbac-ui.test.tsx` (4 tests), `auth-and-modular-ui.test.tsx` (9 tests), `settings-page.test.tsx` (5 tests), `ui-and-a11y.test.tsx` (10 tests), `gendate-parser.test.ts` (18 tests) |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | Clean production bundle in 984ms |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | 0 blocker or major findings |
| **QA & Browser Verification** | `qa-engineer` + Playwright MCP | **PASS (0 open defects)** | All 9 DoD items and live browser scenarios verified (hero header deceased cross, kinship pill, all 4 tabs, notes CRUD, inline bio, tree focus navigation, public masking) |

---

## 3. Technical Debt Resolved & Identified

See [`docs/technical-debt.md`](../technical-debt.md) for details:
- **No new technical debt introduced.**
- **Existing active items:**
  - **TD-006** (Optional): Bundle `@fontsource/*` WOFF2 assets in Docker image when network access is enabled.
  - **TD-007** (Phase 2): Mutation Response Envelope Standardization across domain APIs.

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 8 — Omnibox Search + People Directory:** [`docs/features/13-search-people-directory.md`](../features/13-search-people-directory.md) & [`docs/pages/08-directory-pages.md`](../pages/08-directory-pages.md).
