# Makefile for EF ubench TBF

# EF_TOCK_BASEDIR is set the absolute path of the encapfn-tock crate:
EF_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../encapfn-tock")

EF_BIN_NAME     := ef_ubench

SRCDIR          := $(EF_TOCK_BASEDIR)/../examples/ubench/c_src/
BUILDDIR        := $(EF_TOCK_BASEDIR)/../examples/ubench/build/

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/EncapfnTBF.mk
