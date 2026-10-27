-- =============================================================================
-- Migration: 20261027000006_onboarding_and_home_person.sql
-- Empty-Tree Onboarding & Home Person (`docs/features/16-dashboard-home.md §2, §3, §5`)
-- =============================================================================

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS start_page VARCHAR(32) NOT NULL DEFAULT 'tree'
        CHECK (start_page IN ('tree', 'dashboard')),
    ADD COLUMN IF NOT EXISTS digest_opt_in BOOLEAN NOT NULL DEFAULT FALSE;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_users_person_id'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT fk_users_person_id
            FOREIGN KEY (person_id) REFERENCES individuals(id) ON DELETE SET NULL;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_users_home_person_id'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT fk_users_home_person_id
            FOREIGN KEY (home_person_id) REFERENCES individuals(id) ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_users_person_id
    ON users (person_id)
    WHERE person_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_users_home_person_id
    ON users (home_person_id)
    WHERE home_person_id IS NOT NULL;

INSERT INTO instance_settings (key, value)
VALUES ('default_home_person_id', 'null'::jsonb)
ON CONFLICT (key) DO NOTHING;

UPDATE system_metadata
SET value = '{"version": 6, "updated_at": "2026-10-27T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
