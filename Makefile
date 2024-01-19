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

.PHONY: all
all: release

.PHONY: release
release: $(RS_FILES) | $(CARGO_EXEC)
	$(CARGO) build --release

.PHONY: debug
debug: $(RS_FILES) | $(CARGO_EXEC)
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

include ./deps/eng/tools/mk/Makefile.deps
include ./deps/eng/tools/mk/Makefile.targ
include ./deps/eng/tools/mk/Makefile.rust.targ

