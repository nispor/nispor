include ./Makefile.inc

RUST_DEBUG_BIN_DIR=./target/debug
RUST_RELEASE_BIN_DIR=./target/release
CLI_EXEC=npc
CLI_EXEC_DEBUG=$(RUST_DEBUG_BIN_DIR)/$(CLI_EXEC)
CLIB_HEADER=nispor.h
CLIB_HEADER_IN=src/clib/$(CLIB_HEADER).in
CLIB_SO_DEV_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(CLIB_SO_DEV)
CLIB_PKG_CONFIG_FILE=nispor.pc
CLIB_PKG_CONFIG_FILE_IN=src/clib/$(CLIB_PKG_CONFIG_FILE).in
PYTHON_MODULE_NAME=nispor
PYTHON_MODULE_SRC=src/python/nispor
CLI_EXEC_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(CLI_EXEC)
SOCKET_FILE=/run/nispor/nispor.so
SOCKET_DIR=$(dir $(SOCKET_FILE))
SOCKET_ADDR=unix:$(SOCKET_FILE)
PREFIX ?= /usr/local
TARBALL=nispor-$(VERSION).tar.gz
VENDOR_TARBALL=nispor-vendor-$(VERSION).tar.xz

CPU_BITS = $(shell getconf LONG_BIT)
ifeq ($(CPU_BITS), 32)
    LIBDIR ?= $(PREFIX)/lib
else
    LIBDIR ?= $(PREFIX)/lib$(CPU_BITS)
endif

INCLUDE_DIR ?= $(PREFIX)/include
PKG_CONFIG_LIBDIR ?= $(LIBDIR)/pkgconfig

SKIP_PYTHON_INSTALL ?=0

all: $(CLI_EXEC_DEBUG) $(CLI_EXEC_RELEASE)

SYSTEMD_SYS_UNIT_DIR ?= $(shell \
	pkg-config --variable=systemdsystemunitdir systemd)

PYTHON3_SITE_DIR ?=$(shell \
	python3 -c "import sysconfig; print(sysconfig.get_path('purelib'))")

# Always invoke cargo build for debug
.PHONY: $(CLI_EXEC_DEBUG)

debug: $(CLI_EXEC_DEBUG)
	$(CLI_EXEC_DEBUG) $(ARGS)


$(CLI_EXEC_DEBUG):
	cargo build --all

$(CLI_EXEC_RELEASE) $(CLIB_SO_DEV_RELEASE):
	cargo build --all --release

check:
	cargo test -- --test-threads=1 --show-output;
	if [ "CHK$(CI)" != "CHKtrue" ]; then \
		cargo test -- --test-threads=1 --show-output --ignored; \
	fi
	make check -C test/clib

check_in_container:
	./tools/nispor-in-container -- make check

clean:
	cargo clean
	make clean -C test/clib

install: $(CLI_EXEC_RELEASE)
	install -p -v -D -m755 $(CLI_EXEC_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	install -p -D -m755 $(CLIB_SO_DEV_RELEASE) \
		$(DESTDIR)$(LIBDIR)/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_FULL) $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MAN)
	ln -sfv $(CLIB_SO_FULL) $(DESTDIR)$(LIBDIR)/$(CLIB_SO_DEV)
	if [ $(SKIP_PYTHON_INSTALL) != 1 ];then \
	    install -p -v -D -d -m755 $(PYTHON_MODULE_SRC) \
		    $(DESTDIR)$(PYTHON3_SITE_DIR)/$(PYTHON_MODULE_NAME); \
	    install -p -v -D -m644 $(PYTHON_MODULE_SRC)/*.py \
		    $(DESTDIR)$(PYTHON3_SITE_DIR)/$(PYTHON_MODULE_NAME)/; \
	fi
	install -p -v -D -m644 $(CLIB_HEADER_IN) \
		$(DESTDIR)$(INCLUDE_DIR)/$(CLIB_HEADER)
	sed -i -e 's/@_VERSION_MAJOR@/$(VERSION_MAJOR)/' \
		$(DESTDIR)$(INCLUDE_DIR)/$(CLIB_HEADER)
	sed -i -e 's/@_VERSION_MINOR@/$(VERSION_MINOR)/' \
		$(DESTDIR)$(INCLUDE_DIR)/$(CLIB_HEADER)
	sed -i -e 's/@_VERSION_MICRO@/$(VERSION_MICRO)/' \
		$(DESTDIR)$(INCLUDE_DIR)/$(CLIB_HEADER)
	install -p -v -D -m644 $(CLIB_PKG_CONFIG_FILE_IN) \
		$(DESTDIR)$(PKG_CONFIG_LIBDIR)/$(CLIB_PKG_CONFIG_FILE)
	sed -i -e 's/@VERSION@/$(VERSION)/' \
		$(DESTDIR)$(PKG_CONFIG_LIBDIR)/$(CLIB_PKG_CONFIG_FILE)
	sed -i -e 's/@PREFIX@/$(subst /,\/,$(PREFIX))/' \
		$(DESTDIR)$(PKG_CONFIG_LIBDIR)/$(CLIB_PKG_CONFIG_FILE)
	sed -i -e 's/@LIBDIR@/$(subst /,\/,$(LIBDIR))/' \
		$(DESTDIR)$(PKG_CONFIG_LIBDIR)/$(CLIB_PKG_CONFIG_FILE)
	sed -i -e 's/@INCLUDE_DIR@/$(subst /,\/,$(INCLUDE_DIR))/' \
		$(DESTDIR)$(PKG_CONFIG_LIBDIR)/$(CLIB_PKG_CONFIG_FILE)


uninstall:
	- rm -fv $(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	- rm -fv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_DEV)
	- rm -fv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MAN)
	- rm -fv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MIN)
	- rm -fv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_FULL)
	- rm -fv $(DESTDIR)$(INCLUDE_DIR)/$(CLIB_HEADER)
	- if [ $(SKIP_PYTHON_INSTALL) != 1 ];then \
		rm -rfv $(DESTDIR)$(PYTHON3_SITE_DIR)/$(PYTHON_MODULE_NAME); \
	fi


dist:
	git archive --prefix=nispor-$(VERSION)/ \
		--format=tar.gz -o $(TARBALL) HEAD
	$(eval TMPDIR := $(shell mktemp -d))
	cargo vendor-filterer $(TMPDIR)/vendor || \
		(echo -en "\nNot cargo-vendor-filterer, Please install via "; \
		 echo -e "'cargo install cargo-vendor-filterer'\n")
	cd $(TMPDIR)
	tar cfJ $(VENDOR_TARBALL) vendor
	rm -rf $(TMPDIR)

upstream_release:
	./tools/upstream_release.sh
