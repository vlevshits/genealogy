# Proxmox VE Deployment & Community Scripts Integration

This directory contains integration scripts for deploying **Genealogy Family Tree** on **Proxmox VE (PVE)** as a lightweight, unprivileged Debian 12 LXC container.

---

## 1. Quick Start: Deploy directly on Proxmox VE Host

Run the one-liner directly in your Proxmox VE node's root shell:

```bash
bash -c "$(curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/proxmox/install-pve.sh)"
```

### What this does:
1. Provisions an **unprivileged Debian 12 LXC container** with optimal defaults:
   - **CPU**: 2 vCPUs
   - **RAM**: 1024 MiB (1 GB)
   - **Disk**: 8 GB storage
   - **Nesting**: Enabled
2. Installs and configures **PostgreSQL 16** with a dedicated role and database.
3. Deploys the latest precompiled Genealogy release bundle (Rust Axum + React 19 SPA).
4. Configures systemd unit with `CAP_NET_BIND_SERVICE` for direct native listening on **port 80** (zero Nginx or proxy layer).
5. Registers the container `update` command.

---

## 2. Proxmox VE Community Scripts Compatibility

The files in this directory follow the exact structure required by the [community-scripts/ProxmoxVE](https://github.com/community-scripts/ProxmoxVE) project:

| File | Target in `community-scripts/ProxmoxVE` | Description |
| :--- | :--- | :--- |
| `ct/genealogy.sh` | `ct/genealogy.sh` | Host script handling UI dialogs, resource settings, container creation, and update hooks. |
| `install/genealogy-install.sh` | `install/genealogy-install.sh` | In-container script handling OS package provisioning and application installation. |
| `json/genealogy.json` | `json/genealogy.json` | Catalog entry metadata for [community-scripts.github.io/ProxmoxVE](https://community-scripts.github.io/ProxmoxVE). |

### Contributing to Community Scripts Upstream
To submit Genealogy to the official Proxmox VE Helper-Scripts repository:
1. Fork `community-scripts/ProxmoxVE`.
2. Copy:
   - `scripts/proxmox/ct/genealogy.sh` → `ct/genealogy.sh`
   - `scripts/proxmox/install/genealogy-install.sh` → `install/genealogy-install.sh`
   - `scripts/proxmox/json/genealogy.json` → `json/genealogy.json`
3. Test locally using their contribution guidelines and submit a Pull Request.

---

## 3. Compatibility with `clean-lxcs` Maintenance Utility

The [PVE LXC Cleaner (`clean-lxcs.sh`)](https://community-scripts.github.io/ProxmoxVE/scripts?id=clean-lxcs) is a popular maintenance script that cleans containers by:
- Running `apt-get -y --purge autoremove`
- Emptying `/var/lib/apt/lists/*` and `/var/cache/*`
- Deleting `/tmp/*` and `/var/tmp/*`
- Truncating rotated log files

### How Genealogy is Hardened Against `clean-lxcs`:
- **Manual Package Marking**: All core packages (`postgresql`, `postgresql-contrib`, `curl`, `jq`, `openssl`, `ca-certificates`) are explicitly marked as `manual` via `apt-mark manual`. `clean-lxcs` will **never** accidentally remove PostgreSQL or libraries.
- **Zero `/tmp` Runtime Dependency**: All persistent data (`/var/lib/genealogy`), database files (`/var/lib/postgresql`), and configs (`/etc/genealogy`) live in permanent system paths. The temporary extraction directory is discarded immediately during installation.
- **Systemd Journal Logging**: Logging is managed via `journald` (`journalctl -u genealogy`), preventing unhandled log rotation failures.
- **Socket Isolation**: PostgreSQL uses `/var/run/postgresql` (managed by `systemd-tmpfiles`) or TCP `127.0.0.1:5432`, immune to `/tmp` sweeps.

---

## 4. In-Container Updates

To update an existing Genealogy container to the latest version at any time:
1. Open the container console (`pct enter <CTID>` or via Proxmox web UI).
2. Simply run:
   ```bash
   update
   ```
   *(Or: `sudo /opt/genealogy/scripts/install.sh`)*

