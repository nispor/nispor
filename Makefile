VARLINK_SRV_EXEC="./target/debug/npd"
CLI_EXEC="./target/debug/npc"
SOCKET_ADDR="unix:/tmp/nispor.so"

debug: $(CLI_EXEC)
	$(CLI_EXEC) $(ARGS)

$(CLI_EXEC) $(VARLINK_SRV_EXEC):
	cargo build --all

srv:
	$(VARLINK_SRV_EXEC) $(SOCKET_ADDR)

cli:
	varlink call $(SOCKET_ADDR)/info.grisge.nispor.Get

clean:
	cargo clean

all:
	cargo build --all
