# Implementation Report: Phase 1.1 — Deployment & Single-Instance Operations

**Date:** 2026-10-26  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 1 of [`docs/roadmap.md`](../roadmap.md) — single-instance Docker Compose deployment ([`08-deployment-single-instance.md`](../features/08-deployment-single-instance.md)), non-root container hardening, 32-variable environment configuration & precedence ([`10-core-data-model.md §7`](../features/10-core-data-model.md)), database startup exponential retry loop (**TD-001**), advisory-locked automatic migrations with pre-upgrade backup ([`17-api-security-operations.md §6.2`](../features/17-api-security-operations.md)), `genealogy-cli` disaster recovery binary, nightly backup scheduler & retention pruning, operations/settings REST API, and the two-column `/settings` Administration UI ([`07-admin-settings-page.md`](../pages/07-admin-settings-page.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Container Security & Single-Instance Deployment (`docker-compose.yml`, `Dockerfile`)
- **Non-Root Hardening:**
  - [`backend/Dockerfile`](../../backend/Dockerfile): Builds both `genealogy-backend` and `genealogy-cli`, symlinks `/usr/local/bin/genealogy-cli`, installs `postgresql-client`, creates unprivileged `appuser:appgroup` (`10001:10001`), sets ownership on `/app/data/media` and `/app/data/backups`, and runs as `USER 10001:10001`.
  - [`frontend/Dockerfile`](../../frontend/Dockerfile) & [`frontend/nginx.conf`](../../frontend/nginx.conf): Creates unprivileged `appuser:appgroup` (`10001:10001`), configures `/tmp/nginx.pid`, grants `cap_net_bind_service=+ep` on `/usr/sbin/nginx`, adds security headers (`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`, `Referrer-Policy: same-origin`, `Permissions-Policy`), and runs as `USER 10001:10001`.

### 1.2 Backend Operations, Configuration, CLI & Scheduler (`backend/`)
- **Exhaustive 32-Variable Configuration ([`backend/src/config.rs`](../../backend/src/config.rs)):**
  - Parses all 32 settings from `08-deployment-single-instance.md §3` with spec defaults, `from_env()`, deterministic `from_map()`, and `is_password_auth_effectively_enabled()` (`"auto"` → enabled when non-HTTPS or SMTP disabled per `17 §3.5`).
  - Implements configuration precedence (`10 §7`) in [`backend/src/state.rs`](../../backend/src/state.rs): UI-saved values in `instance_settings` override `.env` defaults unless `SETTINGS_LOCKED_FROM_ENV=true` (which blocks mutations with `403 settings_locked_from_env`).
- **Database Startup Retry (`TD-001` Resolved) & Advisory-Locked Migrations ([`backend/src/services/db_init.rs`](../../backend/src/services/db_init.rs)):**
  - Exponential backoff retry schedule (`500ms`, `1s`, `2s`, `4s`, `5s`×3) when `database_required` is active in production, while keeping instant standalone fallback for offline tests.
  - Runs SQLx migrations inside Postgres advisory lock `pg_advisory_lock(5_135_656_515_814_555_463)` and automatically creates a `pre_upgrade` backup archive whenever pending migrations are detected on an already-initialized database (`17 §6.2`).
- **Operations & Settings Migration ([`backend/migrations/20261026000002_operations_and_settings.sql`](../../backend/migrations/20261026000002_operations_and_settings.sql)):**
  - Creates `instance_settings`, `jobs` (`idx_jobs_status_created_at`), and `emergency_admin_links` (`idx_emergency_admin_links_token`).
- **Backup Engine, Scheduler & `genealogy-cli` ([`backend/src/services/backup.rs`](../../backend/src/services/backup.rs), [`scheduler.rs`](../../backend/src/services/scheduler.rs), [`bin/cli.rs`](../../backend/src/bin/cli.rs)):**
  - Generates `backup-[TIMESTAMP].tar.gz` archives containing `database.dump` (`pg_dump -Fc` or logical JSON snapshot fallback), `media/originals/`, optional `tree.gdz`, and `manifest.json` with SHA-256 digests (`sha2::Sha256`).
  - `verify_and_restore` validates `manifest.json`, schema compatibility (`incompatible_schema`), and every file's SHA-256 checksum (`checksum_mismatch`) before restoring data.
  - Background scheduler runs nightly backups and prunes archives older than `BACKUP_RETENTION_DAYS`.
  - `genealogy-cli` supports `backup [--include-gedcom]`, `restore <archive.tar.gz>`, `create-admin-link <email>`, `regenerate-thumbnails`, and `export-gedcom <output.gdz>`.
- **REST API Endpoints ([`backend/src/routes/system.rs`](../../backend/src/routes/system.rs)):**
  - `GET /api/v1/health`, `GET /api/v1/system/info`, `GET /api/v1/admin/system`, `GET /api/v1/admin/settings`, `PATCH /api/v1/admin/settings`, `GET /api/v1/admin/backups`, `POST /api/v1/admin/backups`, and `GET /api/v1/admin/backups/{filename}` (with path-traversal protection and OpenAPI 3.1 schemas).

### 1.3 Frontend Administration & Settings UI (`frontend/`)
- **Typed API Client ([`frontend/src/api/client.ts`](../../frontend/src/api/client.ts)):**
  - Added `fetchSystemInfo`, `fetchAdminSystem`, `fetchAdminSettings`, `updateAdminSettings`, `fetchBackups`, `triggerBackup`, and `getBackupDownloadUrl`.
- **Two-Column Settings Page ([`frontend/src/pages/SettingsPage.tsx`](../../frontend/src/pages/SettingsPage.tsx)):**
  - Built exclusively with Phase 0 design-system components (`PageHeader`, `SideNav`, `Card`, `FormSection`, `FormField`, `TextInput`, `Select`, `Switch`, `Button`, `CopyButton`, `Badge`, `DescriptionList`, `Alert`, `EmptyState`, `Divider`, `useToast`).
  - **General & Archive Tab (`/settings`):** Edits `tree_title`, `default_locale`, `enable_public_access`, and `living_person_age_threshold`, syncs the live header title in [`App.tsx`](../../frontend/src/App.tsx) / [`app-shell.tsx`](../../frontend/src/components/ui/app-shell.tsx), renders read-only mode when `settings_locked_from_env === true`, and displays the plain-language **System** card (`17 §6.1`, `00 §3.4`).
  - **Backups & Snapshots Tab (`/settings/backups`):** One-click `Download Full Archive (Database & Media)` button, automated nightly backup schedule card, available backup archives list with download buttons, and CLI disaster recovery reference with `CopyButton`.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Backend Format** | `cargo fmt --all -- --check` | **Passed** | Zero formatting diffs |
| **Backend Lints** | `cargo clippy --all-targets -- -D warnings` | **Passed (0 warnings)** | Clean compilation |
| **Backend Tests** | `cargo test` | **Passed (11/11 tests)** | 32-var config, DB retry/pre-upgrade backup, CLI subcommands, SHA-256 restore, settings & backup APIs, Dockerfile non-root checks |
| **Frontend Unit & A11y Tests** | `npm test -- --run` | **Passed (38/38 tests)** | `settings-page.test.tsx` (5 tests), `ui-and-a11y.test.tsx` (10 tests), `parity.test.ts` (5 tests), `gendate-parser.test.ts` (18 tests) |
| **Frontend Build & Types** | `npm run build` | **Passed (0 warnings)** | `tsc --noEmit && vite build` clean in 895ms |
| **Architecture Review** | `architect-reviewer` (Mode 2) | **APPROVE** | 0 blocker or major findings |
| **QA & Browser Verification** | `qa-analyst` + Playwright MCP | **PASS (0 defects)** | All 8 DoD items and 4 live full-stack browser scenarios verified |

---

## 3. Technical Debt & Tradeoffs Identified

See [`docs/technical-debt.md`](../technical-debt.md) for the full registry:
- **Resolved:** **TD-001 (DB Retry & Auto-reconnect)** — Resolved via exponential backoff connection loop and Postgres advisory-locked migration runner in [`backend/src/services/db_init.rs`](../../backend/src/services/db_init.rs).
- **No new technical debt introduced.**

---

## 4. Next Steps
According to [`docs/roadmap.md`](../roadmap.md):
- **Phase 1, Item 2 — Authentication, Passkeys & Sessions:** [`docs/features/01-authentication-passkeys.md`](../features/01-authentication-passkeys.md) and [`docs/features/17-api-security-operations.md §3`](../features/17-api-security-operations.md) (`/setup` first-run wizard, invitations, WebAuthn passkeys + password fallback for LAN installs, recovery codes, server-side sessions, CSRF middleware resolving **TD-002**, and rate limiting resolving **TD-003**).
