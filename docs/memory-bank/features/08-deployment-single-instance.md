# Deployment (Single Instance) — Implementation Memory

**Spec:** [docs/features/08-deployment-single-instance.md](../../features/08-deployment-single-instance.md) · **Status:** implemented

## 1. Summary
Single-instance, self-hosted deployment architecture supporting two primary installation models:
1. **LXC / Bare-Metal (Recommended Production):** Direct web listening on port 80 (Axum native serving of SPA static files via `FRONTEND_STATIC_PATH` and `/api/v1` REST endpoints) with ZERO Nginx / proxy layer on Debian 11/12 and Ubuntu 22.04/24.04 via single-command installation and atomic update script (`scripts/install.sh`) consuming precompiled GitHub Releases archives (`x86_64` and `aarch64`). Systemd unit configures capability `CAP_NET_BIND_SERVICE` allowing non-root `genealogy` user to bind directly to port 80.
2. **Docker Compose:** Isolated three-container stack (`frontend` Nginx reverse proxy, `backend` Axum server, and `postgres` database).

Accompanied by an on-demand cross-compilation release pipeline (`.github/workflows/release.yml`), release automation skill (`.agents/skills/genealogy-release/SKILL.md`), and standalone administrative CLI (`genealogy-cli`) for headless database backups, restorations, and emergency admin access generation.

## 2. Code Map
| Layer | Location | Key symbols / notes |
| :--- | :--- | :--- |
| LXC Installer / Updater | `scripts/install.sh` | Single-command root script; handles OS detection (`x86_64`, `aarch64`), automated PostgreSQL installation/configuration (role `genealogy`, db `genealogy`, `pg_hba.conf` local auth, connection test), systemd service setup with `CAP_NET_BIND_SERVICE`, atomic updates, and direct port 80 healthcheck verification (zero Nginx) |
| Systemd Service | `deploy/lxc/genealogy.service` | Hardened unit file (`User=genealogy`, `CapabilityBoundingSet=CAP_NET_BIND_SERVICE`, `AmbientCapabilities=CAP_NET_BIND_SERVICE` for direct port 80 binding, `ProtectSystem=full`, `ProtectHome=true`, `NoNewPrivileges=true`, `Restart=always`) |
| LXC Environment | `deploy/lxc/genealogy.env.example` | Production environment template for `/etc/genealogy/genealogy.env` (`PORT=80`, `FRONTEND_STATIC_PATH=/var/www/genealogy`) |
| Release Workflow | `.github/workflows/release.yml` | GitHub Actions pipeline triggered on `v*.*.*` tags or manual `workflow_dispatch`; builds `x86_64` and `aarch64` binaries, frontend SPA, tarballs (`genealogy-backend`, `genealogy-cli`, `frontend/`, `scripts/install.sh`, `deploy/lxc/genealogy.service`, `deploy/lxc/genealogy.env.example`), and `SHA256SUMS` (zero Nginx config) |
| Release Skill | `.agents/skills/genealogy-release/SKILL.md` | Operator skill for semantic version bumping, pre-flight test verification, and git tagging |
| Backend Static Serving | `backend/src/routes/mod.rs` | Native Axum SPA static file serving (`ServeDir` / `ServeFile` fallback to `index.html`) when `frontend_static_path` is configured; serves directly on port 80 in LXC |
| Configuration | `backend/src/config.rs` | `AppConfig::from_env` parses `.env` including `frontend_static_path` (`FRONTEND_STATIC_PATH`) |
| Compose | `docker-compose.yml` | 3 services (`frontend`, `backend`, `postgres`), persistent volumes, internal network |
| Frontend Docker | `frontend/Dockerfile` | Multi-stage build (Node 22 builder → Nginx 1.27 alpine runner) |
| Frontend Nginx | `frontend/nginx.conf` | SPA fallback (`try_files $uri /index.html`), reverse proxy for `/api/v1/` and `/media/` (Docker only) |
| Backend Docker | `backend/Dockerfile` | Multi-stage build (Rust 1-slim builder with curl → Debian-slim runner) |
| Admin CLI | `backend/src/bin/cli.rs` | `genealogy-cli backup`, `restore`, `create-admin-link` commands |
| Backup Service | `backend/src/services/backup.rs` | `create_backup_tarball`, `restore_backup_tarball`, archive verification |
| DB Init & Retry | `backend/src/services/db_init.rs` | Exponential retry loop, Postgres advisory locking (`ADR-008`) |
| Tests | `backend/tests/operations_test.rs` | CLI execution tests, backup and restore tarball roundtrip |

## 3. Key Flows

### 1. LXC / Bare-Metal Fresh Installation
- Operator runs `curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/install.sh | sudo bash`.
- Script checks root and OS architecture (`x86_64` or `aarch64`), fetches target release tag from GitHub, and downloads `genealogy-vX.Y.Z-linux-<arch>.tar.gz`.
- Provisions system user `genealogy`, directories (`/opt/genealogy`, `/var/lib/genealogy`, `/var/www/genealogy`, `/etc/genealogy`).
- Installs local PostgreSQL and configures role/database `genealogy` with generated credentials, configures `pg_hba.conf` local auth, and verifies database connectivity (`pg_isready` & `psql`).
- Generates `/etc/genealogy/genealogy.env` with random `APP_SECRET`, Postgres credentials, `PORT=80`, and `FRONTEND_STATIC_PATH=/var/www/genealogy` (permissions `0600`).
- Configures systemd unit `/etc/systemd/system/genealogy.service` with `CAP_NET_BIND_SERVICE`, enables and starts service.
- Axum directly listens on port 80 serving SPA static files (`/var/www/genealogy`) and `/api/v1` REST endpoints with zero Nginx or reverse proxy layer.
- Polls `http://127.0.0.1:80/api/v1/health` until healthy.

### 2. LXC In-Place Atomic Update
- Operator runs `sudo /opt/genealogy/scripts/install.sh` (or specifies `VERSION=vX.Y.Z`).
- Detects existing install at `/opt/genealogy/genealogy-backend` and runs in update mode.
- Downloads target release archive and unpacks to temporary directory.
- Stops `genealogy.service`, backs up binary to `genealogy-backend.old`, installs new binaries (`genealogy-backend`, `genealogy-cli`).
- Updates `/var/www/genealogy` via atomic directory swap (`/var/www/genealogy.old`).
- Preserves `/etc/genealogy/genealogy.env` and all media/database records in `/var/lib/genealogy`.
- Starts `genealogy.service`, which automatically executes pending database migrations under PostgreSQL advisory lock.
- Verifies health check at `http://127.0.0.1:80/api/v1/health` (or configured `PORT`).

### 3. Application Startup & Database Upgrade
- On startup (Docker or systemd), backend executes `services::db_init::init_database`.
- Connects to PostgreSQL, acquires session advisory lock (`pg_advisory_lock(94218731)`, `ADR-008`), executes pending forward SQLx migrations sequentially, and releases lock.

### 4. On-Demand Release Tagging & Packaging
- Releases are never published on every commit (`ADR-012`).
- Developer/operator uses the `genealogy-release` skill to verify tests (`cargo test`, `npm test`, `npm run test:i18n`, `npm run build`, `scripts/check-memory-bank.sh`), bumps version in `frontend/package.json` and `backend/Cargo.toml`, and creates annotated git tag `vX.Y.Z`.
- Tag push triggers `.github/workflows/release.yml`, which compiles the frontend SPA, builds Linux `x86_64` (native) and `aarch64` (via `cross`) binaries, packages unified `.tar.gz` archives with deployment templates (backend, cli, frontend dist, systemd unit, env template, install script; zero Nginx proxy layer), generates `SHA256SUMS`, and publishes the GitHub Release.

### 5. Disaster Recovery Backup & Restore
- **Backup:**
  - Bare-metal/LXC: `sudo genealogy-cli backup` (writes archive to `/var/lib/genealogy/backups/`).
  - Docker: `docker compose exec backend ./genealogy-cli backup`.
  - Streams `pg_dump` of tables, bundles media directory, and writes gzipped tarball.
- **Restore:**
  - Bare-metal/LXC: `sudo genealogy-cli restore /var/lib/genealogy/backups/backup-YYYY-MM-DD.tar.gz`.
  - Docker: `docker compose exec backend ./genealogy-cli restore /app/data/backups/backup-YYYY-MM-DD.tar.gz`.
  - Replaces schema/tables, synchronizes media files, and runs migrations to ensure compatibility.

## 4. Invariants & Business Rules
- **1 Instance = 1 Tree (`ADR-001`):** Complete data isolation; no multi-tenant leak potential.
- **Atomic Operations (`ADR-008`):** Upgrades and restores acquire PostgreSQL session-level advisory locks to prevent simultaneous writes.
- **On-Demand Release Cadence (`ADR-012`):** Releases are cut deliberately via version tags or manual workflow dispatch, never automatically on every commit to `main`.
- **Zero-Proxy LXC Architecture (`ADR-012`):** Native LXC binds directly to port 80 via Axum native static serving with `CAP_NET_BIND_SERVICE`; no Nginx or reverse proxy process is installed.
- **Non-Root Execution:** Daemon runs as unprivileged system user (`genealogy` on LXC, `appuser` on Docker) with systemd sandboxing (`NoNewPrivileges=true`, `ProtectSystem=full`, `ProtectHome=true`, `CapabilityBoundingSet=CAP_NET_BIND_SERVICE`, `AmbientCapabilities=CAP_NET_BIND_SERVICE`).
- **Minimal Footprint:** Native Axum backend idles at ~30–50MB RAM, allowing deployment on resource-constrained Proxmox LXC containers.
- **Lockout Recovery:** `genealogy-cli create-admin-link <email>` generates emergency admin login links without requiring the web UI.

## 5. Decisions
- **`ADR-001`: Single-Instance Architecture:** 1 instance = 1 family tree.
- **`ADR-002`: Offline Compilation:** Builds require zero network calls to a running database during `cargo build` or Docker image creation.
- **`ADR-008`: In-Database Advisory Locking:** Session-level advisory lock coordinates migrations and restores.
- **`ADR-012`: On-Demand Releases and Native LXC / Bare-Metal Deployment:** Single-script deployment and atomic updates using precompiled binary releases, direct port 80 Axum listening with zero Nginx layer, systemd `CAP_NET_BIND_SERVICE`, and automated PostgreSQL setup.

## 6. Gotchas & Known Limitations
- `GOT-005`: Reverse proxy header mismatches causing CSRF rejection; relevant when using Docker Compose (`frontend/nginx.conf`) or external TLS reverse proxies (e.g. Caddy, Tailscale); native LXC deployment listens directly on port 80 without proxy header rewrite concerns.

_Last curated: 2026-11-08 (Zero-proxy LXC port 80 direct binding, CAP_NET_BIND_SERVICE, automated PostgreSQL setup)_
