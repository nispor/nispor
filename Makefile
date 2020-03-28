EXEC="./target/debug/zateld"
SOCKET_ADDR="unix:/tmp/zatel.so"

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
