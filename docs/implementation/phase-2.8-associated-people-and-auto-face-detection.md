# Implementation Report — Phase 2.8: Associated People in Media & Automatic Face Detection

**Date:** 2026-10-29  
**Status:** **COMPLETE**  
**Milestone:** 2.8  
**UX Review Verdict:** **PASS** (`docs/ux-reviews/2026-10-29-media-associated-people-and-face-detection-ux-review.md`)

---

## 1. Overview & Objective

Phase 2.8 introduced associated (untagged) relatives visibility in the Media Lightbox Studio, non-face person associations, and client-side automatic face detection with candidate bounding box suggestions.

### Core Problems Addressed
1. **Untagged Depicted Relatives:** Individuals associated with a photo (e.g. whose profile picture it is, or who are depicted without bounding boxes) were invisible in the media sidebar.
2. **Manual Face Drawing Chore:** Tagging multiple faces required manually drawing rectangles for each person from scratch.
3. **Destructive Media Loss Risk:** Media deletion lacked a confirmation safeguard.

---

## 2. Key Changes Implemented

### 2.1 Backend Associated Links & Avatar Reconciliation (`backend/src/services/privacy.rs`)
- In `sync_media_links_and_living`: automatically creates and preserves links for individuals whose `avatar_media_id == media.id`.
- In `delete_face_tag`: retains depicted links so individuals transition seamlessly to untagged "Associated Relatives" rather than vanishing.
- In `redact_media_for_caller`: shields living persons in `linked_person_ids` and `links` for public guests.

### 2.2 Associated People in Media Panel (`MediaMetadataPanel.tsx`)
- Partitioned into **Tagged Faces** and **Associated Relatives** with distinct badges (`Primary Avatar`, `Profile photo`, `Associated`).
- **"+ Tag Face" Quick-Arming:** Clicking "+ Tag Face" arms that specific relative in the canvas banner without opening search pickers.
- **"+ Link Person" Dialog (`LinkPersonDialog.tsx`):** Allows associating tree individuals with cross-script live filtering.
- **Destructive Deletion Confirmation (`useConfirm()`):** Media deletion prompts with localized warnings to prevent accidental data loss.

### 2.3 Client-Side Automatic Face Detection (`faceDetection.ts` & `FaceTagCanvas.tsx`)
- **Native Shape Detection API:** Utilizes `window.FaceDetector` where supported, with lightweight canvas skin-tone heuristic fallback.
- **Candidate Rectangles & Deduplication:** Detected candidates are deduplicated against existing tags and each other ($\text{IoU} \ge 0.30$).
- **1-Click Adoption & Dismissal:** Users can click or press Enter/Space on pulsing candidate boxes to immediately adopt them, or click the dismiss button (`✕`) to reject false positives.
- **Contrast & A11y Polish:** Candidate boxes use `border-sky-600 dark:border-sky-400` ($\ge 4.01:1$ contrast) and full WAI-ARIA button semantics.

---

## 3. Verification & Test Coverage

- **Backend Integration Tests:** `backend/tests/media_test.rs` passing (4/4 tests).
- **Frontend Unit Tests:** `frontend/tests/components/milestone-2-8-associated-people-face-detection.test.tsx` (18/18 tests, 234/234 total frontend tests passing).
- **i18n Parity:** 100% EN/RU key parity with zero glyphs or emojis in translation strings.
- **UX Review:** UX-078 through UX-086 resolved and verified **PASS**.
- **QA Verification:** Verified **PASS** against all Definition of Done criteria.

