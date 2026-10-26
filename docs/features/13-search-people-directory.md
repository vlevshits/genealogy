# Feature Specification: Search & People Directory

## 1. Overview & Objective
The tree canvas is great for browsing, but it cannot answer *"show me everyone named Petrov born before 1900"* or *"who did I forget to add a birth date for?"*. The app provides:

1. **Global Search** — one search engine powering the `Cmd+K` omnibox and the search page.
2. **People Directory** (`/people`) — a fast, sortable, filterable table of all individuals.
3. **Surnames Index** — a browsable list of family names with counts.


---

## 2. Search Engine

### 2.1 What Is Searchable
| Entity | Fields |
| :--- | :--- |
| Individuals | given name, patronymic, surname, birth surname, nickname, alt names, `gid`, birth year |
| Places | name, full name, alt names |
| Media | title, description |

### 2.2 Normalization Pipeline (`search_text`)
Computed by the backend whenever a name changes, stored in `individuals.search_text`, indexed with `pg_trgm`:
1. Lowercase + `unaccent`.
2. **Russian specifics:** `ё → е`; pre-1918 orthography `ѣ → е`, `і → и`, `ѳ → ф`, `ѵ → и`, trailing `ъ` removed (*Левшицъ → левшиц*).
3. **Transliteration both ways:** each Cyrillic token is also stored in Latin (GOST 7.79 / BGN-like simplified: *Щеглов → shcheglov*), and each Latin token in Cyrillic, so `petrov` finds *Петров* and vice versa.
4. **Surname gender folding:** common Slavic feminine endings are folded to a stem token (`-ова/-ов`, `-ева/-ев`, `-ина/-ин`, `-ская/-ский`, `-цкая/-цкий`) so searching *Петров* also matches *Петрова*.

### 2.3 Query Parsing
- Free text: `иван петров 1890` → name tokens + numeric token treated as birth year ±2.
- `gid` exact match (`I0042`) jumps straight to that record.
- Ranking: exact surname/given match > prefix > trigram similarity; ties broken by number of relatives (well-connected people first) then birth year.

### 2.4 Performance
- Omnibox returns top 8 people + 3 places + 3 media in **<100 ms** for trees up to 50,000 individuals.
- Privacy filters (living/private) are applied **in SQL**, never post-filtered in the client — anonymous users must not be able to discover living people by name. Living people are matched for anonymous users **only** by surname and appear as *"Living Petrov"* without dates (configurable: `PUBLIC_SEARCH_SHOW_LIVING=false` hides them entirely).

---

## 3. Omnibox (`Cmd+K` / `Ctrl+K`)
- Grouped results: **People** (avatar, name with patronymic, life years, birthplace), **Places**, **Pages/Actions** (*"Add person"*, *"Import GEDCOM"*, *"Go to Settings"* — role-aware).
- Empty state: recent searches and recently viewed people (stored in `localStorage`).
- `Enter` opens profile; `Shift+Enter` focuses the person in the tree canvas.

---

## 4. People Directory (`/people`)

### 4.1 Layout
```
+------------------------------------------------------------------------------------+
| People (1,420)                    [🔍 Filter by name...]          [+ Add Person]   |
| Filters: [Surname ▼] [Born: 1800–1950] [Sex ▼] [Living ▼] [Has photo ▼] [Clear]    |
+------------------------------------------------------------------------------------+
| ☐ | Avatar | Name ▲                         | Born          | Died        | Place  |
| ☐ | (img)  | Петров Александр Иванович      | 14 Mar 1912   | 22 Oct 1989 | Тверь  |
| ☐ | (img)  | Петрова (Морозова) Елена В.    | abt 1915      | 1990        | Москва |
| ... virtualized rows ...                                                           |
+------------------------------------------------------------------------------------+
| [Selected: 2]  [Export selection to GEDCOM]  [Compare / Merge]                     |
+------------------------------------------------------------------------------------+
```

### 4.2 Capabilities
- **Columns:** Avatar, Name (formatted per locale), Born (date + place), Died (date + place), Sex, `gid` (hidden by default), Last modified (hidden by default). Column visibility persisted per user.
- **Sorting:** name (surname-first), birth, death, last modified.
- **Filters (simple, fixed set):** surname, birth-year range, sex, living/deceased, has photo, **"Incomplete"** (no birth date OR no parents) — the most useful research to-do filter.
- **Virtualized** scrolling with server-side cursor pagination (page size 100).
- **Bulk selection actions (Editor+):** *Compare / Merge* (exactly 2 selected → [15](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md)), *Export selection to GEDCOM*, *Mark private/public*.
- URL-encoded state (`/people?surname=Петров&born=1800-1900`) so filtered views can be shared with relatives.

### 4.3 Surnames Index (`/people/surnames`)
- Alphabetical list (Cyrillic and Latin sections) of surnames with person counts and year span (*"Петров — 34 people, 1802–2015"*).
- Folded gender forms grouped together (*Петров / Петрова*).
- Click → People Directory filtered by that surname.

---

## 5. API Endpoints
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/search?q=&types=person,place,media&limit=` | Global grouped search (omnibox). |
| `GET` | `/api/v1/people?q=&surname=&born_from=&born_to=&sex=&living=&has_photo=&incomplete=&sort=&cursor=` | People directory. |
| `GET` | `/api/v1/people/surnames` | Surname index with counts and year spans. |

---

## 6. Definition of Done (DoD)

- [ ] **Cross-Script Search:** "petrov" finds "Петров"; "Щеглов" finds "Shcheglov"; "Левшицъ" and "Левшиц" match each other; "ё"/"е" are equivalent.
- [ ] **Gender Folding:** Searching "Петров" returns both Петров and Петрова.
- [ ] **Year Hint:** "иван 1890" ranks Ivans born 1888–1892 first.
- [ ] **Speed:** Omnibox responds in <100 ms on a 50k-person dataset.
- [ ] **No Privacy Leak:** Anonymous search never returns living persons' given names, dates, or places.
- [ ] **Directory Scale:** People Directory scrolls smoothly through 50k rows with sorting and filtering.
- [ ] **Shareable Filters:** Filter state is reflected in the URL and restored on reload.
- [ ] **Incomplete Filter:** Shows people missing a birth date or both parents.
