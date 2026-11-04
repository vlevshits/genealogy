# Deployment (Single Instance) — Implementation Memory

**Spec:** [docs/features/08-deployment-single-instance.md](../../features/08-deployment-single-instance.md) · **Status:** implemented

## 1. Summary
Single-instance, self-hosted deployment architecture packaged as a three-container Docker Compose stack (`frontend` Nginx reverse proxy, `backend` Axum server, and `postgres` database). Accompanied by a standalone administrative CLI (`genealogy-cli`) for headless database backups, restorations, and emergency admin access generation.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| Compose | `docker-compose.yml` | 3 services (`frontend`, `backend`, `postgres`), persistent volumes, internal network |
| Frontend Docker | `frontend/Dockerfile` | Multi-stage build (Node 22 builder → Nginx 1.27 alpine runner) |
| Frontend Nginx | `frontend/nginx.conf` | SPA fallback (`try_files $uri /index.html`), reverse proxy for `/api/v1/` and `/media/` |
| Backend Docker | `backend/Dockerfile` | Multi-stage build (Rust 1-slim builder with curl → Debian-slim runner) |
| Configuration | `backend/src/config.rs` | `AppConfig::from_env` reads `.env` variables |
| Admin CLI | `backend/src/bin/cli.rs` | `genealogy-cli backup`, `restore`, `create-admin-link` commands |
| Backup Service | `backend/src/services/backup.rs` | `create_backup_tarball`, `restore_backup_tarball`, archive verification |
| DB Init & Retry | `backend/src/services/db_init.rs` | Exponential retry loop, Postgres advisory locking (`ADR-008`) |
| Tests | `backend/tests/operations_test.rs` | CLI execution tests, backup and restore tarball roundtrip |

## 3. Key Flows
1. **Application Startup & Database Upgrade:**
   - On container start, `backend` executes `services::db_init::init_database`.
   - Acquires advisory lock (`pg_advisory_lock`), executes pending SQLx migrations sequentially, and releases the lock.
2. **Disaster Recovery Backup:**
   - Admin runs `docker compose exec backend ./genealogy-cli backup`.
   - Service streams `pg_dump` of all tables, packs the `/app/data/media` folder, and generates a single gzipped tarball in `/app/data/backups/`.
3. **Disaster Recovery Restore:**
   - Admin runs `docker compose exec backend ./genealogy-cli restore /app/data/backups/backup-2026-10-26.tar.gz`.
   - Restores database schema and rows, synchronizes media files, and runs migrations to bring data up to date.

## 4. Invariants & Business Rules
- **1 Instance = 1 Tree (`ADR-001`):** Complete data isolation; no multi-tenant leak potential.
- **Atomic Operations (`ADR-008`):** Upgrades and restores acquire PostgreSQL session-level advisory locks to prevent simultaneous writes.

## 5. Decisions
- **`ADR-002`: Offline Compilation:** Builds require zero network calls to a running database during `cargo build` or Docker image creation.

## 6. Gotchas & Known Limitations
- `GOT-005`: Reverse proxy header mismatches causing CSRF rejection; Nginx config must pass `Host` and `X-Forwarded-*` headers.

_Last curated: 2026-11-04 (memory bank bootstrap)_
