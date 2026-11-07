<UX Review — Milestone 3.2 (AI Story Intake & MCP) & Whole-Project Audit — 2026-11-07>
## Verdict: NEEDS FIXES
Screens reviewed:
- App Shell & Navigation (`AppShell`, `BottomNav`, `SideNav`, `CommandPalette`, `PublicGuestBanner`)
- Home / Family Dashboard (`/`, `/dashboard`)
- Interactive Family Tree Canvas (`/tree`)
- Person Profile Page (`/persons/:id`)
- People Directory & Surnames Index (`/people`, `/people/surnames`)
- Places & Map Directory (`/places`)
- Media Archive & Face Tagging Lightbox Studio (`/media`)
- Printable PDF Studio (`/export`, `/print`)
- Data Quality & Place Hierarchy Standardization (`/quality`)
- Person Comparison & Merge (`/people/merge`)
- Change History & Tree Snapshots (`/history`)
- GEDCOM Import, Export & Media Reconciliation (`/gedcom`)
- AI Family Story Intake & Safe Draft Sandbox (`/stories/intake`)
- Settings & Administration — General, My Profile, Members & Roles, Security & Passkeys + MCP Personal Access Tokens, GEDCOM, Backups (`/settings/*`)
- Authentication & Onboarding Flows (`/login`, `/setup`, `/register/:token`, `/auth/magic`)
- Design System Workbench (`/dev/ui`)

Recordings: `code_and_token_audit_2026_11_07` (plus prior baseline recordings `ux_auth_and_shell`, `ux_dashboard_and_tree`, `ux_profile_and_directory`, `ux_places_and_media`, `ux_settings_and_gedcom`, `ux_quality_history_merge`, `ux_print_export`)

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| `UX-105` | Fix mobile PDF export primary CTA occlusion and preview toolbar overflow | **fixed** (`PrintConfigSidebar.tsx:419`, `PrintExportPage.tsx:266`, `PrintPreviewCanvas.tsx:218-281`) |
| `UX-106` | Fix `MediaLightboxStudio` vertical clipping and implement modal focus trap | **fixed** (`MediaLightboxStudio.tsx:115-138,144-169`, `FaceTagCanvas.tsx:277,316`) |
| `UX-107` | Fix WCAG 2.2 AA color contrast tokens in Light and Dark themes | **fixed** (`frontend/src/index.css:35-100`) |
| `UX-108` | Add graceful broken-image fallback to `MediaCard`, `FaceTagCanvas`, and `Avatar` | **fixed** (`MediaCard.tsx:31,82-104`, `FaceTagCanvas.tsx:66,307-334`, `avatar.tsx:62,79`) |
| `UX-109` | Prevent mobile header brand title truncation (`"Семейн"`) at `390px` in Russian | **fixed** (`app-shell.tsx:238,448-451,458,521-528`) |
| `UX-110` | Fix Russian text overflow in `/quality` SegmentedControl, `/settings` sidebar, and `/people` death date column | **fixed** (`DataQualityPage.tsx:396`, `SettingsPage.tsx:395`, `side-nav.tsx:43`, `DirectoryTable.tsx:264,284`) |
| `UX-111` | Add accessible names (`aria-label`) to all `Select` and `SegmentedControl` components | **still open (partial)** — Fixed in `DirectoryFilterBar.tsx`, `MediaFilterBar.tsx`, `PlacesPage.tsx`, and `PrintPreviewCanvas.tsx`; still missing localized `ariaLabel` on `SideNav` in `DevUiWorkbench.tsx:180` / `side-nav.tsx:24` (tracked in `UX-121`) |
| `UX-112` | Enforce `>= 40x40px` touch targets across all buttons, input adornments, and tree rows | **still open (partial)** — Fixed on `size="sm"` in `button.tsx:27`, `icon-button.tsx:22`, `input.tsx:172,218`, `PublicGuestBanner.tsx:48`, `DirectoryCardList.tsx:86`, `PlacesTreeListCard.tsx:113`, and `auth-layout.tsx:45`; `Button size="xs"` (`button.tsx:26`, `28px`) still lacks `touch-target-expand` and is used in `PersonMergePage.tsx:349,380,407,429` (tracked in `UX-121`) |
| `UX-113` | Fix SVG poster title overlap with tile badge and dark-mode paper background in `PrintPreviewCanvas` | **fixed** (`PrintPosterSvgSheet.tsx:185-238`, `index.css:154-169`) |
| `UX-114` | Remove duplicate page title and subtitle in `PlacesTreeListCard` and `PrintConfigSidebar` | **fixed** (`PlacesTreeListCard.tsx:218`, `PrintConfigSidebar.tsx:96`) |
| `UX-115` | Remove raw snake_case keys, rule codes, and hardcoded English strings in localized views | **still open (partial)** — Fixed in `AuthPages.tsx` and `MediaFilterBar.tsx`; still open in `HistoryPage.tsx:522` and `DataQualityPage.tsx:467-469` (tracked in `UX-125`) |
| `UX-116` | Improve offline fallback consistency on `PersonProfilePage`/`DashboardPage` and add empty state to Active Sessions | **still open** (`PersonProfilePage.tsx:127`, `DashboardPage.tsx:223`, `SecurityPasskeysPanel.tsx:518` — tracked in `UX-125`) |
| `UX-117` | Replace remaining raw `<button>`/`<input>` elements and add dark-mode filter to `PlacesMapView` tiles | **still open (partial)** — Fixed in `gen-date-input.tsx`, `markdown-editor.tsx`, and `PlacesMapView.tsx:318`; raw `<button>` remains in `DevUiWorkbench.tsx:542-552` (tracked in `UX-125`) |
| `UX-118` | Align RU glossary terms (`Песочные часы`, `Центральная персона`, `Диагностика системы`) and destructive button styling in `PersonMergePage` | **still open** (tracked in `UX-126`) |

---

## Tasks for frontend-developer

### UX-119 [major] Define or replace missing Tailwind v4 surface and border tokens (`bg-surface-subtle`, `border-subtle`, `bg-border-strong`) across Story Intake, MCP, and Quality views
- **Screen / viewport / theme / lang:** `/stories/intake`, `/settings/security` (MCP Tokens), `/quality?tab=places_hierarchy`, `/places` — `1440px` & `390px` — Light & Dark — EN & RU
- **Problem:** Eleven component and page files reference Tailwind utility classes `bg-surface-subtle`, `border-subtle`, and `bg-border-strong`. However, `frontend/src/index.css` only defines `--color-surface-muted` (`bg-surface-muted`), `--color-border` (`border-border`), and `--color-border-card` (`border-border-card`) inside `@theme`. Because Tailwind CSS v4 only generates utilities for registered `@theme` variables:
  1. In `DraftSubtreeCanvas.tsx:185,212`, the vertical generation connector lines (`className="h-6 w-0.5 bg-border-strong"`) have no background color (`transparent`), making parent-child connectors between generations invisible in both Light and Dark themes (WCAG 1.4.11 Non-text Contrast).
  2. In `DraftSubtreeCanvas.tsx:87`, neutral confidence cards (`border-subtle`), the subtree canvas container (`DraftSubtreeCanvas.tsx:176`), `AudioRecorderWidget.tsx:351,389,418`, `StoryInputPanel.tsx:190,223`, `DraftEntityListCard.tsx:102,157`, `ReconciliationMatrixCard.tsx:121,214`, `McpTokensPanel.tsx:177,203,237`, `PlaceHierarchyCard.tsx:119,187`, `PlaceHierarchyCustomizeModal.tsx:117`, `PlacesQualityBanner.tsx:24`, `AiStoryIntakePage.tsx:394`, and `error-boundary.tsx:45` lose their intended subtle surface fill and border contrast, causing `border` without a color utility to fall back to `currentColor` (harsh text-colored borders in dark mode).
- **Evidence:**
  - `frontend/src/index.css:6-32` (`@theme` block defines `--color-surface-muted`, `--color-border`, `--color-border-card`, but not `--color-surface-subtle`, `--color-subtle`, or `--color-border-strong`)
  - `frontend/src/pages/AiStoryIntakePage.tsx:394`
  - `frontend/src/components/stories/DraftSubtreeCanvas.tsx:87,131,176,185,197,212`
  - `frontend/src/components/stories/AudioRecorderWidget.tsx:351,389,418`
  - `frontend/src/components/stories/StoryInputPanel.tsx:190,223`
  - `frontend/src/components/stories/DraftEntityListCard.tsx:102,157`
  - `frontend/src/components/stories/ReconciliationMatrixCard.tsx:121,214`
  - `frontend/src/components/settings/McpTokensPanel.tsx:177,203,237`
  - `frontend/src/components/quality/PlaceHierarchyCard.tsx:119,187`
  - `frontend/src/components/quality/PlaceHierarchyCustomizeModal.tsx:117`
  - `frontend/src/components/quality/PlacesQualityBanner.tsx:24`
  - `frontend/src/components/ui/error-boundary.tsx:45`
- **Proposed fix:**
  1. In `frontend/src/index.css` inside `@theme`, register token aliases so both naming conventions work seamlessly across Light and Dark modes:
     ```css
     --color-surface-subtle: var(--color-surface-muted);
     --color-subtle: var(--color-border);
     --color-border-subtle: var(--color-border);
     --color-border-strong: var(--color-border-card);
     ```
  2. Replace `bg-surface-subtle` with `bg-surface-muted`, `border-subtle` with `border-border`, and `bg-border-strong` with `bg-border-card` across the 11 files above for strict token consistency.
- **Acceptance criteria:**
  - [ ] Vertical generation connector lines in `DraftSubtreeCanvas` are visibly rendered (`2px` wide, `--color-border-card`) in Light and Dark themes.
  - [ ] All cards, audio widgets, engine selector bars, MCP token panels, and place hierarchy cards render with subtle muted backgrounds (`--color-surface-muted`) and crisp 1px borders (`--color-border`) instead of `currentColor` borders.
- **Tags:** consistency | a11y | layout

---

### UX-120 [major] Add AI Story Intake (`/stories/intake`) to mobile `BottomNav` "More" sheet, Dashboard quick actions, and unauthenticated route guard
- **Screen / viewport / theme / lang:** `BottomNav` (`390px`), `DashboardPage` (`/`, `1440px` & `390px`), `App.tsx` (`/stories/intake`) — Light & Dark — EN & RU
- **Problem:**
  1. **Mobile `BottomNav` discoverability gap:** On mobile viewports (`390px`), the top header `+ Add` button is hidden (`hidden sm:inline-flex` in `app-shell.tsx:266`). When a mobile user taps the 5th tab "More" (`Ещё`) in `BottomNav` (`frontend/src/components/ui/bottom-nav.tsx:108-139`), the bottom sheet lists `places`, `print`, `history`, `quality`, `gedcom`, and `settings`, but omits `/stories/intake` (`nav.storyIntake`). Mobile users who navigate via the bottom bar cannot discover AI Story Intake.
  2. **Dashboard hero discoverability gap:** On `DashboardPage.tsx:340-361`, the hero call-to-action area only shows "Open Tree" and "Add Person", missing a quick-action button for "Tell Me About My Family" (`/stories/intake`) for Editors and Administrators.
  3. **Unauthenticated route guard:** In `App.tsx:196-206`, `isRestrictedUnauthenticatedRoute` checks `/people`, `/media`, `/places`, `/settings`, `/history`, `/quality`, and `/export`, but omits `/stories/intake`. Furthermore, `AiStoryIntakePage.tsx:60` computes `const effectiveRole = userRole ?? user?.role ?? 'admin';`, defaulting to `'admin'` when `user` is `null` instead of blocking unauthenticated guests.
- **Evidence:**
  - `frontend/src/components/ui/bottom-nav.tsx:108-139` (`moreLinks` omits `/stories/intake`)
  - `frontend/src/pages/DashboardPage.tsx:340-361` (hero CTA row lacks `/stories/intake` button)
  - `frontend/src/App.tsx:196-206,494-508` (`/stories/intake` missing from `isRestrictedUnauthenticatedRoute`)
  - `frontend/src/pages/AiStoryIntakePage.tsx:60` (`effectiveRole` fallback when `user` is null)
- **Proposed fix:**
  1. In `frontend/src/components/ui/bottom-nav.tsx`, import `Sparkles` from `lucide-react` and add `{ id: 'stories', label: t('nav.storyIntake'), href: '/stories/intake', icon: <Sparkles className="h-4 w-4" /> }` to `moreLinks` (and include `currentPath.startsWith('/stories')` in `isMoreActive`).
  2. In `frontend/src/pages/DashboardPage.tsx:340-361`, add a secondary/ghost hero action button for Editors/Admins (`!isViewer`):
     ```tsx
     <Button
       variant="secondary"
       leftIcon={<Sparkles className="h-4 w-4 text-brand" />}
       onClick={() => navigateTo('/stories/intake')}
       data-testid="dashboard-story-intake-btn"
     >
       {t('nav.storyIntake')}
     </Button>
     ```
  3. In `frontend/src/App.tsx:196-206`, add `currentPath === '/stories/intake' || currentPath.startsWith('/stories/')` to `isRestrictedUnauthenticatedRoute`.
- **Acceptance criteria:**
  - [ ] Tapping "More" (`Ещё`) in `BottomNav` at `390px` displays "Tell Me About My Family" (`Расскажите о семье`) and navigates to `/stories/intake`.
  - [ ] `DashboardPage` hero section includes a "Tell Me About My Family" button for Editors and Admins.
  - [ ] Unauthenticated users navigating to `/stories/intake` are redirected to `/login` when public mode is off or shown `AccessDeniedCard` when public mode is on.
- **Tags:** hierarchy | consistency | states

---

### UX-121 [major] Add `touch-target-expand` to `Button size="xs"` and localize `SideNav` default `aria-label` (completing `UX-111` & `UX-112`)
- **Screen / viewport / theme / lang:** `/people/merge`, `/dev/ui`, and all screens using `Button size="xs"` or `SideNav` — `1440px` & `390px` — Light & Dark — EN & RU
- **Problem:**
  1. In `frontend/src/components/ui/button.tsx:26`, `size="xs"` is styled as `h-7 px-2.5 text-xs rounded-md gap-1.5` (`28px` height) **without** `touch-target-expand` (unlike `size="sm"` on line 27 which includes `touch-target-expand`). In `PersonMergePage.tsx:349,380,407,429`, four "Open Profile" buttons (`merge-target-open-profile-chip-btn`, `merge-source-open-profile-chip-btn`, `merge-target-open-profile-btn`, `merge-source-open-profile-btn`) use `size="xs"`, resulting in `28px` hit targets that violate WCAG 2.5.8 and the project's `>= 40x40px` touch target requirement.
  2. In `frontend/src/components/ui/side-nav.tsx:24`, `ariaLabel` defaults to hardcoded English `'Section navigation'`, and `DevUiWorkbench.tsx:180` renders `<SideNav>` without an `ariaLabel` prop, leaving an unlocalized English `<nav aria-label="Section navigation">` landmark when viewing in Russian (`RU`).
- **Evidence:**
  - `frontend/src/components/ui/button.tsx:26` (`xs: 'h-7 px-2.5 text-xs rounded-md gap-1.5'`)
  - `frontend/src/pages/PersonMergePage.tsx:349,380,407,429`
  - `frontend/src/components/ui/side-nav.tsx:24` (`ariaLabel = 'Section navigation'`)
  - `frontend/src/pages/DevUiWorkbench.tsx:180`
- **Proposed fix:**
  1. In `frontend/src/components/ui/button.tsx:26`, add `touch-target-expand` to the `xs` size variant:
     ```ts
     xs: 'h-7 px-2.5 text-xs rounded-md gap-1.5 touch-target-expand',
     ```
  2. In `frontend/src/components/ui/side-nav.tsx`, use `useI18n()` so the default `aria-label` is `ariaLabel ?? t('settings.navAriaLabel')`, and pass `ariaLabel={t('settings.navAriaLabel')}` in `DevUiWorkbench.tsx:180`.
- **Acceptance criteria:**
  - [ ] Every `Button` (`xs`, `sm`, `md`, `lg`) provides at least a `40×40px` interactive hit area (`touch-target-expand` on `xs` and `sm`).
  - [ ] `SideNav` has a localized `aria-label` in both EN and RU on all pages.
- **Tags:** a11y | consistency

---

### UX-122 [minor] Fix 3-step progression badge state and localize default fallback speaker options on `AiStoryIntakePage`
- **Screen / viewport / theme / lang:** `/stories/intake` (`1440px` & `390px`, Light & Dark, EN & RU)
- **Problem:**
  1. **Premature Step 3 completion state:** In `AiStoryIntakePage.tsx:374`, `currentStage` is computed as `const currentStage = activeDraft ? 2 : 1;`. However, in lines 435–446, Step 2 (`stories.steps.step2`) and Step 3 (`stories.steps.step3`, "3. Merge into Tree") both check `currentStage >= 2` and render `variant="success"` with a green `<CheckCircle2 />` icon as soon as a draft is extracted. This makes Step 3 ("3. Merge into Tree") look already completed while the user is still reviewing Step 2 and hasn't merged anything yet.
  2. **Hardcoded Russian fallback speakers in English mode:** In `AiStoryIntakePage.tsx:44-48`, `DEFAULT_SPEAKERS` hardcodes Russian Cyrillic names (`'Анна Сергеевна Волкова (I0001)'`, `'Пётр Фёдорович Щеглов (I0002)'`, `'Мария Ивановна Волкова (I0003)'`) regardless of active locale. When the page renders in English (`EN`) without live API people, the Anchor Speaker combobox displays untranslated Cyrillic names.
- **Evidence:**
  - `frontend/src/pages/AiStoryIntakePage.tsx:44-48` (`DEFAULT_SPEAKERS`)
  - `frontend/src/pages/AiStoryIntakePage.tsx:374,420-447` (3-Stage Progression Tracker badges)
- **Proposed fix:**
  1. Track a `committedStage` boolean or highlight the active step with `variant="brand"` while completed steps use `variant="success"` and upcoming steps use `variant="neutral"`:
     - When `currentStage === 1`: Step 1 is `brand` (active), Step 2 is `neutral`, Step 3 is `neutral`.
     - When `currentStage === 2` (draft active): Step 1 is `success` (`<CheckCircle2 />`), Step 2 is `brand` (active), Step 3 is `neutral` (`<GitMerge />`).
  2. Localize `DEFAULT_SPEAKERS` in `AiStoryIntakePage.tsx` based on `locale === 'ru'` vs `'en'` (e.g., `'Anna Sergeyevna Volkova (I0001)'`, `'Pyotr Fyodorovich Shcheglov (I0002)'`, `'Maria Ivanovna Volkova (I0003)'` in EN).
- **Acceptance criteria:**
  - [ ] Before draft extraction, Step 1 is highlighted as current and Steps 2–3 are neutral.
  - [ ] While reviewing a staged draft, Step 1 shows completed (`success`), Step 2 shows active (`brand`), and Step 3 remains `neutral` until committed.
  - [ ] Fallback speaker options match the active UI locale (`EN` vs `RU`).
- **Tags:** states | copy | hierarchy

---

### UX-123 [minor] Improve `AudioRecorderWidget` accessibility (`aria-live`) and primary button sizing for older relatives
- **Screen / viewport / theme / lang:** `/stories/intake` (`AudioRecorderWidget`, `1440px` & `390px`, Light & Dark, EN & RU)
- **Problem:**
  1. In `AudioRecorderWidget.tsx:354-384`, the recording status banner (`Recording voice story…`, `Recording paused`, `Audio clip ready for transcription`) and timer (`00:00 / 10:00`) update dynamically without an `aria-live="polite"` region. Screen-reader users receive no announcement when recording starts, pauses, stops, or finishes.
  2. In `AudioRecorderWidget.tsx:433-511`, primary voice recording and transcription buttons (`Record Voice`, `Pause`, `Resume`, `Stop`, `Transcribe Audio`) use `size="sm"` (`32px` visual height). Because voice story recording is specifically designed for older, non-technical relatives on mobile phones and laptops, `32px`-tall primary controls look cramped compared to the `40px` (`size="md"`) standard used for primary actions.
- **Evidence:**
  - `frontend/src/components/stories/AudioRecorderWidget.tsx:354-384` (status header missing `aria-live="polite"`)
  - `frontend/src/components/stories/AudioRecorderWidget.tsx:433-511` (`size="sm"` on primary recording controls)
- **Proposed fix:**
  1. Add `aria-live="polite"` to the recording status text container in `AudioRecorderWidget.tsx:369`.
  2. Upgrade the primary action buttons (`start-recording-btn`, `stop-recording-btn`, `transcribe-audio-btn`) from `size="sm"` to `size="md"`.
- **Acceptance criteria:**
  - [ ] Recording status transitions are announced politely to assistive technologies via `aria-live="polite"`.
  - [ ] Primary voice recording and transcription buttons render at `size="md"` (`40px` height).
- **Tags:** a11y | hierarchy

---

### UX-124 [minor] Fix `McpTokensPanel` endpoint URL mobile overflow and add confirmation before revoking Personal Access Tokens
- **Screen / viewport / theme / lang:** `/settings/security` (`McpTokensPanel`, `390px` & `1440px`, Light & Dark, EN & RU)
- **Problem:**
  1. In `McpTokensPanel.tsx:176-180`, the MCP endpoint badge `<code className="rounded-lg ... font-mono text-xs text-secondary ...">{mcpEndpointUrl}</code>` inside `CardHeader` lacks `break-all` or `max-w-full`, causing long domain URLs to overflow horizontally on `390px` mobile viewports.
  2. In `McpTokensPanel.tsx:341-351`, clicking `Revoke` (`revoke-mcp-token-btn-${token.id}`) immediately deletes the Personal Access Token without confirmation. Because revoking a PAT immediately breaks connected external MCP clients (Claude Desktop, Cursor) and tokens cannot be recovered, destructive revocation should require confirmation or use `ConfirmDialog`.
- **Evidence:**
  - `frontend/src/components/settings/McpTokensPanel.tsx:176-180` (`<code>` without `break-all`)
  - `frontend/src/components/settings/McpTokensPanel.tsx:148-162,341-351` (`handleRevokeToken` executes immediately without confirmation)
- **Proposed fix:**
  1. Add `break-all max-w-full` to the `<code>` element displaying `mcpEndpointUrl` in `McpTokensPanel.tsx:177`.
  2. Add a `ConfirmDialog` (or confirmation step) before revoking a Personal Access Token, or provide an explicit confirmation modal with the token name.
- **Acceptance criteria:**
  - [ ] `McpTokensPanel` has `0px` horizontal overflow at `390px` viewport width even with long hostnames.
  - [ ] Revoking an MCP Personal Access Token prompts for confirmation before permanent deletion.
- **Tags:** layout | states | consistency

---

### UX-125 [minor] Complete remaining fixes for `UX-115`, `UX-116`, and `UX-117` (raw codes in `/history` & `/quality`, offline fallbacks, Active Sessions empty state, raw `<button>` in Workbench)
- **Screen / viewport / theme / lang:** `/history`, `/quality`, `/persons/:id`, `/`, `/settings/security`, `/dev/ui` — `1440px` & `390px` — Light & Dark — EN & RU
- **Problem:** Four minor items from the previous review remain open in the codebase:
  1. **Raw snake_case field keys and uppercase rule codes (`UX-115`):**
     - `HistoryPage.tsx:522` renders `${translatedField} (${d.field})`, exposing raw database column names like `(birth_place)` and `(given_name)` alongside localized labels.
     - `DataQualityPage.tsx:467-469` renders `<Badge variant="neutral" size="sm">{issue.code}</Badge>`, exposing raw uppercase developer identifiers like `DEATH_BEFORE_BIRTH` and `MARRIED_TOO_YOUNG` right next to the translated rule title.
  2. **Offline fallback consistency (`UX-116`):**
     - `PersonProfilePage.tsx:127` sets `setNotFoundError(t('profile.notFoundDesc'))` when the backend is unreachable instead of falling back to `buildDefaultPersonProfile(personId)`.
     - `DashboardPage.tsx:223` initializes with `buildCleanDashboardPayload()` (all zeros) instead of falling back to `buildDefaultDashboardPayload()` when `dashboardApi.get()` fails offline.
  3. **Active Sessions empty state (`UX-116`):**
     - `SecurityPasskeysPanel.tsx:518` renders an empty `<div className="divide-y divide-border">` when `sessions.length === 0` with no empty-state message.
  4. **Raw `<button>` in Workbench (`UX-117`):**
     - `DevUiWorkbench.tsx:542-552` uses a raw `<button>` element for `GENDATE_PRESETS` instead of `<Button size="xs" variant="secondary">`.
- **Evidence:**
  - `frontend/src/pages/HistoryPage.tsx:522`
  - `frontend/src/pages/DataQualityPage.tsx:467-469`
  - `frontend/src/pages/PersonProfilePage.tsx:125-133`
  - `frontend/src/pages/DashboardPage.tsx:221-245`
  - `frontend/src/components/settings/SecurityPasskeysPanel.tsx:518-561`
  - `frontend/src/pages/DevUiWorkbench.tsx:542-552`
- **Proposed fix:**
  1. In `HistoryPage.tsx:522`, render `{translatedField}` without `(${d.field})` when a translation exists. In `DataQualityPage.tsx:467-469`, remove the redundant raw `{issue.code}` badge (since `{getRuleTitle(issue)}` already displays the human-readable rule name).
  2. In `PersonProfilePage.tsx:127`, only show `notFoundError` on HTTP 404 (`ApiError` status 404); on network/offline errors in standalone preview, fall back to `buildDefaultPersonProfile(personId)`. In `DashboardPage.tsx:241`, fall back to `buildDefaultDashboardPayload()` when `dashboardApi.get()` fails with a network error.
  3. In `SecurityPasskeysPanel.tsx:518`, render a muted empty message when `sessions.length === 0`.
  4. In `DevUiWorkbench.tsx:542-552`, replace the raw `<button>` with `<Button key={preset} size="xs" variant="secondary" onClick={() => setGenDateSample(preset)}>{preset}</Button>`.
- **Acceptance criteria:**
  - [ ] `/history` and `/quality` show zero raw `snake_case` or `UPPER_SNAKE_CASE` internal identifiers to end users.
  - [ ] `PersonProfilePage` and `DashboardPage` degrade gracefully when offline, and `SecurityPasskeysPanel` displays an empty message when `sessions` is empty.
  - [ ] Zero raw `<button>` elements exist outside `frontend/src/components/ui/`.
- **Tags:** copy | states | consistency

---

### UX-126 [suggestion] Align RU glossary terms (`Песочные часы`) and destructive merge button styling in `PersonMergePage`
- **Screen / viewport / theme / lang:** `/tree`, `/people/merge` — `1440px` & `390px` — Light & Dark — RU & EN
- **Problem:**
  1. `frontend/src/i18n/ru.json:626` translates `tree.modes.hourglass` as `"Две ветви"`, whereas `print.layoutHourglass` (`ru.json:1951`) uses `"Песочные часы (Оба)"` and `docs/memory-bank/glossary.md` defines `"Песочные часы"` as the standard Russian term for Hourglass view.
  2. In `PersonMergePage.tsx:624,663`, the `Confirm & Merge` button (`merge-open-confirm-btn` and `merge-execute-confirm-btn`) uses `variant="primary"` instead of `variant="danger"` (or warning accent) to signal that merging two person records is a high-impact archival operation.
- **Evidence:**
  - `frontend/src/i18n/ru.json:626` (`"hourglass": "Две ветви"`)
  - `frontend/src/pages/PersonMergePage.tsx:624-631,663-671`
- **Proposed fix:**
  1. Update `tree.modes.hourglass` in `frontend/src/i18n/ru.json:626` to `"Песочные часы"` (updating any snapshot/test assertions if applicable).
  2. Consider `variant="danger"` on the final confirmation button inside the `Confirm Person Merge` dialog (`merge-execute-confirm-btn` in `PersonMergePage.tsx:664`).
- **Acceptance criteria:**
  - [ ] Hourglass mode uses consistent Russian terminology (`"Песочные часы"`) across `/tree` and `/export`.
- **Tags:** copy | hierarchy

---

## Positive Notes
- **Rapid Resolution of High-Severity Issues (`UX-105`–`UX-110`, `UX-113`, `UX-114`):** All critical and major layout, contrast, broken-image, lightbox focus-trap, and mobile Russian header truncation issues from the previous full audit have been cleanly resolved.
- **Thoughtful AI Story Intake Architecture (`/stories/intake`):** The split-screen workbench with Anchor Speaker selection, dual Extraction Engine picker (`Gemini 3.8 Flash` vs `Built-in Offline`), visual confidence badges (`High Confidence`, `Needs Review`, `Clarification Needed`), interactive clarification loop, and explicit per-person reconciliation (`Enrich Existing` vs `Create New`) with 1-click Undo Toast makes AI-assisted genealogy safe and approachable for non-technical relatives.
- **Clean Place Hierarchy Standardization UI (`/quality?tab=places_hierarchy`):** The side-by-side comparison of flat/unparsed place strings against the proposed indented jurisdiction chain (`CornerDownRight` steps with action badges and candidate parent distance buttons) makes complex gazetteer cleanup intuitive.
- **Scoped MCP Token Security (`McpTokensPanel`):** The one-time raw token reveal banner with `CopyButton`, explicit scope checkboxes (`read:tree`, `write:draft`, `write:direct`, `admin:maintenance`), and RBAC filtering for non-admin users provide clear security boundaries.
</UX Review — Milestone 3.2 (AI Story Intake & MCP) & Whole-Project Audit — 2026-11-07>
