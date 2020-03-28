EXEC="./target/debug/zatel"
SOCKET_ADDR="unix:/tmp/zatel.so"

srv:
	cargo build
	$(EXEC) $(SOCKET_ADDR)

cli:
	cargo build
	varlink call $(SOCKET_ADDR)/info.grisge.zatel.Get

clean:
	cargo clean

