# Family Tree Application
## Feature Specifications & Architecture Overview

### 1. Project Vision
A modern, lightweight, self-hosted web application for building, browsing, and sharing a family tree. Families keep **complete ownership and privacy** of their data while enjoying a **clean, quiet, and simple** user experience focused entirely on family history.

See [roadmap.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/roadmap.md) for product principles and implementation phases.

---

### 2. Key Capabilities

| Area | What the app provides |
| :--- | :--- |
| **User Experience & UI** | Clean, responsive interface, GPU-accelerated SVG pan-zoom canvas, rich card avatars, glassmorphism accents. |
| **Visualization Modes** | Pedigree, Hourglass, Descendants, Fan Chart, and Kinship Finder with fast live switching. |
| **Media & Photos** | Media library with rectangular face tagging, thumbnail pipeline, and document viewer. |
| **Print & PDF Export** | High-res vector PDFs: single poster (A0–A3) or multi-page tiled sheets with cut guides and overlap margins. |
| **Authentication & Security** | WebAuthn / Passkeys (Touch ID, Face ID, Windows Hello, YubiKey), magic link, recovery codes, password fallback for LAN installs. |
| **Mobile, Tablet & PWA** | Progressive Web App installability (standalone mode), touch-first navigation (`BottomNav`, swipeable bottom sheets), safe-area insets, multi-touch pinch-to-zoom. |
| **Deployment** | 1 instance = 1 family tree; Docker Compose with 3 containers (frontend, backend, PostgreSQL). |

---

### 3. Architecture & Tech Stack

> [!IMPORTANT] Tech Stack Invisible in UI:
> The technologies listed below represent internal implementation details. Per [00 §3](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md#3-ui-copywriting-tone-of-voice--anti-fluff-standards), no tech stack names (Rust, React, PostgreSQL, Tailwind, Axum, etc.) or marketing buzzwords may ever be displayed on user-facing web pages, headers, footers, tooltips, or dialogs.

- **Frontend:** **React 19 + Tailwind CSS v4**
  - **Visualization Canvas:** **SVG + React (`d3-hierarchy` / `dagre` / `d3-zoom`)** for crisp vector typography, rich Tailwind card styling, accessible DOM elements, and 60fps pan/zoom.
- **Backend:** **Rust (Axum + SQLx + lettre)**
  - **GEDCOM Standard:** `ged_io` (active crate supporting both GEDCOM 5.5.1 and 7.0 parsing/writing).
  - **Passkeys / WebAuthn:** `webauthn-rs` (industry-standard, memory-safe FIDO2/WebAuthn L2/L3 implementation).
  - **Database & Jobs:** **PostgreSQL 16** (relational tree topology, `pg_trgm` search) — also hosts the background job queue (`SKIP LOCKED`), see [17 §5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/17-api-security-operations.md#5-background-jobs).
  - **Printable PDF Engine:** Rust vector layout engine utilizing `printpdf` / `typst` / headless vector slicing with exact millimeter tiling math, overlap bleed, and registration crop marks.
- **Authentication Strategy:** **Passkeys-first (WebAuthn)** with **Email Magic Link fallback** (delivered strictly via transactional SMTP with zero token leakage), single-use emergency recovery codes, and an **optional password fallback** for plain-HTTP LAN installs / instances without SMTP (see [17 §3.5](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/17-api-security-operations.md)).
- **Deployment Model:** Single-Instance = Single-Family-Tree via **Docker Compose**. Tiny memory footprint (~30-50MB RAM for backend).

---

### 4. Feature Specifications Directory

The detailed specifications and Definitions of Done (DoD) are organized in the following feature files:

0. [00-internationalization-localization.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/00-internationalization-localization.md): Day 0 Level Requirement: Full English & Russian localization, Slavic patronymics (отчество), bilingual kinship calculations, Russian date expressions, Cyrillic printable PDF support, strict UI copywriting standards (zero marketing buzzwords, zero tech-stack exposure), pure text translations with zero icon/glyph duplication, and comprehensive error code localization (zero raw English errors in Russian UI).
1. [01-authentication-passkeys.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/01-authentication-passkeys.md): WebAuthn Passkeys, registration, login, zero-leakage transactional SMTP magic links, device management, recovery fallback, safe relative `returnUrl` post-authentication navigation, and public archive egress.
2. [02-rbac-access-control.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/02-rbac-access-control.md): Master Segregation & RBAC Matrix across 5 access tiers (Default Private, Public Mode, Authenticated Viewer, Authenticated Editor, Administrator), login egress & returnUrl preservation, serialization living-person masking (single initial character with dot, e.g. "И." / "J.", without surnames or patronymics), complete living visibility for verified family Viewers, and strict Admin isolation for GEDCOM portability, PDF poster export, audit feed, and system settings.
3. [03-gedcom-import-export.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/03-gedcom-import-export.md): GEDCOM 5.5.1 and 7.0 import/export, GEDZIP media packaging, Gramps Web 6.0.1 & MyHeritage dialect handling (married/maiden names, coordinates, associations, external IDs), and streaming Media Archive Reconciliation (`POST /api/v1/gedcom/media-bundle`) for bulk ZIP archives up to 500MB with case-insensitive stub matching.
4. [04-tree-visualizations.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/04-tree-visualizations.md): Interactive canvas, pedigree, compact & wide collateral-expanded hourglass (aunts/uncles, first cousins, siblings, nieces/nephews), fan charts, descendant views, relationship calculation with Multi-Trace Kinship Path Finder, All Nodes complete forest graph view, interactive connection selection & Navigation HUD, anti-collinear line clearance, and Multiple Spouses / Successive Marriages with family-partitioned children distribution buses.
5. [05-printable-tree-pdf.md](05-printable-tree-pdf.md): Single-page high-resolution poster PDFs and multi-page tiled PDFs with cut marks and overlap margins, configurable partner and sibling inclusion, and live sheet count estimation.
6. [06-media-face-tagging.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/06-media-face-tagging.md): Photo and document gallery, batch ZIP archive ingestion (`POST /api/v1/media/bundle`) up to 500MB, automatic stub reconciliation, associated people in media panel (tagged vs untagged profile/depicted links), client-side automatic face detection with candidate suggestion rectangles, and interactive face tagging with coordinate bounding boxes.
7. [07-person-family-management.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/07-person-family-management.md): Rich person and family profile editing, life events, places, event sources (free text), notes, and quick-add actions.
8. [08-deployment-single-instance.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/08-deployment-single-instance.md): 1 instance = 1 family tree model, Docker Compose stack, configuration, and backup/restore workflows.
9. [09-tree-import-merge-reconciliation.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/09-tree-import-merge-reconciliation.md): External tree import, candidate matching engine (probabilistic record linkage), side-by-side conflict reconciliation, and 1-click snapshot rollback.
10. [10-core-data-model.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md): **Canonical schema** — individuals, names, families, events, notes, simple event sources, genealogical dates, stable IDs, living-person algorithm, integrity & delete rules, system tables, settings precedence.
12. [12-places.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/12-places.md): Place hierarchy (historical jurisdictions), place picker, merge, maps, privacy-safe geocoding, and automated data quality hierarchy restructuring integration.
13. [13-search-people-directory.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md): Cyrillic/Latin-aware search, omnibox, People Directory, surname index.
14. [14-change-history-concurrency.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/14-change-history-concurrency.md): Change log, undo/restore, optimistic locking & conflict dialog, tree snapshots.
15. [15-data-quality-duplicates.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/15-data-quality-duplicates.md): Soft validation warnings, relationship-centric duplicate detection with structural kinship guards, in-tree person merge, and automated place hierarchy resolution (Admin only).
16. [16-dashboard-home.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/16-dashboard-home.md): Home person, empty-tree onboarding, minimal dashboard, anniversaries.
17. [17-api-security-operations.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/17-api-security-operations.md): API conventions, sessions/CSRF, rate limits, LAN login fallback, upload safety, authorized media delivery, Postgres job queue, health & upgrades.
18. [18-progressive-web-app-mobile.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/18-progressive-web-app-mobile.md): Progressive Web App installability, Web App Manifest, maskable/Apple touch icons, standalone launch, service worker static asset caching, mobile/tablet ergonomics, touch targets, and tree gesture navigation.
19. [19-ai-family-story-intake-and-mcp.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/19-ai-family-story-intake-and-mcp.md): Multimodal oral/written family story intake ("Tell Me About My Family"), sandbox draft subtree staging, conversational clarifications, relationship-centric reconciliation, built-in dual provider (BYOK/local/offline fallback), and protected Model Context Protocol (MCP) server with role-gated tools and personal access tokens.

**Design system:** [theme-and-tokens.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/theme-and-tokens.md) (colors, typography, spacing) and [components.md](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/design-system/components.md) (reusable UI controls, built before pages).
