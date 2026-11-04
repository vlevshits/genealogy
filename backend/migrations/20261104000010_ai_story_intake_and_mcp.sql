-- 20261104000010_ai_story_intake_and_mcp.sql
-- Milestone 3.2: AI Family Story Intake ("Tell Me About My Family") & Protected Model Context Protocol (MCP) Server
-- Specs: docs/features/19-ai-family-story-intake-and-mcp.md, docs/pages/10-ai-story-intake-page.md

CREATE TABLE IF NOT EXISTS story_intake_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL DEFAULT 'Untitled Family Story',
    raw_text TEXT NOT NULL DEFAULT '',
    audio_media_id UUID REFERENCES media_assets(id) ON DELETE SET NULL,
    speaker_person_id UUID REFERENCES individuals(id) ON DELETE SET NULL,
    is_new_speaker BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(32) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'reconciling', 'committed', 'discarded')),
    extracted_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    reconciliation_matches JSONB NOT NULL DEFAULT '[]'::jsonb,
    committed_tx_id VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_story_intake_drafts_created_by
    ON story_intake_drafts(created_by, status, updated_at DESC);

CREATE TABLE IF NOT EXISTS personal_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(120) NOT NULL,
    token_prefix VARCHAR(24) NOT NULL,
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    scopes TEXT[] NOT NULL DEFAULT ARRAY['read:tree']::TEXT[],
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_personal_access_tokens_user_id
    ON personal_access_tokens(user_id)
    WHERE revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_personal_access_tokens_token_hash
    ON personal_access_tokens(token_hash)
    WHERE revoked_at IS NULL;

-- Update system_metadata schema version to 10
UPDATE system_metadata
SET value = '{"version": 10, "updated_at": "2026-11-04T00:00:00Z"}'::jsonb,
    updated_at = NOW()
WHERE key = 'schema_version';
