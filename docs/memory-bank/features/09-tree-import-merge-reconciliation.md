# Tree Import Merge & Reconciliation — Implementation Memory

**Spec:** [docs/features/09-tree-import-merge-reconciliation.md](../../features/09-tree-import-merge-reconciliation.md) · **Status:** partial

## 1. Summary
Provides candidate matching and side-by-side entity reconciliation for merging disparate branches or external imports into the family tree. The duplicate candidate matching engine and person merge execution are fully implemented, while full automated multi-tree branch merging is integrated into Data Quality workflows. 1-click snapshot rollbacks protect against erroneous merges.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Matching engine | `backend/src/services/quality/mod.rs` | Probabilistic record linkage, phonetic name matching, birth date proximity |
| Snapshot rollback | `backend/src/services/history/mod.rs` | `create_manual_snapshot`, `rollback_to_snapshot` |
| Merge handlers | `backend/src/routes/rbac/mod.rs` | `preview_merge_people_post`, `merge_duplicate_people`, `mark_pair_not_duplicate` |
| Merge Studio UI | `frontend/src/pages/PersonMergePage.tsx` | Side-by-side attribute comparison, radio selector per field |
| Quality UI | `frontend/src/pages/DataQualityPage.tsx` | Duplicate candidates tab with 1-click navigation to Merge Studio |
| Tests | `backend/tests/data_quality_places_test.rs`| Duplicate detection scoring, merge execution |
| Frontend Tests | `frontend/tests/components/` | `person-merge-and-quality.test.tsx` |

## 3. Key Flows
1. **Candidate Discovery & Preview:**
   - Background scanning or user request triggers candidate matching.
   - User reviews potential duplicates in `DataQualityPage`.
   - Clicks "Compare & Merge" → opens `PersonMergePage` with side-by-side attribute diffs.
2. **Atomic Merge Execution:**
   - Editor selects winning values for each conflicting field (names, dates, places).
   - Frontend calls `POST /api/v1/people/merge`.
   - Backend automatically creates a pre-merge `tree_snapshot` (`services::history`).
   - Repoints all family links, events, media face tags, and notes to the survivor ID, and deletes the duplicate record.
3. **Emergency Rollback:**
   - If an accidental merge occurs, Admin navigates to History → Snapshots and triggers 1-click rollback (`POST /api/v1/snapshots/{id}/rollback`).

## 4. Invariants & Business Rules
- **Automatic Pre-Merge Snapshot:** Any person or place merge operation must create an instant rollback snapshot before executing destructive mutations.
- **Structural Kinship Guards:** Merging two individuals is strictly prohibited if they are direct ancestors or descendants of each other (prevents biological paradox cycles).

## 5. Decisions
- **Manual Field Selection:** Never merge conflicting attributes automatically; always present explicit side-by-side radio choices to the Editor.

## 6. Gotchas & Known Limitations
- Merging spouses in the same family requires special handling to prevent self-marriage loops.

_Last curated: 2026-11-04 (memory bank bootstrap)_
