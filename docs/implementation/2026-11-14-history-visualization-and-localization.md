# Implementation Report — Change History Visualization & Localization (Feature 14)

**Date:** 2026-11-14  
**Status:** **COMPLETE**  
**Feature Spec:** [`docs/features/14-change-history-concurrency.md`](../features/14-change-history-concurrency.md)  
**Architect Review Verdict:** **APPROVE** (`0` open findings)  
**UX Review:** [`docs/ux-reviews/2026-11-14-history-visualization-ux-review.md`](../ux-reviews/2026-11-14-history-visualization-ux-review.md) — initial verdict `CHANGES REQUESTED` (`UX-138`–`UX-150`); all 13 tasks subsequently implemented and verified by unit/E2E suites and headless Chromium checks (formal re-run skipped per user request to finalize ASAP).

---

## 1. Overview & Objective

Improved Change History readability, humanization, and EN/RU localization across `/history` (`HistoryPage`), person profile `RecordHistoryDrawer`, and optimistic-concurrency `ConflictResolutionModal`, replacing raw `snake_case` field keys, 36-character UUIDs, and raw JSON strings (`GenDate`, `FamilyChildDto`, merge/delete bundles) with localized labels and meaningful before/after comparisons.

---

## 2. Backend Changes (`backend/`)

### 2.1 Read-Time History Enrichment
- **`backend/src/services/history/enrich.rs`**, **`enrich_index.rs`**, **`enrich_diff.rs`**, **`enrich_format.rs`**:
  - Added read-time enrichment over both `GET /api/v1/history` (`query_history_feed`) and `GET /api/v1/history/{entity_type}/{entity_id}` (`query_record_history`), preserving compact `live-store.json` serialization via `#[serde(default, skip_serializing_if = "...")]`.
  - Built `HistoryLookupIndex` overlaying live store records over historical snapshots (with `resolve_id_by_gid` fallback) to resolve foreign-key UUIDs (`person_id`, `partner1_id`, `partner2_id`, `child_id`, `place_id`, `parent_id`, `media_id`, `event_id`, etc.) into human-readable `"Name (GID)"` labels.
  - Unwrapped composite payloads (`DeletedPersonSnapshotPayload`, `person_merge_bundle`, `event_merge_bundle`, `record_counts`) into clean `FieldDiffDto` entries (`diffs: Vec<FieldDiffDto>` on `ChangeLogEntryDto` in `backend/src/services/privacy/dto.rs`) and structured `record_counts` JSON objects.
  - Populated `entity_gid: Option<String>` on `ChangeLogEntryDto` and `RecordHistoryItemDto` (`backend/src/services/history/types.rs`), stripping redundant trailing ` ({gid})` suffixes from `entity_label`.
- **`backend/src/services/history/feed.rs`**, **`backend/src/services/history/mod.rs`**, **`backend/src/services/quality/merge.rs`**, **`backend/src/services/gedcom/importer.rs`**:
  - Wired enrichment helpers, structured `record_counts` logging on snapshot restore/rollback, and clean merge/import history metadata.

---

## 3. Frontend & Accessibility Changes (`frontend/`)

### 3.1 Diff Formatting & Bilingual Label Localization
- **Diff & Label Modules (`frontend/src/components/history/`):**
  - `history-diff-constants.ts` (`140` lines), `history-diff-utils.ts` (`426` lines), `history-label-formatters.ts` (`253` lines).
  - Mapped all history fields (`history.fields.*`), transaction labels (`history.txLabels.*`, including splitting composite `"<prefix> — <target>"` labels), entity types (`history.entityTypes.*`), actions (`history.actions.*`), status/name/living values (`history.values.*`), `common.{yes,no}`, and ICU-pluralized `history.recordCounts.*`.
  - Localized embedded backend tokens (`"Note — ..."`, `"<event_type> — ..."`, `[primary avatar]`, child pedigree suffixes `(Adopted)`/`(Biological)`), formatted `GenDate` objects and date fields via `parseGenDate` + `formatGenDate(genDate, locale)`, and formatted `undone_tx_id` via `formatUndoneTxRef` (avoiding title-cased internal IDs like `Tx Profile 201`).
- **`frontend/src/api/history-api.ts`**, **`frontend/src/lib/error-formatter.ts`**, **`frontend/src/i18n/{en,ru}.json`**:
  - Extended history DTOs (`entity_gid`, `summary`, `can_revert`, `EntityHistoryResponseDto.entity_label`, `'merge' | 'merged' | 'gedcom_import'`), extracted shared `isOfflineFallbackError(err)`, and replaced `(s)` strings with ICU plural templates.

### 3.2 History UI, Conflict Resolution, Overlay Stack & Mobile UX (`UX-138`–`UX-150`, `DEF-HIST-001`–`005`)
- **Overlay Stack & Focus Management (`UX-138`, `UX-139`, `UX-140`, `UX-146`):**
  - Created `frontend/src/components/ui/overlay-stack.ts` (`210` lines) providing `useModalOverlay` shared by `Dialog` (`dialog.tsx`) and `Drawer` (`drawer.tsx`) so `Escape` closes only the topmost overlay, background siblings stay `inert` / `aria-hidden="true"` (skipping `[data-toast-region]`) until the stack unwinds, and focus restores to the triggering element.
  - Upgraded `DropdownMenu` (`dropdown-menu.tsx`) with viewport horizontal edge clamping (`clampMenuHorizontalOffset`), `aria-haspopup`/`aria-expanded`/`aria-controls`, DOM focus movement across `ArrowDown`/`ArrowUp`/`Home`/`End`, and `Escape` focus restoration.
  - Added `color-scheme: light dark` in `frontend/index.html` and `color-scheme: light` / `color-scheme: dark` in `frontend/src/index.css` for dark-mode native date picker icon contrast, plus `touch-target-expand` on `Switch` and `>= 40px` hit targets on Drawer close and primary/standalone actions.
- **Page, Drawer & Conflict Modal Enhancements:**
  - `HistoryPage.tsx` (`653` lines): deduplicated `(GID)` in card headers, added clickable entity links to `/persons/:id`, `/places`, `/tree`, added client-side `fromDate`/`toDate` filtering in `filterHistoryGroups`, added `Field`/`Before`/`After` diff headers, rendered `<EmptyState>` on `200 OK []`, and gated offline fallbacks with `isOfflineFallbackError`.
  - `HistoryFilterBar.tsx` (`256` lines): added collapsible mobile filter toggle (`history-mobile-filters-toggle`) with active count badge and `FilterChip` pills.
  - `RecordHistoryDrawer.tsx` (`492` lines): made `loading` / `loadError` (with Retry button) / `<EmptyState>` / list mutually exclusive, hid restore controls on `create` or `can_revert: false`, and updated responsive metadata layout.
  - `ConflictResolutionModal.tsx` (`372` lines) & `PersonEditDialog.tsx`: added non-interactive initial comparison view (`0` radios until `"Merge field by field"` is clicked), added accessible `RadioChoiceRow` in `radio-group.tsx`, moved actions into sticky `<Dialog footer={...}>` with single `variant="primary"`, forwarded `remoteVersion`, formatted `updatedAt`, surfaced retry errors in `conflict-error-alert`, and handled nested `409` refreshes cleanly.
  - `access-denied-card.tsx`, `bottom-nav.tsx`, `ProfileHeroHeader.tsx`, `TreeSnapshotsCard.tsx`, `RecentActivityWidget.tsx`: fixed mobile RU badge wrapping, gated admin routes in `BottomNav` More drawer, gated `viewHistory` on `capabilities.can_edit`, widened checkpoint input, and hid raw `(snap-*)` IDs when labeled.

---

## 4. Verification & Test Coverage

All checks ran directly after the final code edit:
- **Backend Formatting, Linting & Tests:**
  - `cargo fmt --all -- --check`: **PASS**
  - `cargo clippy --all-targets -- -D warnings`: **PASS** (0 warnings)
  - `cargo test`: **159 passed, 0 failed, 0 ignored** (15 `lib.rs` + 15 `main.rs` + 129 integration tests including `history_concurrency_test` 5/5)
- **Frontend Tests & Production Build:**
  - `npm run test:i18n`: **16/16 passed** (100% EN/RU key parity, zero glyph/emoji violations)
  - `npm test`: **28/28 files, 342/342 tests passed** (including `tests/components/history-concurrency.test.tsx` 18/18)
  - `npm run build` (`tsc --noEmit && vite build`): **PASS** (0 errors, built in `1.31s`)
- **Playwright End-to-End Tests (`frontend/e2e/history-visualization.spec.ts`):**
  - `npx tsc --noEmit`: **PASS** (0 errors)
  - `npx playwright test e2e/history-visualization.spec.ts`: **42 passed (10.5s), 0 failed, 0 skipped** across `chromium` and `mobile-chromium`
- **File Size Audit:** All created and modified files under `backend/src/` and `frontend/src/` are `<= 800` lines (largest touched frontend file: `HistoryPage.tsx` at `653` lines).

---

## 5. Deferred Non-Critical / Pre-Existing Items (`docs/technical-debt.md`)

- `TD-HIST-01` (`DEF-HIST-006`): `log_change` + `verify_optimistic_version` on the 5 media face-tag and link routes in `backend/src/routes/rbac/media_tags_links.rs`.
- `TD-HIST-02` (`DEF-HIST-007`): `DeletedPersonSnapshot` restore parity for stripped `face_tags`/`media_links`, subsequent-edit conflict guard in `undo.rs`, and `RecordHistoryDrawer` triggers on Places/Families pages.
- `TD-HIST-03` (`DEF-HIST-008`): Wiring `snapshots.rs::prune_expired_history_and_snapshots` (`HISTORY_RETENTION_DAYS`) into `scheduler.rs::spawn_background_scheduler`.
- `TD-E2E-PRE`: Pre-existing E2E selector drift in untouched `frontend/e2e/family-links.spec.ts` and `frontend/e2e/stories-and-print.spec.ts`.

