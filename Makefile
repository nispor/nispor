RUST_DEBUG_BIN_DIR=./target/debug
RUST_RELEASE_BIN_DIR=./target/release
VARLINK_SRV_EXEC=npd
VARLINK_SRV_EXEC_DEBUG=$(RUST_DEBUG_BIN_DIR)/$(VARLINK_SRV_EXEC)
VARLINK_SRV_EXEC_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(VARLINK_SRV_EXEC)
CLI_EXEC=npc
CLI_EXEC_DEBUG=$(RUST_DEBUG_BIN_DIR)/$(CLI_EXEC)
CLIB_VERSION_MAJOR=0
CLIB_VERSION_MINOR=3
CLIB_VERSION_MICRO=0
CLIB_HEADER=nispor.h
CLIB_HEADER_RELEASE=src/clib/$(CLIB_HEADER)
CLIB_SO_DEV=libnispor.so
CLIB_SO_MAN=$(CLIB_SO_DEV).$(CLIB_VERSION_MAJOR)
CLIB_SO_MIN=$(CLIB_SO_MAN).$(CLIB_VERSION_MINOR)
CLIB_SO_FULL=$(CLIB_SO_MIN).$(CLIB_VERSION_MICRO)
CLIB_SO_DEV_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(CLIB_SO_DEV)
PYTHON_MODULE_NAME=nispor
PYTHON_MODULE_SRC=src/python/nispor
CLI_EXEC_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(CLI_EXEC)
SOCKET_FILE=/run/nispor/nispor.so
SOCKET_DIR=$(dir $(SOCKET_FILE))
SOCKET_ADDR=unix:$(SOCKET_FILE)
SYSTEMD_SERVICE_FILE=src/varlink/systemd/nispor.service
SYSTEMD_SOCKET_FILE=src/varlink/systemd/nispor.socket
PREFIX ?= /usr/local

libdir.x86_64 = lib64
libdir.i686   = lib
MACHINE = $(shell uname -m)
LIBDIR ?= $(PREFIX)/$(libdir.$(MACHINE))

INCLUDE_DIR ?= $(PREFIX)/include

SKIP_PYTHON_INSTALL ?=0

all: $(VARLINK_SRV_EXEC_DEBUG) $(CLI_EXEC_DEBUG) \
    $(VARLINK_SRV_EXEC_RELEASE) $(CLI_EXEC_RELEASE)

# Always invoke cargo build
.PHONY: $(VARLINK_SRV_EXEC_DEBUG) $(CLI_EXEC_DEBUG) \
    $(VARLINK_SRV_EXEC_RELEASE) $(CLI_EXEC_RELEASE)

SYSTEMD_SYS_UNIT_DIR ?= $(shell \
	pkg-config --variable=systemdsystemunitdir systemd)

PYTHON3_SITE_DIR ?=$(shell \
	python3 -c \
		"from distutils.sysconfig import get_python_lib; \
		 print(get_python_lib())")

debug: $(CLI_EXEC_DEBUG)
	$(CLI_EXEC_DEBUG) $(ARGS)

$(CLI_EXEC_DEBUG) $(VARLINK_SRV_EXEC_DEBUG):
	cargo build --all

$(CLI_EXEC_RELEASE) $(VARLINK_SRV_EXEC_RELEASE):
	cargo build --all --release

test:
	cargo test -- --test-threads=1 --show-output

srv: $(VARLINK_SRV_EXEC_DEBUG)
	echo $(SOCKET_DIR)
	if [ ! -d $(SOCKET_DIR) ]; then \
		sudo mkdir $(SOCKET_DIR); \
		sudo chmod 0777 $(SOCKET_DIR); \
	fi
	$(VARLINK_SRV_EXEC_DEBUG) $(SOCKET_ADDR)

cli:
	varlink call $(SOCKET_ADDR)/info.nispor.Get

clean:
	cargo clean

install: $(VARLINK_SRV_EXEC_RELEASE) $(CLI_EXEC_RELEASE)
	install -v -D -m755 $(VARLINK_SRV_EXEC_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(VARLINK_SRV_EXEC)
	install -v -D -m755 $(CLI_EXEC_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	install -v -D -m644 $(SYSTEMD_SOCKET_FILE) \
		$(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/nispor.socket
	install -v -D -m644 $(SYSTEMD_SERVICE_FILE) \
		$(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/nispor.service
	install -v -D -m644 $(CLIB_SO_DEV_RELEASE) \
		$(DESTDIR)$(LIBDIR)/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_FULL) $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MAN)
	ln -sfv $(CLIB_SO_FULL) $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MIN)
	ln -sfv $(CLIB_SO_FULL) $(DESTDIR)$(LIBDIR)/$(CLIB_SO_DEV)
	if [ $(SKIP_PYTHON_INSTALL) != 1 ];then \
	    install -v -D -d -m755 $(PYTHON_MODULE_SRC) \
		    $(DESTDIR)$(PYTHON3_SITE_DIR)/$(PYTHON_MODULE_NAME); \
	    install -v -D -m644 $(PYTHON_MODULE_SRC)/*.py \
		    $(DESTDIR)$(PYTHON3_SITE_DIR)/$(PYTHON_MODULE_NAME)/; \
	fi
	install -v -D -m644 $(CLIB_HEADER_RELEASE) \
		$(DESTDIR)$(INCLUDE_DIR)/$(CLIB_HEADER)

uninstall:
	- rm -fv $(DESTDIR)$(PREFIX)/bin/$(VARLINK_SRV_EXEC)
	- rm -fv $(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	- rm -fv $(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/nispor*
	- rm -rv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_DEV)
	- rm -rv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MAN)
	- rm -rv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MIN)
	- rm -rv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_FULL)
	- rm -fv $(DESTDIR)$(INCLUDE_DIR)/$(CLIB_HEADER)
	- if [ $(SKIP_PYTHON_INSTALL) != 1 ];then \
		rm -rfv $(DESTDIR)$(PYTHON3_SITE_DIR)/$(PYTHON_MODULE_NAME); \
	fi
