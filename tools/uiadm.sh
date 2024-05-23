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

#
# If TRACE is set in the environment, enable xtrace.  Additionally,
# assuming the current shell is bash version 4.1 or later, more advanced
# tracing output will be emitted and some additional features may be used:
#
#   TRACE_LOG   Send xtrace output to this file instead of stderr.
#   TRACE_FD    Send xtrace output to this fd instead of stderr.
#               The file descriptor must be open before the shell
#               script is started.
#
if [[ -n ${TRACE} ]]; then
    if [[ ${BASH_VERSINFO[0]} -ge 4 && ${BASH_VERSINFO[1]} -ge 1 ]]; then
        PS4=
        PS4="${PS4}"'[\D{%FT%TZ}] ${BASH_SOURCE}:${LINENO}: '
        PS4="${PS4}"'${FUNCNAME[0]:+${FUNCNAME[0]}(): }'
        export PS4
        if [[ -n ${TRACE_LOG} ]]; then
            exec 4>>"${TRACE_LOG}"
            export BASH_XTRACEFD=4
        elif [[ -n ${TRACE_FD} ]]; then
            export BASH_XTRACEFD=${TRACE_FD}
        fi
    fi
    set -o xtrace
fi

# Default well-known source of SmartOS UI Images
URL_PREFIX=https://us-central.manta.mnx.io/Joyent_Dev/public/builds/smartos-ui
INSTALL_PREFIX=/opt/smartos/ui

EXECUTOR_FMRI=svc:/system/smartdc/smartos-ui-executor
EXECUTOR_MANIFEST=/var/svc/manifest/site/smartos-ui-executor.xml
UI_FMRI=svc:/system/smartdc/smartos-ui
UI_MANIFEST=/var/svc/manifest/site/smartos-ui.xml

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
# shellcheck disable=SC1091
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

vtar() {
	if [[ $VERBOSE -gt 0 ]]; then
		gtar -v "$@"
	else
		gtar "$@"
	fi
}

info() {
	ui="$INSTALL_PREFIX/bin/ui"
	if [[ -f "$ui" ]]; then
		echo "Version: $("$ui" version) $("$ui" stamp)"
		# shellcheck disable=SC2154
		echo "URL: https://$Admin_IP:4443"
	else
		echo "Not currently installed".
	fi
}

get_avail_versions() {
	branch="$1"
	ui="$INSTALL_PREFIX/bin/ui"
	if [[ -f "$ui" ]]; then
		stamp="$("$ui" stamp)"
		marker="&marker=${stamp}"
		vcurl "${URL_PREFIX}/?limit=1000$marker" | json -ga name |\
			grep "$branch" | grep -v "$stamp" | grep -v latest
	else
		vcurl "${URL_PREFIX}/?limit=1000" | json -ga name |\
			grep "$branch" | grep -v latest
	fi
}

install() {

	OPTIND=1
	while getopts "b:" options; do
	   case $options in
	      b ) BRANCH="${OPTARG}";;
	      * ) usage ;;
	   esac
	done

	shift $(( OPTIND -1 ))

	echo "1 is $1"
	echo "2 is $2"

	# Accept an alternate installation root
	root="/${2}"

	if [[ -z "$1" ]]; then
		err "Either a version or 'latest' must be provided"
	elif [[ "$1" == "latest" ]]; then
		ui="$INSTALL_PREFIX/bin/ui"
		version=$(basename "$(vcurl "${URL_PREFIX}/${BRANCH:=master}-latest")")
		if [[ ${root} == '/' ]] && [[ -f "$ui" ]] && \
		    [[ "$("$ui" stamp)" == "$version" ]]; then
			err "Latest version is already installed: $version"
		fi
		echo "Installing latest version: $version"
	else
		version="$1"
	fi

	# # Get the minimium supported PI from the Manta object's metadata
	# minimium_pi=$("${CURL[@]}" -I -o /dev/null -w '%header{m-minimum-pi}' \
	#   "${URL_PREFIX}/smartos-ui-$version.tar.gz")

	# if [[ "${Live_Image:?}" < "$minimium_pi" ]]; then
	# 	err "$version requires a platform image of $minimium_pi or newer"
	# fi

	TAR_PATH="${version}/smartos-ui/smartos-ui-${version}.tar.gz"

	"${VCURL[@]}" "${URL_PREFIX}/${TAR_PATH}" | \
		vtar --strip-components=1 -xzpf - -C "$root"

	cert_install_prefix="${root}usbkey/tls"
	cert_file="$cert_install_prefix/smartos_ui_cert.pem"
	key_file="$cert_install_prefix/smartos_ui_key.pem"

	if [ ! -f "$cert_file" ] || [ ! -f "$key_file" ]; then
		echo "Generating TLS key and certificate"
		vecho "Writing Key to: $key_file"
		vecho "Writing certificate to: $cert_file"

		[[ -d "$cert_install_prefix" ]] || mkdir -p "$cert_install_prefix"

		openssl req -x509 -nodes -subj '/CN=*' \
		  -newkey rsa:4096 -sha256 -days 365 \
		  -keyout "$key_file" \
		  -out "$cert_file"
	fi

	# Don't install services if an alternate root was provided.
	if [[ ${root} == '/' ]]; then
		install_services
		echo "Service running at https://$Admin_IP:4443"
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

while getopts "v" options; do
   case $options in
      v ) VERBOSE=1;;
      * ) usage ;;
   esac
done

shift $(( OPTIND-1 ))

cmd=$1
shift 1

case $cmd in

	avail )
		OPTIND=1
		while getopts "b:" options; do
		   case $options in
		      b ) BRANCH="${OPTARG}";;
		      * ) usage ;;
		   esac
		done

		get_avail_versions "${BRANCH:=master}"
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
