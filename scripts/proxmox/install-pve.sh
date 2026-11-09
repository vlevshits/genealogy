#!/usr/bin/env bash
# ==============================================================================
# Genealogy Family Tree — Standalone Proxmox VE Host Provisioner
# Creates a clean, unprivileged Debian 12 LXC container and installs Genealogy.
#
# Usage (run from Proxmox VE host shell):
#   bash -c "$(curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/proxmox/install-pve.sh)"
# ==============================================================================

set -euo pipefail

YW=$(echo "\033[33m")
BL=$(echo "\033[36m")
RD=$(echo "\033[01;31m")
GN=$(echo "\033[1;92m")
CL=$(echo "\033[m")

echo -e "${BL}"
cat << "EOF"
   ____                      _                 
  / ___| ___ _ __   ___  __ _| | ___   __ _ _   _ 
 | |  _ / _ \ '_ \ / _ \/ _` | |/ _ \ / _` | | | |
 | |_| |  __/ | | |  __/ (_| | | (_) | (_| | |_| |
  \____|\___|_| |_|\___|\__,_|_|\___/ \__, |\__, |
                                      |___/ |___/ 
      Proxmox VE LXC Automated Deployment
EOF
echo -e "${CL}"

# 1. Host Validation
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RD}[ERROR] This script must be run as root on the Proxmox VE host.${CL}" >&2
    exit 1
fi

if ! command -v pct >/dev/null 2>&1 || ! command -v pveversion >/dev/null 2>&1; then
    echo -e "${RD}[ERROR] This script is meant to be run directly on a Proxmox VE host.${CL}" >&2
    exit 1
fi

echo -e "${BL}[INFO] Running on Proxmox VE: $(pveversion)${CL}"

# 2. Defaults & Detection
NEXT_ID="$(pvesh get /cluster/nextid 2>/dev/null || echo 150)"
CT_ID="${CT_ID:-$NEXT_ID}"
HOSTNAME="${HOSTNAME:-genealogy}"
RAM="${RAM:-1024}"
CORES="${CORES:-2}"
DISK_SIZE="${DISK_SIZE:-8}"
BRIDGE="${BRIDGE:-vmbr0}"

# Auto-detect default storage pool supporting rootdir
STORAGE=""
for pool in "local-lvm" "local-zfs" "local"; do
    if pvesm status -storage "$pool" >/dev/null 2>&1; then
        STORAGE="$pool"
        break
    fi
done
STORAGE="${STORAGE:-local-lvm}"

# Interactive Confirmation if running in a TTY
if [ -t 0 ]; then
    echo -e "${YW}------------------------------------------------------------${CL}"
    echo -e "${BL}Container Configuration Defaults:${CL}"
    echo -e "  Container ID  : ${GN}${CT_ID}${CL}"
    echo -e "  Hostname      : ${GN}${HOSTNAME}${CL}"
    echo -e "  CPU Cores     : ${GN}${CORES}${CL}"
    echo -e "  Memory (MB)   : ${GN}${RAM}${CL}"
    echo -e "  Disk (GB)     : ${GN}${DISK_SIZE}${CL}"
    echo -e "  Storage Pool  : ${GN}${STORAGE}${CL}"
    echo -e "  Network Bridge: ${GN}${BRIDGE}${CL}"
    echo -e "${YW}------------------------------------------------------------${CL}"
    read -r -p "Proceed with these settings? [Y/n]: " CONFIRM || CONFIRM="y"
    if [[ "$CONFIRM" =~ ^[Nn] ]]; then
        read -r -p "Enter Container ID [${CT_ID}]: " INPUT_ID
        CT_ID="${INPUT_ID:-$CT_ID}"
        read -r -p "Enter Hostname [${HOSTNAME}]: " INPUT_HN
        HOSTNAME="${INPUT_HN:-$HOSTNAME}"
        read -r -p "Enter Memory in MB [${RAM}]: " INPUT_RAM
        RAM="${INPUT_RAM:-$RAM}"
        read -r -p "Enter Storage Pool [${STORAGE}]: " INPUT_ST
        STORAGE="${INPUT_ST:-$STORAGE}"
    fi
fi

# 3. Ensure Debian 12 Template
echo -e "${BL}[INFO] Checking Debian 12 LXC template...${CL}"
pveam update >/dev/null 2>&1 || true
TEMPLATE="$(pveam available -section system | grep -E 'debian-12-standard' | awk '{print $2}' | sort -V | tail -n 1 || true)"
if [ -z "$TEMPLATE" ]; then
    echo -e "${RD}[ERROR] Could not find a Debian 12 LXC template in 'pveam available'.${CL}" >&2
    exit 1
fi

TEMPLATE_STORAGE="local"
if ! pvesm status -storage local >/dev/null 2>&1; then
    TEMPLATE_STORAGE="$STORAGE"
fi

if ! pveam list "$TEMPLATE_STORAGE" | grep -q "$TEMPLATE"; then
    echo -e "${BL}[INFO] Downloading ${TEMPLATE} to ${TEMPLATE_STORAGE}...${CL}"
    pveam download "$TEMPLATE_STORAGE" "$TEMPLATE"
fi

# 4. Create Unprivileged Container
echo -e "${BL}[INFO] Creating LXC container ${CT_ID} (${HOSTNAME})...${CL}"
pct create "$CT_ID" "${TEMPLATE_STORAGE}:vztmpl/${TEMPLATE}" \
    -cores "$CORES" \
    -memory "$RAM" \
    -swap 512 \
    -hostname "$HOSTNAME" \
    -net0 "name=eth0,bridge=${BRIDGE},ip=dhcp" \
    -rootfs "${STORAGE}:${DISK_SIZE}" \
    -unprivileged 1 \
    -features nesting=1 \
    -onboot 1 \
    -start 0

echo -e "${GN}[OK] Container ${CT_ID} created successfully.${CL}"

# 5. Boot Container
echo -e "${BL}[INFO] Starting container ${CT_ID}...${CL}"
pct start "$CT_ID"

# Wait for container networking
echo -e "${BL}[INFO] Waiting for network connection inside container...${CL}"
for i in {1..30}; do
    if pct exec "$CT_ID" -- ping -c 1 -W 1 1.1.1.1 >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

# 6. Run In-Container Installer
echo -e "${BL}[INFO] Launching in-container installer...${CL}"
pct exec "$CT_ID" -- bash -c "$(curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/proxmox/install/genealogy-install.sh)"

# 7. Discover Container IP
IP="$(pct exec "$CT_ID" -- ip route get 1.1.1.1 2>/dev/null | awk '{print $7}' || true)"
if [ -z "$IP" ]; then
    IP="$(pct exec "$CT_ID" -- hostname -I 2>/dev/null | awk '{print $1}' || echo "container_ip")"
fi

echo ""
echo -e "${GN}==================================================================${CL}"
echo -e "${GN} Genealogy Family Tree LXC is online!${CL}"
echo -e "${GN}==================================================================${CL}"
echo -e " Container ID       : ${BL}${CT_ID}${CL}"
echo -e " Web Interface      : ${YW}http://${IP}:80${CL}"
echo -e " Container Console  : ${BL}pct enter ${CT_ID}${CL}"
echo -e " Future Updates     : Type ${BL}update${CL} inside container console"
echo -e "${GN}==================================================================${CL}"

