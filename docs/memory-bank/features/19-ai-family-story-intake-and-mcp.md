# AI Family Story Intake & MCP Server — Implementation Memory

**Spec:** [docs/features/19-ai-family-story-intake-and-mcp.md](../../features/19-ai-family-story-intake-and-mcp.md) · **Status:** implemented

## 1. Summary
Multimodal family story intake ("Tell Me About My Family") and a protected Model Context Protocol (MCP) server. Users can record spoken memories in-browser or enter written narratives; the system transcribes audio and parses entities into staged draft subtrees via server-side Google Gemini 3.8 Flash (configured via `GEMINI_API_KEY` and `GEMINI_MODEL` in `.env`), with automatic fallback to the built-in offline NLP engine. No API keys are ever entered or stored in the client browser. Reconciles candidate entities against canonical tree individuals and allows atomic commit. External AI agents integrate via role-scoped Personal Access Tokens (PAT) with living-person redaction enforced on read tools.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261104000010_ai_story_intake_and_mcp.sql` | `story_intake_drafts`, `personal_access_tokens` |
| Config | `backend/src/config.rs` | `gemini_api_key`, `gemini_model` (defaults to `"gemini-3.8-flash"`) |
| System Route | `backend/src/routes/system.rs` | `GET /api/v1/system/info` exposes `ai_enabled: bool` |
| Story Routes | `backend/src/routes/rbac/ai_stories.rs` | `POST /api/v1/ai/parse-story`, `POST /api/v1/ai/transcribe`, `GET/PATCH/DELETE /api/v1/ai/drafts/{id}`, `POST /api/v1/ai/drafts/{id}/reconcile`, `POST /api/v1/ai/drafts/{id}/commit`, `GET/PUT /api/v1/admin/ai-settings` |
| MCP Routes | `backend/src/routes/rbac/mcp.rs` | `GET/POST /api/v1/mcp/tokens`, `DELETE /api/v1/mcp/tokens/{id}`, `POST /api/v1/mcp`, `GET /api/v1/mcp/sse`, `GET /api/v1/mcp/tools` |
| Route Re-export | `backend/src/routes/rbac/ai_mcp.rs` | Re-exports `ai_stories::*` and `mcp::*` |
| Story Service | `backend/src/services/story_intake/` | Modular package: `gemini.rs`, `types_api.rs`, `extraction.rs`, `nlp_clauses.rs`, `nlp_entities.rs`, `reconciliation.rs`, `commit.rs` |
| Gemini Service | `backend/src/services/story_intake/gemini.rs` | `transcribe_audio_with_gemini`, `extract_story_with_gemini` (uses `x-goog-api-key` header; truncates upstream error logs to 200 chars) |
| MCP Service | `backend/src/services/mcp/` | Modular package: `catalog.rs`, `rpc_read.rs` (`redact_person_for_mcp`, `redact_family_for_mcp`), `tools_mutation.rs`, `types_tokens.rs` |
| Engine Selector | `frontend/src/api/gemini-story-extractor.ts` | `getExtractionEngine`, `setExtractionEngine` (`'gemini' \| 'builtin'`), purges legacy `genealogy_gemini_api_key` |
| Story API Client | `frontend/src/api/stories-mcp-api.ts` | `storyApi` (`transcribeAudio`, `parseStory`, `refineDraft`, `reconcileDraft`, `commitDraft`), `mcpApi` (`listTokens`, `createToken`, `revokeToken`), `normalizeStoryDraft` |
| Intake Page | `frontend/src/pages/AiStoryIntakePage.tsx` | Mode toggles (Gemini 3.8 Flash vs Built-in), audio recorder, draft tree preview, commit |
| Story Components | `frontend/src/components/stories/` | `AudioRecorderWidget.tsx`, `StoryInputPanel.tsx`, `SpeakerSelectorCard.tsx`, `DraftSubtreeCanvas.tsx`, `ReconciliationMatrixCard.tsx`, `DraftClarificationBar.tsx` |
| Settings UI | `frontend/src/components/settings/McpTokensPanel.tsx` | Scoped PAT token generator and revocation confirmation dialog |
| i18n | `frontend/src/i18n/en.json` / `ru.json` | `stories.engine.*`, `stories.gemini.*`, `stories.recorder.*`, `stories.input.*` |
| Tests | `backend/tests/story_intake_and_mcp_test.rs` | Story parsing with/without `extracted_data`, reconciliation, MCP JSON-RPC tools & redaction |
| Frontend Tests | `frontend/tests/components/gemini-story-extractor.test.tsx`, `frontend/tests/api/stories-mcp-api.test.ts` | Engine storage, DTO normalization, HTTP error propagation (`GOT-012`) |

## 3. Key Flows
1. **Server-Side Gemini 3.8 Flash Extraction:**
   - User inputs family story narrative on `AiStoryIntakePage` with engine set to "Gemini 3.8 Flash".
   - Browser sends `POST /api/v1/ai/parse-story` with `raw_text`, `engine`, and optional `speaker_person_id` (aliased as `speaker_id`).
   - Backend checks `state.config.gemini_api_key`. If present, calls `extract_story_with_gemini` with temperature 0.0, JSON schema mode, and the `x-goog-api-key` HTTP header.
   - If Gemini extraction fails or key is missing, backend logs a truncated warning (≤ 200 chars) and falls back to the built-in NLP engine (`extract_narrative_graph`).
   - Persists draft to `story_intake_drafts` and executes `reconcile_story_draft_in_place` against canonical tree records.
2. **Audio Intake & Server-Side Transcription:**
   - User records speech in `AudioRecorderWidget.tsx` (using `MediaRecorder` chunked at 250ms, with full `MediaStream` track and `SpeechRecognition` unmount cleanup) or uploads an audio file (`.webm`, `.mp3`, `.wav`, `.m4a`, `.ogg`).
   - Browser calls `POST /api/v1/ai/transcribe` (multipart/form-data or JSON with `audio_base64`).
   - Backend calls `transcribe_audio_with_gemini`, encoding audio into base64 inline data for Gemini `generateContent`.
   - Returns transcribed text to UI, appending it to the narrative input.
3. **Built-in Offline Extraction:**
   - Selected explicitly via `setExtractionEngine('builtin')` or invoked automatically when no server `GEMINI_API_KEY` is configured.
   - Executes rule-based NLP extraction (`extract_narrative_graph`) without external network calls.
4. **Draft Review, Reconciliation & Atomic Commit:**
   - User reviews matched candidates (`link_enrich` normalized to `'link_existing'`) and kinship links in `DraftSubtreeCanvas.tsx` and `ReconciliationMatrixCard.tsx`, or triggers `POST /api/v1/ai/drafts/{id}/reconcile`.
   - Submitting triggers `POST /api/v1/ai/drafts/{id}/commit` inside an atomic SQL transaction, logging an undoable `change_log` entry.

## 4. Invariants & Business Rules
- **Server AI Availability Discovery:** `GET /api/v1/system/info` exposes `ai_enabled: bool` (`state.config.gemini_api_key.is_some() || store.ai_config.has_api_key`), allowing the UI to adapt engine options without exposing credentials.
- **No Client-Side API Keys:** `GEMINI_API_KEY` lives exclusively on the server and is transmitted via `x-goog-api-key`. `frontend/src/api/gemini-story-extractor.ts` purges any legacy `genealogy_gemini_api_key` from `localStorage`.
- **Draft Sandbox Isolation:** Extracted records are strictly held in `story_intake_drafts` and never touch canonical `persons` or `families` tables until user review and explicit commit.
- **CSRF-Protected Reconciliation:** `/api/v1/ai/drafts/{id}/reconcile` accepts `POST` only and is guarded by CSRF middleware.
- **MCP Living-Person & Event Redaction:** `Viewer`-tier MCP tokens map to `CallerAccess::AnonymousPublic` via `mcp_privacy_access`. `redact_person_for_mcp`, `redact_family_for_mcp`, and `redact_event_for_tree_view` filter out private records and mask living persons, their marriage details, and their timeline events across `search_individuals`, `get_individual`, `get_family_immediate`, `get_ancestors_descendants`, and `list_events_timeline`.
- **MCP PAT Scoping:** External agent tokens (`personal_access_tokens`) carry explicit RBAC scopes (`read:tree`, `write:draft`, `write:direct`, `admin:maintenance`) and authenticate via `Authorization: Bearer <pat>`.

## 5. Decisions
- [ADR-011](../decisions.md#adr-011-server-side-gemini-38-flash-extraction-and-audio-transcription-via-gemini_api_key): Server-Side Gemini 3.8 Flash Extraction and Audio Transcription via `GEMINI_API_KEY` (supersedes [ADR-010](../decisions.md#adr-010-client-side-external-llm-extraction-with-server-side-tree-reconciliation)).

## 6. Gotchas & Known Limitations
- [GOT-012](../gotchas.md#got-012-api-client-offline-fallback-guards-must-rethrow-apierror-when-status--0): Offline fallback guards in `stories-mcp-api.ts` must rethrow `ApiError` when `err.status > 0` so HTTP 4xx/5xx responses are not masked by demo data.
- [GOT-011](../gotchas.md#got-011-microphone-mediastream-track--speechrecognition-cleanup-on-unmount): `AudioRecorderWidget.tsx` must stop all `MediaStream` tracks and abort `SpeechRecognition` on unmount.
- [GOT-010](../gotchas.md#got-010-server-side-audio-transcription-requires-gemini_api_key-in-env): Audio uploads to `POST /api/v1/ai/transcribe` require `GEMINI_API_KEY` in `.env` unless in-browser speech recognition transcript is provided.
- [GOT-009](../gotchas.md#got-009-browser-audio-recording-codec-compatibility--object-url-leaks): Cross-browser `MediaRecorder` mime type differences (Safari `audio/mp4` vs Chrome `audio/webm;codecs=opus`) and object URL cleanup.

## 7. Extension Points
- Future LLM/STT providers (local Whisper, Ollama, Anthropic Claude) can implement the transcription and extraction trait interfaces in `backend/src/services/story_intake/`.

_Last curated: 2026-11-07 (Batch 3 modularization, security hardening & staleness audit)_

