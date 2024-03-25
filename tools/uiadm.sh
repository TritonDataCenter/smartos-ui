#!/bin/bash

#
# This file and its contents are supplied under the terms of the
# Common Development and Distribution License ("CDDL"), version 1.0.
# You may only use this file in accordance with the terms of version
# 1.0 of the CDDL.
#
# A full copy of the text of the CDDL should have accompanied this
# source.  A copy of the CDDL is also available via the Internet at
# http://www.illumos.org/license/CDDL.
#

#
# Copyright 2024 MNX Cloud, Inc.
#

# Default well-known source of SmartOS UI Images
URL_PREFIX=https://us-central.manta.mnx.io/tpaul/public/smartos_ui/release/
#URL_PREFIX=https://us-central.manta.mnx.io/Joyent_Dev/public/SmartOS-UI/
INSTALL_PREFIX=/opt/smartos/ui
EXECUTOR_FMRI=system/smartdc/smartos-ui-executor
EXECUTOR_MANIFEST=$INSTALL_PREFIX/smf/manifests/executor.xml
UI_FMRI=svc://system/smartdc/smartos-ui
UI_MANIFEST=$INSTALL_PREFIX/smf/manifests/ui.xml


eecho() {
	echo "$@" 1>&2
}

err() {
	eecho "$@"
	exit 1
}

fatal() {
	eecho
	if [[ -n "$1" ]]; then
		eecho "ERROR: $1"
	fi
	eecho
	exit 2
}


# Only run in the global zone.
[[ "$(zonename)" == "global" ]] || err "Must run uiadm in the global zone"

# Only run as root.
[[ "$(id -u)" == 0 ]] || err "Must run uiadm as root"

# TODO: Don't allow running on a CN or HN?
# Determine if we're running on a Triton Compute Node (CN) or not:
# sysinfo setup should be false?
# bootparams | grep -E -q 'smartos=|headnode=' || initialize_as_CN
# bootparams | grep -q 'headnode=' && initialize_as_HN

usage() {
	eecho ""
	eecho "Usage: uiadm [-v] <command> [command-specific arguments]"
	eecho ""
	eecho "    uiadm avail"
	eecho "    uiadm install [version]"
	eecho "    uiadm remove"
	err ""
}

# Defined as a variable in case we need to add parameters (like -o) to it.
CURL=(curl -s -f)
VCURL=(curl -f --progress-bar)

vcurl() {
	if [[ $VERBOSE -eq 1 ]]; then
		# Verbose curls show progress.
		"${VCURL[@]}" "$@"
	else
		# Non-verbose ones do not.
		"${CURL[@]}" "$@"
	fi
}

avail() {
	version_file="$INSTALL_PREFIX/version"
	fetch_avail="$CURL ${URL_PREFIX}/?limit=1000"
	if [[ -f "$version_file" ]]; then
		version="$(< "$version_file")"
		vcurl "${URL_PREFIX}/?limit=1000" | \
			json -ga -c name | grep -v "$version"
	else
		vcurl "${URL_PREFIX}/?limit=1000" | json -ga -c name
	fi
}

install() {

}

remove() {

}

if [[ "$1" == "-v" ]]; then
	VERBOSE=1
	shift 1
elif [[ "$1" == "-vv" ]]; then
	set -x
	VERBOSE=1
	shift 1
else
	VERBOSE=0
fi

cmd=$1
shift 1

case $cmd in

	avail )
		avail
		;;

	install )
		install "$@"
		;;

	remove )
		remove "$@"
		;;

	*)
		usage
		;;

esac

exit 0
