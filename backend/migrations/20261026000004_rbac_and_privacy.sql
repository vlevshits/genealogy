-- Phase 1.3: Access Control, Master RBAC Matrix & Living-Person Privacy (`02-rbac-access-control.md`, `10-core-data-model.md §5`)

ALTER TABLE users ADD COLUMN IF NOT EXISTS deactivated_at TIMESTAMPTZ NULL;
CREATE INDEX IF NOT EXISTS idx_users_role_is_active ON users (role, is_active);

UPDATE system_metadata
SET value = '{"version": 4, "updated_at": "2026-10-26T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
