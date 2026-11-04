# AI Family Story Intake & MCP Server — Implementation Memory

**Spec:** [docs/features/19-ai-family-story-intake-and-mcp.md](../../features/19-ai-family-story-intake-and-mcp.md) · **Status:** implemented

## 1. Summary
Multimodal family story intake ("Tell Me About My Family") and a protected Model Context Protocol (MCP) server. Older relatives can record spoken memories or paste written family stories; the system transcribes audio, parses familial entities and life events into staged draft subtrees, clarifies ambiguities, and safely reconciles draft records into the canonical tree. External AI tools can integrate via role-scoped Personal Access Tokens (PAT).

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261104000010_ai_story_intake_and_mcp.sql` | `ai_story_drafts`, `ai_story_recordings`, `mcp_tokens`, `ai_settings` |
| Routes | `backend/src/routes/rbac.rs` | `/ai/*` story intake endpoints, `/mcp` JSON-RPC handler, `/mcp/sse`, `/auth/mcp-tokens` |
| Story Service | `backend/src/services/story_intake.rs` | Audio transcription orchestrator, LLM story parser, draft staging & commit engine |
| MCP Service | `backend/src/services/mcp.rs` | JSON-RPC 2.0 transport, SSE streaming, PAT token validation, tool dispatcher |
| Intake Page | `frontend/src/pages/AiStoryIntakePage.tsx`| Audio recorder interface, story transcript viewer, draft review panel |
| Components | `frontend/src/components/stories/` | Voice recording widget, draft person card, relationship preview |
| Settings UI | `frontend/src/components/settings/` | MCP token generator, AI provider configuration (Gemini / OpenAI / Local Whisper) |

## 3. Key Flows
1. **Oral Story Intake & Transcription:**
   - Relative presses "Record" on `AiStoryIntakePage` and recounts memories.
   - Client uploads audio file to `POST /api/v1/ai/transcribe`.
   - Backend transcribes audio via configured provider (e.g. OpenAI Whisper or local model).
2. **Draft Subtree Staging:**
   - Raw transcript sent to `POST /api/v1/ai/parse-story`.
   - LLM extracts individuals, relationships, life events, and locations into an isolated `ai_story_draft` row (draft subtree).
   - Draft records remain in a sandbox and do not alter the main tree.
3. **Reconciliation & Commit:**
   - User reviews extracted relationships side-by-side with existing tree members.
   - User clicks "Commit Story" → `POST /api/v1/ai/drafts/{id}/commit` takes a snapshot and merges confirmed records into canonical tables.
4. **MCP Agent Interaction:**
   - External agent connects to `/api/v1/mcp` with `Authorization: Bearer <pat_token>`.
   - Calls tools (`search_people`, `get_person_profile`, `stage_story_draft`) subject to token role restrictions.

## 4. Invariants & Business Rules
- **Draft Sandbox Isolation:** AI-generated records are never committed directly to canonical tables (`persons`, `families`) without human verification and review.
- **PAT Token Scoping:** MCP tokens inherit RBAC roles (Admin, Editor, Viewer) and cannot execute actions forbidden to their assigned role.

## 5. Decisions
- **Pluggable AI Providers:** Support both external cloud APIs (BYOK: Gemini, OpenAI) and offline/local fallback options to respect self-hosted privacy principles.

## 6. Gotchas & Known Limitations
- `services/story_intake.rs` (3,086 lines) and `services/mcp.rs` (1,655 lines) exceed the recommended file size ceiling and require future submodule decomposition.

_Last curated: 2026-11-04 (memory bank bootstrap)_
