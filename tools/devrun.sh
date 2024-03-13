#!/bin/bash
trap "kill 0" EXIT

export RUST_BACKTRACE=full

touch /tmp/smartos_{ui,executor}.log

./test/mock/vminfod &

LOG_FILE=/tmp/smartos_executor.log \
  SHADOW_PATH=test/data/shadow \
  GZ_CONFIG_PATH=test/data/config \
  PATH=test/mock:$PATH \
	./target/debug/smartos_executor &

LOG_FILE=/tmp/smartos_ui.log \
	./target/debug/smartos_ui &

tail -f /tmp/smartos_{ui,executor}.log | ./ui/assets/node_modules/.bin/bunyan