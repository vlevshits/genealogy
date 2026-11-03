# Implementation Report: Phase 3 — Printable PDF Posters & Tiling

**Feature:** Phase 3 — Printable PDF Posters / Tiles ([05](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/05-printable-tree-pdf.md), [Page Spec 05](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/05-printable-pdf-page.md), [Roadmap L335](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/roadmap.md#L335))  
**Status:** Completed & Verified  
**Date:** 2026-11-02  
**UX Review Verdict:** PASSED ([Report](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/phase-3-printable-pdf.md))  

---

## 1. Executive Summary

This feature empowers administrators to transform digital genealogy family trees into high-resolution, physical keepsakes. It delivers two distinct printing formats:
1. **Single-Page Poster Mode:** Large-format vector output (A0, A1, A2, A3) suitable for professional print shops.
2. **Multi-Page Tiled Mode:** Automatically partitioned across an $N \times M$ grid of standard home sheets (A4 or US Letter) with configurable overlap margins (5–20mm, default 10mm), dashed cut lines (`✂ CUT LINE`), corner alignment crosshairs (`+`), and sheet assembly coordinate footers (`Sheet X of Y [Row R, Col C]`).

All generation is executed by a high-performance native Rust vector engine with TrueType font embedding, sub-second latency (<800ms), direct HTTP binary streaming, strict Admin-only RBAC protection, living individual privacy masking, and an interactive **Printable PDF Studio** at `/export/print`.

---

## 2. Architecture & Components

### 2.1 Native Rust Vector PDF Engine (`backend/src/services/pdf/`)
- **Crate Selection:** `printpdf = "0.7"` + `image = "0.25"`. Native pure-Rust PDF generation with vector lines, cubic bezier curves, rectangle fills, and 300 DPI image embedding without external C library dependencies.
- **Embedded Typography (`backend/assets/fonts/`):**
  - **Modern Clean (Sans-serif):** `Roboto-Regular.ttf` & `Roboto-Bold.ttf`.
  - **Heritage Antique (Serif):** `PTSerif-Regular.ttf` & `PTSerif-Bold.ttf`.
  - Bundled directly into the compiled binary via `include_bytes!`, ensuring offline / containerized execution with 100% Cyrillic (RU) and Latin (EN) glyph coverage.
- **Layout & Slicing (`layout.rs` & `tiling.rs`):**
  - Millimeter-space coordinate placement for `pedigree`, `hourglass`, and `descendant` tree projections.
  - Slicing geometry computes effective pitch ($W_{cell} = W_{sheet} - 2 M_{print} - O_{margin}$) and partitions master bounds into an $N \times M$ grid without dropped content.
  - Injects shaded overlap margins, dashed cut lines, corner crosshairs, and coordinate footers on every sheet.
  - Optional Sheet 1 Assembly Guide matrix with visual bird's-eye map of the grid.
- **Privacy & Living Redaction:**
  - When `redact_living = true`, living persons' names are masked ("Living" / "Ныне живёт"), dates replaced by "—", and portrait photos suppressed.

### 2.2 API Contracts (`backend/src/routes/rbac.rs`)
- `POST /api/v1/export/pdf`: Streams `application/pdf` binary attachment with `X-PDF-Total-Sheets` and `X-PDF-Grid` headers.
- `POST /api/v1/export/pdf/preview-info`: Returns calculated metadata (`total_sheets`, `grid_columns`, `grid_rows`, `canvas_width_mm`, `canvas_height_mm`, `sheet_width_mm`, `sheet_height_mm`, `overlap_mm`, `total_nodes`, `estimated_print_time_sec`).
- **RBAC:** Guarded by `RequireAdmin`. Unauthenticated requests return `401 Unauthorized`; Viewers and Editors receive `403 Forbidden`.

### 2.3 Printable PDF Studio Frontend (`/export/print`)
- **Dual-Pane Studio (`frontend/src/pages/PrintExportPage.tsx`):**
  - Left Sidebar (`w-96`): Focal person picker (`PersonPicker`), chart layout, generation depth slider (3–8), printing format toggle (Tiled vs Poster), paper size selector, overlap bleed slider, cut marks & assembly guide checkboxes, theme cards, living redaction checkbox, portrait photo toggle, and sticky primary download CTA.
  - Right Preview Area (`flex-1`): Interactive visual tree preview with dynamic paper aspect ratio, theme styling (Modern Clean vs Heritage Antique), tiled sheet boundary overlay with cut lines and crosshairs, and zoom toolbar (Zoom in, Zoom out, Fit to view, 100% reset).
  - Summary Bar: Metric chips showing total sheet count & grid layout, total physical dimensions in mm, overlap bleed, living privacy status, and person count.
  - Mobile & Tablet Viewport Toggle: Responsive `SegmentedControl` on `<lg` screens switching between configuration controls and live preview.

---

## 3. Definition of Done (DoD) Verification

| Item | Requirement | Status | Evidence |
| :---: | :--- | :---: | :--- |
| 1 | Single-Page PDF Generation (A0, A1, A2, A3 vector PDFs) | **PASS** | `test_single_page_pdf_generation_all_sizes_a0_a1_a2_a3` |
| 2 | Multi-Page Tiling Math ($N \times M$ grid of A4/Letter pages) | **PASS** | `test_multi_page_tiling_math_cleanly_partitions_grid_without_dropped_content` |
| 3 | Assembly Markers (Dashed cut lines, corner crosshairs `+`, coordinate footers) | **PASS** | `test_admin_can_generate_tiled_multi_page_pdf_with_cut_marks_and_overlap` |
| 4 | Overlap Margins (5–20mm configurable bleed) | **PASS** | `test_configurable_overlap_margins_5mm_to_20mm` |
| 5 | Theme Support (Modern Clean & Heritage Antique) | **PASS** | `test_both_modern_and_heritage_themes_generate_valid_pdf` |
| 6 | Photo Inclusion & Living Person Masking | **PASS** | `test_photo_inclusion_and_living_person_photo_masking` |
| 7 | Preview Studio (Dual-pane UI at `/export/print`) | **PASS** | `frontend/tests/components/print-studio.test.tsx` (11 passing tests) |
| 8 | Admin-Only Isolation (403 for non-admins, AccessDeniedCard) | **PASS** | `test_non_admin_callers_receive_403_or_401` & RBAC frontend guards |
| 9 | i18n Quality (100% EN/RU parity, zero emojis/glyphs) | **PASS** | `tests/i18n/parity.test.ts` (16 passing tests) |

---

## 4. UX Review Outcome & Resolution

### 4.1 Phase 3 Initial Studio Review
The initial UX review of `/export/print` was executed and reached a **PASSED** verdict ([`docs/ux-reviews/phase-3-printable-pdf.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/phase-3-printable-pdf.md)).
All identified tasks UX-087 to UX-097 were resolved:
- **UX-087 [CRITICAL]:** Added mobile view switcher tab (`config` vs `preview`) on `<lg` viewports.
- **UX-088 [MAJOR]:** Added `pb-28 lg:pb-6` bottom padding in sidebar for mobile `BottomNav` clearance.
- **UX-089 [MAJOR]:** Suppressed global footer on `/export/print` in `AppShell` and fixed viewport height styling.
- **UX-090 [MAJOR]:** Fixed Russian living person localization (`Ныне живёт`) and added Cyrillic ancestor hierarchy.
- **UX-091 [MAJOR]:** Implemented dynamic paper aspect ratio canvas scaling, eliminating horizontal scrolling on 1280px laptops.
- **UX-092 [MAJOR]:** Added `aria-hidden="true"` to cut marks/crosshairs and `role="region"` with accessible label on scrollable canvas.
- **UX-093 [MAJOR]:** Added programmatic accessible label to chart title text input (`htmlFor` / `id="print-chart-title-input"`).
- **UX-094 [MINOR]:** Sticky sidebar download CTA and secondary toolbar variant.
- **UX-095 [MINOR]:** Slavic ICU pluralization for sheet grid ("4 листа") and localized generation tier headers.
- **UX-096 [SUGGESTION]:** Added accessible "Fit to view" button (`Maximize2`) in zoom toolbar.
- **UX-097 [MINOR]:** Omitted breadcrumbs bar on `/export/print` matching `/tree`, eliminating residual 38px overflow.

### 4.2 Phase 3 Milestone 3.1 Expansion Review (`TASK-PDF-STUDIO-EXPANSION`)
The follow-up UX review for the Tree View feature parity expansion reached a **PASSED** verdict ([`docs/ux-reviews/phase-3-printable-pdf-expansion.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/ux-reviews/phase-3-printable-pdf-expansion.md)).
All identified tasks were resolved:
- **UX-098 [MAJOR]:** Refactored chart layout `SegmentedControl` into a 2×2 grid (`w-full grid grid-cols-2 gap-1 items-stretch`), preventing horizontal overflow on the 384px sidebar and 390px mobile viewports.
- **UX-099 [MAJOR]:** Replaced hardcoded English sheet tags and assembly guide tags in the SVG canvas with localized strings (`Лист X из Y [Ряд R, Колонка C]` and `Лист 1: Схема сборки включена`).
- **UX-100 [MINOR]:** Added `ariaLabel={t('print.focalPerson')}` to `PersonPicker` (forwarded to `Combobox`) and linked subtitle helper note via `aria-describedby="print-chart-subtitle-hint"`.
- **UX-101 [SUGGESTION]:** Added accessible `IconButton` clear button (`X`) in Title and Subtitle inputs to immediately revert to dynamic smart defaults.

---

## 5. Milestone 3.1 Expansion Details (`TASK-PDF-STUDIO-EXPANSION`)

To achieve full feature parity with the interactive tree visualization modes, Phase 3 Milestone 3.1 delivered:
1. **All Nodes Mode (`view_mode: "all_nodes"`):**
   - Full forest graph projection partitioning disconnected components via Disjoint Set Union (DSU) with $\ge 30\text{mm}$ horizontal clearance.
   - Singletons shelf: isolated individuals arranged in a neat 4-column shelf grid at the bottom.
   - Generation depth slider automatically hidden with informative archive explanation.
2. **Hourglass Collateral Density:**
   - Wide (`collateral: true`): collateral relatives (aunts/uncles, first cousins, nieces/nephews) included on generational baselines.
   - Compact (`collateral: false`): strictly lineal ascendants and descendants.
3. **Spouse & Sibling Inclusion:**
   - Spouse/partner checkbox (`include_partner`): marital connector rendered with amber ochre stroke (`#d97706`).
   - Sibling checkbox (`include_siblings`): siblings branch from parental distribution bus on Gen 0.
4. **Portrait Photos & Living Masking:**
   - 300 DPI vector circular cropped images with SVG `<clipPath>`.
   - Living individuals masked in unauthenticated/guest contexts.
5. **Custom Headline & Subtitle Banner:**
   - User-defined title and subtitle inputs with localized dynamic fallback when empty.
   - Quick reset affordance (`UX-101`).
6. **Fan Chart Safeguards:**
   - Visual disable and explanatory hint banner (`print.fanDisabledHint`) for incompatible controls when in fan chart projection.

---

## 6. Technical Debt Registered

- **`TD-PDF-01` (Streaming Spooling / Job Queue for Mega-Trees >2,000 Persons):** In-memory vector PDF generation executes in <800ms for typical family trees (3–8 generations, up to ~500 individuals). For extreme trees (>2,000 individuals or hundreds of uncompressed 4K portraits), spooling to temporary files or offloading to the Postgres background job queue with SSE progress updates can be introduced if single-request RAM usage exceeds 64MB. Registered in [`docs/technical-debt.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/technical-debt.md).
- **`TD-PDF-02` (Client-side Preview Avatar Reverse-Proxy Fallback):** In live SVG preview, avatar portraits render circular vector silhouette fallbacks or loaded URLs. If client media is behind protected reverse-proxy endpoints without credentials in SVG `<image>` tags, the live preview renders the vector placeholder while the final rendered PDF embeds the full 300 DPI image bytes from the server-side media cache. Registered in [`docs/technical-debt.md`](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/technical-debt.md).

