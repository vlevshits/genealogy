# Change History & Concurrency — Implementation Memory

**Spec:** [docs/features/14-change-history-concurrency.md](../../features/14-change-history-concurrency.md) · **Status:** implemented

## 1. Summary
Full audit change logging, read-time label/GID/diff enrichment, transactional undo/restore, optimistic concurrency control (`409` field reconciliation), and 1-click tree checkpoints (`TreeSnapshotsCard`). Every record modification appends to `change_log` with a shared `tx_id` and entity `version`, enriched at read time via `HistoryLookupIndex` into localized entity names, canonical `gid`s, and human-readable field diffs without bloating stored JSON.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261027000007_change_log_concurrency_snapshots.sql` | `change_log`, `tree_snapshots`, `version` columns |
| Routes | `backend/src/routes/rbac/history_quality.rs`, `person_delete_families.rs` | `GET /api/v1/history`, `GET /api/v1/history/{entity_type}/{entity_id}`, `POST /api/v1/history/tx/{tx_id}/undo`, `POST /api/v1/history/{id}/restore`, `GET/POST /api/v1/snapshots`, `POST /api/v1/snapshots/{id}/rollback` |
| DTOs | `backend/src/services/privacy/dto.rs`, `backend/src/services/history/types.rs` | `ChangeLogEntryDto` (`entity_gid`, `diffs` with `skip_serializing_if`), `FieldDiffDto`, `RecordHistoryItemDto`, `ChangeLogTransactionGroupDto`, `HistoryFeedQuery`, `TreeSnapshotRecordCountsDto` |
| History Service | `backend/src/services/history/` | `mod.rs`, `types.rs`, `feed.rs` (`query_history_feed`, `query_record_history`), `undo.rs` (`undo_transaction`, `"event_merge_bundle"`, `restore_change`), `snapshots.rs` (`create_snapshot`, `rollback_snapshot`, `prune_expired_history_and_snapshots`) |
| Read-Time Enrichment | `backend/src/services/history/enrich*.rs` | `enrich_index.rs` (`HistoryLookupIndex::from_store`), `enrich.rs` (`enrich_entry`, `resolve_id_by_gid`, `resolve_entity_gid`, `resolve_entity_label`, `strip_trailing_gid`), `enrich_diff.rs` (`compute_diffs_with_index`, `diff_person_merge`, `diff_event_merge`, `normalize_snapshot_object`), `enrich_format.rs` (`format_gendate_value`, `format_children_array`, `safe_fallback_id`) |
| History Page & Filters | `frontend/src/pages/HistoryPage.tsx`, `frontend/src/components/history/HistoryFilterBar.tsx` | Admin-only `/history` grouped feed, `filterHistoryGroups`, inline diffs with clickable entity links, `AccessDeniedCard`; `HistoryFilterBar` filters by entity type, action `op`, `fromDate`/`toDate`, and `onlyDeletions` (no user filter; snapshots live on `/settings/backups`) |
| Drawer & Conflict UI | `frontend/src/components/history/` | `RecordHistoryDrawer.tsx`, `ConflictResolutionModal.tsx` (`computeConflictFields`, `RadioChoiceRow` when `mergeMode`), `history-diff-constants.ts`, `history-diff-utils.ts` (`extractEntryDiffs`, `formatDiffValue`), `history-label-formatters.ts` (`formatTxLabel`, `formatUndoneTxRef`, `formatEntityLabel`, `formatFieldLabel`, `formatRecordCounts`, `localizeEmbeddedDisplayString`), `index.ts` |
| Related Components | `frontend/src/components/settings/TreeSnapshotsCard.tsx`, `frontend/src/components/person/PersonEditDialog.tsx`, `frontend/src/components/profile/ProfileHeroHeader.tsx`, `frontend/src/components/dashboard/RecentActivityWidget.tsx` | `TreeSnapshotsCard` (`/settings/backups` checkpoints + `ROLLBACK` confirm); `PersonEditDialog` (`openConflictFromError`, chained `409` retries); `ProfileHeroHeader` (`buildProfileOverflowItems` gating History by `capabilities.can_edit`); `RecentActivityWidget` (`formatTxLabel`) |
| API & Hooks | `frontend/src/api/history-api.ts`, `frontend/src/lib/error-formatter.ts`, `frontend/src/hooks/` | Typed DTOs + `getHistoryFeed`, `getEntityHistory`, `undoTransaction`, `restoreChange`, snapshots; `isOfflineFallbackError`; `useLiveVersion.ts`, `useUndoToast.ts` |
| Design System Support | `frontend/src/components/ui/` | `overlay-stack.ts` (`useModalOverlay`), `dialog.tsx`, `drawer.tsx`, `radio-group.tsx` (`RadioChoiceRow`), `dropdown-menu.tsx`, `switch.tsx`, `bottom-nav.tsx`, `access-denied-card.tsx` |
| Backend Tests | `backend/tests/history_concurrency_test.rs` | 5 integration tests (`test_change_log_feed_and_per_record_field_diffs`, undo/restore, `409` version check, snapshots, `test_history_enrichment_labels_gids_and_clean_diffs`) |
| Frontend & E2E Tests | `frontend/tests/components/history-concurrency.test.tsx`, `frontend/e2e/history-visualization.spec.ts` | 18 Vitest specs (`history-concurrency.test.tsx`) + 15 Playwright specs (`e2e/fixtures/history-api-mock.ts`, `e2e/pages/history.page.ts`) covering EN/RU viewports, diffs, `409` retries, empty states, RBAC |

## 3. Key Flows
1. **Read-Time History Enrichment (`query_history_feed` & `query_record_history`):**
   - `HistoryLookupIndex::from_store` indexes historical `change_log` snapshots first and overlays current live store entities (so renamed or deleted targets retain human names and GIDs), plus transaction metadata (`tx_id` → primary target & `tx_label`).
   - `query_record_history` resolves `gid` slugs via `resolve_id_by_gid` when canonical UUIDs are not passed.
   - `enrich_entry` / `compute_diffs_with_index` strip internal `NOISE_KEYS` (`id`, `gid`, `version`, `tx_id`, sort keys, raw bundle snapshots), format `GenDate`, foreign keys, children pedigree tags, `undone_tx_id`, and `record_counts`, and attach `entity_gid` and `diffs` on read.
   - `HistoryPage` renders `formatEntityLabel(..., { stripGid: true })` with a single monospace GID pill (avoiding `(I0001)` duplication) and clickable links to `/persons/:id`, `/places`, or `/tree` for non-deleted records; `RecordHistoryDrawer` keeps `{ stripGid: false }` in its description and hides Restore when `op === "create"` or `can_revert === false`.
2. **Optimistic Concurrency & Conflict Resolution (`409 Conflict`):**
   - Client edits carry `version` / `If-Match`. On version mismatch, backend returns `409` (`version_conflict`) with remote fields and `current_version` / `version`.
   - `PersonEditDialog` (`openConflictFromError`) opens `ConflictResolutionModal` showing a read-only 3-column diff by default (`mergeMode = false`) with localized scalar/`sex` values and `Intl.DateTimeFormat` timestamps.
   - Switching to **Merge Field by Field** reveals `RadioChoiceRow` controls (`mine` vs `theirs`), and chained `409` errors during retry recursively refresh `ConflictResolutionModal` with the latest `remoteVersion` while resetting `mergeMode` when reopened.
3. **Undo, Restore & Offline Fallback Guardrails:**
   - `HistoryPage`, `RecordHistoryDrawer`, `DashboardPage`, and `useUndoToast` share `isOfflineFallbackError` (`status === 0 || code === 'network_error'`) so HTTP `4xx`/`5xx` server errors display localized alerts/toasts (`history.undoErrorTitle`, `history.restore.errorTitle`) while genuine offline states fall back cleanly; empty `200 OK` feeds render `EmptyState` rather than sample rows.
4. **Tree Checkpoints & Rollback (`TreeSnapshotsCard`):**
   - Mounted under Settings → Backups (`/settings/backups`), creating manual checkpoints and requiring `ROLLBACK` typed confirmation before `rollback_snapshot` restores tree tables behind an automatic safety snapshot.

## 4. Invariants & Business Rules
- **Immutable Audit Trail & Atomic `tx_id`:** `change_log` rows are append-only; related mutations within a user action share a `tx_id`.
- **Zero Storage Bloat for Enrichment (`ADR-018`):** Writers (`log_change`, `execute_person_merge`, `import_gedcom_into_store`) leave `entity_gid: None` and `diffs: Vec::new()`; `ChangeLogEntryDto` skips serializing them when empty (`#[serde(default, skip_serializing_if = "...")]`).
- **RBAC Boundaries:** `/history` feed and navigation links (`AppShell`, `BottomNav`) are Admin-only (`AccessDeniedCard` for Editor/Viewer). Per-record `RecordHistoryDrawer` access on profile pages is gated by `capabilities.can_edit` (`ProfileHeroHeader`), allowing Editors to inspect/restore record history without granting Admin tree-feed access.
- **No Raw UUID or Raw JSON Leaks:** Foreign keys, `undone_tx_id`, `GenDate` objects, and merge bundles render localized human labels or compact `#xxxxxxxx` references.

## 5. Decisions
- **`ADR-008`: Database Advisory Lock for Snapshots:** Snapshot rollback acquires an advisory lock to prevent concurrent state corruption.
- **`ADR-017`: Scoped `"event_merge_bundle"` Undo:** Duplicate event merges capture a scoped bundle restored cleanly by `undo_transaction`.
- **`ADR-018`: Read-Time History Enrichment, Shared Overlay Stack & Non-Radio Conflict Default:** Enriches history DTOs on read without mutating `live-store.json`, unifies `Dialog`/`Drawer` focus isolation via `useModalOverlay`, defaults `ConflictResolutionModal` out of radio mode, and gates profile History via `can_edit`.

## 6. Gotchas, Limitations & Open Debt
- See **`GOT-023`** (GID stripping/dedup, non-empty `diffs`, empty `200 OK` feeds, strict `isOfflineFallbackError`), **`GOT-024`** (`formatTxLabel`/`formatUndoneTxRef`, `GenDate` formatting, `[...]` vs `(...)` pedigree delimiters, conflict retry/state reset), and **`GOT-025`** (`overlay-stack.ts` release-`inert`-before-focus, `BottomNav` admin gating, dark `color-scheme`).
- Undoing a parent deletion re-binds the restored parent to existing family container nodes; note MCP `undo_transaction` `(current_year, age_threshold)` arg order (`GOT-021`).
- **Tracked Technical Debt ([docs/technical-debt.md](../../technical-debt.md)):**
  - `TD-HIST-01`: Face-tag and media-link routes in `backend/src/routes/rbac/media_tags_links.rs` do not yet call `log_change` or `verify_optimistic_version`.
  - `TD-HIST-02`: `DeletedPersonSnapshot` (`person_delete_families.rs`) omits stripped `face_tags`/`media_links`, `undo.rs` lacks subsequent-edit conflict checks on single-entry revert, and Places/Families pages do not yet wire `RecordHistoryDrawer`.
  - `TD-HIST-03`: `prune_expired_history_and_snapshots` (`snapshots.rs`) is implemented and tested but not yet scheduled inside `scheduler.rs`.

_Last curated: 2026-11-14 (Change History Visualization, read-time enrichment & localization)_
