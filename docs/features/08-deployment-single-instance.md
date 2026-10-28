# Feature Specification: Single-Instance Architecture & Docker Compose Deployment

## 1. Overview & Architectural Philosophy
The application follows a **Single-Instance = Single-Family-Tree** architecture. A self-hoster deploys **one single Docker Compose file** to run their entire private family archive.

### Core Architectural Guarantees:
1. **Single-Command Startup:** Running `docker compose up -d` boots the complete production stack (Frontend, Backend, PostgreSQL) with zero manual database provisioning or manual migration scripts.
2. **100% Configurable via Environment Variables:** Every single configuration option (domain, ports, SMTP credentials, WebAuthn RP ID, living person age thresholds, public access) is defined in a single `.env` file with sensible production defaults.
3. **Absolute Data Privacy:** Family data, private photos, and sensitive records are physically isolated in dedicated local volumes (`postgres-data`, `media-data`, `backup-data`).
4. **Zero Multi-Tenant Complexity:** No tenant IDs, no cross-account data leaks, and in-memory tree topology caching with ~30-50MB RAM footprint for the backend.
5. **Default Clean Slate (Zero Dummy Data in Production):** Fresh database migrations initialize an empty archive (0 individuals, 0 families, 0 events, 0 media, 0 places). No demo, test, or mock genealogical data may ever be seeded in production builds. The frontend displays the genuine Empty Tree Onboarding flow until real family data is imported or added.

---

## 2. Complete Production `docker-compose.yml` Reference

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

## 3. Exhaustive Environment Variables Reference (`.env`)

Every configurable setting is externalized into environment variables:

### 3.1 Web & Server Configuration
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `PORT` | No | `8080` | External host port exposed for the web app | `80` or `8080` |
| `APP_ENV` | No | `production` | Environment mode (`production`, `development`) | `production` |
| `RUST_LOG` | No | `info,genealogy=debug` | Logging level for Rust Axum backend | `info` |

### 3.2 Domain & Passkeys (WebAuthn / FIDO2)
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `APP_ORIGIN` | **Yes** | — | Fully qualified public URL of the web app | `https://tree.example.com` |
| `RP_ID` | **Yes** | — | WebAuthn Relying Party ID (hostname without scheme/port) | `tree.example.com` or `localhost` |
| `RP_NAME` | No | `Family Tree Archive` | Name displayed in browser Touch ID / Passkey prompts | `Levshits Family Tree` |

### 3.3 Database & Cache
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `POSTGRES_USER` | No | `genealogy` | PostgreSQL database username | `genealogy` |
| `POSTGRES_PASSWORD` | **Yes** | — | Strong password for PostgreSQL database | `sUpErS3cr3tP@ss!` |
| `POSTGRES_DB` | No | `genealogy` | PostgreSQL database name | `genealogy` |

### 3.4 Media & Storage Settings
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `MAX_UPLOAD_SIZE_MB` | No | `100` | Maximum file upload size in megabytes | `200` |
| `THUMBNAIL_QUALITY` | No | `85` | WebP compression quality for cached avatars | `90` |
| `MEDIA_STORAGE_PATH` | No | `/app/data/media` | Path inside container where photos are stored | `/app/data/media` |

### 3.5 Privacy, Access & Internationalization
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `TREE_TITLE` | No | `Family Tree Archive` | Display title shown in navbar and print banners | `Архив Семьи Левшиц` |
| `DEFAULT_LOCALE` | No | `en` | Default UI language (`en` or `ru`) | `ru` |
| `ENABLE_PUBLIC_ACCESS` | No | `false` | Allow anonymous visitors to view tree with living data & photos masked | `true` |
| `ENABLE_REGISTRATION` | No | `false` | Allow open user registration (if false, invite-only) | `false` |
| `LIVING_PERSON_AGE_THRESHOLD` | No | `110` | Age in years to assume an individual is living if no death date | `100` |

### 3.6 Transactional Email (SMTP) for Magic Links & Invites
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `SMTP_ENABLED` | No | `false` | Enable transactional email delivery | `true` |
| `SMTP_HOST` | If enabled| — | SMTP server hostname | `smtp.mailgun.org` |
| `SMTP_PORT` | No | `587` | SMTP server port (`587` for STARTTLS, `465` for SSL) | `587` |
| `SMTP_USER` | If enabled| — | SMTP authentication username | `postmaster@example.com` |
| `SMTP_PASSWORD` | If enabled| — | SMTP authentication password | `smtp-secret-key` |
| `SMTP_FROM_EMAIL` | If enabled| — | Sender email address for magic links | `tree@example.com` |
| `SMTP_FROM_NAME` | No | `Family Tree` | Sender display name | `Levshits Archive` |

### 3.7 Automated Backups & Disaster Recovery
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `BACKUP_ENABLED` | No | `true` | Enable automated scheduled snapshots | `true` |
| `BACKUP_CRON` | No | `0 3 * * *` | Standard cron schedule (default: 3:00 AM daily) | `0 3 * * *` |
| `BACKUP_RETENTION_DAYS` | No | `30` | Number of daily archives kept before auto-pruning | `30` |
| `BACKUP_INCLUDE_GEDCOM` | No | `false` | Also write a GEDZIP export next to each nightly backup | `true` |

### 3.8 Authentication, Sessions & Limits (added by specs 01 / 17)
| Variable | Required | Default | Description | Example |
| :--- | :---: | :---: | :--- | :--- |
| `AUTH_PASSWORD_ENABLED` | No | `auto` | Password fallback: `auto` (on when no HTTPS or no SMTP), `true`, `false` | `auto` |
| `SESSION_TTL_DAYS` | No | `30` | Sliding session lifetime | `30` |
| `MAX_IMPORT_SIZE_MB` | No | `2048` | Max uncompressed size of GEDZIP/ZIP imports | `4096` |
| `JOB_WORKERS` | No | `2` | Concurrent background job workers | `2` |

### 3.9 Maps, History & Settings (added by specs 10 / 12 / 14)
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

## 4. Disaster Recovery & CLI Operations

### 4.1 Creating an On-Demand Backup
```bash
docker compose exec backend ./genealogy-cli backup
```
Generates `/app/data/backups/backup-[TIMESTAMP].tar.gz` containing `database.dump`, media assets, and checksum manifest.

### 4.2 Restoring from Backup Archive
```bash
docker compose exec backend ./genealogy-cli restore /app/data/backups/backup-2026-10-26.tar.gz
```

### 4.3 Other CLI Commands
```bash
docker compose exec backend ./genealogy-cli create-admin-link <email>   # emergency sign-in link if all admins are locked out
docker compose exec backend ./genealogy-cli regenerate-thumbnails
docker compose exec backend ./genealogy-cli export-gedcom /app/data/backups/tree.gdz
```

---

## 5. Hosting on a Home Network (HTTPS & Passkeys)
Passkeys only work on `https://` or `http://localhost`. Choose one:

| Option | Effort | Result |
| :--- | :--- | :--- |
| **Plain HTTP on LAN** (`http://192.168.1.20:8080`) | None | Works with **password** / admin sign-in links only (passkeys hidden). Fine for a home-only install. |
| **Tailscale** (`https://tree.tailnet-name.ts.net`) | Low | Real certificate, passkeys work, accessible to family anywhere on the tailnet, nothing exposed to the internet. Recommended. |
| **Caddy reverse proxy + own domain** | Medium | Automatic Let's Encrypt; public internet access (consider `ENABLE_PUBLIC_ACCESS=false`). |
| **Caddy with internal CA** (`tls internal`) | Medium | HTTPS on LAN, but each device must trust the local root certificate. |

`APP_ORIGIN` and `RP_ID` must match the URL family members actually use; changing `RP_ID` later invalidates existing passkeys (users fall back to password/magic link and re-enroll).

---

## 6. Definition of Done (DoD)

- [ ] **Single File Execution:** Running `docker compose up -d` with a populated `.env` brings all services online into a healthy state without interactive terminal prompts.
- [ ] **Full Variable Coverage:** Changing any variable in `.env` (ports, locale, public access, SMTP, living age threshold) immediately takes effect upon container restart.
- [ ] **Volume Persistence:** Completely stopping containers (`docker compose down`) and starting them again preserves all data, accounts, passkeys, and media.
- [ ] **Healthcheck Verification:** Backend waits for PostgreSQL to be fully healthy before initiating database connection pools and automatic migrations.
- [ ] **Non-Root Security:** Containers run as unprivileged users (`appuser`) with minimal attack surface.
- [ ] **Minimal Footprint:** Full stack runs with 3 containers (frontend, backend, postgres) and idles under ~150 MB RAM total.
- [ ] **Lockout Recovery:** `genealogy-cli create-admin-link` lets the server owner regain admin access without the UI.
- [ ] **Default Clean Slate:** Fresh deployment initializes an empty database with zero dummy/demo individuals, families, places, or mock dashboard items; the UI presents the authentic empty-tree onboarding card.
