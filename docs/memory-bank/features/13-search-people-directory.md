# Search & People Directory — Implementation Memory

**Spec:** [docs/features/13-search-people-directory.md](../../features/13-search-people-directory.md) · **Status:** implemented

## 1. Summary
Global omnibox search, dedicated People Directory, and alphabetical Surname Index. Powered by PostgreSQL `pg_trgm` (trigram matching) and `unaccent` extensions for fuzzy, accent-insensitive, and cross-alphabet searching across Cyrillic and Latin name variants. Features responsive virtualized tables and cursor-based pagination.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/` | Trigram GIN indexes on `names.given_name`, `names.surname`, and `places.name` |
| Routes | `backend/src/routes/rbac/mod.rs` | `global_search`, `list_people_directory`, `list_people_surnames` |
| Omnibox | `frontend/src/components/search/` | Global search modal in navigation bar with keyboard shortcut (`Cmd+K` / `Ctrl+K`) |
| People Page | `frontend/src/pages/PeoplePage.tsx` | Virtualized table/cards, sorting by birth date, gender filters |
| Surnames Page | `frontend/src/pages/SurnamesPage.tsx` | Alphabetical surname frequency index and person drilldown |
| Directory UI | `frontend/src/components/directory/` | People filters, pagination controls, living status badges |
| Tests | `backend/tests/search_directory_test.rs` | Cyrillic search, partial matching, accent insensitivity |
| Frontend Tests | `frontend/tests/components/` | `search-and-directory.test.tsx` |

## 3. Key Flows
1. **Omnibox Quick Search:**
   - User hits `Cmd+K` anywhere in the app → opens Search Modal.
   - Types 2+ characters → debounced request to `GET /api/v1/search?q=...`.
   - Results group matched individuals, places, and media items with privacy redaction applied.
2. **Surname Frequency Drilldown:**
   - User navigates to `/people/surnames` → views aggregated surname cards with frequency counts.
   - Clicking a surname navigates to People Directory pre-filtered by that surname.

## 4. Invariants & Business Rules
- **Privacy in Search Results:** Unauthenticated search results never display exact birth dates or contact details of living relatives (`ADR-005`).
- **Slavic & Latin Transliteration:** Trigram indexes match partial and accented Slavic names (e.g. `Шереметев` matches `Шереметьев`).

## 5. Decisions
- **Cursor-Based Directory Pagination:** Standard `{ items, next_cursor, total }` pagination prevents offset degradation on large trees.

## 6. Gotchas & Known Limitations
- Cyrillic case-insensitive matching requires PostgreSQL `ILIKE` or `lower()` expression indexes alongside `unaccent`.

_Last curated: 2026-11-04 (memory bank bootstrap)_
