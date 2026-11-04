# Media & Face Tagging — Implementation Memory

**Spec:** [docs/features/06-media-face-tagging.md](../../features/06-media-face-tagging.md) · **Status:** implemented

## 1. Summary
Rich photo and document management for family archives. Supports batch uploads, ZIP media bundle ingestion (up to 500MB) with automatic GEDCOM stub reconciliation, EXIF metadata extraction, interactive coordinate-based face tagging, client-side automatic face suggestion rectangles, and associated person linkages (tagged faces vs untagged profile/depicted links).

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Migrations | `backend/migrations/` | `media`, `media_links`, `media_face_tags`, `media_bundle_batches` |
| Routes | `backend/src/routes/rbac.rs` | Media CRUD, `/media/bundle`, `/media/{id}/crop`, `/media/{id}/tags` |
| Bundle Service | `backend/src/services/media_bundle.rs`| ZIP extraction, file hash calculation, stub matching engine |
| Gallery Page | `frontend/src/pages/MediaPage.tsx` | Media grid, filter by person/place/date, batch upload dropzone |
| Lightbox Studio | `frontend/src/components/media/` | `MediaLightboxStudio.tsx` (full-screen viewer, zoom/pan) |
| Face Tagging | `frontend/src/components/media/` | `FaceTagCanvas.tsx` (draggable bounding boxes, face crop generation) |
| Metadata Panel | `frontend/src/components/media/` | `MediaMetadataPanel.tsx` (title, date, place picker, associated people list) |
| Tests | `backend/tests/media_test.rs` | Upload validation, crop dimension math, face tag permissions |
| Bundle Tests | `backend/tests/media_bundle_reconciliation_test.rs` | Reconciling media ZIP archive with GEDCOM stubs |
| Frontend Tests | `frontend/tests/components/` | `media-gallery.test.tsx`, `milestone-2-8-associated-people-face-detection.test.tsx` |

## 3. Key Flows
1. **Interactive Face Tagging:**
   - User opens photo in `MediaLightboxStudio`.
   - Clicks "Tag Face" → `FaceTagCanvas` enables interactive rectangle drawing.
   - User selects family member from autocomplete picker.
   - Frontend posts normalized coordinates `{ x, y, width, height }` (0.0 to 1.0) to `POST /api/v1/media/{id}/tags`.
2. **Crop Generation:**
   - When displaying person avatars in tree nodes or profile headers, client calls `GET /api/v1/media/{id}/crop?tag_id=...` which streams server-side cropped image thumbnail.
3. **ZIP Bundle Reconciliation:**
   - Admin uploads ZIP of photo files after a GEDCOM import → Backend unzips, computes SHA256 hashes, and matches filenames to imported `OBJE` stubs.

## 4. Invariants & Business Rules
- **Privacy for Living Person Media:** Photos linked exclusively to living individuals are hidden from unauthenticated public visitors (`ADR-005`).
- **Normalized Coordinates:** Face coordinates are always stored as proportional fractions (0.0–1.0) relative to image dimensions, ensuring coordinates remain valid if source files are resized.

## 5. Decisions
- **Client-Side Face Suggestion:** Uses HTML5 canvas edge/shape heuristic algorithms directly in the browser to suggest initial face bounding boxes without expensive server-side ML dependencies.

## 6. Gotchas & Known Limitations
- High-resolution smartphone uploads (HEIC/large JPEG) require client-side pre-scaling before canvas drawing to prevent memory exhaustion on mobile browsers.

_Last curated: 2026-11-04 (memory bank bootstrap)_
