-- Phase 1.11: Change Log, Undo, Restore Deleted, Optimistic Concurrency & Tree Snapshots
-- Spec: `docs/features/14-change-history-concurrency.md`

-- 1. Enhance `change_log` with transaction labels, entity display labels, and actor display names
ALTER TABLE change_log
    ADD COLUMN IF NOT EXISTS tx_label VARCHAR(200) NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS entity_label VARCHAR(200) NULL,
    ADD COLUMN IF NOT EXISTS actor_name VARCHAR(200) NULL;

CREATE INDEX IF NOT EXISTS idx_change_log_created_desc
    ON change_log (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_change_log_entity_created_desc
    ON change_log (entity_type, entity_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_change_log_actor_created_desc
    ON change_log (actor_id, created_at DESC);

-- 2. Enhance `tree_snapshots` with reason, file_path, expiration, and record_counts
ALTER TABLE tree_snapshots
    ADD COLUMN IF NOT EXISTS reason VARCHAR(32) NOT NULL DEFAULT 'manual',
    ADD COLUMN IF NOT EXISTS file_path TEXT NULL,
    ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '14 days'),
    ADD COLUMN IF NOT EXISTS record_counts JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX IF NOT EXISTS idx_tree_snapshots_created_desc
    ON tree_snapshots (created_at DESC);

-- 3. Update `system_metadata` schema version to 7
UPDATE system_metadata
SET value = '{"version": 7, "updated_at": "2026-10-27T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
