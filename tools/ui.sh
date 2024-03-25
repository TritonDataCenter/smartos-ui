#!/bin/bash

# TODO: When the UI starts up, get the admin IP from sysinfo via the executor
# and use it to populate the IP in UI_BIND_HTTP*_ADDRESS
[[ -f /tmp/.sysinfo.parsable ]] || sysinfo -u
source /tmp/.sysinfo.parsable
ip="$Admin_IP"

UI_BIND_HTTP_ADDRESS=$ip:8880 UI_BIND_HTTPS_ADDRESS=$ip:4443 ./bin/ui
