# Architecture Decision Log (ADR)

This document records the architectural, design, and structural decisions made across the Genealogy system. Decisions are assigned permanent identifiers (`ADR-NNN`) and are never deleted. When a decision is superseded, it is marked with strikethrough and links to its replacement.

---

## Index

- [ADR-001: Single-Instance Architecture (1 Instance = 1 Family Tree)](#adr-001-single-instance-architecture-1-instance--1-family-tree)
- [ADR-002: Runtime SQLx Parameter Binding (No Compile-Time Macros)](#adr-002-runtime-sqlx-parameter-binding-no-compile-time-macros)
- [ADR-003: Session Cookies with `X-Requested-With` CSRF Header Verification](#adr-003-session-cookies-with-x-requested-with-csrf-header-verification)
- [ADR-004: Standardized Error Envelope with Machine-Readable Codes Linked to i18n](#adr-004-standardized-error-envelope-with-machine-readable-codes-linked-to-i18n)
- [ADR-005: Privacy-404 for Redacted and Living Person Access](#adr-005-privacy-404-for-redacted-and-living-person-access)
- [ADR-006: Dual Identifier Resolution (`UUID` and Alphanumeric `gid`)](#adr-006-dual-identifier-resolution-uuid-and-alphanumeric-gid)
- [ADR-007: CSS-First Design Tokens via Tailwind CSS v4 `@theme`](#adr-007-css-first-design-tokens-via-tailwind-css-v4-theme)
- [ADR-008: In-Database Advisory Locking for Sequential Upgrades & CLI Operations](#adr-008-in-database-advisory-locking-for-sequential-upgrades--cli-operations)
- [ADR-009: Strict Unauthenticated Navigation Segregation & Default Clean Slate](#adr-009-strict-unauthenticated-navigation-segregation--default-clean-slate)
- [~~ADR-010: Client-Side External LLM Extraction with Server-Side Tree Reconciliation~~](#adr-010-client-side-external-llm-extraction-with-server-side-tree-reconciliation)
- [ADR-011: Server-Side Gemini 3.8 Flash Extraction and Audio Transcription via GEMINI_API_KEY](#adr-011-server-side-gemini-38-flash-extraction-and-audio-transcription-via-gemini_api_key)
- [ADR-012: On-Demand GitHub Releases and Native LXC / Bare-Metal Deployment Pipeline](#adr-012-on-demand-github-releases-and-native-lxc--bare-metal-deployment-pipeline)
- [ADR-013: Panics Caught Gracefully via CatchPanicLayer & Dead Cookie Eviction on 401](#adr-013-panics-caught-gracefully-via-catchpaniclayer--dead-cookie-eviction-on-401)

---

### ADR-001: Single-Instance Architecture (1 Instance = 1 Family Tree)
- **Status:** Accepted
- **Context:** Many genealogy SaaS platforms use multi-tenant databases with complicated organization or tenant IDs, which risks data leaks between trees and complicates self-hosting.
- **Decision:** Every deployment represents exactly one family tree. Multi-tenancy is deliberately omitted.
- **Consequences:** Greatly simplifies database queries, permissions, and backups. Scaling out means running separate Docker Compose stacks for different families.

### ADR-002: Runtime SQLx Parameter Binding (No Compile-Time Macros)
- **Status:** Accepted
- **Context:** SQLx's `query!` and `query_as!` macros connect to a live PostgreSQL instance at compile time or require an offline query cache file (`sqlx-data.json`), which easily falls out of sync and breaks Docker and CI sandbox builds.
- **Decision:** Use runtime query binding via `sqlx::query` and `sqlx::query_as::<_, T>` with handwritten parameter bindings.
- **Consequences:** 100% reproducible offline builds (`cargo build`, `cargo test`) without external database dependencies. Slightly less compile-time schema checking, mitigated by extensive integration test suites.

### ADR-003: Session Cookies with `X-Requested-With` CSRF Header Verification
- **Status:** Accepted
- **Context:** Browser-based single-page applications require protection against Cross-Site Request Forgery (CSRF). Traditional token synchronizer patterns complicate stateless APIs.
- **Decision:** Authenticate via `HttpOnly`, `SameSite=Lax` cookies. Require all mutating HTTP requests (`POST`, `PUT`, `PATCH`, `DELETE`) to send custom header `X-Requested-With: genealogy` and verify origin matching.
- **Consequences:** Browsers prohibit cross-origin requests from setting custom headers without CORS preflight approval, preventing simple CSRF exploits while keeping the API clean.

### ADR-004: Standardized Error Envelope with Machine-Readable Codes Linked to i18n
- **Status:** Accepted
- **Context:** Returning ad-hoc string error messages leads to untranslated errors or brittle client-side string scraping.
- **Decision:** All non-2xx responses return `{"error": {"code": "...", "message": "...", "details": ...}}`. The `code` string maps directly to frontend translation keys (`errors.<code_value>`).
- **Consequences:** The frontend provides localized error messages in Russian and English with consistent fallbacks.

### ADR-005: Privacy-404 for Redacted and Living Person Access
- **Status:** Accepted
- **Context:** Returning `403 Forbidden` for a living or private individual confirms that a record exists with that identifier, creating an information leak.
- **Decision:** Return `404 Not Found` indistinguishable from a missing ID whenever an unauthorized visitor requests a private or living person.
- **Consequences:** Preserves total privacy for living relatives. Authorized users (Editor, Admin, Viewer) authenticate to inspect the profile.

### ADR-006: Dual Identifier Resolution (`UUID` and Alphanumeric `gid`)
- **Status:** Accepted
- **Context:** Database relational integrity requires UUID primary keys, but users and GEDCOM files use short, human-readable IDs like `I0001` or `F0002`.
- **Decision:** Support both `id` (UUID) and `gid` (string) across all public API routes and UI URLs.
- **Consequences:** URL paths accept `/persons/I0001` or `/persons/550e8400-e29b-41d4-a716-446655440000`.

### ADR-007: CSS-First Design Tokens via Tailwind CSS v4 `@theme`
- **Status:** Accepted
- **Context:** Tailwind CSS v4 transitions away from `tailwind.config.js` to native CSS variables and `@theme` directives in stylesheet entrypoints.
- **Decision:** Define all design tokens, palette shades, lineage colors, and radii directly in `frontend/src/index.css`.
- **Consequences:** Zero configuration overhead, instant hot-reloading with Vite, and seamless switching between light and dark modes via CSS custom properties.

### ADR-008: In-Database Advisory Locking for Sequential Upgrades & CLI Operations
- **Status:** Accepted
- **Context:** Running concurrent migrations or CLI operations (backup, restore, import) against a shared database can corrupt schemas or duplicate data.
- **Decision:** Acquire a PostgreSQL session-level advisory lock (`pg_advisory_lock(94218731)`) before performing startup migrations, restorations, or bulk imports.
- **Consequences:** Guarantees serial execution of dangerous administrative operations.

### ADR-009: Strict Unauthenticated Navigation Segregation & Default Clean Slate
- **Status:** Accepted
- **Context:** Unauthenticated visitors accessing a private family tree should not see active edit buttons, broken links to administrative sections, or partial UI chrome.
- **Decision:** Unauthenticated navigation routes strictly isolate public vs private capabilities. If the instance has public access disabled, all routes immediately redirect to `/login`.
- **Consequences:** Completely prevents unauthorized modification attempts and creates a clean presentation for public visitors.

### ~~ADR-010: Client-Side External LLM Extraction with Server-Side Tree Reconciliation~~
- **Status:** Superseded by [ADR-011](#adr-011-server-side-gemini-38-flash-extraction-and-audio-transcription-via-gemini_api_key) (client-side BYOK removed; `GeminiApiKeyModal.tsx` deleted and `genealogy_gemini_api_key` purged from `localStorage`)
- **Context:** Extracting genealogical entities from unstructured narratives using state-of-the-art LLMs (e.g. Gemini 3.8 Flash) requires an API key. Initially, client-side BYOK was used to avoid storing keys on the server.
- **Decision:** ~~Execute Gemini LLM calls client-side directly from the browser using a user-provided Google AI Studio API key kept in browser `localStorage`.~~
- **Consequences:** Superseded because requiring relatives to manage API keys in the browser hurt usability and exposed keys in client storage. Replaced by server-only `GEMINI_API_KEY` in ADR-011.

### ADR-011: Server-Side Gemini 3.8 Flash Extraction and Audio Transcription via GEMINI_API_KEY
- **Status:** Accepted
- **Context:** While client-side BYOK (ADR-010) avoided handling API keys on the server, requiring every family member to obtain and enter a Google AI Studio API key hindered usability, failed in mobile browsers without local storage setup, and prevented server-side transcription of uploaded audio files. Self-hosters prefer configuring `GEMINI_API_KEY` once in `.env` with zero client-side key exposure.
- **Decision:** Execute Gemini 3.8 Flash entity extraction and speech transcription exclusively on the backend (`backend/src/services/story_intake/gemini.rs`), configured centrally via `GEMINI_API_KEY` and `GEMINI_MODEL` (defaults to `"gemini-3.8-flash"`), sending credentials via the `x-goog-api-key` HTTP header. Expose `ai_enabled: bool` in `GET /api/v1/system/info`. Purge any legacy client-side API key (`genealogy_gemini_api_key`) in `frontend/src/api/gemini-story-extractor.ts` and retain the built-in offline NLP engine as an automatic fallback.
- **Consequences:** Zero-configuration experience for family members using the web UI; server-side audio transcription accepts `.webm`, `.mp3`, `.wav`, `.m4a`, `.ogg`; API key is kept strictly on the server in `.env` without client leakage or URL query-string exposure; instances without an API key gracefully fall back to the built-in offline NLP engine.

### ADR-012: On-Demand GitHub Releases and Native LXC / Bare-Metal Deployment Pipeline
- **Status:** Accepted
- **Context:** Container-only deployment (Docker Compose) introduces memory overhead and operational friction on minimal Linux containers (e.g. Proxmox VE LXC) or low-resource VPSs. Furthermore, compiling Docker images or cutting releases on every single commit produces release churn, fills package registries with intermediate builds, and complicates production stability. Earlier designs introduced unnecessary proxy layers (Nginx) and manual database setup steps on LXC.
- **Decision:** Establish an on-demand GitHub Releases pipeline (`.github/workflows/release.yml`) triggered strictly on tag push (`v*.*.*`) or manual `workflow_dispatch`, building cross-compiled release archives (`x86_64` and `aarch64`) with checksums (`SHA256SUMS`). Release packages bundle `genealogy-backend`, `genealogy-cli`, `frontend/` dist, `scripts/install.sh`, `deploy/lxc/genealogy.service`, and `deploy/lxc/genealogy.env.example` (`deploy/lxc/nginx.conf` removed). Complement with a single-command installer/updater (`scripts/install.sh`) targeting Debian/Ubuntu systemd environments with:
  1. Direct web listening on port 80 (Axum native serving of SPA static files via `FRONTEND_STATIC_PATH` and `/api/v1` REST endpoints) with ZERO Nginx / proxy layer.
  2. Capability `CAP_NET_BIND_SERVICE` configured in systemd unit (`AmbientCapabilities=CAP_NET_BIND_SERVICE`, `CapabilityBoundingSet=CAP_NET_BIND_SERVICE`), allowing non-root user `genealogy` to bind directly to port 80.
  3. Automated PostgreSQL installation and configuration by `scripts/install.sh` (role `genealogy`, database `genealogy`, `pg_hba.conf` local MD5/SCRAM authentication, and pre-start connectivity verification via `pg_isready` and `psql`).
  4. Release automation skill (`.agents/skills/genealogy-release/SKILL.md`) for operator release preparation.
- **Consequences:** Provides a high-performance native deployment path with ~30–50MB RAM idle usage, completely eliminates intermediate Nginx/proxy configuration complexity, preserves database and media files across in-place atomic updates, guarantees automatic schema migration execution on service start, and prevents untagged/unstable builds from reaching production instances.

### ADR-013: Panics Caught Gracefully via CatchPanicLayer & Dead Cookie Eviction on 401
- **Status:** Accepted
- **Context:** Uncaught handler panics in Axum drop underlying TCP sockets abruptly, causing upstream reverse proxies (or browser HTTP engines) to emit generic 502 Bad Gateway errors without diagnostic logs or JSON error bodies. Concurrently, clients with expired or corrupted session cookies faced endless 401 loops because 401 responses never evicted dead cookies, and public-mode trees rejected visitors bearing stale cookies instead of granting public read access.
- **Decision:**
  1. Register `tower_http::catch_panic::CatchPanicLayer::custom(handle_panic)` as the outermost middleware in `backend/src/routes/mod.rs` to intercept panics, capture backtraces in structured tracing logs, and serialize a standard 500 `internal_error` JSON response envelope.
  2. In `backend/src/routes/auth/passkeys_sessions.rs` (`get_current_user`), return a direct `axum::response::Response` on `NotAuthenticated` and `SessionExpired` to attach an eviction `Set-Cookie: session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax` (including `; Secure` when behind an HTTPS proxy or configured with HTTPS).
  3. In `backend/src/services/rbac.rs` (`PublicOrAuth`), treat invalid, expired, or deactivated sessions as a graceful fallback to `AnonymousPublic` if `enable_public_access = true`, returning 401 only when public access is strictly disabled.
  4. In `frontend/src/api/core.ts` and `frontend/src/lib/auth-context.tsx`, defensively sanitize token strings (`"null"`, `"undefined"`, `""` normalized to `null`) and purge tokens upon encountering 401 / `not_authenticated` / `session_expired` responses.
- **Consequences:** Eliminates 502 Bad Gateway socket drops during edge-case panics, prevents client lockouts from poisoned cookies/tokens, and guarantees frictionless public browsing even when users retain expired credentials.

_Last curated: 2026-11-09 (CatchPanicLayer 500 handling, dead cookie eviction on 401, PublicOrAuth fallback, defensive token sanitization)_

