# AGENTS.md — Genealogy (self-hosted family tree)

Guidance for AI coding agents (and humans) working in this repository.
Detailed, always-current implementation knowledge lives in the
**memory bank** — start there.

## 1. Start here: the memory bank

`docs/memory-bank/` is the project's long-term memory. You have no memory of
previous sessions; the memory bank is how you recall what exists, where it
lives, why it is that way, and what to avoid.

1. [`docs/memory-bank/README.md`](docs/memory-bank/README.md) — protocol & index.
2. [`active-context.md`](docs/memory-bank/active-context.md) — current phase, recent changes, next steps.
3. [`glossary.md`](docs/memory-bank/glossary.md) — shared EN/RU vocabulary and code identifiers.
4. The relevant [`features/NN-*.md`](docs/memory-bank/features/) — code map, flows, invariants, gotchas per feature.
5. Before coding: [`conventions-and-patterns.md`](docs/memory-bank/conventions-and-patterns.md),
   [`gotchas.md`](docs/memory-bank/gotchas.md), [`decisions.md`](docs/memory-bank/decisions.md);
   for orientation: [`architecture.md`](docs/memory-bank/architecture.md),
   [`codebase-map.md`](docs/memory-bank/codebase-map.md).

Protocol (full text: [`.agents/rules/memory-bank.md`](.agents/rules/memory-bank.md)):
**recall** before acting → end every subagent report with **`## Memory Notes`**
→ only **`memory-bank-curator`** writes to the memory bank, at the end of every
feature/phase and after significant fixes/refactors.

## 2. Project at a glance

- **Product:** private, invite-only family tree for non-technical relatives;
  **1 instance = 1 family tree**; private by default (living-person
  protection); EN + RU from day 0 (Slavic naming, patronymics).
- **Stack:** Rust (Axum 0.8, Tokio, SQLx 0.8 runtime queries, utoipa OpenAPI)
  + PostgreSQL 16 (`pg_trgm`, `unaccent`, `pgcrypto`); React 19 + TypeScript
  (strict) + Vite 6 + Tailwind CSS v4; Docker Compose with 3 containers
  (`frontend` Nginx reverse proxy → `backend` → `postgres`).
- **Specs:** [`docs/features/README.md`](docs/features/README.md) (index),
  [`docs/pages/`](docs/pages/), [`docs/design-system/`](docs/design-system/);
  plan in [`docs/roadmap.md`](docs/roadmap.md); debt in
  [`docs/technical-debt.md`](docs/technical-debt.md); phase reports in
  [`docs/implementation/`](docs/implementation/); UX reports in
  [`docs/ux-reviews/`](docs/ux-reviews/).

| Path | Contents |
| :--- | :--- |
| `backend/src/routes/` | Thin Axum handlers (HTTP + RBAC adapters) |
| `backend/src/services/` | Business logic (auth, privacy, genealogy, history, quality, gedcom, pdf, mcp, …) |
| `backend/src/models/` | Domain types (e.g. genealogical dates) |
| `backend/migrations/` | Forward-only, timestamp-prefixed SQLx migrations |
| `backend/tests/` | Rust integration tests |
| `frontend/src/pages/`, `components/<domain>/` | Pages and domain components; DS primitives in `components/ui/` |
| `frontend/src/api/client.ts` | Typed API client (`ApiError`, `X-Requested-With: genealogy`) |
| `frontend/src/i18n/{en,ru}.json` | Translations (key parity enforced by tests) |
| `frontend/tests/` | Vitest unit/component tests |
| `frontend/e2e/` | Playwright end-to-end tests & Page Object Models |
| `.agents/agents/`, `.agents/rules/` | Subagent roles and always-on rules |

## 3. Commands

| Task | Command (run from) |
| :--- | :--- |
| Backend format / lint / test | `cargo fmt --all -- --check` · `cargo clippy --all-targets -- -D warnings` · `cargo test` (`backend/`) |
| Frontend build (tsc + vite) / tests | `npm run build` · `npm test` · `npm run test:i18n` · `npm run test:e2e` (`frontend/`) |
| Frontend dev server | `npm run dev` → http://localhost:5173 (`frontend/`) |
| Full stack | `docker compose up -d --build` → http://localhost (repo root) |
| Admin CLI | `docker compose exec backend ./genealogy-cli backup \| restore <file> \| create-admin-link <email>` |
| Memory bank link check | `scripts/check-memory-bank.sh` (repo root) |

## 4. Working agreements

- **Delegate by role** — see [`.agents/rules/subagent-delegation.md`](.agents/rules/subagent-delegation.md)
  for the role table and the standard workflow:
  *Recall → Specify → Design → Implement → Review → E2E Tests → Verify → UX Review → Close → Remember*.
- **Rules (always on):** [`memory-bank.md`](.agents/rules/memory-bank.md),
  [`subagent-delegation.md`](.agents/rules/subagent-delegation.md),
  [`ui-and-i18n-conventions.md`](.agents/rules/ui-and-i18n-conventions.md),
  [`modular-architecture-and-file-size.md`](.agents/rules/modular-architecture-and-file-size.md).
- **Non-negotiables:** `/api/v1` + `snake_case` + standard error envelope with
  stable `code` (= frontend i18n key); RBAC on every path and privacy-404 for
  hidden records; CSRF header/Origin on mutations; no secrets/PII in logs;
  forward-only migrations; design tokens only (no raw hex); every UI string in
  both `en.json` and `ru.json` with no icons/emojis in strings; design-system
  components instead of raw `<button>/<input>/<select>`; source files ≤ 800 lines.
- **Scope discipline:** build only what the current roadmap phase asks for;
  record shortcuts in `docs/technical-debt.md`; ask when specs are ambiguous.

