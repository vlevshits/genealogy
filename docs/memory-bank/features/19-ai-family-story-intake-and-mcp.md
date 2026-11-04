# AI Family Story Intake & MCP Server — Implementation Memory

**Spec:** [docs/features/19-ai-family-story-intake-and-mcp.md](../../features/19-ai-family-story-intake-and-mcp.md) · **Status:** implemented

## 1. Summary
Multimodal family story intake ("Tell Me About My Family") and a protected Model Context Protocol (MCP) server. Users can record spoken memories in-browser or enter written narratives; the system transcribes audio and parses entities into staged draft subtrees via server-side Google Gemini 3.8 Flash (configured via `GEMINI_API_KEY` and `GEMINI_MODEL` in `.env`), with optional client-side BYOK override and built-in offline NLP engine fallback. Reconciles candidate entities against canonical tree individuals and allows atomic commit. External AI agents integrate via role-scoped Personal Access Tokens (PAT).

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261104000010_ai_story_intake_and_mcp.sql` | `story_intake_drafts`, `personal_access_tokens` |
| Config | `backend/src/config.rs` | `gemini_api_key`, `gemini_model` (defaults to `"gemini-3.8-flash"`) |
| System Route | `backend/src/routes/system.rs` | `GET /api/v1/system/info` exposes `ai_enabled: bool` |
| Routes | `backend/src/routes/rbac/ai_mcp.rs` | `/api/v1/ai/parse-story`, `/api/v1/ai/transcribe`, `/api/v1/ai/drafts/*`, `/api/v1/mcp`, `/api/v1/auth/mcp-tokens` |
| Story Service | `backend/src/services/story_intake/` | Modular package: `gemini.rs`, `types_api.rs`, `extraction.rs`, `nlp_clauses.rs`, `nlp_entities.rs`, `reconciliation.rs`, `commit.rs` |
| Gemini Service | `backend/src/services/story_intake/gemini.rs` | `transcribe_audio_with_gemini`, `extract_story_with_gemini` |
| MCP Service | `backend/src/services/mcp/` | Modular package: `catalog.rs`, `rpc_read.rs`, `tools_mutation.rs`, `types_tokens.rs` |
| LLM Extractor | `frontend/src/api/gemini-story-extractor.ts` | `extractStoryWithGemini`, `getGeminiApiKey`, `setGeminiApiKey`, `getExtractionEngine`, `setExtractionEngine` |
| Story API Client | `frontend/src/api/stories-mcp-api.ts` | `transcribeAudioApi`, `parseStoryIntoDraftApi`, `commitStoryDraftApi`, `createMcpTokenApi`, etc. |
| Intake Page | `frontend/src/pages/AiStoryIntakePage.tsx` | Mode toggles (Gemini 3.8 Flash vs Built-in), audio recorder, draft tree preview, commit |
| Story Components | `frontend/src/components/stories/` | `AudioRecorderWidget.tsx`, `StoryInputPanel.tsx`, `GeminiApiKeyModal.tsx`, `DraftSubtreeCanvas.tsx` |
| Settings UI | `frontend/src/components/settings/McpTokensPanel.tsx` | Scoped PAT token generator and management dialog |
| i18n | `frontend/src/i18n/en.json` / `ru.json` | `stories.engine.*`, `stories.gemini.*`, `stories.recorder.*`, `stories.input.*` |
| Tests | `backend/tests/story_intake_and_mcp_test.rs` | Story parsing with/without `extracted_data`, reconciliation, MCP JSON-RPC tools |
| Frontend Tests | `frontend/tests/components/gemini-story-extractor.test.tsx` | Key storage, schema transformation, fallback error handling |

## 3. Key Flows
1. **Server-Side Gemini 3.8 Flash Extraction:**
   - User inputs family story narrative on `AiStoryIntakePage` with engine set to "Gemini 3.8 Flash".
   - Browser sends `POST /api/v1/ai/parse-story` with `raw_text` and optional `speaker_id`.
   - Backend checks `state.config.gemini_api_key`. If present, calls `extract_story_with_gemini` with temperature 0.0 and JSON schema mode.
   - If Gemini extraction fails or key is missing, backend logs warning and falls back to built-in NLP engine (`extract_narrative_graph`).
   - Persists draft to `story_intake_drafts` and executes `reconcile_story_draft_in_place` against canonical tree records.
2. **Audio Intake & Server-Side Transcription:**
   - User records speech in `AudioRecorderWidget.tsx` (using `MediaRecorder` chunked at 250ms) or uploads audio file (`.webm`, `.mp3`, `.wav`, `.m4a`, `.ogg`).
   - Browser calls `POST /api/v1/ai/transcribe` (multipart/form-data or JSON with `audio_base64`).
   - Backend calls `transcribe_audio_with_gemini`, encoding audio into base64 inline data for Gemini `generateContent`.
   - Returns transcribed text to UI, appending it to the narrative input.
3. **Optional Client-Side BYOK Override:**
   - User can provide custom Google AI Studio key via `GeminiApiKeyModal.tsx` stored in `localStorage` (`genealogy_gemini_api_key`).
   - If custom key is present, browser can invoke `extractStoryWithGemini` directly and forward `ParseStoryRequest.extracted_data`.
4. **Built-in Offline Extraction:**
   - Selected explicitly or invoked when no Gemini key is configured.
   - Executes rule-based NLP extraction (`extract_narrative_graph`) without network calls.
5. **Draft Review & Atomic Commit:**
   - User reviews matched candidates and kinship links in `DraftSubtreeCanvas.tsx`.
   - Submitting triggers `POST /api/v1/ai/drafts/{id}/commit` inside an atomic SQL transaction, logging an undoable `change_log` entry.

## 4. Invariants & Business Rules
- **Server AI Availability Discovery:** `GET /api/v1/system/info` exposes `ai_enabled: bool` (`state.config.gemini_api_key.is_some() || store.ai_config.has_api_key`), allowing the UI to adapt engine options.
- **Draft Sandbox Isolation:** Extracted records are strictly held in `story_intake_drafts` and never touch canonical `persons` or `families` tables until user review and explicit commit.
- **Graceful Fallback:** If server Gemini extraction fails or rate limits, parsing automatically falls back to the built-in offline NLP engine.
- **Audio MIME Support:** `POST /api/v1/ai/transcribe` accepts `.webm`, `.mp3`, `.wav`, `.m4a`, and `.ogg` audio files.
- **Uniform Tree Reconciliation:** Regardless of extraction engine (Gemini server, Gemini client BYOK, or offline parser), `reconcile_story_draft_in_place` always runs on the backend against database individuals.
- **MCP PAT Scoping:** External agent tokens (`personal_access_tokens`) carry explicit RBAC scopes (`read:tree`, `write:draft`, `write:direct`) and authenticate via standard `Authorization: Bearer <pat>`.

## 5. Decisions
- [ADR-011](../decisions.md#adr-011-server-side-gemini-38-flash-extraction-and-audio-transcription-via-gemini_api_key): Server-Side Gemini 3.8 Flash Extraction and Audio Transcription via `GEMINI_API_KEY`.
- [ADR-010](../decisions.md#adr-010-client-side-external-llm-extraction-with-server-side-tree-reconciliation): Client-Side External LLM Extraction with Server-Side Tree Reconciliation (superseded by ADR-011; retained as optional client BYOK override).

## 6. Gotchas & Known Limitations
- [GOT-010](../gotchas.md#got-010-server-side-audio-transcription-requires-gemini_api_key-in-env): Audio uploads to `POST /api/v1/ai/transcribe` require `GEMINI_API_KEY` in `.env` unless in-browser speech recognition transcript is provided.
- [GOT-009](../gotchas.md#got-009-browser-audio-recording-codec-compatibility--object-url-leaks): Cross-browser `MediaRecorder` mime type differences (Safari `audio/mp4` vs Chrome `audio/webm;codecs=opus`) and object URL cleanup.

## 7. Extension Points
- Future LLM/STT providers (local Whisper, Ollama, Anthropic Claude) can implement the transcription and extraction trait interfaces in `backend/src/services/story_intake/`.

_Last curated: 2026-11-05 (Milestone 3.2 server-side Gemini 3.8 Flash)_

