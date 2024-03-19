#!/bin/bash
trap "kill 0" EXIT

export RUST_BACKTRACE=full

touch /tmp/smartos_{ui,executor}.log

CERT_FILE=/tmp/smartos_ui_cert.pem
KEY_FILE=/tmp/smartos_ui_key.pem

if [ ! -f "$CERT_FILE" ]; then
  echo "Generating TLS Certificate"
  openssl req -x509 -nodes -subj '/CN=*' \
      -newkey rsa:4096 -sha256 -days 365 \
      -keyout "$KEY_FILE" \
      -out "$CERT_FILE"
fi

./test/mock/vminfod &

LOG_FILE=/tmp/smartos_executor.log \
  SHADOW_PATH=test/data/shadow \
  GZ_CONFIG_PATH=test/data/config \
  PATH=test/mock:$PATH \
	./target/debug/smartos_executor &

LOG_FILE=/tmp/smartos_ui.log \
  CERT_FILE="$CERT_FILE" \
  KEY_FILE="$KEY_FILE" \
	./target/debug/smartos_ui &

tail -f /tmp/smartos_{ui,executor}.log | ./ui/assets/node_modules/.bin/bunyan