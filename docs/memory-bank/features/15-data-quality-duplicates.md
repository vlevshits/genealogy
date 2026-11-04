# Data Quality & Duplicates — Implementation Memory

**Spec:** [docs/features/15-data-quality-duplicates.md](../../features/15-data-quality-duplicates.md) · **Status:** implemented

## 1. Summary
Automated tree health analysis, soft validation warnings, relationship-centric duplicate detection, and in-tree entity merging. Scans genealogical records for biological impossibilities (mother too young/old, child born after mother's death, self-marriages), identifies duplicate individuals using phonetic and kinship algorithms, and resolves broken place hierarchies.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/` | `duplicate_dismissals`, `quality_warning_dismissals` tables |
| Routes | `backend/src/routes/rbac/mod.rs` | Quality summary, warnings list/dismiss, duplicate candidates, place fixes |
| Service | `backend/src/services/quality/mod.rs` | Modular quality engine: rule evaluators, candidate scoring, kinship guards |
| Quality Page | `frontend/src/pages/DataQualityPage.tsx` | Overview cards, warnings tab, duplicate candidates tab, place issues tab |
| Merge Page | `frontend/src/pages/PersonMergePage.tsx` | Interactive side-by-side reconciliation interface |
| Components | `frontend/src/components/quality/` | Issue cards, dismissal controls, batch resolution triggers |
| Tests | `backend/tests/data_quality_places_test.rs`| Biological validation rules, duplicate matching precision |
| Frontend Tests | `frontend/tests/components/` | `person-merge-and-quality.test.tsx` |

## 3. Key Flows
1. **Quality Rule Evaluation:**
   - Background scanner runs rules: `MOTHER_TOO_YOUNG` (<12y), `FATHER_TOO_YOUNG` (<14y), `BORN_AFTER_MOTHER_DEATH`, `MARRIAGE_BEFORE_BIRTH`.
   - Generates soft warnings displayed with severity levels (Warning vs Error) in `DataQualityPage`.
2. **Duplicate Pair Linkage & Resolution:**
   - Scoring algorithm evaluates: name string distance (Jaro-Winkler/Trigram), birth date proximity, shared parents, and shared spouses.
   - Pairs scoring > 70% threshold surface in Duplicate Candidates.
   - Editor either clicks "Not a Duplicate" (persists to `duplicate_dismissals`) or clicks "Merge" to open `PersonMergePage`.

## 4. Invariants & Business Rules
- **Non-Blocking Soft Warnings:** Biological warnings do not block form submissions, accommodating historical uncertainty and edge cases.
- **Structural Kinship Guards:** Cannot merge two people who are direct ancestors/descendants of one another.

## 5. Decisions
- **Persistent Dismissals:** Editors can permanently dismiss warnings as "false positive" or "known historical exception" with a reason note.

## 6. Gotchas & Known Limitations
- `services/quality.rs` is 5,279 lines long (see `codebase-map.md` Hotspots) and is a top candidate for modular decomposition.

_Last curated: 2026-11-04 (memory bank bootstrap)_
