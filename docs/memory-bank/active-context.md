# Active Context

Current operational status, recent milestones, and immediate roadmap priorities.

---

## 1. Current Phase & Milestone

- **Current Roadmap Phase:** Milestone 3.2 (Story Intake & Gemini 3.8 Flash Integration) complete. Preparing for Milestone 3.3 (Conversational Adjustments & Advanced MCP Operations).
- **Recent Completed Milestones:**
  - **Milestone 3.2 AI Story Intake & Gemini 3.8 Flash Integration (`19-ai-family-story-intake-and-mcp`):** Server-side Google Gemini 3.8 Flash extraction and audio transcription configured via `GEMINI_API_KEY` in `.env`, `ai_enabled` flag in `GET /api/v1/system/info`, server-side tree reconciliation, cross-browser audio recorder, and multi-engine toggle (Gemini Server vs Client BYOK vs Built-in Offline).
  - **Backend Modular Architecture Refactoring:** Decomposed monolithic routes and services (`routes/rbac/`, `services/quality/`, `services/story_intake/`, `services/auth/`, `services/genealogy/`, `services/gedcom/`, `services/pdf/`, `services/privacy/`, `services/history/`, `services/mcp/`) into cohesive modules strictly ≤ 800 lines.
  - **Phase 3 Printable PDF Export (`05-printable-tree-pdf`):** High-resolution poster exports and multi-page tiled PDFs with cut marks, preview canvas, and full EN/RU studio controls.
  - **Phase 2.8 Associated People & Client Face Detection (`06-media-face-tagging`):** Linked people in photo sidebars, client-side face suggestion rectangles, and manual coordinate tagging.

---

## 2. Recent Key Changes

1. **Server-Side Gemini 3.8 Flash Integration (`backend/src/services/story_intake/gemini.rs`):** Implemented `transcribe_audio_with_gemini` and `extract_story_with_gemini`, providing direct server-side audio transcription and structured genealogical entity extraction using `GEMINI_API_KEY` and `GEMINI_MODEL`.
2. **Centralized AI Config & Instance Discovery:** Configured `gemini_api_key` and `gemini_model` in `backend/src/config.rs`; exposed `ai_enabled: bool` on `GET /api/v1/system/info` (`backend/src/routes/system.rs`) so the frontend dynamically adapts intake options.
3. **Multipart & JSON Audio Transcription (`backend/src/routes/rbac/ai_mcp.rs`):** `POST /api/v1/ai/transcribe` supports both `multipart/form-data` uploads and JSON `audio_base64` payloads for `.webm`, `.mp3`, `.wav`, `.m4a`, and `.ogg` formats.
4. **Client BYOK Override & Offline Fallback:** Retained optional user-provided API key storage in `localStorage` (`genealogy_gemini_api_key`) in `frontend/src/api/gemini-story-extractor.ts`, with graceful automatic fallback to the built-in offline NLP engine when Gemini is unconfigured or encounters errors.
5. **Backend Architecture Modularization:** Modularized backend routes and services into domain packages conforming to the 800-line ceiling rule (`b565cbc`).

---

## 3. Next Steps & Open Focus Areas

1. **Frontend Hotspot Modularization:**
   - Decompose `frontend/src/api/client.ts` (3,319 lines) into modular domain clients under `frontend/src/api/` (auth, people, places, media, quality).
   - Decompose `frontend/src/lib/tree-layouts/hourglass-layout.ts` (1,925 lines) and `frontend/src/components/tree/TreeCanvas.tsx` (1,447 lines).
2. **Conversational Adjustments & Multimodal MCP (Milestone 3.3):**
   - Interactive clarification dialogs for unresolved ambiguity notes in story drafts.
   - MCP server extension for external agent-driven family tree modifications.
3. **Memory Bank Upkeep:**
   - Run `scripts/check-memory-bank.sh` on every change and keep references current.

---

## 4. Open Questions & Invariants

- **Draft Sandbox Isolation:** AI-generated records are never committed directly to canonical tables (`persons`, `families`) without human verification and review.
- **Server AI Configuration with Optional BYOK:** Setting `GEMINI_API_KEY` in `.env` provides a zero-configuration experience for family members; individual users can still optionally provide their own API key via `GeminiApiKeyModal`.
- **MCP Security Token Expiration:** Personal Access Tokens (PAT) for external AI agents adhere strictly to RBAC scope restrictions (`read:tree`, `write:draft`, `write:direct`).

_Last curated: 2026-11-05 (Milestone 3.2 server-side Gemini 3.8 Flash)_
