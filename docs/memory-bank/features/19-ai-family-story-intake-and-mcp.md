# AI Family Story Intake & MCP Server — Implementation Memory

**Spec:** [docs/features/19-ai-family-story-intake-and-mcp.md](../../features/19-ai-family-story-intake-and-mcp.md) · **Status:** implemented

## 1. Summary
Multimodal family story intake ("Tell Me About My Family") and a protected Model Context Protocol (MCP) server. Users can record spoken memories in-browser or enter written narratives; the system transcribes audio and parses entities into staged draft subtrees via server-side Google Gemini 3.8 Flash (configured via `GEMINI_API_KEY` and `GEMINI_MODEL` in `.env`). Pure LLM extraction is enforced in production; if `GEMINI_API_KEY` is missing, the backend returns an explicit 400 validation error and the UI displays a clear configuration banner (the offline rule-based parser is retained strictly for offline integration test suites). No API keys are ever entered or stored in the client browser. Reconciles candidate entities against canonical tree individuals and allows atomic commit. External AI agents integrate via role-scoped Personal Access Tokens (PAT) with living-person redaction enforced on read tools.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261104000010_ai_story_intake_and_mcp.sql` | `story_intake_drafts`, `personal_access_tokens` |
| Config | `backend/src/config.rs` | `gemini_api_key`, `gemini_model` (defaults to `"gemini-3.8-flash"`) |
| System Route | `backend/src/routes/system.rs` | `GET /api/v1/system/info` exposes `ai_enabled: bool` |
| Story Routes | `backend/src/routes/rbac/ai_stories.rs` | `POST /api/v1/ai/parse-story` (validates `GEMINI_API_KEY` in non-test envs, returns `ValidationFailed` if unset), `POST /api/v1/ai/transcribe`, `GET/PATCH/DELETE /api/v1/ai/drafts/{id}`, `POST /api/v1/ai/drafts/{id}/reconcile`, `POST /api/v1/ai/drafts/{id}/commit`, `GET/PUT /api/v1/admin/ai-settings` |
| MCP Routes | `backend/src/routes/rbac/mcp.rs` | `GET/POST /api/v1/mcp/tokens`, `DELETE /api/v1/mcp/tokens/{id}`, `GET/POST /api/v1/auth/mcp-tokens`, `DELETE /api/v1/auth/mcp-tokens/{id}`, `POST /api/v1/mcp`, `GET /api/v1/mcp/sse` (`list_mcp_tokens_handler`, `create_mcp_token_handler`, `revoke_mcp_token_handler`, `resolve_mcp_caller`, `mcp_jsonrpc_handler`, `mcp_sse_handler`) |
| Route Re-export | `backend/src/routes/rbac/ai_mcp.rs` | Re-exports `ai_stories::*` and `mcp::*` |
| Story Service | `backend/src/services/story_intake/` | Modular package: `gemini.rs`, `types_api.rs` (`DraftMatchOverrideDto` with serde aliases `draft_person_id` / `target_person_id`), `extraction.rs`, `nlp_clauses.rs`, `nlp_entities.rs`, `reconciliation.rs`, `commit.rs` |
| Gemini Service | `backend/src/services/story_intake/gemini.rs` | `transcribe_audio_with_gemini`, `extract_story_with_gemini` (uses `x-goog-api-key` header; truncates upstream error logs to 200 chars) |
| MCP Service | `backend/src/services/mcp/` | Modular package (`mod.rs`): `catalog.rs`, `db_tokens.rs` (`PersistedMcpTokenRecord`, `serialize_persisted_tokens`, `persist_mcp_tokens_to_dir`, `load_mcp_tokens_from_dir`, `is_mcp_token_expired`, `merge_mcp_tokens_into_store`, `load_seed_mcp_token`, `deterministic_seed_pat_id`, `is_seed_or_legacy_admin_token`, DB sync/lookup helpers), `rpc_read.rs` (`handle_mcp_jsonrpc`, `mcp_privacy_access`, `redact_person_for_mcp`, `redact_family_for_mcp`), `tools_mutation.rs` (`execute_mcp_mutation_or_admin_tool` incl. `merge_duplicate_individuals`, `list_duplicate_events`, `merge_duplicate_events` → `events_duplicates_batch.rs`, `undo_transaction`), `types_tokens.rs` (`allowed_scopes_for_access`, `create_mcp_token`, `list_mcp_tokens`, `revoke_mcp_token`) |
| PAT Persistence & Resets | `backend/src/services/privacy/store.rs`, `backend/src/services/privacy/masking.rs`, `backend/src/services/auth/hydration.rs`, `backend/src/routes/auth/login.rs`, `backend/src/services/gedcom/importer.rs` | `PersonalAccessTokenDto.token_hash` hidden over HTTP (`skip_serializing`), persisted to `live-store.json` + `mcp-tokens.json` via `serialize_persisted_tokens` / `persist_to_dir`; `AuthService::hydrate_from_db` + `ensure_user_by_id`; `complete_initial_setup` & `import_gedcom_into_store` preserve PATs & `ai_config` |
| Engine Storage | `frontend/src/api/gemini-story-extractor.ts` | `getExtractionEngine`, `setExtractionEngine` (`'gemini' \| 'builtin'`), purges legacy `genealogy_gemini_api_key` |
| Story API Client | `frontend/src/api/stories-mcp-api.ts` | `storyApi` (`transcribeAudio`, `parseStory`, `refineDraft`, `reconcileDraft`, `commitDraft` with dual alias `match_overrides`), `mcpApi` (`listTokens`, `createToken`, `revokeToken`), `McpTokenScope`, `normalizeStoryDraft` |
| Error Formatter | `frontend/src/lib/error-formatter.ts` | `getLocalizedErrorMessage` preserves specific backend message on `validation_failed` |
| Intake Page | `frontend/src/pages/AiStoryIntakePage.tsx` | Pure Gemini 3.8 Flash intake, `gemini-not-configured-banner` when unconfigured, derived proactive clarification questions (`derivedQuestions`), audio recorder, draft tree preview, commit |
| Story Components | `frontend/src/components/stories/` | `AudioRecorderWidget.tsx`, `StoryInputPanel.tsx` (server AI badge + config banner; local engine toggle dropped), `SpeakerSelectorCard.tsx`, `DraftSubtreeCanvas.tsx`, `ReconciliationMatrixCard.tsx` (interpolated action labels), `DraftClarificationBar.tsx` (question chips + title count) |
| Settings UI | `frontend/src/components/settings/McpTokensPanel.tsx` | Scoped PAT generator (`availableScopesForRole`, re-exported via `frontend/src/components/settings/index.ts`), role-change scope sync, revocation confirmation dialog |
| i18n | `frontend/src/i18n/en.json` / `ru.json` | `stories.engine.*`, `stories.gemini.*`, `stories.recorder.*`, `stories.input.*`, `stories.roles.*` (including `self`, `partner`, grandparent variations), `settings.mcp.*` |
| Tests | `backend/tests/story_intake_and_mcp_test.rs`, `backend/tests/data_quality_events_test.rs` | Story parsing with/without `extracted_data`, reconciliation, MCP JSON-RPC tools & redaction, PAT disk/DB persistence, expiration, merge aliases & duplicate events MCP tools |
| Frontend Tests | `frontend/tests/components/gemini-story-extractor.test.tsx`, `frontend/tests/components/milestone-2-9-and-3-2.test.tsx`, `frontend/tests/api/stories-mcp-api.test.ts` | Engine storage, `McpTokensPanel` role-ceiling defaults & PAT flows, DTO normalization, HTTP error propagation (`GOT-012`) |

## 3. Key Flows
1. **Server-Side Gemini 3.8 Flash Extraction:**
   - User inputs family story narrative on `AiStoryIntakePage`.
   - Browser sends `POST /api/v1/ai/parse-story` with `raw_text`, `engine: 'gemini'`, and optional `speaker_person_id` (aliased as `speaker_id`).
   - Backend checks `state.config.gemini_api_key`. If absent and `app_env != "test"`, returns an explicit `AppError::ValidationFailed` (`gemini_api_key: missing`).
   - If present, calls `extract_story_with_gemini` with temperature 0.0, JSON schema mode, and the `x-goog-api-key` HTTP header.
   - Persists draft to `story_intake_drafts` and executes `reconcile_story_draft_in_place` against canonical tree records.
2. **Audio Intake & Server-Side Transcription:**
   - User records speech in `AudioRecorderWidget.tsx` (using `MediaRecorder` chunked at 250ms, with full `MediaStream` track and `SpeechRecognition` unmount cleanup) or uploads an audio file (`.webm`, `.mp3`, `.wav`, `.m4a`, `.ogg`).
   - Browser calls `POST /api/v1/ai/transcribe` (multipart/form-data or JSON with `audio_base64`).
   - Backend calls `transcribe_audio_with_gemini`, encoding audio into base64 inline data for Gemini `generateContent`.
   - Returns transcribed text to UI, appending it to the narrative input.
3. **Built-in Offline Parser (Test Environments Only):**
   - Retained exclusively for automated test suites (`app_env == "test"` in `backend/tests/story_intake_and_mcp_test.rs`).
   - Executes rule-based NLP extraction (`extract_narrative_graph`) deterministically without external network calls.
4. **Draft Review, Reconciliation & Atomic Commit:**
   - User reviews matched candidates (`link_enrich` normalized to `'link_existing'`) and kinship links in `DraftSubtreeCanvas.tsx` and `ReconciliationMatrixCard.tsx`, or triggers `POST /api/v1/ai/drafts/{id}/reconcile`.
   - Interactive clarification questions in `DraftClarificationBar.tsx` allow answering questions (e.g. birth dates, maiden names) to update the draft.
   - Submitting triggers `POST /api/v1/ai/drafts/{id}/commit` inside an atomic SQL transaction, logging an undoable `change_log` entry.
5. **PAT Dual Persistence, Expiration & Role-Default Scoping:**
   - `McpTokensPanel.tsx` (`availableScopesForRole`) pre-selects all scopes permitted for the caller's role, synchronizes on role changes, and resets after token creation; backend `create_mcp_token` likewise defaults omitted or empty `scopes` to `allowed_scopes_for_access(access)` (`ADR-016`).
   - `PersonalAccessTokenDto.token_hash` stays hidden over HTTP (`#[serde(skip_serializing, default)]`) while `GenealogyPrivacyStore::persist_to_dir` writes `PersistedMcpTokenRecord`s (`serialize_persisted_tokens` in `db_tokens.rs`) atomically to both `live-store.json` and `mcp-tokens.json`, and `routes/rbac/mcp.rs` syncs PostgreSQL `personal_access_tokens` (`upsert_mcp_token_in_db`, `revoke_mcp_token_in_db`, `list_mcp_tokens_handler`).
   - `AuthService::hydrate_from_db` merges DB tokens, `mcp-tokens.json`, and optional `SEED_MCP_PAT` / `seed-pat.txt` (`load_seed_mcp_token`, `deterministic_seed_pat_id`). At request time `resolve_mcp_caller` checks memory and `find_mcp_token_by_hash_in_db`, enforces UTC expiration via `is_mcp_token_expired`, updates `touch_mcp_token_last_used_in_db`, and only allows orphaned-user admin fallback when `is_seed_or_legacy_admin_token` is true and `!is_user_deactivated_in_db`.
6. **MCP Duplicate Person Merge Execution:**
   - `handle_mcp_jsonrpc` unwraps `arguments`, `args`, `input`, stringified JSON payloads, or top-level `params`.
   - `merge_duplicate_individuals` (`tools_mutation.rs`) normalizes `surviving_id` (`surviving_id`, `keep_id`, `target_id`, `primary_person_id`, `surviving_person_id`, `person_a_id`) and `duplicate_id` (`duplicate_id`, `merge_id`, `source_id`, `secondary_person_id`, `duplicate_person_id`, `person_b_id`), validates non-empty IDs upfront (`ValidationFailed` / JSON-RPC `-32602`), forwards `field_choices` / `field_overrides` + `BACKUP_PATH` to `execute_person_merge`, preserves chosen `birth`/`death` fields across vital recomputation, calls `store.persist_if_production()`, and returns `merged_person_id` alongside `merged_id`.

## 4. Invariants & Business Rules
- **Server AI Availability Discovery & Enforcement:** `GET /api/v1/system/info` exposes `ai_enabled: bool` (`state.config.gemini_api_key.is_some() || store.ai_config.has_api_key`). When disabled, `AiStoryIntakePage` renders an amber warning banner and blocks extraction submissions with a descriptive toast.
- **Pure LLM Extraction in Production:** Narrative extraction strictly requires Gemini 3.8 Flash in production (`ADR-014`). Silent fallback to regex heuristics is eliminated to avoid corrupting kinship graphs.
- **No Client-Side API Keys:** `GEMINI_API_KEY` lives exclusively on the server and is transmitted via `x-goog-api-key`. `frontend/src/api/gemini-story-extractor.ts` purges any legacy `genealogy_gemini_api_key` from `localStorage`.
- **Draft Sandbox Isolation:** Extracted records are strictly held in `story_intake_drafts` and never touch canonical `persons` or `families` tables until user review and explicit commit.
- **Full Role & Action i18n Translation:** All kinship roles (`self`, `partner`, `paternal_grandmother`, etc.) and reconciliation card actions interpolate properly with zero unreplaced raw `{place}` or `{name}` tokens.
- **CSRF-Protected Reconciliation:** `/api/v1/ai/drafts/{id}/reconcile` accepts `POST` only and is guarded by CSRF middleware.
- **Commit Payload Compatibility & Deserialization Resilience:** `DraftMatchOverrideDto` accepts either `draft_temp_id` or `draft_person_id`, and `matched_person_id` or `target_person_id`, ensuring backward-compatible draft commitment across web UI, MCP tools, and external callers. `error-formatter.ts` surfaces explicit validation failure messages to operators rather than generic fallback strings.
- **MCP Living-Person & Event Redaction:** `Viewer`-tier MCP tokens map to `CallerAccess::AnonymousPublic` via `mcp_privacy_access`. `redact_person_for_mcp`, `redact_family_for_mcp`, and `redact_event_for_tree_view` filter out private records and mask living persons, their marriage details, and their timeline events across `search_individuals`, `get_individual`, `get_family_immediate`, `get_ancestors_descendants`, and `list_events_timeline`.
- **MCP PAT Scoping, Hash Secrecy & Reset Preservation:** External agent tokens (`personal_access_tokens`) carry explicit RBAC scopes (`read:tree`, `write:draft`, `write:direct`, `admin:maintenance`), authenticate via `Authorization: Bearer <pat>`, default omitted/empty creation scopes to the caller's role ceiling, never serialize `token_hash` in HTTP DTOs (`ADR-016`), preserve tokens + `ai_config` across `complete_initial_setup` and `import_gedcom_into_store` resets, and reject orphaned PATs with `401 NotAuthenticated` unless `is_seed_or_legacy_admin_token` applies.

## 5. Decisions
- [ADR-017](../decisions.md#adr-017-duplicate-events-detection-non-destructive-merge-union-consolidation--undo-bundle): Duplicate Events Detection, Non-Destructive Merge, Union Consolidation & Undo Bundle (exposed via Tier C `list_duplicate_events` & `merge_duplicate_events` MCP tools).
- [ADR-016](../decisions.md#adr-016-dual-persistence-mcp-personal-access-token-storage-without-http-hash-exposure): Dual-Persistence MCP Personal Access Token Storage Without HTTP Hash Exposure.
- [ADR-014](../decisions.md#adr-014-enforce-pure-llm-extraction-via-gemini-38-flash-and-drop-local-regex-fallback): Enforce Pure LLM Extraction via Gemini 3.8 Flash and Drop Local Regex Fallback (amends [ADR-011](../decisions.md#adr-011-server-side-gemini-38-flash-extraction-and-audio-transcription-via-gemini_api_key)).
- [ADR-011](../decisions.md#adr-011-server-side-gemini-38-flash-extraction-and-audio-transcription-via-gemini_api_key): Server-Side Gemini 3.8 Flash Extraction and Audio Transcription via `GEMINI_API_KEY` (supersedes [ADR-010](../decisions.md#adr-010-client-side-external-llm-extraction-with-server-side-tree-reconciliation)).

## 6. Gotchas & Known Limitations
- [GOT-021](../gotchas.md#got-021-mcp-undo_transaction-parameter-order--custom-event-merge-field-clearing): MCP `undo_transaction` must pass `(current_year, age_threshold)` in exact positional order.
- [GOT-018](../gotchas.md#got-018-mcp-merge_duplicate_individuals-parameter-aliases-vs-rest-empty-body-rbac-smoke-test): MCP `merge_duplicate_individuals` argument wrappers/aliases must be normalized and validated upfront so missing params do not hit the empty-body REST RBAC smoke-test no-op.
- [GOT-017](../gotchas.md#got-017-mcp-personal-access-token-hash-stripping-non-utc-expiration--store-resets): `PersonalAccessTokenDto.token_hash` requires `PersistedMcpTokenRecord` disk serialization, UTC parsing via `is_mcp_token_expired`, and store-reset preservation.
- [GOT-015](../gotchas.md#got-015-story-intake-commit-deserialization-failure-on-draft_person_id-vs-draft_temp_id): `DraftMatchOverrideDto` commit payload deserialization requires serde aliases (`draft_person_id`, `target_person_id`) to prevent 400 validation failures.
- [GOT-012](../gotchas.md#got-012-api-client-offline-fallback-guards-must-rethrow-apierror-when-status--0): Offline fallback guards in `stories-mcp-api.ts` must rethrow `ApiError` when `err.status > 0` so HTTP 4xx/5xx responses are not masked by demo data.
- [GOT-011](../gotchas.md#got-011-microphone-mediastream-track--speechrecognition-cleanup-on-unmount): `AudioRecorderWidget.tsx` must stop all `MediaStream` tracks and abort `SpeechRecognition` on unmount.
- [GOT-010](../gotchas.md#got-010-server-side-audio-transcription-requires-gemini_api_key-in-env): Audio uploads to `POST /api/v1/ai/transcribe` require `GEMINI_API_KEY` in `.env` unless in-browser speech recognition transcript is provided.
- [GOT-009](../gotchas.md#got-009-browser-audio-recording-codec-compatibility--object-url-leaks): Cross-browser `MediaRecorder` mime type differences (Safari `audio/mp4` vs Chrome `audio/webm;codecs=opus`) and object URL cleanup.

## 7. Extension Points
- Future LLM/STT providers (local Whisper, Ollama, Anthropic Claude) can implement the transcription and extraction trait interfaces in `backend/src/services/story_intake/`.

_Last curated: 2026-11-14 (MCP list_duplicate_events/merge_duplicate_events tools, undo_transaction arg fix GOT-021, PAT persistence/merge aliases)_

