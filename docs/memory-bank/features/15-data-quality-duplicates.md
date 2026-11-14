# Data Quality & Duplicates — Implementation Memory

**Spec:** [docs/features/15-data-quality-duplicates.md](../../features/15-data-quality-duplicates.md) · **Status:** implemented

## 1. Summary
Automated tree health analysis, soft validation warnings, relationship-centric duplicate person detection, duplicate life/family event clustering & merging, and in-tree entity merging. Scans genealogical records for biological impossibilities (mother too young/old, child born after mother's death, self-marriages) and duplicate life/union events (`DUPLICATE_EVENT`), proposes non-destructive merges preserving media/notes/sources, supports custom and conflict-free batch merges with 1-click Undo (`event_merge_bundle`), and resolves broken place hierarchies.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/` | `duplicate_dismissals`, `quality_warning_dismissals` tables |
| Quality & Merge Routes | `backend/src/routes/rbac/history_quality.rs` | Quality summary, warnings list/dismiss, duplicate candidates, `preview_merge_people_get`, `preview_merge_people_post`, `merge_duplicate_people`, place fixes |
| Duplicate Events Routes | `backend/src/routes/rbac/quality_events.rs`, `backend/src/routes/mod.rs`, `backend/src/routes/openapi.rs` | Admin-only `GET /api/v1/quality/events/duplicates` (`list_duplicate_events`), `POST /api/v1/quality/events/merge` (`merge_duplicate_events_handler`), `POST /api/v1/quality/events/batch-merge` (`batch_merge_duplicate_events_handler`) |
| Quality Service Core | `backend/src/services/quality/mod.rs` | `candidates.rs` (prefix-safe `DUPLICATE_EVENT:` / `dup_event:` dismissal parsing), `scoring.rs`, `warnings.rs` (Rule 9 `RULE_DUPLICATE_EVENT`), `merge.rs` (`preview_person_merge`, `execute_person_merge`), `types.rs` (`PersonMergeRequest`) |
| Duplicate Events Service | `backend/src/services/quality/events_duplicates*.rs` | `events_duplicates_types.rs` (`DuplicateEventGroupDto`, `MergeDuplicateEventsRequest`, `is_union_context`, `synthesize_group_proposal`, `is_event_group_dismissed`), `events_duplicates.rs` (`RULE_DUPLICATE_EVENT`, `compute_duplicate_event_groups`, `filter_duplicate_event_groups`), `events_duplicates_merge.rs` (`merge_duplicate_events_in_store`, `consolidate_union_families`, `build_event_merge_before_bundle`), `events_duplicates_batch.rs` (`batch_merge_duplicate_events_in_store`, `execute_mcp_list_duplicate_events`, `execute_mcp_merge_duplicate_events`) |
| Undo Service | `backend/src/services/history/undo.rs` | Restores scoped `"event_merge_bundle"` snapshots (`events_before`, `families_before`, `notes_before`, `media_before`, `persons_before`) |
| MCP Adapter | `backend/src/services/mcp/catalog.rs`, `backend/src/services/mcp/tools_mutation.rs` | `merge_duplicate_individuals`, Tier C `list_duplicate_events` & `merge_duplicate_events` |
| API Client | `frontend/src/api/delight-types.ts`, `frontend/src/api/delight-api.ts` | `DuplicateEventGroupDto`, `MergeDuplicateEventsRequest`, `BatchMergeDuplicateEventsResponse`, `qualityApi.{getDuplicateEvents, mergeDuplicateEvents, batchMergeDuplicateEvents}` |
| Quality & Merge Pages | `frontend/src/pages/DataQualityPage.tsx`, `frontend/src/pages/PersonMergePage.tsx` | Overview cards, `All`/`Critical`/`Warnings`/`Duplicates`/`Places` tabs, `handleReviewDuplicateEvent` scroll-and-highlight CTA, side-by-side person merge UI |
| Quality Components | `frontend/src/components/quality/` | `DuplicateEventsSubView.tsx` (`isOfflineFallbackError`, `buildDefaultDuplicateEventGroups`), `DuplicateEventCard.tsx`, `DuplicateEventCustomizeModal.tsx` (`buildCustomEventMergePayload`, `CustomEventMergeFormValues`), `default-quality-fixtures.ts`, `PlaceHierarchy*.tsx`, `index.ts` |
| i18n | `frontend/src/i18n/{en,ru}.json` | `quality.duplicateEvents.*`, `quality.rules.duplicateEvent`, `quality.duplicates.*` |
| Backend Tests | `backend/tests/data_quality_places_test.rs`, `backend/tests/data_quality_events_test.rs`, `backend/tests/story_intake_and_mcp_test.rs` | Biological rules, duplicate matching, 6 duplicate-events integration tests (incl. `FAMC` vital isolation, OR privacy, `event_merge_bundle` undo, MCP tools) |
| Frontend & E2E Tests | `frontend/tests/components/person-merge-and-quality.test.tsx`, `frontend/e2e/duplicate-events.spec.ts` | 18 unit/component tests; 16 E2E test specs (52 Desktop/Mobile Chromium runs) with `frontend/e2e/fixtures/quality-api-mock.ts` and `frontend/e2e/pages/quality.page.ts` |

## 3. Key Flows
1. **Quality Rule Evaluation:**
   - Background scanner runs biological rules (`MOTHER_TOO_YOUNG`, `FATHER_TOO_YOUNG`, `BORN_AFTER_MOTHER_DEATH`, `MARRIAGE_BEFORE_BIRTH`) and emits `DUPLICATE_EVENT` warnings (`RULE_DUPLICATE_EVENT`) on `/quality`.
   - Clicking `Review Duplicates` (`handleReviewDuplicateEvent`) on a `DUPLICATE_EVENT` warning switches to the `Duplicates` tab, matches the group (`g.id === w.entity_id` → `g.events.some(...)` → `person`/`partner`), scrolls, and highlights the card.
2. **Duplicate Person Pair Linkage & Resolution:**
   - Scoring evaluates name distance (Jaro-Winkler/Trigram), birth date proximity, shared parents, and shared spouses (> 70% threshold).
   - Editor marks "Not a Duplicate" (`duplicate_dismissals`) or opens `PersonMergePage` (`POST /api/v1/people/merge` → `execute_person_merge`, or MCP `merge_duplicate_individuals`), preserving `birth`/`death` overrides, logging an undoable `change_log` transaction, and calling `store.persist_if_production()`.
3. **Duplicate Event Clustering, Conflict Resolution, Batch Merge & Undo:**
   - `compute_duplicate_event_groups` clusters duplicate singleton individual vitals (`birth`, `baptism`, `death`, `burial`, `cremation`), compatible repeatable events (`dates_compatible && places_compatible`), and union events (`marriage`, `engagement`, `divorce`, `annulment`), producing `ProposedMergedEventDto` with `has_conflict` and `conflict_fields`.
   - Safe groups (`has_conflict === false`) highlight primary quick `Merge Events`; conflict groups mark differing fields (`Conflict` / `Расхождение`) and elevate `Customize` (`DuplicateEventCustomizeModal`) to `variant="primary"` with recorded-value chips and a one-click `Combine sources` chip.
   - Confirming invokes `POST /api/v1/quality/events/merge` (or `POST /api/v1/quality/events/batch-merge` for conflict-free groups via `Auto-Merge All`), unioning media/notes, OR-joining `is_private`, consolidating couple unions (`consolidate_union_families`) when `is_union_context` is true, and returning an undoable `tx_id` (`"event_merge_bundle"`) surfaced via a non-truncated Undo `Toast`. `Looks Correct` dismisses via `DUPLICATE_EVENT:{group_id}`.

## 4. Invariants & Business Rules
- **Non-Blocking Soft Warnings:** Biological warnings do not block form submissions, accommodating historical uncertainty and edge cases.
- **Structural Kinship Guards:** Cannot merge two people who are direct ancestors/descendants of one another.
- **Empty-Body REST RBAC Probe vs Partial Inputs:** `execute_person_merge` returns its legacy `200 OK` no-op (`"Duplicate person merge authorized for administrator"`) strictly when *both* `raw_keep` and `raw_merge` are empty (`backend/tests/rbac_test.rs`); partial missing IDs fail with `AppError::ValidationFailed`. MCP callers validate non-empty IDs upfront (`GOT-018`).
- **`is_union_context` Isolation & Privacy OR-Join:** Singleton vitals (`birth`, `baptism`, `death`, `burial`, `cremation`) carrying legacy `FAMC` `family_id` are never treated as union events during validation, execution, or `"event_merge_bundle"` Undo (`surviving.individual_id` stays intact); merged events preserve `is_private = keep || any(merged)`.
- **Union Consolidation Child Slot Remapping:** `consolidate_union_families` remaps `FamilyChildDto.rel_to_partner1` / `rel_to_partner2` by matching partner ID rather than slot index.
- **Explicit Metadata Clearing & Safe Offline Guard:** `buildCustomEventMergePayload` transmits cleared text fields as `""` and omits `place_id` unless matching a known event/proposed `place_full_name`; `isOfflineFallbackError` strictly restricts demo fallback to `status === 0 || code === 'network_error'` so HTTP `4xx`/`5xx` errors show error toasts.

## 5. Decisions
- [ADR-017](../decisions.md#adr-017-duplicate-events-detection-non-destructive-merge-union-consolidation--undo-bundle): Duplicate Events Detection, Non-Destructive Merge, Union Consolidation & Undo Bundle.
- **Persistent Dismissals:** Editors can permanently dismiss warnings (`DUPLICATE_EVENT:{group_id}` and biological rules) or pairs (`duplicate_dismissals`) across tree re-evaluations.

## 6. Gotchas & Known Limitations
- Dismissal keys are scoped by `rule_code`, `entity_type`, `entity_id`, and optional `secondary_id`; `DUPLICATE_EVENT` keys embed colons (`GOT-020`).
- [GOT-020](../gotchas.md#got-020-duplicate-event-colon-dismissal-keys-legacy-famc-vitals--union-child-slot-remapping): Prefix-based `DUPLICATE_EVENT:` parsing, legacy `FAMC` vital isolation via `is_union_context`, partner-ID child slot remapping, and `is_private` OR-join.
- [GOT-021](../gotchas.md#got-021-mcp-undo_transaction-parameter-order--custom-event-merge-field-clearing): MCP `undo_transaction` `(current_year, age_threshold)` parameter order and `buildCustomEventMergePayload` `""`/omitted-`place_id` serialization.
- [GOT-022](../gotchas.md#got-022-quality-frontend-guardrails--offline-fallback-icu-plurals-dialog-toast-exclusions--card-layouts): Strict offline fallback, non-nested `formatIcuPlural`, `Dialog` `inert` `[data-toast-region]` exclusion, `flex-wrap` card footers, and warning-to-group matching order.
- [GOT-018](../gotchas.md#got-018-mcp-merge_duplicate_individuals-parameter-aliases-vs-rest-empty-body-rbac-smoke-test): MCP `merge_duplicate_individuals` parameter aliases must be normalized and validated non-empty before calling `execute_person_merge`.
- **Deferred Mobile Dialog Sheet Polish (`TD-QUAL-01` / `UX-137`):** Bottom-sheet docking and safe-area bottom inset polish on `< 768px` viewports for `DuplicateEventCustomizeModal` (`Dialog`) is recorded in [technical-debt.md](../../technical-debt.md).

_Last curated: 2026-11-14 (Duplicate Events detection/merge/undo feature, ADR-017, GOT-020–GOT-022)_
