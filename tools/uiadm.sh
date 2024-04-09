#!/bin/bash
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#

#
# Copyright 2024 MNX Cloud, Inc.
#

set -o pipefail

# Default well-known source of SmartOS UI Images
URL_PREFIX=https://us-central.manta.mnx.io/tpaul/public/smartos_ui/release
#URL_PREFIX=https://us-central.manta.mnx.io/Joyent_Dev/public/SmartOS-UI
INSTALL_PREFIX=/opt/smartos/ui

EXECUTOR_FMRI=svc:/system/smartdc/smartos-ui-executor
EXECUTOR_MANIFEST=/var/svc/manifest/site/smartos-ui-executor.xml
UI_FMRI=svc:/system/smartdc/smartos-ui
UI_MANIFEST=/var/svc/manifest/site/smartos-ui.xml

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

# Only run in the global zone.
[[ "$(zonename)" == "global" ]] || err "Must run uiadm in the global zone"

# Only run as root.
[[ "$(id -u)" == 0 ]] || err "Must run uiadm as root"

# Don't allow running on a CN or HN
bootparams | grep -q "smartos=" || err "Must run on stand-alone SmartOS"

[[ -f /tmp/.sysinfo.parsable ]] || sysinfo -u
source /tmp/.sysinfo.parsable

usage() {
	eecho ""
	eecho "Usage: uiadm [-v] <command> [command-specific arguments]"
	eecho ""
	eecho "    uiadm avail"
	eecho "    uiadm install [version]"
	eecho "    uiadm info"
	eecho "    uiadm remove"
	err ""
}

# Defined as a variable in case we need to add parameters (like -o) to it.
CURL=(curl -k -s -f)
VCURL=(curl -k -f --progress-bar)

vcurl() {
	if [[ $VERBOSE -gt 0 ]]; then
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

info() {
	ui="$INSTALL_PREFIX/bin/ui"
	if [[ -f "$ui" ]]; then
		echo "Version: $("$ui" version)"
		echo "URL: https://$Admin_IP:4443"
	else
		echo "Not currently installed".	
	fi
}

get_avail_versions() {
	vcurl "${URL_PREFIX}/?limit=1000" | \
	  json -ga name | \
	  sed -e 's/^smartos\-ui\-//' -e 's/\.tar\.gz$//' | \
	  sort -t '.' -n -k1,1 -k2,2 -k3,3
}

avail() {
	ui="$INSTALL_PREFIX/bin/ui"
	if [[ -f "$ui" ]]; then
		version="$("$ui" version)"
		get_avail_versions | grep -v "$version"
	else
		get_avail_versions
	fi
}

install() {
	current_pi=$(sysinfo | json "Live Image")
	if [[ -z "$1" ]]; then
		err "Either a version or 'latest' must be provided"
	elif [[ "$1" == "latest" ]]; then
		ui="$INSTALL_PREFIX/bin/ui"
		version="$(get_avail_versions | tail -n1)"
		if [[ -f "$ui" ]] && [[ "$("$ui" version)" == "$version" ]]; then
			err "Latest version is already installed: $version"
		fi
		echo "Installing latest version: $version"
	else
		version="$1"
	fi

	minimium_pi=$("${CURL[@]}" -I -o /dev/null -w '%header{m-minimum-pi}' \
	  "${URL_PREFIX}/smartos-ui-$version.tar.gz")

	if [[ "$current_pi" < "$minimium_pi" ]]; then
		err "Version $version requires a platform image of $minimium_pi or newer"
	fi

	"${VCURL[@]}" "${URL_PREFIX}/smartos-ui-$version.tar.gz" | \
		gtar --strip-components=1 -xzf - -C /

	generate_cert

	install_services

	echo "Service running at https://$Admin_IP:4443"
}

vsvcadm() {
	if [[ $VERBOSE -gt 0 ]]; then
		svcadm -v "$@"
	else
		svcadm "$@"
	fi
}

vsvccfg() {
	if [[ $VERBOSE -gt 0 ]]; then
		svccfg -v "$@"
	else
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
	vecho "Checking if UI service is installed"
	if svcs -H -o state "$UI_FMRI" &> /dev/null; then
		vsvccfg import "$UI_MANIFEST"
		vsvcadm restart "$UI_FMRI"
	else
		vsvccfg import "$UI_MANIFEST"
	fi

	vecho "Checking if UI Exectuor service is installed"
	if svcs -H -o state "$EXECUTOR_FMRI" &> /dev/null; then
		vsvccfg import "$EXECUTOR_MANIFEST"
		vsvcadm restart "$EXECUTOR_FMRI"
	else
		vsvccfg import "$EXECUTOR_MANIFEST"
	fi
}

remove() {
	remove_services
	rm -rf "$INSTALL_PREFIX"
	rm -f "$EXECUTOR_MANIFEST" "$UI_MANIFEST"

	# TODO remove this before release. SMF files have been moved around a bit
	# during the beta tests.
	rm -f /var/svc/smartos-ui*.xml /opt/custom/smf/smartos-ui*.xml
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

	info )
		info "$@"
		;;

	remove )
		remove "$@"
		;;

	*)
		usage
		;;

esac

exit 0
