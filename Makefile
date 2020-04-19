EXEC="./target/debug/zateld"
CLI_EXEC="./target/debug/zc"
SOCKET_ADDR="unix:/tmp/zatel.so"

debug:
	cargo build --all
	$(CLI_EXEC) $(ARGS)

srv:
	cargo build --all
	$(EXEC) $(SOCKET_ADDR)

cli:
	cargo build --all
	varlink call $(SOCKET_ADDR)/info.grisge.zatel.Get

clean:
	cargo clean

all:
	cargo build --all
