# Feature Specification: Home Dashboard, Empty-Tree Onboarding & Anniversaries

## 1. Overview & Objective
Defines:
- what a brand-new instance shows when the tree is **empty**,
- the user's **home person** (the default focal point and the anchor for "Your great-grandfather"),
- a lightweight **home page** that gives family members a reason to come back (birthdays, recent additions) — 4 fixed widgets.

---

## 2. Home Person

- Each user has `users.home_person_id` (defaults to `users.person_id`, the person they are linked to).
- Instance default: `instance_settings.default_home_person_id` — used for anonymous visitors and users without a home person.
- Used for: initial tree focal person, kinship pill on profiles ([03 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/03-person-profile-page.md#31-hero-identity-header)), "Relationship to me" in kinship finder, anniversaries "relatives only" filter.
- Set via profile `⋯ → Set as my home person` or *Settings → My Profile*. 🏠 icon on the tree card.
- If no home person is resolvable, the kinship pill is hidden (not an error).

---

## 3. Empty-Tree Onboarding (first run after `/setup`)

When `individuals` is empty, `/` and `/tree` show a centered welcome card:

```
+--------------------------------------------------------------+
|                 🌳  Let's start your family tree              |
|                                                              |
|   [ 👤 Start with yourself ]     [ 📥 Import a GEDCOM file ]  |
|                                                              |
|   Coming from MyHeritage, Ancestry, Geni or another app?     |
|   Export a GEDCOM (.ged) there and import it here.           |
+--------------------------------------------------------------+
```
- **Start with yourself:** short form (given name, patronymic, surname, sex, birth date, birth place). Creates the person, links it to the admin user (`person_id` and `home_person_id`), and opens the tree with pulsing `+` hints for *Add father / Add mother*.
- **Import:** opens the GEDCOM import wizard ([03](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/03-gedcom-import-export.md)); after import the user is asked *"Which person are you?"* (search picker, skippable).
- Viewers/anonymous users on an empty tree see: *"This family tree is still being prepared."*
- **Zero Marketing & Jargon Policy:** Onboarding cards and welcome prompts must remain quiet, welcoming, and strictly focused on family history. Promotional SaaS slogans (e.g. *"Next-gen platform"*, *"Unlock the power of your lineage"*), tech stack brags, or developer status widgets are strictly forbidden on user-facing onboarding and dashboard screens per [00 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md#3-ui-copywriting-tone-of-voice--anti-fluff-standards).

---

## 4. Home Dashboard (`/`)

Default landing page for authenticated users once the tree has data (`/tree` remains one click away; users can choose *"Open tree on start"* in their profile settings). Anonymous visitors land on `/tree`.

```
+------------------------------------------------------------------------------------+
| Good evening, Lev                                      [Open Tree →] [+ Add Person]|
+-----------------------------------+------------------------------------------------+
| 🎂 THIS MONTH                      | 🕓 RECENTLY ADDED & CHANGED                    |
| Today   · Vera Petrova turns 34   | Vera added Пётр Щеглов — 2h ago                |
| Nov 2   · Ivan Levshits (1931†)   | Lev uploaded 6 photos — yesterday              |
|          would be 95 — birthday   | Lev edited Елена Морозова — 2 days ago         |
| Nov 7   · Wedding anniversary     |                              [All changes →]   |
|          A. & E. Petrov — 90 yrs  |                                                |
+-----------------------------------+------------------------------------------------+
| 📊 TREE AT A GLANCE                | 🖼 LATEST PHOTOS                                |
| 1,420 people · 512 families       | [thumb][thumb][thumb][thumb][thumb][thumb]     |
| 340 photos · 12 generations       |                                                |
| Oldest ancestor: b. abt 1702      |                                                |
| 48 people missing birth dates →   |                                                |
+-----------------------------------+------------------------------------------------+
```

### 4.1 Widgets (fixed set)
1. **Anniversaries (This month / next 30 days)**
   - Birthdays of **living** people (only for users allowed to see them).
   - Memorial birthdays and death anniversaries of deceased relatives at round numbers (every 5 / 10 years: *"would be 100"*, *"died 80 years ago"*) — keeps the list short.
   - Wedding anniversaries of couples where both partners are living, or round-number for deceased.
   - Toggle: *All people* / *Close relatives* (within 4 degrees of home person).
   - Only exact day+month dates qualify; Julian dates converted to Gregorian.
2. **Recently added & changed** — latest 5 transactions from [14](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md) (Editors/Admins); Viewers see *"Recently added people"* (create ops only, privacy-filtered).
3. **Tree at a glance** — counts (people, families, photos, places), generation depth from oldest ancestor to youngest descendant, oldest known birth, plus one data-quality nudge link for Editors ([15](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md)).
4. **Latest photos** — 6 most recent media thumbnails (privacy-filtered) → `/media`.


### 4.2 Optional Email Digest (off by default)
- If SMTP is configured, users can opt in (*Settings → My Profile*) to a **monthly email**: upcoming anniversaries + count of new people/photos added. Localized (en/ru). Sent by the scheduled job runner.

---

## 5. API Endpoints
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/dashboard` | All widget data in one privacy-filtered payload. |
| `GET` | `/api/v1/anniversaries?from=&to=&scope=all|close` | Anniversary list. |
| `GET` | `/api/v1/stats` | Tree statistics (cached 5 min). |
| `PUT` | `/api/v1/me/home-person` | `{ person_id }`. |
| `PATCH` | `/api/v1/me` | Preferences: `locale`, `theme`, `start_page`, `digest_opt_in`. |

---

## 6. Definition of Done (DoD)

- [ ] **Empty-Tree Welcome:** A fresh instance never shows an empty canvas; the admin can create themselves or import in one click.
- [ ] **"Which person are you?":** After first import, the admin can link themselves to a person and it becomes their home person.
- [ ] **Home Person:** The tree opens focused on the user's home person; kinship pills are relative to it.
- [ ] **Anniversaries:** Upcoming birthdays/anniversaries list respects living-person privacy and only uses exact day/month dates.
- [ ] **Single Request:** Dashboard loads in one API call in <200 ms on a 10k-person tree.
- [ ] **Role-Aware Widgets:** Viewers never see edit history details or private records.
- [ ] **Anti-Marketing Copy:** All onboarding cards, welcome prompts, and dashboard widgets adhere to clean, functional, family-centered copy without promotional buzzwords or tech stack mentions.
