# TODO: how to avoid the redundancy between this and the library's Makefile?
EF_TOCK_BASEDIR := $(shell pwd)/../../../encapfn-tock
EF_BIN_NAME     := efotcrypto
EF_ARCH         := rv32imac
EF_LAYOUT_LD    := $(shell pwd)/ef_layout.ld
EF_TARGET       := earlgrey-cw310
EF_CFLAGS       := -I$(shell readlink -f $(EF_TOCK_BASEDIR)/../opentitan-cryptolib/include)

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/Configuration.mk

.PHONY: all
all: combined-image

.PHONY: ../../otcrypto/build/earlgrey-cw310_efotcrypto.tab
../../otcrypto/build/earlgrey-cw310_efotcrypto.tab:
	make -C ../../otcrypto/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH)

# Alias, to only build the foreign library:
foreign-libs: ../../otcrypto/build/earlgrey-cw310_efotcrypto.tab

.PHONY: ../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310
../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310:
	EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release

../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310.bin: ../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310
	riscv32-none-elf-objcopy --output-target=binary $< $@
	sha256sum $@

../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-combined.bin: ../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310.bin ../../otcrypto/build/earlgrey-cw310_efotcrypto.tab
	rm -vf $@
	tockloader flash --board opentitan_earlgrey --flash-file $@ --address 0x20000000 ../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310.bin
	tockloader install --board opentitan_earlgrey --flash-file $@ --app-address 0x20030000 ../../otcrypto/build/earlgrey-cw310_efotcrypto.tab

.PHONY: combined-image
combined-image: ../../../target/riscv32imc-unknown-none-elf/release/encapfn-earlgrey-cw310-combined.bin

.PHONY: clean
clean:
	cargo clean
	make -C ../../otcrypto/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH) clean
