# Gotchas & Pitfalls Log

This register catalogs known pitfalls, subtleties, edge cases, and past bugs with their root causes and solutions. Each entry has a permanent identifier (`GOT-NNN`).

---

## Index

- [GOT-001: React Hook Order Crash (Error #300) in Conditional Auth Contexts](#got-001-react-hook-order-crash-error-300-in-conditional-auth-contexts)
- [GOT-002: Spurious 401 Network Requests on Public Unauthenticated Loads](#got-002-spurious-401-network-requests-on-public-unauthenticated-loads)
- [GOT-003: Icons / Emojis Embedded in Localization Strings Cause Duplicate UI Icons](#got-003-icons--emojis-embedded-in-localization-strings-cause-duplicate-ui-icons)
- [GOT-004: Anti-Collinear Edge Crossing and Detour Pruning in Kinship / Tree Layouts](#got-004-anti-collinear-edge-crossing-and-detour-pruning-in-kinship--tree-layouts)
- [GOT-005: Reverse Proxy Origin & Host Mismatches in CSRF Middleware](#got-005-reverse-proxy-origin--host-mismatches-in-csrf-middleware)
- [GOT-006: Living Person 110-Year Assumption When Death Event is Missing](#got-006-living-person-110-year-assumption-when-death-event-is-missing)
- [GOT-007: PDF Export Connector Drops & SVG Tiling Scaling Discrepancies](#got-007-pdf-export-connector-drops--svg-tiling-scaling-discrepancies)
- [GOT-008: SQLx Offline Mode Compilation Failure If Compile-Time Macros Are Used](#got-008-sqlx-offline-mode-compilation-failure-if-compile-time-macros-are-used)

---

### GOT-001: React Hook Order Crash (Error #300) in Conditional Auth Contexts
- **File / Area:** `frontend/src/App.tsx`, `frontend/src/lib/auth-context.tsx`
- **Symptom:** React minified error #300 ("Rendered more/fewer hooks than during previous render") crashing the whole application on initial load or route transition.
- **Root Cause:** Hooks (`useCallback`, `useEffect`, `useState`) were invoked conditionally after early returns for loading or setup states.
- **Rule:** Never place hook invocations below conditional `if (isLoading) return ...` guards. All hooks must execute unconditionally at the top of the component function.

### GOT-002: Spurious 401 Network Requests on Public Unauthenticated Loads
- **File / Area:** `frontend/src/lib/auth-context.tsx`, `frontend/src/api/client.ts`
- **Symptom:** Console flooded with red 401 error traces when a guest or unauthenticated user visits the public landing page.
- **Root Cause:** Components mounted and automatically fired `/api/v1/auth/me` or `/api/v1/onboarding/status` without checking if the app was configured for public mode or if a session cookie existed.
- **Rule:** Suppress noisy 401 logs in the fetch wrapper when probing initial session status, and gracefully treat 401 as `user: null`.

### GOT-003: Icons / Emojis Embedded in Localization Strings Cause Duplicate UI Icons
- **File / Area:** `frontend/src/i18n/en.json`, `frontend/src/i18n/ru.json`
- **Symptom:** UI displays doubled icons like `+ + Add Person` or `← ← Back`.
- **Root Cause:** Translation strings included literal icon characters (`+ `, `← `, `✓ `) while the React component also rendered a Lucide icon (`<Button leftIcon={<Plus className="h-4 w-4" />}>`).
- **Rule:** Never include leading/trailing icon glyphs or emojis in `en.json` or `ru.json`. `frontend/tests/i18n/parity.test.ts` enforces this automatically.

### GOT-004: Anti-Collinear Edge Crossing and Detour Pruning in Kinship / Tree Layouts
- **File / Area:** `backend/src/services/genealogy.rs`, `frontend/src/lib/tree-layouts/hourglass-layout.ts`
- **Symptom:** Visual lines in tree views overlapping cards, collinear lines obscuring each other, or kinship path finder selecting absurdly long circular routes through distant in-laws.
- **Root Cause:** Naive breadth-first search found valid graph paths without penalizing generational leaps or partner detours.
- **Rule:** In `services::genealogy::find_kinship_paths`, prune paths that jump across unrelated spouses or exceed the direct path weight. In layouts, apply anti-collinear clearance offsets to vertical connection buses.

### GOT-005: Reverse Proxy Origin & Host Mismatches in CSRF Middleware
- **File / Area:** `backend/src/services/security.rs`, `frontend/nginx.conf`
- **Symptom:** Legitimate mutating POST/PUT requests failing with `403 Forbidden` (`csrf_detected`) when running behind Docker Nginx.
- **Root Cause:** Nginx forwarded `Host: localhost` while the browser sent `Origin: http://localhost:8080`, causing an origin-header string mismatch.
- **Rule:** Ensure Nginx passes `X-Forwarded-Host`, `X-Forwarded-Proto`, and `Host` transparently, and `csrf_protection_middleware` permits same-origin reverse-proxied traffic.

### GOT-006: Living Person 110-Year Assumption When Death Event is Missing
- **File / Area:** `backend/src/services/privacy.rs`
- **Symptom:** 19th-century historical figures being classified as "Living" and having their names/events redacted.
- **Root Cause:** If no death event was recorded and birth date was missing or partial, the algorithm failed to evaluate estimated age.
- **Rule:** In `privacy::is_living`, estimate birth year from child or spouse events if direct birth date is absent. If inferred birth is > 110 years ago, mark as deceased.

### GOT-007: PDF Export Connector Drops & SVG Tiling Scaling Discrepancies
- **File / Area:** `backend/src/services/pdf/layout.rs`, `backend/src/services/pdf/render.rs`
- **Symptom:** Lines in exported printable PDFs dropping below card boundaries or cutting off at tile page seams.
- **Root Cause:** Inconsistency between millimetre coordinates in layout calculation and DPI rasterization in SVG renderer.
- **Rule:** Use integer point units (`pt`) uniformly across layout generation and render passes, accounting for bleed margins and cut marks explicitly.

### GOT-008: SQLx Offline Mode Compilation Failure If Compile-Time Macros Are Used
- **File / Area:** `backend/src/`
- **Symptom:** `cargo check` or Docker build fails with `error: error returned from database: connection refused` or missing `sqlx-data.json`.
- **Root Cause:** Developer used `sqlx::query!` or `sqlx::query_as!` macros instead of runtime functions.
- **Rule:** Always use `sqlx::query(...)` and `sqlx::query_as::<_, T>(...)` without the exclamation mark.

_Last curated: 2026-11-04 (memory bank bootstrap)_
