# Feature Specification: Single-Instance Architecture & Production Deployment (LXC & Docker)

## 1. Overview & Architectural Philosophy
The application follows a **Single-Instance = Single-Family-Tree** architecture. A self-hoster deploys their entire private family archive using either:
1. **LXC / Bare-Metal Linux (Recommended):** A single script (`scripts/install.sh`) that downloads precompiled release binaries and web assets from GitHub Releases and configures `systemd` and `nginx` without any Docker layer.
2. **Docker Compose:** A 3-service stack (`frontend`, `backend`, `postgres`).

### Core Architectural Guarantees:
1. **Single-Command Startup:** Running `scripts/install.sh` on LXC (or `docker compose up -d`) boots the complete production stack with zero manual database schema provisioning or manual migration scripts.
2. **Precompiled GitHub Releases on Demand:** Releases are created deliberately when milestones are cut via git tags (`v*.*.*`), never on every commit. Precompiled Linux x86_64 and aarch64 binaries with frontend static bundles are published to GitHub Releases.
3. **100% Configurable via Environment Variables:** Every single configuration option (domain, ports, SMTP credentials, WebAuthn RP ID, living person age thresholds, public access) is defined in a single `.env` file with sensible production defaults.
4. **Absolute Data Privacy:** Family data, private photos, and sensitive records are physically isolated in dedicated directories/volumes (`media`, `backups`, database).
5. **Zero Multi-Tenant Complexity:** No tenant IDs, no cross-account data leaks, and in-memory tree topology caching with ~30-50MB RAM footprint for the backend.
6. **Default Clean Slate (Zero Dummy Data in Production):** Fresh database migrations initialize an empty archive (0 individuals, 0 families, 0 events, 0 media, 0 places). No demo, test, or mock genealogical data may ever be seeded in production builds. The frontend displays the genuine Empty Tree Onboarding flow until real family data is imported or added.

---

## 2. Production Bare-Metal & LXC Deployment (Debian / Ubuntu)

For bare-metal servers or lightweight LXC containers (Debian 11/12, Ubuntu 22.04/24.04), the entire deployment is managed via `scripts/install.sh`. The application listens **directly on port 80** with zero Nginx or reverse proxy layer, with PostgreSQL automatically installed and configured.

### 2.1 Fresh Installation (One-Liner)
```bash
curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/install.sh | sudo bash
```

**Automated operations executed by the installer:**
1. Checks Debian/Ubuntu compatibility and CPU architecture (`x86_64` or `aarch64`).
2. Installs core utilities (`curl`, `tar`, `gzip`, `ca-certificates`, `jq`, `openssl`).
3. Installs and configures PostgreSQL (`postgresql`, `postgresql-contrib`), creates `genealogy` user & database, configures `pg_hba.conf`, and verifies local connectivity.
4. Creates dedicated `genealogy` system user.
5. Downloads precompiled release archive (`genealogy-vX.Y.Z-linux-<arch>.tar.gz`) from GitHub Releases.
6. Installs binaries in `/opt/genealogy/` and creates `/usr/local/bin/genealogy-cli` symlink.
7. Installs frontend static files to `/var/www/genealogy/`.
8. Generates production config in `/etc/genealogy/genealogy.env` (`PORT=80`, `FRONTEND_STATIC_PATH=/var/www/genealogy`, permissions `0600`).
9. Configures systemd unit `/etc/systemd/system/genealogy.service` with `AmbientCapabilities=CAP_NET_BIND_SERVICE` allowing direct binding to port 80 without root privileges, and enables/starts the service.
10. Axum backend directly serves both SPA static assets and the `/api/v1` REST endpoints on port 80 with SPA fallback and no-cache headers for `/sw.js` and `/manifest.webmanifest`.
11. Verifies `/api/v1/health` status.

### 2.2 Updating an Existing Installation
When updating an already-installed LXC instance, simply rerun:
```bash
update
# or:
sudo /opt/genealogy/scripts/install.sh
# or explicit target release:
VERSION=v0.2.0 sudo /opt/genealogy/scripts/install.sh
```
The updater automatically:
- Detects the existing installation.
- Fetches the target release from GitHub.
- Stops `genealogy.service`.
- Replaces backend and CLI binaries in `/opt/genealogy/`.
- Replaces frontend assets in `/var/www/genealogy/`.
- Preserves `/etc/genealogy/genealogy.env`, database, and media uploads intact.
- Starts `genealogy.service` — the backend automatically applies any new SQLx migrations under an advisory lock.
- Verifies backend healthcheck on the configured port.

### 2.3 Proxmox VE & Community Scripts (`clean-lxcs`) Compatibility
For users running on **Proxmox VE**, automated deployment and integration with the [Proxmox VE Community Scripts](https://community-scripts.github.io/ProxmoxVE/) ecosystem is supported:

1. **Host-Level Deployment (Proxmox Node Shell):**
   ```bash
   bash -c "$(curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/proxmox/genealogy.sh)"
   ```
   Launches the standard Community-Scripts Whiptail wizard, provisions an unprivileged Debian 12 LXC container (2 vCPU, 1 GB RAM, 8 GB disk, Nesting enabled), installs PostgreSQL, registers the container `update` alias, and binds to port 80.

2. **Upstream Helper-Scripts Files (`scripts/proxmox/`):**
   - `ct/genealogy.sh`: Community Scripts host container wrapper.
   - `install/genealogy-install.sh`: In-container provisioning script.
   - `json/genealogy.json`: Metadata catalog entry for `community-scripts.github.io/ProxmoxVE`.

3. **`clean-lxcs` Hardening Guarantees:**
   Containers running Genealogy are fully resilient to the Community Scripts [`clean-lxcs`](https://community-scripts.github.io/ProxmoxVE/scripts?id=clean-lxcs) maintenance routine:
   - Core packages (`postgresql`, `postgresql-contrib`, `curl`, `jq`, `openssl`, `ca-certificates`) are flagged `apt-mark manual` to prevent accidental removal by `apt-get autoremove`.
   - Zero temporary directory dependencies: Database, media (`/var/lib/genealogy/media`), backups, and configs (`/etc/genealogy`) reside in permanent paths; `/tmp` is wiped cleanly after install.
   - Sockets and logs use systemd runtime standards (`/var/run/postgresql`, `journalctl`), preventing broken state after `/var/log` or `/tmp` purges.

---

## 3. Production `docker-compose.yml` Reference

```yaml
services:
  # ---------------------------------------------------------------------------
  # 1. Reverse Proxy & Frontend Web Application (React 19 + Tailwind v4 + Nginx)
  # ---------------------------------------------------------------------------
  frontend:
    image: genealogy/frontend:latest
    build:
      context: ./frontend
      dockerfile: Dockerfile
    container_name: genealogy-frontend
    restart: unless-stopped
    ports:
      - "${PORT:-8080}:80"
    environment:
      - VITE_APP_TITLE=${TREE_TITLE:-Family Tree Archive}
      - VITE_DEFAULT_LOCALE=${DEFAULT_LOCALE:-en}
    depends_on:
      backend:
        condition: service_healthy
    networks:
      - genealogy-net

  # ---------------------------------------------------------------------------
  # 2. Core API Backend Engine (Rust Axum + ged_io + webauthn-rs + background job workers)
  # ---------------------------------------------------------------------------
  backend:
    image: genealogy/backend:latest
    build:
      context: ./backend
      dockerfile: Dockerfile
    container_name: genealogy-backend
    restart: unless-stopped
    expose:
      - "3000"
    env_file:
      - .env
    environment:
      - DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/${POSTGRES_DB}?sslmode=disable
      - MEDIA_STORAGE_PATH=/app/data/media
      - BACKUP_PATH=/app/data/backups
    volumes:
      - media-data:/app/data/media
      - backup-data:/app/data/backups
    depends_on:
      postgres:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/health"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 5s
    networks:
      - genealogy-net

  # ---------------------------------------------------------------------------
  # 3. Primary Relational Database (PostgreSQL 16)
  # ---------------------------------------------------------------------------
  postgres:
    image: postgres:16-alpine
    container_name: genealogy-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_USER=${POSTGRES_USER:-genealogy}
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD:?Database password required}
      - POSTGRES_DB=${POSTGRES_DB:-genealogy}
    volumes:
      - postgres-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-genealogy} -d ${POSTGRES_DB:-genealogy}"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - genealogy-net

volumes:
  postgres-data:
    driver: local
  media-data:
    driver: local
  backup-data:
    driver: local

networks:
  genealogy-net:
    driver: bridge
```

---

## 4. Exhaustive Environment Variables Reference (`.env`)

Every configurable setting is externalized into environment variables:

### 4.1 Web & Server Configuration
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `PORT` | No | `8080` | External host port exposed for the web app | `80` or `8080` |
| `APP_ENV` | No | `production` | Environment mode (`production`, `development`) | `production` |
| `RUST_LOG` | No | `info,genealogy=debug` | Logging level for Rust Axum backend | `info` |

### 4.2 Domain & Passkeys (WebAuthn / FIDO2)
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `APP_ORIGIN` | **Yes** | — | Fully qualified public URL of the web app | `https://tree.example.com` |
| `RP_ID` | **Yes** | — | WebAuthn Relying Party ID (hostname without scheme/port) | `tree.example.com` or `localhost` |
| `RP_NAME` | No | `Family Tree Archive` | Name displayed in browser Touch ID / Passkey prompts | `Levshits Family Tree` |

### 4.3 Database & Cache
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `POSTGRES_USER` | No | `genealogy` | PostgreSQL database username | `genealogy` |
| `POSTGRES_PASSWORD` | **Yes** | — | Strong password for PostgreSQL database | `sUpErS3cr3tP@ss!` |
| `POSTGRES_DB` | No | `genealogy` | PostgreSQL database name | `genealogy` |
| `DATABASE_URL` | No | Constructed | Full PostgreSQL connection string (overrides individual credentials) | `postgresql://user:pass@127.0.0.1:5432/genealogy?sslmode=disable` |

### 4.4 Media & Storage Settings
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `MAX_UPLOAD_SIZE_MB` | No | `100` | Maximum file upload size in megabytes | `200` |
| `THUMBNAIL_QUALITY` | No | `85` | WebP compression quality for cached avatars | `90` |
| `MEDIA_STORAGE_PATH` | No | `/app/data/media` | Path where photos and documents are stored | `/var/lib/genealogy/media` |
| `BACKUP_PATH` | No | `/app/data/backups` | Path where disaster recovery snapshots are stored | `/var/lib/genealogy/backups` |
| `FRONTEND_STATIC_PATH` | No | — | Optional path to frontend static bundle for native Axum serving | `/var/www/genealogy` |

### 4.5 Privacy, Access & Internationalization
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `TREE_TITLE` | No | `Family Tree Archive` | Display title shown in navbar and print banners | `Архив Семьи Левшиц` |
| `DEFAULT_LOCALE` | No | `en` | Default UI language (`en` or `ru`) | `ru` |
| `ENABLE_PUBLIC_ACCESS` | No | `false` | Allow anonymous visitors to view tree with living data & photos masked | `true` |
| `ENABLE_REGISTRATION` | No | `false` | Allow open user registration (if false, invite-only) | `false` |
| `LIVING_PERSON_AGE_THRESHOLD` | No | `110` | Age in years to assume an individual is living if no death date | `100` |

### 4.6 Transactional Email (SMTP) for Magic Links & Invites
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `SMTP_ENABLED` | No | `false` | Enable transactional email delivery | `true` |
| `SMTP_HOST` | If enabled| — | SMTP server hostname | `smtp.mailgun.org` |
| `SMTP_PORT` | No | `587` | SMTP server port (`587` for STARTTLS, `465` for SSL) | `587` |
| `SMTP_USER` | If enabled| — | SMTP authentication username | `postmaster@example.com` |
| `SMTP_PASSWORD` | If enabled| — | SMTP authentication password | `smtp-secret-key` |
| `SMTP_FROM_EMAIL` | If enabled| — | Sender email address for magic links | `tree@example.com` |
| `SMTP_FROM_NAME` | No | `Family Tree` | Sender display name | `Levshits Archive` |

### 4.7 Automated Backups & Disaster Recovery
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `BACKUP_ENABLED` | No | `true` | Enable automated scheduled snapshots | `true` |
| `BACKUP_CRON` | No | `0 3 * * *` | Standard cron schedule (default: 3:00 AM daily) | `0 3 * * *` |
| `BACKUP_RETENTION_DAYS` | No | `30` | Number of daily archives kept before auto-pruning | `30` |
| `BACKUP_INCLUDE_GEDCOM` | No | `false` | Also write a GEDZIP export next to each nightly backup | `true` |

### 4.8 Authentication, Sessions & Limits (added by specs 01 / 17)
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `AUTH_PASSWORD_ENABLED` | No | `auto` | Password fallback: `auto` (on when no HTTPS or no SMTP), `true`, `false` | `auto` |
| `SESSION_TTL_DAYS` | No | `30` | Sliding session lifetime | `30` |
| `MAX_IMPORT_SIZE_MB` | No | `2048` | Max uncompressed size of GEDZIP/ZIP imports | `4096` |
| `JOB_WORKERS` | No | `2` | Concurrent background job workers | `2` |

### 4.9 Maps, History & Settings (added by specs 10 / 12 / 14)
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `MAP_TILE_URL` | No | OSM tiles | Map tile URL template; **empty disables maps** (no external requests) | `https://tiles.home.lan/{z}/{x}/{y}.png` |
| `MAP_ATTRIBUTION` | No | `© OpenStreetMap contributors` | Map attribution text | |
| `GEOCODING_ENABLED` | No | `false` | Allow explicit "Find coordinates" lookups | `true` |
| `GEOCODING_URL` | No | `https://nominatim.openstreetmap.org` | Nominatim endpoint | |
| `HISTORY_RETENTION_DAYS` | No | `365` | Change-log retention (`0` = forever) | `0` |
| `SETTINGS_LOCKED_FROM_ENV` | No | `false` | Env values override Admin UI settings ([10 §7](file:///Users/levshitsvv/Projects/Personal/genealogy/docs/features/10-core-data-model.md#7-configuration-precedence-env-vs-admin-ui)) | `true` |
| `PUBLIC_SEARCH_SHOW_LIVING` | No | `false` | In public mode, show masked initial results (e.g. "И." / "J.") in search | `false` |

---

## 5. Disaster Recovery & CLI Operations

### 5.1 Creating an On-Demand Backup
```bash
# LXC / Bare-Metal:
sudo genealogy-cli backup

# Docker:
docker compose exec backend ./genealogy-cli backup
```
Generates `/var/lib/genealogy/backups/backup-[TIMESTAMP].tar.gz` (or `/app/data/backups/`) containing `database.dump`, media assets, and checksum manifest.

### 5.2 Restoring from Backup Archive
```bash
# LXC / Bare-Metal:
sudo genealogy-cli restore /var/lib/genealogy/backups/backup-2026-10-26.tar.gz

# Docker:
docker compose exec backend ./genealogy-cli restore /app/data/backups/backup-2026-10-26.tar.gz
```

### 5.3 Other CLI Commands
```bash
genealogy-cli create-admin-link <email>   # emergency sign-in link if all admins are locked out
genealogy-cli regenerate-thumbnails
genealogy-cli export-gedcom /var/lib/genealogy/backups/tree.gdz
```

---

## 6. Release Pipeline & Versioning Cadence

### 6.1 On-Demand Release Strategy
- Releases are **never** generated for each individual commit. Commits land on `main` via PRs or direct milestones.
- Releases are triggered exclusively by pushing an annotated git tag (`v*.*.*`) or through manual `workflow_dispatch` on GitHub Actions (`.github/workflows/release.yml`).
- Use the `.agents/skills/genealogy-release/` skill to automate version bumping, verification test runs, tag creation, and release dispatch.

### 6.2 Artifact Packaging
Each release builds two cross-compiled packages:
- `genealogy-vX.Y.Z-linux-x86_64.tar.gz` (Standard 64-bit x86 LXC / Proxmox / Bare-Metal)
- `genealogy-vX.Y.Z-linux-aarch64.tar.gz` (64-bit ARM LXC / Raspberry Pi / Ampere)

Each tarball bundles the precompiled `genealogy-backend` binary, `genealogy-cli` binary, production `frontend/` static assets, systemd unit file, and `scripts/install.sh`.

---

## 7. Hosting on a Home Network (HTTPS & Passkeys)
Passkeys only work on `https://` or `http://localhost`. Choose one:

| Option | Effort | Result |
| :--- | :--- | :--- |
| **Plain HTTP on LAN** (`http://192.168.1.20:8080`) | None | Works with **password** / admin sign-in links only (passkeys hidden). Fine for a home-only install. |
| **Tailscale** (`https://tree.tailnet-name.ts.net`) | Low | Real certificate, passkeys work, accessible to family anywhere on the tailnet, nothing exposed to the internet. Recommended. |
| **Caddy reverse proxy + own domain** | Medium | Automatic Let's Encrypt; public internet access (consider `ENABLE_PUBLIC_ACCESS=false`). |
| **Caddy with internal CA** (`tls internal`) | Medium | HTTPS on LAN, but each device must trust the local root certificate. |

`APP_ORIGIN` and `RP_ID` must match the URL family members actually use; changing `RP_ID` later invalidates existing passkeys (users fall back to password/magic link and re-enroll).

---

## 8. Definition of Done (DoD)

- [ ] **LXC Single-Script Deployment:** Running `scripts/install.sh` on a Debian or Ubuntu system installs binaries, provisions PostgreSQL if needed, configures systemd and Nginx, and starts a healthy application without Docker.
- [ ] **LXC In-Place Update:** Running `scripts/install.sh` on an existing install updates backend binaries and frontend assets while preserving `.env`, database records, and media files, automatically running database migrations on startup.
- [ ] **On-Demand Releases:** GitHub Actions release workflow triggers solely on tag push (`v*.*.*`) or manual dispatch, cross-compiling binaries for `x86_64` and `aarch64` and packaging complete tarballs with checksums.
- [ ] **Single File Execution (Docker):** Running `docker compose up -d` with a populated `.env` brings all services online into a healthy state without interactive terminal prompts.
- [ ] **Full Variable Coverage:** Changing any variable in `.env` (ports, locale, public access, SMTP, living age threshold) immediately takes effect upon container/service restart.
- [ ] **Volume Persistence:** Completely stopping containers/services and starting them again preserves all data, accounts, passkeys, and media.
- [ ] **Healthcheck Verification:** Backend waits for PostgreSQL to be fully healthy before initiating database connection pools and automatic migrations.
- [ ] **Non-Root Security:** Processes run as unprivileged user (`genealogy` on LXC, `appuser` on Docker) with minimal attack surface.
- [ ] **Minimal Footprint:** Full stack runs with minimal resource usage (~30-50MB RAM for Axum backend).
- [ ] **Lockout Recovery:** `genealogy-cli create-admin-link` lets the server owner regain admin access without the UI.
- [ ] **Default Clean Slate:** Fresh deployment initializes an empty database with zero dummy/demo individuals, families, places, or mock dashboard items; the UI presents the authentic empty-tree onboarding card.
