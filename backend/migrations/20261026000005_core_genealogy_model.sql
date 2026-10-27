-- Phase 1.4: Core Genealogy Data Model (`10-core-data-model.md`, `07-person-family-management.md`, `12-places.md`)

-- 1. Monotonically increasing GID sequences (never reused after delete)
CREATE SEQUENCE IF NOT EXISTS seq_gid_individual START WITH 100 INCREMENT BY 1;
CREATE SEQUENCE IF NOT EXISTS seq_gid_family     START WITH 100 INCREMENT BY 1;
CREATE SEQUENCE IF NOT EXISTS seq_gid_event      START WITH 100 INCREMENT BY 1;
CREATE SEQUENCE IF NOT EXISTS seq_gid_place      START WITH 100 INCREMENT BY 1;
CREATE SEQUENCE IF NOT EXISTS seq_gid_media      START WITH 100 INCREMENT BY 1;

-- 2. Places & Alternate Names (`12-places.md §2`)
CREATE TABLE IF NOT EXISTS places (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gid         VARCHAR(20) UNIQUE NOT NULL,
    name        VARCHAR(200) NOT NULL,
    place_type  VARCHAR(16) NOT NULL DEFAULT 'other'
                CHECK (place_type IN ('country', 'region', 'district', 'city', 'village', 'parish', 'cemetery', 'address', 'other')),
    parent_id   UUID NULL REFERENCES places(id) ON DELETE RESTRICT,
    latitude    DOUBLE PRECISION NULL,
    longitude   DOUBLE PRECISION NULL,
    full_name   TEXT NOT NULL,
    version     INT NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by  UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by  UUID NULL REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS place_alt_names (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    place_id UUID NOT NULL REFERENCES places(id) ON DELETE CASCADE,
    name     VARCHAR(200) NOT NULL,
    lang     VARCHAR(8) NULL,
    note     VARCHAR(200) NULL
);

-- 3. Media Assets (`10-core-data-model.md §4.5`, `06-media-face-tagging.md`)
CREATE TABLE IF NOT EXISTS media_assets (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gid                 VARCHAR(20) UNIQUE NOT NULL,
    title               VARCHAR(255) NOT NULL DEFAULT '',
    caption             TEXT NULL,
    mime_type           VARCHAR(64) NOT NULL DEFAULT 'image/webp',
    storage_path        TEXT NOT NULL DEFAULT '',
    file_size_bytes     BIGINT NOT NULL DEFAULT 0,
    sha256              VARCHAR(64) NULL,
    date                JSONB NULL,
    date_sort           INT NULL,
    place_id            UUID NULL REFERENCES places(id) ON DELETE SET NULL,
    is_private          BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at          TIMESTAMPTZ NULL,
    version             INT NOT NULL DEFAULT 1,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by          UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by          UUID NULL REFERENCES users(id) ON DELETE SET NULL
);

-- 4. Individuals & Alternate Names (`10-core-data-model.md §4.1`)
CREATE TABLE IF NOT EXISTS individuals (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gid             VARCHAR(20) UNIQUE NOT NULL,
    given_name      VARCHAR(120) NULL,
    patronymic      VARCHAR(120) NULL,
    surname         VARCHAR(120) NULL,
    birth_surname   VARCHAR(120) NULL,
    nickname        VARCHAR(120) NULL,
    name_prefix     VARCHAR(40)  NULL,
    name_suffix     VARCHAR(40)  NULL,
    sex             VARCHAR(10)  NOT NULL DEFAULT 'unknown'
                    CHECK (sex IN ('male', 'female', 'other', 'unknown', 'M', 'F', 'U')),
    living_override BOOLEAN NULL,
    biography       TEXT NULL,
    avatar_media_id UUID NULL REFERENCES media_assets(id) ON DELETE SET NULL,
    avatar_crop     JSONB NULL,
    is_private      BOOLEAN NOT NULL DEFAULT FALSE,
    external_ids    JSONB NOT NULL DEFAULT '{}'::jsonb,
    birth_sort      INT NULL,
    death_sort      INT NULL,
    has_death_event BOOLEAN NOT NULL DEFAULT FALSE,
    search_text     TEXT NOT NULL DEFAULT '',
    version         INT NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by      UUID NULL REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS individual_alt_names (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    individual_id UUID NOT NULL REFERENCES individuals(id) ON DELETE CASCADE,
    name_type     VARCHAR(20) NOT NULL DEFAULT 'other'
                  CHECK (name_type IN ('married', 'alias', 'transliteration', 'religious', 'other')),
    given_name    VARCHAR(120) NULL,
    patronymic    VARCHAR(120) NULL,
    surname       VARCHAR(120) NULL,
    lang          VARCHAR(8) NULL
);

-- 5. Families & Children (`10-core-data-model.md §4.2`)
CREATE TABLE IF NOT EXISTS families (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gid               VARCHAR(20) UNIQUE NOT NULL,
    partner1_id       UUID NULL REFERENCES individuals(id) ON DELETE SET NULL,
    partner2_id       UUID NULL REFERENCES individuals(id) ON DELETE SET NULL,
    relationship_type VARCHAR(20) NOT NULL DEFAULT 'married'
                      CHECK (relationship_type IN ('married', 'civil_union', 'partners', 'unknown')),
    is_private        BOOLEAN NOT NULL DEFAULT FALSE,
    external_ids      JSONB NOT NULL DEFAULT '{}'::jsonb,
    version           INT NOT NULL DEFAULT 1,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by        UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by        UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    CHECK (partner1_id IS NULL OR partner2_id IS NULL OR partner1_id <> partner2_id)
);

CREATE TABLE IF NOT EXISTS family_children (
    family_id       UUID NOT NULL REFERENCES families(id) ON DELETE CASCADE,
    child_id        UUID NOT NULL REFERENCES individuals(id) ON DELETE CASCADE,
    sort_order      INT NOT NULL DEFAULT 0,
    rel_to_partner1 VARCHAR(10) NOT NULL DEFAULT 'birth'
                    CHECK (rel_to_partner1 IN ('birth', 'adopted', 'foster', 'step', 'unknown')),
    rel_to_partner2 VARCHAR(10) NOT NULL DEFAULT 'birth'
                    CHECK (rel_to_partner2 IN ('birth', 'adopted', 'foster', 'step', 'unknown')),
    PRIMARY KEY (family_id, child_id)
);

-- 6. Life Events (`10-core-data-model.md §4.3`)
CREATE TABLE IF NOT EXISTS life_events (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gid           VARCHAR(20) UNIQUE NOT NULL,
    individual_id UUID NULL REFERENCES individuals(id) ON DELETE CASCADE,
    family_id     UUID NULL REFERENCES families(id)    ON DELETE CASCADE,
    event_type    VARCHAR(32) NOT NULL,
    custom_type   VARCHAR(80) NULL,
    date          JSONB NULL,
    date_sort     INT NULL,
    place_id      UUID NULL REFERENCES places(id) ON DELETE RESTRICT,
    description   TEXT NULL,
    source_text   TEXT NULL,
    is_private    BOOLEAN NOT NULL DEFAULT FALSE,
    version       INT NOT NULL DEFAULT 1,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by    UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by    UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    CHECK ((individual_id IS NULL) <> (family_id IS NULL))
);

-- 7. Notes (`10-core-data-model.md §4.4`)
CREATE TABLE IF NOT EXISTS notes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(16) NOT NULL
                CHECK (entity_type IN ('individual', 'family', 'event', 'place', 'media')),
    entity_id   UUID NOT NULL,
    body        TEXT NOT NULL,
    is_private  BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order  INT NOT NULL DEFAULT 0,
    version     INT NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by  UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by  UUID NULL REFERENCES users(id) ON DELETE SET NULL
);

-- 8. Media Links & Face Tags (`10-core-data-model.md §2, §4.5`)
CREATE TABLE IF NOT EXISTS media_links (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id    UUID NOT NULL REFERENCES media_assets(id) ON DELETE CASCADE,
    entity_type VARCHAR(16) NOT NULL
                CHECK (entity_type IN ('individual', 'family', 'event', 'place')),
    entity_id   UUID NOT NULL,
    sort_order  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (media_id, entity_type, entity_id)
);

CREATE TABLE IF NOT EXISTS media_face_tags (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id      UUID NOT NULL REFERENCES media_assets(id) ON DELETE CASCADE,
    individual_id UUID NULL REFERENCES individuals(id) ON DELETE CASCADE,
    x             REAL NOT NULL,
    y             REAL NOT NULL,
    w             REAL NOT NULL,
    h             REAL NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 9. Change Log & Tree Snapshots (`10-core-data-model.md §4.6, §6.2`, `14-change-history-concurrency.md`)
CREATE TABLE IF NOT EXISTS change_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_id       UUID NOT NULL DEFAULT gen_random_uuid(),
    actor_id    UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    op          VARCHAR(16) NOT NULL CHECK (op IN ('create', 'update', 'delete', 'merge', 'restore')),
    entity_type VARCHAR(16) NOT NULL,
    entity_id   UUID NOT NULL,
    summary     TEXT NOT NULL DEFAULT '',
    before_json JSONB NULL,
    after_json  JSONB NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tree_snapshots (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label        VARCHAR(200) NOT NULL,
    trigger_kind VARCHAR(32) NOT NULL DEFAULT 'manual',
    payload_json JSONB NOT NULL,
    created_by   UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 10. Key Indexes (`10-core-data-model.md §4.7`, `12-places.md §2`)
CREATE INDEX IF NOT EXISTS idx_family_children_child      ON family_children (child_id);
CREATE INDEX IF NOT EXISTS idx_families_partner1          ON families (partner1_id);
CREATE INDEX IF NOT EXISTS idx_families_partner2          ON families (partner2_id);
CREATE INDEX IF NOT EXISTS idx_life_events_individual     ON life_events (individual_id, date_sort);
CREATE INDEX IF NOT EXISTS idx_life_events_family         ON life_events (family_id, date_sort);
CREATE INDEX IF NOT EXISTS idx_life_events_place          ON life_events (place_id);
CREATE INDEX IF NOT EXISTS idx_individuals_names          ON individuals (surname, given_name);
CREATE INDEX IF NOT EXISTS idx_individuals_birth_sort     ON individuals (birth_sort);
CREATE INDEX IF NOT EXISTS idx_individuals_search_trgm    ON individuals USING gin (search_text gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_places_name_trgm           ON places USING gin (name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_places_parent              ON places (parent_id);
CREATE INDEX IF NOT EXISTS idx_place_alt_names_place      ON place_alt_names (place_id);
CREATE INDEX IF NOT EXISTS idx_notes_entity               ON notes (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_media_links_entity         ON media_links (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_change_log_entity          ON change_log (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_change_log_tx              ON change_log (tx_id);

UPDATE system_metadata
SET value = '{"version": 5, "updated_at": "2026-10-26T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
