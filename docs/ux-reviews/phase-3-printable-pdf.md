# UX Review — Phase 3: Printable PDF Posters & Tiling — 2026-11-02 (Re-Review)

## Verdict: PASSED

Screens & routes reviewed:
- `/export/print` (Printable PDF Studio — Dual-Pane Studio Layout: interactive configuration sidebar, summary bar, scalable print preview canvas, sheet tiling grid overlay, crop bleed markers, and PDF download pipeline).

Components inspected:
- `frontend/src/pages/PrintExportPage.tsx`
- `frontend/src/components/print/PrintConfigSidebar.tsx`
- `frontend/src/components/print/PrintPreviewCanvas.tsx`
- `frontend/src/components/print/PrintSummaryBar.tsx`
- `frontend/src/components/print/index.ts`
- `frontend/src/components/ui/app-shell.tsx`
- `frontend/src/App.tsx`
- `frontend/src/i18n/en.json` & `ru.json`

Viewports & Locales Verified:
- Desktop (1440px), Laptop (1280px), Tablet (768px), Mobile (390px).
- Light and Dark themes.
- English (`en`) and Russian (`ru`).
- Playwright Browser verification:
  - Mobile view switcher tab (`SegmentedControl`) toggling between `config` and `preview` at 390px.
  - Sticky CTA download button and bottom input clearance above `BottomNav` (`pb-28 lg:pb-6`).
  - Suppression of global footer on `/export/print` in `app-shell.tsx`.
  - Russian living person localization (`А. (Ныне живёт)`) and Cyrillic simulated ancestor names (`Пётр Фёдорович Петров`, etc.).
  - Dynamic canvas aspect ratio calculation and zero horizontal scrolling on 1280px laptop viewport.
  - Decorative crosshairs and scissors wrapped in `aria-hidden="true"`, with `role="region"` and accessible label on scrollable canvas.
  - Accessible label on chart title text input (`htmlFor="print-chart-title-input"` / `id="print-chart-title-input"`).
  - Sticky primary CTA in sidebar and secondary variant in preview toolbar.
  - Russian pluralization (`4 листа`) and generation tier labels (`Поколение 1`).
  - "Fit to view" button (`Maximize2`) in zoom toolbar.

---

## Status of Previous Tasks
| ID | Title | Status (fixed / still open / obsolete) |
| :--- | :--- | :--- |
| **UX-001** | Fix mobile toolbar overlap and bottom navigation occlusion on tree canvas | fixed |
| **UX-007** | Fix `BottomNav` accessibility label and replace raw 36px buttons | fixed |
| **UX-021** | Keyboard zoom shortcuts or visible hint in `PlacesMapView` and `FanChartCanvas` | suggestion (open) |
| **UX-022** | DropdownMenu triggers (`AppShell` nav more, +add, user profile) inaccessible to keyboard navigation | open |
| **UX-023** | Modal Dialogs and Drawers lack keyboard Tab focus trapping and initial focus management | open |
| **UX-024** | Sub-40px touch targets and raw `<button>` elements in inputs and media controls | open |
| **UX-036** | Form error alerts lack programmatic association (`aria-invalid`, `aria-describedby`) with invalid fields | open |
| **UX-037** | Stacking / Overflow collision in `TreeToolbar` on tablet and medium viewports (640px - 1024px) | fixed |
| **UX-038** | Direct-lineage vertical stems overwritten by collateral SVG paths | fixed |
| **UX-039** | Low non-text contrast (< 3:1) for collateral connector edges in dark mode | fixed |
| **UX-040** | Initial camera pan hardcoded to `{ x: 480, y: 280 }`, placing tree root off-screen on mobile screens | fixed |
| **UX-078** | Unconfirmed Destructive Action on Media Asset Deletion (Data Loss Risk) | fixed |
| **UX-079** | Overlapping Collision of Person Name and "Primary Avatar" / "Associated" Badges on Mobile Viewports | fixed |
| **UX-080** | Keyboard & Screen Reader Inaccessible Auto-Detected Face Candidate Rectangles | fixed |
| **UX-081** | Target Person Armed Banner Truncates and Hides Relative's Name on Mobile Viewports | fixed |
| **UX-082** | Sub-40px Touch Targets on Studio Navigation, Move/Resize Handles, and Panel Buttons | fixed |
| **UX-083** | Duplicate "Auto-Detect Faces" Buttons Rendered Simultaneously on Canvas Stage | fixed |
| **UX-084** | Auto-Detected Face Candidates Lack Dismiss / Reject Affordance for False Detections | fixed |
| **UX-085** | Low Contrast on Sky-Blue Candidate Rectangles (`border-sky-400`) in Light Theme | fixed |
| **UX-086** | WAI-ARIA Structure Violation: Nested `<button>` Inside `role="option"` and Missing Arrow Key Navigation | fixed |
| **UX-087** | Mobile & Tablet (<1024px) Preview Canvas Completely Unreachable Due to `h-full` Sidebar in `overflow-hidden` Container | fixed |
| **UX-088** | Mobile BottomNav Occludes Bottom Interactive Controls & Download CTA in Print Sidebar | fixed |
| **UX-089** | Triple Scrollbar Collision & Window Scroll Overflow from `h-[calc(100vh-3.5rem)]` and `AppShell` Footer | fixed (residual 38px overflow tracked in UX-097) |
| **UX-090** | Hardcoded English `(Living)` Suffix and Transliterated Ancestor Names in Russian Locale | fixed |
| **UX-091** | Fixed Canvas Dimensions (880×620px) Distort Paper Aspect Ratio and Force Horizontal Scroll on 1280px Screens | fixed |
| **UX-092** | Screen Reader Pollution from Decorative Alignment Crosshairs (`+` Glyphs) and Missing Accessible Label on Canvas Region | fixed |
| **UX-093** | Missing Programmatic Accessible Label on Chart Title Input (`TextInput`) | fixed |
| **UX-094** | Duplicate Primary Action Buttons and Non-Sticky Sidebar CTA | fixed |
| **UX-095** | Russian Toolbar Text Wrapping and Slavic Pluralization Errors in Sheet & Generation Counters | fixed |
| **UX-096** | Add "Fit to Window" Zoom Preset and Canvas Pan Affordance | fixed |

---

## Tasks for frontend-developer

### UX-097 [minor] Residual 38px Document Scrollbar from Breadcrumbs Bar Height Collision with `h-[calc(100dvh-4rem)]`
- **Screen / viewport / theme / lang:** `/export/print` (`PrintExportPage.tsx:231`, `App.tsx:406-409`), desktop viewports (1440px / 1280px), all themes and locales.
- **Status:** **FIXED** (Option 1 implemented in `App.tsx`: breadcrumbs omitted on full-viewport studio route matching `/tree`, eliminating the 38px overflow and fitting the 100dvh studio cleanly).
- **Tags:** layout, space

---

## Positive Notes
- **Flawless Mobile Viewport Adaptation:** The new `SegmentedControl` on `<lg` screens smoothly toggles between configuration controls and live PDF preview, ensuring tablet and phone users have full access to both panes without clipped content.
- **Complete Slavic Localization:** Russian grammar is strictly observed with correct genitive plural forms ("4 листа (сетка 2×2)") and Cyrillic ancestor hierarchy ("Пётр Фёдорович Петров", "А. (Ныне живёт)").
- **High-Quality Accessibility Compliance:** The scrollable canvas has a clear `role="region"` with `tabIndex={0}`, focus indicator ring, and descriptive accessible label. Decorative crosshair `+` glyphs and scissors icons are cleanly ignored by assistive technologies via `aria-hidden="true"`.
- **Clean Action Hierarchy:** The primary CTA is sticky at the bottom of the sidebar with ample clearance above mobile navigation, and the canvas toolbar features a well-differentiated secondary variant.
- **Dynamic Paper Proportions:** Canvas dimensions adapt dynamically based on paper aspect ratio, preventing distorted previews and eliminating horizontal scrolling on standard laptop displays.
