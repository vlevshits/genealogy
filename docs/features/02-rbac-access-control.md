# Feature Specification: Role-Based Access Control (RBAC) & Privacy

## 1. Overview & Objective
Enable secure, multi-user collaboration across family members with varying technical and editorial responsibilities. The system defines three standardized access levels (**Admin**, **Editor**, and **Viewer**) and implements strict privacy filters for living individuals and sensitive family records.

---

## 2. Roles & Permissions Matrix

| Permission Capability | Admin | Editor | Viewer | Public Guest (Anonymous) |
| :--- | :---: | :---: | :---: | :---: |
| **Manage Users & Settings** | Yes | No | No | No |
| **Backup / Restore Database** | Yes | No | No | No |
| **Import / Export GEDCOM Data** | Yes | Yes | Optional | No |
| **Create / Edit / Delete Records** | Yes | Yes | No | No |
| **Upload / Tag Media Assets** | Yes | Yes | No | No |
| **View Tree Visualizations** | Yes | Yes | Yes | Yes (if Public Mode enabled) |
| **View Deceased Individuals & Photos** | Yes | Yes | Yes | Yes |
| **View Living Persons' Names/Dates** | Yes | Yes | Configurable (Masked) | **Strictly Masked (`Living`)** |
| **View Living Persons' Photos/Avatars**| Yes | Yes | Configurable | **Strictly Blocked / Silhouette** |
| **Download Printable PDF** | Yes | Yes | Yes | Yes (Redacted Living Data) |
| **View Change History / Restore** | Yes | Yes | No | No |
| **Data Quality & Merge People** | Yes | Yes | No | No |
| **Rollback Snapshots** | Yes | No | No | No |
| **Invite Members** | Yes | Yes (Viewer role only) | No | No |

---

## 3. Privacy Protection & Public Guest Mode

### 3.1 The "Living Individual" Rule
Genealogy privacy standards require protecting personal data for living people (GDPR, identity theft prevention, and family sensitivity):
- An individual is considered **Living** if:
  - They have no recorded death event AND
  - Their birth date is less than 110 years ago (configurable threshold) OR no birth date is known and their generational position indicates they are alive.
- The canonical algorithm (incl. relative-based birth estimation, manual override, and "unknown → living" default) is defined in [10 §5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#5-living-status-rule-canonical-algorithm).

### 3.2 Public Site Access (Anonymous Guest Mode)
An administrator can configure the instance to operate as a **Public Family Tree Archive** (toggled via `ENABLE_PUBLIC_ACCESS=true` or in Admin Settings):
- **Unauthenticated Exploration:** Any visitor without logging in can explore the interactive tree, search deceased ancestors, read historical biographies, and inspect old photos/documents.
- **Strict Living Person Data Masking:**
  - **Names:** Displayed as `Living [Surname]` / `Живущий [Фамилия]` or simply `[Living / Скрыто]`.
  - **Dates & Places:** Birth dates, marriage dates, locations, addresses, occupations, and personal notes are completely hidden.
  - **Tree Connectors:** Branch connections (parent-child, spouse) remain visible so family structure is understandable, but nodes appear anonymized.
- **Strict Living Person Photo & Media Protection:**
  - **Avatars:** Profile pictures of living individuals are replaced by a generic neutral avatar silhouette; the underlying image file is never transmitted to anonymous visitors.
  - **Individual Media:** Photos and documents linked exclusively to living persons are inaccessible (API returns `404 / 403` to unauthenticated requests).
  - **Group Photos:** If an archived group photo includes both deceased and living persons, the photo remains viewable, but face-tag bounding boxes and tooltips for living persons are stripped from the response.

### 3.3 Privacy for Authenticated Viewers
- When viewed by an authenticated `Viewer`:
  - Living person details follow the configured privacy level.
  - **Exception ("My Profile" linkage):** If a `Viewer` account is linked to an `Individual` record via `users.person_id`, that user can see their own complete profile and immediate family (spouse, children, parents) unredacted.

### 3.4 Private Records & Notes
- Editors and Admins can toggle any individual, event, or media item as `is_private = TRUE`.
- Private items are excluded from anonymous guest queries, viewer accounts, exported public PDFs, and viewer GEDCOM exports.

---

## 4. User Management & Invitation Workflow

### 4.1 Inviting Family Members
1. Admin or Editor clicks **"Invite Family Member"** in the Settings panel.
2. Form fields:
   - Recipient First Name & Last Name
   - Email (optional if sharing a direct link)
   - Assigned Role (`Editor` or `Viewer`)
   - Linked Tree Node (search picker to link the invitee to their avatar in the tree)
3. Backend generates a random one-time invitation token (stored hashed in `invitations`, see [10 §4.6](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#46-system-tables)) with an expiration (e.g. 7 days). Pending invites can be listed, re-copied, and revoked.
4. The generated link is displayed (for copy-pasting into WhatsApp, Telegram, or Email) and optionally sent via SMTP if configured.

### 4.2 Managing Active Users
- Admins can promote/demote user roles.
- Admins can deactivate/reactivate users with a single click.
- Prevent self-demotion or deletion of the last remaining Admin account.

---

## 5. Technical Implementation & Middleware

### 5.1 RBAC Enforcement (Rust / Axum)
- API routes are guarded by Axum custom extractors and Tower middleware layers:
  - `RequireRole::<Admin>`: Route only accessible by administrators.
  - `RequireRole::<Editor>`: Accessible by Editors and Admins.
  - `RequireRole::<Viewer>`: Accessible by Viewers, Editors, and Admins.
  - `OptionalUser`: Extracts session if present, allows anonymous guests if `ENABLE_PUBLIC_ACCESS` is true.
- SQLx query builders and response serialization interceptors automatically redact dates, places, notes, and photos for `Viewer` and `Anonymous` requests when an individual matches the `is_living` condition.

---

## 6. Definition of Done (DoD)

- [ ] **Role Assignment:** Admins can change user roles between `Admin`, `Editor`, and `Viewer`.
- [ ] **Public Site Toggle:** Admin can enable or disable anonymous public browsing via UI toggle or environment variable (`ENABLE_PUBLIC_ACCESS`).
- [ ] **Anonymous Living Redaction:** Unauthenticated public visitors see strictly masked names (`Living` / `Живущий`) and zero dates, places, notes, or events for all living people.
- [ ] **Living Person Photo Shield:** Avatars of living people are replaced with placeholder silhouettes, and direct media URLs for living persons' photos return `403 / 404` for anonymous users.
- [ ] **Group Photo Face Tag Privacy:** Group photos containing living individuals have living tags stripped from public responses, while deceased tags remain interactive.
- [ ] **Permission Boundaries:** Viewers and anonymous guests attempting write actions receive `401 Unauthorized` or `403 Forbidden`.
- [ ] **Linked Individual View:** An authenticated Viewer linked to their personal tree node (`person_id`) can view their own full data unredacted.
- [ ] **Invitation Generation:** Admins and Editors can generate invite links with preset roles and tree associations.
- [ ] **Safety Rails:** System prevents deleting or demoting the last active administrator.
