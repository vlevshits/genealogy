# Feature Specification: Media Management & Face Tagging

## 1. Overview & Objective
Photographs and historical documents (scanned birth certificates, census records, gravestones, military records) are the emotional heart of family genealogy. This feature provides a modern media library with an interactive **face-tagging tool** (similar to Apple Photos, Google Photos, and MyHeritage), allowing users to tag individuals in group family portraits and assign tagged face crops directly as profile avatars.

---

## 2. Core Capabilities & Workflows

### 2.1 Media Asset Upload & Organization
- **Supported Formats:**
  - Images: JPEG, PNG, WebP, TIFF, AVIF.
  - Documents: PDF (scanned multi-page records).
  - Audio (optional): MP3/WAV oral history recordings.
- **Metadata Fields:** Title, Description, Date Taken (supports fuzzy genealogy dates, e.g. `abt 1935`), Place Taken, Associated Individuals, and Notes. A scan can also be attached directly to an event as its documentary evidence.
- **Storage Architecture:** Stored locally in a dedicated persistent Docker volume (e.g. `/data/media`) with hashed filenames; configurable S3-compatible backend for cloud deployments.
- **GEDCOM Stubs & External Reconciliation:** When importing a standalone `.ged` file whose media binaries were not bundled, stub entries are created with `is_missing = TRUE`, recording the original filename and external path (e.g. `/root/gramps/000085_*.jpg`). Uploading a matching photos folder or archive later reconciles the assets by base filename, clearing `is_missing` and linking thumbnails ([03 §3.5](03-gedcom-import-export.md#35-step-5-import-report--post-import-media-reconciliation)).

### 2.2 Interactive Face Tagging
1. In the photo detail viewer, an Editor or Admin clicks **"Tag Faces"**.
2. **Drawing Tag Boxes:** The user clicks and drags a rectangular bounding box around a person's face.
3. **Linking Individual:** A searchable dropdown auto-completes names from the family tree.
4. **Primary Avatar Assignment:** A checkbox offers *"Set as primary profile avatar for [Person]"*. When checked, the system crops and caches a high-res thumbnail used across tree cards and profile headers.
5. **Hover Interactions:** When viewing the photo, hovering over any face displays an elegant tooltip card with the person's name and life dates. Clicking navigates directly to their profile.

### 2.3 Media Privacy & Living Person Shield (Public Site Mode)
When the instance is exposed publicly to anonymous visitors:
- **Profile Avatars of Living Persons:** Replaced with generic placeholder silhouettes on all tree cards and profile endpoints.
- **Solely-Living Media Files:** Any photo or document where *all* linked/tagged persons are living is completely omitted from the public media gallery and returns `404 / 403` on direct asset URL requests.
- **Group Historical Photos:** Photos containing deceased ancestors alongside living descendants remain viewable for historical value, but all face-tag coordinates, names, and profiles of living persons are stripped from the payload delivered to anonymous visitors.

---

## 3. Data Model (PostgreSQL)

```sql
CREATE TABLE media_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_name VARCHAR(255) NOT NULL,
    original_file_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    width INT NULL,
    height INT NULL,
    title VARCHAR(255) NULL,
    description TEXT NULL,
    date_taken VARCHAR(100) NULL, -- Supports fuzzy dates like "Circa 1920"
    place_id UUID NULL REFERENCES places(id) ON DELETE SET NULL,
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    is_missing BOOLEAN NOT NULL DEFAULT FALSE, -- TRUE when imported from GEDCOM without binary asset
    external_path TEXT NULL,                   -- Preserved raw path from GEDCOM e.g. /root/gramps/000085_*.jpg
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## 4. Definition of Done (DoD)

- [ ] **Drag & Drop Upload:** Multiple photos can be uploaded simultaneously with real-time thumbnail generation.
- [ ] **Bounding Box Editor:** Users can draw, resize, move, and delete face boxes on photos.
- [ ] **Autocomplete Association:** Tagged boxes quickly search and link to existing tree members.
- [ ] **Avatar Synchronization:** Tagging a face and checking "Set as primary avatar" immediately updates the individual's profile and tree card avatar.
- [ ] **Hover Tooltips:** In viewer mode, hovering over a tagged face highlights the box and shows the person's name with a link to their profile.
- [ ] **Individual Media Gallery:** Opening any person's profile showcases all photos they are linked to or tagged in.
- [ ] **Responsive Image Pipeline:** High-resolution originals are preserved, while WebP thumbnails (small, medium, large) are generated for fast web delivery.
