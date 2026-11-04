# GEDCOM Import & Export — Implementation Memory

**Spec:** [docs/features/03-gedcom-import-export.md](../../features/03-gedcom-import-export.md) · **Status:** implemented

## 1. Summary
Provides high-fidelity bidirectional migration with standard GEDCOM 5.5.1 and 7.0 files. Features pre-import syntax analysis with warning previews, safe atomic import within database transactions, Cyrillic character set conversion (ANSEL, UTF-8, Windows-1251), and customizable export with living-person privacy filtering.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Routes | `backend/src/routes/rbac/mod.rs` | `analyze_gedcom`, `import_gedcom`, `export_gedcom`, body limit 500MB |
| Service | `backend/src/services/gedcom/mod.rs` | Line tokenizer, grammar parser, tag processors (`INDI`, `FAM`, `OBJE`), serializer |
| Media bundle | `backend/src/services/media_bundle.rs` | `reconcile_gedcom_media_bundle` matches ZIP files to imported `OBJE` stubs |
| Frontend components| `frontend/src/components/settings/gedcom/` | `GedcomAnalysisModal.tsx`, `GedcomPostImportReport.tsx` |
| Settings Page | `frontend/src/pages/SettingsPage.tsx` | GEDCOM management tab with file drag-and-drop |
| Tests | `backend/tests/gedcom_test.rs` | Roundtrip parsing, ANSEL/UTF-8 decoding, tag preservation |
| Fixtures | `fixtures/`, `test-data/` | Standard test trees (e.g. `royal92.ged`, Slavic Cyrillic samples) |

## 3. Key Flows
1. **Pre-Import Analysis:**
   - User uploads `.ged` file to `POST /api/v1/gedcom/analyze`.
   - Backend scans records without saving to database; reports counts (individuals, families, media links) and localized syntax warnings.
2. **Execution & Reconciliation:**
   - User confirms import → `POST /api/v1/gedcom/import` writes entities atomically.
   - If media files were referenced, user uploads companion media ZIP archive to `POST /api/v1/gedcom/media-bundle` to reconcile image stubs.
3. **Privacy-Filtered Export:**
   - User triggers `GET /api/v1/gedcom/export`.
   - If requested without Admin privilege or with privacy filtering enabled, all living persons have private events stripped before serialization.

## 4. Invariants & Business Rules
- **Non-Destructive Parsing:** Unrecognized proprietary vendor tags (e.g. `_UID`, `_FREL`) are preserved as raw note attributes rather than discarded.
- **Slavic Character Encoding:** Gracefully auto-detects ANSEL, UTF-8 with BOM, and Windows-1251 (CP1251) encodings.

## 5. Decisions
- **Pre-Analysis Step:** Always require client approval of the analysis report before executing the database commit.

## 6. Gotchas & Known Limitations
- Large GEDCOM files (>50MB) require streaming multipart parsing to avoid memory spikes. Body limit is explicitly set to 500MB in Axum routes.

_Last curated: 2026-11-04 (memory bank bootstrap)_
