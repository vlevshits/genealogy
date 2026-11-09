#!/usr/bin/env bash
# Entrypoint for Proxmox VE Community Scripts workflow
# Redirects to ct/genealogy.sh or provides fallback
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/ct/genealogy.sh" ]]; then
  exec bash "${SCRIPT_DIR}/ct/genealogy.sh" "$@"
else
  exec bash <(curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/proxmox/ct/genealogy.sh) "$@"
fi

