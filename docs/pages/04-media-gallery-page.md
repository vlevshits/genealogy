# Page Specification: Media Archive & Face Tagging (`/media`)

## 1. Overview & Objective
The Media Archive is a visual gallery celebrating family history through photographs, scanned metrical records, letters, certificates, and oral histories. It combines a **fluid masonry media wall** with an interactive **Face Tagging Lightbox Studio** featuring client-side **automatic face detection** and an **associated people panel** (see [06-media-face-tagging.md](../features/06-media-face-tagging.md)).

> [!IMPORTANT] Authenticated-Only Access & Role Segregation:
> - **Authenticated-Only Boundary:** The `/media` route is strictly restricted to authenticated family members (`Viewer`, `Editor`, `Admin`), per [02-rbac-access-control.md](../features/02-rbac-access-control.md). Unauthenticated visitors (in both default private mode and public tree mode) cannot access `/media` and are redirected to `/login`. Direct asset requests for living person media by unauthenticated guests return `404 Not Found`.
> - **Authenticated Viewers (`Viewer`):** Trusted family circle members can browse all family photos, view face tags and associated people, inspect hover tooltips, and download individual assets. All write controls (`+ Upload Media`, auto-detect faces, face box drawing, tag assignment, metadata edits, person linking) are hidden.
> - **Editors & Admins (`Editor`, `Admin`):** Full media management authority: batch uploading, automatic face detection, rectangular face tagging, primary avatar designation, person linking, and caption editing.

---

## 2. Page Wireframe & Layout

```
+------------------------------------------------------------------------------------+
| [Global Header: Logo | Navigation | Omnibox Search | RU/EN | Theme | Profile]      |
+------------------------------------------------------------------------------------+
| FILTER & SEARCH BAR:                                                               |
| [🔍 Search photos...] [Person: All ▼] [Decade: 1940s ▼] [Type: Photos ▼]  [+ Upload]
+------------------------------------------------------------------------------------+
| MEDIA WALL (Responsive Masonry Grid):                                              |
|                                                                                    |
| +-------------------+  +-------------------+  +-------------------+                |
| | [Photo 1941]      |  | [Scanned Record]  |  | [Wedding 1965]    |                |
| | 3 people tagged   |  | Metrical Book     |  | 5 people tagged   |                |
| +-------------------+  +-------------------+  +-------------------+                |
|                                                                                    |
| +-------------------+  +-------------------+  +-------------------+                |
| | [Portrait 1918]   |  | [Passport 1930]   |  | [Family Gathering]|                |
| +-------------------+  +-------------------+  +-------------------+                |
+------------------------------------------------------------------------------------+
| LIGHTBOX STUDIO MODAL (Opens on photo click):                                      |
| +-----------------------------------------------+--------------------------------+ |
| | TOP STUDIO BAR: [Title: 1941 Baltic Front Photo]                 [<] [>] [✕ Close] |
| +-----------------------------------------------+--------------------------------+ |
| | LEFT CANVAS PANE (7 cols)                     | RIGHT METADATA & PEOPLE (5 cols| |
| |                                               |                                | |
| | [BANNER: Tagging Mode Active]                 | [GID: M0012] [MIME: image/jpeg]| |
| |                                               | Title: [1941 Baltic Front] [💾]| |
| |          +-------------+                      | Date:  [Circa Sep 1941]        | |
| |          | [Face Box]  |                      | Place: [Near Leningrad]        | |
| |          | Aleksandr P.|                      |                                | |
| |          +-------------+                      | PEOPLE IN THIS MEDIA (4):      | |
| |                                               |                                | |
| |    .- - - - - - - - - - - .                   | -- TAGGED (2) ---------------- | |
| |    ' [Suggested Face Box] '                   | 👤 Aleksandr Petrov [Avatar ★] | |
| |    ' (Click to tag face)  '                   | 👤 Mikhail Volkov              | |
| |    '- - - - - - - - - - - '                   |                                | |
| |                                               | -- UNTAGGED ASSOCIATED (2) --- | |
| |                                               | 👤 Elena Petrova               | |
| |                                               |    [Profile Photo] [+ Tag Face]| |
| |                                               | 👤 Ivan Petrov                 | |
| |                                               |    [Associated]    [+ Tag Face]| |
| |                                               |                                | |
| | [Tool: Tag Faces] [Auto-Detect] [Zoom] [⬇]    | [+ Link Person] [+ Tag Face]   | |
| +-----------------------------------------------+--------------------------------+ |
+------------------------------------------------------------------------------------+
```

---

## 3. Structural Sections

### 3.1 Media Filter & Action Bar
- **Search Input:** Filters by title, description, or handwritten notes.
- **Filter Chips:**
  - **Person Filter:** Filter photos containing a specific individual.
  - **Decade Timeline:** Quick jumps (`1880s`, `1900s`, `1920s`, `1940s`, etc.).
  - **Asset Type:** Photos, Scanned Documents, Audio, Gravestones.
  - **Untagged Filter:** Shows photos where faces have not yet been identified.
- **Upload Action (`+ Upload Media`):**
  - Visible and active exclusively for **Editors and Admins** (hidden for Viewers).
  - Clicking opens the **Media Upload Dialog** and also accepts drag-and-drop directly onto the gallery view.
  - **Supported Ingestion Modes:**
    - **Individual Files:** Up to 50 photos/documents simultaneously (JPEG, PNG, WebP, AVIF, TIFF, PDF).
    - **Batch ZIP Media Archive:** A single `.zip` archive bundle up to 500MB (such as a Gramps Web media export).
  - **Upload Dialog Dropzone Copy:** *"Drag photos or a media ZIP archive here, or browse files (up to 500MB)"*.
  - **Multi-Stage Progress Display:**
    1. `Stage 1: Uploading...` [===>    ] 45% (23 MB / 51.5 MB)
    2. `Stage 2: Decompressing & matching...` (e.g. `Checking 129 files against missing stubs`)
    3. `Stage 3: Generating thumbnails...` (e.g. `Optimizing 129 WebP variants`)
  - **Batch Ingestion Result Modal:**
    - Detailed summary: `129 files processed` (e.g., `129 missing stubs reconciled`, `0 new photos added`, `0 skipped`).
    - Masonry gallery automatically re-queries and displays newly available photo cards immediately.

### 3.2 Responsive Media Wall
- Masonry / Virtualized Grid displaying thumbnails with lazy loading.
- Hover card overlays show:
  - Year / Date taken
  - Number of tagged individuals (with mini-avatars)
  - Asset title

### 3.3 Face Tagging Lightbox Studio (`MediaLightboxStudio`)
When a photo is clicked in the masonry wall, a full-screen dual-pane modal opens (`MediaLightboxStudio.tsx`). The workspace divides into a responsive 7-col / 5-col split on desktop:

#### 3.3.1 Left Canvas Pane (`FaceTagCanvas.tsx`, 7 columns on desktop)
- **High-Res Viewport:** Image display with pan/zoom controls (50% to 250% with 1-click reset) and download action.
- **Tag Faces Mode (`Tool: Tag Faces`):**
  - Toggle button in bottom controls bar. Rendered only for Editors and Admins.
  - Activating renders the top tagging banner: *"Face Tagging Mode: Click and drag to create a face box, or click a detected face candidate."*
- **Automatic Face Detection (`Auto-Detect Faces`):**
  - Toolbar button with `Sparkles` / `Scan` icon.
  - When triggered, analyzes the current photo using the browser's `window.FaceDetector` API (or client-side fallback).
  - Normalizes detected coordinates to `[0.0, 1.0]`.
  - Filters out candidate boxes that overlap existing `face_tags` (IoU $\ge 0.30$ or centroid containment).
  - Displays remaining candidate boxes as **Candidate Suggestion Rectangles**:
    - Stylized with pulsing dashed border (`border-2 border-dashed border-sky-400 bg-sky-400/10 hover:bg-sky-400/25`).
    - Pill label on hover: *"Suggested Face (Click to tag)"*.
- **1-Click Candidate Adoption:**
  - Clicking any candidate suggestion rectangle instantly converts it to the active `draftBox`.
  - If an untagged person was pre-selected (via `+ Tag Face` in the panel), the system immediately confirms and creates the tag.
  - Otherwise, opens the `PersonFacePicker` dropdown anchored directly beneath the box, pre-focusing the search input.
  - Selecting a relative and pressing Enter or clicking Save persists the tag via `POST /api/v1/media/:id/tags` and removes the candidate rectangle.
- **Manual Drawing & Adjustments (Fallback):**
  - Clicking and dragging on unboxed canvas areas draws a custom rectangular bounding box.
  - In tagging mode, existing tagged boxes expose `Move` and `Resize` corner handles for manual fine-tuning.
- **Viewer Hover Tooltips:**
  - For all roles (when tagging mode is off), hovering over any tagged face highlights the box and opens a frosted tooltip card showing the person's name, vital dates (`b. 1912 · † 1989`), primary avatar indicator, and a direct profile link.

**Canvas Behavioral States:**
- *Idle View State:* Tagging mode off. Hovering over tagged boxes reveals viewer tooltip card; pan/zoom active.
- *Tagging Mode Active:* Crosshair cursor; existing tagged boxes display corner drag and resize handles; top banner active.
- *Face Detection Loading:* "Auto-Detect Faces" button displays spinner with *"Detecting faces..."* label; canvas temporarily locks cursor.
- *Candidate Rectangles Rendered:* Pulsing sky-blue dashed boxes overlay untagged faces. Hovering expands pill label to *"Click to tag face"*.
- *Pre-Selected Target Person State:* Active when `+ Tag Face` is triggered from the right panel. Top banner displays: *"Click a suggested face rectangle or draw a box for {name}"* with a "Cancel" action. Clicking any candidate box or completing a manual box immediately assigns the coordinates to `{name}` without search.
- *Unsupported Browser Fallback State:* If `window.FaceDetector` is undefined and heuristics yield zero candidates, displays toast: *"Face auto-detection unavailable in this browser; drag to draw boxes manually."* Canvas remains fully functional in manual drag mode.

#### 3.3.2 Right Metadata & People Panel (`MediaMetadataPanel.tsx`, 5 columns on desktop)
- **Asset Metadata Form:**
  - Displays GID, MIME type, title, date taken, place taken, description, and private toggle.
  - Editable with dirty tracking and Save button for Editors/Admins; read-only display for Viewers.
- **"PEOPLE IN THIS MEDIA" Unified Section:**
  - Displays total count badge: `PEOPLE IN THIS MEDIA ({count})`.
  - Partitioned into two visual groups:
    1. **Tagged Faces:**
       - Face crop thumbnail preview (using square crop `/api/v1/media/{id}/crop`).
       - Full name with clickable link to `/persons/:id`.
       - Vital dates badge.
       - `Primary Avatar` badge (if designated as primary profile picture).
       - Hovering on row highlights the face box on the canvas.
       - Actions (Editors/Admins): *"Set as primary profile avatar"* toggle, *"Remove tag"* (`Trash2` icon) which deletes the bounding box while preserving the general person link.
    2. **Untagged Associated Individuals:**
       - Profile silhouette/avatar, full name with clickable link to `/persons/:id`, and vital dates.
       - Context Badge:
         - `Profile Photo` badge: If this media asset is set as the person's profile picture (`avatar_media_id == asset.id` or `media_links.role == 'primary_avatar'`), but no specific face coordinates have been tagged yet.
         - `Associated` badge: If the person is associated via `media_links` (`entity_type = 'individual'`, `role = 'depicted'`).
       - Quick Action: **`+ Tag Face`** button:
         - Activates Tagging Mode and pre-selects this individual as the target.
         - Highlights their panel row with an active brand ring (`ring-2 ring-brand`).
         - Prompts user on the canvas to click any candidate suggestion rectangle or draw a manual box for them.
         - Adopting a candidate rectangle or finishing a manual box immediately links it to this person without searching!
       - Unlink Action: Allows removing the person association if they are not in the photo.
  - Action Header:
    - `+ Link Person`: Opens an autocomplete picker to associate a family tree member to the media asset without drawing a face box.
    - `+ Tag Face`: Activates face tagging mode.

**Panel Behavioral States:**
- *Both Tagged & Untagged Present:* Grouped under distinct subheadings (`Tagged ({count})` and `Associated ({count})`).
- *Only Tagged Present:* Shows tagged rows with crop previews; untagged section displays empty hint with `+ Link Person` action.
- *Only Untagged Present:* Shows untagged associated rows with badges and prominent `+ Tag Face` quick-action buttons.
- *Completely Empty State:* Displays *"No people associated with this media yet"* with `+ Link Person` and `+ Tag Face` quick actions.
- *Target Person Pre-Selected:* The targeted untagged row displays an active highlight ring (`ring-2 ring-brand bg-brand/5`) and a "Cancel selection" button until a box is assigned.

---

## 4. Definition of Done (DoD)

- [ ] **Authenticated-Only Route:** Unauthenticated visitors attempting to access `/media` are redirected to `/login`; media assets tagged solely to living persons return 404 for unauthenticated requests.
- [ ] **Viewer Read-Only Experience:** Authenticated Viewers can search, filter, view full-resolution photos, examine face tags and associated people, and download individual photos, with all edit and detection controls hidden.
- [ ] **Unified People in Media Panel:** The lightbox right panel displays all individuals linked to the asset, clearly distinguishing tagged faces from untagged associated persons.
- [ ] **Untagged Context Badges:** Untagged individuals display an "Associated" or "Profile photo" badge indicating why they are linked to the asset.
- [ ] **Quick `+ Tag Face` Trigger:** Clicking `+ Tag face` on an untagged person enters tagging mode with that person pre-selected, allowing 1-click candidate box assignment.
- [ ] **Direct Individual Linking (`+ Link Person`):** Editors and Admins can associate tree relatives to a media file without drawing a face bounding box.
- [ ] **Automatic Face Detection Action:** In Tagging Mode, clicking "Auto-Detect Faces" executes client-side face detection and highlights untagged candidate face rectangles.
- [ ] **Existing Tag Deduplication:** Detected face candidates that overlap existing face tags (IoU $\ge 0.30$) are automatically excluded from suggestion overlays.
- [ ] **1-Click Candidate Adoption:** Clicking a candidate suggestion rectangle immediately adopts its coordinates as the active draft box and opens the person picker (or commits the pre-selected person).
- [ ] **Manual Bounding Box Fallback:** Manual click-and-drag box drawing, coordinate dragging, and corner resizing remain fully functional alongside auto-detection.
- [ ] **Instant Avatar Crop Sync:** Designating a tagged face as primary avatar refreshes the individual's avatar across tree cards and profile headers immediately.
- [ ] **Batch ZIP Ingestion & Reconciliation:** Uploading a `.zip` file up to 500MB via the upload modal or gallery drag-and-drop displays live 3-stage progress and updates the gallery upon completion.
- [ ] **Localization Quality:** All new UI copy is localized in English and Russian with zero emojis or icon glyphs in `en.json` / `ru.json`.
