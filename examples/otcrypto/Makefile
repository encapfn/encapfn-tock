# Makefile for EF demo binary

# EF_TOCK_BASEDIR is set the absolute path of the encapfn-tock crate:
EF_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../encapfn-tock")

EF_BIN_NAME     := efotcrypto
EF_CFLAGS       := -I$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/include
EF_LINK_OBJ     := $(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/libotcrypto.a

SRCDIR          := $(EF_TOCK_BASEDIR)/../examples/otcrypto/c_src/
BUILDDIR        := $(EF_TOCK_BASEDIR)/../examples/otcrypto/build/

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/EncapfnTBF.mk

$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz:
	wget -O"$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz" \
	  https://alpha.mirror.svc.schuermann.io/files/encapfn/2024-12-03_opentitan_cryptolib_earlgrey_es_sival_bbbd9b0d00_patched.tar.xz
	cd "$(EF_TOCK_BASEDIR)/../third-party"; \
	  echo '95a955f9037850489d77b80afd97bf4338225bd7197e12f47dbf29632eafbf97  opentitan-cryptolib.tar.xz' | sha256sum -c

$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/libotcrypto.a: $(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz
	rm -rvf "$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib"
	mkdir -p "$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib"
	tar -xvf "$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz" \
	  -C "$(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib" \
	  --strip-components=1 crypto/
