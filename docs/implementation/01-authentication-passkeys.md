# Implementation Report: Phase 1.2 — Authentication, Passkeys & Sessions

**Date:** 2026-10-26  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 2 of [`docs/roadmap.md`](../roadmap.md) — First-run instance initialization (`/setup`), WebAuthn Passkeys (`01-authentication-passkeys.md`), password fallback for LAN/non-HTTPS installs (`17-api-security-operations.md §3.5`), email magic links, 10 single-use emergency recovery codes, 7-day family member invitations & 24-hour out-of-band sign-in links, server-side revocable sessions (`17 §3.1`), CSRF & Origin middleware (**TD-002**), sliding-window rate limiting & 5-failure account lockout (**TD-003**), and modularized frontend UI components (`BrandLogoIcon`, `ThemeToggleButton`, `LanguageToggleButton`, `src/components/auth/*`, `src/components/settings/*`).

---

## 1. Summary of What Was Implemented

### 1.1 Database Schema (`backend/migrations/20261026000003_auth_users_sessions.sql`)
- Created [`20261026000003_auth_users_sessions.sql`](../../backend/migrations/20261026000003_auth_users_sessions.sql) defining:
  - `users` (`id`, `email`, `display_name`, `role`, `linked_person_id`, `preferred_locale`, `preferred_theme`, `password_hash`, `failed_login_attempts`, `locked_until`, `created_at`, `last_login_at`).
  - `passkey_credentials` (`id`, `user_id`, `credential_id`, `public_key_cbor`, `sign_count`, `device_name`, `transports`, `created_at`, `last_used_at`).
  - `recovery_codes` (`id`, `user_id`, `code_hash`, `used_at`, `created_at`).
  - `magic_links` (`id`, `user_id`, `email`, `token_hash`, `purpose`, `expires_at`, `used_at`, `created_at`).
  - `auth_challenges` (`id`, `challenge`, `purpose`, `user_id`, `email`, `expires_at`, `used_at`, `created_at`).
  - `sessions` (`id`, `user_id`, `token_hash`, `ip_address`, `user_agent`, `created_at`, `last_active_at`, `expires_at`, `revoked_at`) with `idx_sessions_user_id` and `idx_sessions_expires_at` (`10-core-data-model.md §4.6.1`).
  - `invitations` (`id`, `token`, `email`, `first_name`, `last_name`, `role`, `linked_person_id`, `invited_by`, `expires_at`, `accepted_at`, `revoked_at`, `created_at`) (`10-core-data-model.md §4.6.2`).

### 1.2 Backend Security Middleware, Auth Service & REST API (`backend/`)
- **CSRF & Origin Enforcement (`TD-002` Resolved) & Rate Limiting (`TD-003` Resolved) ([`backend/src/services/security.rs`](../../backend/src/services/security.rs)):**
  - `csrf_protection_middleware`: Rejects mutating requests (`POST`, `PUT`, `PATCH`, `DELETE`) lacking `X-Requested-With: genealogy` or carrying a mismatched `Origin` header with `403 Forbidden` (`csrf_failed`).
  - `RateLimiter`: Sliding-window limiter enforcing 10 req/min/IP on auth endpoints, 3 req/15min/email on magic-link requests, and 60 req/min/IP on search endpoints (`429 rate_limited`), plus 5-failure 15-minute account lockout (`429 account_locked`).
  - Password hashing via PBKDF2-HMAC-SHA256 with constant-time verification and 128-bit random salt.
- **Authentication & Session Service ([`backend/src/services/auth.rs`](../../backend/src/services/auth.rs)) & Routes ([`backend/src/routes/auth.rs`](../../backend/src/routes/auth.rs)):**
  - 24 endpoints registered under `/api/v1` with OpenAPI 3.1 (`utoipa`) schemas:
    - Setup & Registration: `POST /api/v1/auth/register/challenge`, `POST /api/v1/auth/setup`, `GET /api/v1/auth/invites/{token}`, `POST /api/v1/auth/register`.
    - Sign-In & Recovery: `POST /api/v1/auth/login/challenge`, `POST /api/v1/auth/login/verify`, `POST /api/v1/auth/password/login`, `POST /api/v1/auth/magic-link/request`, `POST /api/v1/auth/magic-link/verify`, `POST /api/v1/auth/recovery-code`, `POST /api/v1/auth/logout`.
    - Self-Service Account & Security: `GET /api/v1/auth/me`, `PATCH /api/v1/auth/me`, `PUT /api/v1/auth/password`, `GET /api/v1/auth/passkeys`, `POST /api/v1/auth/passkeys/add/challenge`, `POST /api/v1/auth/passkeys/add/verify`, `PATCH /api/v1/auth/passkeys/{id}`, `DELETE /api/v1/auth/passkeys/{id}` (blocks deleting last remaining auth method with `last_auth_method`), `POST /api/v1/auth/recovery-codes/regenerate`, `GET /api/v1/auth/sessions`, `DELETE /api/v1/auth/sessions/{id}`, `DELETE /api/v1/auth/sessions`.
    - Admin User & Invitation Management: `GET /api/v1/admin/users`, `POST /api/v1/admin/users/{id}/sign-in-link`, `DELETE /api/v1/admin/users/{id}/sessions`, `GET /api/v1/admin/invites`, `POST /api/v1/admin/invites`, `DELETE /api/v1/admin/invites/{id}`.

### 1.3 Modular Frontend Authentication, Settings Panels & Brand Identity (`frontend/`)
- **Reusable UI Buttons & Family Tree Brand Identity (`frontend/src/components/ui/`):**
  - Created [`BrandLogoIcon`](../../frontend/src/components/ui/brand-logo-icon.tsx) and [`public/favicon.svg`](../../frontend/public/favicon.svg) (organic family tree icon with canopy, roots, and 3 ancestral nodes, replacing all `GitBranch`/`GitFork` icons) and updated title typography in [`index.css`](../../frontend/src/index.css) to crisp screen sans-serif (`Plus Jakarta Sans` / `Inter`).
  - Extracted [`ThemeToggleButton`](../../frontend/src/components/ui/theme-toggle-button.tsx) and [`LanguageToggleButton`](../../frontend/src/components/ui/language-toggle-button.tsx) so [`app-shell.tsx`](../../frontend/src/components/ui/app-shell.tsx) and [`auth-layout.tsx`](../../frontend/src/components/ui/auth-layout.tsx) contain zero raw `<button>` tags.
- **Modular Authentication Cards (`frontend/src/components/auth/` & [`AuthPages.tsx`](../../frontend/src/pages/AuthPages.tsx)):**
  - [`SetupWizardCard.tsx`](../../frontend/src/components/auth/SetupWizardCard.tsx) (`/setup`), [`RecoveryCodesCard.tsx`](../../frontend/src/components/auth/RecoveryCodesCard.tsx), [`LoginCard.tsx`](../../frontend/src/components/auth/LoginCard.tsx) (`/login`), [`RegisterInviteCard.tsx`](../../frontend/src/components/auth/RegisterInviteCard.tsx) (`/register`), and [`MagicLinkVerifyCard.tsx`](../../frontend/src/components/auth/MagicLinkVerifyCard.tsx) (`/magic-link/verify`), backed by [`webauthn.ts`](../../frontend/src/lib/webauthn.ts) and [`auth-context.tsx`](../../frontend/src/lib/auth-context.tsx).
- **Modular Settings Panels (`frontend/src/components/settings/` & [`SettingsPage.tsx`](../../frontend/src/pages/SettingsPage.tsx)):**
  - Split `SettingsPage.tsx` into [`GeneralSettingsPanel.tsx`](../../frontend/src/components/settings/GeneralSettingsPanel.tsx), [`SystemDiagnosticsCard.tsx`](../../frontend/src/components/settings/SystemDiagnosticsCard.tsx), [`BackupsSettingsPanel.tsx`](../../frontend/src/components/settings/BackupsSettingsPanel.tsx), [`ProfileSettingsPanel.tsx`](../../frontend/src/components/settings/ProfileSettingsPanel.tsx), [`SecurityPasskeysPanel.tsx`](../../frontend/src/components/settings/SecurityPasskeysPanel.tsx), and [`FamilyMembersInvitesPanel.tsx`](../../frontend/src/components/settings/FamilyMembersInvitesPanel.tsx).

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | Zero formatting diffs |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (15/15 tests)** | `auth_test.rs` (4 tests), `operations_test.rs` (7 tests), `gendate_fixtures_test.rs` (2 tests), `health_test.rs` (2 tests) |
| **Frontend Unit & A11y Tests** | `npm test -- --run` | **Passed (47/47 tests)** | `auth-and-modular-ui.test.tsx` (9 tests), `settings-page.test.tsx` (5 tests), `ui-and-a11y.test.tsx` (10 tests), `parity.test.ts` (5 tests), `gendate-parser.test.ts` (18 tests) |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | `tsc --noEmit && vite build` clean |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | Rounds 1 & 2 approved; 0 blocker or major findings |
| **QA & Browser Verification** | `qa-engineer` + Playwright MCP | **PASS (0 open defects)** | All 15 DoD items verified; Round 1 caught `DEF-001`..`DEF-003` in live WebAuthn & Magic-Link flows, all resolved and verified in Round 2 |

---

## 3. Technical Debt & Tradeoffs Identified

See [`docs/technical-debt.md`](../technical-debt.md) for the full registry:
- **Resolved:** **TD-002 (CSRF & Origin Middleware Enforcement)** — Enforced on all mutating HTTP methods in [`backend/src/services/security.rs`](../../backend/src/services/security.rs).
- **Resolved:** **TD-003 (Rate Limiter Middleware)** — Enforced via sliding-window `RateLimiter` and 5-failure account lockout in [`backend/src/services/security.rs`](../../backend/src/services/security.rs).
- **No new technical debt introduced.**

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 3 — Roles (`admin` / `editor` / `viewer`) & Living-Person Privacy:** [`docs/features/02-rbac-access-control.md`](../features/02-rbac-access-control.md) (Master RBAC matrix enforcement across routes/UI, public link read-only mode, living-person privacy calculation with 110-year fallback rule, and anonymous visitor masking of living individuals).
