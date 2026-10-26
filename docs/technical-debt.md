# Technical Debt & Tradeoff Register

This document tracks technical debt, design tradeoffs, postponed items, and architectural considerations across feature implementations. It is continuously updated and referenced before starting each subsequent feature.

---

## 1. Active Technical Debt & Postponed Items

| ID | Originating Feature / Step | Description | Rationale / Tradeoff | Recommended Resolution Phase |
| :--- | :--- | :--- | :--- | :--- |
| **TD-001** | `00-base-skeleton` | **DB Retry & Auto-reconnect:** In `main.rs`, the backend attempts to connect to Postgres with a 5s acquire timeout; if DB is unreachable on cold boot, it currently logs a warning and boots in degraded mode instead of an exponential backoff loop. | Allowed dev without Postgres container running, but for production Docker Compose startup, an exponential retry loop ensures Axum doesn't start health check before DB connects. | Address during `08-deployment` / `17-security-ops` (Phase 1) |
| **TD-002** | `00-base-skeleton` | **CSRF & Origin Middleware Enforcement:** Standard header check `X-Requested-With: genealogy` is placed in client fetcher and allowed in CORS, but backend router middleware enforcing it on mutating HTTP methods (`POST`, `PATCH`, `DELETE`) is not yet wired as a tower middleware. | Minimal skeleton stage; will be strictly enforced as auth & mutation endpoints arrive. | Address during `01-authentication-passkeys` (Phase 1) |
| **TD-003** | `00-base-skeleton` | **Rate Limiter Middleware:** Rate limiting limits defined in `17-api-security-operations.md §3.3` (10/min IP, 60/min search) need an in-memory token bucket / sliding window middleware (`governor` or `tower_governor`). | Kept skeleton lightweight; needs to be integrated once auth challenge endpoints are implemented. | Address during `01-authentication-passkeys` (Phase 1) |
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
