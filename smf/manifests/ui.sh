#!/bin/bash

# TODO: When the UI starts up, get the admin IP from sysinfo via the executor
[[ -f /tmp/.sysinfo.parsable ]] || sysinfo -u
source /tmp/.sysinfo.parsable
ip="$Admin_IP"

CERT_FILE=/usbkey/tls/smartos_ui_cert.pem
KEY_FILE=/usbkey/tls/smartos_ui_key.pem

if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
  echo "Generating TLS Certificate"
  openssl req -x509 -nodes -subj '/CN=*' \
      -newkey rsa:4096 -sha256 -days 365 \
      -keyout "$KEY_FILE" \
      -out "$CERT_FILE"
fi

UI_BIND_HTTP_ADDRESS=$ip:8880 UI_BIND_HTTPS_ADDRESS=$ip:4443 ./bin/ui
