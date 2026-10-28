# Feature Specification: API Conventions, Security & Operations

## 1. Overview & Objective
Cross-cutting rules every endpoint and background task must follow. Other specs list endpoints; this one defines **how** they behave (errors, pagination, auth, uploads, media delivery, jobs, health) so the backend stays consistent and safe for self-hosters.

---

## 2. REST Conventions

### 2.1 Basics
- Base path `/api/v1`, JSON (`application/json; charset=utf-8`), `snake_case` fields, UUIDs as strings, timestamps ISO-8601 UTC.
- Resources: `GET /people`, `GET /people/:id`, `POST /people`, `PATCH /people/:id` (partial), `DELETE /people/:id`.
- IDs in URLs accept **UUID or `gid`** (`/people/I0042`) for friendly links.
- OpenAPI 3.1 document generated from Rust types (`utoipa`) at `/api/v1/openapi.json`; TypeScript client generated from it for the frontend.

### 2.2 Error Format
```json
{
  "error": {
    "code": "version_conflict",          // stable machine code, used as i18n key on frontend
    "message": "Record was modified by another user",   // English, for logs/devs
    "details": { "current_version": 7 },
    "field_errors": { "birth.date": "date_unparseable" }
  }
}
```
| HTTP | Codes (examples) |
| :--- | :--- |
| 400 | `validation_failed`, `date_unparseable` |
| 401 | `not_authenticated`, `session_expired` |
| 403 | `forbidden`, `public_access_disabled` |
| 404 | `not_found` *(also returned for privacy-hidden records — never reveal existence)* |
| 409 | `version_conflict`, `ancestor_cycle`, `place_in_use` |
| 413 | `upload_too_large` |
| 429 | `rate_limited` |

> [!IMPORTANT] Client-Side Error Localization:
> The `message` field is intended strictly for backend server logs and debugging. The frontend client must **never** display `error.message` directly in non-English locales; it must resolve `error.code` (or HTTP status) via the localized dictionary under `errors.<code_or_status>` per [00 §2.7](00-internationalization-localization.md#27-error-code-localization-architecture-api--network-failures). Browser fetch failures (e.g. `TypeError: Failed to fetch`) must similarly map to `errors.network_error`.

### 2.3 Pagination, Sorting, Filtering
- Cursor-based: `?limit=100&cursor=<opaque>` → `{ "items": [...], "next_cursor": "...", "total": 1420 }`. `total` is cheap-count or omitted for expensive queries.
- `?sort=birth,-surname` (prefix `-` = desc).

### 2.4 Expansion
- `?include=events,families,media,notes` on detail endpoints to avoid N+1 round trips (profile page = **one** request).

---

## 3. Authentication & Session Security

### 3.1 Sessions
- Opaque random session token (256-bit) in cookie `session`: `HttpOnly; Secure (when HTTPS); SameSite=Lax; Path=/`. Hash stored in `sessions` ([10 §4.6](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#46-system-tables)).
- Sliding expiry 30 days (`SESSION_TTL_DAYS`), revocable per device from *Settings → Security* and by admins.
- Server-side sessions so any session can be revoked instantly.
- `SameSite=Lax` (not `Strict`) so magic-link clicks from email clients land signed in.

### 3.2 CSRF
- All state-changing requests require header `X-Requested-With: genealogy` (custom header ⇒ not sendable cross-site without CORS preflight) **and** `Origin` must equal `APP_ORIGIN`. No CORS allowed by default.

### 3.3 Rate Limiting (in-process, per IP + per account)
| Endpoint group | Limit |
| :--- | :--- |
| Login / passkey challenge / password / recovery code | 10 / min / IP, 5 failures → 15-min lockout per account |
| Magic link request | 3 / 15 min / email |
| Search (anonymous) | 60 / min / IP |
| Uploads | 120 files / hour / user |

### 3.4 Security Headers (served by frontend Nginx)
`Content-Security-Policy` (self + configured map tile host), `X-Content-Type-Options: nosniff`, `Referrer-Policy: same-origin`, `Permissions-Policy` (no camera/mic/geolocation), `Strict-Transport-Security` when behind HTTPS.

### 3.5 Local-Network Hosting & Login Fallback ⚠️
WebAuthn passkeys only work in a **secure context**: `https://…` or `http://localhost`. A family member opening `http://192.168.1.20:8080` on their phone **cannot use passkeys**, and many home installs have **no SMTP** for magic links. Without a fallback, nobody but the server owner could sign in.

Therefore:
- **Optional password login** (`AUTH_PASSWORD_ENABLED`, default `auto` = enabled when the app is not served over HTTPS **or** SMTP is disabled). Argon2id hashes, min 10 characters, rate limited. Users can set a password in *Settings → Security*; invite registration offers *"Create passkey"* or *"Set password"* depending on what the browser supports.
- **Admin-generated one-time login links** (no SMTP needed): admin clicks *"Create sign-in link"* for a user and sends it via WhatsApp/Telegram (same mechanism as invites, 24 h validity).
- Login page detects `window.isSecureContext === false` and hides passkey buttons with a hint: *"Passkeys require HTTPS. Ask your admin to enable HTTPS, or sign in with your password."*
- Deployment doc lists three easy HTTPS options for home use: **Caddy** with local CA, **Tailscale** (`*.ts.net` certs), or a reverse proxy with a real domain ([08](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/08-deployment-single-instance.md)).

---

## 4. File Uploads & Media Delivery

### 4.1 Upload Validation
- Max size `MAX_UPLOAD_SIZE_MB`; streamed to a temp file, never buffered fully in RAM.
- Type determined by **magic bytes**, not extension/`Content-Type`. Allowlist: JPEG, PNG, WebP, AVIF, HEIC (converted to JPEG on ingest), TIFF, GIF, PDF, MP3, M4A, WAV. Reject SVG and HTML (XSS vectors).
- EXIF: original preserved on disk; **GPS tags stripped from derived thumbnails**; `DateTimeOriginal` offered as suggested "date taken".
- Image decoding in a size-limited worker (max 100 megapixels) to prevent decompression bombs.
- ZIP/GEDZIP extraction: reject absolute paths and `..` (zip-slip), cap total uncompressed size (`MAX_IMPORT_SIZE_MB`, default 2048) and file count.

### 4.2 Storage Layout
```
/app/data/media/
  originals/ab/cd/<sha256>.<ext>      # content-addressed → dedup for free
  thumbs/ab/cd/<sha256>_{256,768,1600}.webp
  avatars/<person_id>_<crop_hash>_{96,256}.webp
  .trash/                              # soft-deleted, purged after 30 days
```

### 4.3 Authorized Delivery
- Media is **never** served as a public static directory. URLs: `/api/v1/media/:id/file?size=thumb|medium|large|original`.
- Backend checks session + privacy rules (living/private, see [02](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md)) then responds with `X-Accel-Redirect` to an Nginx `internal` location → fast zero-copy serving.
- `Cache-Control: private, max-age=86400` + `ETag` (sha256). Public-mode deceased-only media may use `public`.
- PDFs served with `Content-Disposition: inline` and `Content-Security-Policy: sandbox`.

---

## 5. Background Jobs
- Job queue stored in the Postgres `jobs` table ([10 §4.6](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#46-system-tables)), claimed with `SELECT … FOR UPDATE SKIP LOCKED`; workers run as Tokio tasks inside the backend process (`JOB_WORKERS`, default 2).
- Progress streamed via **SSE** `GET /api/v1/jobs/:id/events`; polling fallback `GET /api/v1/jobs/:id`.
- Jobs survive restarts (`running` jobs older than heartbeat timeout are re-queued or marked failed).
- Scheduler (cron, in-process): nightly backup, retention pruning (history, snapshots, trash, expired sessions/magic links/invites), duplicate detection, monthly digest.

---

## 6. Operations

### 6.1 Health & Info
- `GET /api/v1/health` → `200 {"status":"ok"}` when DB reachable and migrations applied (used by Docker healthcheck).
- `GET /api/v1/admin/system` (Admin): app version, DB size, media size & file count, free disk space, last backup time + status, job queue length. Shown on *Settings → General* as a small "System" card.

### 6.2 Migrations & Upgrades
- SQLx migrations run automatically at startup inside an advisory lock; a **pre-upgrade backup** is taken automatically when pending migrations exist.
- Release notes state the minimum version to upgrade from.

### 6.3 Logging
- Structured JSON logs to stdout (`RUST_LOG`). Never log session tokens, magic-link tokens, recovery codes, or personal data payloads; log record IDs only.

### 6.4 Backup Contents (clarifies [08 §4](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/08-deployment-single-instance.md#4-disaster-recovery--cli-operations))
- `database.dump` (pg_dump custom format), `media/originals/` (thumbnails are regenerated), `manifest.json` (app version, schema version, counts, sha256 list).
- Optional: also write a **GEDZIP export** alongside each nightly backup (`BACKUP_INCLUDE_GEDCOM=true`) so data is readable by any genealogy app even if this project dies.
- Restore verifies manifest checksums and schema compatibility before touching data.

---

## 7. Definition of Done (DoD)

- [ ] **Consistent Errors:** Every non-2xx response uses the standard error envelope with a stable `code`.
- [ ] **Privacy 404:** Requests for hidden (living/private) records return `404`, indistinguishable from missing.
- [ ] **CSRF Protection:** State-changing requests without the custom header or with a foreign `Origin` are rejected.
- [ ] **LAN Login Works:** On `http://<LAN-IP>` without SMTP, a family member can register via invite and sign in with a password.
- [ ] **Secure Media:** No media file is reachable without passing the privacy check; direct paths under `/app/data/media` are not web-exposed.
- [ ] **Upload Safety:** SVG/HTML uploads are rejected; zip-slip and decompression-bomb test files are rejected.
- [ ] **Postgres Job Queue:** Imports, exports, PDFs, and backups run via the Postgres job queue and survive a backend restart.
- [ ] **Auto Pre-Upgrade Backup:** Starting a new version with pending migrations creates a backup first.
- [ ] **OpenAPI:** `/api/v1/openapi.json` is generated and the frontend client is built from it.
