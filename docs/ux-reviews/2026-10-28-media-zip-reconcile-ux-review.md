# UX Review — Milestone 2.1: Gramps Web Media Bundle Reconciliation & Large Archive Ingestion — 2026-10-28

## Verdict: OK
Screens reviewed:
- `/settings/gedcom`: `GedcomMediaReconcileCard.tsx`, `GedcomSettingsPanel.tsx` (Missing stubs alert, 500MB ZIP dropzone, live upload progress bar, post-upload statistics card with metrics).
- `/media`: `MediaUploadDialog.tsx` (.zip bundle selection notice, 3-stage progress indicators, batch reconciliation summary modal).

Viewports & Locales Verified:
- Desktop (1440px / 1280px), Tablet (768px), and Mobile (390px / 360px).
- Light and Dark themes.
- English (`en`) and Russian (`ru`).

Recordings / Visual Evidence:
- `settings_gedcom_desktop_ru_light`
- `settings_gedcom_desktop_ru_dark`
- `settings_gedcom_mobile_390_ru_dark`
- `media_upload_dialog_desktop_1440_ru_dark`
- `media_upload_dialog_mobile_390_ru_dark`
- Automated test suite: `tests/components/gedcom-settings.test.tsx`, `tests/components/media-gallery.test.tsx`, `tests/components/ui-and-a11y.test.tsx`, `tests/i18n/parity.test.ts` (all 121 tests pass)

---

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-022** | DropdownMenu triggers (`AppShell` nav more, +add, user profile) inaccessible to keyboard navigation | open |
| **UX-023** | Modal Dialogs and Drawers lack keyboard Tab focus trapping and initial focus management | open |
| **UX-024** | Sub-40px touch targets and raw `<button>` elements in inputs and media controls | open |
| **UX-025** | Misleading "Person record not found" alert on network or server failure | open |
| **UX-026** | Header title truncation and right action crowding on narrow mobile viewports | open |
| **UX-027** | Missing keyboard navigation (ArrowLeft / ArrowRight) in Media Lightbox Studio | open |
| **UX-028** | Theme switcher button width crowding in mobile "More" drawer | suggestion (open) |
| **UX-029** | Tree canvas pointer capture swallows left clicks on person cards; keyboard activation missing | open |
| **UX-030** | Mobile `PersonBottomSheet` missing authoring actions (Add Mother, Add Spouse, Add Event, Delete) | open |
| **UX-031** | Face tags on photograph canvas keyboard inaccessible; missing focus rings, tooltip disclosure, and delete | open |
| **UX-032** | Non-admin authorized users (Editors & Viewers) locked out of Profile and Passkeys in mobile "More" drawer | open |
| **UX-033** | Invitation creation modal lacks 'Admin' role selection option | open |
| **UX-034** | Authenticated mobile header button congestion and title squeeze | open |
| **UX-035** | Destructive "Delete Person" button in `PersonDrawer` lacks visual danger treatment | open |
| **UX-036** | Form error alerts lack programmatic association (`aria-invalid`, `aria-describedby`) with invalid fields | open |
| **UX-037** | Duplicate counts in `MediaUploadDialog` post-ingestion stat cards and missing pluralization resilience | **fixed** |
| **UX-038** | Live progress announcement missing `aria-live` and `ProgressBar` lacks accessible localized label | **fixed** |
| **UX-039** | Misleading imperative verb phrase in static header badge of `GedcomMediaReconcileCard` | **fixed** |
| **UX-040** | Raw `<button>` with sub-40px touch target and missing keyboard focus on scrollable unmatched files list | **fixed** |
| **UX-041** | Missing focus management on post-upload reconciliation completion | **fixed** |

---

## Tasks for frontend-developer (Resolved)

### UX-037 [major] Duplicate counts in `MediaUploadDialog` post-ingestion stat cards and missing pluralization resilience — FIXED
- **Status:** **fixed**
- **Resolution & Verification:**
  - Removed duplicate `{count}` interpolation from stat card label keys (`archiveFilesProcessed`, `archiveReconciledStubs`, `archiveNewlyCreated`, `archiveUnmatched`) in both `en.json` and `ru.json`. The prominent number is now rendered once cleanly beneath the title.
  - Replaced the vulnerable Russian phrasing in `ru.json` (`gedcom.reconcile.missingStubsAlertTitle`) with plural-resilient text: `"Медиафайлы без прикреплённых изображений: {count}"`, ensuring grammatical correctness for all quantities.

- **Screen / viewport / theme / lang:** `/media` (`MediaUploadDialog.tsx:210-243`), all viewports, light & dark themes, EN & RU.
- **Problem:**
  1. In `MediaUploadDialog.tsx`, each of the 4 summary stat cards embeds `{count}` into both the label string (`archiveFilesProcessed`, `archiveReconciledStubs`, `archiveNewlyCreated`, `archiveUnmatched`) AND renders the big metric number in the row below:
     - Card 1: "129 files processed" / "129"
     - Card 2: "129 missing stubs reconciled" / "129"
     - Card 3: "0 new photos added" / "0"
     - Card 4: "0 skipped or unmatched" / "0"
     In Russian:
     - "Обработано файлов: 129" / "129"
     - "Согласовано отсутствующих записей: 129" / "129"
     - "Добавлено новых фотографий: 0" / "0"
     - "Пропущено или не сопоставлено: 0" / "0"
     This creates redundant visual noise and cramps the compact card headers on mobile (390px).
  2. In `ru.json`, `gedcom.reconcile.missingStubsAlertTitle` is hardcoded as `"{count} медиафайлов в вашем древе не имеют изображений"`. When `count === 1`, this renders as `"1 медиафайлов в вашем древе не имеют изображений"`, violating Russian grammar.
- **Evidence:** `frontend/src/components/media/MediaUploadDialog.tsx:210-243`, `frontend/src/i18n/en.json:1252-1255`, `frontend/src/i18n/ru.json:1252-1255`, `frontend/src/i18n/ru.json:1444`.
- **Proposed fix:**
  1. In `en.json` and `ru.json`, define clean title labels without `{count}` for stat cards (e.g. `Total Files Processed` / `Обработано файлов`, `Stubs Reconciled` / `Согласовано записей`, `New Photos Added` / `Добавлено новых фото`, `Skipped / Unmatched` / `Не сопоставлено`), allowing the bold number below to represent the metric clearly without redundancy.
  2. Update `gedcom.reconcile.missingStubsAlertTitle` in `ru.json` to plural-resilient phrasing: `"Медиафайлы без прикреплённых изображений: {count}"`.
- **Acceptance criteria:**
  - Stat cards in `MediaUploadDialog` display a clean descriptive title above a single prominent numeric value.
  - In Russian, the missing stubs alert reads grammatically correctly for any count (1, 2, 5, 129).
- **Tags:** layout, copy, consistency

---

### UX-038 [major] Live progress announcement missing `aria-live` and `ProgressBar` lacks accessible localized label — FIXED
- **Status:** **fixed**
- **Resolution & Verification:**
  - Progress containers in both `GedcomMediaReconcileCard.tsx` and `MediaUploadDialog.tsx` now include `role="status"` and `aria-live="polite"`.
  - In `ProgressBar.tsx`, localized `label` and `aria-label` attributes are supported and passed from parents (`uploadingProgress`, `stage1Uploading`, `stage2Matching`, `stage3Thumbnails`).
  - Added indeterminate progress support (`indeterminate={processing}` and `indeterminate={archiveStage === 'matching' || archiveStage === 'thumbnails'}`) with `animate-pulse` and omitting `aria-valuenow` per WAI-ARIA progressbar conventions.

- **Screen / viewport / theme / lang:** `/settings/gedcom` (`GedcomMediaReconcileCard.tsx:206-234`) and `/media` (`MediaUploadDialog.tsx:327-374`), all viewports, light & dark themes, EN & RU.
- **Problem:**
  1. During large ZIP upload and decompression/reconciliation (which can take several seconds to minutes for a 500MB archive), the progress bar in both `GedcomMediaReconcileCard` and `MediaUploadDialog` has no `label` or `aria-label` passed to `ProgressBar`. As a result, the `role="progressbar"` element falls back to hardcoded English `aria-label="Progress"` even in Russian locale (WCAG 2.2 AA Criterion 4.1.2 Name, Role, Value).
  2. The multi-stage indicators (Stage 1: Uploading, Stage 2: Decompressing & matching, Stage 3: Thumbnails) and extracting status text (`gedcom.reconcile.extracting`) lack `role="status"` and `aria-live="polite"`. Screen reader users receive no announcements as the upload progresses or when it transitions to backend extraction.
  3. During backend extraction/matching, the progress bar in `GedcomMediaReconcileCard` stays frozen at 100% full instead of indicating active background processing via `indeterminate={true}`, leading users to suspect the browser tab has crashed or frozen.
- **Evidence:** `frontend/src/components/settings/gedcom/GedcomMediaReconcileCard.tsx:222-232`, `frontend/src/components/media/MediaUploadDialog.tsx:353-372`, `frontend/src/components/ui/progress-bar.tsx:50-56`.
- **Proposed fix:**
  1. In `GedcomMediaReconcileCard.tsx`, pass `label={t('gedcom.reconcile.uploadingProgress')}` and `aria-label={t('gedcom.reconcile.uploadingProgress')}` to `ProgressBar`. When `processing` is true, pass `indeterminate={true}`.
  2. In `MediaUploadDialog.tsx`, pass `label` and `aria-label` with the active stage name to `ProgressBar`. Set `indeterminate={archiveStage === 'matching' || archiveStage === 'thumbnails'}`.
  3. Wrap the progress status messages in `aria-live="polite"` and `role="status"`.
- **Acceptance criteria:**
  - Progress bars have localized `aria-label` in both EN and RU.
  - When backend server decompression/matching is active, the progress bar indicates active indeterminate processing.
  - Screen readers announce stage transitions politely without focus interruption.
- **Tags:** a11y, states, feedback

---

### UX-039 [minor] Misleading imperative verb phrase in static header badge of `GedcomMediaReconcileCard` — FIXED
- **Status:** **fixed**
- **Resolution & Verification:**
  - Replaced the imperative string in `GedcomMediaReconcileCard.tsx` with format metadata key `badgeFormat`.
  - In `en.json`: `"ZIP / GDZ bundle (≤ 500 MB)"`, and in `ru.json`: `"ZIP / GDZ архив (≤ 500 МБ)"`. Badges now consistently convey format metadata rather than resembling clickable button controls.

- **Screen / viewport / theme / lang:** `/settings/gedcom` (`GedcomMediaReconcileCard.tsx:156-158`), all viewports, all themes, EN & RU.
- **Problem:** In `GedcomMediaReconcileCard.tsx`, the card header renders a static badge with the text from `gedcom.reconcile.dropzoneLabel`:
  - English: `"Upload Media Archive (.zip, .gdz)"`
  - Russian: `"Загрузить медиаархив (.zip, .gdz)"`
  Because badges in the design system represent metadata tags/statuses and are non-clickable, using an imperative verb phrase ("Upload Media Archive...") causes user confusion. It appears like an action button that is unresponsive to clicks, in contrast with `GedcomImportCard` which cleanly uses `"GEDCOM 5.5.1 / 7.0 & GEDZIP"`.
- **Evidence:** `frontend/src/components/settings/gedcom/GedcomMediaReconcileCard.tsx:156-158`, `frontend/src/i18n/en.json:1446`, `frontend/src/i18n/ru.json:1446`.
- **Proposed fix:**
  - Update the badge in `GedcomMediaReconcileCard.tsx` to display format/size metadata: e.g. `".ZIP / .GDZ (≤ 500 MB)"` or introduce a localized key `gedcom.reconcile.badgeFormat` ("ZIP / GDZ архив" / "ZIP / GDZ bundle").
- **Acceptance criteria:**
  - The badge in the card header displays format metadata instead of an imperative command phrase.
  - Styling is consistent with the badge in `GedcomImportCard`.
- **Tags:** hierarchy, copy, consistency

---

### UX-040 [minor] Raw `<button>` with sub-40px touch target and missing keyboard focus on scrollable unmatched files list — FIXED
- **Status:** **fixed**
- **Resolution & Verification:**
  - In `FileDropzone.tsx`, replaced raw `<button>` with `IconButton` (`size="sm" variant="ghost"` with `touch-target-expand` ensuring $\ge 40\times 40\text{px}$ touch boundary) and localized `label={t('dropzone.removeFile', { name: item.file.name })}`.
  - In `GedcomMediaReconcileCard.tsx`, the scrollable unmatched filenames container has `tabIndex={0}`, `role="region"`, `aria-label={t('gedcom.reconcile.unmatchedListTitle', ...)}`, and `focus-visible:ring-2` for full keyboard scrollability.
  - Added `break-all` to `samplePaths` text container in `GedcomMediaReconcileCard.tsx` to eliminate horizontal scroll on 360px viewports.

- **Screen / viewport / theme / lang:** `/settings/gedcom` (`GedcomMediaReconcileCard.tsx:315`) and dropzone items (`FileDropzone.tsx:173-180`), all viewports, all themes.
- **Problem:**
  1. In `FileDropzone.tsx` (rendered on both GEDCOM settings and media upload dialog), the remove file button for queued items is implemented as a raw HTML `<button type="button">` with `p-1` (24×24px touch target). This violates the project rule forbidding raw `<button>` elements and WCAG 2.2 AA touch target sizing (≥ 40×40px, Criterion 2.5.8).
  2. In `GedcomMediaReconcileCard.tsx`, the unmatched filenames container is scrollable (`max-h-24 overflow-y-auto`) but lacks `tabIndex={0}` and an accessible label. Keyboard users cannot scroll through unmatched filenames if there are more than fit in the box (WCAG 2.1.1 Keyboard).
  3. Long file paths in `samplePaths` (`GedcomMediaReconcileCard.tsx:182`) lack `break-all` / `break-words`, which can trigger horizontal scrollbar overflow on 360px viewports if paths have no spaces.
- **Evidence:** `frontend/src/components/ui/file-dropzone.tsx:173-180`, `frontend/src/components/settings/gedcom/GedcomMediaReconcileCard.tsx:182, 315`.
- **Proposed fix:**
  1. In `FileDropzone.tsx`, replace the raw `<button>` with an `IconButton` (or `Button variant="ghost" size="xs"`) with minimum 40×40px touch padding.
  2. In `GedcomMediaReconcileCard.tsx`, add `tabIndex={0}`, `role="region"`, and `aria-label={t('gedcom.reconcile.unmatchedListTitle', { count: report.unmatched_filenames.length })}` to the scrollable unmatched filenames box.
  3. Add `break-all` to `samplePaths` `<p>` element.
- **Acceptance criteria:**
  - No raw `<button>` elements exist in `FileDropzone.tsx`. Touch target is ≥ 40×40px.
  - The scrollable unmatched filenames box is focusable and scrollable via keyboard.
  - Long file paths wrap cleanly on 360px viewports without horizontal scrolling.
- **Tags:** a11y, layout, consistency

---

### UX-041 [suggestion] Missing focus management on post-upload reconciliation completion — FIXED
- **Status:** **fixed**
- **Resolution & Verification:**
  - In `GedcomMediaReconcileCard.tsx`, `viewGalleryButtonRef` programmatically receives focus via `useEffect` once `report` is set.
  - In `MediaUploadDialog.tsx`, `archiveDoneRef` programmatically receives focus via `useEffect` once `archiveReport` is displayed.
  - Focus is smoothly directed to the primary completion action rather than resetting to document top.

- **Screen / viewport / theme / lang:** `/settings/gedcom` (`GedcomMediaReconcileCard.tsx:248-348`) and `/media` (`MediaUploadDialog.tsx:197-258`), all viewports, all themes.
- **Problem:** When an archive upload completes, the file dropzone and upload button unmount and are replaced by the reconciliation report/summary card. The user's focus is dropped, resetting to the top of the page/dialog (WCAG 2.4.3 Focus Order).
- **Proposed fix:** Programmatically shift focus to the completion summary card (via `tabIndex={-1}`) or autofocus the primary action button ("View in Media Gallery" in GEDCOM card; "Done" in Media upload dialog).
- **Acceptance criteria:**
  - When the reconciliation report renders, focus is moved to the report summary or primary action button.
- **Tags:** a11y, states

---

## Positive Notes
- **Intuitive Workflow Architecture:** The order of cards in `GedcomSettingsPanel` (`Import GEDCOM` → `Reconcile Media Archive` → `Export Family Tree`) mirrors the real-world migration journey from Gramps Web smoothly.
- **Dual-Purpose Ingestion in Media Gallery:** Automatically detecting `.zip` bundles in `MediaUploadDialog` and switching from individual photo metadata fields to archive bundle reconciliation provides a great archivist workflow without having to visit Settings.
- **Robust 500MB Boundary Handling:** Uploading large ZIP bundles displays byte-level precision (`{loaded} / {total} ({percent}%)`), keeping users well informed throughout large file ingestion.
- **Clear Post-Upload Actionability:** The completed report card provides explicit next steps: "Upload Another Archive" to continue ingestion, or "View in Media Gallery" to jump directly into browsing the newly linked portraits.
- **Pristine Dark Mode Styling:** Dark theme tokens (`bg-surface`, `border-border`, `bg-success/5`, `border-success/30`) provide balanced contrast and legibility without visual glaring.
