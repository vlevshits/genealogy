# Implementation Report: Phase 1.8 — Omnibox Search, People Directory & Surnames Index

**Date:** 2026-10-27  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 8 of [`docs/roadmap.md`](../roadmap.md) — Fast Omnibox search modal (`Cmd+K` / `Ctrl+K`), People Directory (`/people`) with virtualized table, responsive mobile cards, full filter suite (including incomplete research records), floating bulk actions bar, Surnames index (`/people/surnames`) with bilingual alphabetical jump bar, pre-1918 Russian orthography normalization (`ѣ/і/ѳ/ъ/ё`), Slavic surname gender folding, bidirectional Cyrillic/Latin transliteration, birth year proximity ranking, and public-mode living privacy shielding ([`13-search-people-directory.md`](../features/13-search-people-directory.md), [`08-directory-pages.md`](../pages/08-directory-pages.md), [`02-rbac-access-control.md`](../features/02-rbac-access-control.md), [`00-internationalization-localization.md`](../features/00-internationalization-localization.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Linguistic Normalization & Search Engine (`backend/`)
- **Pre-1918 Russian Orthography & Phonetic Normalizer ([`backend/src/services/privacy.rs`](../../backend/src/services/privacy.rs)):**
  - Canonical token normalization mapping `ё -> е`, `ѣ -> е` (yat), `і -> и` (decimal i), `ѳ -> ф` (fita), `ѵ -> и` (izhitsa), and stripping trailing word-final `ъ` (yer).
  - Slavic surname gender-folding engine mapping feminine endings (`-ова`, `-ева`, `-ёва`, `-ина`, `-ына`, `-ская`, `-цкая`, `-ova`, `-eva`, `-ina`, `-skaya`) to canonical masculine forms (`-ов`, `-ев`, `-ин`, `-ский`, etc.).
  - Bidirectional transliteration between Cyrillic and Latin alphabets (`petrov` <-> `Петров`, `shcheglov` <-> `Щеглов`).
  - Extract birth year proximity hints (e.g., query `"иван 1890"` extracts name `"иван"` and ranks individuals born in `1888–1892` highest).
- **Global Search Endpoint (`GET /api/v1/search?q=&types=&limit=`):**
  - Instant grouped search results across:
    - `people`: name, birth/death year, place, avatar thumbnail, relevance score.
    - `places`: resolved places and geocoded locations.
    - `media`: photo and document titles/transcriptions.
    - `actions`: fast navigation shortcuts (`Add Person`, `Tree View`, `Settings`, etc.).
  - Responds in `<100ms` with exact timing metadata (`timing_ms`).
- **People Directory Endpoint (`GET /api/v1/people`):**
  - Supports comprehensive filters: `q`, `surname`, `born_from`, `born_to`, `sex`, `living`, `has_photo`, `incomplete` (missing birth date or parents), `sort`, `cursor`, and `limit`.
  - Cursor-based pagination with stable tie-breaker sorting.
- **Surnames Index Endpoint (`GET /api/v1/people/surnames?letter=&min_count=`):**
  - Aggregates gender-folded surname clusters with counts, year ranges (e.g., `Петров` -> `count: 3`, `1850–1995`, `forms: ["Петров", "Петрова"]`).
- **Access Control & Living Privacy Shield:**
  - In default private mode, unauthenticated search and directory requests yield `401 Unauthorized`.
  - When public mode is enabled, living individuals are masked or excluded from search results; given names and life details are never exposed to unauthenticated viewers.

### 1.2 Frontend Omnibox, Directory & Surnames Index (`frontend/`)
- **Global Omnibox Modal ([`OmniboxDialog.tsx`](../../frontend/src/components/search/OmniboxDialog.tsx) & [`OmniboxResultsList.tsx`](../../frontend/src/components/search/OmniboxResultsList.tsx)):**
  - Accessible dialog triggered anywhere via `Cmd+K` (macOS) / `Ctrl+K` (Windows/Linux) or header search button.
  - Debounced input with keyboard navigation (`ArrowUp`, `ArrowDown`, `Escape`).
  - `Enter` navigates to person profile (`/persons/:id`); `Shift+Enter` focuses person directly on interactive tree canvas (`/tree?focus=:id`).
  - Persistent recent search history and recently viewed people stored in `localStorage`.
- **People Directory Page ([`PeoplePage.tsx`](../../frontend/src/pages/PeoplePage.tsx)):**
  - Mounted at `/people` in `App.tsx`.
  - Virtualized row windowing table (`DirectoryTable.tsx`) with customizable column visibility (`loadColumnPrefs`, defaulting `showGid: false`).
  - Responsive mobile card list (`DirectoryCardList.tsx`) for narrow screens.
  - Full filter bar (`DirectoryFilterBar.tsx`): text search, surname filter, birth year range, sex dropdown, living toggle, photo toggle, incomplete research records toggle.
  - Bidirectional URL query synchronization: filter states serialize to URL params and restore accurately on reload or external link.
  - Floating bulk actions bar (`DirectoryBulkBar.tsx`) allowing multi-selection, CSV export, and pair comparison/merging.
- **Surnames Index Page ([`SurnamesPage.tsx`](../../frontend/src/pages/SurnamesPage.tsx)):**
  - Mounted at `/people/surnames` in `App.tsx`.
  - Alphabetical jump bar with dual Cyrillic (`А–Я`) and Latin (`A–Z`) scripts.
  - Surname count cards with year ranges and gender variants; clicking `View People` deep-links directly to `/people?surname=...`.
- **UI & i18n Quality:**
  - 100% EN/RU translation parity with zero icon glyphs or emojis in strings.
  - Zero raw HTML elements (`<button>`, `<input>`, `<select>`, `<textarea>`); all controls use design-system primitives.
  - No "Living" badge pill; deceased individuals denoted by subtle `†` cross.
  - Avatar initials handle parenthesized maiden names gracefully (`"Петрова (Морозова) Анна"` -> `"ПА"`).

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Architect Review** | `architect-reviewer` (Mode 2) | **APPROVE** | 0 blocker/major findings. Validated orthography normalization, RBAC shields, and API contracts. |
| **Backend Integration Tests** | `cargo test --test search_directory_test` | **PASS (3/3)** | Verified normalization, transliteration, gender folding, proximity ranking, GID lookup, filters, and privacy guards. |
| **Full Backend Test Suite** | `cargo test` | **PASS (37/37)** | Zero regressions across all backend suites. |
| **Clippy & Code Formatting** | `cargo fmt -- --check && cargo clippy -- -D warnings` | **PASS** | Zero linter warnings or style defects. |
| **Frontend Test Suite** | `npm test -- --run` | **PASS (78/78)** | 11/11 test files passed including `search-and-directory.test.tsx` and i18n parity. |
| **Production Build** | `npm run build` (`tsc --noEmit && vite build`) | **PASS** | Bundle compiled cleanly with zero TypeScript errors in 1.08s. |
| **Interactive QA / Browser** | `qa-engineer` (Playwright) | **PASS** | Omnibox shortcuts (`Cmd+K`, `Enter`, `Shift+Enter`), Directory filters & URL sync, Surnames jump bar, and mobile cards verified. |

---

## 3. Defects Identified & Resolved During Testing

| Defect ID | Severity | Description | Resolution | Status |
| :--- | :--- | :--- | :--- | :--- |
| **DEF-001** | Medium | Untranslated sex keys (`person.sexMale`, `person.sexFemale`, `person.sexUnknown`) in Directory table and filter bar. | Corrected callers in `DirectoryFilterBar.tsx` and `DirectoryTable.tsx` to use nested keys `person.sex.male`, `person.sex.female`, `person.sex.unknown`. | **Fixed & Verified** |
| **DEF-002** | Low | Avatar initials rendering `"П("` on maiden names in parentheses; `gid` column visible by default. | Updated `getInitials` in `avatar.tsx` to strip parenthetical maiden names before taking initials; updated `loadColumnPrefs` in `DirectoryTable.tsx` to default `showGid: false`. | **Fixed & Verified** |

---

## 4. Documentation & Roadmap Alignment

- Updated [`docs/technical-debt.md`](../technical-debt.md) with item `TD-008` (database-level trigram/FTS indexing for trees >50,000 individuals).
- Satisfies all Definition of Done criteria for [`docs/features/13-search-people-directory.md`](../features/13-search-people-directory.md) and [`docs/pages/08-directory-pages.md`](../pages/08-directory-pages.md).
