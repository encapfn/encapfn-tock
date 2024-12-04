# EF_TOCK_BASEDIR is set the absolute path of the encapfn-tock crate:
EF_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../../encapfn-tock")

EF_ARCH         := rv32imac
EF_LAYOUT_LD    := $(EF_TOCK_BASEDIR)/../examples/boards/earlgrey-cw310/ef_layout.ld
EF_TARGET       := earlgrey-cw310

.PHONY: all
all: combined-image

include ../../otcrypto/Makefile

.PHONY: $(EF_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_efotcrypto.tab
$(EF_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_efotcrypto.tab:
	make -C ../../otcrypto/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH)

# Alias, to only build the foreign library:
foreign-libs: $(EF_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_efotcrypto.tab

.PHONY: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310
$(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310: $(EF_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/libotcrypto.a
	EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release

$(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310.bin: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310
	riscv32-none-elf-objcopy --output-target=binary $< $@
	sha256sum $@

$(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-combined.bin: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310.bin $(EF_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_efotcrypto.tab
	rm -vf $@
	tockloader flash --board opentitan_earlgrey --flash-file $@ --address 0x20000000 $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310.bin
	tockloader install --board opentitan_earlgrey --flash-file $@ --app-address 0x20030000 $(EF_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_efotcrypto.tab

.PHONY: combined-image
combined-image: $(EF_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-combined.bin

.PHONY: clean
clean:
	cargo clean
	make -C $(EF_TOCK_BASEDIR)/../examples/otcrypto/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH) clean
