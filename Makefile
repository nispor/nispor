RUST_DEBUG_BIN_DIR=./target/debug
RUST_RELEASE_BIN_DIR=./target/release
VARLINK_SRV_EXEC=npd
VARLINK_SRV_EXEC_DEBUG=$(RUST_DEBUG_BIN_DIR)/$(VARLINK_SRV_EXEC)
VARLINK_SRV_EXEC_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(VARLINK_SRV_EXEC)
CLI_EXEC=npc
CLI_EXEC_DEBUG=$(RUST_DEBUG_BIN_DIR)/$(CLI_EXEC)
PYTHON_EXTENTION=libnispor.so
PYTHON_EXTENTION_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(PYTHON_EXTENTION)
CLI_EXEC_RELEASE=$(RUST_RELEASE_BIN_DIR)/$(CLI_EXEC)
SOCKET_FILE=/run/nispor/nispor.so
SOCKET_DIR=$(dir $(SOCKET_FILE))
SOCKET_ADDR=unix:$(SOCKET_FILE)
SYSTEMD_SERVICE_FILE=src/varlink/systemd/nispor.service
SYSTEMD_SOCKET_FILE=src/varlink/systemd/nispor.socket
PREFIX ?= /usr/local


all: $(VARLINK_SRV_EXEC_DEBUG) $(CLI_EXEC_DEBUG) \
    $(VARLINK_SRV_EXEC_RELEASE) $(CLI_EXEC_RELEASE)

# Always invoke cargo build
.PHONY: all clean

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

$(CLI_EXEC_RELEASE) $(VARLINK_SRV_EXEC_RELEASE) $(PYTHON_EXTENTION_RELEASE):
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

install:
	install -D -m755 $(VARLINK_SRV_EXEC_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(VARLINK_SRV_EXEC)
	install -D -m755 $(CLI_EXEC_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	install -D -m644 $(SYSTEMD_SOCKET_FILE) \
		$(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/nispor.socket
	install -D -m644 $(SYSTEMD_SERVICE_FILE) \
		$(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/nispor.service
	install -D -m755 $(PYTHON_EXTENTION_RELEASE) \
		$(DESTDIR)$(PYTHON3_SITE_DIR)/nispor.so

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(VARLINK_SRV_EXEC)
	rm -f $(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	rm -f $(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/nispor*
	rm -f $(DESTDIR)$(PYTHON3_SITE_DIR)/nispor.so
