#!/bin/bash

#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#

#
# Copyright 2024 MNX Cloud, Inc.
#

set -o errexit
set -o pipefail

while getopts "b:n" options; do
   case $options in
      b ) BRANCH="${OPTARG}";;
      n ) NOT_REALLY=1;;
      * ) usage ;;
   esac
done

MANTA_URL=https://us-central.manta.mnx.io
LATEST_URI="/Joyent_Dev/public/builds/smartos-ui/${BRANCH:=master}-latest"
FNAME=smartos-ui/uiadm.sh
installer=$(mktemp /tmp/ui-installer.XXXXXX)

latest=$(curl -sf "${MANTA_URL}/${LATEST_URI}")

if (( NOT_REALLY == 1 )); then
    echo "${MANTA_URL}${latest}/${FNAME}"
    exit
fi

curl -L -o "${installer}" -sf "${MANTA_URL}${latest}/${FNAME}"

chmod +x "${installer}"
${installer} install -b "${BRANCH}" latest
