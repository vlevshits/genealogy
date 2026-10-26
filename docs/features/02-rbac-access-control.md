# Feature Specification: Role-Based Access Control (RBAC), Security Segregation & Privacy

## 1. Overview & Objective
Establish a defense-in-depth security model that balances complete family data privacy with frictionless collaboration. The platform operates under a strict **Default Private Instance** posture: out of the box, unauthenticated visitors see only the Login page (`/login`) and cannot view tree nodes, people, or media.

Access is partitioned into four distinct tiers:
1. **Unauthenticated (Default Private):** Blocked from all content; redirected to `/login`.
2. **Unauthenticated (Public Tree Mode Enabled by Admin):** Allowed to view a strictly masked/redacted tree visualizer and deceased ancestor records. Living individuals, private records, media, directories, exports, and tools are completely inaccessible.
3. **Authenticated Viewer (`Viewer`):** Trusted, authenticated family members. Granted full read-only access to browse the complete family tree, including personal data of living relatives (vital dates, surnames, relationships, life events, photos, biographies). Strictly read-only; bulk export and administrative tools are blocked.
4. **Authenticated Editor (`Editor`):** Content curators. Possess all Viewer permissions plus the ability to create, update, and delete individuals, families, events, places, and notes, upload and tag media, and use the 60s inline undo.
5. **Administrator (`Admin`):** Full instance ownership. Holds exclusive authority over instance configuration, public tree sharing, user management, backups, snapshot rollback, GEDCOM import/export, printable PDF tree generation, global audit history (`/history`), data quality inspections (`/quality`), and in-tree person deduplication (`/people/merge`).

---

## 2. Master Segregation & RBAC Permission Matrix

| Capability / Route / Action | Unauthenticated (Default Private) | Unauthenticated (Public Tree Enabled) | Authenticated Viewer | Authenticated Editor | Administrator |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **INITIAL ACCESS & AUTH** | | | | | |
| View Login page (`/login`) | Yes | Yes | Yes (Redirects to `/`) | Yes (Redirects to `/`) | Yes (Redirects to `/`) |
| First-time Setup (`/setup`, 0 users) | Yes | Yes | No | No | No |
| Accept Invitation & Register | Yes (with token) | Yes (with token) | No | No | No |
| Manage Own Passkeys & Profile | No | No | Yes | Yes | Yes |
| **TREE & GENEALOGICAL BROWSING** | | | | | |
| View Interactive Tree (`/tree`) | **No (Redirects `/login`)** | **Yes (Strictly Masked)** | **Yes (Full Unredacted)** | **Yes (Full Unredacted)** | **Yes (Full Unredacted)** |
| View Deceased Individuals & Facts | No | Yes (Public only) | Yes | Yes | Yes |
| View Living Persons' Names & Surnames | No | **Strictly Masked (`Living`)** | **Yes (Full Name)** | **Yes (Full Name)** | **Yes (Full Name)** |
| View Living Persons' Dates & Places | No | **Blocked (Hidden)** | **Yes (Full Dates/Places)** | **Yes (Full Dates/Places)** | **Yes (Full Dates/Places)** |
| View Living Persons' Photos & Avatars | No | **Blocked (Silhouette)** | **Yes (Full Media)** | **Yes (Full Media)** | **Yes (Full Media)** |
| View Private Records (`is_private=true`) | No | No | No | **Yes** | **Yes** |
| People & Surnames Directory (`/people`) | No | No | Yes | Yes | Yes |
| Places Directory & Maps (`/places`) | No | No | Yes | Yes | Yes |
| Media Archive & Gallery (`/media`) | No | No | Yes (Deceased + Living) | Yes (Deceased + Living) | Yes |
| Dashboard & Anniversaries (`/`) | No | No | Yes (Living Birthdays) | Yes | Yes |
| Kinship Finder ("Relationship to me") | No | No | Yes | Yes | Yes |
| **GENEALOGICAL CURATION & EDITING** | | | | | |
| Create / Edit / Delete Individuals | No | No | No | **Yes** | **Yes** |
| Create / Edit / Delete Unions & Events | No | No | No | **Yes** | **Yes** |
| Upload & Face-Tag Media Assets | No | No | No | **Yes** | **Yes** |
| Toggle Record Privacy (`is_private`) | No | No | No | **Yes** | **Yes** |
| Undo Own Recent Edit (Within 60s) | No | No | No | **Yes** | **Yes** |
| **BULK DATA, AUDIT & EXPORT TOOLS** | | | | | |
| Printable PDF / Poster Export (`/export/print`)| No | No | No | No | **Yes** |
| GEDCOM Import (Clean Slate & Merge) | No | No | No | No | **Yes** |
| GEDCOM Export (`.ged` / `.gdz`) | No | No | No | No | **Yes** |
| System Audit Feed / History (`/history`)| No | No | No | No | **Yes** |
| **ADMINISTRATION & SYSTEM OPERATIONS** | | | | | |
| Resolve Data Quality Warnings (`/quality`)| No | No | No | No | **Yes** |
| In-Tree Person Merge (`/people/merge`) | No | No | No | No | **Yes** |
| Enable/Disable Public Tree Sharing | No | No | No | No | **Yes** |
| User & Role Management (`/settings/users`)| No | No | No | No | **Yes** |
| Generate User Invitations | No | No | No | No | **Yes** |
| Full Database Backup Download | No | No | No | No | **Yes** |
| Tree Snapshot Rollback (`/settings/backups`)| No | No | No | No | **Yes** |
| System Diagnostics & Storage Card | No | No | No | No | **Yes** |

---

## 3. Access Tiers & Operational Boundaries

### 3.1 Tier 1: Default Private Instance (Login-First Gatekeeper)
By default, the application is strictly private (`ENABLE_PUBLIC_ACCESS=false`):
- Any request from an unauthenticated user to `/`, `/tree`, `/people`, `/media`, `/settings`, or any `/api/v1/*` endpoint (except `/api/v1/auth/*` and `/api/v1/health`) immediately terminates in a `401 Unauthorized` response or an HTTP redirect to `/login`.
- Unauthenticated visitors cannot discover person counts, ancestor names, surnames, or tree existence.
- The browser tab and title display only the instance title and a clean login form per [01-auth-pages.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/01-auth-pages.md).

### 3.2 Tier 2: Public Tree Mode (Strict Backend Masking)
When an administrator explicitly toggles public sharing (`ENABLE_PUBLIC_ACCESS=true` in Admin Settings or `.env`):
- **Permitted Surface:** Unauthenticated visitors may access ONLY the tree visualizer (`/tree`) and individual profile views (`/persons/:id`).
- **Strict Living Person Masking:** The backend response serialization layer redacts all personal data of living individuals:
  - First Name & Surname: Replaced with `Living` / `Живущий` (or `[Living / Скрыто]`).
  - Patronymic: Stripped completely (`NULL`).
  - Birth, Baptism, Marriage Dates: Hidden completely (`NULL`).
  - Addresses, Locations, Coordinates: Stripped (`NULL`).
  - Biographies, Notes, Event Sources: Redacted (`NULL`).
  - Avatar & Photos: Profile avatars are replaced by a generic neutral silhouette. Direct media requests for photos tagged with living persons return `404 Not Found`.
  - Group Photos: If a historical photo includes deceased and living persons, the image remains viewable, but face-tag bounding boxes for living persons are stripped from the API payload.
- **Blocked Pages:** All auxiliary views—Dashboard (`/`), People Directory (`/people`), Media Archive (`/media`), Places (`/places`), Print PDF (`/export/print`), History (`/history`), Data Quality (`/quality`), Settings (`/settings`)—return `401 Unauthorized` and prompt the visitor to sign in.

### 3.3 Tier 3: Authenticated Viewers (`Viewer` — Trusted Family Circle)
Viewers are verified family members who have accepted an invitation and authenticated via Passkey or password:
- **Full Living Person Visibility:** Because Viewers belong to the trusted family circle, they have full read access to living family members: full names, patronymics, exact birth dates, phone/email contact details, family relationships, timelines, and tagged photos.
- **Private Item Shield:** If an Editor or Admin has explicitly marked an item as private (`is_private = TRUE`, e.g. sensitive adoption notes or private legal disputes), it remains hidden from Viewers.
- **Strict Read-Only Boundary:** Viewers cannot add, edit, or delete any records, upload media, or dismiss quality warnings. All write endpoints reject Viewer requests with `403 Forbidden`.
- **Bulk Extraction Protection:** Viewers cannot export GEDCOM files, generate printable PDF posters, or download database backups. This prevents unauthorized bulk data harvesting or leakage outside the family circle.

### 3.4 Tier 4: Authenticated Editors (`Editor` — Content Curators)
Editors are active family historians entrusted with recording family memory:
- **Genealogical CRUD:** Full permission to create, edit, link, and delete individuals, families, life events, places, citations, and notes.
- **Media Management:** Permission to upload photos, scan documents, adjust crop coordinates, and tag family faces.
- **Undo Capability:** Access to the 60-second inline undo toast for their own editorial transactions.
- **Operational Restrictions:** Editors **cannot** access instance settings, manage users, promote roles, enable public tree mode, create/rollback snapshots, export/import GEDCOM data, trigger PDF poster generation, view global audit history (`/history`), inspect or resolve data quality warnings (`/quality`), or execute duplicate person merges (`/people/merge`). These administrative and high-impact operations remain strictly isolated to Tier 5.

### 3.5 Tier 5: Administrators (`Admin` — Instance Ownership)
Administrators maintain complete control over the application, server storage, and data portability:
- **Instance Configuration:** Toggling public site sharing, setting default language, managing tree title.
- **User Governance:** Inviting members, assigning/revoking roles (`Editor`, `Viewer`), deactivating accounts, and managing sign-in links.
- **Bulk Data Portability:** Exclusive authority to perform GEDCOM imports (Clean Slate or Merge), GEDCOM exports (`.ged` / `.gdz`), and Printable PDF poster exports (`/export/print`).
- **Disaster Recovery & Snapshots:** Exclusive access to download full `.tar.gz` database and media archives, execute tree snapshot rollbacks, and view the global audit log (`/history`).
- **Quality & Deduplication Authority:** Exclusive authority to inspect data quality checks and soft warnings (`/quality`), dismiss warnings, and merge duplicate person records (`/people/merge`).

---

## 4. Privacy & Masking Rules (Backend Serialization Layer)

### 4.1 Canonical Living Status Evaluation
The backend computes `is_living` dynamically on read using the canonical algorithm in [10 §5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#5-living-status-rule-canonical-algorithm):
1. `is_living_override = FALSE` $\to$ Deceased.
2. `is_living_override = TRUE` $\to$ Living.
3. Any death/burial event present $\to$ Deceased.
4. Birth date known and current year $-$ birth year $\ge 110$ $\to$ Deceased.
5. Max child birth date known and current year $-$ child birth year $\ge 90$ $\to$ Deceased.
6. Otherwise $\to$ **Living**.

### 4.2 Serialization Interceptor Redaction Contract
When serializing responses for an **Unauthenticated Public Visitor** (`user_role = Anonymous`), the serializer executes field-level stripping:

```json
// Authenticated Viewer/Editor/Admin response for a living relative:
{
  "id": "c7a8b9d0-1234-5678-90ab-cdef12345678",
  "gid": "I0042",
  "is_living": true,
  "primary_name": {
    "given_name": "Elena",
    "patronymic": "Alexandrovna",
    "surname": "Levshits"
  },
  "birth": {
    "date": "1994-06-14",
    "place": "Saint Petersburg, Russia"
  },
  "avatar_url": "/api/v1/media/avatar-42.webp"
}

// Unauthenticated Public Visitor response for the exact same record:
{
  "id": "c7a8b9d0-1234-5678-90ab-cdef12345678",
  "gid": "I0042",
  "is_living": true,
  "primary_name": {
    "given_name": "Living",
    "patronymic": null,
    "surname": "Levshits"
  },
  "birth": null,
  "avatar_url": null
}
```

---

## 5. User Management & Security Rails

### 5.1 Invitation Workflow
- Only **Administrators** can generate user invitations (`POST /api/v1/admin/invitations`).
- Form specifies recipient name, assigned role (`Editor` or `Viewer`), and pre-linked person node (`person_id`).
- Invitation token is a cryptographically secure 256-bit random string, stored hashed in `invitations` table with a 7-day expiration.
- Revoking an invitation invalidates the token immediately.

### 5.2 Safety Rails
- **Sole Admin Protection:** An administrator cannot demote their own account to Editor/Viewer or deactivate themselves if they are the sole remaining active Admin.
- **Session Revocation:** Deactivating a user immediately purges all active session tokens from the backend session store, disconnecting any active browser or standalone PWA sessions.

---

## 6. Technical Implementation & Middleware

### 6.1 Axum Tower Middleware Extractors
API routes are guarded by Axum custom extractors that enforce role hierarchies:

| Extractor | Enforced Requirement | Unauthorized Outcome |
| :--- | :--- | :--- |
| `RequireAdmin` | Caller must possess valid session with `role = Admin`. | `403 Forbidden` |
| `RequireEditor`| Caller must possess valid session with `role IN (Admin, Editor)`. | `403 Forbidden` |
| `RequireAuth`  | Caller must possess valid session (`role IN (Admin, Editor, Viewer)`). | `401 Unauthorized` |
| `PublicOrAuth` | Session optional; if unauthenticated, checks `ENABLE_PUBLIC_ACCESS=true`. | `401 Unauthorized` (if public disabled) |

### 6.2 Redaction Query Helper
All SQLx read queries pass through a privacy-filtering query pipeline:
- In `PublicOrAuth` mode with no session:
  - Automatically joins `individuals` to check `is_living` condition.
  - Strips private records (`WHERE is_private = FALSE`).
  - Transforms living fields via SQL `CASE WHEN is_living THEN ... ELSE ... END` or Rust serializer hooks before transmitting payload.

---

## 7. Definition of Done (DoD)

- [ ] **Default Private Gatekeeper:** On a fresh instance with `ENABLE_PUBLIC_ACCESS=false`, any unauthenticated visit to `/`, `/tree`, `/people`, or `/media` redirects to `/login`.
- [ ] **Public Mode Masking:** When `ENABLE_PUBLIC_ACCESS=true`, unauthenticated visitors to `/tree` see living people masked as `"Living"` with zero birth dates, places, or photos.
- [ ] **Direct Media Shield:** Attempting to fetch a photo of a living person via `/api/v1/media/:id/file` as an unauthenticated visitor returns `404 Not Found`.
- [ ] **Viewer Full Living Visibility:** Authenticated Viewers can see full names, birth dates, places, relationships, and photos of living family members.
- [ ] **Viewer Read-Only Boundary:** Any `POST`, `PUT`, `PATCH`, or `DELETE` attempt by a Viewer returns `403 Forbidden`.
- [ ] **Admin Bulk Isolation:** Attempting to access GEDCOM import/export, PDF poster export (`/export/print`), full database backup, or snapshot rollback as a Viewer or Editor returns `403 Forbidden`.
- [ ] **Admin History Isolation:** Global change log (`/history` and `GET /api/v1/history`) is strictly restricted to Administrators; Viewers and Editors receive `403 Forbidden`.
- [ ] **Admin Quality & Deduplication Isolation:** Access to Data Quality (`/quality`, `GET/POST /api/v1/quality/*`) and Duplicate Person Merge (`/people/merge`, `GET/POST /api/v1/people/merge*`) is strictly restricted to Administrators; Viewers and Editors receive `403 Forbidden`.
- [ ] **Invitation Authorization:** Only Administrators can generate, list, or revoke user invitation tokens.
- [ ] **Sole Admin Protection:** The system rejects any request that would deactivate or demote the last remaining active Administrator.
