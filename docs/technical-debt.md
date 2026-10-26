# Technical Debt & Tradeoff Register

This document tracks technical debt, design tradeoffs, postponed items, and architectural considerations across feature implementations. It is continuously updated and referenced before starting each subsequent feature.

---

## 1. Active Technical Debt & Postponed Items

| ID | Originating Feature / Step | Description | Rationale / Tradeoff | Recommended Resolution Phase |
| :--- | :--- | :--- | :--- | :--- |
| ~~**TD-001**~~ | `00-base-skeleton` | **DB Retry & Auto-reconnect (RESOLVED in Phase 1.1):** Implemented exponential backoff retry loop (`500ms` -> `5s`, up to 8 attempts when `DATABASE_REQUIRED=true`), Postgres advisory lock (`pg_advisory_lock`), and automatic `pre_upgrade` backup in `backend/src/services/db_init.rs`. | Resolved in `08-deployment-single-instance`; standalone dev/test fallback preserved when `DATABASE_REQUIRED=false`. | **Resolved** (Phase 1.1) |
| ~~**TD-002**~~ | `00-base-skeleton` | **CSRF & Origin Middleware Enforcement (RESOLVED in Phase 1.2):** Implemented `csrf_protection_middleware` in `backend/src/services/security.rs` enforcing `X-Requested-With: genealogy` and `Origin == APP_ORIGIN` on all mutating HTTP methods (`POST`, `PUT`, `PATCH`, `DELETE`). | Resolved in `01-authentication-passkeys`. | **Resolved** (Phase 1.2) |
| ~~**TD-003**~~ | `00-base-skeleton` | **Rate Limiter Middleware (RESOLVED in Phase 1.2):** Implemented sliding-window `RateLimiter` in `backend/src/services/security.rs` enforcing 10 req/min/IP on auth endpoints, 3 req/15min/email on magic link requests, 60 req/min/IP on anonymous search, and 5-failure 15-min account lockout. | Resolved in `01-authentication-passkeys`. | **Resolved** (Phase 1.2) |
| **TD-004** | `00-base-skeleton` | **Client-side OpenAPI code generation:** The backend exposes `/api/v1/openapi.json`, but the TypeScript client currently uses handwritten fetch functions rather than auto-generated OpenAPI types (`openapi-typescript` / `kubb`). | Rapid prototyping of initial skeleton; setup auto-generation script as schema expands. | Address during `10-core-data-model` (Phase 1) |
| **TD-005** | `00-base-skeleton` | **SQLx Offline Mode (`sqlx-data.json`):** Using `sqlx::migrate!` macro without a pre-generated `sqlx-data.json` or live database during offline `cargo build --release` in isolated Docker builds could fail if queries with `query!` are added later. | Standard migrations work at runtime; will add `cargo sqlx prepare` workflow when compile-time verified queries are added. | Address in `10-core-data-model` (Phase 1) |
| **TD-006** | `00-design-system` | **Offline Sandbox Package Cache & Self-Hosted Font Bundles:** In the offline sandboxed build environment, `class-variance-authority`, `zod`, and `vitest` were installed from the local npm cache, while accessible UI primitives, virtualized `DataTable`, form controller, and SSR accessibility checks were implemented directly in React 19 + TypeScript without requiring external un-cached npm tarballs (`@fontsource/*`, `vitest-axe`). | Ensures zero-network reproducible builds and tests (`npm test` & `npm run build` pass in <1.5s) while satisfying all keyboard/ARIA, 50k row virtualization, and `zod` form validation requirements. | Optional: bundle `@fontsource/*` WOFF2 assets in Docker image when network access is enabled (Phase 1) |

---

## 2. Architectural Decisions & Tradeoffs Made

- **Dual-mode Backend Boot (Standalone & DB-connected):**
  - *Decision:* If Postgres is unavailable (e.g. running `cargo run` locally for fast API testing without docker), the backend boots and serves health/docs rather than immediately panicking.
  - *Tradeoff:* Healthcheck returns `degraded` or `standalone_ready` with 200/503 depending on mode.
- **Tailwind CSS v4 Inline `@theme`:**
  - *Decision:* Configured with `@tailwindcss/vite` without legacy `tailwind.config.js`. Uses modern CSS variables for dark mode and semantic tokens.
  - *Tradeoff:* Modern CSS syntax requires recent browser engine (2023+), perfectly matching modern web design requirements.
- **Nginx Reverse Proxy & X-Accel-Redirect:**
  - *Decision:* Frontend container acts as external reverse proxy (`:80`), routing `/api/` to backend (`:3000`) and handling `/internal-media/`.
  - *Tradeoff:* Requires both frontend and backend on Docker bridge network, keeping backend off public host ports.
