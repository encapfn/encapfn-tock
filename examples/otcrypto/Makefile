# Makefile for EF demo binary

EF_TOCK_BASEDIR := ../../encapfn-tock
EF_BIN_NAME     := efotcrypto
EF_CFLAGS       := -I$(shell readlink -f $(EF_TOCK_BASEDIR)/../opentitan-cryptolib/include)
EF_LINK_OBJ     := $(EF_TOCK_BASEDIR)/../opentitan-cryptolib/libotcrypto.a

SRCDIR          := ./c_src/

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/EncapfnTBF.mk
