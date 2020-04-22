VARLINK_SRV_EXEC="./target/debug/npd"
CLI_EXEC="./target/debug/npc"
SOCKET_ADDR="unix:/tmp/nispor.so"

debug:
	cargo build --all
	$(CLI_EXEC) $(ARGS)

srv:
	cargo build --all
	$(VARLINK_SRV_EXEC) $(SOCKET_ADDR)

cli:
	cargo build --all
	varlink call $(SOCKET_ADDR)/info.grisge.nispor.Get

clean:
	cargo clean

all:
	cargo build --all
