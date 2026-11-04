# Printable Tree PDF — Implementation Memory

**Spec:** [docs/features/05-printable-tree-pdf.md](../../features/05-printable-tree-pdf.md) · **Status:** implemented

## 1. Summary
Generates print-ready high-resolution vector PDF charts of family lineages. Supports single-sheet large-format posters (A0, A1, A2, Arch E) and multi-page tiled PDFs for standard home printers (A4, US Letter) with alignment cut marks and overlap margins. Configured via the Print Studio frontend UI.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Routes | `backend/src/routes/rbac.rs` | `export_printable_pdf`, `get_pdf_preview_info` |
| PDF Engine | `backend/src/services/pdf/mod.rs` | PDF generation pipeline coordinator |
| Layout Planner | `backend/src/services/pdf/layout.rs` | Coordinate calculations, ancestor/descendant positioning, sheet bounding boxes |
| Vector Renderer | `backend/src/services/pdf/render.rs` | Direct PDF/SVG stream rendering with embedded TrueType fonts |
| Tiling Engine | `backend/src/services/pdf/tiling.rs` | Computes tile grids, cut guides, overlap margins, page headers |
| Themes | `backend/src/services/pdf/themes.rs` | Color themes: Modern Clean, Classic Parchment, High Contrast Monochromatic |
| Embedded Fonts | `backend/assets/fonts/` | `PTSerif-Regular.ttf`, `PTSerif-Bold.ttf`, `Roboto-Regular.ttf`, `Roboto-Bold.ttf` |
| Frontend Studio | `frontend/src/pages/PrintExportPage.tsx` | Print studio page wrapper |
| UI Controls | `frontend/src/components/print/` | `PrintConfigSidebar.tsx`, `PrintPreviewCanvas.tsx`, `PrintSummaryBar.tsx` |
| Tests | `backend/tests/printable_pdf_tests.rs` | Poster bounding box tests, multi-page tiling, Cyrillic font glyph tests |
| UI Tests | `frontend/tests/components/` | `print-studio.test.tsx` |

## 3. Key Flows
1. **Live Print Preview:**
   - User navigates to `/print?root=I0001` in Print Studio.
   - Adjusts configuration (layout type, generations, paper size, partner inclusion, theme).
   - Frontend requests `POST /api/v1/export/pdf/preview-info` → returns total sheet count, physical dimensions (mm), and node counts.
   - `PrintPreviewCanvas` displays interactive layout with tiling seam outlines.
2. **Export Generation:**
   - User clicks "Download PDF" → triggers `POST /api/v1/export/pdf`.
   - Backend renders PDF buffer with vector cards and cut marks, returning `Content-Type: application/pdf`.

## 4. Invariants & Business Rules
- **Physical Scale Consistency (`GOT-007`):** Coordinates are converted accurately between millimetres and typographic points (`72 pt = 1 inch`).
- **Cyrillic Font Embedding:** All exported PDFs embed PT Serif and Roboto font subsets directly in the document stream to guarantee correct rendering on all operating systems.

## 5. Decisions
- **Server-Side Generation:** PDF rendering executes completely on the backend to avoid browser print CSS inconsistencies and memory crashes on huge trees.

## 6. Gotchas & Known Limitations
- `GOT-007`: Line drops or cut marks shifting when switching between portrait and landscape tiled sheets.

_Last curated: 2026-11-04 (memory bank bootstrap)_
