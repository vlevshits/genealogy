# Active Context

Current operational status, recent milestones, and immediate roadmap priorities.

---

## 1. Current Phase & Milestone

- **Current Roadmap Phase:** Milestone 3.2 (Story Intake & Gemini 3.8 Flash Integration) + Full Codebase Modularization & Hardening + Production LXC / Bare-Metal Deployment Pipeline complete. Preparing for Milestone 3.3 (Conversational Adjustments & Advanced MCP Operations).
- **Recent Completed Milestones:**
  - **Production LXC / Bare-Metal Deployment Pipeline & Release Automation (`08-deployment-single-instance`):** Unified single-command installer and atomic updater (`scripts/install.sh`) for Debian 11/12 and Ubuntu 22.04/24.04 (`x86_64` and `aarch64`); direct web listening on port 80 (Axum native serving of SPA static files via `FRONTEND_STATIC_PATH` and `/api/v1` endpoints) with ZERO Nginx / proxy layer; hardened systemd unit (`deploy/lxc/genealogy.service`) configured with `CAP_NET_BIND_SERVICE` allowing non-root `genealogy` user to bind directly to port 80; automated PostgreSQL setup (role `genealogy`, database `genealogy`, `pg_hba.conf` local auth, connection test); on-demand GitHub Actions release pipeline (`.github/workflows/release.yml`) producing cross-compiled release archives with `SHA256SUMS` (packaging backend, cli, frontend dist, systemd unit, env template, and install script; `deploy/lxc/nginx.conf` removed); release automation skill (`.agents/skills/genealogy-release/SKILL.md`).
  - **Codebase Modularization, Security Hardening & UX Polish (`19-ai-family-story-intake-and-mcp`, `02-rbac-access-control`, `xx-design-system`):** Decomposed all backend and frontend source files to strictly ≤ 800 lines (zero files > 800 lines across `backend/src/` and `frontend/src/`); split `ai_mcp.rs` into `ai_stories.rs` and `mcp.rs`; split `openapi-types.ts` into `openapi-types.ts` and `openapi-types-domain.ts`; configured Vite `manualChunks` code-splitting; enforced living-person event and family redaction across MCP read tools (`redact_event_for_tree_view`, `redact_family_for_mcp`); moved Gemini API key to `x-goog-api-key` HTTP header; purged client-side BYOK keys; fixed microphone `MediaStream` and `SpeechRecognition` unmount cleanup; and resolved `UX-119`–`UX-126`.
  - **Milestone 3.2 AI Story Intake & Gemini 3.8 Flash Integration (`19-ai-family-story-intake-and-mcp`):** Server-side Google Gemini 3.8 Flash extraction and audio transcription configured via `GEMINI_API_KEY` in `.env`, `ai_enabled` flag in `GET /api/v1/system/info`, server-side tree reconciliation, cross-browser audio recorder, and engine toggle (Gemini 3.8 Flash vs Built-in Offline).
  - **Phase 3 Printable PDF Export (`05-printable-tree-pdf`):** High-resolution poster exports and multi-page tiled PDFs with cut marks, preview canvas, and full EN/RU studio controls.
  - **Phase 2.8 Associated People & Client Face Detection (`06-media-face-tagging`):** Linked people in photo sidebars, client-side face suggestion rectangles, and manual coordinate tagging.

---

## 2. Recent Key Changes

1. **LXC / Bare-Metal Single-Script Installer & In-Place Updater (`scripts/install.sh`):** Root shell script supporting Debian 11/12 and Ubuntu 22.04/24.04 on `x86_64` and `aarch64`. Automates system packages, local PostgreSQL provisioning with auto-generated credentials, role/db `genealogy`, `pg_hba.conf` local authentication, and connectivity verification (`pg_isready` & `psql`). Configures dedicated system user `genealogy`, directory layout (`/opt/genealogy`, `/var/lib/genealogy`, `/var/www/genealogy`, `/etc/genealogy`), hardened systemd service (`deploy/lxc/genealogy.service`) with `CAP_NET_BIND_SERVICE` allowing non-root user `genealogy` to bind directly to port 80, Axum native static file and `/api/v1` serving on port 80 with ZERO Nginx / proxy layer, and atomic directory swap update workflow.
2. **On-Demand GitHub Releases Pipeline (`.github/workflows/release.yml`, `ADR-012`):** Releases trigger strictly on annotated tag push (`v*.*.*`) or manual `workflow_dispatch`, never on regular commits to `main`. Builds frontend SPA, cross-compiles native `x86_64` and `aarch64` binaries, bundles unified distribution archives (`genealogy-vX.Y.Z-linux-<arch>.tar.gz` containing backend, cli, frontend dist, systemd unit, env template, and install script; `deploy/lxc/nginx.conf` removed from packaging), and computes `SHA256SUMS`.
3. **Release Automation Skill (`.agents/skills/genealogy-release/SKILL.md`):** Custom agent skill guiding pre-flight test verification (cargo test, npm test, i18n parity, build, memory-bank check), semantic version bumping across `frontend/package.json` and `backend/Cargo.toml`, and annotated git tag creation.
4. **Native Axum Static File Serving (`backend/src/config.rs`, `backend/src/routes/mod.rs`):** Added `frontend_static_path` (`FRONTEND_STATIC_PATH`) enabling Axum to serve pre-built SPA static assets directly with `ServeDir` and fallback to `index.html`, facilitating standalone or alternative reverse proxy setups.
5. **Server-Side Gemini 3.8 Flash Integration & Header Security (`backend/src/services/story_intake/gemini.rs`):** Implemented `transcribe_audio_with_gemini` and `extract_story_with_gemini` using `GEMINI_API_KEY` and `GEMINI_MODEL`, passing the key via the `x-goog-api-key` HTTP header (never in URL query parameters) and truncating upstream error logs to 200 characters.
6. **Centralized Server AI Config (Zero Client BYOK):** Configured `gemini_api_key` and `gemini_model` in `backend/src/config.rs` and exposed `ai_enabled: bool` on `GET /api/v1/system/info` (`backend/src/routes/system.rs`). `frontend/src/api/gemini-story-extractor.ts` purges any legacy `genealogy_gemini_api_key` from `localStorage` and stores only the engine preference (`'gemini' | 'builtin'`).
7. **Multipart & JSON Audio Transcription (`backend/src/routes/rbac/ai_stories.rs`):** `POST /api/v1/ai/transcribe` supports both `multipart/form-data` uploads and JSON `audio_base64` payloads for `.webm`, `.mp3`, `.wav`, `.m4a`, and `.ogg` formats; `/api/v1/ai/drafts/{id}/reconcile` is strictly `POST` (protected by CSRF middleware).
8. **MCP Privacy Redaction & PAT Hardening (`backend/src/services/privacy/masking.rs`, `backend/src/services/mcp/rpc_read.rs`, `backend/src/routes/rbac/mcp.rs`):** Added `redact_event_for_tree_view` and `redact_family_for_mcp` so `list_events_timeline`, `get_family_immediate`, and `get_ancestors_descendants` enforce living-person and private-record redaction for `Viewer` and public MCP callers.
9. **100% Compliance with the 800-Line Ceiling & Bundle Splitting:** Decomposed all backend (`backend/src/` max 795 lines) and frontend (`frontend/src/` max 744 lines) modules, and configured Rollup `manualChunks` in `frontend/vite.config.ts` (`vendor`, `vendor-icons`, `i18n-locales`, `domain-tree`, `domain-print`, `domain-stories`, `domain-quality`).
10. **Frontend Resilience & UX Polish (`UX-119`–`UX-126`):** Added `MediaStream` track and `SpeechRecognition` unmount cleanup in `AudioRecorderWidget.tsx` (`GOT-011`), rethrew HTTP `ApiError` (`status > 0`) in `stories-mcp-api.ts` (`GOT-012`), preserved `AbortError` in `frontend/src/api/core.ts`, and aligned Tailwind v4 `@theme` surface/border tokens in `frontend/src/index.css`.

---

## 3. Next Steps & Open Focus Areas

1. **Conversational Adjustments & Multimodal MCP (Milestone 3.3):**
   - Enhanced multi-turn interactive clarification workflows for unresolved ambiguity notes in story drafts.
   - Advanced MCP batch operations and external agent tooling.
2. **Continuous Quality & Memory Bank Upkeep:**
   - Maintain 0 files > 800 lines across `backend/src/` and `frontend/src/`.
   - Run `scripts/check-memory-bank.sh` on every Close/Remember step.

---

## 4. Open Questions & Invariants

- **Deliberate Release Cadence:** Commits to `main` do not generate production releases. Production releases are cut deliberately via version tags (`v*.*.*`) or manual dispatch using the `genealogy-release` skill.
- **Draft Sandbox Isolation:** AI-generated records are never committed directly to canonical tables (`persons`, `families`) without human verification and review.
- **Server-Only AI Credentials:** `GEMINI_API_KEY` is configured exclusively on the server in `.env` (or admin instance settings); no API keys are entered, stored, or exposed in the browser. When unconfigured, the system falls back to the built-in offline NLP parser.
- **MCP Security & Privacy Scoping:** Personal Access Tokens (PAT) for external AI agents adhere strictly to RBAC scope restrictions (`read:tree`, `write:draft`, `write:direct`, `admin:maintenance`) and apply `AnonymousPublic` living-person redaction to `Viewer`-tier MCP tokens.

_Last curated: 2026-11-08 (Zero-proxy LXC port 80 direct binding, CAP_NET_BIND_SERVICE, automated PostgreSQL setup)_
