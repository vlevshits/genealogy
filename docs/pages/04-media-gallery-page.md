# Page Specification: Media Archive & Face Tagging (`/media`)

## 1. Overview & Objective
The Media Archive is a visual gallery celebrating family history through photographs, scanned metrical records, letters, certificates, and oral histories. It combines a **fluid masonry media wall** with an interactive **Face Tagging Lightbox Studio**.

> [!IMPORTANT] Authenticated-Only Access & Role Segregation:
> - **Authenticated-Only Boundary:** The `/media` route is strictly restricted to authenticated family members (`Viewer`, `Editor`, `Admin`). Unauthenticated visitors (in both default private mode and public tree mode) cannot access `/media` and are redirected to `/login`. Direct asset requests for living person media by unauthenticated guests return `404 Not Found`.
> - **Authenticated Viewers (`Viewer`):** Trusted family circle members can browse all family photos, view face tags, and download individual assets. All write controls (`+ Upload Media`, face box drawing, tag assignment, metadata edits) are hidden.
> - **Editors & Admins (`Editor`, `Admin`):** Full media management authority: batch uploading, rectangular face tagging, primary avatar designation, and caption editing.

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
| LIGHTBOX VIEWER MODAL (Opens on photo click):                                      |
| +-----------------------------------------------+--------------------------------+ |
| |                                               | METADATA & TAGS PANEL          | |
| |               [ PHOTO CANVAS ]                |                                | |
| |                                               | Title: 1941 Baltic Front Photo | |
| |          +-------------+                      | Date: Circa Sep 1941           | |
| |          | [Face Box]  |                      | Place: Near Leningrad          | |
| |          | Aleksandr P.|                      |                                | |
| |          +-------------+                      | TAGGED PEOPLE (3):             | |
| |                                               | 👤 Aleksandr Petrov [Avatar ✓] | |
| | [Tool: Draw Face Box] [Zoom] [Download]       | 👤 Mikhail Volkov              | |
| |                                               | ➕ Tag Another Face            | |
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
  - Supports drag-and-drop batch upload of up to 50 photos/documents with progress indicators.

### 3.2 Responsive Media Wall
- Masonry / Virtualized Grid displaying thumbnails with lazy loading.
- Hover card overlays show:
  - Year / Date taken
  - Number of tagged individuals (with mini-avatars)
  - Asset title

### 3.3 Face Tagging Lightbox Studio
When a photo is clicked, a full-screen dual-pane modal opens:
- **Left Canvas Pane:**
  - High-res image display with pan/zoom.
  - **Face Tagging Mode Toggle (`Tool: Draw Face Box`):**
    - Rendered only for **Editors and Admins**. When enabled, user can click and drag rectangular bounding boxes.
    - For Viewers: Canvas displays existing bounding boxes on hover without drawing or editing capability.
- **Right Metadata & Tag Management Pane:**
  - Title, description, date taken (supporting approximate dates like *"Circa 1941"*), and location. Editable by Editors/Admins; read-only for Viewers.
  - List of currently tagged individuals with click-to-profile links.
  - Quick action: *"Use as primary profile avatar"* (available to Editors/Admins only).
  - Linked events (e.g. "Birth record of Иван Петров") and notes.

---

## 4. Definition of Done (DoD)

- [ ] **Authenticated-Only Route:** Unauthenticated visitors attempting to access `/media` are redirected to `/login`; media assets tagged to living persons return 404 for unauthenticated requests.
- [ ] **Viewer Read-Only Experience:** Authenticated Viewers can search, filter, view full-resolution photos, examine face tags, and download individual photos, but cannot upload media or alter face coordinates.
- [ ] **Editor & Admin Media Management:** Editors and Admins can batch-upload media, draw/resize/delete rectangular face boxes, and designate primary profile avatars.
- [ ] **Fluid Masonry Layout:** Images of varying aspect ratios arrange seamlessly without awkward whitespace.
- [ ] **Interactive Face Bounding Boxes:** Drawing, resizing, and deleting face boxes updates coordinates in real-time.
- [ ] **Instant Avatar Crop:** Designating a tagged face as primary avatar crops and refreshes avatar images across tree cards immediately.
