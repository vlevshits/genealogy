-- Phase 2 ("Delight"): Data Quality, Duplicate Detection, Person Merge, Dashboard & Visualizations
-- Specs:
--   `docs/features/15-data-quality-duplicates.md`
--   `docs/features/16-dashboard-home.md`
--   `docs/features/04-tree-visualizations.md`
--   `docs/features/12-places.md`

-- 1. Dismissed Data Quality Warnings (`15-data-quality-duplicates.md §2.3, §6`)
CREATE TABLE IF NOT EXISTS dismissed_warnings (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_code    TEXT NOT NULL,
    entity_type  TEXT NOT NULL DEFAULT 'individual',
    entity_id    UUID NOT NULL,
    secondary_id UUID NULL,
    dismissed_by UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    dismissed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (rule_code, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_dismissed_warnings_entity
    ON dismissed_warnings (entity_id, rule_code);

-- 2. Marked Not-Duplicate Pairs (`15-data-quality-duplicates.md §4.3, §6`)
CREATE TABLE IF NOT EXISTS not_duplicate_pairs (
    person_a  UUID NOT NULL REFERENCES individuals(id) ON DELETE CASCADE,
    person_b  UUID NOT NULL REFERENCES individuals(id) ON DELETE CASCADE,
    marked_by UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    marked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (person_a, person_b),
    CHECK (person_a < person_b)
);

-- 3. Cached Duplicate Candidates (`15-data-quality-duplicates.md §4, §6`)
CREATE TABLE IF NOT EXISTS duplicate_candidates (
    person_a   UUID NOT NULL REFERENCES individuals(id) ON DELETE CASCADE,
    person_b   UUID NOT NULL REFERENCES individuals(id) ON DELETE CASCADE,
    score      INT NOT NULL,
    reasons    JSONB NOT NULL DEFAULT '[]'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (person_a, person_b),
    CHECK (person_a < person_b)
);

CREATE INDEX IF NOT EXISTS idx_duplicate_candidates_score_desc
    ON duplicate_candidates (score DESC);

-- 4. GID Alias Table for Merged Records (`10-core-data-model.md §3.1`, `15-data-quality-duplicates.md §5.2`)
CREATE TABLE IF NOT EXISTS gid_aliases (
    old_gid      VARCHAR(16) PRIMARY KEY,
    entity_type  VARCHAR(16) NOT NULL DEFAULT 'individual',
    surviving_id UUID NOT NULL,
    merged_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_gid_aliases_surviving
    ON gid_aliases (surviving_id);

-- 5. Default `users.start_page` to 'dashboard' (`16-dashboard-home.md §4`)
ALTER TABLE users
    ALTER COLUMN start_page SET DEFAULT 'dashboard';

UPDATE users
SET start_page = 'dashboard'
WHERE start_page = 'tree';

-- 6. Update `system_metadata` schema version to 8
UPDATE system_metadata
SET value = '{"version": 8, "updated_at": "2026-10-28T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
