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
- Viewers on an empty tree see: *"This family tree is still being prepared."*
- **Default Clean Slate & Zero Dummy Data Rule:** Production deployments and newly initialized databases must never seed, insert, or fall back to dummy or mock individuals (e.g. "Nikolai Petrovich Sokolov", "Elena Alexandrovna Levshits", "Alexander Mikhailovich Levshits", "Pyotr Fyodorovich Shcheglov", etc.). When the `individuals` table is empty (0 records), the application must display the genuine `EmptyTreeOnboardingCard` rather than mock names or fake dashboard activity. When populated, dashboard widgets must display only authentic database entities, using clean empty states when lists are empty.
- **Zero Marketing & Jargon Policy:** Onboarding cards and welcome prompts must remain quiet, welcoming, and strictly focused on family history. Promotional SaaS slogans (e.g. *"Next-gen platform"*, *"Unlock the power of your lineage"*), tech stack brags, or developer status widgets are strictly forbidden on user-facing onboarding and dashboard screens per [00 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md#3-ui-copywriting-tone-of-voice--anti-fluff-standards).

---

## 4. Home Dashboard (`/`)

Default landing page for authenticated users once the tree has data (`/tree` remains one click away; users can choose *"Open tree on start"* in their profile settings).
- **Default Private Posture (`ENABLE_PUBLIC_ACCESS=false`):** Unauthenticated visitors are immediately redirected to `/login` ([02 §3.1](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md#31-tier-1-default-private-instance-login-first-gatekeeper)).
- **Public Guest Mode (`ENABLE_PUBLIC_ACCESS=true`, Unauthenticated):** Unauthenticated visitors can view the Home Dashboard in **Strictly Read-Only Mode**:
  - Greeting renders a calm family title (e.g. `[Tree Title] • Family Archive`) rather than personal user greeting.
  - The `[+ Add Person]` button is **strictly omitted**; only `[Open Tree →]` is rendered.
  - Absolutely zero mutation or authoring buttons are rendered anywhere across the dashboard.

```
+------------------------------------------------------------------------------------+
| Levshits Family Archive                                              [Open Tree →] | <- (Public Guest: No Add button)
+-----------------------------------+------------------------------------------------+
| 🎂 THIS MONTH                      | 🕓 RECENTLY ADDED & CHANGED                    |
| Today   · I. turns 34 (Living)    | Added Пётр Щеглов — 2h ago                     |
| Nov 2   · Ivan Levshits (1931†)   | Uploaded 6 photos — yesterday                  |
|          would be 95 — birthday   | Edited E. — 2 days ago                         |
| Nov 7   · Wedding anniversary     |                               (Read-only,      |
|          A. & E. Petrov — 90 yrs  |                                zero Undo CTAs) |
+-----------------------------------+------------------------------------------------+
| 📊 TREE AT A GLANCE                | 🖼 LATEST PHOTOS                                |
| 1,420 people · 512 families       | [thumb][thumb][thumb][thumb][thumb][thumb]     |
| 340 photos · 12 generations       |                                                |
| Oldest ancestor: b. abt 1702      | (Public deceased media only;                   |
| (Admin nudge link hidden)         |  clicking prompts login to view full archive)  |
+-----------------------------------+------------------------------------------------+
```

### 4.1 Widgets (fixed set)
1. **Anniversaries (This month / next 30 days)**
   - Birthdays of **living** people: visible only to authenticated users permitted to see them; in public guest mode, living names are masked to single initial with dot (`И.` / `J.`) or excluded.
   - Memorial birthdays and death anniversaries of deceased relatives at round numbers (every 5 / 10 years: *"would be 100"*, *"died 80 years ago"*) — keeps the list short.
   - Wedding anniversaries of couples where both partners are living (authenticated only), or round-number for deceased.
   - Toggle: *All people* / *Close relatives* (within 4 degrees of home person; hidden for public guests when no home person is established).
   - Only exact day+month dates qualify; Julian dates converted to Gregorian.
2. **Recently added & changed**
   - Shows latest 5 transactions from [14](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md).
   - Admins see full transactional audit feed with Undo buttons.
   - Editors and Viewers see *"Recently added people"* (create operations only, privacy-filtered).
   - Public guests see read-only additions with living persons masked to single initial; **strictly zero Undo buttons** and zero administrative diff links.
3. **Tree at a glance**
   - Aggregate counts (people, families, photos, places), generation depth from oldest ancestor to youngest descendant, oldest known birth.
   - Data-quality nudge link (`48 people missing birth dates →`) is visible **strictly to Administrators** ([15](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md)); hidden for Viewers, Editors, and public guests.
4. **Latest photos**
   - 6 most recent media thumbnails (privacy-filtered).
   - Public guests see photos of deceased ancestors only; photos of living persons are excluded.
   - Clicking photos in public guest mode displays a preview dialog or prompts login to access the full `/media` archive.
5. **Zero Edit Controls Enforcement:**
   - In unauthenticated mode, no `+ Add Person` button, quick-add dialog, inline edit pencil, or undo trigger may ever be rendered.


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
- [ ] **Default Private Gatekeeper:** Unauthenticated visitors to `/` are immediately redirected to `/login` when public mode is disabled.
- [ ] **Unauthenticated Public Dashboard View:** When public access is enabled (`ENABLE_PUBLIC_ACCESS=true`), `/` renders in strictly read-only mode with zero `+ Add Person` button, living relatives masked to single initial with dot (`И.` / `J.`), zero Undo buttons, and administrative data-quality links hidden.
- [ ] **Default Clean Slate & Zero Dummy Data:** Production builds never insert or display hardcoded mock individuals or fake dashboard payloads; an empty database always triggers the genuine empty-tree onboarding card.
