# Implementation Report: Phase 1.5 — Empty-Tree Onboarding & Home Person

**Date:** 2026-10-27  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 5 of [`docs/roadmap.md`](../roadmap.md) — Empty-tree onboarding welcome states and "Start with yourself" flow, home person resolution hierarchy and 🏠 indicator, personal user preferences (start page preference & monthly anniversary digest opt-in), and instance default home person configuration ([`16-dashboard-home.md §2–3`](../features/16-dashboard-home.md), [`02-tree-view-page.md`](../pages/02-tree-view-page.md), [`07-admin-settings-page.md`](../pages/07-admin-settings-page.md), [`00-internationalization-localization.md §3`](../features/00-internationalization-localization.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Database Schema (`backend/migrations/20261027000006_onboarding_and_home_person.sql`)
- Created migration [`20261027000006_onboarding_and_home_person.sql`](../../backend/migrations/20261027000006_onboarding_and_home_person.sql):
  - Added `users.start_page VARCHAR(32) NOT NULL DEFAULT 'tree' CHECK (start_page IN ('tree', 'dashboard'))`.
  - Added `users.digest_opt_in BOOLEAN NOT NULL DEFAULT FALSE`.
  - Added foreign key constraints with `ON DELETE SET NULL` on `users(person_id)` and `users(home_person_id)` referencing `individuals(id)`.
  - Added partial indexes on non-null user person links (`idx_users_person_id`, `idx_users_home_person_id`).
  - Seeded default home person key in `instance_settings`: `('default_home_person_id', 'null'::jsonb)`.
  - Advanced schema metadata version to `6`.

### 1.2 Backend Endpoints & Domain Logic (`backend/`)
- **Empty-Tree Detection & Status ([`backend/src/routes/rbac.rs`](../../backend/src/routes/rbac.rs)):**
  - Implemented `GET /api/v1/onboarding/status` (`PublicOrAuth`): returns `is_empty`, `total_individuals`, `can_onboard`, `resolved_home_person_id`, `default_home_person_id`, `user_home_person_id`, and `start_page`.
  - Enforced Default Private Gatekeeper: unauthenticated requests return `401 Unauthorized` when `enable_public_access == false`.
- **Atomic "Start with yourself" Flow:**
  - Implemented `POST /api/v1/onboarding/start-with-self` (`RequireEditor`): creates root person with monotonic GID (`I0100`), optional birth event (`E0100`) with parsed `GenDate` and auto-split place chain, links caller's `person_id` and `home_person_id`, sets `instance_settings.default_home_person_id` if unset, records in `change_log`, and syncs in-memory privacy and auth stores within an atomic transaction.
  - Rejects non-admin calls on a non-empty tree with `409 Conflict` (`already_onboarded`).
- **4-Tier Home Person Resolution Hierarchy:**
  - Evaluates: `user.home_person_id` → `user.person_id` → `instance_settings.default_home_person_id` → `None`.
  - Exposed `resolved_home_person_id` on `TreeResponse` and `OnboardingStatusResponse`.
  - Handled person deletion cascading: cleared in-memory user person references when an individual is removed (`ON DELETE SET NULL` parity).
- **User Preferences & Admin Settings:**
  - `PUT /api/v1/me/home-person`: sets or clears (`null`) caller's explicit home person override.
  - `PATCH /api/v1/me` and `PATCH /api/v1/auth/me`: updates `start_page`, `digest_opt_in`, `locale`, `theme`, and names, persisting directly to PostgreSQL.
  - `PATCH /api/v1/admin/settings`: manages `default_home_person_id`.

### 1.3 Modular Frontend UI & Integrations (`frontend/`)
- **Onboarding Components ([`frontend/src/components/onboarding/`](../../frontend/src/components/onboarding/)):**
  - [`EmptyTreeOnboardingCard.tsx`](../../frontend/src/components/onboarding/EmptyTreeOnboardingCard.tsx): Centered archival welcome card for empty instances. Admins and Editors receive `Start with yourself` (`<UserPlus />`), `Import a GEDCOM file` (`<Upload />`), and GEDCOM migration guidance. Viewers see calm message `"This family tree is still being prepared."` with action buttons hidden.
  - [`StartWithSelfDialog.tsx`](../../frontend/src/components/onboarding/StartWithSelfDialog.tsx): Dialog for Given Name, Patronymic, Surname, Sex (`M`/`F`/`U`), Birth Date (`GenDateInput`), and Birth Place.
- **Tree Canvas & App Routing ([`frontend/src/components/tree/`](../../frontend/src/components/tree/), [`frontend/src/App.tsx`](../../frontend/src/App.tsx)):**
  - Updated [`TreePrivacyPreviewCard.tsx`](../../frontend/src/components/tree/TreePrivacyPreviewCard.tsx) to render `EmptyTreeOnboardingCard` when tree has 0 nodes, display the 🏠 `Home Person` badge on the resolved home person node, offer `"Set as my home person"` on other nodes, and pulse `Add Father` / `Add Mother` buttons when the tree has a single starter individual.
  - Updated [`App.tsx`](../../frontend/src/App.tsx) to check onboarding status on `/`, redirect unauthenticated visitors to `/login` when private, and route authenticated users based on `user.start_page`.
  - Purged legacy Phase 0 promotional marketing text and developer tech-stack badges from user-facing screens and translations.
- **Settings Panels ([`frontend/src/components/settings/`](../../frontend/src/components/settings/)):**
  - [`ProfileSettingsPanel.tsx`](../../frontend/src/components/settings/ProfileSettingsPanel.tsx): Added Default Start Page selector, Home Person badge & selector with Clear button, and monthly email digest opt-in switch.
  - [`GeneralSettingsPanel.tsx`](../../frontend/src/components/settings/GeneralSettingsPanel.tsx): Added instance `default_home_person_id` badge, selector, and Clear button for Admins.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | 0 formatting diffs |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (28/28 tests)** | 7 test suites: `onboarding_home_person_test.rs` (3 tests), `core_data_model_test.rs` (6 tests), `auth_test.rs` (4 tests), `operations_test.rs` (7 tests), `rbac_test.rs` (4 tests), `gendate_fixtures_test.rs` (2 tests), `health_test.rs` (2 tests) |
| **Frontend Unit & Component Tests** | `npm test -- --run` | **Passed (62/62 tests)** | 8 test suites: `onboarding-and-home-person.test.tsx` (5 tests), `parity.test.ts` (6 tests), `person-and-places.test.tsx` (5 tests), `rbac-ui.test.tsx` (4 tests), `auth-and-modular-ui.test.tsx` (9 tests), `settings-page.test.tsx` (5 tests), `ui-and-a11y.test.tsx` (10 tests), `gendate-parser.test.ts` (18 tests) |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | Clean production bundle in 947ms |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | 0 blocker or major findings |
| **QA & Browser Verification** | `qa-engineer` + Playwright MCP | **PASS (0 open defects)** | All 7 DoD items and 4 browser scenarios verified end-to-end |

---

## 3. Technical Debt Resolved & Identified

See [`docs/technical-debt.md`](../technical-debt.md) for details:
- **No new technical debt introduced.**
- **Existing active items:**
  - **TD-006** (Optional): Bundle `@fontsource/*` WOFF2 assets in Docker image when external network access is enabled.
  - **TD-007** (Phase 2): Mutation Response Envelope Standardization across domain APIs.

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 6 — Tree Canvas:** Hourglass + Pedigree + Descendants, quick-add, drawer ([`docs/features/04-tree-visualizations.md`](../features/04-tree-visualizations.md)).
