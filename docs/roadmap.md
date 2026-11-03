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

### Phase 2 — "Delight" (Completed: Core, Milestones 2.1, 2.2, 2.3, 2.4, 2.5; In Progress: Milestones 2.6, 2.7, 2.8)
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

#### Milestone 2.2 — Wide & Collateral-Expanded Tree Visualizations
- **`TASK-TREE-WIDE-HOURGLASS`**: Collateral-Expanded Wide Hourglass Layout & Traversal
  - **User Story & Trigger:** "looks much better but i want to ne able to see more wide tree in hourglass mode like main person is me but my grandparents had their kids and i want their kids and their families to be included as well" ([04 §2.2](features/04-tree-visualizations.md#22-hourglass-family-view)).
  - **Backend Acceptance Criteria:**
    - Extend `TreeViewQuery` in `backend/src/routes/rbac.rs` with `pub collateral: Option<bool>` (defaults to `true` when `mode="hourglass"`).
    - Implement bidirectional collateral expansion algorithm:
      - Upward BFS traversal discovers direct ancestor unions up to `generations` depth limit.
      - For each discovered ancestor union, cascade downward through non-direct children (collateral branches) collecting aunts, uncles, first cousins, siblings, nieces, nephews and their respective spouses.
      - Restrict collateral descent to generational tier boundaries (Gen -1 for aunts/uncles, Gen 0 for first cousins & siblings, Gen +1 for nieces/nephews & first cousins' children).
      - Set `generation: Option<i32>` relative tier index (Gen -2..+2) and `is_direct_lineage: Option<bool>` across all nodes and edges.
      - Enforce strict RBAC privacy: living collateral relatives in unauthenticated public mode display single initial with dot (`И.` / `J.`) and silhouette avatars with zero vital dates, places, notes, or media. In default private posture, unauthenticated callers receive `401 Unauthorized`.
      - Preserve `collateral=false` compact direct lineage mode.
    - Add integration tests in `backend/tests/tree_view_test.rs`: verify aunts/uncles/cousins inclusion, generational tier indices, `collateral=false` compact fallback, and living person masking.
  - **Frontend Acceptance Criteria:**
    - Refactor `frontend/src/lib/tree-layouts/hourglass-layout.ts`:
      - Implement recursive bottom-up bounding width allocation ($W_{\text{branch}} = \max(W_{\text{couple}}, W_{\text{children}})$) dynamically expanding grandparent and parent branches to accommodate collateral subtrees without card overlap.
      - Anchor focal couple symmetrically at $X \approx 0$, expanding paternal branches to the left and maternal branches to the right.
      - Align all cards strictly to their relative horizontal generational baselines ($Y = \text{tier} \times \text{ROW\_STEP}$).
      - Route orthogonal distribution buses with horizontal runners at vertical midpoints and vertical drops into children cards.
      - Visually accentuate direct biological ancestry edges with distinct stroke color and weight.
    - Update `frontend/src/components/tree/TreeToolbar.tsx`:
      - Add Hourglass Projection Toggle: "Wide" (`collateral=true`, default) vs "Compact" (`collateral=false`) with subtitle/tooltip.
      - Synchronize selection with URL query parameters (`?mode=hourglass&collateral=true|false`).
    - UI & i18n Quality: Zero icon glyphs or emojis in translation strings (`en.json`/`ru.json`); Lucide icons used exclusively in React components; full EN/RU localization parity.
  - **QA & Verification Criteria:**
    - Zero card collision: verified across large test trees with 5+ aunts/uncles and 10+ cousins.
    - Generational alignment: all first cousins and siblings verified at $Y = 0$; all aunts and uncles verified at $Y = -1 \times \text{ROW\_STEP}$.
    - Public mode privacy: living cousins and aunts verified with single initial masking and zero facts exposed.
    - Performance: 60fps pan/zoom maintained with 50+ collateral cards on desktop and mobile touch screens.

#### Milestone 2.3 — Critical Authentication Hardening & Transactional Email Delivery
- **`TASK-AUTH-MAGIC-LINK-SECURITY-HARDENING`**: Zero-Leakage Magic Link Delivery & SMTP Integration
  - **Vulnerability & Trigger:** Magic links currently leak `preview_url` and `preview_token` in `MagicLinkRequestResponse` when `SMTP_ENABLED=false` or `APP_ENV != "production"`. `LoginCard.tsx` renders an "Open Sign-In Link (Local Preview)" button, allowing any unauthenticated visitor to enter an arbitrary email (including admin) and immediately authenticate without email verification. Furthermore, no SMTP mailer was integrated in `AuthService::request_magic_link`.
  - **Backend Acceptance Criteria:**
    - Zero Token Leakage DTO: Permanently remove `preview_url` and `preview_token` fields from `MagicLinkRequestResponse` schema (`status: String`, `message: String` only). No token or link URL is ever returned in HTTP responses under any environment.
    - Expose `smtp_enabled: bool` in `SetupStatusResponse` (`GET /api/v1/auth/setup/status`) so frontend components know whether transactional email is configured.
    - Non-SMTP Behavior: When `SMTP_ENABLED=false` (or unconfigured), `POST /api/v1/auth/magic-link/request` immediately returns `503 Service Unavailable` with `{"error":{"code":"smtp_not_configured","message":"Email delivery is not configured on this server"}}`.
    - SMTP Mailer Dispatch (`lettre`): When `SMTP_ENABLED=true`, integrate `lettre` (tokio1-rustls, smtp-transport) to asynchronously dispatch multipart (HTML + plaintext) transactional emails using `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM_EMAIL`, and `SMTP_FROM_NAME`.
    - Support STARTTLS (port 587), direct TLS/SSL (port 465), and unencrypted dev SMTP (e.g. Mailpit/MailHog on port 1025).
    - Bilingual Email Templates: Render email subject and body in English and Russian per [00 §2.6](features/00-internationalization-localization.md#26-backend-email--notification-templates), containing a 15-minute single-use link `${APP_ORIGIN}/magic-link/verify?token=${raw_token}`.
    - Anti-Enumeration Protection: If the requested email is not found or inactive, return an identical `200 OK` (`{"status":"sent","message":"..."}`) without dispatching an email.
    - Token Security & Logging: Store only `token_hash` (SHA-256) in `magic_links`. Never log raw tokens in stdout or `lettre` trace logs.
    - Update backend tests: Update `backend/tests/auth_test.rs` to assert zero token fields in responses, test mock SMTP dispatch trap, and test `smtp_not_configured` 503 response.
  - **Frontend Acceptance Criteria:**
    - Refactor `frontend/src/components/auth/LoginCard.tsx`:
      - Permanently remove `magicPreviewUrl` state and the "Open Sign-In Link (Local Preview)" button.
      - In `mode === 'magic_sent'`, show only the confirmation alert (*"Check your inbox..."*) and a single button *"Back to sign-in options"*.
      - When `smtp_not_configured` (HTTP 503) is returned, display localized alert directing user to Passkey sign-in, password fallback, or contacting an administrator for a sign-in link.
    - Localization (`frontend/src/i18n/en.json` & `ru.json`):
      - Remove deprecated `magicLinkPreviewCta`.
      - Add `errors.smtp_not_configured` to error catalog.
      - Adhere strictly to UI & i18n rules (zero emoji/glyph prefixes in translation strings).
  - **QA & Verification Criteria:**
    - Zero Token Leakage: Verify `POST /api/v1/auth/magic-link/request` response payload contains strictly `status` and `message`; no `preview_url` or `preview_token` present.
    - Account Takeover Prevention: Attempting to sign in via magic link without access to the SMTP recipient inbox is strictly impossible; no preview button exists in the DOM.
    - Non-SMTP Fail-Closed: With `SMTP_ENABLED=false`, `POST /api/v1/auth/magic-link/request` returns HTTP 503 `smtp_not_configured`; UI renders localized error banner.
    - Live SMTP Delivery: With `SMTP_ENABLED=true` (Mailpit / mock SMTP), email is delivered with valid 15-minute token; clicking link validates session and redirects.
    - Anti-Enumeration: Submitting unregistered email yields identical 200 response without errors or emails.
    - Rate Limiting: 4th magic link request within 15 minutes is blocked with 429 `rate_limited`.

#### Milestone 2.4 — Public Archive Navigation & Safe Return URL Redirection
- **`TASK-AUTH-PUBLIC-NAVIGATION`**: Public Guest Egress & `returnUrl` Redirection
  - **User Story & Trigger:** "i can't get to home page from login page, can we fix that especially when site is configured as public" ([01 page](pages/01-auth-pages.md), [01](features/01-authentication-passkeys.md), [02](features/02-rbac-access-control.md)).
  - **Frontend Acceptance Criteria:**
    - `AuthLayout`: Render the brand logo and instance title as an interactive control (button or link), navigating to `/` with clear focus ring and hover styling.
    - Public vs. Private Guard Behavior:
      - When `enable_public_access === true`: clicking the brand logo navigates unauthenticated visitors to `/` (Home dashboard in public guest mode).
      - When `enable_public_access === false`: clicking the brand logo is intercepted by the private router guard, safely retaining the user on `/login` without leaking tree records or living identities.
    - `LoginCard`:
      - When `enable_public_access === true`: render a prominent secondary action "Browse Public Archive / Back to Family Tree" (`variant="secondary"` / `variant="outline"`).
      - When `enable_public_access === false`: strictly omit this action from the DOM.
      - Guest return route resolution: if `returnUrl` is present and targets a public-accessible route (`/`, `/tree`, `/persons/:id`), navigate to `returnUrl`; if `returnUrl` is omitted or points to an authenticated-only restricted route (e.g. `/people`, `/media`, `/settings`), safely fallback to `/` (Home) to avoid redirect bounce loops.
    - Post-Authentication `returnUrl` Preservation:
      - In `AuthPages.tsx` / `LoginCard.tsx` / `RegisterInviteCard.tsx` / `MagicLinkVerifyCard.tsx`: preserve and validate incoming `returnUrl`.
      - On successful authentication (`onSuccess`), navigate to the sanitized relative `returnUrl` (defaulting to `/` if omitted or invalid).
      - Open-redirect protection: strictly reject external URLs (`http:`, `https:`), protocol-relative URLs (`//`), and javascript URIs, falling back to `/`.
    - UI & i18n Quality:
      - Add translation keys to `en.json` and `ru.json` with zero icon glyphs and zero emojis.
      - Use calm, family-archival copywriting across all auth prompts and buttons.
  - **QA & Verification Criteria:**
    - Unauthenticated Public Egress: Verify unauthenticated visitor on `/login` can click header brand logo or "Browse Public Archive" button to reach public Home / Tree when `enable_public_access=true`.
    - Private Posture Defense: Verify clicking brand logo when `enable_public_access=false` retains visitor on `/login` with zero private data leaked.
    - Deep Link Preservation: Verify logging in from `/login?returnUrl=%2Ftree` or `/login?returnUrl=%2Fpersons%2F<id>` forwards user directly to the requested page.
    - Safe Fallback: Verify guest return button with a restricted `returnUrl=%2Fsettings` safely lands on `/` without redirect loops.
    - Open-Redirect Sanitization: Verify malicious `returnUrl=https://evil.com` or `//evil.com` safely redirects to `/`.

#### Milestone 2.5 — All Nodes / Complete Graph View Mode
- **`TASK-TREE-ALL-NODES-MODE`**: Complete Forest & Graph View Canvas
  - **User Story & Trigger:** "I like how wide tree is implemented, but i would like to have an option to display ALL nodes" ([04 §2.6](features/04-tree-visualizations.md#26-all-nodes-complete-graph-view), [02 page](pages/02-tree-view-page.md#38-all-nodes-canvas-layout--multi-component-forest-rendering)).
  - **Backend Acceptance Criteria:**
    - Support `mode="all"` in `TreeViewQuery` (`GET /api/v1/tree/view`).
    - Bypass focal BFS pruning: include all visible persons (`store.persons`) and all visible families (`store.families`) matching caller RBAC access tier.
    - Disconnected forest partitioning: group graph into connected components; assign topological generational tiers ($Y = \text{tier} \times \text{ROW\_STEP}$) based on parent-child chains anchored at focal person ($Gen = 0$) for the primary component, and chronologically/topologically leveled for secondary components.
    - RBAC privacy shielding: living individuals in all components strictly masked with single initial with dot (`И.` / `J.`), silhouette avatars, and zero vital facts in public guest mode. In default private posture, unauthenticated callers receive `401 Unauthorized`.
    - Add integration tests in `backend/tests/tree_view_test.rs`: verify all visible nodes returned without pruning, disconnected component generation assignment, and living privacy masking across all branches.
  - **Frontend Acceptance Criteria:**
    - Create `frontend/src/lib/tree-layouts/all-nodes-layout.ts` (and export via `index.ts`):
      - Multi-component forest layout: partition graph into connected components, calculate bottom-up subtree widths with zero card collision, place components side-by-side with $\ge 300\text{px}$ gaps, and organize isolated singletons into a clean shelf grid.
      - Continuous orthogonal edge routing for spouse connectors and parent-child distribution buses.
      - Accentuate direct biological lineage edges of the focal person if designated.
    - Update `frontend/src/components/tree/TreeToolbar.tsx`:
      - Add `all` option to `TreeToolbar` SegmentedControl with `Network` Lucide icon.
      - When `mode === 'all'`: hide or replace `generations` depth dropdown with total individuals count badge; hide Wide/Compact toggle.
      - Synchronize mode with URL query parameter (`?mode=all`).
    - Update `frontend/src/components/tree/TreeCanvas.tsx`:
      - Integrate `all-nodes-layout.ts` into `computeTreeLayout`.
      - Mini-map radar covers entire multi-component bounding box.
      - Level of Detail (LOD) and frustum culling: maintain smooth 60fps pan/zoom for archives with up to 1,500 individuals.
    - UI & i18n Quality: Zero icon glyphs or emojis in translation strings (`en.json`/`ru.json`); Lucide icons used exclusively in React components; full bilingual EN/RU parity.
  - **QA & Verification Criteria:**
    - Full tree inclusion: verify that all visible persons and families in database are present on canvas with zero nodes pruned.
    - Multi-component layout: verify disconnected family branches are laid out side-by-side with clean spacing and zero card collisions.
    - Generational alignment: verify parent-child generational tiers align horizontally without sloping.
    - Toolbar adaptability: verify generations selector and wide/compact toggle disappear in `mode="all"`, and mode switcher reflects "All Nodes".
    - Public mode privacy: verify living individuals in all components display with single-initial masking and zero vital facts.
    - 60fps pan/zoom performance: verified on desktop and mobile touch screens for archives with up to 1,500 nodes.

#### Milestone 2.6 — Interactive Tree Connections, Edge Disambiguation & Public Dashboard Privacy
- **`TASK-TREE-INTERACTIVE-CONNECTIONS`**: Interactive Tree Connections, Edge Disambiguation & Navigation HUD
  - **User Story & Trigger:** "it looks almost perfect, the only issue is that lines are overlapping in some cases maybe we can use different colours or highlight the connection when i click on it? it would be also nice to have some easy way to navigate by connection to nodes connected to it" ([04 §2.7, §3.3, §4.3](features/04-tree-visualizations.md), [02 page §3.9](pages/02-tree-view-page.md)).
  - **Frontend Acceptance Criteria:**
    - **Visual Differentiation by Relationship Type:**
      - Marriage lines: Warm Archival Ochre / Amber (`--color-brand-accent`: Light `#C68A34`, Dark `#D9A04B`, $2.5\text{px}$ solid stroke) with a small circular anchor junction dot ($r = 3\text{px}$) at union midpoint.
      - Direct lineage parent-child: Primary Brand Blue (`--color-brand-primary`: Light `#2D5584`, Dark `#4F7EB5`, $2.5\text{px}$ solid stroke, elevated z-index).
      - Collateral parent-child: Neutral Slate Gray (`--color-lineage-neutral`: Light `#64748B`, Dark `#8896AB`, $1.5\text{px}$ solid stroke, fully opaque).
      - Active selected edge: High-visibility glowing accent line ($3.5\text{px}$ stroke with luminous drop-shadow filter).
    - **Interactive Hit Area Ergonomics:**
      - Dual SVG path architecture: visible styled path + invisible wider interaction path (`strokeWidth={14}` desktop / `20px` mobile, `pointer-events: stroke`, `cursor: pointer`).
      - Hover feedback: line stroke expands by $+1\text{px}$ and displays pointer cursor.
    - **Selection & Endpoint Synchronization:**
      - Clicking connector sets `selectedEdgeId = edge.id`.
      - Connector illuminates with active glowing accent.
      - Both connected endpoint cards receive active focus rings (`ring-2 ring-brand-primary` / Level 2 elevation shadow).
    - **Connection Navigation HUD (`ConnectionNavPopover`):**
      - Floating frosted popover (`bg-surface/95 backdrop-blur-md border border-border rounded-xl shadow-elevation-2 p-3 z-30`) anchored near the clicked edge midpoint (or docked top-center / bottom sheet on mobile `< 768px`).
      - Shows relationship type badge ("Marriage" / "Parent → Child" with date/place).
      - Displays interactive participant chips for Person A and Person B (avatar, full name, role/generation badge).
      - 1-Click "Jump to [Person]" button (`Navigation` icon): smooth camera glide (`camera.lerp`) centering the viewport directly on that relative's card and pulsing the card ring.
      - "Find Kinship Path" CTA (`Route` icon): opens Kinship Finder pre-populated with Person A and Person B.
      - Profile link (`/persons/:id`) and close (`✕` / `Esc`) action.
    - **Anti-Collinear Edge Routing & Overlap Reduction:**
      - Distribution bus horizontal runners from adjacent unions sharing the same vertical gap are assigned staggered lanes with $\ge 12\text{px}$ vertical clearance (e.g. $Y + 144\text{px}$ vs $Y + 156\text{px}$), preventing merged collinear cuts.
      - Collinear vertical drops from separate branches are offset by $\ge 12\text{px}$ horizontally.
    - **Living Person Privacy in HUD:**
      - In public guest mode, endpoint chips display masked single-initial names (`И.` / `J.`), silhouette avatars, and zero vital dates.
    - **UI & i18n Quality:**
      - Full EN/RU bilingual parity for all connection and HUD strings with zero hardcoded emojis or icon glyphs in JSON string values.
  - **QA & Verification Criteria:**
    - Edge colors verified for marriages, direct lineage, collateral lines, and active selection.
    - Clicking connector reliably selects edge via 14px/20px transparent hit path.
    - Endpoint cards highlighted with active focus rings when edge is clicked.
    - Clicking "Jump to [Person]" smoothly pans and centers camera on that person's card.
    - Clicking "Find Kinship Path" opens Kinship Finder modal pre-populated with both endpoints.
    - Zero collinear line overlaps verified in complex multi-child test trees.

- **`TASK-TREE-CONTRAST-SHARPNESS`**: Visual Contrast & Subpixel Anti-Aliasing Standardization (WCAG 2.2 AA)
  - **User Story & Trigger:** "can we dfix contrast a bit? both themes looks a bit blurry" ([04 §3.4](features/04-tree-visualizations.md#34-visual-contrast-sharpness--subpixel-anti-aliasing-wcag-22-aa-compliance), [theme-and-tokens §2.4, §3.4](design-system/theme-and-tokens.md#24-tree-connection-lines--wcag-22-aa-contrast-standards)).
  - **Frontend Acceptance Criteria:**
    - **WCAG 2.2 AA Contrast Standards:**
      - All card typography (names, dates, locations, masked initials) meets $\ge 4.5:1$ contrast against card backgrounds in both light and dark themes.
      - All connector lines, card borders, and badges meet $\ge 3.0:1$ contrast against the canvas background (`#F8F9FA` light / `#0E1116` dark).
      - Remove washed-out opacity multipliers (e.g. `stroke-secondary/40`); use solid semantic color tokens.
    - **Subpixel Anti-Aliasing & Sharpness:**
      - Integer pixel coordinate snapping (`Math.round(pan.x)`, `Math.round(pan.y)`) on pan translation idle, eliminating subpixel text, border, and line blurriness caused by non-integer CSS transforms.
      - Apply `shapeRendering="geometricPrecision"` across all orthogonal SVG lines.
      - Node cards feature solid, crisp borders (`1px solid var(--color-border-card)`: `#CBD5E1` light / `#334155` dark).
      - Enforce `-webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; text-rendering: optimizeLegibility;` on canvas typography.
  - **QA & Verification Criteria:**
    - Contrast ratios verified with automated accessibility audit (`axe` / WCAG 2.2 AA).
    - Pan/zoom visual inspection: zero subpixel font or border blurriness when canvas is idle.

- **`TASK-DASHBOARD-PUBLIC-PRIVACY`**: Public Guest Dashboard 2-Widget Grid & Photo Privacy
  - **User Story & Trigger:** "can we remove these 2 nodes from publis web site dashboard" ([02 §3.2](features/02-rbac-access-control.md#32-tier-2-public-tree-mode-strict-module-segregation--zero-edit-controls), [16 §4.1](features/16-dashboard-home.md#41-widgets-fixed-set), [09 page §2.2, §3.3, §3.5](pages/09-dashboard-page.md)).
  - **Backend Acceptance Criteria:**
    - In `backend/src/services/dashboard.rs`: For `CallerAccess::PublicAnonymous`, return empty `Vec::new()` for both `recent_activity` and `latest_photos` in the dashboard response payload.
  - **Frontend Acceptance Criteria:**
    - In `frontend/src/pages/DashboardPage.tsx`: When `effectiveTier === 'public_anonymous'`, strictly omit `RecentActivityWidget` and `LatestPhotosWidget` from rendering.
    - Render a balanced 2-widget grid displaying exclusively `AnniversariesWidget` and `TreeGlanceWidget`.
  - **QA & Verification Criteria:**
    - Verify unauthenticated public dashboard `/` renders only 2 widgets (Anniversaries and Tree at a glance).
    - Verify `recent_activity` and `latest_photos` are empty in network payload for anonymous callers.
    - Verify authenticated users continue to see all 4 widgets based on their permissions.

#### Milestone 2.7 — Multiple Spouses / Successive Marriages & Family-Partitioned Children Buses
- **`TASK-TREE-MULTIPLE-SPOUSES`**: Multiple Spouses Visualization & Family-Partitioned Children Buses
  - **User Story & Trigger:** "it someone had more than one marriage this is not visualised properly on hourglass mode or descendant mode can we fix that as well" ([04 §2.7](features/04-tree-visualizations.md#27-multiple-spouses-successive-marriages--family-partitioned-children-buses), [02 page §3.10](pages/02-tree-view-page.md#310-multiple-spouses-placement--family-partitioned-bus-visualization)).
  - **Frontend Acceptance Criteria:**
    - `TreeGraphIndex`: Enhance `buildTreeGraphIndex` (`types.ts`) to preserve explicit family unions (`family_id`, partner pairs, children per family via `FamilyUnion` records) rather than flattening all spouses and children into monolithic per-person arrays.
    - Strict Child-to-Marriage Binding: Children must be associated strictly with the specific marriage/union they belong to; under no circumstances may children of Spouse 1 hang from Spouse 2's bus or share a common runner with half-siblings.
    - `computeHourglassLayout` (`hourglass-layout.ts`):
      - Remove single-spouse `.find(...)` restriction: discover and render all spouses of an individual on identical generational baselines ($Y = \text{tier} \times \text{ROW\_STEP}$).
      - Symmetrical placement for 2 spouses: `[Spouse 1] === [Person] === [Spouse 2]` with chronological ordering (earliest marriage left, later marriage right).
      - Partition children by family union: Union 1 children hang strictly under Union 1 bus; Union 2 children hang strictly under Union 2 bus.
      - Single-parent fallback bus: children with no recorded second parent hang from a vertical drop stem at bottom-center of the parent's card.
      - Ancestor multiple spouses in Wide Hourglass (`collateral=true`): direct ancestors with remarriages render additional spouses on Gen -1 and half-siblings on Gen 0 under their respective union bus. In Compact Hourglass (`collateral=false`), prune ancestor remarriages while retaining focal person's spouses.
      - Recursive width allocation: symmetrical wing widths ($W_{\text{left}} + W_{\text{card}} + W_{\text{right}}$) ensure zero node collisions between children clusters.
    - `computeDescendantsLayout` (`descendants-layout.ts`):
      - Recursively support multiple spouses per descendant node with family-partitioned children subtrees.
    - `computeAllNodesLayout` (`all-nodes-layout.ts`):
      - Group multi-partner units with family-specific children distribution buses and zero card collisions.
    - Bus Clearance & Edge Routing:
      - Symmetrical union anchors maintain $\ge 264\text{px}$ horizontal clearance.
      - Horizontal bus runners from distinct unions or adjacent families in the same generational gap use staggered lanes with $\ge 12\text{px}$ clearance.
    - Connection HUD Integration:
      - Clicking either marriage connector opens `ConnectionNavPopover` showing the specific union's participants and wedding facts.
    - UI & i18n Quality: Full EN/RU parity; zero emojis or icon glyphs in translation JSON values.
  - **QA & Verification Criteria:**
    - Person with 2 spouses verified in Hourglass and Descendants: both spouses displayed on identical horizontal baseline with ochre marriage bars.
    - Symmetrical layout verified: Spouse 1 on left, Person in center, Spouse 2 on right.
    - Children of Spouse 1 and children of Spouse 2 verified to hang strictly from their respective union buses with zero child lumping.
    - Half-sibling distinction verified: children from different marriages do not share a horizontal runner.
    - Single-parent child verified: hangs from direct bottom-center stem when second parent is unknown.
    - Width allocation verified: zero card collisions across multi-spouse subtrees.
    - Connection HUD verified: clicking each marriage line displays the correct pair and wedding details.
    - Compact Hourglass mode verified: ancestor remarriages pruned, focal person spouses preserved.

- **`TASK-TREE-KINSHIP-MULTI-TRACES`**: Multi-Trace Kinship Path Finder & Canvas Visualization
  - **User Story & Trigger:** "Also kinkship path finder need to be able to visualise multiple traces in this case" ([04 §2.5](features/04-tree-visualizations.md#25-kinship--relationship-path-calculator-multi-trace-support), [02 page §3.11](pages/02-tree-view-page.md#311-kinship-path-finder-multi-trace-visualization--legend-hud)).
  - **Backend Acceptance Criteria:**
    - In `backend/src/services/genealogy.rs` (`compute_kinship_path`): support discovering up to 5 distinct minimal genealogical paths across multiple marriages, remarriages, and collateral branches.
    - Update `KinshipResponse` payload: return `traces: Vec<KinshipTraceDto>` containing trace ID, relationship key, localized labels, `is_blood_relation`, `degree_of_kinship`, common ancestor IDs, and step nodes, while preserving top-level fields for backwards compatibility.
    - Add tests in `backend/tests/tree_view_test.rs` / `backend/tests/phase2_delight_test.rs` verifying multi-trace generation for half-siblings and complex marriages.
  - **Frontend Acceptance Criteria:**
    - In `frontend/src/components/tree/KinshipFinderDialog.tsx`:
      - Discover / consume multiple traces from API or local graph traversal.
      - Render segmented trace switcher tabs when multiple paths exist, indicating trace title, degree badge, and common ancestors.
      - Action triggers: "Highlight Selected Trace" and "Highlight All Paths".
    - In `frontend/src/components/tree/TreeCanvas.tsx`:
      - Support rendering multiple active kinship traces simultaneously: Trace 1 (Primary Brand Blue, $3.5\text{px}$ solid glow), Trace 2 (Warm Archival Ochre, $3.0\text{px}$), Trace 3 (Regal Violet, $3.0\text{px}$) with matching card rings.
      - Render floating top-center Multi-Trace Legend HUD (`absolute top-4 left-1/2 -translate-x-1/2 z-30`) with trace focus pills and a 1-click "Clear Highlights" (`✕`) button.
    - UI & i18n Quality: Bilingual EN/RU parity with zero icon characters or emojis in `en.json` / `ru.json`.
  - **QA & Verification Criteria:**
    - Verify half-siblings or relatives with multiple connections display multiple traces in Kinship Finder dialog.
    - Verify switching trace tabs updates the step-by-step breadcrumb ladder.
    - Verify clicking "Highlight All Paths" renders multiple colored paths on canvas simultaneously with zero visual glitching.
    - Verify floating legend displays active trace pills, 1-click camera lerp works, and "Clear Highlights" restores standard tree styling.

#### Milestone 2.8 — Associated People in Media & Automatic Face Detection
- **`TASK-MEDIA-ASSOCIATED-PEOPLE-AND-FACE-DETECTION`**: Associated People in Media Panel & Client-Side Automatic Face Detection
  - **User Story & Trigger:** "in people in this media i'd like to see people associated with this media (like this is profile picture) even if their face were not tagged. Also i would like to have automatic face detection, so instead of drawing areas manually i can select rectangles. as future improvement we can try to do autodetection of people based on existing samples" ([06 §2.2, §3.3, §3.4](features/06-media-face-tagging.md), [04 page §2, §3.3](pages/04-media-gallery-page.md)).
  - **Frontend Acceptance Criteria:**
    - **Associated People in Media Panel (`MediaMetadataPanel.tsx`):**
      - Rename and expand section heading to "PEOPLE IN THIS MEDIA ({count})".
      - Partition list into two distinct groups:
        1. **Tagged Faces:** Displays face crop thumbnail (`/api/v1/media/{id}/crop`), full name linking to `/persons/:id`, vital dates, `Primary Avatar` badge (if primary), primary avatar toggle, and delete face tag button. Hovering highlights the corresponding bounding box on the canvas.
        2. **Untagged Associated Individuals:** Displays profile avatar silhouette, full name linking to `/persons/:id`, vital dates, context badge (`Profile Photo` if `avatar_media_id == asset.id`, or `Associated`), `+ Tag Face` quick-action button, and unlink action.
      - Clicking `+ Tag Face` on an untagged individual enters Tagging Mode (`isTaggingMode = true`) with that person pre-selected. Prompts: *"Click a suggested face rectangle or draw a box for {name}"*. Clicking any candidate rectangle or drawing a box immediately creates the tag for this individual without requiring manual search in the picker.
      - Add `+ Link Person` action allowing Editors and Admins to associate a family tree individual to this photo/document without drawing a face box.
    - **Automatic Face Detection & Candidate Suggestion Rectangles (`FaceTagCanvas.tsx`):**
      - Provide an "Auto-Detect Faces" button (`Sparkles` / `Scan` icon) in the tagging controls bar.
      - Execute client-side face detection using the browser's `window.FaceDetector` API (Shape Detection API) with lightweight canvas heuristics fallback.
      - Normalize detected coordinates to `[0.0, 1.0]` and filter out candidate boxes overlapping existing `face_tags` (IoU $\ge 0.30$).
      - Render candidate rectangles on the canvas with distinct pulsing dashed border (`border-2 border-dashed border-sky-400 bg-sky-400/10 hover:bg-sky-400/25`) and a "Suggested Face" pill label.
      - **1-Click Candidate Adoption:** Clicking a candidate suggestion rectangle instantly adopts its bounding box as `draftBox`, opens `PersonFacePicker` (or assigns the pre-selected person), and creates the tag upon confirmation.
      - **Manual Drawing Fallback:** Clicking and dragging on the canvas remains fully functional for manual box creation, moving, and corner resizing.
    - **UI & i18n Quality:**
      - Complete bilingual EN/RU translations in `en.json` and `ru.json` with zero icon glyphs or emojis in translation string values.
  - **Backend Acceptance Criteria:**
    - In `GET /api/v1/media/{id}`: Return complete `linked_person_ids`, `face_tags`, and `links`, including individuals having `person.avatar_media_id == media.id`.
    - In `POST /api/v1/media/{id}/tags`: Synchronize `linked_person_ids` and update profile avatars when `is_primary_avatar = true`.
    - In `POST /api/v1/media/{id}/links` and `DELETE /api/v1/media/{id}/links/{link_id}`: Allow associating and unlinking individuals without face tags.
    - In public guest mode (`CallerAccess::AnonymousPublic`): Living associated individuals and face tags are strictly stripped or masked according to RBAC tier rules.
  - **QA & Verification Criteria:**
    - Verify associated people without face tags appear under "PEOPLE IN THIS MEDIA" with "Profile photo" or "Associated" badge.
    - Verify clicking "+ Tag face" enters tagging mode with the person pre-selected.
    - Verify clicking "Auto-Detect Faces" detects face bounding boxes and displays candidate suggestion rectangles on canvas.
    - Verify overlapping candidate rectangles with existing tags are filtered out.
    - Verify clicking a candidate rectangle adopts its coordinates and opens person picker or assigns pre-selected person.
    - Verify manual box drawing and resizing continues to work alongside auto-detection.
    - Verify living person privacy masking in public guest mode.

### Phase 3 — "Power tools"
- Printable PDF posters / tiles ([05](features/05-printable-tree-pdf.md), [implementation](implementation/05-printable-tree-pdf.md)) — **Complete**.
- Append & merge import with reconciliation ([09](features/09-tree-import-merge-reconciliation.md)).
- Automatic Face Recognition & Identity Clustering: facial feature embedding extraction (128-d/512-d vectors) from confirmed face tags, cosine similarity clustering, and automated person identity suggestion on new photo uploads ([06 §3.5](features/06-media-face-tagging.md#35-future-vision-facial-embedding-extraction--identity-clustering-phase-3), `TD-MEDIA-01`).
- Monthly digest email, GEDZIP in nightly backups.

#### Milestone 3.1 — Printable Tree Export Studio: Tree View Parity & Banner Customization
- **`TASK-PDF-STUDIO-EXPANSION`**: Printable Tree Export Studio — Tree View Parity (All Nodes, Collateral Density, Partners, Siblings, Photos) & Custom Banner
  - **User Story & Trigger:** "I want feature parity with interactive tree mode in the printable PDF studio: All Nodes full graph view, Wide vs Compact density for collateral branches, explicit toggles for spouse and siblings, portrait photos on cards, and full customization of both the main Title and Description/Subtitle banner." ([05 §1–3](features/05-printable-tree-pdf.md), [05 page §1–3](pages/05-printable-pdf-page.md)).
  - **Backend Acceptance Criteria:**
    - Extend `PdfExportRequest` schema:
      - `view_mode`: Support `"all_nodes"` (and `"all"`) in addition to `"pedigree"`, `"hourglass"`, `"descendants"`, and `"fan"`.
      - `collateral: Option<bool>`: Defaults to `true` for hourglass. When `true` (Wide), expands ancestral branches downward to encompass collateral aunts/uncles, first cousins, nieces/nephews; when `false` (Compact), confines tree strictly to direct lineal ascendants and descendants.
      - `include_partner: Option<bool>`: Defaults to `true`.
      - `include_siblings: Option<bool>`: Defaults to `false`.
      - `show_photos: Option<bool>`: Defaults to `true`. Embeds 300 DPI circular cropped profile portraits on cards from local media storage or base64 avatar strings.
      - `title: Option<String>`: Custom headline text override; falls back to `"{tree_title}: {focal_name}"` or `"Family Tree of {focal_name}"` when omitted.
      - `subtitle: Option<String>`: Custom description text override; falls back to localized metadata banner (`"Layout: {mode} • Generations: {gens} • Persons: {count} • {date}"`) when omitted.
    - Update `compute_tree_layout` in `backend/src/services/pdf/layout.rs`:
      - **All Nodes Layout:** Implement complete forest traversal using Disjoint Set Union (DSU) to partition the tree into connected components and an isolated singletons shelf grid, arranging components with $\ge 30\text{ mm}$ spacing and computing multi-sheet tiled bounding matrices.
      - **Hourglass Collateral Expansion:** Implement bottom-up width allocation with horizontal generational tiers ($Y = \text{tier} \times \text{ROW\_STEP}$) for collateral relatives when `collateral = true`.
      - **Partner & Sibling Placement:** Pair partners on generational baseline with warm ochre marital connector (`#d97706`); align siblings on Gen 0 branching from the parental distribution bus runner; support symmetrical multi-spouse placement (`[Spouse 1] === [Person] === [Spouse 2]`).
      - **Banner Generation:** Format decorative header with custom or fallback title and subtitle.
    - Update `POST /api/v1/export/pdf/preview-info` to return updated preview cards, connectors, resolved `title`, `subtitle`, and `total_persons`.
  - **Frontend Acceptance Criteria:**
    - Update `PrintConfigSidebar.tsx`:
      - Add `All Nodes` to Layout SegmentedControl; hide generation depth when All Nodes is active.
      - Add Tree Density SegmentedControl (`Wide (Collateral)` vs `Compact`) when Hourglass is selected.
      - Add "Include spouse / partner" (default checked) and "Include siblings" (default unchecked) checkboxes under "Relatives & Content Density".
      - Add "Show portrait photos" (default checked) checkbox.
      - Add text inputs for "Chart Title" and "Description / Subtitle" with smart placeholder and helper hint.
      - Disable partner, sibling, and collateral controls when Fan Chart is selected with an explanatory tooltip.
    - Update `PrintPreviewCanvas.tsx` and `PrintSummaryBar.tsx`:
      - Render custom headline and description in the canvas header banner with live updates as the user types.
      - Render 300 DPI circular profile portraits on cards when `show_photos = true`.
      - Render partner cards with ochre connector bars and siblings on parental bus runners.
      - Render multi-component forest layout in All Nodes mode.
      - Update Summary Bar badges: reflect accurate `total_persons` count, canvas dimensions ($W \times H\text{ mm}$), and sheet grid counts ($N \times M$).
    - UI & i18n Quality: Full EN/RU localization parity; zero emojis or icon glyphs in translation JSON strings.
  - **QA & Verification Criteria:**
    - Verify All Nodes mode generates valid multi-component poster and tiled grids.
    - Verify Hourglass Wide vs Compact toggles collateral branches correctly.
    - Verify toggling "Include spouse / partner" and "Include siblings" adds/removes cards and adjusts sheet counts.
    - Verify portrait photos appear on person cards in preview and exported vector PDF.
    - Verify custom title and subtitle text appear in PDF header banner; verify localized fallback when blank.
    - Verify Fan Chart layout disables incompatible collateral, partner, and sibling controls.
    - Verify living person privacy masking redacts birth dates and photos of living individuals.

---

## 3. Stack
| Component | Choice |
| :--- | :--- |
| Frontend | React 19 + Tailwind CSS v4, served by Nginx (`client_max_body_size 500M;`) |
| Backend | Rust (Axum + SQLx, `lettre` for SMTP transactional email, `zip` crate for archive ingestion, `DefaultBodyLimit 500MB`) |
| Database | PostgreSQL 16 (`pg_trgm`, `unaccent`) |
| Background jobs | Postgres `jobs` table + in-process workers ([17 §5](features/17-api-security-operations.md#5-background-jobs)) |
| Media storage | Local Docker volume ([17 §4.2](features/17-api-security-operations.md#42-storage-layout)), streaming multipart ingest up to 500MB |
| Maps | MapLibre / Leaflet with configurable tile URL ([12 §4](features/12-places.md#4-geocoding--self-hosting-privacy)) |
| PWA & Mobile | Web App Manifest, Service Worker (static asset caching only), standalone display mode, safe-area insets ([18](features/18-progressive-web-app-mobile.md)) |
