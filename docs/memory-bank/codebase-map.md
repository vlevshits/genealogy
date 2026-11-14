# Codebase Map

A complete directory and module responsibility map of the Genealogy repository.

---

## 1. Backend (`backend/`)

### 1.1 Source Code (`backend/src/`)

| File / Directory | Lines | Responsibility |
| :--- | :--- | :--- |
| `main.rs` | ~70 | Entrypoint: initializes tracing, loads config, runs DB migrations, builds router, binds TCP listener. |
| `lib.rs` | ~25 | Crate root: exports `config`, `error`, `models`, `routes`, `services`, and `state`. |
| `config.rs` | 268 | Configuration struct `AppConfig`: environment variables (`GEMINI_API_KEY`, `GEMINI_MODEL`, `FRONTEND_STATIC_PATH`, etc.), defaults, URL parsing, validation. |
| `error.rs` | 260 | Standard error enum `AppError` and JSON response envelope with machine-readable `code`. |
| `state.rs` | 343 | Shared runtime state `AppState`: database pool, config, rate limiter, challenge stores. |
| `bin/cli.rs` | ~250 | Standalone CLI (`genealogy-cli`): backup, restore, and emergency `create-admin-link`. |
| `models/` | | **Domain Models** |
| `models/mod.rs` | ~15 | Module declarations and exports. |
| `models/gendate.rs` | 517 | Genealogical date parser/serializer (`GenDate`): exact, approximate, before/after, ranges, dual years. |
| `routes/` | | **HTTP Handlers & Adapters** |
| `routes/mod.rs` | 428 | Router creation (`create_router`), OpenAPI spec definition (`ApiDoc`), panic handling layer (`CatchPanicLayer`, `handle_panic`), middleware assembly, native static SPA fallback (`ServeDir` / `ServeFile`). |
| `routes/auth/` | ~600 | Modular authentication routes: passkey registration/login, magic links, passwords, sessions (with dead cookie eviction on 401 in `passkeys_sessions.rs`), recovery codes. |
| `routes/rbac/` | ~4,100 | Modular domain routes: tree view (`tree_view.rs`), person/family/event CRUD (`person_profile.rs`, `person_mutations.rs`, `person_delete_families.rs`, `events.rs`, `quick_add.rs`), places (`places.rs`), media (`media_list_create.rs`, `media_tags_links.rs`, `notes_media_files.rs`), search/directory (`search.rs`, `people_directory.rs`), history & quality (`history_quality.rs`, `quality_events.rs`), GEDCOM/PDF (`gedcom_pdf.rs`), dashboard/kinship (`dashboard_kinship.rs`), onboarding (`onboarding_tree.rs`), AI story intake (`ai_stories.rs`), and MCP (`mcp.rs`). |
| `routes/system.rs` | 383 | Health check, system information (`ai_enabled` flag), and database readiness endpoints. |
| `routes/health.rs` | ~40 | Minimal liveness and readiness ping handler. |
| `services/` | | **Business Logic & Workflows** |
| `services/auth/` | ~1,200 | Modularized authentication: passkeys, magic links, session lifecycle, password lockout. |
| `services/backup.rs` | 664 | Database dump (`pg_dump` wrapper), media archive bundling, archive unpacking, restore pipeline. |
| `services/dashboard.rs` | 744 | Dashboard aggregation, home person metrics, upcoming birthdays and anniversaries. |
| `services/db_init.rs` | ~150 | Database startup connectivity check, exponential retry loop, advisory lock acquisition. |
| `services/email.rs` | 630 | SMTP mailer, HTML email templates (invitations, magic links, account recovery). |
| `services/gedcom/` | ~1,200 | Modularized GEDCOM parser, exporter, record linkage, and family relations; `importer.rs` preserves PATs and `ai_config` across resets. |
| `services/genealogy/` | ~1,100 | Modularized traversal, pedigree trees, descendant trees, and multi-trace kinship paths. |
| `services/history/` | 2,881 | Modularized change log recording (`feed.rs`, `types.rs`, `mod.rs`), read-time label/GID/diff enrichment (`enrich.rs`, `enrich_index.rs`, `enrich_diff.rs`, `enrich_format.rs`), snapshots (`snapshots.rs`), and undo engine (`undo.rs`). |
| `services/mcp/` | 2,775 | Modularized MCP server (6 files): `catalog.rs`, `db_tokens.rs` (PAT file/DB persistence, expiration, seed bootstrap), `rpc_read.rs`, `tools_mutation.rs` (incl. `merge_duplicate_individuals`, `list_duplicate_events`, `merge_duplicate_events`, `undo_transaction`), `types_tokens.rs`. |
| `services/media_bundle.rs` | 433 | Batch ZIP upload extraction, checksum validation, automatic reconciliation with GEDCOM stubs. |
| `services/pdf/` | ~1,500 | Modularized printable tree PDF engine: layout planning (`layout/`), vector rendering (`render/`), and themes. |
| `services/privacy/` | 1,677 | Modularized living-person rules (`living.rs`), masking (`masking.rs`), DTOs (`dto.rs`), and store persistence (`store.rs` → `persist_to_dir` writing `live-store.json` + `mcp-tokens.json`). |
| `services/quality/` | 7,687 | Modularized validation checks (`warnings.rs`), duplicate candidate scoring (`candidates.rs`, `scoring.rs`), duplicate person merge (`merge.rs`), duplicate event detection & merge (`events_duplicates*.rs`), place hierarchy repair (`places_*.rs`). |
| `services/rbac.rs` | 248 | Role verification, permission policy checks (Admin, Editor, Viewer, Public), `PublicOrAuth` extractor with graceful fallback. |
| `services/scheduler.rs` | ~200 | Periodic background task runner (daily snapshot, automated backup, cleanup). |
| `services/security/` | ~500 | Modularized rate limiter, CSRF/Origin enforcement, RequestContext middleware. |
| `services/story_intake/` | ~1,600 | Modularized multimodal story intake: `gemini.rs` (server-side transcription & entity extraction), `extraction.rs`, `nlp_clauses.rs`, `nlp_entities.rs`, `reconciliation.rs`, `commit.rs`, `types_api.rs`. |

### 1.2 Database Migrations (`backend/migrations/`)

| Migration File | Responsibility |
| :--- | :--- |
| `20261026000001_init_system.sql` | Postgres extensions (`pg_trgm`, `unaccent`, `pgcrypto`) and system metadata tables. |
| `20261026000002_operations_and_settings.sql` | System settings, configuration keys, and scheduled job registry. |
| `20261026000003_auth_users_sessions.sql` | Users, roles, sessions, passkeys (WebAuthn credentials), magic links, recovery codes. |
| `20261026000004_rbac_and_privacy.sql` | Privacy preferences, living person flags, invitation tokens. |
| `20261026000005_core_genealogy_model.sql` | Primary genealogy schema: `persons`, `names`, `families`, `events`, `places`, `media`, `notes`. |
| `20261027000006_onboarding_and_home_person.sql` | User preferences: `home_person_id` and initial setup onboarding state. |
| `20261027000007_change_log_concurrency_snapshots.sql` | `change_log` audit table, entity version triggers, tree snapshots table. |
| `20261028000008_phase2_delight_schema.sql` | Face tags coordinates, places hierarchy columns, duplicate dismissals table. |
| `20261028000009_media_bundle_reconciliation.sql` | Media file hash index, bundle batch tracking, stub link mappings. |
| `20261104000010_ai_story_intake_and_mcp.sql` | AI draft subtrees, intake audio recordings, MCP personal access tokens (PAT). |

### 1.3 Backend Integration Tests (`backend/tests/`)

| Test Suite | Coverage Scope |
| :--- | :--- |
| `auth_test.rs` | Passkey authentication, magic links, passwords, session revocation, lockout. |
| `core_data_model_test.rs` | Schema validation, foreign keys, cascading rules, `gid` uniqueness. |
| `data_quality_events_test.rs` | Duplicate life/union event detection, conflicts, custom/batch merge, `FAMC` vital isolation, `"event_merge_bundle"` Undo, dismissal, MCP tools (6 tests). |
| `data_quality_places_test.rs` | Soft validation warnings, duplicate candidate detection, place hierarchy repair. |
| `gedcom_test.rs` | Import and export roundtrip of GEDCOM 5.5.1 and 7.0 sample files. |
| `gendate_fixtures_test.rs` | Complex genealogical date string parsing, serialization, and comparisons. |
| `health_test.rs` | Health check route, system status, uptime. |
| `history_concurrency_test.rs` | Audit logs, read-time label/GID/diff enrichment (`test_history_enrichment_labels_gids_and_clean_diffs`), optimistic locking (`409 Conflict`), rollback, undo (5 tests). |
| `media_bundle_reconciliation_test.rs`| ZIP archive batch ingestion, reconciliation of stubs with physical files. |
| `media_test.rs` | Uploads, EXIF parsing, image thumbnail generation, face tag management. |
| `onboarding_home_person_test.rs` | Setup wizard, first administrator registration, "Start with self" flow. |
| `operations_test.rs` | Backup creation, restore verification, database advisory locks. |
| `person_profile_test.rs` | Person details, life events, timeline generation, name variants. |
| `phase2_delight_test.rs` | Hourglass tree layout, fan chart, place map coordinates, anniversary feeds. |
| `printable_pdf_tests.rs` | PDF export layout calculation, multi-page cut marks, SVG vector rendering. |
| `rbac_test.rs` | Role-based authorization, privacy redactions, unauthenticated 404 segregation. |
| `search_directory_test.rs` | Cyrillic & Latin search, omnibox suggestions, surname index sorting. |
| `session_panic_test.rs` | Panic interception via `CatchPanicLayer`, 401 dead cookie eviction, and `PublicOrAuth` graceful fallback. |
| `story_intake_and_mcp_test.rs` | Multimodal story intake, pre-extracted graph ingestion, draft reconciliation, MCP server JSON-RPC. |
| `tree_view_test.rs` | Ancestor/descendant graph traversal, relationship calculations. |

---

## 2. Frontend (`frontend/src/`)

### 2.1 Directory Responsibilities

| Directory | Responsibility |
| :--- | :--- |
| `api/` | Modular typed API clients: `core.ts` (`apiFetch` with auto-purge on 401, defensive token sanitization, `ApiError`), `client.ts` (re-export facade), `auth-api.ts`, `genealogy-api.ts`, `delight-api.ts`, `delight-types.ts`, `history-api.ts`, `media-gedcom-pdf-api.ts`, `stories-mcp-api.ts` (story intake & MCP), `gemini-story-extractor.ts` (extraction engine selector), `openapi-types.ts`, and `openapi-types-domain.ts`. |
| `components/ui/` | Reusable Design System primitives (`~60` files): `overlay-stack.ts` (`useModalOverlay`), `Dialog` (`Modal`, `dialog-scroll-fade`), `Drawer`, `DropdownMenu` (`clampMenuHorizontalOffset`), `RadioChoiceRow`, `Switch`, `Toast` (`data-toast-region`), `SegmentedControl`, `BottomNav`, `AccessDeniedCard`, `Button`, `TextInput`, `Select`, `Tabs`. |
| `components/domain/` | Domain-specific widgets: `GenDateInput`, `PlacePicker`, `RoleBadge`, `GenderIcon`. |
| `components/auth/` | Auth UI: `LoginCard`, `MagicLinkVerifyCard`, `RecoveryCodesCard`, `RegisterInviteCard`. |
| `components/tree/` | Interactive canvas: `TreeCanvas`, `PersonNodeCard`, `FanChartCanvas`, `KinshipFinderDialog`, `TreePrivacyPreviewCard`. |
| `components/profile/` | Person profile view: `ProfileHeroHeader` (`buildProfileOverflowItems`), timeline events, relatives lists. |
| `components/person/` | Person creation and edit modals (`PersonEditDialog` with recursive `409` `openConflictFromError`, quick-add components). |
| `components/media/` | Gallery components: `MediaLightboxStudio`, `FaceTagCanvas`, `MediaMetadataPanel`. |
| `components/places/` | Place hierarchy tree, place edit modal, interactive Leaflet/SVG place maps. |
| `components/quality/` | Quality dashboard cards, candidate duplicate comparisons, duplicate events (`DuplicateEventsSubView`, `DuplicateEventCard`, `DuplicateEventCustomizeModal`, `default-quality-fixtures.ts`), place issue fixes. |
| `components/print/` | PDF configuration options, tile cut-mark previews (`PrintPreviewCanvas`). |
| `components/history/` | `RecordHistoryDrawer`, `ConflictResolutionModal`, `HistoryFilterBar`, `history-diff-constants.ts`, `history-diff-utils.ts`, `history-label-formatters.ts`. |
| `components/stories/` | Audio recording widget (`AudioRecorderWidget`), story input panel (`StoryInputPanel`), speaker selector (`SpeakerSelectorCard`), draft review canvas (`DraftSubtreeCanvas`, `ReconciliationMatrixCard`, `DraftClarificationBar`). |
| `components/settings/` | Settings management panels (`McpTokensPanel` with `availableScopesForRole`, `TreeSnapshotsCard` on `/settings/backups`, re-exported via `index.ts`). |
| `components/onboarding/`| First-run setup wizards (`EmptyTreeOnboardingCard`). |
| `hooks/` | Custom React hooks (`useLiveVersion`, `useUndoToast`). |
| `i18n/` | Localization engine (`index.tsx`), translation dictionaries (`en.json`, `ru.json`). |
| `lib/` | Utilities: `cn.ts` (Tailwind merge), `auth-context.tsx`, `theme.tsx`, `kinship.ts`, `patronymic.ts`, `webauthn.ts`, `error-formatter.ts` (`getLocalizedErrorMessage`, `isOfflineFallbackError`), `tree-layouts/` (`hourglass/`, `hourglass-layout.ts`, `all-nodes-layout.ts`, `pedigree-layout.ts`, `descendants-layout.ts`). |
| `pages/` | Top-level routing views (`PeoplePage`, `PlacesPage`, `MediaPage`, `HistoryPage`, `DataQualityPage`, `PersonMergePage`, `AiStoryIntakePage`, `PrintExportPage`, `DevUiWorkbench`, etc.). |
| `types/` | Domain data types (`person.ts`, `place.ts`, `event.ts`, `gendate.ts`). |

### 2.2 Key Frontend Unit & Integration Tests (`frontend/tests/` & `frontend/e2e/`)

| Test Suite | Coverage Scope |
| :--- | :--- |
| `tests/api/auth-token-resilience.test.ts` | Defensive token sanitization (`cleanToken`), 401 auto-purge in `apiFetch`, and `AuthProvider` token cleanup. |
| `tests/components/history-concurrency.test.tsx` | Change History feed/drawer/conflict modal EN/RU diff formatting, `{ stripGid }` dedup, `can_edit` profile History gate, overlay stack `Escape`/focus, and `DropdownMenu` / `RadioChoiceRow` UX checks (18 tests). |
| `tests/components/milestone-2-9-and-3-2.test.tsx` | Story intake UI, Gemini engine controls, and `McpTokensPanel` PAT role-ceiling scope & lifecycle flows. |
| `tests/components/person-merge-and-quality.test.tsx` | Person merge preview, Duplicate Events cards/customize modal/offline guard, `Dialog` focus trap & `SegmentedControl` tests (18 tests). |
| `tests/i18n/parity.test.ts` | Bilingual key parity between `en.json` and `ru.json`, and emoji/icon prohibition. |
| `e2e/duplicate-events.spec.ts` | Playwright Desktop/Mobile E2E tests for Duplicate Events (`e2e/fixtures/quality-api-mock.ts`, `e2e/pages/quality.page.ts`, `playwright.config.ts` with `serviceWorkers: 'block'`; 16 tests / 52 runs). |
| `e2e/history-visualization.spec.ts` | Playwright Desktop/Mobile E2E tests for Change History feed, `RecordHistoryDrawer`, `ConflictResolutionModal` retries, filter/RBAC & EN/RU parity (`e2e/fixtures/history-api-mock.ts`, `e2e/pages/history.page.ts`; 15 test specs). |

---

## 3. Deployment & Operations

| Path | Responsibility |
| :--- | :--- |
| `scripts/install.sh` | Unified single-command installer and atomic updater for Debian/Ubuntu LXC and bare-metal environments (automated PostgreSQL setup, direct port 80 Axum serving, zero Nginx). |
| `deploy/lxc/genealogy.service` | Hardened systemd service unit template (`CAP_NET_BIND_SERVICE`, `NoNewPrivileges`, `ProtectSystem=full`). |
| `deploy/lxc/genealogy.env.example` | Production environment configuration template for LXC (`PORT=80`, `FRONTEND_STATIC_PATH`). |
| `.github/workflows/release.yml` | On-demand GitHub Actions release pipeline cross-compiling Linux `x86_64` and `aarch64` tarballs with checksums. |
| `.agents/skills/genealogy-release/SKILL.md` | Skill automating pre-flight verification test suites, version bumping, and release tag publishing. |
| `docker-compose.yml` | 3-container Docker Compose deployment definition (`frontend`, `backend`, `postgres`). |
| `scripts/check-memory-bank.sh` | Automated verification script ensuring all file path references in `docs/memory-bank/` exist. |

---

## 4. File Size Compliance & Bundle Splitting

Per [`.agents/rules/modular-architecture-and-file-size.md`](../../.agents/rules/modular-architecture-and-file-size.md), the hard ceiling for source code files is **800 lines**.
- **Zero hotspots remain:** Every `.rs` file in `backend/src/` (max 781 lines in `backend/src/routes/rbac/tree_view.rs`) and every `.ts`/`.tsx` file in `frontend/src/` (max 794 lines in `frontend/src/pages/PersonMergePage.tsx`) is strictly ≤ 800 lines.
- **Former hotspots decomposed:**
  - `frontend/src/api/client.ts` → split into `core.ts`, `auth-api.ts`, `genealogy-api.ts`, `delight-api.ts`, `history-api.ts`, `media-gedcom-pdf-api.ts`, and `stories-mcp-api.ts`.
  - `frontend/src/api/openapi-types.ts` → split into `openapi-types.ts` and `openapi-types-domain.ts`.
  - `frontend/src/lib/tree-layouts/hourglass-layout.ts` → decomposed into `frontend/src/lib/tree-layouts/hourglass/` sub-modules.
  - `frontend/src/components/tree/TreeCanvas.tsx` and `frontend/src/pages/DevUiWorkbench.tsx` → extracted sub-components and domain panels.
  - `backend/src/routes/rbac/ai_mcp.rs` → split into `ai_stories.rs` and `mcp.rs`.
- **Vite Rollup Chunking (`frontend/vite.config.ts`):** `manualChunks` splits bundles into `vendor`, `vendor-icons`, `i18n-locales`, `domain-tree`, `domain-print`, `domain-stories`, and `domain-quality` (all chunks < 420 kB).

---

## 5. Where to Put New Code

- **New Backend Route:** Create thin handler in `backend/src/routes/rbac/<domain>.rs`, register in `backend/src/routes/rbac/mod.rs` and `backend/src/routes/mod.rs`, and annotate with `#[utoipa::path(...)]`.
- **New Business Logic / Service:** Add to `backend/src/services/<domain>/`. Keep files ≤ 600 lines; expose public interface via `mod.rs`.
- **New Migration:** Add timestamped SQL file in `backend/migrations/YYYYMMDDHHMMSS_<description>.sql`.
- **New Design System UI Primitive:** Add to `frontend/src/components/ui/` with full keyboard/ARIA support and light/dark theme tokens.
- **New Domain UI Component:** Add to `frontend/src/components/<domain>/`.
- **New Page:** Add to `frontend/src/pages/` and register route in `frontend/src/App.tsx`.
- **New User-Facing String:** Add identical keys to both `frontend/src/i18n/en.json` and `frontend/src/i18n/ru.json`.

_Last curated: 2026-11-14 (Change History enrichment/UI/E2E modules, Overlay Stack, Duplicate Events & verified module line counts)_
