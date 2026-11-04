# UX Review — Full Application Audit (All Routes, 1440px & 390px, Light & Dark, EN & RU) — 2026-11-04

## Verdict: NEEDS FIXES

### Screens Reviewed
- **Auth & Setup**:
  - `/login` ([AuthPages.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/AuthPages.tsx), [AuthLayout.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/auth/AuthLayout.tsx))
  - `/setup` ([AuthPages.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/AuthPages.tsx))
  - `/register` ([AuthPages.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/AuthPages.tsx))
- **Core App Shell & Dashboard**:
  - `/` ([DashboardPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DashboardPage.tsx), [app-shell.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/app-shell.tsx), [PublicGuestBanner.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/auth/PublicGuestBanner.tsx))
  - `/tree` ([TreeCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/tree/TreeCanvas.tsx), [TreeHeaderToolbar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/tree/TreeHeaderToolbar.tsx), [TreeFloatingZoomControls.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/tree/TreeFloatingZoomControls.tsx))
- **Directory & Person Profiles**:
  - `/people` ([PeopleDirectoryPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PeopleDirectoryPage.tsx), [DirectoryFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryFilterBar.tsx), [DirectoryTable.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryTable.tsx), [DirectoryCardList.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryCardList.tsx))
  - `/people/surnames` ([SurnamesIndexPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SurnamesIndexPage.tsx), [SurnameAlphaBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/SurnameAlphaBar.tsx), [SurnameCardGrid.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/SurnameCardGrid.tsx))
  - `/persons/:id` ([PersonProfilePage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PersonProfilePage.tsx))
- **Media & Places**:
  - `/media` & Lightbox Studio ([MediaGalleryPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/MediaGalleryPage.tsx), [MediaFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaFilterBar.tsx), [MediaCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaCard.tsx), [MediaLightboxStudio.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaLightboxStudio.tsx), [FaceTagCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/FaceTagCanvas.tsx), [MediaMetadataPanel.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaMetadataPanel.tsx))
  - `/places` & `/places?view=map` ([PlacesPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PlacesPage.tsx), [PlacesTreeListCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesTreeListCard.tsx), [PlacesMapViewCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesMapViewCard.tsx), [PlaceDetailCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlaceDetailCard.tsx))
- **History, Data Quality, Merge & Print Studio**:
  - `/history` ([HistoryPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/HistoryPage.tsx))
  - `/quality` (Warnings & Duplicates tabs — [DataQualityPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DataQualityPage.tsx))
  - `/people/merge` ([MergePeoplePage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/MergePeoplePage.tsx))
  - `/export/print` ([PrintExportPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PrintExportPage.tsx), [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx), [PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx))
- **Settings & Dev Workbench**:
  - `/settings`, `/settings/profile`, `/settings/users`, `/settings/security`, `/settings/gedcom`, `/settings/backups` ([SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx))
  - `/dev/ui` ([DevUiWorkbench.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DevUiWorkbench.tsx))

### Screenshot Archive (196 Screenshots across 1440px & 390px, Light & Dark, EN & RU)
All 196 screenshots are stored in `/Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/`:
- `login-{1440,390}-{light,dark}-{en,ru}.png`
- `setup-{1440,390}-{light,dark}-{en,ru}.png`
- `register-{1440,390}-{light,dark}-{en,ru}.png`
- `dashboard-{1440,390}-{light,dark}-{en,ru}.png`
- `tree-{1440,390}-{light,dark}-{en,ru}.png`
- `people-{1440,390}-{light,dark}-{en,ru}.png`
- `surnames-{1440,390}-{light,dark}-{en,ru}.png`
- `person-profile-{1440,390}-{light,dark}-{en,ru}.png`
- `media-{1440,390}-{light,dark}-{en,ru}.png`
- `media-lightbox-{1440,390}-{light,dark}-{en,ru}.png`
- `places-list-{1440,390}-{light,dark}-{en,ru}.png`
- `places-map-{1440,390}-{light,dark}-{en,ru}.png`
- `history-{1440,390}-{light,dark}-{en,ru}.png`
- `quality-warnings-{1440,390}-{light,dark}-{en,ru}.png`
- `quality-duplicates-{1440,390}-{light,dark}-{en,ru}.png`
- `merge-{1440,390}-{light,dark}-{en,ru}.png`
- `print-{1440,390}-{light,dark}-{en,ru}.png` & `print-preview-390-{light,dark}-{en,ru}.png`
- `settings-tree-{1440,390}-{light,dark}-{en,ru}.png`
- `settings-profile-{1440,390}-{light,dark}-{en,ru}.png`
- `settings-users-{1440,390}-{light,dark}-{en,ru}.png`
- `settings-security-{1440,390}-{light,dark}-{en,ru}.png`
- `settings-gedcom-{1440,390}-{light,dark}-{en,ru}.png`
- `settings-backups-{1440,390}-{light,dark}-{en,ru}.png`
- `dev-ui-{1440,390}-{light,dark}-{en,ru}.png`

---

## Status of Previous Tasks

| ID | Title | Status |
| :--- | :--- | :--- |
| `UX-021` | Replace raw `<input>` in `SettingsPage.tsx` with `TextInput` | **fixed** |
| `UX-022` | Replace raw `<button>` tabs in `PersonProfilePage.tsx` | **fixed** |
| `UX-023` | Replace raw `<button>` in `PlacesTreeListCard.tsx` | **fixed** |
| `UX-024` | Localize hardcoded English strings in `DevUiWorkbench.tsx` | **still open** (`aria-label="Section navigation"` remains hardcoded; consolidated into `UX-111`) |
| `UX-025` | Fall back to demo profile on `/persons/:id` when API fails | **still open** (consolidated into `UX-116`) |
| `UX-026` | Add `aria-label` to filter `<Select>` controls on `/people` | **still open** (consolidated into `UX-111`) |
| `UX-036` | Replace raw `<input type="checkbox">` with `<Checkbox>` | **fixed** |
| `UX-076` | Handle broken image fallback cleanly in `MediaCard` / `Avatar` | **still open** (consolidated into `UX-108`) |
| `UX-097` | Offset SVG poster title below multi-page tile badge on `/export/print` | **still open** (consolidated into `UX-113`) |
| `UX-098` | Remove duplicate PDF download CTA from preview canvas header | **still open** (consolidated into `UX-105`) |
| `UX-099` | Add `ariaLabel` to preview mode `SegmentedControl` on `/export/print` | **still open** (consolidated into `UX-111`) |
| `UX-100` | Hide duplicate mobile section heading in `PrintConfigSidebar` | **still open** (consolidated into `UX-114`) |
| `UX-101` | Localize `"mm"` unit suffix in cut margin selector on `/export/print` | **still open** (consolidated into `UX-115`) |
| `UX-102`..`UX-104` | Phase 3 Printable PDF Expansion tasks | **fixed** |

---

## Tasks for `frontend-developer`

### UX-105 [critical] Fix mobile PDF export CTA occlusion by `BottomNav` and horizontal toolbar overflow on `/export/print`
- **Screen / viewport / theme / lang:** `/export/print` — `390×844` mobile (`EN` & `RU`, Light & Dark)
- **Problem:**
  1. In the **Configuration** mobile tab (`Параметры печати`), the sticky bottom footer containing the primary `"Скачать PDF для печати"` (`"Download Printable PDF"`) button sits at the bottom of the viewport directly underneath the fixed `BottomNav` bar (`h-16 z-40`), completely occluding the primary CTA so mobile users cannot tap it.
  2. In the **Preview** mobile tab (`Предпросмотр`), the header bar of [PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx) packs `"Интерактивный предпросмотр макета"`, the zoom toolbar (`50%`, `75%`, `100%`, `125%`, `По размеру`), and a duplicate `"Скачать PDF для печати"` button into a non-wrapping row that overflows off the right edge of the `390px` viewport, slicing the button text vertically (`"Ск... / PD... / п..."`).
- **Evidence:**
  - Screenshots: [print-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-390-light-ru.png), [print-390-dark-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-390-dark-ru.png), [print-preview-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-preview-390-light-ru.png)
  - Source: [PrintExportPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PrintExportPage.tsx), [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx), [PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx)
- **Proposed fix:**
  1. In [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx) / [PrintExportPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PrintExportPage.tsx), add bottom clearance for the mobile `BottomNav` (`pb-20 md:pb-4` or `bottom-16 md:bottom-0` on the sticky footer) so the `"Скачать PDF для печати"` CTA is completely visible above `BottomNav`.
  2. In [PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx), remove the duplicate `"Скачать PDF для печати"` button from the preview canvas header (resolving `UX-098`) and allow the preview header controls (`SegmentedControl` and zoom pills) to wrap cleanly (`flex-wrap gap-2`) on `< sm` viewports so nothing overflows `390px`.
- **Acceptance criteria:**
  - [ ] On `390×844` mobile in `/export/print` (both Configuration and Preview tabs, in EN and RU), the `"Download Printable PDF"` / `"Скачать PDF для печати"` CTA is 100% visible above `BottomNav` and clickable.
  - [ ] No element inside `PrintPreviewCanvas` overflows horizontally at `390px` width (`scrollWidth <= clientWidth`).
- **Tags:** layout | space | hierarchy | a11y

---

### UX-106 [critical] Fix `MediaLightboxStudio` vertical clipping of left-pane canvas controls and right-pane face-tag cards on `1440×900` desktop, plus focus trapping
- **Screen / viewport / theme / lang:** `/media` (Lightbox Studio modal) — `1440×900` desktop & `390×844` mobile (`EN` & `RU`, Light & Dark)
- **Problem:**
  1. Inside [MediaLightboxStudio.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaLightboxStudio.tsx#L506-L599) (`max-h-[92vh]`), the dual-pane grid (`lg:grid-cols-[minmax(0,1fr)_390px]`) sets `overflow-y-auto` on the grid wrapper while [FaceTagCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/FaceTagCanvas.tsx) enforces `min-h-[460px]` plus a bottom control bar. At `1440×900`, the left column overflows the modal bottom edge, **completely clipping the bottom action toolbar** (`Отметить лица`, `Отдалить`, `100%`, `Приблизить`, `Сброс`, `Скачать`) off-screen!
  2. In the right column ([MediaMetadataPanel.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaMetadataPanel.tsx)), the second tagged person card (`Левшиц Елизавета`) has its bottom action buttons (`Сделать портретом`, `Удалить`) clipped at the bottom border because the right pane does not scroll independently on desktop.
  3. Pressing `Tab` inside `[role="dialog"]` in `MediaLightboxStudio` does not trap keyboard focus inside the modal (WCAG 2.2 SC 2.4.3 Focus Order).
- **Evidence:**
  - Screenshots: [media-lightbox-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/media-lightbox-1440-light-ru.png), [media-lightbox-1440-dark-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/media-lightbox-1440-dark-ru.png), [media-lightbox-1440-light-en.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/media-lightbox-1440-light-en.png)
  - Source: [MediaLightboxStudio.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaLightboxStudio.tsx#L506-L599), [FaceTagCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/FaceTagCanvas.tsx)
- **Proposed fix:**
  1. On desktop (`lg:`), make the dual-pane grid `min-h-0 flex-1 lg:overflow-hidden`.
  2. Constrain the left column (`FaceTagCanvas`) to `lg:h-full min-h-0 flex flex-col`, with the image stage using `min-h-0 flex-1` (reducing `min-h-[460px]` to `min-h-[280px] lg:min-h-0`) so the bottom toolbar (`Отметить лица`, zoom controls, `Скачать`) is always pinned and visible inside the dialog without scrolling.
  3. Give the right column (`MediaMetadataPanel` container) `lg:h-full lg:overflow-y-auto` so metadata and tagged people scroll independently on desktop.
  4. Add a `Tab` / `Shift+Tab` focus trap handler to `MediaLightboxStudio` `[role="dialog"]` and restore focus to the triggering card on close.
- **Acceptance criteria:**
  - [ ] At `1440×900`, opening `MediaLightboxStudio` displays the entire bottom canvas toolbar (`Tag Faces` / `Отметить лица`, zoom controls, `Download` / `Скачать`) without clipping.
  - [ ] The right metadata panel scrolls independently on desktop so all face-tag cards and their action buttons (`Set as Portrait`, `Remove`) are reachable.
  - [ ] `Tab` and `Shift+Tab` cycle focus strictly within `MediaLightboxStudio` while open.
- **Tags:** layout | space | a11y

---

### UX-107 [major] Fix WCAG 2.2 AA color contrast failures across Light and Dark theme tokens, `Badge` variants, and Dark Mode primary buttons
- **Screen / viewport / theme / lang:** All routes (`1440px` & `390px`, Light & Dark themes, `EN` & `RU`)
- **Problem:** Mathematical WCAG 2.2 SC 1.4.3 contrast verification (`>= 4.5:1` for normal/small text) revealed contrast failures in both themes:
  - **Light Mode (`#FAF8F5` canvas / `#FFFFFF` surface / `#F1F3F5` muted)**:
    1. `Badge variant="accent"` (`text-accent` `#C68A34` on `bg-accent/15` `#F6EDE0`): **2.56:1** (fails 4.5:1; used on `/dev/ui`, `/people`, and archival badges).
    2. `Badge variant="warning"` (`text-warning` `#D97706` on `bg-warning/15` `#F9EBDA`): **2.72:1** (fails 4.5:1; heavily used on `/quality` `"Открытых предупреждений: 3"`, `"Предупреждение"`, `"Совпадение 92%"` and `/places` `"Нет"`).
    3. `Badge variant="success"` (`text-success` `#2E7D32` on `bg-success/15` `#E0ECE0`): **4.21:1** (below 4.5:1 for `12px` badge text; used on `/places` `"Геокодировано"`, `/settings` `"Активен"`, `"Готово"`).
    4. `Badge variant="danger"` (`text-danger` `#C53030` on `bg-danger/15` `#F6E0E0`): **4.34:1** (below 4.5:1 for `12px` badge text; used on `/quality` `"Критично"`).
    5. `text-tertiary` (`#6B7280`) on `bg-surface-muted` (`#F1F3F5` table headers, segmented controls, `/history` diff rows, `/people/merge` headers): **4.35:1** (below 4.5:1).
  - **Dark Mode (`#0E1116` canvas / `#171B22` surface / `#212630` muted)**:
    6. `Button variant="primary"` (`text-surface` `#171B22` on `bg-brand` `#4F7EB5`): **4.10:1** (fails 4.5:1 for `12px`/`14px` button labels and makes primary CTAs look dim/murky across `/login`, `/people`, `/export/print`, `/settings`).
    7. `text-brand` (`#4F7EB5`) on `bg-surface` (`#171B22`): **4.10:1**, and on `bg-surface-muted` (`#212630` active nav pill): **3.60:1** (fails 4.5:1).
    8. `Badge variant="brand"` (`text-brand` `#4F7EB5` on `bg-brand/15` `#1F2A38`): **3.45:1** (fails 4.5:1).
    9. `Badge variant="danger"` (`text-danger` `#EF4444` on `bg-danger/15` `#372127`): **3.96:1** (fails 4.5:1).
- **Evidence:**
  - Screenshots: [quality-warnings-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/quality-warnings-1440-light-ru.png), [quality-warnings-1440-dark-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/quality-warnings-1440-dark-ru.png), [places-list-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/places-list-1440-light-ru.png), [print-1440-dark-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-1440-dark-ru.png), [dev-ui-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/dev-ui-1440-light-ru.png), [dev-ui-1440-dark-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/dev-ui-1440-dark-ru.png)
  - Source: [index.css](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/index.css#L13-L60), [badge.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/badge.tsx#L10-L15), [button.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/button.tsx#L11-L16)
- **Proposed fix:**
  Update the CSS theme tokens in [index.css](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/index.css#L13-L60) to mathematically verified WCAG 2.2 AA values (`>= 4.5:1` across surface, muted, and 15% badge tints):
  - **Light theme (`:root`)**:
    - `--color-text-tertiary: #5B6472;` (`5.85:1` on surface, `5.38:1` on `#F1F3F5` muted)
    - `--color-brand-accent: #8E5B12;` (`5.54:1` on surface, `4.64:1` on 15% badge tint)
    - `--color-status-warning: #9A4F00;` (`5.75:1` on surface, `4.84:1` on 15% badge tint)
    - `--color-status-success: #256B29;` (`6.46:1` on surface, `5.24:1` on 15% badge tint)
    - `--color-status-danger: #B82828;` (`5.90:1` on surface, `4.88:1` on 15% badge tint)
  - **Dark theme (`.dark`)**:
    - `--color-brand-primary: #6899D3;` (`5.83:1` on `#171B22` surface, `5.12:1` on `#212630` muted, `4.63:1` on 15% badge tint, and `5.83:1` with `#171B22` dark text on `Button variant="primary"`)
    - `--color-brand-primary-hover: #7EA9DC;`
    - `--color-status-danger: #F87171;` (`6.24:1` on `#171B22` surface, `5.00:1` on 15% badge tint, and `6.24:1` with `#171B22` text on `Button variant="danger"`)
- **Acceptance criteria:**
  - [ ] Every `Badge` variant (`neutral`, `brand`, `accent`, `success`, `warning`, `danger`) achieves `>= 4.5:1` text contrast in both Light and Dark themes.
  - [ ] `text-tertiary` on `bg-surface-muted` achieves `>= 4.5:1` in Light and Dark themes.
  - [ ] `Button variant="primary"` and `text-brand` achieve `>= 4.5:1` in Dark mode.
- **Tags:** a11y | consistency

---

### UX-108 [major] Fix broken image fallback collisions in `MediaCard`, `FaceTagCanvas`, and `Avatar` (`UX-076` regression)
- **Screen / viewport / theme / lang:** `/media` & `MediaLightboxStudio` (`1440px` & `390px`, Light & Dark, `EN` & `RU`)
- **Problem:**
  1. In [MediaCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaCard.tsx#L42-L48), `<img src={asset.file_url} alt={asset.title} />` has no `onError` fallback state. When the demo/offline image URL fails to load, the browser renders a broken-image icon and the raw `alt` text at the top-left of the card **directly underneath the floating year badge** (`"1955"` overlaps `"Mikhail Levshits Archival Portrait (1955)"`). Additionally, `aspectRatioStyle` on portrait items (`M0003`, `1800×2400`) forces an empty broken box twice the height of neighboring cards in the grid.
  2. In [FaceTagCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/FaceTagCanvas.tsx), when the image fails to load, the white face-tag bounding boxes (`border-white/90`) still render in mid-air directly on top of the broken `<img>` alt text.
  3. In [avatar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/avatar.tsx) (visible inside `MediaMetadataPanel` under `ОТМЕЧЕННЫЕ ЛИЦА`), broken face-crop image URLs render a broken image icon and clipped alt text (`"Левши"`) inside the `36×36px` circle instead of falling back to the person's initials (`ЛМ`, `ЛЕ`).
- **Evidence:**
  - Screenshots: [media-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/media-1440-light-ru.png), [media-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/media-390-light-ru.png), [media-lightbox-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/media-lightbox-1440-light-ru.png)
  - Source: [MediaCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaCard.tsx#L38-L55), [FaceTagCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/FaceTagCanvas.tsx), [avatar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/avatar.tsx)
- **Proposed fix:**
  1. Add `const [imgError, setImgError] = useState(false)` with `onError={() => setImgError(true)}` to `MediaCard.tsx`, `FaceTagCanvas.tsx`, and `avatar.tsx`.
  2. When `imgError` is true in `MediaCard.tsx`, render the existing styled `ImageIcon` archival placeholder (using a uniform `aspect-[4/3]` container so broken portrait images do not blow out row heights) instead of the broken `<img>` tag.
  3. When `imgError` is true in `FaceTagCanvas.tsx`, render a centered archival image placeholder in the canvas stage so the broken-image browser icon does not collide with face-tag regions.
  4. When `imgError` is true in `avatar.tsx`, fall back to rendering the `initials` / `fallback` span.
- **Acceptance criteria:**
  - [ ] No broken browser `<img>` icons or overlapping `alt` text appear on `/media` cards, inside `FaceTagCanvas`, or inside `Avatar` circles when image URLs fail to load.
  - [ ] All cards in `/media` maintain a balanced grid height when showing placeholder thumbnails.
- **Tags:** states | layout | consistency

---

### UX-109 [major] Fix mobile header brand title truncation (`"Семейн"`) on `390px` viewports in Russian
- **Screen / viewport / theme / lang:** All `AppShell` routes (`/`, `/tree`, `/people`, `/places`, `/history`, `/quality`, `/settings`, etc.) — `390×844` mobile (`RU`, Light & Dark)
- **Problem:** On `390px` mobile screens in Russian (`RU`), the top header in [app-shell.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/app-shell.tsx#L76-L103) packs 6 elements into a single row: Hamburger button (`44px`), Brand mark (`36px`), Brand title (`Семейный архив Левшиц`), Language toggle (`60px`), Theme toggle (`44px`), and the `"Войти"` (`Sign In`) button (`96px`). This leaves only `~55px` for the brand title, truncating `"Семейный архив Левшиц"` mid-word to `"Семейн"` (`6 characters`, without even room for a proper word) right against the `[RU]` button. Note that `BottomNav` already displays a dedicated `"Войти"` tab item at the bottom of the screen for unauthenticated guests.
- **Evidence:**
  - Screenshots: [places-list-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/places-list-390-light-ru.png), [history-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/history-390-light-ru.png), [quality-warnings-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/quality-warnings-390-light-ru.png), [settings-tree-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/settings-tree-390-light-ru.png)
  - Source: [app-shell.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/app-shell.tsx#L76-L103)
- **Proposed fix:**
  In [app-shell.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/app-shell.tsx#L207-L228), hide the redundant top-bar `"Войти"` / `"Sign in"` button on extra-small viewports (`hidden sm:inline-flex`, since `BottomNav` already provides `"Войти"` on mobile and the mobile drawer can also include it), or move `LanguageToggleButton` / `ThemeToggleButton` into the mobile navigation drawer on `< sm` viewports so `"Семейный архив Левшиц"` has at least `160px+` to display legibly.
- **Acceptance criteria:**
  - [ ] At `390px` width in Russian (`RU`), the top header displays `"Семейный архив Левшиц"` (or at minimum `"Семейный архив..."`) without chopping the first word to `"Семейн"`.
- **Tags:** layout | space | hierarchy

---

### UX-110 [major] Fix Russian text overflow and truncation in `/quality` SegmentedControl, `/settings` desktop sidebar, and `/people` death date column
- **Screen / viewport / theme / lang:**
  - `/quality` (`390px` mobile, `RU`, Light & Dark)
  - `/settings` (`1440px` desktop, `RU`, Light & Dark)
  - `/people` (`1440px` desktop, `RU`, Light & Dark)
- **Problem:**
  1. **`/quality` (`390px`, `RU`)**: The 5-tab `SegmentedControl` (`Все (3)`, `Критические (1)`, `Предупреждения (1)`, `Справка (1)`, `Дубликаты (1)`) wraps every Russian tab label onto 2 lines and still overflows horizontally past the right edge of the `390px` screen, cutting off `"Дубликаты (1)"`!
  2. **`/settings` (`1440px`, `RU`)**: The desktop left sidebar (`md:w-60`) uses `truncate` on single-line buttons, cutting off 3 out of 6 Russian section titles with ellipses: `"Безопасность и ключи вх..."`, `"Импорт и экспорт GEDC..."`, and `"Резервные копии и сним..."`.
  3. **`/people` (`1440px`, `RU`)**: In [DirectoryTable.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryTable.tsx), the `"Смерть"` (Died) column (`121px` wide) wraps `† 1958-05-12` onto two lines (`† 1958-05-` / `12`) while the `"Рождение"` column stays on one line.
- **Evidence:**
  - Screenshots: [quality-warnings-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/quality-warnings-390-light-ru.png), [settings-tree-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/settings-tree-1440-light-ru.png), [people-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/people-1440-light-ru.png)
  - Source: [DataQualityPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DataQualityPage.tsx), [SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx), [DirectoryTable.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryTable.tsx)
- **Proposed fix:**
  1. In [DataQualityPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DataQualityPage.tsx) / [segmented-control.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/segmented-control.tsx), allow `SegmentedControl` to wrap (`flex-wrap`) or scroll horizontally (`max-w-full overflow-x-auto whitespace-nowrap`) on `< md` viewports so all 5 tabs (`Все (3)` .. `Дубликаты (1)`) are readable without overflowing the page width.
  2. In [SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx), widen the desktop sidebar from `md:w-60` to `md:w-68 lg:w-72` and allow nav labels to wrap cleanly (`whitespace-normal text-left leading-snug`) instead of `truncate`.
  3. In [DirectoryTable.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryTable.tsx), add `whitespace-nowrap` to both Birth and Death date table cells (`<td>`).
- **Acceptance criteria:**
  - [ ] On `/quality` at `390px` in Russian, all 5 filter options (`Все (3)` through `Дубликаты (1)`) are accessible without page-level horizontal overflow.
  - [ ] On `/settings` at `1440px` in Russian, all 6 sidebar navigation items display their full Russian text without `...` truncation.
  - [ ] On `/people` at `1440px` in Russian, death dates (`† 1958-05-12`) render on a single line.
- **Tags:** layout | space | copy

---

### UX-111 [major] Add missing accessible names (`aria-label` / `aria-labelledby`) to unnamed `combobox` and `radiogroup` controls
- **Screen / viewport / theme / lang:** `/people`, `/media`, `/places`, `/export/print`, `/dev/ui` (`1440px` & `390px`, `EN` & `RU`)
- **Problem (WCAG 2.2 SC 4.1.2 Name, Role, Value):**
  Accessibility tree inspection revealed several interactive widgets without accessible names (or with untranslated accessible names):
  1. **`/people` ([DirectoryFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryFilterBar.tsx))**: The 3 filter dropdowns (`Пол` / `Sex`, `Статус жизни` / `Vital Status`, `Портретное фото` / `Portrait Photo`) render visual label text above `<Select>` without linking `id`/`aria-labelledby` or passing `aria-label`, resulting in 3 unnamed `combobox` controls in the accessibility tree (`UX-026` still open).
  2. **`/media` ([MediaFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaFilterBar.tsx#L161))**: The person filter `<Select>` (`"Все отмеченные персоны"` / `"All Tagged People"`) has no `aria-label`, exposing an unnamed `combobox`.
  3. **`/places` ([PlacesPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PlacesPage.tsx))**: The top-right view switcher `SegmentedControl` (`Иерархия и список` | `Кластерная карта`) has no `ariaLabel`, exposing an unnamed `radiogroup`.
  4. **`/export/print` ([PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx))**: The preview mode `SegmentedControl` (`Макет древа` | `PDF документ`) has no `ariaLabel`, exposing an unnamed `radiogroup` (`UX-099` still open).
  5. **`/dev/ui` ([DevUiWorkbench.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DevUiWorkbench.tsx))**: The mobile section jump `<Select>` has a hardcoded English `aria-label="Section navigation"` even in Russian mode (`UX-024` still open).
- **Evidence:**
  - Source: [DirectoryFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryFilterBar.tsx), [MediaFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaFilterBar.tsx#L161), [PlacesPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PlacesPage.tsx), [PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx), [DevUiWorkbench.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DevUiWorkbench.tsx)
- **Proposed fix:**
  - Pass localized `aria-label` props to the `<Select>` components in `DirectoryFilterBar.tsx` (`t('directory.filters.sex')`, `t('directory.filters.vitalStatus')`, `t('directory.filters.hasPhoto')`), `MediaFilterBar.tsx` (`t('media.filters.personLabel')`), and `DevUiWorkbench.tsx`.
  - Pass localized `ariaLabel` props to `SegmentedControl` in `PlacesPage.tsx` and `PrintPreviewCanvas.tsx`.
- **Acceptance criteria:**
  - [ ] Every `role="combobox"` and `role="radiogroup"` across `/people`, `/media`, `/places`, `/export/print`, and `/dev/ui` has a non-empty, localized accessible name in both EN and RU.
- **Tags:** a11y

---

### UX-112 [major] Enforce `>= 40×40px` touch target sizing on interactive controls across `AuthLayout`, `PasswordInput`, `PublicGuestBanner`, `DirectoryCardList`, `MediaCard`, and `PlacesTreeListCard`
- **Screen / viewport / theme / lang:** `/login`, `/setup`, `/register`, `/`, `/people`, `/media`, `/places` (`1440px` & `390px`, `EN` & `RU`)
- **Problem (Checklist #1 & Design System `>= 40×40px` touch target rule):**
  Bounding-box measurements from the live DOM showed multiple interactive controls smaller than `40×40px` without `touch-target-expand`:
  1. **`/login`, `/setup`, `/register` ([AuthLayout.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/auth/AuthLayout.tsx#L56-L57))**: `LanguageToggleButton size="sm"` (`60×32px`) and `ThemeToggleButton size="sm"` (`32×32px`) are only `32px` tall; inside [input.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/input.tsx) (`PasswordInput`), the `"Show password"` / `"Показать пароль"` eye toggle button is only `24×24px` (`box=877,516,24,24`).
  2. **`PublicGuestBanner` ([PublicGuestBanner.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/auth/PublicGuestBanner.tsx))**: `"Sign In to Family Archive"` / `"Войти в семейный архив"` button (`height: 32px`) and Close `X` `IconButton` (`32×32px`).
  3. **`/people` ([DirectoryCardList.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryCardList.tsx))**: Mobile card footer buttons `"В древе"` (`40×32px`) and `"Профиль"` (`40×32px`) have `32px` height on `390px` touch devices.
  4. **`/media` ([MediaCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaCard.tsx#L144))**: `"Открыть студию"` / `"Open Studio"` button overrides `size="sm"` with `h-7 min-h-0 px-2.5` (`28px` height!).
  5. **`/places` ([PlacesTreeListCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesTreeListCard.tsx))**: `"Добавить место"` (`138×32px`), `"Изменить"` (`99×32px`), and `"Объединить с..."` (`134×32px`) action buttons are `32px` tall.
- **Evidence:**
  - Source: [AuthLayout.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/auth/AuthLayout.tsx#L56-L57), [input.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/input.tsx), [PublicGuestBanner.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/auth/PublicGuestBanner.tsx), [DirectoryCardList.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/directory/DirectoryCardList.tsx), [MediaCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaCard.tsx#L144), [PlacesTreeListCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesTreeListCard.tsx)
- **Proposed fix:**
  - In [AuthLayout.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/auth/AuthLayout.tsx#L56-L57), use `size="md"` (`44px`) on `LanguageToggleButton` and `ThemeToggleButton`.
  - In [input.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/input.tsx), expand `PasswordInput`'s toggle button to `h-9 w-9 touch-target-expand` (`>= 40×40px` hit target).
  - Add `touch-target-expand` (or `min-h-10`) to `Button size="sm"` and `IconButton size="sm"` in [button.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/button.tsx) and [icon-button.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/ui/icon-button.tsx), and remove the `h-7 min-h-0` override in [MediaCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaCard.tsx#L144).
- **Acceptance criteria:**
  - [ ] All interactive buttons and toggles on `/login`, `/`, `/people`, `/media`, and `/places` have an effective tap target of at least `40×40px`.
- **Tags:** a11y | consistency

---

### UX-113 [major] Fix SVG poster title overlap with tile badge (`UX-097`) and dark-mode paper sheet background on `/export/print`
- **Screen / viewport / theme / lang:** `/export/print` (`1440px` & `390px`, Light & Dark, `EN` & `RU`)
- **Problem:**
  1. When multi-page tiling (`2×2` or `3×3`) is active on `/export/print`, the top-center tile badge (`"Лист 1 из 4 [Ряд 1, Колонка 1]"` / `"Page 1 of 4 [Row 1, Col 1]"`) is drawn at the exact same `Y` coordinate as the poster title `"Предки: Александр Петров"` (`"Ancestors of Alexander Petrov"`), colliding directly over the heading text (`UX-097` still open).
  2. When switching the application UI to **Dark Mode** (`print-1440-dark-ru.png`) while the print theme is set to `"Современный"` (Modern Light print style), the SVG paper canvas renders with a dark `#171B22` background instead of paper white (`#FFFFFF`), making the preview look like a black-paper poster and reducing contrast with dark connector lines.
- **Evidence:**
  - Screenshots: [print-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-1440-light-ru.png), [print-1440-dark-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-1440-dark-ru.png), [print-preview-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-preview-390-light-ru.png)
  - Source: [PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx)
- **Proposed fix:**
  1. In [PrintPreviewCanvas.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintPreviewCanvas.tsx), shift the poster title/subtitle group `<g>` down by `28px` whenever multi-page tiling (`totalTiles > 1`) is active so there is clear vertical separation below the tile badge.
  2. Bind the SVG paper sheet fill and node card fills inside `PrintPreviewCanvas.tsx` explicitly to the selected **print theme** (`#FFFFFF` for Modern, `#FDFBF7` for Archival Cream) rather than CSS variables that flip in app Dark Mode.
- **Acceptance criteria:**
  - [ ] The tile badge (`"Page 1 of 4..."` / `"Лист 1 из 4..."`) never overlaps the poster heading or subtitle in any paper size or orientation.
  - [ ] In Dark Mode, the simulated physical paper sheet in `PrintPreviewCanvas` remains true to the chosen print style (`#FFFFFF` for Modern, `#FDFBF7` for Archival).
- **Tags:** layout | consistency | states

---

### UX-114 [minor] Remove duplicate page title and subtitle inside `PlacesTreeListCard` on `/places` and mobile `PrintConfigSidebar` on `/export/print`
- **Screen / viewport / theme / lang:** `/places` (`1440px` & `390px`, `EN` & `RU`) and `/export/print` (`390px`, `EN` & `RU`)
- **Problem:**
  1. On `/places`, [PlacesPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PlacesPage.tsx) renders the page `h1` `"Места и историческое деление"` and subtitle `"Иерархический справочник стран, губерний/областей и городов..."`. Immediately below it, [PlacesTreeListCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesTreeListCard.tsx) renders an `h2` with the **exact same title and 2-line subtitle** a second time!
  2. On `/export/print` at `390px` mobile, `"Параметры печати"` (`"Print Settings"`) appears in the mobile view-switcher `SegmentedControl` and immediately below as the `h2` inside [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx) (`UX-100`).
- **Evidence:**
  - Screenshots: [places-list-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/places-list-1440-light-ru.png), [places-list-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/places-list-390-light-ru.png), [print-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/print-390-light-ru.png)
  - Source: [PlacesPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PlacesPage.tsx), [PlacesTreeListCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesTreeListCard.tsx), [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx)
- **Proposed fix:**
  1. In [PlacesTreeListCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesTreeListCard.tsx), replace the duplicated page title/subtitle with a concise card header (e.g., `"Иерархия мест"` / `"Place Hierarchy"` without repeating the 2-line page description).
  2. In [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx), add `hidden lg:block` to the sidebar header block on mobile.
- **Acceptance criteria:**
  - [ ] `/places` does not repeat the page title and subtitle twice in the top left area.
  - [ ] `/export/print` on `390px` mobile does not repeat `"Параметры печати"` twice at the top of the configuration view.
- **Tags:** hierarchy | space | copy

---

### UX-115 [minor] Fix Russian localization gaps, raw database identifiers, and duplicated labels across `/login`, `/media`, `/history`, `/quality`, `/people/merge`, `/export/print`, and `/settings`
- **Screen / viewport / theme / lang:** `/login`, `/setup`, `/register`, `/media`, `/history`, `/quality`, `/people/merge`, `/export/print`, `/settings` (`1440px` & `390px`, `RU`)
- **Problem:**
  1. **`/login`, `/setup`, `/register` ([AuthPages.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/AuthPages.tsx))**: `AuthPages.tsx` defaults `instanceName = 'Levshits Family Archive'` and passes it to `AuthLayout`, overriding `AuthLayout`'s localized `t('app.instanceName')` (`"Семейный архив Левшиц"`). As a result, all auth screens show `"Levshits Family Archive"` in English even when switched to Russian.
  2. **`/media` ([MediaFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaFilterBar.tsx#L49-L57))**: Decade pills render English `"s"` suffixes (`"1880s"`, `"1900s"`, `"1920s"`, `"1940s"`, `"1960s"`, `"1980s"`, `"2000s"`) in Russian mode instead of `"1880-е"`, `"1900-е"`, etc. Also, the `"Загрузить медиа"` button in the filter row wraps onto 2 lines while the search input placeholder is truncated (`"Поиск фото и документов по н"`).
  3. **`/history` ([HistoryPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/HistoryPage.tsx))**: Diff tables display raw snake_case database column names (`given_name`, `patronymic`, `surname`, `birth_date`, `birth_place_text`, `title`, `crop_box`, `status: active -> deleted`) and raw entity type `(person)` instead of localized human-readable field names (`Имя`, `Отчество`, `Фамилия`, `Дата рождения`, `Место рождения`, `(персона)`).
  4. **`/quality` ([DataQualityPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DataQualityPage.tsx))**: Issue cards render raw uppercase backend enum codes (`DEATH_BEFORE_BIRTH`, `PARENT_TOO_YOUNG`, `MISSING_BIRTH`) next to the severity badge.
  5. **`/people/merge` ([MergePeoplePage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/MergePeoplePage.tsx))**: The Sex comparison row renders raw `"M"` instead of localized `"Мужской"` / `"Male"`; the bottom summary badges repeat the number twice (`"События: 6"` followed by `"6"`, `"Родственные связи: 5"` followed by `"5"`); and on `390px` mobile (`merge-390-light-ru.png`), the primary person's name (`"Pyotr Fyodorovich Shch..."`) is truncated with an ellipsis.
  6. **`/settings` & `/export/print` ([SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx), [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx))**: In `/settings`, the System Diagnostics `<dl>` repeats the term inside the definition (`term: "Сервер"` -> `definition: "Сервер: В норме"`, `term: "База данных"` -> `definition: "База данных: Подключена (12 MB)"`), uses English `"MB"` / `"GB"` units in `/settings` and `/settings/backups`, and `/export/print` uses English `"10 mm"` / `"15 mm"` / `"20 mm"` (`UX-101`).
- **Evidence:**
  - Screenshots: [login-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/login-1440-light-ru.png), [media-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/media-1440-light-ru.png), [history-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/history-1440-light-ru.png), [quality-warnings-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/quality-warnings-1440-light-ru.png), [merge-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/merge-1440-light-ru.png), [merge-390-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/merge-390-light-ru.png), [settings-tree-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/settings-tree-1440-light-ru.png)
- **Proposed fix:**
  1. In [AuthPages.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/AuthPages.tsx), omit the default `'Levshits Family Archive'` fallback so `AuthLayout` uses `t('app.instanceName')`.
  2. In [MediaFilterBar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/media/MediaFilterBar.tsx), format decade labels via i18n (`1880-е` in RU, `1880s` in EN) and add `shrink-0 whitespace-nowrap` to the `"Загрузить медиа"` button.
  3. Map audit diff field keys in [HistoryPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/HistoryPage.tsx), sex codes (`M`/`F`/`U`) and duplicate count badges in [MergePeoplePage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/MergePeoplePage.tsx), and units (`мм`, `МБ`, `ГБ`) / diagnostics values in [SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx) and [PrintConfigSidebar.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/print/PrintConfigSidebar.tsx) to localized i18n strings.
- **Acceptance criteria:**
  - [ ] Switching to `RU` on `/login`, `/setup`, `/register`, `/media`, `/history`, `/quality`, `/people/merge`, `/export/print`, and `/settings` shows zero raw English strings, snake_case field keys, or duplicated label prefixes.
- **Tags:** copy | consistency

---

### UX-116 [minor] Unify offline/demo fallback states across `DashboardPage`, `PersonProfilePage`, and `SettingsPage` (`/`, `/persons/:id`, `/settings/users`, `/settings/security`)
- **Screen / viewport / theme / lang:** `/`, `/persons/:id`, `/settings/users`, `/settings/security` (`1440px` & `390px`, `EN` & `RU`)
- **Problem:**
  While `/people`, `/people/surnames`, `/media`, `/places`, `/history`, `/quality`, `/people/merge`, `/export/print`, and `/settings/backups` gracefully fall back to rich demo datasets when the backend API is offline or running without a database:
  1. **`/persons/:id` ([PersonProfilePage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PersonProfilePage.tsx#L63))**: Even though `buildDefaultPersonProfile()` is already defined in `PersonProfilePage.tsx`, the `catch` block sets an error state and renders a destructive red `"Карточка персоны не найдена"` / `"Person Profile Not Found"` alert instead of falling back to the demo profile (`UX-025` still open).
  2. **`/` ([DashboardPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DashboardPage.tsx#L37))**: Falls back to `buildCleanDashboardPayload()` (all zeros and empty states) even though `buildDefaultDashboardPayload()` exists on line 183 and `/people` + `/media` + `/history` show populated archive data in the same session.
  3. **`/settings/users` & `/settings/security` ([SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx))**: When `users`, `invitations`, or `sessions` arrays are empty, the cards render empty `<div className="divide-y">` containers with zero rows and no `EmptyState` message.
- **Evidence:**
  - Screenshots: [person-profile-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/person-profile-1440-light-ru.png), [dashboard-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/dashboard-1440-light-ru.png), [settings-users-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/settings-users-1440-light-ru.png), [settings-security-1440-light-ru.png](file:///Users/levshitsvv/.gemini/antigravity/brain/1b133d0e-75b9-4ff7-80a1-6d38946dba46/screenshots/settings-security-1440-light-ru.png)
  - Source: [PersonProfilePage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PersonProfilePage.tsx#L63), [DashboardPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DashboardPage.tsx#L37-L183), [SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx)
- **Proposed fix:**
  - In [PersonProfilePage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/PersonProfilePage.tsx), fall back to `buildDefaultPersonProfile(id)` when the API request fails (matching `/people` and `/media`).
  - In [SettingsPage.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/SettingsPage.tsx), render a helpful `EmptyState` (or fallback demo rows) when `users`, `invitations`, or `sessions` lists are empty instead of blank containers.
- **Acceptance criteria:**
  - [ ] Navigating from `/people` to `/persons/00000000-0000-0000-0000-000000000001` renders the person profile tabs (Timeline, Family, Media, Sources, Notes, History) instead of a dead-end error box when in demo/fallback mode.
  - [ ] `/settings/users` and `/settings/security` never render blank 0px list containers.
- **Tags:** states | consistency

---

### UX-117 [minor] Replace remaining raw `<button>` and `<input>` elements in `GenDateInput`, `MarkdownEditor`, and `DevUiWorkbench`, and soften dark-mode map tiles on `/places?view=map`
- **Screen / viewport / theme / lang:** `/dev/ui` (`1440px` & `390px`, `EN` & `RU`) and `/places?view=map` (`1440px` & `390px`, Dark mode)
- **Problem:**
  1. **Rule 2 (No raw `<button>`/`<input>`)**: Code audit found raw `<button>` and `<input>` elements remaining in:
     - [gen-date-input.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/form/gen-date-input.tsx#L212-L387) (lines 212, 235, 249, 258, 268, 279, 288, 302, 326, 387)
     - [markdown-editor.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/form/markdown-editor.tsx#L45-L59) (lines 45, 59)
     - [DevUiWorkbench.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DevUiWorkbench.tsx#L684) (line 684)
  2. **`/dev/ui` (`390px` mobile)**: The `ComparisonTable` specimen (`width: 486px`) overflows its `358px` parent card horizontally without an `overflow-x-auto` wrapper.
  3. **`/places?view=map` (Dark mode)**: In [places-map-1440-dark-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/places-map-1440-dark-ru.png), the MapLibre/OSM raster map tiles remain bright white in Dark Mode, creating harsh glare against the `#0E1116` dark canvas.
- **Evidence:**
  - Screenshots: [places-map-1440-dark-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/places-map-1440-dark-ru.png), [dev-ui-390-light-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/dev-ui-390-light-ru.png)
  - Source: [gen-date-input.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/form/gen-date-input.tsx#L212-L387), [markdown-editor.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/form/markdown-editor.tsx#L45-L59), [DevUiWorkbench.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/pages/DevUiWorkbench.tsx#L684), [PlacesMapViewCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesMapViewCard.tsx)
- **Proposed fix:**
  - Replace raw `<button>` / `<input>` elements in `gen-date-input.tsx`, `markdown-editor.tsx`, and `DevUiWorkbench.tsx` with design-system `Button`, `IconButton`, and `TextInput`.
  - Wrap `ComparisonTable` in `overflow-x-auto`.
  - Apply a subtle dark-mode CSS filter (`dark:[&_.maplibregl-canvas]:brightness-90 dark:[&_.maplibregl-canvas]:contrast-95`) or dark tile style to the map canvas in [PlacesMapViewCard.tsx](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/components/places/PlacesMapViewCard.tsx).
- **Acceptance criteria:**
  - [ ] Zero raw `<button>` or `<input>` elements remain in `frontend/src/pages/` or `frontend/src/components/form/`.
  - [ ] `ComparisonTable` on `/dev/ui` scrolls cleanly within its card on `390px` viewports.
- **Tags:** consistency | layout

---

### UX-118 [major] Harmonize EN and RU button, tab, and segmented control label lengths so controls look visually identical across languages
- **Screen / viewport / theme / lang:** All routes (`1440px` & `390px`, `EN` vs `RU`, Light & Dark)
- **Problem:**
  Side-by-side comparison of the EN and RU screenshots across all 24 views (and an automated length-delta audit of [en.json](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/i18n/en.json) vs [ru.json](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/i18n/ru.json)) shows that many buttons, segmented controls, sidebar navigation items, and table headers use verbose multi-word Russian translations (or overly long English phrases) that are **40%–120% longer** than their counterpart. This causes buttons and segmented pills to change width dramatically, wrap onto 2 lines in one language while staying 1 line in the other, or truncate with `...`:
  1. **Settings Navigation Sidebar (`/settings`, `settings.tabs.*`)**:
     - `settings.tabs.security`: EN `"Security & Passkeys"` (19 chars, fits 1 line) vs RU `"Безопасность и ключи входа"` (26 chars -> truncates to `"Безопасность и ключи вх..."`).
     - `settings.tabs.gedcom`: EN `"GEDCOM Import/Export"` (20 chars) vs RU `"Импорт и экспорт GEDCOM"` (23 chars -> truncates to `"Импорт и экспорт GEDC..."`).
     - `settings.tabs.backups`: EN `"Backups & Snapshots"` (19 chars) vs RU `"Резервные копии и снимки"` (24 chars -> truncates to `"Резервные копии и сним..."`).
     - `settings.tabs.users`: EN `"Family Members & RBAC"` (21 chars) vs RU `"Участники семьи и роли"` (22 chars).
  2. **Data Quality Segmented Control & Action Buttons (`/quality`, `quality.tabs.*`)**:
     - `quality.tabs.all`: EN `"All Warnings ({count})"` (22 chars, wraps to 2 lines) vs RU `"Все ({count})"` (13 chars, 9 chars shorter!).
     - `quality.tabs.warning`: EN `"Warnings ({count})"` (18 chars) vs RU `"Предупреждения ({count})"` (24 chars).
     - `quality.tabs.critical`: EN `"Critical ({count})"` (18 chars) vs RU `"Критические ({count})"` (21 chars).
     - `quality.duplicates.viewProfileBtn`: EN `"View Profile"` (12 chars) vs RU `"Профиль"` (7 chars).
     - `quality.duplicates.compareMergeBtn`: EN `"Compare & Merge"` (15 chars) vs RU `"Сравнить и объединить"` (21 chars).
     - `quality.dismissBtn`: EN `"This is correct"` (15 chars) vs RU `"Это верно"` (9 chars).
  3. **Print Studio Segmented Controls, Toggles & Toolbar Buttons (`/export/print`, `print.*`)**:
     - `print.downloadPdf`: EN `"Download Printable PDF"` (22 chars, 1 line) vs RU `"Скачать PDF для печати"` (22 chars, wraps/clips in toolbar).
     - `print.tabPreview`: EN `"Preview & Summary"` (17 chars) vs RU `"Предпросмотр"` (12 chars).
     - `print.previewModeTree` / `print.previewModePdf`: EN `"Tree Layout"` (11) / `"PDF Document"` (12) vs RU `"Макет древа"` (11) / `"PDF документ"` (12) — good parity, whereas zoom buttons:
       - `print.zoomFit`: EN `"Fit to view"` (11 chars) vs RU `"По размеру окна"` (15 chars -> `"По размеру"` 10 chars matches EN!).
       - `print.zoomReset`: EN `"Reset zoom"` (10 chars) vs RU `"Сбросить масштаб"` (16 chars -> `"Сбросить"` 8 chars).
     - `print.densityWide`: EN `"Wide (Collateral)"` (17 chars) vs RU `"Широкая (С боковыми линиями)"` (28 chars, +11 chars!).
     - `print.densityCompact`: EN `"Compact (Lineal)"` (16 chars) vs RU `"Компактная (Прямая линия)"` (25 chars, +9 chars!).
     - `print.formatSingle` / `print.formatTiled`: EN `"Single Sheet"` (12) / `"Multi-Page Tiled"` (16) vs RU `"Один лист"` (9) / `"Составной (На листах)"` (21).
     - `print.themeModern` / `print.themeHeritage`: EN `"Modern Clean"` (12) / `"Heritage Antique"` (16) vs RU `"Современный"` (11) / `"Классический архивный"` (21).
  4. **Media Gallery Filter Bar & Lightbox Studio (`/media`, `mediaArchive.*`)**:
     - Upload button in `MediaFilterBar`: EN `"Upload Media"` (12 chars, 1 line) vs RU `"Загрузить медиа"` (15 chars, wraps onto 2 lines `"Загрузить / медиа"` in `media-1440-light-ru.png`).
     - `mediaArchive.typeAll` / `typePhotos` / `typeDocuments`: EN `"All Media"` (9) / `"Photos"` (6) / `"Documents"` (9) vs RU `"Все материалы"` (13) / `"Фотографии"` (10) / `"Документы"` (9).
     - `mediaArchive.filterPersonAll`: EN `"All Tagged People"` (17) vs RU `"Все отмеченные персоны"` (22).
     - `mediaArchive.setAsPrimaryAvatarBtn` / `canvas.setPrimaryAvatarBtn` / `metadata.setAvatarBtn`: EN `"Use as Avatar"` (13) vs RU `"Сделать портретом"` (17).
     - `mediaArchive.doneTaggingBtn`: EN `"Done Tagging"` (12) vs RU `"Завершить разметку"` (18).
     - `mediaArchive.saveMetadataBtn`: EN `"Save Details"` (12) vs RU `"Сохранить сведения"` (18).
  5. **Interactive Tree Header Toolbar & Modes (`/tree`, `tree.*`)**:
     - `tree.modes.pedigree` / `hourglass` / `descendants` / `fan`: EN `"Pedigree"` (8) / `"Hourglass"` (9) / `"Descendants"` (11) / `"Fan Chart"` (9) vs RU `"Предки"` (6) / `"Песочные часы"` (13) / `"Потомки"` (7) / `"Веерная диаграмма"` (17, nearly 2× EN length!).
     - `tree.drawer.makeFocalBtn`: EN `"Make Focal Person"` (17) vs RU `"Сделать центральной персоной"` (28, +11 chars!).
  6. **People Directory & Merge Page (`/people`, `/people/merge`, `directory.*`, `merge.*`)**:
     - `directory.colName`: EN `"Name"` (4) vs RU `"Имя и фамилия"` (13).
     - `directory.colBorn` / `colDied`: EN `"Born"` (4) / `"Died"` (4) vs RU `"Рождение"` (8) / `"Смерть"` (6).
     - `directory.addPersonBtn`: EN `"Add Person"` (10) vs RU `"Добавить персону"` (16).
     - `directory.filterLivingOnly`: EN `"Living only"` (11) vs RU `"Только ныне живущие"` (19).
     - `merge.colTarget` / `merge.colSource`: EN `"Keep Primary Record"` (19) / `"Merge Candidate Record"` (22) vs RU `"Основная запись (оставить)"` (26) / `"Вторая запись (влить)"` (21).
     - `merge.confirmMergeBtn`: EN `"Confirm & Merge Records"` (23) vs RU `"Подтвердить и объединить записи"` (31).
  7. **Places Page (`/places`, `places.*`)**:
     - Top view switcher (`PlacesPage.tsx`): EN `"Hierarchy & List"` (16) / `"Cluster Map"` (11) vs RU `"Иерархия и список"` (17) / `"Кластерная карта"` (16).
     - `places.viewTree` / `places.viewTable`: EN `"Hierarchy Tree"` (14) / `"Flat Table"` (10) vs RU `"Иерархическое дерево"` (20) / `"Плоская таблица"` (15).
     - `places.addPlaceBtn`: EN `"Add Place"` (9) vs RU `"Добавить место"` (14).
     - `places.coordsPresent` / `places.coordsMissing`: EN `"Set"` (3) / `"Missing"` (7) vs RU `"Указаны"` (7) / `"Нет"` (3).
  8. **Auth & Common Buttons (`common.*`, `auth.*`, `settings.*`)**:
     - `common.edit`: EN `"Edit"` (4) vs RU `"Редактировать"` (13, >3× longer! Using `"Изменить"` 8 chars is standard and compact).
     - `settings.general.saveButton`: EN `"Save Changes"` (12) vs RU `"Сохранить изменения"` (19 -> `"Сохранить"` 9 chars or EN `"Save"` / RU `"Сохранить"`).
     - `auth.login.magicLinkBtn`: EN `"Send Magic Link"` (15) vs RU `"Отправить ссылку для входа"` (26 -> `"Войти по ссылке"` 15 chars!).
     - `auth.setup.createAdminPasskeyBtn`: EN `"Create Admin Passkey"` (20) vs RU `"Создать ключ администратора (Passkey)"` (37 -> `"Создать ключ входа"` 18 chars!).
     - `settings.security.regenerateCodesBtn`: EN `"Regenerate 10 Recovery Codes"` (28) vs RU `"Перевыпустить 10 кодов восстановления"` (37 -> `"Обновить резервные коды"` 23 chars / EN `"Refresh Recovery Codes"` 22 chars).
     - `settings.security.signOutEverywhereBtn`: EN `"Sign out everywhere else"` (24) vs RU `"Выйти на всех остальных устройствах"` (35 -> `"Выйти на других устройствах"` 27 chars).
     - `gendate.julianToggle` / `gregorianToggle`: EN `"Julian (O.S.)"` (13) / `"Gregorian (N.S.)"` (16) vs RU `"Старый стиль (ст. ст.)"` (22) / `"Новый стиль (нов. ст.)"` (22) -> RU `"Юлианский (ст. ст.)"` (19) or `"Старый стиль"` (12) / `"Новый стиль"` (11).
- **Evidence:**
  - Compare EN vs RU screenshots in [docs/ux-reviews/screenshots-2026-11-04/](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/):
    - [settings-tree-1440-light-en.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/settings-tree-1440-light-en.png) vs [settings-tree-1440-light-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/settings-tree-1440-light-ru.png)
    - [quality-warnings-390-light-en.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/quality-warnings-390-light-en.png) vs [quality-warnings-390-light-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/quality-warnings-390-light-ru.png)
    - [media-1440-light-en.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/media-1440-light-en.png) vs [media-1440-light-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/media-1440-light-ru.png)
    - [print-1440-light-en.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/print-1440-light-en.png) vs [print-1440-light-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/print-1440-light-ru.png)
    - [login-1440-light-en.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/login-1440-light-en.png) vs [login-1440-light-ru.png](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/screenshots-2026-11-04/login-1440-light-ru.png)
- **Proposed fix:**
  Update button, tab, and segmented-control labels in [en.json](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/i18n/en.json) and [ru.json](file:///Users/levshitsvv/Projects/Personal/genealogy/frontend/src/i18n/ru.json) so each pair has nearly identical character length (`±3–4 chars` max) and identical line count across viewports:

  | i18n Key | Current EN (len) | Current RU (len) | Proposed EN (len) | Proposed RU (len) |
  | :--- | :--- | :--- | :--- | :--- |
  | `common.edit` | `"Edit"` (4) | `"Редактировать"` (13) | `"Edit"` (4) | `"Изменить"` (8) |
  | `settings.tabs.general` | `"General & Archive"` (17) | `"Общие и архив"` (13) | `"Archive Settings"` (16) | `"Настройки архива"` (16) |
  | `settings.tabs.users` | `"Family Members & RBAC"` (21) | `"Участники семьи и роли"` (22) | `"Members & Roles"` (15) | `"Участники и роли"` (16) |
  | `settings.tabs.security` | `"Security & Passkeys"` (19) | `"Безопасность и ключи входа"` (26) | `"Security & Keys"` (15) | `"Защита и ключи"` (14) |
  | `settings.tabs.gedcom` | `"GEDCOM Import/Export"` (20) | `"Импорт и экспорт GEDCOM"` (23) | `"GEDCOM Exchange"` (15) | `"Обмен GEDCOM"` (12) |
  | `settings.tabs.backups` | `"Backups & Snapshots"` (19) | `"Резервные копии и снимки"` (24) | `"Backups & Copies"` (16) | `"Резервные копии"` (15) |
  | `quality.tabs.all` | `"All Warnings ({count})"` (22) | `"Все ({count})"` (13) | `"All ({count})"` (13) | `"Все ({count})"` (13) |
  | `quality.tabs.critical` | `"Critical ({count})"` (18) | `"Критические ({count})"` (21) | `"Critical ({count})"` (18) | `"Критично ({count})"` (18) |
  | `quality.tabs.warning` | `"Warnings ({count})"` (18) | `"Предупреждения ({count})"` (24) | `"Warnings ({count})"` (18) | `"Внимание ({count})"` (18) |
  | `quality.tabs.info` | `"Info ({count})"` (14) | `"Справка ({count})"` (17) | `"Notes ({count})"` (15) | `"Заметки ({count})"` (17) |
  | `quality.duplicates.viewProfileBtn` | `"View Profile"` (12) | `"Профиль"` (7) | `"Profile"` (7) | `"Профиль"` (7) |
  | `quality.duplicates.compareMergeBtn` | `"Compare & Merge"` (15) | `"Сравнить и объединить"` (21) | `"Compare & Merge"` (15) | `"Сравнить записи"` (15) |
  | `quality.dismissBtn` | `"This is correct"` (15) | `"Это верно"` (9) | `"Looks Correct"` (13) | `"Всё правильно"` (13) |
  | `print.downloadPdf` | `"Download Printable PDF"` (22) | `"Скачать PDF для печати"` (22) | `"Download PDF Poster"` (19) | `"Скачать PDF-постер"` (18) |
  | `print.tabPreview` | `"Preview & Summary"` (17) | `"Предпросмотр"` (12) | `"Layout Preview"` (14) | `"Макет и превью"` (14) |
  | `print.zoomFit` | `"Fit to view"` (11) | `"По размеру окна"` (15) | `"Fit view"` (8) | `"Вписать"` (7) |
  | `print.zoomReset` | `"Reset zoom"` (10) | `"Сбросить масштаб"` (16) | `"Reset"` (5) | `"Сброс"` (5) |
  | `print.densityWide` | `"Wide (Collateral)"` (17) | `"Широкая (С боковыми линиями)"` (28) | `"Full (All Branches)"` (19) | `"Полная (Все ветви)"` (18) |
  | `print.densityCompact` | `"Compact (Lineal)"` (16) | `"Компактная (Прямая линия)"` (25) | `"Direct Line Only"` (16) | `"Прямая линия"` (12) |
  | `print.formatSingle` | `"Single Sheet"` (12) | `"Один лист"` (9) | `"Single Sheet"` (12) | `"Один лист"` (9) |
  | `print.formatTiled` | `"Multi-Page Tiled"` (16) | `"Составной (На листах)"` (21) | `"Multi-Page Grid"` (15) | `"Сетка листов"` (12) |
  | `print.themeModern` | `"Modern Clean"` (12) | `"Современный"` (11) | `"Modern Light"` (12) | `"Современный"` (11) |
  | `print.themeHeritage` | `"Heritage Antique"` (16) | `"Классический архивный"` (21) | `"Archival Classic"` (16) | `"Архивный стиль"` (14) |
  | `tree.modes.pedigree` | `"Pedigree"` (8) | `"Предки"` (6) | `"Ancestors"` (9) | `"Предки"` (6) |
  | `tree.modes.hourglass` | `"Hourglass"` (9) | `"Песочные часы"` (13) | `"Hourglass"` (9) | `"Две ветви"` (9) |
  | `tree.modes.descendants` | `"Descendants"` (11) | `"Потомки"` (7) | `"Descendants"` (11) | `"Потомки"` (7) |
  | `tree.modes.fan` | `"Fan Chart"` (9) | `"Веерная диаграмма"` (17) | `"Fan Chart"` (9) | `"Веерная"` (7) |
  | `tree.drawer.makeFocalBtn` | `"Make Focal Person"` (17) | `"Сделать центральной персоной"` (28) | `"Center in Tree"` (14) | `"В центр древа"` (13) |
  | `mediaArchive.typeAll` | `"All Media"` (9) | `"Все материалы"` (13) | `"All Media"` (9) | `"Все медиа"` (9) |
  | `mediaArchive.typePhotos` | `"Photos"` (6) | `"Фотографии"` (10) | `"Photos"` (6) | `"Фото"` (4) |
  | `mediaArchive.filterPersonAll` | `"All Tagged People"` (17) | `"Все отмеченные персоны"` (22) | `"All Tagged People"` (17) | `"Все люди на фото"` (16) |
  | `mediaArchive.doneTaggingBtn` | `"Done Tagging"` (12) | `"Завершить разметку"` (18) | `"Done Tagging"` (12) | `"Готово"` (6) / `"Завершить"` (9) |
  | `mediaArchive.setAsPrimaryAvatarBtn` | `"Use as Avatar"` (13) | `"Сделать портретом"` (17) | `"Set Portrait"` (12) | `"На аватар"` (9) / `"В портрет"` (9) |
  | `places.viewTree` | `"Hierarchy Tree"` (14) | `"Иерархическое дерево"` (20) | `"Hierarchy"` (9) | `"Иерархия"` (8) |
  | `places.viewTable` | `"Flat Table"` (10) | `"Плоская таблица"` (15) | `"Table View"` (10) | `"Таблица"` (7) |
  | `places.coordsPresent` | `"Set"` (3) | `"Указаны"` (7) | `"Mapped"` (6) | `"Точка"` (5) / `"Есть"` (4) |
  | `places.coordsMissing` | `"Missing"` (7) | `"Нет"` (3) | `"None"` (4) | `"Нет"` (3) |
  | `directory.filterLivingOnly` | `"Living only"` (11) | `"Только ныне живущие"` (19) | `"Living only"` (11) | `"Ныне живущие"` (12) |
  | `merge.colTarget` | `"Keep Primary Record"` (19) | `"Основная запись (оставить)"` (26) | `"Primary Record"` (14) | `"Основная запись"` (15) |
  | `merge.colSource` | `"Merge Candidate Record"` (22) | `"Вторая запись (влить)"` (21) | `"Second Record"` (13) | `"Вторая запись"` (13) |
  | `merge.confirmMergeBtn` | `"Confirm & Merge Records"` (23) | `"Подтвердить и объединить записи"` (31) | `"Confirm & Merge"` (15) | `"Объединить записи"` (17) |
  | `auth.login.magicLinkBtn` | `"Send Magic Link"` (15) | `"Отправить ссылку для входа"` (26) | `"Send Sign-In Link"` (17) | `"Войти по ссылке"` (15) |
  | `auth.setup.createAdminPasskeyBtn` | `"Create Admin Passkey"` (20) | `"Создать ключ администратора (Passkey)"` (37) | `"Create Admin Passkey"` (20) | `"Создать ключ входа"` (18) |
  | `gendate.julianToggle` | `"Julian (O.S.)"` (13) | `"Старый стиль (ст. ст.)"` (22) | `"Old Style (O.S.)"` (16) | `"Старый стиль"` (12) |
  | `gendate.gregorianToggle` | `"Gregorian (N.S.)"` (16) | `"Новый стиль (нов. ст.)"` (22) | `"New Style (N.S.)"` (16) | `"Новый стиль"` (11) |

- **Acceptance criteria:**
  - [ ] Every button, tab, and segmented control across all routes renders with visually matching width and identical line count (single-line) in both English (`EN`) and Russian (`RU`) on `1440px` and `390px` viewports.
  - [ ] No navigation tab, segmented control pill, or action button wraps onto 2 lines in one language while remaining 1 line in the other.
- **Tags:** copy | layout | consistency

---

## Positive Notes
- **Zero Emojis / Icons in i18n Strings (Rule 1)**: All translation keys in `en.json` and `ru.json` strictly use React Lucide icon components (`leftIcon`/`rightIcon`) without any baked-in symbol characters (`+`, `←`, `→`) or decorative emojis.
- **Strong Responsive Architecture on Core Flows**:
  - `/people` seamlessly transitions from the 8-column `DirectoryTable` on `1440px` desktop to a scannable `DirectoryCardList` on `390px` mobile with zero horizontal scroll (`scrollWidth === 390`).
  - `/people/surnames` adapts the Latin (`A–Z`) and Cyrillic (`А–Я`) alphabet bar (`SurnameAlphaBar`) and card grid cleanly across `1440px` and `390px` viewports in both EN and RU.
  - `/tree` provides a full-bleed interactive canvas (`calc(100vh - 3.5rem)`) with floating zoom controls (`44×44px`) properly offset above the mobile `BottomNav` (`bottom-20`).
- **Warm Archival Aesthetic**: The warm paper palette (`#FAF8F5` canvas, `#FFFFFF` elevated cards) paired with literary serif headings (`Source Serif 4`) and crisp `Inter` + `JetBrains Mono` metadata feels inviting and respectful of family history.
