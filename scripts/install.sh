#!/usr/bin/env bash
# ==============================================================================
# Genealogy Family Tree — Unified LXC / Bare-Metal Installer & Updater
# Target OS: Debian 11/12, Ubuntu 22.04/24.04 (x86_64, aarch64)
# Direct listening on port 80 (no Nginx reverse proxy) + automatic PostgreSQL
#
# Usage:
#   Install latest: curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/install.sh | sudo bash
#   Update:         sudo /opt/genealogy/scripts/install.sh
#   Options:        VERSION=v0.1.0 PORT=80 sudo -E bash install.sh
# ==============================================================================

set -euo pipefail

REPO="${REPO:-vlevshits/genealogy}"
APP_VERSION="${APP_VERSION:-${VERSION:-latest}}"
INSTALL_DIR="/opt/genealogy"
WEB_DIR="/var/www/genealogy"
DATA_DIR="/var/lib/genealogy"
CONFIG_DIR="/etc/genealogy"
CONFIG_FILE="${CONFIG_DIR}/genealogy.env"
SERVICE_NAME="genealogy"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# ------------------------------------------------------------------------------
# 1. Pre-flight Checks (Root, OS, Architecture)
# ------------------------------------------------------------------------------
if [ "$(id -u)" -ne 0 ]; then
    log_error "This script must be run as root (or via sudo)."
    exit 1
fi

if [ ! -f /etc/os-release ]; then
    log_error "/etc/os-release not found. Unsupported Linux distribution."
    exit 1
fi

. /etc/os-release
OS_ID="${ID:-}"
OS_LIKE="${ID_LIKE:-}"

if [[ "$OS_ID" != "debian" && "$OS_ID" != "ubuntu" && ! "$OS_LIKE" =~ "debian" && ! "$OS_LIKE" =~ "ubuntu" ]]; then
    log_warn "Distribution is '${OS_ID}'. This script is primarily tested on Debian and Ubuntu."
fi

UNAME_M="$(uname -m)"
case "$UNAME_M" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        log_error "Unsupported CPU architecture: ${UNAME_M}. Supported: x86_64, aarch64"
        exit 1
        ;;
esac

log_info "Detected OS: ${NAME:-Linux} (${ARCH})"

# ------------------------------------------------------------------------------
# 2. Resolve Target Version
# ------------------------------------------------------------------------------
fetch_latest_release_tag() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    local tag=""
    if command -v curl >/dev/null && command -v jq >/dev/null; then
        tag="$(curl -fsSL "$api_url" 2>/dev/null | jq -r '.tag_name // empty' || true)"
    fi
    if [ -z "$tag" ]; then
        # Fallback to redirect header
        tag="$(curl -fsSI "https://github.com/${REPO}/releases/latest" 2>/dev/null | grep -i '^location:' | sed -E 's|.*/tag/(.*)|\1|' | tr -d '\r\n' || true)"
    fi
    if [ -z "$tag" ]; then
        log_error "Failed to detect latest release tag from https://github.com/${REPO}."
        log_error "You can specify a version explicitly: VERSION=v0.1.0 bash install.sh"
        exit 1
    fi
    echo "$tag"
}

# ------------------------------------------------------------------------------
# 3. Detect Mode: Update vs Initial Install
# ------------------------------------------------------------------------------
IS_UPDATE=0
if [ -f "${INSTALL_DIR}/genealogy-backend" ] && [ -f "/etc/systemd/system/${SERVICE_NAME}.service" ]; then
    IS_UPDATE=1
fi

TMP_DIR="$(mktemp -d /tmp/genealogy-install-XXXXXX)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# ------------------------------------------------------------------------------
# 4. Update Mode Workflow
# ------------------------------------------------------------------------------
if [ "$IS_UPDATE" -eq 1 ]; then
    log_info "============================================================"
    log_info " Existing installation detected at ${INSTALL_DIR}"
    log_info " Running UPDATE mode"
    log_info "============================================================"

    # Read current PORT from config
    CURRENT_PORT="80"
    if [ -f "${CONFIG_FILE}" ]; then
        CURRENT_PORT="$(grep -E '^PORT=' "${CONFIG_FILE}" | cut -d '=' -f2 | tr -d ' ' || echo "80")"
    fi
    CURRENT_PORT="${CURRENT_PORT:-80}"

    if [ "$APP_VERSION" = "latest" ]; then
        log_info "Checking latest release version..."
        TARGET_TAG="$(fetch_latest_release_tag)"
    else
        TARGET_TAG="$APP_VERSION"
    fi

    log_info "Target release: ${TARGET_TAG}"
    ARCHIVE_NAME="genealogy-${TARGET_TAG}-linux-${ARCH}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TARGET_TAG}/${ARCHIVE_NAME}"

    log_info "Downloading ${DOWNLOAD_URL}..."
    if ! curl -fsSL -o "${TMP_DIR}/${ARCHIVE_NAME}" "$DOWNLOAD_URL"; then
        log_error "Failed to download release archive from ${DOWNLOAD_URL}."
        exit 1
    fi

    log_info "Extracting release bundle..."
    mkdir -p "${TMP_DIR}/extracted"
    tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}/extracted"

    log_info "Stopping ${SERVICE_NAME}.service..."
    systemctl stop "${SERVICE_NAME}" || true

    log_info "Backing up current binaries..."
    cp -p "${INSTALL_DIR}/genealogy-backend" "${INSTALL_DIR}/genealogy-backend.old" || true

    log_info "Installing updated binaries..."
    install -m 755 "${TMP_DIR}/extracted/genealogy-backend" "${INSTALL_DIR}/genealogy-backend"
    if [ -f "${TMP_DIR}/extracted/genealogy-cli" ]; then
        install -m 755 "${TMP_DIR}/extracted/genealogy-cli" "${INSTALL_DIR}/genealogy-cli"
    fi
    # Also save updater inside install directory
    mkdir -p "${INSTALL_DIR}/scripts"
    if [ -f "${TMP_DIR}/extracted/scripts/install.sh" ]; then
        install -m 755 "${TMP_DIR}/extracted/scripts/install.sh" "${INSTALL_DIR}/scripts/install.sh"
    fi

    if [ -d "${TMP_DIR}/extracted/frontend" ]; then
        log_info "Updating frontend static assets..."
        mkdir -p "${WEB_DIR}"
        # Atomic directory swap
        rm -rf "${WEB_DIR}.old"
        if [ -d "${WEB_DIR}" ]; then
            mv "${WEB_DIR}" "${WEB_DIR}.old"
        fi
        mkdir -p "${WEB_DIR}"
        cp -r "${TMP_DIR}/extracted/frontend/"* "${WEB_DIR}/"
        rm -rf "${WEB_DIR}.old"
        chown -R genealogy:genealogy "${WEB_DIR}"
    fi

    log_info "Starting ${SERVICE_NAME}.service (applying database migrations)..."
    systemctl start "${SERVICE_NAME}"

    log_info "Verifying backend health check on port ${CURRENT_PORT}..."
    HEALTHY=0
    for i in {1..15}; do
        if curl -fsS "http://127.0.0.1:${CURRENT_PORT}/api/v1/health" >/dev/null 2>&1; then
            HEALTHY=1
            break
        fi
        sleep 1
    done

    if [ "$HEALTHY" -eq 1 ]; then
        log_success "Update to ${TARGET_TAG} completed successfully!"
        log_success "Service is online and healthy."
    else
        log_warn "Service started, but healthcheck at http://127.0.0.1:${CURRENT_PORT}/api/v1/health did not return 200 within 15s."
        log_warn "Check status: journalctl -u ${SERVICE_NAME} -n 50 --no-pager"
    fi
    exit 0
fi

# ------------------------------------------------------------------------------
# 5. Initial Install Workflow
# ------------------------------------------------------------------------------
log_info "============================================================"
log_info " Genealogy Family Tree — Fresh LXC / Bare-Metal Installer"
log_info " Target: ${INSTALL_DIR}"
log_info " Direct web listening (no proxy) + PostgreSQL auto-setup"
log_info "============================================================"

# 5.1 System packages
log_info "Installing core system packages..."
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq curl tar gzip ca-certificates jq openssl

# 5.2 PostgreSQL installation & configuration
log_info "Installing and configuring PostgreSQL..."
apt-get install -y -qq postgresql postgresql-contrib
systemctl enable --now postgresql

log_info "Configuring local PostgreSQL database and role 'genealogy'..."
PG_PASS="$(openssl rand -hex 16)"

su - postgres -c "psql -c \"DO \\\$do\\\$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'genealogy') THEN CREATE USER genealogy WITH ENCRYPTED PASSWORD '${PG_PASS}'; ELSE ALTER USER genealogy WITH ENCRYPTED PASSWORD '${PG_PASS}'; END IF; END \\\$do\\\$;\"" >/dev/null

PG_DB_EXISTS=$(su - postgres -c "psql -tAc \"SELECT 1 FROM pg_database WHERE datname='genealogy'\"" 2>/dev/null || true)
if [ "$PG_DB_EXISTS" != "1" ]; then
    su - postgres -c "psql -c \"CREATE DATABASE genealogy OWNER genealogy;\"" >/dev/null
fi
su - postgres -c "psql -c \"GRANT ALL PRIVILEGES ON DATABASE genealogy TO genealogy;\"" >/dev/null

# Ensure pg_hba.conf permits password authentication over 127.0.0.1
PG_HBA=$(find /etc/postgresql/ -name "pg_hba.conf" 2>/dev/null | head -n 1 || true)
if [ -n "$PG_HBA" ] && [ -f "$PG_HBA" ]; then
    if ! grep -qE "^host\s+all\s+genealogy\s+127.0.0.1/32" "$PG_HBA"; then
        sed -i '1i host    all             genealogy       127.0.0.1/32            scram-sha-256' "$PG_HBA"
        sed -i '2i host    all             genealogy       ::1/128                 scram-sha-256' "$PG_HBA"
        systemctl reload postgresql || systemctl restart postgresql
    fi
fi

DB_URL="postgresql://genealogy:${PG_PASS}@127.0.0.1:5432/genealogy?sslmode=disable"

# Verify PostgreSQL connection immediately
if command -v psql >/dev/null; then
    if PGPASSWORD="${PG_PASS}" psql -h 127.0.0.1 -U genealogy -d genealogy -c "SELECT 1;" >/dev/null 2>&1; then
        log_success "PostgreSQL configured and verified successfully."
    else
        log_warn "Connection verification via 127.0.0.1 returned non-zero; proceeding with socket fallback."
    fi
fi

# 5.3 System user & directories
log_info "Creating dedicated system user 'genealogy'..."
if ! id -u genealogy >/dev/null 2>&1; then
    useradd --system --no-create-home --home-dir "${DATA_DIR}" --shell /usr/sbin/nologin genealogy
fi

log_info "Creating directory structure..."
mkdir -p "${INSTALL_DIR}" "${INSTALL_DIR}/scripts"
mkdir -p "${WEB_DIR}"
mkdir -p "${DATA_DIR}/media" "${DATA_DIR}/backups"
mkdir -p "${CONFIG_DIR}"

chown -R genealogy:genealogy "${DATA_DIR}"
chmod 750 "${DATA_DIR}"
chown -R genealogy:genealogy "${INSTALL_DIR}" "${CONFIG_DIR}" "${WEB_DIR}"

# 5.4 Download Release Bundle
if [ "$APP_VERSION" = "latest" ]; then
    log_info "Checking latest release version..."
    TARGET_TAG="$(fetch_latest_release_tag)"
else
    TARGET_TAG="$APP_VERSION"
fi

ARCHIVE_NAME="genealogy-${TARGET_TAG}-linux-${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TARGET_TAG}/${ARCHIVE_NAME}"

log_info "Downloading release ${TARGET_TAG} (${ARCH})..."
if ! curl -fsSL -o "${TMP_DIR}/${ARCHIVE_NAME}" "$DOWNLOAD_URL"; then
    log_error "Failed to download release archive: ${DOWNLOAD_URL}"
    exit 1
fi

log_info "Extracting release bundle..."
mkdir -p "${TMP_DIR}/extracted"
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}/extracted"

# 5.5 Install binaries and web assets
log_info "Installing binaries to ${INSTALL_DIR}..."
install -m 755 "${TMP_DIR}/extracted/genealogy-backend" "${INSTALL_DIR}/genealogy-backend"
if [ -f "${TMP_DIR}/extracted/genealogy-cli" ]; then
    install -m 755 "${TMP_DIR}/extracted/genealogy-cli" "${INSTALL_DIR}/genealogy-cli"
    ln -sf "${INSTALL_DIR}/genealogy-cli" /usr/local/bin/genealogy-cli
fi

if [ -f "${TMP_DIR}/extracted/scripts/install.sh" ]; then
    install -m 755 "${TMP_DIR}/extracted/scripts/install.sh" "${INSTALL_DIR}/scripts/install.sh"
fi

if [ -d "${TMP_DIR}/extracted/frontend" ]; then
    log_info "Installing frontend static files to ${WEB_DIR}..."
    cp -r "${TMP_DIR}/extracted/frontend/"* "${WEB_DIR}/"
    chown -R genealogy:genealogy "${WEB_DIR}"
fi

# 5.6 Create /etc/genealogy/genealogy.env
TARGET_PORT="${PORT:-80}"
PRIMARY_IP="$(ip route get 1.1.1.1 2>/dev/null | awk '{print $7}' || hostname -I | awk '{print $1}')"
PRIMARY_IP="${PRIMARY_IP:-localhost}"

if [ "$TARGET_PORT" = "80" ]; then
    DEFAULT_ORIGIN="http://${PRIMARY_IP}"
else
    DEFAULT_ORIGIN="http://${PRIMARY_IP}:${TARGET_PORT}"
fi

DETECTED_ORIGIN="${APP_ORIGIN:-$DEFAULT_ORIGIN}"
DETECTED_RP_ID="${RP_ID:-$PRIMARY_IP}"

if [ ! -f "${CONFIG_FILE}" ]; then
    log_info "Generating production environment configuration (${CONFIG_FILE})..."
    cat > "${CONFIG_FILE}" <<EOF
# ==============================================================================
# Genealogy Family Tree — Production Configuration
# Auto-generated by install.sh on $(date)
# ==============================================================================

# Direct listening on port ${TARGET_PORT} (Axum native serving)
PORT=${TARGET_PORT}
APP_ENV=production
RUST_LOG=info,genealogy=debug

APP_ORIGIN=${DETECTED_ORIGIN}
RP_ID=${DETECTED_RP_ID}
RP_NAME=Family Tree Archive

DATABASE_URL=${DB_URL}
DATABASE_REQUIRED=true

MEDIA_STORAGE_PATH=${DATA_DIR}/media
BACKUP_PATH=${DATA_DIR}/backups
FRONTEND_STATIC_PATH=${WEB_DIR}
MAX_UPLOAD_SIZE_MB=100
THUMBNAIL_QUALITY=85

TREE_TITLE=Family Tree Archive
DEFAULT_LOCALE=ru
ENABLE_PUBLIC_ACCESS=false
ENABLE_REGISTRATION=false
LIVING_PERSON_AGE_THRESHOLD=110

AUTH_PASSWORD_ENABLED=auto
SESSION_TTL_DAYS=30
MAX_IMPORT_SIZE_MB=2048
JOB_WORKERS=2

BACKUP_ENABLED=true
BACKUP_CRON=0 3 * * *
BACKUP_RETENTION_DAYS=30
BACKUP_INCLUDE_GEDCOM=false

SMTP_ENABLED=false
GEMINI_API_KEY=
GEMINI_MODEL=gemini-3.8-flash
EOF
    chmod 600 "${CONFIG_FILE}"
    chown genealogy:genealogy "${CONFIG_FILE}"
    log_success "Created ${CONFIG_FILE}"
fi

# 5.7 Install systemd service with CAP_NET_BIND_SERVICE
log_info "Configuring systemd service..."
cat > "/etc/systemd/system/${SERVICE_NAME}.service" <<EOF
[Unit]
Description=Genealogy Family Tree Server
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=genealogy
Group=genealogy
WorkingDirectory=${DATA_DIR}
EnvironmentFile=${CONFIG_FILE}
ExecStart=${INSTALL_DIR}/genealogy-backend
Restart=always
RestartSec=5s

# Grant non-root user ability to bind directly to privileged port 80/443
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

NoNewPrivileges=true
ProtectSystem=full
ProtectHome=true
ReadWritePaths=${DATA_DIR}
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now "${SERVICE_NAME}"
log_success "Enabled and started ${SERVICE_NAME}.service"

# 5.8 Health check
log_info "Verifying backend health on port ${TARGET_PORT}..."
HEALTHY=0
for i in {1..20}; do
    if curl -fsS "http://127.0.0.1:${TARGET_PORT}/api/v1/health" >/dev/null 2>&1; then
        HEALTHY=1
        break
    fi
    sleep 1
done

echo ""
echo "=================================================================="
if [ "$HEALTHY" -eq 1 ]; then
    log_success "Genealogy has been successfully installed and started!"
else
    log_warn "Installation finished, but backend health check is still initializing."
fi
echo "=================================================================="
echo " Web Application URL : ${DETECTED_ORIGIN}"
echo " Listening directly  : Port ${TARGET_PORT} (Zero proxy layer)"
echo " Database            : PostgreSQL (Local 127.0.0.1:5432)"
echo " Configuration file  : ${CONFIG_FILE}"
echo " Media folder        : ${DATA_DIR}/media"
echo " Backups folder      : ${DATA_DIR}/backups"
echo " Service status      : systemctl status ${SERVICE_NAME}"
echo " Service logs        : journalctl -u ${SERVICE_NAME} -f"
echo ""
echo " Next Steps:"
echo "   1. Open ${DETECTED_ORIGIN} in your browser."
echo "   2. Follow the First-Run Setup Wizard in the UI to create your"
echo "      admin account, set a password/passkey, and name your family tree."
echo ""
echo " Optional Emergency CLI (only needed if locked out in the future):"
echo "   sudo genealogy-cli create-admin-link your-email@example.com"
echo ""
echo " To update to the latest release in the future, simply run:"
echo "   sudo ${INSTALL_DIR}/scripts/install.sh"
echo "=================================================================="
