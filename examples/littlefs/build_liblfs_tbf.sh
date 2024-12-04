#!/usr/bin/env bash

set -e
set -x

make EF_LAYOUT_LD=../boards/qemu_rv32_virt/ef_layout.ld EF_TARGET=qemu_rv32_virt EF_ARCH=rv32imac