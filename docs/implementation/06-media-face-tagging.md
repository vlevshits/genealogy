# Implementation Report: Phase 1.9 — Media Upload, Gallery, Face Tagging & Avatars

**Date:** 2026-10-27  
**Status:** Completed & Tested  
**Scope:** Phase 1, Item 9 of [`docs/roadmap.md`](../roadmap.md) — Media upload, drag-and-drop batch upload, fluid masonry Media Archive (`/media`), interactive dual-pane Face Tagging Lightbox Studio, normalized coordinate bounding boxes (`0.0..1.0`), primary profile avatar synchronization, documentary evidence linking, and living-person privacy shielding ([`06-media-face-tagging.md`](../features/06-media-face-tagging.md), [`04-media-gallery-page.md`](../pages/04-media-gallery-page.md), [`02-rbac-access-control.md`](../features/02-rbac-access-control.md), [`10-core-data-model.md §4.5`](../features/10-core-data-model.md)).

---

## 1. Summary of What Was Implemented

### 1.1 Backend Storage, Streaming & Face Tagging Engine (`backend/`)
- **Binary Storage & Ingestion Pipeline (`POST /api/v1/media`):**
  - Accepts image, PDF, audio, and video formats up to 50MB per file with automatic validation against permitted MIME types.
  - Automatically computes SHA-256 digests (`sha256`), detects dimensions (`width`, `height`), allocates canonical GIDs (`M0100`...), and persists files bit-for-bit to disk under `<media_storage_path>/originals/`.
  - Integrates `GenDate` fuzzy parsing into `date_sort` and auto-resolves hierarchical places (`place_id` / `place_path`).
- **Media Gallery Archive (`GET /api/v1/media`):**
  - Authenticated-only endpoint (`Viewer`, `Editor`, `Admin`) supporting query parameters: `q`, `person_id`, `decade`, `type` (`all`, `photos`, `documents`, `audio`), `untagged` boolean, and cursor pagination.
  - Full-text search across titles, descriptions, original filenames, place names, and tagged people using pre-1918 orthography normalization and transliteration.
- **Media Detail, Editing & Safe Deletion (`GET/PATCH/DELETE /api/v1/media/{id}`):**
  - Optimistic concurrency control via `version` field (`409 Conflict` on mismatch).
  - Deletion automatically clears primary avatar designations (`avatar_media_id`, `avatar_crop`, `avatar_url`) on any individuals using the asset, and detaches the asset from event media links.
- **Streaming & Dynamic Avatar Cropping (`GET /api/v1/media/{id}/file`, `GET /api/v1/media/{id}/crop`):**
  - Original file streaming with proper MIME types, `Content-Length`, and cache headers.
  - Crop endpoint (`/crop?x=..&y=..&w=..&h=..`) accepts normalized coordinates (`0.0..1.0`), extracts high-resolution face crops, and caches square WebP thumbnails for profile avatars.
- **Normalized Face Tagging & Primary Avatar Synchronization:**
  - `POST /api/v1/media/{id}/tags`, `PATCH /api/v1/media/{id}/tags/{tag_id}`, `DELETE /api/v1/media/{id}/tags/{tag_id}` validate normalized bounding boxes (`0.0 <= x, y <= 1.0`, `0.0 < w, h <= 1.0`).
  - Designating a tag as primary avatar atomically updates the individual's `avatar_media_id`, `avatar_crop`, and `avatar_url`, instantly cascading to Tree Canvas node cards, directory rows, and profile headers.
- **Documentary Evidence Linking (`POST/DELETE /api/v1/media/{id}/links`):**
  - Links media assets to individuals, families, and life events as documentary citations.
- **Living-Person Privacy Shield:**
  - Unauthenticated visits to `/media` redirect to `/login`; unauthenticated direct requests return `401 Unauthorized`.
  - Any media asset tagged solely to living individuals returns `404 Not Found` for unauthenticated requests.
  - Group historical photos containing deceased ancestors alongside living descendants display deceased relatives while completely redacting living tags and names for anonymous viewers.

### 1.2 Frontend Media Archive & Lightbox Studio (`frontend/`)
- **Media Archive Page ([`MediaPage.tsx`](../../frontend/src/pages/MediaPage.tsx)):**
  - Mounted at `/media` with strict authentication boundary (`!isAuthenticated -> redirect /login`).
  - Synchronizes active filters (decade, person, type, untagged, query) with URL search parameters.
- **Fluid Masonry Media Wall ([`MediaWall.tsx`](../../frontend/src/components/media/MediaWall.tsx), [`MediaCard.tsx`](../../frontend/src/components/media/MediaCard.tsx)):**
  - Responsive multi-column layout (`columns-1 sm:columns-2 lg:columns-3 gap-5`) preserving native aspect ratios without layout shifts.
  - Lazy-loaded thumbnails with date badges, GID badges, tagged relative mini-avatars, and hover overlays.
- **Filter & Action Bar ([`MediaFilterBar.tsx`](../../frontend/src/components/media/MediaFilterBar.tsx)):**
  - Search input, person filter dropdown, decade timeline chips (`1880s`–`2000s`), asset type segmented control, `Untagged Only` toggle, and `+ Upload Media` button (Editor/Admin only).
- **Batch Upload Dialog ([`MediaUploadDialog.tsx`](../../frontend/src/components/media/MediaUploadDialog.tsx)):**
  - Accessible modal with drag-and-drop `FileDropzone`, multi-file upload progress bar, and metadata fields (title, date taken, place, privacy).
- **Face Tagging Lightbox Studio ([`MediaLightboxStudio.tsx`](../../frontend/src/components/media/MediaLightboxStudio.tsx)):**
  - Full-screen dual-pane workspace opening upon photo click.
  - **Left Canvas Pane ([`FaceTagCanvas.tsx`](../../frontend/src/components/media/FaceTagCanvas.tsx)):**
    - High-resolution pan and zoom controls (`50%`–`250%`, fit-to-screen, reset).
    - Viewer mode: Hovering over face boxes displays floating tooltip cards with full name, vital dates (`†`), and profile links.
    - Editor mode: Interactive `Tag Faces` tool supporting click-and-drag drawing, 8-handle resizing, center movement with normalized coordinates, and tree member autocomplete via `PersonFacePicker.tsx`.
  - **Right Metadata Panel ([`MediaMetadataPanel.tsx`](../../frontend/src/components/media/MediaMetadataPanel.tsx)):**
    - Editable title, description, date taken, place selector (read-only for Viewer).
    - List of tagged individuals with avatar, name, vital dates, "Set as primary avatar" star button, and delete tag action.
- **Profile Integration ([`ProfileMediaTab.tsx`](../../frontend/src/components/profile/ProfileMediaTab.tsx)):**
  - Person Profile media tab opens photos directly in `MediaLightboxStudio`. Setting a primary avatar immediately updates the profile hero header.

---

## 2. Code Review & Verification

| Check | Tool / Command | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Architect Review** | `architect-reviewer` (Mode 2) | **APPROVE** | 0 blocker/major findings. Validated living shield, normalized coordinates, and avatar cascade. |
| **Backend Integration Tests** | `cargo test --test media_test` | **PASS (3/3)** | Verified upload, streaming, `/crop`, filtering, face tags, avatar sync, entity links, and privacy shield. |
| **Full Backend Test Suite** | `cargo test` | **PASS (40/40)** | 10 integration test suites passed with zero regressions. |
| **Clippy & Code Formatting** | `cargo fmt -- --check && cargo clippy -- -D warnings` | **PASS** | Zero linter warnings or style defects. |
| **Frontend Test Suite** | `npm test -- --run` | **PASS (84/84)** | 12/12 test files passed including `media-gallery.test.tsx` and i18n parity. |
| **Production Build** | `npm run build` (`tsc --noEmit && vite build`) | **PASS** | Clean build in 1.09s with zero TypeScript errors. |
| **Interactive QA / Browser** | `qa-engineer` (Playwright) | **PASS** | Live browser tests verified `/media` auth guard, masonry filtering with URL sync, batch upload dialog, Lightbox Studio drawing, and instant avatar sync on `/persons/:id`. |

---

## 3. UI & i18n Quality Conformance

- **Zero Icon Glyphs / Emojis:** Verified 100% key parity between `en.json` and `ru.json` with zero decorative emojis or icon characters in translation strings.
- **Design System Components:** All controls use design-system primitives (`Button`, `IconButton`, `TextInput`, `Select`, `Dialog`, `FileDropzone`, `ProgressBar`); zero raw `<button>`, `<input>`, `<select>`, `<textarea>` elements.
- **No Living Pill:** Living individuals display birth details only without a "Living" pill; deceased individuals are denoted by subtle `†`.
