# Change History & Concurrency — Implementation Memory

**Spec:** [docs/features/14-change-history-concurrency.md](../../features/14-change-history-concurrency.md) · **Status:** implemented

## 1. Summary
Full audit change logging, transactional undo/restore, optimistic concurrency control, and 1-click tree snapshots. Captures every record modification in an immutable `change_log` table, tracks entity versions (`version`), and allows editors to revert recent edits via toast notifications or browse historical entity revisions in a drawer.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/` | `change_log` table, `tree_snapshots` table, `version` columns across tables |
| Routes | `backend/src/routes/rbac.rs` | Audit endpoints, undo transaction, snapshot CRUD, version check |
| Service | `backend/src/services/history.rs` | `record_change`, `undo_transaction`, `create_snapshot`, `rollback_snapshot` |
| History Page | `frontend/src/pages/HistoryPage.tsx` | Tree audit log table, user filter, date filter, snapshot manager |
| History Drawer | `frontend/src/components/history/` | `RecordHistoryDrawer.tsx` (entity-level timeline diff view) |
| Hooks | `frontend/src/hooks/` | `useLiveVersion.ts` (polling version updates), `useUndoToast.ts` (transient undo action) |
| Tests | `backend/tests/history_concurrency_test.rs` | Version conflict `409`, transaction undo, snapshot rollback |
| Frontend Tests | `frontend/tests/components/` | `history-concurrency.test.tsx` |

## 3. Key Flows
1. **Optimistic Concurrency Protection:**
   - Client fetches person with `version: 3`.
   - Another editor updates person to `version: 4`.
   - First client submits update with expected version 3 → Backend detects mismatch and returns `409 Conflict` (`version_conflict`).
   - Frontend shows conflict modal displaying remote changes vs local edits.
2. **Transient Undo Toast:**
   - Editor edits a birth date → Update commits with transaction ID `tx_id`.
   - Frontend pops toast: "Person updated — [Undo]".
   - Editor clicks [Undo] → calls `POST /api/v1/history/tx/{tx_id}/undo` within 10 seconds to restore previous values.
3. **Tree Snapshots:**
   - Admin triggers `POST /api/v1/snapshots` before a major import or merge.
   - Point-in-time snapshot serialized and stored in database; can be rolled back with 1-click.

## 4. Invariants & Business Rules
- **Immutable Audit Trail:** `change_log` records are append-only and cannot be altered or deleted through standard APIs.
- **Transactional Atomicity:** All entity modifications within a single mutation must be assigned the same `transaction_id`.

## 5. Decisions
- **`ADR-008`: Database Advisory Lock for Snapshots:** Snapshot rollback acquires an advisory lock to prevent data corruption during restoration.

## 6. Gotchas & Known Limitations
- Undoing a parent deletion must automatically re-bind the restored parent to existing family container nodes.

_Last curated: 2026-11-04 (memory bank bootstrap)_
