VERSION=$(shell cargo pkgid nispor|cut -d@ -f2)
ROOT_DIR?=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
RUST_DEBUG_BIN_DIR=./target/debug
RUST_RELEASE_BIN_DIR=./target/release
CLI_EXEC=npc
CLI_EXEC_DEBUG=$(RUST_DEBUG_BIN_DIR)/$(CLI_EXEC)
CLI_EXEC_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(CLI_EXEC)
PREFIX ?= /usr/local
TARBALL=nispor-$(VERSION).tar.gz
VENDOR_TARBALL=nispor-vendor-$(VERSION).tar.xz

all: $(CLI_EXEC_DEBUG) $(CLI_EXEC_RELEASE)

# Always invoke cargo build for debug
.PHONY: $(CLI_EXEC_DEBUG)

debug: $(CLI_EXEC_DEBUG)
	$(CLI_EXEC_DEBUG) $(ARGS)


$(CLI_EXEC_DEBUG):
	cargo build --all

$(CLI_EXEC_RELEASE):
	cargo build --all --release

check:
	cargo test -- --test-threads=1 --show-output;
	if [ "CHK$(CI)" != "CHKtrue" ]; then \
		cargo test -- --test-threads=1 --show-output --ignored; \
	fi

clean:
	cargo clean

install: $(CLI_EXEC_RELEASE)
	install -p -v -D -m755 $(CLI_EXEC_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)


uninstall:
	- rm -fv $(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)


dist:
	git archive --prefix=nispor-$(VERSION)/ \
		--format=tar.gz -o $(TARBALL) HEAD
	$(eval TMPDIR := $(shell mktemp -d))
	cargo vendor-filterer $(TMPDIR)/vendor || \
		(echo -en "\nNot cargo-vendor-filterer, Please install via "; \
		 echo -e "'cargo install cargo-vendor-filterer'\n")
	tar cfJ $(VENDOR_TARBALL) -C $(TMPDIR) vendor
	rm -rf $(TMPDIR)

upstream_release:
	./tools/upstream_release.sh
