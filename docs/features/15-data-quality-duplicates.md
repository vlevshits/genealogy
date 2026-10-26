# Feature Specification: Data Quality Checks & In-Tree Person Merge

## 1. Overview & Objective
Family trees accumulate mistakes: typos in years, duplicates after a relative adds the same grandmother twice, impossible ages. The app provides:

1. **Inline soft warnings** while editing (never blocking).
2. A small **Data Quality** page with a fixed set of the most useful checks.
3. **Merge two people** already in the tree (merging an *imported* tree is covered by [09](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/09-tree-import-merge-reconciliation.md)).

---

## 2. Validation Rules

### 2.1 Hard Errors (block save)
Defined in [10-core-data-model.md §6.1](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#61-hard-constraints-save-is-rejected): ancestor cycles, self-partnering, malformed events.

### 2.2 Soft Warnings (shown inline + on Data Quality page)
Fixed set; thresholds configurable via env:

| Code | Rule | Default threshold |
| :--- | :--- | :--- |
| `DEATH_BEFORE_BIRTH` | Death/burial before birth | — |
| `TOO_OLD` | Age at death or (if living) current age above limit | 110 |
| `PARENT_TOO_YOUNG` | Parent younger than N at child's birth | 13 |
| `MOTHER_TOO_OLD` | Mother older than N at child's birth | 55 |
| `BORN_AFTER_PARENT_DEATH` | Child born >1 year after father's death or after mother's death | — |
| `MARRIED_TOO_YOUNG` | Partner younger than N at marriage | 14 |
| `EVENT_AFTER_DEATH` | Non-burial event after death | — |
| `POSSIBLE_DUPLICATE` | Duplicate candidate score ≥ 80% (see §3.1) | 80 |
| `MISSING_BIRTH` | Deceased person with no birth date and no death date | — *(info level)* |
| `UNCONNECTED` | Person with no family links at all | — *(info level)* |

- Approximate dates use their widest plausible range — `abt 1880` never triggers a warning that a ±5 year window would avoid.
- Each warning can be **dismissed per record** ("This is correct") and stays dismissed (`dismissed_warnings(entity_id, code, user_id)`).

### 2.3 Inline Presentation
- In edit forms, a yellow hint under the field: *"Mother would be 9 years old at this birth. Check the date?"* — save still allowed.
- On profiles, a discreet ⚠️ badge in the header opens the list of warnings (Admins only).

---

## 3. Duplicate Detection & Person Merge

### 3.1 Finding Duplicates
- Reuses the matching engine from [09 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/09-tree-import-merge-reconciliation.md#3-candidate-matching-algorithm-record-linkage) (names incl. transliteration/gender folding, dates, relatives), but **within** the live tree.
- Runs as a background job (nightly + on demand); results cached in `duplicate_candidates(person_a, person_b, score, computed_at)`.
- **Real-time check when adding a person:** the quick-add drawer shows *"Possible match: Пётр Щеглов (1868–1931) — Use existing?"* if score ≥ 80%, which prevents most duplicates at the source.

### 3.2 Merge Flow (`/people/merge?a=…&b=…`, Admin only)
Entry points: Data Quality page, People Directory (select 2 → *Compare / Merge*, Admins only), profile `⋯ → Merge with…` (Admins only).

```
+-----------------------------------------------------------------------------------+
| Merge People                                                                      |
+----------------------+------------------------------+-----------------------------+
| Field                | Пётр Щеглов (I0012)          | Петр Щеглов (I0388)         |
+----------------------+------------------------------+-----------------------------+
| Name                 | (•) Пётр Фёдорович Щеглов    | ( ) Петр Щеглов  → alt name |
| Birth                | ( ) abt 1868                 | (•) 3 Feb 1868, Тверь       |
| Death                | (•) 1931                     | ( ) —                       |
| Events               | 4  ─────── all kept, exact duplicates collapsed ─────── 2  |
| Parents / Families   | 1 family                     | 1 family → merged if same   |
| Photos / Notes       | 3 / 2                        | 1 / 1  → all kept           |
+----------------------+------------------------------+-----------------------------+
| Result keeps ID I0012.   [ Cancel ]                         [ Merge people ]      |
+-----------------------------------------------------------------------------------+
```

Merge rules:
- Scalar fields: user picks per field; the non-chosen name is kept as an **alternate name** automatically.
- Events, notes, media links, face tags: union; exact duplicates (same type + date + place) collapsed.
- Families: if both are children in families with the same partners → families merged; if both are partners in families with the same other partner → families merged (children unioned). Otherwise families are re-pointed to the surviving person.
- `users.person_id` and `home_person_id` re-pointed.
- The losing `gid` is recorded as an alias so old links/bookmarks redirect.
- Executed in one transaction, logged as one `change_log` transaction → fully undoable ([14](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md)).
- Cycle check runs on the merged result; merge is blocked if it would create an ancestor loop.

Family merge (two duplicate families with the same partners) uses the same UI in simplified form. Place merge is a simple "merge into" action ([12](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/12-places.md)).

---

## 4. Data Quality Page (`/quality`, Admin only)
The Data Quality page, its soft warning reviews, and candidate duplicate queues are restricted strictly to **Administrators** (`RequireRole::<Admin>`). Viewers and Editors attempting to access `/quality` receive `403 Forbidden`.
- Summary chips: *"12 warnings · 5 possible duplicates · 48 people missing birth dates · 3 unconnected"*.
- Tabs: **Warnings**, **Possible duplicates** (pairs with score, *Compare* button, *Not a duplicate* button), **Incomplete** (shortcut to People Directory filter).
- Each row links to the record; fixing it removes the row on the next recompute (immediately for the edited person).

---

## 5. API Endpoints
| Method | Endpoint | Access Level | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/v1/quality/summary` | Admin only | Counts per category. |
| `GET` | `/api/v1/quality/warnings?code=&cursor=` | Admin only | Warning list. |
| `POST` | `/api/v1/quality/warnings/dismiss` | Admin only | `{ entity_id, code }`. |
| `GET` | `/api/v1/quality/duplicates` | Admin only | Candidate pairs. |
| `POST` | `/api/v1/quality/duplicates/not-duplicate` | Admin only | `{ person_a, person_b }`. |
| `GET` | `/api/v1/people/match?given=&surname=&birth=` | Editors & Admins | Real-time duplicate check for quick-add. |
| `GET` | `/api/v1/people/merge/preview?a=&b=` | Admin only | Field-by-field comparison payload. |
| `POST` | `/api/v1/people/merge` | Admin only | `{ keep_id, merge_id, field_choices }`. |

---

## 6. Definition of Done (DoD)

- [ ] **Administrator-Only Isolation:** Access to `/quality`, `/people/merge`, and the underlying endpoints (`/api/v1/quality/*`, `/api/v1/people/merge*`) is strictly restricted to Administrators (`RequireRole::<Admin>`); Viewers and Editors receive `403 Forbidden`.
- [ ] **Non-Blocking Warnings:** Implausible dates show inline hints but never prevent saving.
- [ ] **Fuzzy-Date Aware:** `abt`/`bet` dates do not produce false warnings within their range.
- [ ] **Dismissal:** A dismissed warning stays dismissed for that record.
- [ ] **Add-Time Duplicate Prompt:** Quick-add suggests an existing matching person before creating a new one.
- [ ] **Person Merge:** Merging two people unions events/media/notes, merges identical families, and re-points user links.
- [ ] **Undoable Merge:** A person merge can be fully undone from History.
- [ ] **Not-a-Duplicate:** Marked pairs never reappear as candidates.
