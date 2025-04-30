# Makefile for EF demo binary

# EF_TOCK_BASEDIR is set the absolute path of the encapfn-tock crate:
EF_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../encapfn-tock")

EF_BIN_NAME     := ef_littlefs
EF_CFLAGS       := -I$(shell readlink -f $(EF_TOCK_BASEDIR)/../third-party/littlefs)
EF_LINK_OBJ     := $(EF_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/liblfs.a
SRCDIR          := $(EF_TOCK_BASEDIR)/../examples/littlefs/c_src/
BUILDDIR        := $(EF_TOCK_BASEDIR)/../examples/littlefs/build/

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/EncapfnTBF.mk

.PHONY: $(EF_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/liblfs.a
$(EF_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/liblfs.a:
	mkdir -p $(EF_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/
	make -C $(EF_TOCK_BASEDIR)/../third-party/littlefs/

