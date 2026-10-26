-- =============================================================================
-- Migration: 20261026000001_init_system.sql
-- Base extensions, sequences, and system metadata
-- =============================================================================

-- 1. Required PostgreSQL Extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "unaccent";

-- 2. Human-Readable GID Sequences (for I0001, F0001, etc.)
CREATE SEQUENCE IF NOT EXISTS individual_gid_seq START 1;
CREATE SEQUENCE IF NOT EXISTS family_gid_seq START 1;
CREATE SEQUENCE IF NOT EXISTS event_gid_seq START 1;
CREATE SEQUENCE IF NOT EXISTS place_gid_seq START 1;
CREATE SEQUENCE IF NOT EXISTS media_gid_seq START 1;

-- 3. System Metadata & Persistent Settings
CREATE TABLE IF NOT EXISTS system_metadata (
    key VARCHAR(64) PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed initial schema version
INSERT INTO system_metadata (key, value)
VALUES ('schema_version', '{"version": 1, "installed_at": "2026-10-26T00:00:00Z"}'::jsonb)
ON CONFLICT (key) DO NOTHING;
