# UX Review — Phase 2 ("Delight") — 2026-10-28

## Verdict: OK
Screens reviewed:
- Dashboard: `/` (`DashboardPage.tsx`, `AnniversariesWidget.tsx`, `TreeGlanceWidget.tsx`, `RecentActivityWidget.tsx`, `LatestPhotosWidget.tsx`)
- Fan Chart & Kinship Finder: `/tree` (`FanChartCanvas.tsx`, `KinshipFinderDialog.tsx`, `TreeToolbar.tsx`)
- Data Quality Warnings: `/quality` (`DataQualityPage.tsx`, `SeverityFilter`, `CandidateDuplicates`)
- Person Merge Wizard: `/people/merge` (`PersonMergePage.tsx`)
- Clustered Places Map: `/places?view=map` (`PlacesMapView.tsx`, `PlacesPage.tsx`)
- Duplicate Guard Hint: `QuickAddDuplicateHint.tsx` (in `QuickAddRelativeDialog.tsx`)

Recordings / Inspections:
- `ux_dashboard_widgets_audit`
- `ux_fanchart_keyboard_inspection`
- `ux_kinship_path_sr_inspection`
- `ux_person_merge_mobile_audit`
- `ux_places_map_clustering_audit`
- `ux_quick_add_duplicate_guard_inspection`

---

## Status of Previous Tasks
*From Phase 1.11, 1.12, and 1.13 reviews:*
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

---

## Status of Phase 2 Tasks
| ID | Title | Status |
| :--- | :--- | :--- |
| **UX-015** | Keyboard inaccessible stat cards in `TreeGlanceWidget` | **fixed** |
| **UX-016** | Missing visible focus ring on interactive SVG slices in `FanChartCanvas` | **fixed** |
| **UX-017** | Keyboard inaccessible map cluster pins in `PlacesMapView` | **fixed** |
| **UX-018** | Missing column/source identifiers on mobile stacked layout in `PersonMergePage` | **fixed** |
| **UX-019** | Suffix dagger notation `(1989†)` in `AnniversariesWidget` | **fixed** |
| **UX-020** | Screen reader list navigation distortion from `aria-hidden` intermediate `<li>` arrows in `KinshipFinderDialog` | **fixed** |
| **UX-021** | Keyboard zoom shortcuts or visible hint in `PlacesMapView` and `FanChartCanvas` | suggestion |

---

## Re-Review Findings & Verification Detail

### 1. Dashboard Widget Accessibility & Typography (`UX-015`, `UX-019`)
- **Keyboard Discoverability & Activation:** In `TreeGlanceWidget.tsx:89-100, 131-143`, "Persons" and "Places" statistics cards now declare `role="button"`, `tabIndex={0}`, descriptive localized `aria-label`s (`${t('...Label')}: ${count}`), `onKeyDown` handlers for `Enter` and `Space`, and `focus-visible:ring-2 focus-visible:ring-focus`. Keyboard navigation discovers both cards, and pressing `Enter`/`Space` smoothly triggers tree/places navigation.
- **Genealogical Dagger Notation:** In `AnniversariesWidget.tsx:138`, deceased anniversary items render with prefix dagger notation `(†{item.death_year})`, eliminating footnote-style suffix placement and achieving complete typographic harmony with `LifeSpan`, `PersonNodeCard`, and `formatLifeSpanText`.

### 2. Fan Chart SVG Focus Feedback (`UX-016`)
- In `FanChartCanvas.tsx:443, 456`, interactive ancestor slice `<g>` groups now include `className="group cursor-pointer focus:outline-none focus-visible:outline-none"` while the inner curved `<path>` defines `group-focus-visible:stroke-brand group-focus-visible:stroke-[3] group-focus-visible:outline-none`.
- Tabbing through ancestor generations (in both 180° and 360° fan projections) produces an unmistakable, high-contrast curved stroke along the focused slice perimeter in both light and dark themes (conforming to WCAG 2.4.7 Focus Visible).

### 3. Clustered Places Map Keyboard Navigation (`UX-017`)
- In `PlacesMapView.tsx:350-379`, clustered markers now have `role="button"`, `tabIndex={0}`, an accessible composite label (`${labelText}, ${placesCount}, ${eventsCount}`), and `Enter`/`Space` keyboard handlers.
- When focused, the outer `<circle>` renders `group-focus-visible:stroke-brand group-focus-visible:stroke-[4]`, providing clear visual feedback. Keyboard users can Tab through geographic clusters and press `Enter` to reveal the breakdown inspector list below.

### 4. Person Merge Mobile Layout & Screen Reader Context (`UX-018`)
- In `PersonMergePage.tsx:496, 522-526, 535, 561-565`, radio option containers now provide:
  - Mobile-only uppercase tracking pills (`sm:hidden text-[10px] font-semibold uppercase tracking-wider text-tertiary mb-1 block`) labeling `merge.colTargetShort` ("Keep Primary" / "Основная") and `merge.colSourceShort` ("Merge Candidate" / "Влить").
  - Rich accessible names via `aria-label={`${t('merge.colTargetShort')}: ${row.targetDisplay}`}`.
- On viewports `< 640px` (e.g. 390px mobile), users can instantly distinguish surviving record values from source candidate values, eliminating ambiguity and the danger of unintended data loss.

### 5. Kinship Path Semantics & Screen Reader Alignment (`UX-020`)
- In `KinshipFinderDialog.tsx:634-663`, intermediate `<li aria-hidden="true">` elements were eliminated.
- The path is now a clean semantic `<ol>` where only person step items are rendered as `<li>` elements.
- The connecting relationship descriptor is included semantically via `<span className="sr-only">, {step.relation_to_next}</span>`, while the visual connector arrows are styled cleanly alongside without polluting list item counts.

---

## Positive Notes
- **Flawless Design-System Conformance:** Zero raw HTML form elements (`<button>`, `<input>`, `<select>`) across all Phase 2 views.
- **Full i18n & Localization Rigor:** 100% key parity across `en.json` and `ru.json` with zero forbidden tech buzzwords or icon glyphs in localization keys.
- **Robust Multi-Modal Interaction:** Tree fan charts, clustered maps, and side-by-side merge cards now provide parity between mouse, touch, and keyboard interactions with visible focus and accessible announcements.
- **Clean Test Verification:** All 113 unit, component, and accessibility tests pass cleanly across the frontend test suite.
