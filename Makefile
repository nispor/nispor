VARLINK_SRV_EXEC="./target/debug/npd"
CLI_EXEC="./target/debug/npc"
SOCKET_ADDR="unix:/run/nispor.so"
SYSTEMD_FILES=src/varlink/systemd/nispor.service \
	      src/varlink/systemd/nispor.socket
PREFIX ?= /usr/local

SYSTEMD_SYS_UNIT_DIR ?= $(shell pkg-config --variable=systemdsystemunitdir systemd)

debug: $(CLI_EXEC)
	$(CLI_EXEC) $(ARGS)

$(CLI_EXEC) $(VARLINK_SRV_EXEC):
	cargo build --all

test:
	cargo test -- --test-threads=1 --show-output

srv:
	$(VARLINK_SRV_EXEC) $(SOCKET_ADDR)

cli:
	varlink call $(SOCKET_ADDR)/info.nispor.Get

clean:
	cargo clean

all:
	cargo build --all

install:
	install -m755 $(VARLINK_SRV_EXEC) $(DESTDIR)$(PREFIX)/bin/
	install -m755 $(CLI_EXEC) $(DESTDIR)$(PREFIX)/bin/
	install -m644 $(SYSTEMD_FILES) $(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(VARLINK_SRV_EXEC)
	rm -f $(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	rm -f $(DESTDIR)$(SYSTEMD_SYS_UNIT_DIR)/nispor*
