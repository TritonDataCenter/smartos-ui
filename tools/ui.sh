#!/bin/bash
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#

#
# Copyright 2024 MNX Cloud, Inc.
#

[[ -f /tmp/.sysinfo.parsable ]] || sysinfo -u
source /tmp/.sysinfo.parsable
ip="$Admin_IP"

UI_BIND_HTTP_ADDRESS=$ip:8880 UI_BIND_HTTPS_ADDRESS=$ip:4443 ./bin/ui
