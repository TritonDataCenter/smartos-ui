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
URL_PREFIX=https://us-central.manta.mnx.io/tpaul/public/smartos_ui/release
#URL_PREFIX=https://us-central.manta.mnx.io/Joyent_Dev/public/SmartOS-UI
INSTALL_PREFIX=/opt/smartos/ui

EXECUTOR_FMRI=svc:/system/smartdc/smartos-ui-executor
EXECUTOR_MANIFEST=/opt/custom/smartos-ui-executor.xml
UI_FMRI=svc:/system/smartdc/smartos-ui
UI_MANIFEST=/opt/custom/smartos-ui.xml

CERT_INSTALL_PREFIX=/usbkey/tls
CERT_FILE=$CERT_INSTALL_PREFIX/smartos_ui_cert.pem
KEY_FILE=$CERT_INSTALL_PREFIX/smartos_ui_key.pem

eecho() {
	echo "$@" 1>&2
}

vecho() {
	if [[ $VERBOSE -eq 1 ]]; then
		# Verbose echoes invoked by -v go to stdout, not stderr.
		echo "$@"
	fi
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

# Don't allow running on a CN or HN
bootparams | grep -q "smartos=" || err "Must run on stand-alone SmartOS"

[[ -f /tmp/.sysinfo.parsable ]] || sysinfo -u
source /tmp/.sysinfo.parsable
ip="$Admin_IP"

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

generate_cert() {
	if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
		echo "Generating TLS Certificate"

		[[ -d "$CERT_INSTALL_PREFIX" ]] || mkdir -p "$CERT_INSTALL_PREFIX"

		openssl req -x509 -nodes -subj '/CN=*' \
		  -newkey rsa:4096 -sha256 -days 365 \
		  -keyout "$KEY_FILE" \
		  -out "$CERT_FILE"
	fi
}

avail() {
	ui="$INSTALL_PREFIX/bin/ui"
	if [[ -f "$ui" ]]; then
		version="$("$ui" version)"
		vcurl "${URL_PREFIX}/?limit=1000" | \
		  json -ga name | grep -v "$version" | \
		  sed -e 's/^smartos\-ui\-//' -e 's/\.tar\.gz$//'
	else
		vcurl "${URL_PREFIX}/?limit=1000" | json -ga name | \
		  sed -e 's/^smartos\-ui\-//' -e 's/\.tar\.gz$//'
	fi
}

install() {
	vecho "${URL_PREFIX}/smartos-ui-$1.tar.gz"
	"${VCURL[@]}" "${URL_PREFIX}/smartos-ui-$1.tar.gz" | \
		gtar --strip-components=1 -xzf - -C /
	generate_cert
	remove_services
	install_services
	# TODO: Poll /ping to wait for services to come up
	echo "Service running at https://$Admin_IP:4443"
}

vsvcadm() {
	if [[ $VERBOSE -eq 1 ]]; then
		# Verbose 
		svcadm -v "$@"
	else
		# Non-verbose
		svcadm "$@"
	fi
}

vsvccfg() {
	if [[ $VERBOSE -eq 1 ]]; then
		# Verbose 
		svccfg -v "$@"
	else
		# Non-verbose
		svccfg "$@"
	fi
}

remove_services() {
	vecho "Checking if UI service needs to be removed"
	if svcs -H -o state "$UI_FMRI" &> /dev/null; then
		vsvcadm disable "$UI_FMRI"
		vsvccfg delete -f "$UI_FMRI"
	fi

	vecho "Checking if UI Exectuor service needs to be removed"
	if svcs -H -o state "$EXECUTOR_FMRI" &> /dev/null; then
		vsvcadm disable "$EXECUTOR_FMRI"
		vsvccfg delete -f "$EXECUTOR_FMRI"
	fi
}

install_services() {
	vsvccfg import "$EXECUTOR_MANIFEST"
	vsvccfg import "$UI_MANIFEST"
}

remove() {
	remove_services
	rm -rf "$INSTALL_PREFIX"
	rm -f "$EXECUTOR_MANIFEST" "$UI_MANIFEST"
	# TODO:
	# Clean up logs?
	# Prompt to remove certs?
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
