ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/../../

include $(ROOT_DIR)/Makefile.inc

TMPDIR := $(shell mktemp -d)
DEBUG_NISPOR_SO_FILE=$(ROOT_DIR)/target/debug/libnispor.so

CFLAGS =-g -Wall -Wextra -L$(TMPDIR)
LDFLAGS =-I$(TMPDIR)
LIBS =-lnispor
CC ?=cc

all: check

HEADER_IN = $(ROOT_DIR)/src/clib/nispor.h.in

nispor.h: $(HEADER_IN)
	install -p -v -D -m644 $(HEADER_IN) \
		$(TMPDIR)/nispor.h
	sed -i -e 's/@_VERSION_MAJOR@/$(VERSION_MAJOR)/' \
		$(TMPDIR)/nispor.h
	sed -i -e 's/@_VERSION_MINOR@/$(VERSION_MINOR)/' \
		$(TMPDIR)/nispor.h
	sed -i -e 's/@_VERSION_MICRO@/$(VERSION_MICRO)/' \
		$(TMPDIR)/nispor.h

libnispor.so: $(DEBUG_NISPOR_SO_FILE)
	install -p -v -D -m755 $(DEBUG_NISPOR_SO_FILE) \
		$(TMPDIR)/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_MAN)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_DEV)

nispor_test: nispor_test.c nispor.h libnispor.so
	$(CC) $(CFLAGS) $(LDFLAGS) -o nispor_test nispor_test.c $(LIBS)

check: nispor_test
	LD_LIBRARY_PATH=$(TMPDIR) \
		valgrind --trace-children=yes --leak-check=full \
		--error-exitcode=1 \
		./nispor_test 1>/dev/null
	rm -rf $(TMPDIR)

clean:
	rm -f nispor_test.o nispor_test
