# Places & Historical Jurisdictions — Implementation Memory

**Spec:** [docs/features/12-places.md](../../features/12-places.md) · **Status:** implemented

## 1. Summary
Hierarchical place management modeling historical jurisdictions (e.g. Village → Parish → Uyezd → Guberniya). Provides an interactive place directory, place hierarchy picker for event editing, interactive geographical place map (Leaflet/SVG), place merge tool, and automated hierarchy resolution integrated with Data Quality rules.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/` | `places` table, `parent_place_id`, `latitude`, `longitude`, `jurisdiction_level` |
| Routes | `backend/src/routes/rbac/mod.rs` | Places CRUD, `/places/map`, `/places/{id}/merge`, hierarchy analysis |
| Quality Service | `backend/src/services/quality/mod.rs` | Auto-detection of broken place hierarchies and parent restructuring fixes |
| Places Page | `frontend/src/pages/PlacesPage.tsx` | Places list/tree view, search, map toggle, place details panel |
| Components | `frontend/src/components/places/` | Place hierarchy tree, place edit modal, place map view |
| Picker | `frontend/src/components/domain/` | `PlacePicker.tsx` (autocomplete dropdown with breadcrumb hierarchy path) |
| Tests | `backend/tests/data_quality_places_test.rs`| Hierarchy loop detection, place merge, coordinate validation |
| Frontend Tests | `frontend/tests/components/` | `person-and-places.test.tsx` |

## 3. Key Flows
1. **Event Place Selection:**
   - In `PersonEditDialog`, user types place name into `PlacePicker`.
   - Autocomplete displays hierarchical path: `"дер. Ивановка, Минский уезд, Минская губерния"`.
   - Selecting the place stores the foreign key reference in `events.place_id`.
2. **Place Merging:**
   - When duplicate places exist (e.g. spelling variants or different languages), Editor triggers `POST /api/v1/places/{id}/merge`.
   - Pre-merge snapshot taken; all event references are repointed to the target place, and the duplicate place is removed.
3. **Automated Hierarchy Restructuring:**
   - Data Quality scans find orphaned villages; Admin triggers 1-click batch fix to attach them to appropriate district parents.

## 4. Invariants & Business Rules
- **No Cyclic Hierarchies:** A place cannot be set as a parent or ancestor of itself.
- **Geocoding Privacy:** Coordinates are cached locally in the PostgreSQL database; no third-party tracking calls are made during client browsing.

## 5. Decisions
- **Full Hierarchical Path in Breadcrumbs:** Always format places with full ancestral jurisdiction context to disambiguate identical village names.

## 6. Gotchas & Known Limitations
- When deleting a parent place that has child jurisdictions, child places must either be promoted or reassigned to avoid orphan records.

_Last curated: 2026-11-04 (memory bank bootstrap)_
