-- Migration 20261028000009_media_bundle_reconciliation.sql
-- Gramps Web Media Bundle Reconciliation & Large Archive Ingestion
-- Specs: `docs/features/03-gedcom-import-export.md §3.5, §3.6`, `docs/features/06-media-face-tagging.md §4.2`

ALTER TABLE media_assets
    ADD COLUMN IF NOT EXISTS is_missing BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS original_file_name VARCHAR(255) NULL,
    ADD COLUMN IF NOT EXISTS external_path TEXT NULL,
    ADD COLUMN IF NOT EXISTS width INT NULL,
    ADD COLUMN IF NOT EXISTS height INT NULL;

CREATE INDEX IF NOT EXISTS idx_media_assets_is_missing
    ON media_assets (is_missing);

CREATE INDEX IF NOT EXISTS idx_media_assets_lower_orig_file_name
    ON media_assets (LOWER(original_file_name));

-- Update system_metadata schema version to 9
UPDATE system_metadata
SET value = '{"version": 9, "updated_at": "2026-10-28T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
