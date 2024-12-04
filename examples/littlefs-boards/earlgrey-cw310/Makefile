# EF_TOCK_BASEDIR is set the absolute path of the encapfn-tock crate:
EF_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../../encapfn-tock")

EF_ARCH         := rv32imac
EF_LAYOUT_LD    := $(EF_TOCK_BASEDIR)/../examples/littlefs-boards/earlgrey-cw310/ef_layout.ld
EF_TARGET       := earlgrey-cw310

.PHONY: all
all: combined-image

include ../../littlefs/Makefile

.PHONY: $(EF_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_ef_littlefs.tab
$(EF_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_ef_littlefs.tab:
	make -C ../../littlefs/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH)

# Alias, to only build the foreign library:
foreign-libs: $(EF_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_ef_littlefs.tab

.PHONY: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs
$(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs:
	EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release

$(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs.bin: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs
	riscv32-none-elf-objcopy --output-target=binary $< $@
	sha256sum $@

$(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs-combined.bin: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs.bin $(EF_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_ef_littlefs.tab
	rm -vf $@
	tockloader flash --board opentitan_earlgrey --flash-file $@ --address 0x20000000 $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs.bin
	tockloader install --board opentitan_earlgrey --flash-file $@ --app-address 0x20030000 $(EF_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_ef_littlefs.tab

.PHONY: combined-image
combined-image: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-littlefs-combined.bin

.PHONY: clean
clean:
	cargo clean
	make -C $(EF_TOCK_BASEDIR)/../examples/littlefs/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH) clean
