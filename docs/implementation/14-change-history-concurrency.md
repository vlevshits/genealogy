# Implementation Report: Phase 1.11 — Change Log, Undo Toast, Restore Deleted, Optimistic Concurrency & Tree Snapshots

**Date:** 2026-10-28  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 11 of [`docs/roadmap.md`](../roadmap.md) — Comprehensive append-only change log (`change_log`), multi-record transaction grouping (`tx_id`, `tx_label`), 6-second countdown undo toast, soft-delete restoration, row-level optimistic concurrency with `409 Conflict` and 3-way conflict resolution, point-in-time tree snapshots with safe rollback and automatic pre-rollback safety snapshots, and Admin-only audit isolation ([`14-change-history-concurrency.md`](../features/14-change-history-concurrency.md), [`02-rbac-access-control.md`](../features/02-rbac-access-control.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Backend Service Layer, Data Model & Endpoints (`backend/`)
- **Forward Migration ([`backend/migrations/20261027000007_change_log_concurrency_snapshots.sql`](../../backend/migrations/20261027000007_change_log_concurrency_snapshots.sql)):**
  - Enhanced `change_log` with `tx_label VARCHAR(200)`, `entity_label VARCHAR(200)`, `actor_name VARCHAR(200)`, and indices (`idx_change_log_tx_created`, `idx_change_log_op_created`).
  - Enhanced `tree_snapshots` with `reason VARCHAR(32)`, `file_path TEXT`, `expires_at TIMESTAMPTZ`, and `record_counts JSONB`.
  - Bumped `system_metadata.schema_version` to `7`.
- **History & Snapshot Service ([`backend/src/services/history.rs`](../../backend/src/services/history.rs)):**
  - `log_change`: Atomic before/after capture across all mutations (individuals, families, events, places, notes, media) returning `ChangeLogEntryDto` with `tx_id`.
  - Transaction grouping (`ChangeLogTransactionGroupDto`): Groups related operations (`quick_add_relative`, `merge_place`, `merge_duplicate_people`) under single transaction ID with both `entries` and `changes` populated.
  - Per-record field diff computation (`FieldDiffDto { field, before, after }`).
  - Transaction undo (`POST /api/v1/history/tx/:tx_id/undo`): Reverts changes in reverse chronological order within 10 minutes for author or anytime for Admin.
  - Restore deleted (`POST /api/v1/history/:id/restore` and `POST /api/v1/persons/:id/restore`): Reconstitutes soft-deleted individuals and reconstructs parental/spousal family links.
  - Optimistic concurrency: `verify_optimistic_version` returns `409 Conflict` (`version_conflict`) with `details: { expected_version, current_version, their_data: TreePersonNodeDto }`.
  - Point-in-time tree snapshots (`create_snapshot`, `rollback_snapshot`): Writes compressed snapshot JSON to disk, enforces typed `confirm_text: "ROLLBACK"`, and automatically takes a `pre_rollback` safety snapshot prior to restoring state.
- **RBAC Endpoints ([`backend/src/routes/rbac.rs`](../../backend/src/routes/rbac.rs)):**
  - `GET /api/v1/history` (Admin only)
  - `GET /api/v1/history/:entity_type/:entity_id` (Editor & Admin)
  - `POST /api/v1/history/tx/:tx_id/undo` (Editor author ≤10m, Admin)
  - `POST /api/v1/history/:id/restore` (Editor & Admin)
  - `GET /api/v1/snapshots` and `POST /api/v1/snapshots` (Admin only)
  - `POST /api/v1/snapshots/:id/rollback` (Admin only)
  - `GET /api/v1/:entity_type/:id/version` (Editor & Admin)

### 1.2 Frontend Components & User Flows (`frontend/`)
- **6-Second Undo Toast ([`toast.tsx`](../../frontend/src/components/ui/toast.tsx), [`useUndoToast.ts`](../../frontend/src/hooks/useUndoToast.ts)):**
  - Non-intrusive floating toast with countdown timer badge (`6s`), smooth animated progress bar, and "Отменить" / "Undo" action button. Elevated above mobile `BottomNav` with safe-area clearance.
- **Record History Drawer ([`RecordHistoryDrawer.tsx`](../../frontend/src/components/history/RecordHistoryDrawer.tsx)):**
  - Slide-over drawer displaying chronological revision history, actor attribution, localized timestamps, expandable field diffs, and compact confirmation dialog to restore prior versions.
- **3-Way Conflict Resolution Modal ([`ConflictResolutionModal.tsx`](../../frontend/src/components/history/ConflictResolutionModal.tsx)):**
  - Handles `409 Conflict` on concurrent editing with 3 interactive options:
    1. *Keep my changes* (forces save with latest version)
    2. *Use their changes* (discards local draft and updates with latest data)
    3. *Merge field by field* (side-by-side radio comparisons with visible focus rings and mobile stacked layout)
- **Live Version Polling ([`useLiveVersion.ts`](../../frontend/src/hooks/useLiveVersion.ts)):**
  - 30-second background polling during form editing surfacing a gentle alert if another user saves changes.
- **Admin Global History Feed ([`HistoryPage.tsx`](../../frontend/src/pages/HistoryPage.tsx), [`HistoryFilterBar.tsx`](../../frontend/src/components/history/HistoryFilterBar.tsx)):**
  - Grouped transaction feed with filters (entity type, action, date range, deletions only), one-click transaction undo, and deleted record recovery.
- **Tree Snapshots Card ([`TreeSnapshotsCard.tsx`](../../frontend/src/components/settings/TreeSnapshotsCard.tsx)):**
  - Embedded inside `/settings/backups` with checkpoint list, record counts (individuals, families, events, places), retention countdown, snapshot creation, and typed `ROLLBACK` safety dialog.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Architect Review** | `architect-reviewer` (Mode 2) | **APPROVE** | Validated append-only audit trail, transaction undo window, 409 conflict contract, snapshot rollback safety, and Admin RBAC isolation. |
| **Backend Integration Tests** | `cargo test --test history_concurrency_test` | **PASS (4/4)** | Verified audit logging, undo window, soft-delete restoration, 409 conflict, version polling, snapshot creation, and safe rollback. |
| **Full Backend Test Suite** | `cargo test` | **PASS (46/46)** | 13 integration test suites passed with zero regressions. |
| **Backend Formatting & Linter** | `cargo fmt -- --check && cargo clippy -- -D warnings` | **PASS** | 0 warnings, clean compilation. |
| **Frontend Test Suite** | `npm test -- --run` | **PASS (105/105)** | 15 test suites passed including `history-concurrency.test.tsx` and i18n parity. |
| **Production Build** | `npm run build` | **PASS** | Clean build in 1.12s (`tsc --noEmit && vite build`). |
| **UI/UX Review** | `ux-reviewer` | **OK** | Verified `/history`, drawers, conflict modal, snapshot card, and undo toast across mobile and desktop (`UX-009` through `UX-014` fixed). |
| **QA Full-Stack Browser** | `qa-analyst` (Playwright) | **PASS** | Live browser tests verified `/history` feed, per-record drawer restore, 6-second undo toast, 409 conflict resolution, and rollback confirmation. `DEF-001`, `DEF-002`, `DEF-003` resolved. |
