#!/bin/bash

trap "kill 0" EXIT

mkdir -p /tmp/smartos_ui/chroot
touch /tmp/smartos_ui/smartos_{ui,executor}.log

# Get NIC with the admin tag
nic_if=$(nictagadm list -p -d, | grep '^admin' | awk -F, '{print $3}')

# Get IP address (This is needed for working redirect from http to https)
ip=$(ipadm show-addr -p -o addr "$nic_if/_a" | awk -F/ '{print $1}')

CERT_FILE=/tmp/smartos_ui/cert.pem
KEY_FILE=/tmp/smartos_ui/key.pem

if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
  echo "Generating TLS Certificate"
  openssl req -x509 -nodes -subj '/CN=*' \
      -newkey rsa:4096 -sha256 -days 365 \
      -keyout "$KEY_FILE" \
      -out "$CERT_FILE"
fi

LOG_FILE=/tmp/smartos_ui/smartos_executor.log \
	./smartos_executor &

LOG_FILE=/tmp/smartos_ui/smartos_ui.log \
	CERT_FILE="$CERT_FILE" \
	KEY_FILE="$KEY_FILE" \
	UI_BIND_HTTP_ADDRESS=0.0.0.0:8080 \
	UI_BIND_HTTPS_ADDRESS=$ip:4443 \
	CHROOT=/tmp/smartos_ui/chroot \
	./smartos_ui &

tail -f /tmp/smartos_ui/smartos_{ui,executor}.log | bunyan