#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#

#
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

# We need a Node newer than what's available in sdc-node (v6)
# At least v14 which is available in 21.4.0 is sufficient
.PHONY: nodejs
ifeq ($(shell uname -s),SunOS)
nodejs:
	pkgin -y in npm
else
# On other OSes, assume you have a new enough Node
nodejs:
	node --version
endif

ui/assets/node_modules: ui/assets/package.json ui/assets/package-lock.json
	cd ui/assets && npm install --omit=dev

	# The npm version available in the Jenkins agent will downgrade the
	# package-lock version to v1 causing the workspace to be dirty. Once a newer
	# sdc-node version is available this can be removed (and the pkgsrc node can
	# be replaced with sdc-node.)
	git checkout ui/assets/package-lock.json

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
assets: nodejs ui/assets/main.css.gz ui/assets/main.js.gz

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
	STAMP=$(STAMP) $(CARGO) build --release

.PHONY: debug
debug: assets $(RS_FILES) | $(CARGO_EXEC)
	STAMP=$(STAMP) $(CARGO) build

.PHONY: fmt
fmt: | $(CARGO_EXEC)
	$(CARGO) fmt

.PHONY: clippy
clippy: | $(CARGO_EXEC)
	STAMP=$(STAMP) $(CARGO) clippy

.PHONY: version
version:
	@$(CARGO) metadata --format-version 1 --offline

.PHONY: license-check
license-check: | $(CARGO_EXEC)
	$(CARGO) install cargo-license
	$(CARGO) license

.PHONY: update
update: | $(CARGO_EXEC)
	$(CARGO) update

.PHONY: doc
doc: | $(CARGO_EXEC)
	$(CARGO) doc --open

.PHONY: devrun
devrun: debug
	./tools/devrun.sh

.PHONY: check
check:: assets fmt fmt-js clippy

.PHONY: test
test: | $(CARGO_EXEC)
	STAMP=$(STAMP) $(CARGO) test

.PHONY: release
release: all
	@echo "Building $(NAME)-$(STAMP).tar.gz"

	# Executables
	@mkdir -p $(RELSTAGEDIR)/root/opt/smartos/ui/bin

	cp $(CARGO_TARGET_DIR)/release/smartos_ui \
		$(RELSTAGEDIR)/root/opt/smartos/ui/bin/ui

	cp $(CARGO_TARGET_DIR)/release/smartos_ui_executor \
		$(RELSTAGEDIR)/root/opt/smartos/ui/bin/executor

	cp $(TOP)/tools/ui.sh \
		$(RELSTAGEDIR)/root/opt/smartos/ui/bin

	@mkdir -p $(RELSTAGEDIR)/root/opt/smartdc/bin

	cp $(TOP)/tools/uiadm.sh \
		$(RELSTAGEDIR)/root/opt/smartdc/bin/uiadm

	# Chroot
	@mkdir -p $(RELSTAGEDIR)/root/var/svc/manifest/site

	# SMF Manifests
	cp $(TOP)/smf/manifests/smartos-ui.xml \
		$(RELSTAGEDIR)/root/var/svc/manifest/site

	cp $(TOP)/smf/manifests/smartos-ui-executor.xml \
		$(RELSTAGEDIR)/root/var/svc/manifest/site

	# Logs
	@mkdir -p $(RELSTAGEDIR)/root/var/log

	# UI process doesn't have permission to create this file
	touch $(RELSTAGEDIR)/root/var/log/smartos_ui.log
	chmod g+rw $(RELSTAGEDIR)/root/var/log/smartos_ui.log
	chown nobody $(RELSTAGEDIR)/root/var/log/smartos_ui.log

	cd $(RELSTAGEDIR) && $(TAR) -I pigz -cf \
		$(TOP)/$(NAME)-$(STAMP).tar.gz root

	@rm -rf $(RELSTAGEDIR)

.PHONY: publish
publish: release
	mkdir -p $(ENGBLD_BITS_DIR)/$(NAME)
	cp $(TOP)/$(NAME)-$(STAMP).tar.gz \
		$(ENGBLD_BITS_DIR)/$(NAME)
	cp $(TOP)/tools/uiadm.sh $(ENGBLD_BITS_DIR)/$(NAME)

include ./deps/eng/tools/mk/Makefile.deps
include ./deps/eng/tools/mk/Makefile.targ
include ./deps/eng/tools/mk/Makefile.rust.targ

