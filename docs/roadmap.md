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

### Phase 2 — "Delight"
- Face tagging ([06](features/06-media-face-tagging.md)), fan chart, kinship finder ([04](features/04-tree-visualizations.md)).
- Dashboard widgets & anniversaries ([16](features/16-dashboard-home.md) §4).
- Data-quality warnings & duplicate person merge (Admin only) ([15](features/15-data-quality-duplicates.md)).
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
| PWA & Mobile | Web App Manifest, Service Worker (static asset caching only), standalone display mode, safe-area insets ([18](features/18-progressive-web-app-mobile.md)) |
