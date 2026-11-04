# Person & Family Management — Implementation Memory

**Spec:** [docs/features/07-person-family-management.md](../../features/07-person-family-management.md) · **Status:** implemented

## 1. Summary
Core person, family relationship, and life-event management. Provides rich profile pages, chronological biographical timelines, flexible life events (birth, marriage, divorce, occupation, residence, military service, death), notes, free-text event sources, Slavic patronymic derivation, and 1-click Quick-Add actions for parents, spouses, and children.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261026000005_core_genealogy_model.sql` | `persons`, `names`, `families`, `family_children`, `events`, `event_sources`, `notes` |
| Routes | `backend/src/routes/rbac.rs` | Person/family/event CRUD, `get_person_profile`, `quick_add_relative`, `delete_impact` |
| Profile Page | `frontend/src/pages/PersonProfilePage.tsx` | Main profile view, facts tab, relatives sidebar, timeline tab, media gallery |
| Profile Components | `frontend/src/components/profile/` | `ProfileHeroHeader.tsx`, biography facts, timeline entries |
| Edit Dialog | `frontend/src/components/person/` | `PersonEditDialog.tsx` (names, gender, life events, patronymic auto-fill) |
| Quick-Add | `frontend/src/components/person/` | Quick modals for adding parent, partner, sibling, or child with reciprocal relations |
| Tests | `backend/tests/person_profile_test.rs`| Full profile queries, timeline ordering, life event creation |
| Frontend Tests | `frontend/tests/components/` | `person-profile.test.tsx`, `person-and-places.test.tsx` |

## 3. Key Flows
1. **Quick-Add Relative:**
   - On a person's profile, Editor clicks "Add Child".
   - `quick_add_relative` handler atomically creates the new `Person` record, verifies or creates the `Family` relationship, and binds parent-child linkages in a single database transaction.
2. **Delete Impact Assessment:**
   - Before deleting an individual, UI queries `GET /api/v1/persons/{id}/delete-impact`.
   - Backend analyzes graph: reports how many marriages, child relationships, and media tags would be orphaned or dissolved.
   - Deletion performs soft delete with restore support (`restore_deleted_person`).

## 4. Invariants & Business Rules
- **Optimistic Concurrency:** All person and family updates must supply the current `version`. Conflicts return `409 Conflict`.
- **Chronological Timeline Sorting:** Events are ordered by `GenDate` with approximate and partial dates sorted before exact dates in the same year.

## 5. Decisions
- **Transactional Quick-Add:** Always create family container rows automatically during Quick-Add to guarantee consistent relationship graphs without orphaned child pointers.

## 6. Gotchas & Known Limitations
- When deleting a spouse from a family with children, the family container must be retained to maintain parental links for the children.

_Last curated: 2026-11-04 (memory bank bootstrap)_
