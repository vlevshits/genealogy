# Person & Family Management — Implementation Memory

**Spec:** [docs/features/07-person-family-management.md](../../features/07-person-family-management.md) · **Status:** implemented

## 1. Summary
Core person, family relationship, and life-event management. Provides rich profile pages, chronological biographical timelines, flexible life events (birth, marriage, divorce, occupation, residence, military service, death), notes, free-text event sources, Slavic patronymic derivation, and 1-click Quick-Add actions for parents, spouses, and children.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/20261026000005_core_genealogy_model.sql` | `persons`, `names`, `families`, `family_children`, `events`, `event_sources`, `notes` |
| Routes | `backend/src/routes/rbac/mod.rs`, `person_delete_families.rs` (person/family CRUD, `DELETE /families/{id}`, delete impact), `quick_add.rs` (`quick_add_relative`), `events.rs` (`delete_event`), `family_moves.rs` (link editing) | Spread across files; routes registered in `backend/src/routes/mod.rs` |
| Family link editing | `backend/src/routes/rbac/family_moves.rs` | `move_person_to_family`, `detach_family_child`, `detach_family_partner`, `is_orphaned_family`, `is_childless_union`, `dissolve_family`, `normalize_partner_slots`, `commit_family_edit`; errors `ChildNotFound`/`PartnerNotFound` in `backend/src/error.rs` |
| Link editing UI | `frontend/src/hooks/useFamilyLinkActions.ts`, `components/profile/ProfileFamilyTab.tsx`, `ProfileChangeParentsDialog.tsx`, `ParentUnlinkDialog.tsx`, `lib/family-move-errors.ts` | API: `detachFamilyChild`, `detachFamilyPartner`, `moveToFamily` in `frontend/src/api/genealogy-api.ts`; `onResponse` hook in `api/core.ts` reads `X-Transaction-Id`; i18n `profile.family.*` keys (`changeParentsBtn`, `unlinkParent*`) |
| Profile Page | `frontend/src/pages/PersonProfilePage.tsx` | Main profile view, facts tab, relatives sidebar, timeline tab, media gallery |
| Profile Components | `frontend/src/components/profile/` | `ProfileHeroHeader.tsx`, biography facts, timeline entries |
| Edit Dialog | `frontend/src/components/person/` | `PersonEditDialog.tsx` (names, gender, life events, patronymic auto-fill) |
| Quick-Add | `frontend/src/components/person/` | Quick modals for adding parent, partner, sibling, or child with reciprocal relations |
| Tests | `backend/tests/person_profile_test.rs`, `family_move_test.rs`, `family_detach_test.rs`, `family_link_removal_test.rs` | Profiles/timeline; move-to-family; detach child/partner; marriage-event and unlink dissolve |
| Frontend Tests | `frontend/tests/components/` | `person-profile.test.tsx`, `person-and-places.test.tsx`, `family-link-fixes.test.tsx` |

## 3. Key Flows
1. **Quick-Add Relative:**
   - On a person's profile, Editor clicks "Add Child".
   - `quick_add_relative` handler atomically creates the new `Person` record, verifies or creates the `Family` relationship, and binds parent-child linkages in a single database transaction.
2. **Delete Impact Assessment:**
   - Before deleting an individual, UI queries `GET /api/v1/persons/{id}/delete-impact`.
   - Backend analyzes graph: reports how many marriages, child relationships, and media tags would be orphaned or dissolved.
   - Deletion performs soft delete with restore support (`restore_deleted_person`).

3. **Change parents / Unlink:** Family tab opens `ProfileChangeParentsDialog` (`POST /persons/{id}/move-to-family` with `family_id` or `partner1_id`/`partner2_id`, optional `from_family_id`, `rel_to_partner1/2`; returns `MoveToFamilyResponse` with `tx_id`, `source_family_dissolved`) or `ParentUnlinkDialog`. Unlink calls `DELETE /families/{id}/children/{child_id}` or `DELETE /families/{id}/partners/{person_id}?version=` (`FamilyMutationResponse` `{tx_id, dissolved}`). Errors `child_not_found`, `partner_not_found`. All steps share one `tx_id` so history undo restores them.
4. **Quick-Add link existing:** picker in `QuickAddRelativeDialog.tsx` uses server search (`fetchPeopleDirectory`), not a prop.

## 4. Invariants & Business Rules
- **Optimistic Concurrency:** All person and family updates must supply the current `version`. Conflicts return `409 Conflict`.
- **Two dissolve rules (intentional):** marriage-event removal (`delete_event`) dissolves a childless union regardless of partner count (`is_childless_union`); unlink/detach/PATCH dissolve only when no children and <=1 partner (`is_orphaned_family`). Unions with children are kept (single-parent family, divorced union).
- Dissolve logs the pre-edit snapshot before removal (same `tx_id`); `normalize_partner_slots` keeps a lone partner in slot 1 — see GOT-016.
- `X-Transaction-Id` response header on `DELETE /families/{id}` and `DELETE /events/{id}`.
- **Chronological Timeline Sorting:** Events are ordered by `GenDate` with approximate and partial dates sorted before exact dates in the same year.

## 5. Decisions
- **Transactional Quick-Add:** Always create family container rows automatically during Quick-Add to guarantee consistent relationship graphs without orphaned child pointers.

- Family link editing: [ADR-015](../decisions.md).

## 6. Not built (P2)
Child relation editor, union type editor, reorder, divorce badge, half/step sibling labels, searchable existing-family picker (spec-only; see technical debt).

## 7. Gotchas & Known Limitations
- See [GOT-016](../gotchas.md).
- When deleting a spouse from a family with children, the family container must be retained to maintain parental links for the children.

_Last curated: 2026-11-13 (family relationship link editing P1)_
