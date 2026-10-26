# Roadmap & Implementation Phases

## 1. Product Principles
- **Simple and clear:** every screen must be understandable by a non-technical relative without a manual.
- **Lightweight self-hosting:** one `docker compose up -d`, 3 containers (frontend, backend, PostgreSQL), small RAM footprint.
- **1 instance = 1 family tree.**
- **Private by default:** living people protected, invite-only membership, no data leaves the server unless explicitly configured.

---

## 2. Delivery Phases

### Phase 0 — Design system & component library
1. Theme tokens, light/dark mode, fonts ([theme-and-tokens](design-system/theme-and-tokens.md)).
2. Reusable `ui/`, `form/` and `domain/` components with the `/dev/ui` workbench, in the order from [components §6](design-system/components.md#6-implementation-order-before-pages).

### Phase 1 — MVP ("a family can build and browse their tree")
1. Deployment: single `docker compose up` ([08](features/08-deployment-single-instance.md)), health check, automatic migrations, nightly backup + CLI restore.
2. Auth: `/setup`, invites, passkeys + password fallback, sessions ([01](features/01-authentication-passkeys.md), [17](features/17-api-security-operations.md)).
3. Roles Admin / Editor / Viewer, living-person privacy ([02](features/02-rbac-access-control.md)).
4. Data model + person/family/event editing, date parser, places with picker ([10](features/10-core-data-model.md), [07](features/07-person-family-management.md), [12](features/12-places.md)).
5. Empty-tree onboarding, home person ([16](features/16-dashboard-home.md) §2–3).
6. Tree canvas: Hourglass + Pedigree + Descendants, quick-add, drawer ([04](features/04-tree-visualizations.md)).
7. Person profile: timeline, family, media, notes tabs ([03 page](pages/03-person-profile-page.md)).
8. Omnibox search + People Directory ([13](features/13-search-people-directory.md)).
9. Media upload, gallery, avatars ([06](features/06-media-face-tagging.md)).
10. GEDCOM 5.5.1 + 7.0 import (Clean Slate mode) and export ([03](features/03-gedcom-import-export.md)).
11. Change log, undo toast, restore deleted, conflict handling ([14](features/14-change-history-concurrency.md)).
12. EN + RU localization from day 0 ([00](features/00-internationalization-localization.md)).

### Phase 2 — "Delight"
- Face tagging ([06](features/06-media-face-tagging.md)), fan chart, kinship finder ([04](features/04-tree-visualizations.md)).
- Dashboard widgets & anniversaries ([16](features/16-dashboard-home.md) §4).
- Data-quality warnings, person merge ([15](features/15-data-quality-duplicates.md)).
- Life map, place map, geocoding ([12](features/12-places.md)).
- Public guest mode ([02](features/02-rbac-access-control.md)).

### Phase 3 — "Power tools"
- Printable PDF posters / tiles ([05](features/05-printable-tree-pdf.md)).
- Append & merge import with reconciliation ([09](features/09-tree-import-merge-reconciliation.md)).
- Magic-link email, monthly digest, GEDZIP in nightly backups.

---

## 3. Stack
| Component | Choice |
| :--- | :--- |
| Frontend | React 19 + Tailwind CSS v4, served by Nginx |
| Backend | Rust (Axum + SQLx) |
| Database | PostgreSQL 16 (`pg_trgm`, `unaccent`) |
| Background jobs | Postgres `jobs` table + in-process workers ([17 §5](features/17-api-security-operations.md#5-background-jobs)) |
| Media storage | Local Docker volume ([17 §4.2](features/17-api-security-operations.md#42-storage-layout)) |
| Maps | MapLibre / Leaflet with configurable tile URL ([12 §4](features/12-places.md#4-geocoding--self-hosting-privacy)) |
