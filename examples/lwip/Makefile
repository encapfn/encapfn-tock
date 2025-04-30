# Makefile for LwIP library

# EF_TOCK_BASEDIR is set the absolute path of the encapfn-tock crate:
EF_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../encapfn-tock")

EF_BIN_NAME     := ef_lwip
EF_CFLAGS       := -I$(EF_TOCK_BASEDIR)/../examples/lwip/src/include -I$(EF_TOCK_BASEDIR)/../examples/lwip/config

SRCDIR          := $(EF_TOCK_BASEDIR)/../examples/lwip/src
BUILDDIR        := $(EF_TOCK_BASEDIR)/../examples/lwip/build

CSRC            := \
    $(SRCDIR)/eflwip_tbf.c \
    $(SRCDIR)/core/def.c \
    $(SRCDIR)/core/inet_chksum.c \
    $(SRCDIR)/core/init.c \
    $(SRCDIR)/core/ip.c \
    $(SRCDIR)/core/ipv4/acd.c \
    $(SRCDIR)/core/ipv4/autoip.c \
    $(SRCDIR)/core/ipv4/dhcp.c \
    $(SRCDIR)/core/ipv4/etharp.c \
    $(SRCDIR)/core/ipv4/icmp.c \
    $(SRCDIR)/core/ipv4/ip4.c \
    $(SRCDIR)/core/ipv4/ip4_addr.c \
    $(SRCDIR)/core/ipv4/ip4_frag.c \
    $(SRCDIR)/core/ipv6/icmp6.c \
    $(SRCDIR)/core/ipv6/ip6.c \
    $(SRCDIR)/core/ipv6/ip6_addr.c \
    $(SRCDIR)/core/ipv6/ip6_frag.c \
    $(SRCDIR)/core/ipv6/mld6.c \
    $(SRCDIR)/core/ipv6/nd6.c \
    $(SRCDIR)/core/mem.c \
    $(SRCDIR)/core/memp.c \
    $(SRCDIR)/core/netif.c \
    $(SRCDIR)/core/pbuf.c \
    $(SRCDIR)/core/raw.c \
    $(SRCDIR)/core/tcp.c \
    $(SRCDIR)/core/tcp_in.c \
    $(SRCDIR)/core/tcp_out.c \
    $(SRCDIR)/core/timeouts.c \
    $(SRCDIR)/core/udp.c \
    $(SRCDIR)/netif/ethernet.c

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/EncapfnTBF.mk

cargo:
		EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release --target riscv32imac-unknown-none-elf

doc:
		EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo doc --target riscv32imac-unknown-none-elf
