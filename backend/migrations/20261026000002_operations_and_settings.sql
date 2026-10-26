-- =============================================================================
-- Migration: 20261026000002_operations_and_settings.sql
-- Instance settings, background jobs queue, and emergency admin sign-in links
-- =============================================================================

CREATE TABLE IF NOT EXISTS instance_settings (
    key VARCHAR(64) PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NULL
);

INSERT INTO instance_settings (key, value)
VALUES
    ('tree_title', '"Family Tree Archive"'::jsonb),
    ('default_locale', '"en"'::jsonb),
    ('enable_public_access', 'false'::jsonb),
    ('living_person_age_threshold', '110'::jsonb)
ON CONFLICT (key) DO NOTHING;

CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind VARCHAR(32) NOT NULL,          -- backup | gedcom_import | gedcom_export | pdf | maintenance
    status VARCHAR(16) NOT NULL,        -- queued | running | done | failed | cancelled
    progress SMALLINT NOT NULL DEFAULT 0,
    step_label TEXT NULL,
    params JSONB NOT NULL DEFAULT '{}'::jsonb,
    result JSONB NULL,
    error TEXT NULL,
    created_by UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_jobs_status_created_at ON jobs (status, created_at);

CREATE TABLE IF NOT EXISTS emergency_admin_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_emergency_admin_links_token
    ON emergency_admin_links (token_hash)
    WHERE used_at IS NULL;

UPDATE system_metadata
SET value = '{"version": 2, "updated_at": "2026-10-26T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
