# UX Review — Error Code Localization — 2026-10-28

## Verdict: OK
Screens & Components reviewed:
- Centralized Error Formatter: `frontend/src/lib/error-formatter.ts`
- Toast Notification Pipeline: `useToast`, `useUndoToast.ts`, `toast.tsx`
- Dialog & In-Page Alerts:
  - Auth: `LoginCard.tsx`, `RegisterInviteCard.tsx`, `MagicLinkVerifyCard.tsx`, `SetupWizardCard.tsx`
  - Person & Relatives: `PersonEditDialog.tsx`, `QuickAddRelativeDialog.tsx`, `LifeEventDialog.tsx`, `DeleteImpactDialog.tsx`
  - Profile & Notes: `PersonProfilePage.tsx`, `ProfileHeroHeader.tsx`, `NoteEditDialog.tsx`
  - Places & Quality: `PlacesPage.tsx`, `PlaceFormDialog.tsx`, `PlaceMergeDialog.tsx`, `DataQualityPage.tsx`, `PersonMergePage.tsx`
  - Settings & Administration: `GeneralSettingsPanel.tsx`, `ProfileSettingsPanel.tsx`, `SecurityPasskeysPanel.tsx`, `FamilyMembersInvitesPanel.tsx`, `RoleChangeDialog.tsx`, `UserDeactivateDialog.tsx`, `BackupsSettingsPanel.tsx`, `TreeSnapshotsCard.tsx`, `GedcomImportCard.tsx`
  - Media & Canvas: `MediaUploadDialog.tsx`, `TreeCanvas.tsx`, `TreePrivacyPreviewCard.tsx`, `DashboardPage.tsx`, `HistoryPage.tsx`
- Localization Catalogs: `frontend/src/i18n/en.json`, `frontend/src/i18n/ru.json` (`errors.*` namespace)

Locales & Viewports Verified:
- Russian (`ru`) and English (`en`)
- Desktop (1280px+) and Mobile (390px)
- Light & Dark themes
- Automated parity & mapping suite: `frontend/tests/i18n/parity.test.ts` (115 total frontend tests pass)

---

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-001** | Fix mobile toolbar overlap and bottom navigation occlusion on tree canvas | **fixed** |
| **UX-002** | Resolve multi-touch pinch-to-zoom race condition and card long-press drag threshold | **fixed** |
| **UX-003** | Wire centralized `formatPersonName` into all UI components and fix fallback | **fixed** |
| **UX-004** | Remove prominent "Living" pill badge from `PersonDrawer` and `PersonBottomSheet` [spec-change] | **fixed** |
| **UX-005** | Update `LifeSpan` component to display genealogical dagger `†` for deceased | **fixed** |
| **UX-006** | Fix PWA offline banner top safe-area inset and consolidate install actions | **fixed** |
| **UX-007** | Fix `BottomNav` accessibility label and replace raw 36px buttons | **fixed** |
| **UX-008** | Parametrize family archive instance title in PWA install copy | **fixed** |
| **UX-009** | Elevate toast container above mobile bottom navigation with safe-area clearance | **fixed** |
| **UX-010** | Add missing `role="radiogroup"`, visible focus rings, and mobile layout to conflict modal | **fixed** |
| **UX-011** | Fix untranslated `{count}` token in diff toggle buttons | **fixed** |
| **UX-012** | Prevent diff values from overflowing card boundaries on mobile viewports | **fixed** |
| **UX-013** | Format raw database transaction labels with human-friendly descriptions | **fixed** |
| **UX-014** | Optimize dialog sizing and instruction placement in revision & conflict modals | **fixed** |
| **UX-015** | Keyboard inaccessible stat cards in `TreeGlanceWidget` | **fixed** |
| **UX-016** | Missing visible focus ring on interactive SVG slices in `FanChartCanvas` | **fixed** |
| **UX-017** | Keyboard inaccessible map cluster pins in `PlacesMapView` | **fixed** |
| **UX-018** | Missing column/source identifiers on mobile stacked layout in `PersonMergePage` | **fixed** |
| **UX-019** | Suffix dagger notation `(1989†)` in `AnniversariesWidget` | **fixed** |
| **UX-020** | Screen reader list navigation distortion from `aria-hidden` intermediate `<li>` arrows in `KinshipFinderDialog` | **fixed** |
| **UX-021** | Keyboard zoom shortcuts or visible hint in `PlacesMapView` and `FanChartCanvas` | suggestion (open) |

---

## Detailed Review Findings

### 1. Elimination of Raw English & Technical Leaks
- **Previous defect:** Uncaught network exceptions (`TypeError: Failed to fetch`), backend error codes (`version_conflict`, `sole_admin_protected`), and unlocalized strings (`Forbidden: viewer cannot edit`) previously leaked raw technical English text directly into Russian toast descriptions and alert boxes.
- **Current implementation:** `getLocalizedErrorMessage(err, t)` intercepts all error types across four tiers:
  1. Specific `err.code` -> `errors.<code_key>`
  2. HTTP status code fallback (0 -> network_error, 401 -> not_authenticated, 403 -> forbidden, 404 -> not_found, 409 -> version_conflict, 429 -> rate_limited, 500 -> internal_error)
  3. Browser fetch/network heuristics (`Failed to fetch`, `Network request failed`, `Load failed`) -> `errors.network_error`
  4. Localized fallback -> `errors.unknown` / `common.errorLoading`
- Over 35 frontend components and pages now route errors through `getLocalizedErrorMessage`, guaranteeing zero raw English leaks in Russian locale.

### 2. Typography, Tone & Readability
- **Tone & Respect:** Error messages are written in polite, natural language tailored to family users (avoiding developer/database jargon like "Foreign key constraint failure" or "Axum 500 Internal").
- **Exemplary Russian Copy:**
  - `errors.network_error`: *"Не удалось связаться с сервером. Проверьте сетевое подключение."* — clear, actionable.
  - `errors.session_expired`: *"Срок действия сессии истёк. Пожалуйста, войдите снова."* — polite, reassuring.
  - `errors.version_conflict`: *"Запись была изменена другим пользователем. Обновите страницу и просмотрите изменения."* — explains the concurrent edit clearly.
  - `errors.cycle_detected`: *"Невозможно установить родство: обнаружена циклическая зависимость."* — clean genealogical explanation.
  - `errors.sole_admin_protected`: *"Нельзя понизить или отключить единственного активного администратора."* — clear administrative guardrail.
- **Punctuation:** All error sentences conclude with standard punctuation (periods), maintaining typographic consistency.
- **Font & Hierarchy:** In `<Alert variant="danger">` and `addToast({ type: 'error', ... })`, title and description maintain clear font-weight contrast (`font-semibold` title vs `text-xs/text-sm text-secondary` body).

### 3. Visual Presentation & Accessibility
- **Toast Notifications:**
  - Toasts appear with high contrast red border/icon accents (`text-danger`, `border-danger/30`), clear `role="status"` and `aria-live="polite"`.
  - Positioned with bottom safe-area clearance above mobile `BottomNav` (`bottom-[calc(4.75rem+env(safe-area-inset-bottom,0px))]`).
- **Form & Dialog Alerts:**
  - Form dialog errors (`PersonEditDialog`, `PlaceFormDialog`, `RoleChangeDialog`) appear directly above action buttons or below field sets, drawing immediate attention without blocking form recovery.
  - Focus is retained or returned smoothly, allowing keyboard users to fix invalid inputs immediately.

---

## Positive Notes
- **Comprehensive Coverage:** Complete consistency achieved across auth, tree canvas, profile, places, media, history, and settings panels.
- **Full Parity Test:** Automated vitest test enforces parity and verifies localized error resolution in both `ru` and `en`.
- **Zero Technical Regressions:** 115 unit, component, and accessibility tests pass cleanly across the entire frontend suite.
