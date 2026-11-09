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
- [GOT-009: Browser Audio Recording Codec Compatibility & Object URL Leaks](#got-009-browser-audio-recording-codec-compatibility--object-url-leaks)
- [GOT-010: Server-Side Audio Transcription Requires GEMINI_API_KEY in .env](#got-010-server-side-audio-transcription-requires-gemini_api_key-in-env)
- [GOT-011: Microphone MediaStream Track & SpeechRecognition Cleanup on Unmount](#got-011-microphone-mediastream-track--speechrecognition-cleanup-on-unmount)
- [GOT-012: API Client Offline Fallback Guards Must Rethrow ApiError When status > 0](#got-012-api-client-offline-fallback-guards-must-rethrow-apierror-when-status--0)
- [GOT-013: Stale or Invalid Session Cookies Blocking Anonymous Access on Public Instances](#got-013-stale-or-invalid-session-cookies-blocking-anonymous-access-on-public-instances)
- [GOT-014: Serialized "null" or "undefined" Tokens in Browser Storage Causing Permanent 401 Loops](#got-014-serialized-null-or-undefined-tokens-in-browser-storage-causing-permanent-401-loops)
- [GOT-015: Story Intake Commit Deserialization Failure on draft_person_id vs draft_temp_id](#got-015-story-intake-commit-deserialization-failure-on-draft_person_id-vs-draft_temp_id)

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
- **File / Area:** `backend/src/services/genealogy/kinship_path.rs`, `frontend/src/lib/tree-layouts/hourglass-layout.ts`
- **Symptom:** Visual lines in tree views overlapping cards, collinear lines obscuring each other, or kinship path finder selecting absurdly long circular routes through distant in-laws.
- **Root Cause:** Naive breadth-first search found valid graph paths without penalizing generational leaps or partner detours.
- **Rule:** In `services::genealogy::find_kinship_paths`, prune paths that jump across unrelated spouses or exceed the direct path weight. In layouts, apply anti-collinear clearance offsets to vertical connection buses.

### GOT-005: Reverse Proxy Origin & Host Mismatches in CSRF Middleware
- **File / Area:** `backend/src/services/security/context.rs`, `frontend/nginx.conf`
- **Symptom:** Legitimate mutating POST/PUT requests failing with `403 Forbidden` (`csrf_detected`) when running behind Docker Nginx.
- **Root Cause:** Nginx forwarded `Host: localhost` while the browser sent `Origin: http://localhost:8080`, causing an origin-header string mismatch.
- **Rule:** Ensure Nginx passes `X-Forwarded-Host`, `X-Forwarded-Proto`, and `Host` transparently, and `csrf_protection_middleware` permits same-origin reverse-proxied traffic.

### GOT-006: Living Person 110-Year Assumption When Death Event is Missing
- **File / Area:** `backend/src/services/privacy/living.rs`
- **Symptom:** 19th-century historical figures being classified as "Living" and having their names/events redacted.
- **Root Cause:** If no death event was recorded and birth date was missing or partial, the algorithm failed to evaluate estimated age.
- **Rule:** In `privacy::is_living`, estimate birth year from child or spouse events if direct birth date is absent. If inferred birth is > 110 years ago, mark as deceased.

### GOT-007: PDF Export Connector Drops & SVG Tiling Scaling Discrepancies
- **File / Area:** `backend/src/services/pdf/layout/mod.rs`, `backend/src/services/pdf/render/mod.rs`
- **Symptom:** Lines in exported printable PDFs dropping below card boundaries or cutting off at tile page seams.
- **Root Cause:** Inconsistency between millimetre coordinates in layout calculation and DPI rasterization in SVG renderer.
- **Rule:** Use integer point units (`pt`) uniformly across layout generation and render passes, accounting for bleed margins and cut marks explicitly.

### GOT-008: SQLx Offline Mode Compilation Failure If Compile-Time Macros Are Used
- **File / Area:** `backend/src/`
- **Symptom:** `cargo check` or Docker build fails with `error: error returned from database: connection refused` or missing `sqlx-data.json`.
- **Root Cause:** Developer used `sqlx::query!` or `sqlx::query_as!` macros instead of runtime functions.
- **Rule:** Always use `sqlx::query(...)` and `sqlx::query_as::<_, T>(...)` without the exclamation mark.

### GOT-009: Browser Audio Recording Codec Compatibility & Object URL Leaks
- **File / Area:** `frontend/src/components/stories/AudioRecorderWidget.tsx`
- **Symptom:** Voice recording fails or records silence on iOS/Safari, or memory leaks occur during repeated preview playback.
- **Root Cause:** Safari requires `audio/mp4` MIME type fallback rather than `audio/webm;codecs=opus`, and naive `recorder.start()` without timeslice chunking fails to yield incremental blobs. In addition, unreleased `URL.createObjectURL(blob)` strings leak memory across recording sessions.
- **Rule:** Detect supported audio MIME types dynamically via `MediaRecorder.isTypeSupported()`, use `recorder.start(250)` for chunked data delivery, and clean up object URLs with `URL.revokeObjectURL()` upon unmount or re-recording.

### GOT-010: Server-Side Audio Transcription Requires GEMINI_API_KEY in .env
- **File / Area:** `backend/src/routes/rbac/ai_stories.rs:102-176`, `backend/src/services/story_intake/gemini.rs`
- **Symptom:** Audio file uploads to `POST /api/v1/ai/transcribe` fail with `400 Bad Request` (`gemini_api_key_not_configured` or `stt_provider_not_configured`).
- **Root Cause:** The backend server uses Gemini multimodal audio ingestion for speech-to-text (`transcribe_audio_with_gemini`). If neither `GEMINI_API_KEY` (in `.env`) nor instance `llm_api_key` is configured, raw audio bytes cannot be transcribed server-side.
- **Rule:** Provide `GEMINI_API_KEY` in deployment environment variables (`.env`) for zero-config audio intake. Ensure the frontend verifies `ai_enabled` via `GET /api/v1/system/info` before offering server-only transcription, and fall back to browser `SpeechRecognition` or manual text entry when unconfigured.

### GOT-011: Microphone MediaStream Track & SpeechRecognition Cleanup on Unmount
- **File / Area:** `frontend/src/components/stories/AudioRecorderWidget.tsx:68-101`
- **Symptom:** Browser tab continues showing the active red microphone recording indicator and capturing audio hardware after navigating away from `AiStoryIntakePage` mid-recording.
- **Root Cause:** Calling `mediaRecorder.stop()` alone does not release the underlying `MediaStream` hardware tracks obtained from `navigator.mediaDevices.getUserMedia()`, nor does it stop an active `SpeechRecognition` instance if the component unmounts before `onstop` fires.
- **Rule:** Hold explicit refs (`mediaStreamRef`, `recognitionRef`, `mediaRecorderRef`) and in the unmount `useEffect` cleanup unconditionally stop `mediaRecorderRef`, iterate `mediaStreamRef.current?.getTracks().forEach(t => t.stop())`, and call `recognitionRef.current?.abort()`.

### GOT-012: API Client Offline Fallback Guards Must Rethrow ApiError When status > 0
- **File / Area:** `frontend/src/api/stories-mcp-api.ts:435-625`
- **Symptom:** Server-side `400 ValidationFailed`, `401 Unauthorized`, `403 Forbidden`, or `404 Not Found` responses on story intake and MCP token endpoints were silently swallowed and replaced with synthetic demo data (`buildLocalStoryDraftFallback`), hiding real RBAC/validation errors from users and tests.
- **Root Cause:** Offline workbench `catch (err)` blocks returned local fallback objects for all errors instead of distinguishing network-unreachable/offline errors from real HTTP error responses.
- **Rule:** In API client wrappers that support offline workbench fallbacks, always check `if (err instanceof ApiError && err.status > 0) throw err;` at the top of the `catch` block so genuine HTTP 4xx/5xx responses propagate to the UI and error boundary.

### GOT-013: Stale or Invalid Session Cookies Blocking Anonymous Access on Public Instances
- **File / Area:** `backend/src/services/rbac.rs:205-240`
- **Symptom:** Anonymous visitors to a public family tree (`enable_public_access = true`) receive `401 Unauthorized` instead of being granted public tree viewing rights.
- **Root Cause:** When the visitor's browser held an expired, invalidated, or deactivated session cookie from a previous session, `PublicOrAuth` previously executed `AuthService::authenticate_session(state, &token).await?`, which short-circuited with `AppError::NotAuthenticated` (or `AppError::SessionExpired`) before checking `settings.enable_public_access`.
- **Rule:** In `FromRequestParts` for `PublicOrAuth`, handle `AuthService::authenticate_session` errors and deactivated user states gracefully: if the token is invalid, expired, or deactivated, check `settings.enable_public_access`. If public access is enabled, log at debug level and yield `PublicOrAuth::AnonymousPublic` rather than rejecting with 401.

### GOT-014: Serialized "null" or "undefined" Tokens in Browser Storage Causing Permanent 401 Loops
- **File / Area:** `frontend/src/api/core.ts:150-205`, `frontend/src/lib/auth-context.tsx:90-105`
- **Symptom:** User experiences perpetual 401 Unauthorized errors across all API endpoints after logging out or having a session expire, persisting even across browser restarts or page reloads.
- **Root Cause:** If client code or an external caller passed `null` or `undefined` as a serialized string or improperly initialized storage, `localStorage.getItem(SESSION_TOKEN_STORAGE_KEY)` returned `"null"` or `"undefined"`. Because non-empty strings are truthy, `getAuthToken()` returned this literal string, prompting `apiFetch` to send `Authorization: Bearer null` on all subsequent requests, which the backend rejected with 401.
- **Rule:** Sanitize token reads and writes defensively (`cleanToken`) by normalizing `"null"`, `"undefined"`, empty strings, and pure whitespace to `null` and evicting them from storage immediately. In `apiFetch`, automatically purge stored tokens via `setAuthToken(null)` whenever a 401 or `not_authenticated` / `session_expired` error code is received.

### GOT-015: Story Intake Commit Deserialization Failure on draft_person_id vs draft_temp_id
- **File / Area:** `backend/src/services/story_intake/types_api.rs:65-71`, `frontend/src/api/stories-mcp-api.ts:555-565`, `frontend/src/lib/error-formatter.ts:41-48`
- **Symptom:** Committing an AI story draft (`POST /api/v1/ai/drafts/{id}/commit`) fails with `400 Bad Request` (`validation_failed`) reporting `match_overrides[0]: missing field draft_temp_id`, preventing candidate entities from being committed to the family tree.
- **Root Cause:** Naming discrepancy between client/MCP callers sending `draft_person_id` (and `target_person_id`) and the backend Rust struct `DraftMatchOverrideDto` expecting `draft_temp_id` and `matched_person_id`. Additionally, `getLocalizedErrorMessage` previously masked specific validation details behind the generic "Validation failed" string, hiding the exact missing field name.
- **Rule:** Use `#[serde(alias = "...")]` on Rust DTO structs to support alternative parameter naming aliases across client, MCP, and external callers. On the frontend, provide both aliases (`draft_temp_id` + `draft_person_id`, `matched_person_id` + `target_person_id`) in client commit payloads for backward compatibility, and preserve specific backend error messages for `validation_failed` in `error-formatter.ts`.

_Last curated: 2026-11-09 (GOT-015 Story intake commit deserialization aliases & error formatting)_
