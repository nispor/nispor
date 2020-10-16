TMPDIR := $(shell mktemp -d)
TOP_SRC_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/../../

CFLAGS =-g -Wall -Wextra -lnispor -L$(TOP_SRC_DIR)/target/debug
LDFLAGS =-I$(TMPDIR)
CC ?=gcc

all: check

HEADER_IN = $(TOP_SRC_DIR)/src/clib/nispor.h.in

nispor.h: $(HEADER_IN)
	install -p -v -D -m644 $(HEADER_IN) \
		$(TMPDIR)/nispor.h
	# Use 0.0.1 version for test only
	sed -i -e 's/@_VERSION_MAJOR@/0/' $(TMPDIR)/nispor.h
	sed -i -e 's/@_VERSION_MINOR@/0/' $(TMPDIR)/nispor.h
	sed -i -e 's/@_VERSION_MICRO@/1/' $(TMPDIR)/nispor.h

nispor_test.o: nispor_test.c nispor.h
	$(CC) $(CFLAGS) $(LDFLAGS) -c nispor_test.c

nispor_test: nispor_test.o
	$(CC) $(CFLAGS) $(LDFLAGS) -o nispor_test nispor_test.o
	rm -rf $(TMPDIR)

check: nispor_test
	valgrind --trace-children=yes --leak-check=full ./nispor_test

clean:
	rm -f nispor_test.o nispor_test
