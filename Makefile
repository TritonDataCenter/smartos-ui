#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#

#
# Copyright 2020 Joyent, Inc.
# Copyright 2024 MNX Cloud, Inc.
#

NAME = smartos_ui
RUST_TOOLCHAIN = 1.73.0

ENGBLD_USE_BUILDIMAGE = false
ENGBLD_BITS_UPLOAD_IMGAPI = false

ENGBLD_REQUIRE := $(shell git submodule update --init deps/eng)

include ./deps/eng/tools/mk/Makefile.defs
include ./deps/eng/tools/mk/Makefile.rust.defs

BUILD_PLATFORM = 20210826T002459Z

J2_FILES ?= $(shell find $(TOP)/ui/templates -name *.j2)

ui/assets/node_modules:
	cd ui/assets && npm install

ui/assets/main.css: ui/assets/main.in.css ui/assets/tailwind.config.js $(J2_FILES)
	cd ui/assets && \
		./node_modules/.bin/tailwindcss -m -i ./main.in.css -o ./main.css

ui/assets/main.css.gz: ui/assets/main.css
	cd ui/assets && rm -f ./main.css.gz && gzip ./main.css

ui/assets/main.js: ui/assets/node_modules
	cd ui/assets && \
	./node_modules/.bin/rollup main.in.js \
		--silent \
		--format es \
		--file main.js \
		--plugin @rollup/plugin-node-resolve

ui/assets/provision.js: ui/assets/node_modules
	cd ui/assets && \
	./node_modules/.bin/rollup provision.in.js \
		--silent \
		--format es \
		--file provision.js \
		--plugin @rollup/plugin-node-resolve

ui/assets/main.js.gz: ui/assets/main.js
	cd ui/assets && rm -f ./main.js.gz && gzip ./main.js

ui/assets/provision.js.gz: ui/assets/provision.js
	cd ui/assets && rm -f ./provision.js.gz && gzip ./provision.js

.PHONY: assets
assets: ui/assets/node_modules ui/assets/main.css.gz ui/assets/main.js.gz ui/assets/provision.js.gz

.PHONY: clean-assets
clean-assets:
	rm -f ui/assets/*.gz

.PHONY: js-fmt
fmt-js:
	cd ui/assets && npm run fmt

.PHONY: all
all: release

.PHONY: release
release: assets $(RS_FILES) | $(CARGO_EXEC)
	$(CARGO) build --release

.PHONY: debug
debug: assets $(RS_FILES) | $(CARGO_EXEC)
	$(CARGO) build

.PHONY: fmt
fmt: | $(CARGO_EXEC)
	$(CARGO) fmt

.PHONY: license-check
license-check: | $(CARGO_EXEC)
	$(CARGO) install cargo-license
	$(CARGO) license

.PHONY: update
update: | $(CARGO_EXEC)
	$(CARGO) update

.PHONY: devrun
devrun: debug
	./tools/devrun.sh

include ./deps/eng/tools/mk/Makefile.deps
include ./deps/eng/tools/mk/Makefile.targ
include ./deps/eng/tools/mk/Makefile.rust.targ

