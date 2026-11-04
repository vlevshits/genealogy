# Active Context

Current operational status, recent milestones, and immediate roadmap priorities.

---

## 1. Current Phase & Milestone

- **Current Roadmap Phase:** Transitioning from Phase 2 (Delight & Visualizations) to Phase 3 (Advanced Expansion & Integrations).
- **Recent Completed Milestones:**
  - **Phase 3 Printable PDF Export (`05-printable-tree-pdf`):** High-resolution poster exports and multi-page tiled PDFs with cut marks, preview canvas, and full EN/RU studio controls.
  - **Phase 2.8 Associated People & Client Face Detection (`06-media-face-tagging`):** Linked people in photo sidebars, client-side face suggestion rectangles, and manual coordinate tagging.
  - **Phase 2.7 Multiple Spouses & Multi-Trace Kinship Path Finder (`04-tree-visualizations`):** Pruned spurious graph detours and resolved family-partitioned children distribution.
  - **Phase 19 AI Story Intake & MCP Server (`19-ai-family-story-intake-and-mcp`):** Backend schema, migrations, and MCP endpoints implemented; staging UI available in `AiStoryIntakePage`.

---

## 2. Recent Key Changes

1. **PDF Export Alignment:** Corrected descendant layout connector drops and poster dimensions in `backend/src/services/pdf/layout.rs` and `frontend/src/components/print/PrintPreviewCanvas.tsx` (`d4da455`).
2. **GEDCOM Analysis & Quality Refinements:** Added localized warnings, improved diff formatting, and added tab state URL synchronizations in `DataQualityPage` (`ae9b7b4`).
3. **HTML Email Templates:** Implemented responsive, localized transactional email templates for magic links, account recovery, and family member invitations in `backend/src/services/email.rs` (`0c84fb1`).
4. **Kinship Path Optimization:** Pruned cross-spouse detours and loop artifacts in `backend/src/services/genealogy.rs` (`9b9eabe`).

---

## 3. Next Steps & Open Focus Areas

1. **Modular Architecture Refactoring (Technical Debt Reduction):**
   - Decompose `backend/src/routes/rbac.rs` (10,300 lines) into cohesive domain route submodules (`routes/people.rs`, `routes/tree.rs`, `routes/places.rs`, `routes/media.rs`, etc.).
   - Decompose `frontend/src/api/client.ts` (3,319 lines) into modular domain clients under `frontend/src/api/` (auth, people, places, media, quality).
   - Break down `backend/src/services/quality.rs` (5,279 lines) into submodules.
2. **AI Story Intake Polish:**
   - Connect client-side Whisper / Web Speech API recording flows to `POST /api/v1/ai/transcribe`.
   - Validate draft subtree reconciliation and 1-click merge into live genealogy records.
3. **Memory Bank Upkeep:**
   - Keep feature implementation memories synchronized as features evolve.
   - Run `scripts/check-memory-bank.sh` on every change.

---

## 4. Open Questions & Invariants

- **Living Person 110-Year Boundary:** Ensure approximate date inferences from child/spouse events correctly guard living person status without leaking existence to unauthenticated users.
- **MCP Security Token Expiration:** Personal Access Tokens (PAT) for external AI agents must adhere strictly to RBAC scope restrictions.

_Last curated: 2026-11-04 (memory bank bootstrap)_
