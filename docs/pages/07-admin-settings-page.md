# Page Specification: Settings & Administration (`/settings`)

## 1. Overview & Objective
The Settings page manages instance configuration, user invitations, security passkeys, database backups, and privacy controls. It follows a **Standard Two-Column Admin Layout** with a persistent vertical sub-navigation sidebar on the left and content panels on the right.

> [!IMPORTANT] Administrator-Only Boundary & User Profile Exception:
> The administrative settings panels (`/settings`, `/settings/users`, `/settings/security`, `/settings/gedcom`, `/settings/backups`) are restricted strictly to **Administrators** (`RequireRole::<Admin>`). Viewers and Editors attempting to access these panels receive `403 Forbidden`. The **sole exception** is User Profile (`/settings/profile`), which is self-service and accessible to all authenticated users (Viewer, Editor, Admin).

---

## 2. Page Wireframe & Layout

```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]      |
+------------------------------------------------------------------------------------+
| SETTINGS & ADMINISTRATION                                                          |
+--------------------------+---------------------------------------------------------+
| SETTINGS SUB-NAV (Left)  | ACTIVE SETTING PANEL (Right)                            |
|                          |                                                         |
| ⚙️ General & Archive     | GENERAL INSTANCE CONFIGURATION                          |
| 👥 Family Members & RBAC |                                                         |
| 🔐 Security & Passkeys   | Instance Tree Title:                                    |
| 📁 GEDCOM Import/Export  | [ Levshits Family Archive                             ] |
| 💾 Backups & Snapshots   |                                                         |
|                          | Default Language: [ Русский (RU)                     ▼] |
|                          |                                                         |
|                          | PUBLIC ACCESS & PRIVACY:                                |
|                          | [✓] Enable Public Site for Anonymous Visitors           |
|                          |     When enabled, living persons' vital dates and       |
|                          |     photos will be strictly masked.                     |
|                          |     When disabled (default), anonymous visitors see     |
|                          |     only the Sign-In page.                              |
|                          |                                                         |
|                          | Living Person Age Threshold: [ 110 years             ▼] |
|                          |                                                         |
|                          | [ Save Changes ]                                        |
+--------------------------+---------------------------------------------------------+
```

---

## 3. Structural Sections & Tabs

### 3.1 Settings Sub-Navigation (Left Column, `w-64`)
- **My Profile (`/settings/profile`, all authenticated users):** display name, email, language, theme, start page (Home / Tree), home person, linked person, monthly digest opt-in.
- **General & Archive (`/settings`, Admin only):** Tree metadata, default language, public access toggle (controls default private instance vs public tree mode), living person threshold, **default home person** for visitors, and a read-only **System** card (version, DB/media size, free disk, last backup — [17 §6.1](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/17-api-security-operations.md#61-health--info)). The System card uses functional, plain labels ("Database: Connected", "Server: Healthy", "Storage: X GB used", "Version: vX.Y.Z"), strictly omitting framework and database branding per [00 §3.4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md#34-permitted-exceptions--functional-terminology). Fields managed by env show *"Managed by server configuration"* when `SETTINGS_LOCKED_FROM_ENV=true`.
- **Family Members & RBAC (`/settings/users`, Admin only):** Member table, invite links, role management (Admin, Editor, Viewer).
- **Security & Passkeys (`/settings/security`, Admin only for instance-wide credentials):** Passkey device manager, enrollment of new authenticators, emergency recovery code regeneration.
- **GEDCOM Data Portability (`/settings/gedcom`, Admin only):** Standard export (5.5.1 / 7.0), GEDZIP export, Clean Slate import.
- **Backups & Recovery (`/settings/backups`, Admin only):** One-click full backup download, automated nightly snapshots, merge rollback logs.

### 3.2 Key Panel Contents

#### Panel 1: Family Members & RBAC
- **Header Action:** `+ Invite Family Member` (opens invitation modal with role picker and tree-node linker).
- **Users Table:**
  - Avatar, First & Last Name, Email.
  - Role pill (`Admin`, `Editor`, `Viewer`).
  - Linked Tree Node badge.
  - Active Passkeys count (e.g., `2 devices`).
  - Actions dropdown (Edit role, Resend invite, Deactivate, Revoke sessions, **Create sign-in link** — one-time 24 h link for relatives without passkey/SMTP).
- **Pending Invitations** list: invitee, role, linked person, expires in, *Copy link* / *Revoke*.

#### Panel 2: Security & Passkeys
- List of registered WebAuthn credentials (Device name, AAGUID icon: Touch ID / Windows Hello / YubiKey, date enrolled, last used).
- Button: `+ Register Another Passkey` (e.g. adding phone from laptop).
- Section: Emergency Recovery Codes (status: `10 codes generated, 1 used`).
- Section: **Password** (set / change; shown when password auth is enabled).
- Section: **Active Sessions** (device / browser, last seen, *Sign out* per session, *Sign out everywhere else*).

#### Panel 3: Backups & Snapshots
- Primary CTA: `📥 Download Full Archive` (creates `.tar.gz` containing archive database and media assets; user button label: `Download Full Archive (Database & Media)` / `Скачать полный архив (База данных и медиа)`).
- Merge History Log: Shows recent tree merges with `[Undo Merge]` button available for 14 days.
- **Snapshots** list (merge / clean-slate import / manual) with *Rollback* (typed confirmation) — [14](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md).
- Nightly backup status (last run, size, success/failure) and list of available backup archives with *Download*.

#### Panel 4: GEDCOM Data Portability & Media Archive Reconciliation (`/settings/gedcom`)
```
+------------------------------------------------------------------------------------+
| GEDCOM DATA PORTABILITY                                                            |
|                                                                                    |
| EXPORT TREE:                                                                       |
| Format: (•) GEDCOM 5.5.1   ( ) GEDCOM 7.0                                          |
| Media:  (•) Standalone .ged  ( ) Full Package (.zip bundle)                        |
| [✓] Redact living individuals (single initial mask)                                |
| [ Export Tree ]                                                                    |
|                                                                                    |
| IMPORT TREE (CLEAN SLATE):                                                         |
| +--------------------------------------------------------------------------------+ |
| | Drag and drop tree file (.ged, .zip, .gdz) here, or browse files               | |
| +--------------------------------------------------------------------------------+ |
|                                                                                    |
| MEDIA ARCHIVE RECONCILIATION:                                                      |
| ⚠️ Missing Media References: 129 media items in your tree are missing images.      |
|    Original source paths: /root/gramps/000085_*.jpg                                |
|                                                                                    |
| +--------------------------------------------------------------------------------+ |
| | [Upload Media Archive (.zip)]                                                  | |
| | Drop your Gramps Web media export ZIP (up to 500MB) here to link photos         | |
| +--------------------------------------------------------------------------------+ |
| Upload Progress: [====================================] 100% (51.5 MB / 51.5 MB)   |
|                                                                                    |
| RECONCILIATION REPORT:                                                             |
| ✓ 129 of 129 missing media files reconciled successfully                           |
| Total archive files: 129 | Unmatched: 0 | Remaining missing stubs: 0               |
| [ View in Media Gallery → ]                                                        |
+------------------------------------------------------------------------------------+
```
- **Export Section:** Select GEDCOM 5.5.1 or 7.0, toggle standalone vs full archive with media, living person privacy redaction checkbox, and `Export Tree` trigger.
- **Clean Slate Import Section:** Dropzone for `.ged` text files or `.zip` archives. Spools file, runs pre-import validation, and prompts with summary modal before applying changes.
- **Media Archive Reconciliation Dropzone:**
  - **Missing Stubs Status Alert:** Displayed when `media_assets` records exist with `is_missing = TRUE`. Shows exact count (e.g. `129 media items missing`) and sample missing paths (`/root/gramps/...`).
  - **Dropzone Target:** Accepts `.zip` archives up to 500MB (specifically tested with Gramps Web `gramps-web-media-export-*.zip`).
  - **Live Progress Feedback:** Real-time upload progress bar (streaming bytes and percentage) followed by extraction and thumbnail generation status.
  - **Reconciliation Statistics Card:**
    - Reconciled stubs count (matched against `original_file_name` case-insensitively).
    - Unmatched files count (if any).
    - Remaining missing stubs counter (updates reactively).
    - Link to view newly linked images in `/media`.
  - **Strict i18n & Icon Decoupling:** All UI text is localized in `en.json` and `ru.json` without embedded emoji or unicode symbols; status badges and icons use React components (`CheckCircle2`, `AlertTriangle`, `UploadCloud`) per [UI & i18n rules](file:///Users/levshitsvv/Projects/Personal/genealogy/.agents/rules/ui-and-i18n-conventions.md).

---

## 4. Definition of Done (DoD)

- [ ] **Administrator Route Isolation:** All `/settings` sub-pages (except `/settings/profile`) return `403 Forbidden` for Viewers and Editors, and redirect to `/login` for unauthenticated visitors.
- [ ] **Unified Two-Column Shell:** All settings tabs render inside the identical navigation sidebar and panel structure.
- [ ] **Default Private vs. Public Posture:** Toggling "Enable Public Site" properly flips instance posture between default private (unauthenticated redirected to `/login`) and public tree mode (masked living data on `/tree`).
- [ ] **Role Protection:** Prevents an administrator from demoting or deleting their own account if they are the sole admin.
- [ ] **One-Click Backup Generation:** Generates and downloads a valid full archive containing database and media files.
- [ ] **Media Archive Reconciliation Dropzone:** `/settings/gedcom` displays a dedicated dropzone accepting `.zip` archives up to 500MB when missing stubs exist, with live streaming progress.
- [ ] **Missing Media Count & Sample Paths:** Prominently displays the exact count of missing media references (`is_missing = TRUE`) and original external source paths (e.g. `/root/gramps/...`) to guide the administrator.
- [ ] **Reconciliation Report Card:** Displays immediate post-upload reconciliation statistics (total in archive, reconciled stubs, remaining missing) and updates the missing media banner dynamically upon completion.
- [ ] **UI i18n & Icon Compliance:** All translation keys in `en.json` and `ru.json` for the reconciliation interface strictly omit leading/trailing icon glyphs and emojis per project UI conventions; icons are rendered via React components.
- [ ] **Operational Plain Language:** System status indicators, backup actions, and settings descriptions display in clean, functional terminology with zero tech-stack bragging or promotional jargon.
