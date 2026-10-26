# Implementation Report: Phase 1.3 — Access Control, Master RBAC Matrix & Living-Person Privacy

**Date:** 2026-10-26  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 3 of [`docs/roadmap.md`](../roadmap.md) — 5-tier access control model (`Default Private`, `Public Tree Mode`, `Authenticated Viewer`, `Authenticated Editor`, `Administrator`) ([`02-rbac-access-control.md`](../features/02-rbac-access-control.md)), canonical 5-branch living-status algorithm ([`10-core-data-model.md §5`](../features/10-core-data-model.md)), backend living-person masking and `404` direct media shield, user role management and account deactivation with Sole Admin Protection ([`07-admin-settings-page.md`](../pages/07-admin-settings-page.md)), and modular frontend RBAC UI components (`RoleBadge`, `AccessDeniedCard`, `RoleChangeDialog`, `UserDeactivateDialog`, `UserTableRow`, `PublicGuestBanner`, `TreePrivacyPreviewCard`).

---

## 1. Summary of What Was Implemented

### 1.1 Database Schema (`backend/migrations/20261026000004_rbac_and_privacy.sql`)
- Created [`20261026000004_rbac_and_privacy.sql`](../../backend/migrations/20261026000004_rbac_and_privacy.sql):
  - Added `deactivated_at TIMESTAMPTZ NULL` to `users`.
  - Added composite index `idx_users_role_is_active ON users (role, is_active)` for fast active-administrator counting (Sole Admin Protection).
  - Updated `system_metadata.schema_version` and `CURRENT_SCHEMA_VERSION` to `4`.

### 1.2 Backend RBAC Extractors, Living-Status Engine & Privacy Redaction (`backend/`)
- **Type-Safe Axum RBAC Extractors ([`backend/src/services/rbac.rs`](../../backend/src/services/rbac.rs)):**
  - Implemented `CallerAccess` (`AnonymousPublic`, `Viewer`, `Editor`, `Admin`), `AuthenticatedUser`, `RequireAuth` (`401 Unauthorized` if unauthenticated or deactivated), `RequireEditor` (`403 Forbidden` for `Viewer`), `RequireAdmin` (`403 Forbidden` for `Viewer` and `Editor`), and `PublicOrAuth` (allows `AnonymousPublic` when `enable_public_access == true`, or returns `401 Unauthorized` when `enable_public_access == false`).
- **Canonical 5-Branch Living-Status Evaluator & Privacy Serializer ([`backend/src/services/privacy.rs`](../../backend/src/services/privacy.rs)):**
  - `evaluate_is_living`: Evaluates (1) explicit `living_override`, (2) death/burial/cremation event presence, (3) known `birth_year` against `living_person_age_threshold`, (4) relative birth-year estimation (`earliest_child_birth_year - 25`, `partner_birth_year`, `parent_birth_year + 25`, `marriage_year - 25`), and (5) defaults to `true` (`Living`) when unknown.
  - `redact_person_for_caller` & `redact_media_for_caller`:
    - Masks living individuals as `"Living"` / `"Живущий"` with `patronymic: null`, `birth: null`, `death: null`, `avatar_url: null`, and `notes: null` for `AnonymousPublic`.
    - Returns full unredacted living relatives for authenticated `Viewer`, `Editor`, and `Admin` (`02 §3.3`).
    - Shields `is_private = true` records (`None` / `404 Not Found`) from `AnonymousPublic` and `Viewer`, while exposing them to `Editor` and `Admin`.
    - Enforces the Direct Media Shield (`GET /api/v1/media/{id}/file` returns `404 Not Found` for living/private media when unauthenticated, and strips living face-tag bounding boxes on group photos).
- **Admin Endpoint Hardening, User Governance & Master RBAC Matrix Routes ([`backend/src/routes/rbac.rs`](../../backend/src/routes/rbac.rs), [`auth.rs`](../../backend/src/routes/auth.rs), [`system.rs`](../../backend/src/routes/system.rs)):**
  - Hardened `/api/v1/admin/system`, `/api/v1/admin/settings`, `/api/v1/admin/backups`, `/api/v1/admin/users`, and `/api/v1/admin/invites` with `RequireAdmin`.
  - Added `PATCH /api/v1/admin/users/{id}` supporting role changes (`admin`, `editor`, `viewer`) and account activation/deactivation (`is_active`), enforcing **Sole Admin Protection** (`400 sole_admin_protected`) and immediate session purge upon deactivation.
  - Registered RBAC-enforced routes for `/api/v1/tree`, `/api/v1/persons*`, `/api/v1/media/{id}/file`, `/api/v1/people`, `/api/v1/places`, `/api/v1/media`, `/api/v1/gedcom/export`, `/api/v1/gedcom/import`, `/api/v1/export/pdf`, `/api/v1/history`, `/api/v1/quality`, and `/api/v1/people/merge`.

### 1.3 Modular Frontend RBAC Governance, Route Guards & Privacy Views (`frontend/`)
- **Reusable UI Primitives (`frontend/src/components/ui/`):**
  - [`role-badge.tsx`](../../frontend/src/components/ui/role-badge.tsx) (`RoleBadge`) and [`access-denied-card.tsx`](../../frontend/src/components/ui/access-denied-card.tsx) (`AccessDeniedCard`).
- **Modular User & Role Governance (`frontend/src/components/settings/`):**
  - Split role and account actions into [`UserTableRow.tsx`](../../frontend/src/components/settings/UserTableRow.tsx), [`RoleChangeDialog.tsx`](../../frontend/src/components/settings/RoleChangeDialog.tsx), and [`UserDeactivateDialog.tsx`](../../frontend/src/components/settings/UserDeactivateDialog.tsx), composed inside [`FamilyMembersInvitesPanel.tsx`](../../frontend/src/components/settings/FamilyMembersInvitesPanel.tsx) with Sole Admin Protection warnings.
- **Public Guest Banner & Tree Privacy Preview (`frontend/src/components/tree/`):**
  - Created [`PublicGuestBanner.tsx`](../../frontend/src/components/tree/PublicGuestBanner.tsx) and [`TreePrivacyPreviewCard.tsx`](../../frontend/src/components/tree/TreePrivacyPreviewCard.tsx), plus 5-tier route and navigation guards in [`SettingsPage.tsx`](../../frontend/src/pages/SettingsPage.tsx), [`app-shell.tsx`](../../frontend/src/components/ui/app-shell.tsx), and [`App.tsx`](../../frontend/src/App.tsx).

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | Zero formatting diffs |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (19/19 tests)** | `rbac_test.rs` (4 tests), `auth_test.rs` (4 tests), `operations_test.rs` (7 tests), `gendate_fixtures_test.rs` (2 tests), `health_test.rs` (2 tests) |
| **Frontend Unit & A11y Tests** | `npm test -- --run` | **Passed (51/51 tests)** | `rbac-ui.test.tsx` (4 tests), `auth-and-modular-ui.test.tsx` (9 tests), `settings-page.test.tsx` (5 tests), `ui-and-a11y.test.tsx` (10 tests), `parity.test.ts` (5 tests), `gendate-parser.test.ts` (18 tests) |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | `tsc --noEmit && vite build` clean in 893ms |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | 0 blocker or major findings |
| **QA & Browser Verification** | `qa-engineer` + Playwright MCP | **PASS (0 defects)** | All 11 DoD items and 4 live Playwright browser scenarios verified in EN & RU |

---

## 3. Technical Debt & Tradeoffs Identified

See [`docs/technical-debt.md`](../technical-debt.md) for the full registry:
- **No new technical debt introduced.**
- **Upcoming TD items scheduled for Phase 1.4 (`10-core-data-model`):** **TD-004** (OpenAPI TypeScript client types) and **TD-005** (SQLx offline metadata / query verification).

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 4 — Core Data Model, Person/Family/Event Editing, Date Parser & Places:** [`docs/features/10-core-data-model.md`](../features/10-core-data-model.md), [`docs/features/07-person-family-management.md`](../features/07-person-family-management.md), and [`docs/features/12-places.md`](../features/12-places.md).
