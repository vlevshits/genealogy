# Roadmap & Implementation Phases

## 1. Product Principles
- **Simple, quiet, and clear (zero marketing fluff, zero tech bragging):** every screen must be understandable by an elderly or non-technical relative without a manual. The application is an archive for family memory, never a SaaS marketing showcase or developer portfolio. User-facing web pages strictly forbid promotional buzzwords ("next-gen", "cutting-edge", SaaS marketing slogans) and developer tech stack names (Rust, Axum, React, PostgreSQL, Tailwind, etc.). All copy is calm, respectful, functional, and family-history focused ([00 §3](features/00-internationalization-localization.md#3-ui-copywriting-tone-of-voice--anti-fluff-standards)).
- **Lightweight self-hosting:** one `docker compose up -d`, 3 containers (frontend, backend, PostgreSQL), small RAM footprint.
- **1 instance = 1 family tree.**
- **Private by default (Default Private Instance):** unauthenticated visitors see only the Sign-In view (`/login`) out of the box (`ENABLE_PUBLIC_ACCESS=false`). The family tree is strictly invite-only. If public tree access is explicitly enabled by an Administrator, unauthenticated guests can only view a masked tree canvas (`/tree`) where living persons display as a single initial character with dot (e.g. `И.` / `J.`) with surnames, patronymics, vital dates, places, notes, and photos completely stripped. Authenticated family viewers enjoy full visibility of living relatives within their trusted circle ([02](features/02-rbac-access-control.md)).

---

## 2. Delivery Phases

### Phase 0 — Design system & component library
1. Theme tokens, light/dark mode, fonts ([theme-and-tokens](design-system/theme-and-tokens.md)).
2. Reusable `ui/`, `form/` and `domain/` components with the `/dev/ui` workbench, in the order from [components §6](design-system/components.md#6-implementation-order-before-pages).
3. **UI copy & anti-fluff cleanup:** purge promotional buzzwords ("Next-Gen", marketing badges, SaaS copy) and developer tech-stack branding ("Axum + React 19 + Tailwind v4 + PostgreSQL 16", "Axum 0.8 (Rust)", "PostgreSQL 16", Swagger explorer link) from existing web pages (`frontend/src/App.tsx`, `frontend/src/components/SystemStatusCard.tsx`) and translation bundles (`frontend/src/i18n/en.json`, `frontend/src/i18n/ru.json`), replacing them with functional, domain-oriented family history copy in EN and RU per [00 §3](features/00-internationalization-localization.md#3-ui-copywriting-tone-of-voice--anti-fluff-standards).
4. **Mobile & tablet component ergonomics:** safe-area CSS variables, 44×44px / 48×48px touch targets, `BottomNav`, swipeable `BottomSheet`, and full-screen mobile dialogs ([theme-and-tokens §4.4–4.6](design-system/theme-and-tokens.md#44-responsive-breakpoints--device-targets), [components §3.4](design-system/components.md#34-navigation--structure)).

### Phase 1 — MVP ("a family can build and browse their tree")
1. Deployment: single `docker compose up` ([08](features/08-deployment-single-instance.md)), health check, automatic migrations, nightly backup + CLI restore.
2. Auth: `/setup`, invites, passkeys + password fallback, sessions ([01](features/01-authentication-passkeys.md), [17](features/17-api-security-operations.md)).
3. **Access Control & Master RBAC Matrix:** 5 access tiers (Default Private, Public Mode, Authenticated Viewer, Authenticated Editor, Administrator) with backend living-person masking (single initial character with dot, e.g. "И." / "J.", without surnames or patronymics), complete living visibility for verified family Viewers, and strict Admin isolation for GEDCOM portability, PDF poster generation, global audit log, data quality & duplicate merge, and instance settings ([02](features/02-rbac-access-control.md)).
4. Data model + person/family/event editing, date parser, places with picker ([10](features/10-core-data-model.md), [07](features/07-person-family-management.md), [12](features/12-places.md)).
5. Empty-tree onboarding, home person ([16](features/16-dashboard-home.md) §2–3).
6. Tree canvas: Hourglass + Pedigree + Descendants, quick-add, drawer ([04](features/04-tree-visualizations.md)).
7. Person profile: timeline, family, media, notes tabs ([03 page](pages/03-person-profile-page.md)).
8. Omnibox search + People Directory ([13](features/13-search-people-directory.md)).
9. Media upload, gallery, avatars ([06](features/06-media-face-tagging.md)).
10. GEDCOM 5.5.1 + 7.0 import (Clean Slate mode) and export ([03](features/03-gedcom-import-export.md)), including Gramps Web 6.0.1 & MyHeritage dialect support (married/maiden names, coordinates, associations, external IDs, server media path reconciliation).
11. Change log, undo toast, restore deleted, conflict handling ([14](features/14-change-history-concurrency.md)).
12. EN + RU localization and anti-fluff copywriting from day 0 ([00](features/00-internationalization-localization.md)).
13. **Progressive Web App (PWA) & Mobile/Tablet UX:** Web App Manifest, maskable and Apple touch icons, standalone launch, service worker static asset caching, mobile `BottomNav`, swipeable quick-view bottom sheets, and multi-touch tree gestures (pinch-to-zoom, pan) ([18](features/18-progressive-web-app-mobile.md)).

### Phase 2 — "Delight" (Core Completed; Active Delivery Milestone 2.1)
1. **Visualizations & Discovery:** Face tagging & primary avatar synchronization ([06](features/06-media-face-tagging.md)), fan chart (180°/360° concentric SVG, 4–8 generations, Ahnentafel indexing, quadrant colors), and Kinship Finder (LCA + bidirectional BFS, natural localized EN/RU kinship terms) ([04](features/04-tree-visualizations.md)).
2. **Dashboard Widgets & Anniversaries:** 4 live widgets (`AnniversariesWidget` with Julian-to-Gregorian shift and round milestone badges, `RecentActivityWidget` with 1-click Undo, `TreeGlanceWidget`, `LatestPhotosWidget`), default user `start_page: "dashboard"` setting ([16 §4](features/16-dashboard-home.md#4-dashboard-widgets)).
3. **Data Quality Warnings & Duplicate Person Merge (Admin only):** 10 soft validation rules with fuzzy-date windowing, warning dismissal memory (`dismissed_warnings`), Quick-Add duplicate detection guard, and 2-column side-by-side Person Merge wizard (`/people/merge`) with atomic transactions and GID aliases redirection ([15](features/15-data-quality-duplicates.md)).
4. **Life Map & Clustered Places Map:** `/places?view=map` with distance-based spatial clustering, interactive marker pins, and chronological migration paths, geocoding ([12](features/12-places.md)).
5. **Public Guest Mode:** Living person single-initial shielding (`И.` / `J.`) and private record redaction for unauthenticated and guest users ([02](features/02-rbac-access-control.md)).

#### Milestone 2.1 — Media Archive Reconciliation & Large Ingestion
- **`TASK-MEDIA-ZIP-RECONCILE`**: Gramps Web Media Bundle Reconciliation & Large Archive Ingestion
  - **Trigger & Test Dataset:** Gramps Web export pair (`test-data/gramps-export/gramps-web-export-20261026130902.ged` producing 129 `is_missing = TRUE` stubs with `/root/gramps/` prefixes + `test-data/gramps-web-media-export-20261026130904.zip` containing 129 matching JPEG images, 51.5MB).
  - **Backend Acceptance Criteria:**
    - Add `zip = "2.2"` crate (features = `["deflate"]`) to `backend/Cargo.toml`.
    - Configure Nginx `client_max_body_size 500M;` in `frontend/nginx.conf` (`location /api/` and `server` block).
    - Configure Axum `DefaultBodyLimit::max(500 * 1024 * 1024)` across API router / upload endpoints.
    - Implement streaming multipart upload disk spooling (`/tmp/upload_*.tmp`) preventing server OOM on up to 500MB archives.
    - Implement `POST /api/v1/gedcom/media-bundle` (Admin only) and `POST /api/v1/media/bundle` (Editor/Admin).
    - Zip security checks: reject zip-slip path traversal (`..`, absolute paths, null bytes) and decompression bombs (>100:1 ratio, >2GB uncompressed total) with `400 Bad Request`.
    - Basename reconciliation: match archive entry basenames against `is_missing = TRUE` stubs using case-insensitive matching (`original_file_name` / `external_path`), store original binary in `/app/data/media/originals/ab/cd/<sha256>.<ext>`, generate WebP thumbnails (256w, 768w, 1600w), clear `is_missing = FALSE`.
    - Emit detailed JSON reconciliation report (`total_archive_files`, `reconciled_stubs`, `unmatched_files`, `remaining_missing_stubs`, `duration_ms`).
  - **Frontend Acceptance Criteria:**
    - Settings (`/settings/gedcom`): Add "Upload Media Archive (.zip)" dropzone with missing stubs counter, live streaming progress bar, and post-upload reconciliation statistics card.
    - Media Gallery (`/media`): Update `+ Upload Media` modal to support `.zip` bundles up to 500MB, displaying 3-stage progress (upload, extract/match, thumbnailing) and summary dialog.
    - UI & i18n Quality: Zero hardcoded emojis or icons in translation strings (`en.json`/`ru.json`); icons rendered exclusively via React Lucide components; functional plain-language copy.
  - **QA & Verification Criteria:**
    - Automated integration test with `test-data/gramps-web-media-export-20261026130904.zip`: verifies all 129 stubs reconciled from `gramps-web-export-20261026130902.ged`, `is_missing` becomes `FALSE`, thumbnails generated, links preserved.
    - Malicious zip tests: zip-slip traversal entries (`../../etc/passwd`) rejected; decompression bomb ratio checked.
    - Memory usage test: verify streaming spooling keeps backend memory consumption $< 64\text{ MB}$ during 51MB+ uploads.
    - Role segregation: Viewers and Editors get `403 Forbidden` on `POST /api/v1/gedcom/media-bundle`; Viewers get `403` on `POST /api/v1/media/bundle`.

### Phase 3 — "Power tools"
- Printable PDF posters / tiles ([05](features/05-printable-tree-pdf.md)).
- Append & merge import with reconciliation ([09](features/09-tree-import-merge-reconciliation.md)).
- Magic-link email, monthly digest, GEDZIP in nightly backups.

---

## 3. Stack
| Component | Choice |
| :--- | :--- |
| Frontend | React 19 + Tailwind CSS v4, served by Nginx (`client_max_body_size 500M;`) |
| Backend | Rust (Axum + SQLx, `zip` crate for archive ingestion, `DefaultBodyLimit 500MB`) |
| Database | PostgreSQL 16 (`pg_trgm`, `unaccent`) |
| Background jobs | Postgres `jobs` table + in-process workers ([17 §5](features/17-api-security-operations.md#5-background-jobs)) |
| Media storage | Local Docker volume ([17 §4.2](features/17-api-security-operations.md#42-storage-layout)), streaming multipart ingest up to 500MB |
| Maps | MapLibre / Leaflet with configurable tile URL ([12 §4](features/12-places.md#4-geocoding--self-hosting-privacy)) |
| PWA & Mobile | Web App Manifest, Service Worker (static asset caching only), standalone display mode, safe-area insets ([18](features/18-progressive-web-app-mobile.md)) |
