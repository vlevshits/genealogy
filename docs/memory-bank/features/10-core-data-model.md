# Core Data Model — Implementation Memory

**Spec:** [docs/features/10-core-data-model.md](../../features/10-core-data-model.md) · **Status:** implemented

## 1. Summary
The canonical relational foundation of the Genealogy application. Models individuals (`persons`), name variants (`names`), partnerships (`families`), parent-child links (`family_children`), chronological milestones (`events`), historical jurisdictions (`places`), photo/document attachments (`media`), and text annotations (`notes`). Features structured genealogical dates (`GenDate`) and human-friendly IDs (`gid`).

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261026000005_core_genealogy_model.sql` | Primary relational tables, foreign key constraints, indexes |
| Domain Model | `backend/src/models/gendate.rs` | `GenDate`, `DateModifier` (Exact, About, Before, After, Between), dual dating |
| Domain Root | `backend/src/models/mod.rs` | Structs for `Person`, `Name`, `Family`, `Event`, `Place`, `Media`, `Note` |
| Frontend Types | `frontend/src/types/` | `person.ts`, `place.ts`, `event.ts`, `gendate.ts`, `common.ts` |
| Frontend Parser | `frontend/src/lib/gendate-parser.ts` | `parseGenDate`, `formatGenDate`, validation regexes |
| Tests | `backend/tests/core_data_model_test.rs` | Integrity constraints, cascading behaviors, ID uniqueness |
| Date Tests | `backend/tests/gendate_fixtures_test.rs` | Parsing complex date strings (`"ABT 1880"`, `"BET 1914 AND 1918"`) |
| Parser Tests | `frontend/tests/parser/` | `gendate-parser.test.ts` |

## 3. Key Flows
1. **GenDate Parsing & Storage:**
   - User inputs `"Около 1905"` or `"ABT 1905"`.
   - `gendate-parser.ts` parses input into JSON `{ modifier: "about", year: 1905 }`.
   - Backend persists the JSONB representation and computes an approximate integer timestamp for chronological sorting.
2. **Dual Identifier Lookup:**
   - Client queries `/persons/I0001` or `/persons/<uuid>` → Backend evaluates string format and queries either `gid` or `id` (`ADR-006`).

## 4. Invariants & Business Rules
- **Stable Alphanumeric IDs (`gid`):** Every person receives an immutable, human-readable identifier (e.g. `I0001`, `I0002`) and every family receives `F0001`.
- **Soft Deletion & Versioning:** Entities carry `is_deleted BOOLEAN DEFAULT FALSE` and `version INT DEFAULT 1` for undo and concurrency protection.
- **Relational Integrity:** A person cannot be added as their own parent, child, or spouse.

## 5. Decisions
- **`ADR-006`: Dual Identifier Resolution:** Routes accept both UUID and `gid` seamlessly.
- **JSONB Genealogical Dates:** Store structured date components in JSONB rather than simple SQL `DATE` to preserve historical uncertainty (ranges, circa).

## 6. Gotchas & Known Limitations
- SQL queries sorting by event dates must sort by the normalized `sort_year` column rather than parsing JSONB dates inside the query.

_Last curated: 2026-11-04 (memory bank bootstrap)_
