#!/usr/bin/env bash

# ==============================================================================
# Genealogy Family Tree — Proxmox VE In-Container Installation Routine
# Compatible with community-scripts/ProxmoxVE (install/genealogy-install.sh)
# Target OS: Debian 12 (Bookworm) / Ubuntu 24.04 (LXC Container)
# ==============================================================================

# Sourcing helper library if provided by the community-scripts host framework
if [[ -n "${FUNCTIONS_FILE_PATH:-}" ]]; then
  source /dev/stdin <<< "$FUNCTIONS_FILE_PATH"
elif curl -fsSL -o /tmp/build.func https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/build.func 2>/dev/null; then
  source /tmp/build.func
  rm -f /tmp/build.func
else
  # Portable standalone fallback definitions
  color() { true; }
  verb_ip6() { true; }
  catch_errors() { set -euo pipefail; }
  setting_up_container() { echo -e "\e[34m[INFO]\e[0m Setting up container..."; }
  network_check() { true; }
  update_os() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq && apt-get upgrade -y -qq
  }
  msg_info() { echo -e "\e[34m[INFO]\e[0m $*"; }
  msg_ok() { echo -e "\e[32m[OK]\e[0m $*"; }
  msg_error() { echo -e "\e[31m[ERROR]\e[0m $*" >&2; }
  motd_ssh() { true; }
  customize() { true; }
fi

color
verb_ip6
catch_errors
setting_up_container
network_check
update_os

msg_info "Installing Dependencies (PostgreSQL 16, curl, jq, openssl)"
export DEBIAN_FRONTEND=noninteractive
apt-get install -y -qq curl tar gzip ca-certificates jq openssl postgresql postgresql-contrib

# Mark critical packages as manual to safeguard against aggressive autoremove sweeps (e.g. clean-lxcs)
if command -v apt-mark >/dev/null 2>&1; then
  apt-mark manual curl tar gzip ca-certificates jq openssl postgresql postgresql-contrib >/dev/null 2>&1 || true
fi
msg_ok "Installed Dependencies"

msg_info "Deploying Genealogy Family Tree (Port 80 direct listening)"
curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/install.sh | bash
msg_ok "Deployed Genealogy Family Tree"

motd_ssh
customize

msg_info "Cleaning up temporary files"
apt-get -y autoremove >/dev/null 2>&1 || true
apt-get -y clean >/dev/null 2>&1 || true
rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
msg_ok "Cleaned up"

