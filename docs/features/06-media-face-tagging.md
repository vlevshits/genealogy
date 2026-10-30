# Feature Specification: Media Management & Face Tagging

## 1. Overview & Objective
Photographs and historical documents (scanned birth certificates, census records, gravestones, military records) are the emotional heart of family genealogy. This feature provides a modern media library with an interactive **Face Tagging Lightbox Studio** (similar to Apple Photos, Google Photos, and MyHeritage), allowing users to:
1. Ingest single photos/documents or bulk ZIP archives with automatic stub reconciliation.
2. View **all associated people** in a dedicated media panel — whether explicitly tagged with a bounding box, assigned as a person's profile avatar, or linked as a depicted individual.
3. Rapidly tag faces with **Automatic Face Detection** (suggested candidate rectangles using browser Shape Detection or client-side detection) that can be adopted with a single click, completely removing the chore of drawing manual boxes.
4. Fall back to manual bounding box drawing, resizing, and positioning whenever desired.
5. Crop and assign face tags directly as primary profile avatars across tree cards and profile headers.
6. Protect living persons via privacy shielding in public mode.

---

## 2. Data Model
This feature builds directly upon the canonical schema defined in [10-core-data-model.md](10-core-data-model.md §3).

### 2.1 Entity Relationships & Deltas
- **`media_assets`**: Stores file metadata, mime type, dimensions, fuzzy date taken, place reference, privacy flags, and missing stub indicators.
- **`media_face_tags`**: Normalized bounding box coordinates (`box_x`, `box_y`, `box_width`, `box_height` in `[0.0, 1.0]`), linking a specific face crop to an individual, with `is_primary_avatar` flag.
- **`media_links`**: Polymorphic association table (`entity_type = 'individual' | 'family' | 'event' | 'place'`) linking entities to the media asset.
- **`individuals.avatar_media_id`**: Foreign key on the individual referencing their primary profile photo asset.

```sql
CREATE TABLE media_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gid VARCHAR(32) NOT NULL UNIQUE,
    file_name VARCHAR(255) NOT NULL,
    original_file_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    sha256 VARCHAR(64) NULL,
    width INT NULL,
    height INT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NULL,
    date_taken VARCHAR(100) NULL, -- Supports fuzzy dates like "Circa 1920"
    date_sort BIGINT NULL,
    place_id UUID NULL REFERENCES places(id) ON DELETE SET NULL,
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    is_missing BOOLEAN NOT NULL DEFAULT FALSE, -- TRUE when imported from GEDCOM without binary asset
    external_path TEXT NULL,                   -- Preserved raw path from GEDCOM e.g. /root/gramps/000085_*.jpg
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE media_face_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id UUID NOT NULL REFERENCES media_assets(id) ON DELETE CASCADE,
    person_id UUID NOT NULL REFERENCES individuals(id) ON DELETE CASCADE,
    -- Normalized bounding box coordinates (0.0 to 1.0)
    box_x FLOAT NOT NULL,
    box_y FLOAT NOT NULL,
    box_width FLOAT NOT NULL,
    box_height FLOAT NOT NULL,
    is_primary_avatar BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE media_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id UUID NOT NULL REFERENCES media_assets(id) ON DELETE CASCADE,
    entity_type VARCHAR(32) NOT NULL, -- 'individual', 'family', 'event', 'place'
    entity_id UUID NOT NULL,
    role VARCHAR(64) NULL,            -- e.g. 'depicted', 'primary_avatar', 'source_evidence'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 2.2 Derivation of Associated Persons
In API responses (`MediaAssetSummaryDto`), the asset exposes:
- `face_tags: FaceTagDto[]`: Explicitly tagged face bounding boxes.
- `linked_person_ids: string[]`: De-duplicated union of all individual IDs associated with this media asset via:
  1. Explicit face tags (`media_face_tags.person_id`).
  2. Individual links (`media_links WHERE entity_type = 'individual'`).
  3. Profile avatar assignment (`individuals.avatar_media_id = media_assets.id`).
- An associated person is categorized in the UI as **Tagged** (if a corresponding `face_tags` entry exists) or **Untagged Associated** (if present in `linked_person_ids` or has `avatar_media_id == asset.id` without a corresponding `face_tags` entry).

> [!NOTE] Assumption: Profile Photo vs Associated Badge Derivation
> An untagged associated individual receives the "Profile photo" badge if their person record references this asset (`individuals.avatar_media_id == media_assets.id` or `media_links.role == 'primary_avatar'`). When an Editor tags their face, `is_primary_avatar` defaults to `true`. Other associated individuals receive the "Associated" badge (`media_links.role == 'depicted'`), and `is_primary_avatar` defaults to `false`.

> [!NOTE] Assumption: Browser Shape Detection API Support & Fallback
> Client-side face auto-detection uses the standard W3C Shape Detection API (`window.FaceDetector`), supported natively in Chromium engines. On browsers where `window.FaceDetector` is unavailable (e.g. Firefox or Safari without experimental flags), a lightweight canvas edge-heuristic detector runs, or a quiet informational toast notifies the user that auto-detection is unavailable in the current browser, seamlessly falling back to manual drag-to-draw mode without any application errors.

---

## 3. User Experience & Flows

### 3.1 Media Asset Upload & Organization
- **Supported Formats:**
  - Images: JPEG, PNG, WebP, TIFF, AVIF.
  - Documents: PDF (scanned multi-page records, certificates).
  - Audio: MP3/WAV oral history recordings.
- **Metadata Fields:** Title, Description, Date Taken (supports fuzzy genealogy dates, e.g. `abt 1935`), Place Taken, Associated Individuals, and Notes. A scan can also be attached directly to an event as documentary evidence.
- **Storage Architecture:** Stored locally in a dedicated persistent Docker volume (e.g. `/app/data/media`) with hashed filenames; configurable S3-compatible backend for cloud deployments ([17 §4.2](17-api-security-operations.md#42-storage-layout)).
- **GEDCOM Stubs & External Reconciliation:** When importing a standalone `.ged` file whose media binaries were not bundled, stub entries are created with `is_missing = TRUE`, recording the original filename and external path. Uploading matching files reconciles them in place ([03 §3.5](03-gedcom-import-export.md#35-step-5-import-report--post-import-media-reconciliation-prompt)).

### 3.2 Batch Archive Upload (.zip) & Media Reconciliation
In addition to selecting individual files, Editors and Admins can ingest photo collections in bulk using standard ZIP archives:
- **Archive Ingestion:** Accepts `.zip` bundles up to 500MB via drag-and-drop in the Media Gallery or via `POST /api/v1/media/bundle`.
- **Safe Streaming Extraction:** Extracts files on the fly via temporary disk spooling. Automatically skips macOS and Windows archive artifacts (`__MACOSX/`, `.DS_Store`, `Thumbs.db`). Enforces strict zip-slip rejection and decompression size limits.
- **Unified Dual Ingestion & Reconciliation:**
  1. **GEDCOM Stub Matching:** Matches extracted files against `is_missing = TRUE` stubs by case-insensitive base filename (`original_file_name` / `external_path`). Reconciles stub in place: stores original binary, generates responsive WebP thumbnails (256w, 768w, 1600w), and clears `is_missing = FALSE` while preserving all person associations and face tags.
  2. **Fresh Asset Creation:** For files that do not match any existing stub, creates a new `media_assets` record with `is_missing = FALSE`, adding the photo to the general media library.
- **Result Summary:** Returns total extracted count, number of reconciled stubs, newly created assets, and any skipped unsupported files.

### 3.3 Associated People in Media Panel
When viewing a photo in the Lightbox Studio, the right-hand panel features the **"PEOPLE IN THIS MEDIA"** section, providing a complete view of everyone connected to the asset:
1. **Tagged Individuals:**
   - Displays cropped face thumbnail preview, person name with link to `/persons/:id`, and life dates (e.g. `Born 1912 · † 1989`).
   - If marked as primary avatar, displays a `Primary Avatar` badge (`Star` icon).
   - Hovering over a row highlights the corresponding bounding box on the canvas.
   - Quick actions for Editors/Admins:
     - Toggle primary avatar (`Set as primary profile avatar` / `Clear primary avatar`).
     - Remove face tag (`Trash2` icon) — deletes the bounding box but retains the person as an associated individual if linked.
2. **Untagged Associated Individuals:**
   - Displays standard profile avatar (or placeholder silhouette), person name with link to `/persons/:id`, and life dates.
   - Displays association context badge:
     - `Profile Photo` badge: Assigned as the individual's profile avatar (`avatar_media_id == asset.id`), but no specific face rectangle has been cropped yet.
     - `Associated` badge: Linked to the media asset via `media_links` (or imported GEDCOM association) without a face tag.
   - Quick action: **`+ Tag Face`** button:
     - Immediately activates Tagging Mode (`isTaggingMode = true`).
     - Pre-selects this person as the active target for bounding box assignment.
     - If candidate face rectangles were detected by Auto-Detect, prompts: *"Click a suggested face rectangle or draw a box for [Person Name]"*.
     - Clicking any candidate rectangle or drawing a box immediately creates the tag for this person without requiring manual search in the picker!
   - Unlink action: Allows removing the general media link if the individual is not depicted.
3. **Link Relative Action (`+ Link Person`):**
   - Allows Editors/Admins to associate a family tree individual to this photo/document without drawing a face box (e.g., audio recordings, group documents, or distant figures whose faces cannot be isolated).

### 3.4 Interactive Face Tagging & Automatic Face Detection
When an Editor or Admin clicks **"Tag Faces"** (`Tool: Draw Face Box`):
1. **Automatic Face Detection (Suggested Candidate Rectangles):**
   - The studio provides an **"Auto-Detect Faces"** action in the tagging toolbar (`Scan` / `Sparkles` icon).
   - **Detection Engine:** Leverages the standard browser `window.FaceDetector` API (Shape Detection API supported in modern Chromium/WebKit browsers) with an in-browser canvas heuristics/lightweight detector fallback.
   - **Execution:** Analyzes the displayed high-resolution image in the client and returns detected face bounding boxes normalized to `[0.0, 1.0]`.
   - **Deduplication:** Compares candidate rectangles against existing `face_tags` on the photo using Intersection over Union (IoU $\ge 0.30$) or centroid proximity. Already-tagged faces are filtered out.
   - **Candidate Rendering:** Remaining untagged detected faces are rendered on the canvas as **Candidate Suggestion Rectangles** with distinct visual styling (pulsing dashed sky-blue border `border-sky-400 bg-sky-400/10 hover:bg-sky-400/25` and a `Suggested Face` label pill).
   - **1-Click Candidate Adoption:**
     - Clicking any candidate suggestion rectangle instantly adopts its bounding box as the active `draftBox`.
     - If an untagged person was pre-selected (via `+ Tag Face` in the panel), the system immediately confirms and creates the tag.
     - Otherwise, the `PersonFacePicker` searchable autocomplete dropdown opens immediately beneath the adopted rectangle, pre-focused on search.
     - The user selects the individual, toggles "Set as primary avatar" if desired, and confirms.
     - The candidate rectangle is removed, and the newly saved tag renders as a solid tagged box.
2. **Manual Bounding Box Drawing & Refinement (Fallback):**
   - Drawing manually is always available: clicking and dragging anywhere on the canvas draws a custom bounding box.
   - In tagging mode, existing face boxes display corner drag handles (`Move` and `Resize`), allowing precise manual adjustment of coordinates.
3. **Hover Interactions in Viewer Mode:**
   - When viewing photos (for all roles), hovering over any tagged face highlights the box and displays an elegant tooltip card with the person's name, vital dates, and profile link. Clicking navigates directly to `/persons/:id`.

### 3.5 Future Vision: Facial Embedding Extraction & Identity Clustering (Phase 3)
As a future enhancement in Phase 3 "Power Tools":
- Confirmed face tag crops will be processed by a local client-side or server-side feature extractor (such as a lightweight WebAssembly/ONNX FaceNet/MobileFaceNet model) to generate 128-dimensional or 512-dimensional facial embedding vectors.
- Embeddings will be clustered per person in the local database.
- When auto-detecting candidate faces on newly uploaded photos, candidate embeddings will be compared against existing person clusters using cosine similarity, automatically suggesting: *"Is this Aleksandr Petrov? (88% match)"*.
- Strictly preserves local privacy: zero facial vectors or images are ever transmitted to external cloud APIs or 3rd-party services.

---

## 4. Permissions & Privacy

### 4.1 Role Permissions Matrix
| Action | Public Guest (Anonymous) | Viewer | Editor | Admin |
| :--- | :---: | :---: | :---: | :---: |
| Access `/media` gallery | ❌ (Redirects to `/login`) | ✅ | ✅ | ✅ |
| View photos & tagged faces | ❌ (Direct URL only if deceased) | ✅ | ✅ | ✅ |
| View "People in this Media" panel | ❌ | ✅ (Read-only) | ✅ | ✅ |
| Download original media files | ❌ | ✅ | ✅ | ✅ |
| Upload photos & batch ZIP archives | ❌ | ❌ | ✅ | ✅ |
| Run Auto-Detect Faces & adopt candidate boxes | ❌ | ❌ | ✅ | ✅ |
| Draw, resize, move, delete face boxes | ❌ | ❌ | ✅ | ✅ |
| Link / unlink untagged individuals | ❌ | ❌ | ✅ | ✅ |
| Set / clear primary profile avatar | ❌ | ❌ | ✅ | ✅ |
| Edit title, date, place, description | ❌ | ❌ | ✅ | ✅ |
| Delete media asset | ❌ | ❌ | ✅ | ✅ |

### 4.2 Living Person Shield (Public Site Mode)
When public tree access is enabled (`ENABLE_PUBLIC_ACCESS=true`) and an anonymous visitor requests media:
- **Solely-Living Media Files:** Any photo or document where *all* linked and tagged persons are living is completely omitted from the public media gallery and returns `404 Not Found` on direct asset requests (`/api/v1/media/{id}/file`).
- **Group Historical Photos:** Photos containing deceased ancestors alongside living descendants remain viewable for historical value, but all face-tag coordinates, names, and profiles of living persons are stripped from the payload delivered to anonymous visitors ([02 §3.2](02-rbac-access-control.md#32-tier-2-public-tree-mode-strict-module-segregation--zero-edit-controls)).
- **Profile Avatars of Living Persons:** Replaced with generic placeholder silhouettes on all tree cards and profile endpoints.

---

## 5. API Endpoints

### 5.1 Single Media Upload
- **`POST /api/v1/media`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Body:** `multipart/form-data`, file field `file`, optional fields: `title`, `description`, `date_taken`, `place_id`.
  - **Behavior:** Reconciles matching `is_missing = TRUE` stub if base filename matches; otherwise creates new asset.
  - **Response (`201 Created`):** `MediaAssetSummaryDto`.

### 5.2 Batch Media Archive Upload & Reconciliation
- **`POST /api/v1/media/bundle`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Body:** `multipart/form-data`, file field `file` (`.zip`, up to 500MB).
  - **Query Parameters:** `ingest_unmatched=true|false` (default `true` for gallery uploads: unmatched media files are imported as new assets).
  - **Decompression Safety:** Spools stream to temporary file, strictly rejects zip-slip (`..`, absolute paths), limits uncompressed output to 2GB and entry count to 2,000.
  - **Response (`200 OK`):**
    ```json
    {
      "total_extracted": 129,
      "reconciled_stubs": 129,
      "newly_created": 0,
      "skipped_files": 0,
      "errors": []
    }
    ```
  - **Error Codes:** `400 invalid_archive`, `400 zip_slip_detected`, `400 decompression_limit_exceeded`, `413 upload_too_large`, `415 unsupported_archive_format`.

### 5.3 Media Asset Detail & Metadata
- **`GET /api/v1/media/{id}`**
  - **Auth:** Authenticated (`Viewer`, `Editor`, `Admin`) or Public (redacted for living persons).
  - **Response (`200 OK`):** `MediaAssetSummaryDto`:
    ```json
    {
      "id": "00000000-0000-0000-0000-000000000101",
      "gid": "M0001",
      "title": "Mikhail Levshits Archival Portrait (1955)",
      "mime_type": "image/webp",
      "file_url": "/api/v1/media/00000000-0000-0000-0000-000000000101/file",
      "width": 1200,
      "height": 900,
      "date_taken": "1955",
      "is_private": false,
      "only_living_persons": false,
      "face_tags": [
        {
          "id": "ft-01",
          "person_id": "00000000-0000-0000-0000-000000000001",
          "person_name": "Mikhail Levshits",
          "is_living": false,
          "x": 0.25,
          "y": 0.30,
          "width": 0.20,
          "height": 0.25,
          "is_primary_avatar": true
        }
      ],
      "linked_person_ids": [
        "00000000-0000-0000-0000-000000000001",
        "c7a8b9d0-1234-5678-90ab-cdef12345678"
      ],
      "links": [
        {
          "id": "link-01",
          "media_id": "00000000-0000-0000-0000-000000000101",
          "entity_type": "individual",
          "entity_id": "00000000-0000-0000-0000-000000000001",
          "role": "primary_avatar"
        },
        {
          "id": "link-02",
          "media_id": "00000000-0000-0000-0000-000000000101",
          "entity_type": "individual",
          "entity_id": "c7a8b9d0-1234-5678-90ab-cdef12345678",
          "role": "depicted"
        }
      ],
      "version": 1
    }
    ```
- **`PATCH /api/v1/media/{id}`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Body (`application/json`):** `UpdateMediaRequest` (`title`, `description`, `date_taken`, `place_path`, `is_private`, `version`).
  - **Response (`200 OK`):** Updated `MediaAssetSummaryDto`.
  - **Error Codes:** `403 forbidden`, `404 not_found`, `409 conflict`.
- **`DELETE /api/v1/media/{id}`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Response (`204 No Content`)**.

### 5.4 Face Tags Management
- **`POST /api/v1/media/{id}/tags`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Body (`application/json`):**
    ```json
    {
      "person_id": "00000000-0000-0000-0000-000000000001",
      "x": 0.345,
      "y": 0.210,
      "width": 0.180,
      "height": 0.220,
      "is_primary_avatar": false
    }
    ```
  - **Response (`201 Created`):** `FaceTagDto`.
  - **Side Effects:** Automatically synchronizes `media.linked_person_ids` and creates corresponding `media_links` record with `role: "depicted"`. If `is_primary_avatar = true`, updates `person.avatar_media_id`, `avatar_crop`, and clears other primary tags for this individual.
- **`PATCH /api/v1/media/{id}/tags/{tag_id}`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Body (`application/json`):** Optional fields: `x`, `y`, `width`, `height`, `is_primary_avatar`.
  - **Response (`200 OK`):** Updated `FaceTagDto`.
- **`DELETE /api/v1/media/{id}/tags/{tag_id}`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Response (`204 No Content`)**.
  - **Side Effects:** If the deleted tag was `is_primary_avatar`, clears the person's avatar reference. Preserves `media_links` individual link unless explicitly unlinked.

### 5.5 Entity Media Links Management
- **`POST /api/v1/media/{id}/links`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Body (`application/json`):**
    ```json
    {
      "entity_type": "individual",
      "entity_id": "00000000-0000-0000-0000-000000000001",
      "role": "depicted"
    }
    ```
  - **Response (`201 Created`):** `MediaLinkDto`.
  - **Side Effects:** Automatically appends `entity_id` to `media.linked_person_ids` when `entity_type == "individual"`.
- **`DELETE /api/v1/media/{id}/links/{link_id}`**
  - **Auth:** Editor or Admin (`RequireEditor`).
  - **Response (`204 No Content`)**.

### 5.6 Binary & Cropped Image Streaming
- **`GET /api/v1/media/{id}/file`**: Streams original or responsive WebP variant.
- **`GET /api/v1/media/{id}/crop?x={x}&y={y}&w={w}&h={h}`**: Generates and streams square/rectangular WebP crop thumbnail on the fly.

---

## 6. Localization

Translation keys are maintained in English (`frontend/src/i18n/en.json`) and Russian (`frontend/src/i18n/ru.json`) following strict anti-fluff and zero-icon copywriting rules:
- `mediaArchive.peopleInMediaHeading`: `"People in this media ({count})"` / `"Люди на этом медиа ({count})"`.
- `mediaArchive.taggedPeopleCount`: `"Tagged ({count})"` / `"Отмечено ({count})"`.
- `mediaArchive.associatedPeopleCount`: `"Associated ({count})"` / `"Связано ({count})"`.
- `mediaArchive.profilePhotoBadge`: `"Profile photo"` / `"Фото профиля"`.
- `mediaArchive.associatedBadge`: `"Associated"` / `"Связан с фото"`.
- `mediaArchive.tagFaceAction`: `"Tag face"` / `"Отметить лицо"`.
- `mediaArchive.tagFaceForPersonPrompt`: `"Click a suggested face rectangle or draw a box for {name}"` / `"Выберите предложенное лицо или выделите рамку для {name}"`.
- `mediaArchive.linkPersonBtn`: `"Link person"` / `"Связать персону"`.
- `mediaArchive.unlinkPersonBtn`: `"Unlink"` / `"Отвязать"`.
- `mediaArchive.autoDetectFacesBtn`: `"Auto-detect faces"` / `"Автопоиск лиц"`.
- `mediaArchive.detectingFaces`: `"Detecting faces..."` / `"Поиск лиц..."`.
- `mediaArchive.facesDetectedToast`: `"Detected {count} faces"` / `"Обнаружено лиц: {count}"`.
- `mediaArchive.candidateFaceBadge`: `"Suggested face"` / `"Предложенное лицо"`.
- `mediaArchive.noDetectionSupport`: `"Face auto-detection unavailable in this browser; drag to draw boxes manually"` / `"Автопоиск лиц не поддерживается в этом браузере; выделите лицо вручную"`.

---

## 7. Edge Cases & Non-Goals

### 7.1 Edge Cases
- **No Detectable Faces:** On blurry, historical, or high-contrast scanned photos where the detector finds 0 faces, show a quiet toast: *"No faces detected. Draw a box manually."* Canvas remains in manual drawing mode with zero UI errors.
- **Overlapping Candidate and Existing Tags:** Detectors may return coordinates for an already tagged face. Bounding boxes with IoU $\ge 0.30$ against existing `face_tags` are automatically filtered out.
- **Multiple Close Faces:** When candidate rectangles are closely spaced, hover focus and clear z-indexes ensure only the clicked candidate rectangle is adopted.
- **Untagging a Person without Losing Context:** Deleting a face tag (`DELETE /api/v1/media/{id}/tags/{tag_id}`) deletes the coordinate bounding box, but the person remains listed under "PEOPLE IN THIS MEDIA" as an untagged associated person via their underlying `media_links` record.
- **Offline / Isolated LAN Deployment:** Browser `FaceDetector` API executes 100% on-device inside the browser without downloading heavy external weights or connecting to external servers.

### 7.2 Non-Goals
- **Phase 2 Non-Goal (Facial Identity Recognition):** Automatic identification of *who* a face belongs to based on facial embeddings is explicitly scheduled for Phase 3 "Power Tools". Phase 2 provides candidate rectangle discovery and 1-click coordinate adoption.
- **Automatic Auto-Cropping of Non-Face Objects:** The tool focuses strictly on human faces and documentary evidence, not arbitrary animal/object detection.

---

## 8. Definition of Done (DoD)

- [ ] **Dual People in Media List:** Lightbox Studio lists all individuals connected to the media asset, distinctly presenting both tagged faces (with face crop thumbnail) and untagged associated individuals.
- [ ] **Untagged Associated Person Badging:** Persons connected via `linked_person_ids` or profile avatar (`individuals.avatar_media_id == asset.id`) without face tags display with a "Profile photo" or "Associated" badge.
- [ ] **Quick `+ Tag Face` Assignment:** Clicking `+ Tag face` on an untagged associated person activates Tagging Mode with that person pre-selected, allowing 1-click bbox binding upon selecting or drawing a box.
- [ ] **Direct Individual Linking (`+ Link Person`):** Editors/Admins can associate relatives from the family tree to a media asset without requiring a face bounding box.
- [ ] **Automatic Face Detection Action:** In Tagging Mode, clicking "Auto-Detect Faces" runs client-side face detection and renders candidate suggestion rectangles on the canvas.
- [ ] **Deduplication against Existing Tags:** Candidate rectangles overlapping existing face tags (IoU $\ge 0.30$) are automatically filtered out.
- [ ] **1-Click Candidate Adoption:** Clicking a candidate suggestion rectangle immediately adopts its normalized coordinates as the draft box and opens the person picker (or binds the pre-selected person).
- [ ] **Manual Bounding Box Fallback:** Clicking and dragging on the canvas continues to allow drawing custom bounding boxes, and drag handles support moving and resizing boxes.
- [ ] **Primary Avatar Designation:** Designating any face tag as primary profile avatar instantly updates the individual's profile avatar and tree card avatar across the app.
- [ ] **Living Person Privacy Masking:** Living persons associated with media in public guest mode are stripped or masked according to RBAC tier rules.
- [ ] **Batch ZIP Archive Ingestion:** Uploading `.zip` bundles up to 500MB via `POST /api/v1/media/bundle` spools to disk, reconciles missing stubs, and imports unmatched media without memory spikes.
- [ ] **Localization Parity:** All new UI strings are localized in English and Russian with zero emojis or icon glyphs in translation values.
