#!/usr/bin/env bash
source <(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/build.func)
# Copyright (c) 2024-2026 Vladimir Levshits
# License: MIT | https://github.com/community-scripts/ProxmoxVE/raw/main/LICENSE
# Source: https://github.com/vlevshits/genealogy

APP="Genealogy"
var_tags="genealogy;family-tree"
var_cpu="2"
var_ram="1024"
var_disk="8"
var_os="debian"
var_version="12"
var_unprivileged="1"

header_info "$APP"

variables
color
catch_errors

function update_script() {
    header_info
    check_container_storage
    check_container_resources
    if [[ ! -f /etc/systemd/system/genealogy.service ]]; then
        msg_error "No ${APP} Installation Found!"
        exit
    fi
    msg_info "Updating ${APP}"
    if [[ -x /usr/local/bin/update ]]; then
        /usr/local/bin/update
    elif [[ -f /opt/genealogy/scripts/install.sh ]]; then
        /opt/genealogy/scripts/install.sh
    else
        curl -fsSL https://raw.githubusercontent.com/vlevshits/genealogy/main/scripts/install.sh | bash
    fi
    msg_ok "Updated ${APP}"
    exit
}

start
build_container
description

msg_ok "Completed Successfully!\n"
echo -e "${CREATING}${GN}${APP} setup has been successfully completed!${CL}"
echo -e "${INFO}${YW} Access it at: ${BL}http://${IP}:80${CL}"

