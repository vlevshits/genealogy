# Feature Specification: Places & Maps

## 1. Overview & Objective
Places power the life map, timelines, search ("who was born in Tver?"), and GEDCOM `PLAC` export. Model: **a simple parent-child hierarchy, one primary name, optional alternate names, optional coordinates.** It must handle the reality of ex-USSR / Eastern European research, where the same village was in a *Gubernia → Uyezd → Volost* in 1900 and an *Oblast → Raion* today.

---

## 2. Data Model

```sql
CREATE TABLE places (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gid        VARCHAR(20) UNIQUE NOT NULL,
    name       VARCHAR(200) NOT NULL,          -- "Тверь"
    place_type VARCHAR(16) NOT NULL DEFAULT 'other',
    parent_id  UUID NULL REFERENCES places(id) ON DELETE RESTRICT,
    latitude   DOUBLE PRECISION NULL,
    longitude  DOUBLE PRECISION NULL,
    full_name  TEXT NOT NULL                   -- cached "Тверь, Тверская губерния, Российская империя"
);

CREATE TABLE place_alt_names (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    place_id UUID NOT NULL REFERENCES places(id) ON DELETE CASCADE,
    name     VARCHAR(200) NOT NULL,            -- "Калинин", "Tver", "Twer"
    lang     VARCHAR(8) NULL,
    note     VARCHAR(200) NULL                 -- free text: "1931–1990 Soviet name"
);
CREATE INDEX places_name_trgm ON places USING gin (name gin_trgm_ops);
```

- `full_name` is recomputed for the place and all its descendants when a name or parent changes.
- **Historical vs. modern jurisdiction:** the hierarchy should reflect the jurisdiction **at the time of the records** (recommended and default for imports). Modern names go into `place_alt_names` so search still finds them.
- Cycle protection: a place cannot be its own ancestor.

---

## 3. User Experience

### 3.1 Place Picker (used in every event form)
- Single combobox searching `name`, `full_name`, and alternate names (trigram + transliteration, see [13](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/13-search-people-directory.md)).
- Results show the full hierarchy line to disambiguate (*"Покровка — Новоузенский уезд, Самарская губ."*).
- *"+ Create 'Покровка'"* inline: asks only for **type** and **parent** (parent is itself a place picker). Typing `Покровка, Самарская губерния` auto-splits and matches/creates the chain.

### 3.2 Places List (`/places`)
- Collapsible tree view by hierarchy **and** a flat searchable table toggle.
- Columns: Name, Type, Parent, Events count, Has coordinates (✓/—).
- Filter *"Missing coordinates"* to help fill in the map.
- Actions: Edit, **Merge into…** (re-points all events, children places, and media; standard fix for GEDCOM imports which produce many duplicate place strings), Delete (only when unreferenced).

### 3.3 Place Detail (`/places/:id`)
- Header: name, type, full hierarchy breadcrumbs, alternate names.
- Small map with pin (if coordinates) + **"Pick on map"** to set coordinates by clicking.
- **People & events here:** chronological list of events at this place and its sub-places (*"Born here: 14 people · Died here: 6"*).
- Notes and linked media (e.g. photo of the church, village map).

### 3.4 Maps (Life Map & Place Map)
- Library: **MapLibre GL** (or Leaflet) with tiles from `MAP_TILE_URL`.
- Used on: person profile life map ([03 page](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/pages/03-person-profile-page.md)), place detail, and an optional **Family Map** (`/places?view=map`) showing all geocoded places with clustered pins and event counts.

---

## 4. Geocoding & Self-Hosting Privacy
Self-hosted users may not want family data leaving the server.

| Setting | Default | Behavior |
| :--- | :--- | :--- |
| `MAP_TILE_URL` | `https://tile.openstreetmap.org/{z}/{x}/{y}.png` | Tile source. Set to a self-hosted tile server, or **empty to disable all maps** (UI hides map panes gracefully). |
| `MAP_ATTRIBUTION` | `© OpenStreetMap contributors` | Shown on map. |
| `GEOCODING_ENABLED` | `false` | When `true`, the place form offers *"Find coordinates"* via Nominatim (`GEOCODING_URL`). Only the place name string is sent; requested **explicitly per click**, never in bulk/background. |
| `GEOCODING_URL` | `https://nominatim.openstreetmap.org` | Can point to a self-hosted Nominatim. |

Manual "Pick on map" always works without geocoding.

---

## 5. GEDCOM Mapping
- **Import:** `PLAC Тверь, Тверская губерния, Россия` is split on commas (right-to-left = top-down) into a deduplicated hierarchy. GEDCOM 7 `PLAC.FORM` / 5.5.1 `HEAD.PLAC.FORM` used to guess `place_type`. `MAP.LATI/LONG` → coordinates. Identical chains are merged automatically.
- **Export:** `PLAC` = `full_name`; coordinates as `MAP`; alt names as `PLAC.TRAN` (GEDCOM 7) or dropped (5.5.1).

---

## 6. API Endpoints
| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/v1/places?q=&parent_id=&missing_coords=` | Search / list / browse children. |
| `POST` | `/api/v1/places` | Create (`{ name, place_type, parent_id }` or `{ path: "A, B, C" }`). |
| `GET/PATCH/DELETE` | `/api/v1/places/:id` | Detail incl. events here; update; delete (409 if referenced). |
| `POST` | `/api/v1/places/:id/merge` | Merge `{ other_id }` into this place. |
| `GET` | `/api/v1/places/geocode?q=` | Proxy to Nominatim (only if `GEOCODING_ENABLED`). |

---

## 7. Definition of Done (DoD)

- [ ] **Inline Creation:** Typing a comma-separated path in the place picker creates/matches the full hierarchy.
- [ ] **Disambiguation:** Picker results always show the parent chain.
- [ ] **Alt-Name Search:** Searching "Калинин" or "Tver" finds "Тверь".
- [ ] **Merge Places:** Merging re-points all events, sub-places, and media; no orphans remain.
- [ ] **Import Dedup:** Importing a GEDCOM with 500 repeated `PLAC` strings produces one record per unique place.
- [ ] **Maps Optional:** With `MAP_TILE_URL` empty, the app has no map panes and makes zero external requests.
- [ ] **Explicit Geocoding:** No place data is sent externally unless the user clicks "Find coordinates" with geocoding enabled.
