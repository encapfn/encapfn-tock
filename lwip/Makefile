# Makefile for EF demo binary

EF_CFLAGS       := -I./src/include -I./config
EF_TOCK_BASEDIR := ../encapfn-tock
EF_BIN_NAME     := eflwip
EF_LINK_OBJ     := $(EF_TOCK_BASEDIR)/../opentitan-cryptolib/liblwip.a

SRCDIR          := ./src

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/EncapfnTBF.mk

cargo:
		EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release --target riscv32imac-unknown-none-elf

doc:
		EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo doc --target riscv32imac-unknown-none-elf
