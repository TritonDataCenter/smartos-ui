#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#

#
# Copyright 2020 Joyent, Inc.
# Copyright 2024 MNX Cloud, Inc.
#

NAME = smartos-ui
RUST_TOOLCHAIN = 1.75.0

ENGBLD_USE_BUILDIMAGE = false
ENGBLD_BITS_UPLOAD_IMGAPI = false

ENGBLD_REQUIRE := $(shell git submodule update --init deps/eng)

include ./deps/eng/tools/mk/Makefile.defs
include ./deps/eng/tools/mk/Makefile.rust.defs

BUILD_PLATFORM = 20210826T002459Z
RELEASE_TARBALL :=	$(NAME)-pkg-$(STAMP).tar.gz
RELSTAGEDIR :=		/tmp/$(NAME)-$(STAMP)

SMF_MANIFESTS =	smf/manifests/ui.xml smf/manifests/executor.xml

J2_FILES ?= $(shell find $(TOP)/ui/templates -name *.j2)
JS_FILES ?= $(wildcard $(TOP)/ui/assets/*.js)

# Playwright is a dev dependency, and not needed to build css/js assets
ui/assets/node_modules/@playwright/package.json:
	cd ui/assets && npm install

ui/assets/node_modules: ui/assets/package.json ui/assets/package-lock.json
	cd ui/assets && npm install --omit=dev

ui/assets/main.css: ui/assets/main.in.css ui/assets/tailwind.config.js $(J2_FILES)
	cd ui/assets && \
		./node_modules/.bin/tailwindcss -m -i ./main.in.css -o ./main.css && \
		gsed -i -e 's/\/\*\#\ sourceMappingURL=main.css.map\ \*\///' ./main.css

ui/assets/main.css.gz: ui/assets/node_modules ui/assets/main.css
	cd ui/assets && rm -f ./main.css.gz && gzip ./main.css

ui/assets/main.js: ui/assets/node_modules $(JS_FILES)
	cd ui/assets && \
	./node_modules/.bin/esbuild main.in.js \
		--bundle \
		--format=esm \
		--outfile=main.js

ui/assets/main.js.gz: ui/assets/main.js
	cd ui/assets && rm -f main.js.gz && gzip ./main.js

.PHONY: assets
assets: ui/assets/main.css.gz ui/assets/main.js.gz

.PHONY: clean
clean:: clean-assets

.PHONY: clean-assets
clean-assets:
	rm -f ui/assets/*.gz ui/assets/main.js \
		ui/assets/main.css ui/assets/main.css.map

.PHONY: clean-mock-db
clean-mock-db:
	rm -f test/data/db/{img,vm}/*.json

.PHONY: fmt-js
fmt-js:
	cd ui/assets && npm run fmt

.PHONY: all
all: release_build

.PHONY: release_build
release_build: assets $(RS_FILES) | $(CARGO_EXEC)
	$(CARGO) build --release

.PHONY: debug
debug: assets $(RS_FILES) | $(CARGO_EXEC)
	$(CARGO) build

.PHONY: fmt
fmt: | $(CARGO_EXEC)
	$(CARGO) fmt

.PHONY: clippy
clippy: | $(CARGO_EXEC)
	$(CARGO) clippy

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

.PHONY: check
check:: fmt fmt-js clippy

.PHONY: playwright-first-run
playwright-first-run: clean-mock-db ui/assets/node_modules/@playwright/package.json
	cd ui/assets && npx playwright test --ui first-run.spec.js

.PHONY: playwright-images
playwright-images: ui/assets/node_modules/@playwright/package.json
	cd ui/assets && npx playwright test --ui images.spec.js

.PHONY: playwright-native
playwright-native: ui/assets/node_modules/@playwright/package.json
	cd ui/assets && npx playwright test --ui provision.native.spec.js

.PHONY: playwright-hvm
playwright-hvm: ui/assets/node_modules/@playwright/package.json
	cd ui/assets && npx playwright test --ui provision.hvm.spec.js

.PHONY: release
release: all
	@echo "Building $(RELEASE_TARBALL)"

	@mkdir -p $(RELSTAGEDIR)/root/opt/smartos/ui/bin \
		$(RELSTAGEDIR)/root/opt/smartos/ui/chroot

	cp $(CARGO_TARGET_DIR)/release/smartos_ui \
		$(RELSTAGEDIR)/root/opt/smartos/ui/bin/ui

	cp $(CARGO_TARGET_DIR)/release/smartos_executor \
		$(RELSTAGEDIR)/root/opt/smartos/ui/bin/executor

	@mkdir -p $(RELSTAGEDIR)/root/opt/smartos/ui/smf/manifests
	cp $(TOP)/smf/manifests/ui.xml \
		$(RELSTAGEDIR)/root/opt/smartos/ui/smf/manifests

	cp $(TOP)/smf/manifests/executor.xml \
		$(RELSTAGEDIR)/root/opt/smartos/ui/smf/manifests

	cp $(TOP)/smf/manifests/ui.sh \
		$(RELSTAGEDIR)/root/opt/smartos/ui/bin

	@mkdir -p $(RELSTAGEDIR)/root/var/log

	cd $(RELSTAGEDIR) && $(TAR) -I pigz -cf $(TOP)/$(RELEASE_TARBALL) root

	@rm -rf $(RELSTAGEDIR)

include ./deps/eng/tools/mk/Makefile.deps
include ./deps/eng/tools/mk/Makefile.targ
include ./deps/eng/tools/mk/Makefile.rust.targ

