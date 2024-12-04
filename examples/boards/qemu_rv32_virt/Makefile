EF_TOCK_BASEDIR := $(shell pwd)/../../../encapfn-tock
EF_BIN_NAME     := ef_littlefs
EF_ARCH         := rv32imac
EF_LAYOUT_LD    := $(shell pwd)/ef_layout.ld
EF_TARGET       := qemu_rv32_virt
EF_CFLAGS       := -I$(shell readlink -f $(EF_TOCK_BASEDIR)/../third-party/littlefs)

.PHONY: all
all: run

include $(EF_TOCK_BASEDIR)/../examples/littlefs/Makefile

.PHONY: $(EF_TOCK_BASEDIR)/../examples/littlefs/build/qemu_rv32_virt_ef_littlefs.tbf
$(EF_TOCK_BASEDIR)/../examples/littlefs/build/qemu_rv32_virt_ef_littlefs.tbf:
	make -C $(EF_TOCK_BASEDIR)/../examples/littlefs/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH)

foreign-libs: $(EF_TOCK_BASEDIR)/../examples/littlefs/build/qemu_rv32_virt_ef_littlefs.tbf

.PHONY: run
run: $(EF_TOCK_BASEDIR)/../examples/littlefs/build/qemu_rv32_virt_ef_littlefs.tbf
	EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release
	qemu-system-riscv32 \
	  -machine virt \
	  -semihosting \
	  -global driver=riscv-cpu,property=smepmp,value=true \
	  -global virtio-mmio.force-legacy=false \
	  -device virtio-rng-device \
	  -netdev user,id=n0,net=192.168.1.0/24,dhcpstart=192.168.1.255 \
	  -device virtio-net-device,netdev=n0 \
	  -nographic \
	  -bios $(EF_TOCK_BASEDIR)/../target/riscv32imac-unknown-none-elf/release/encapfn-tock-littlefs-qemu_rv32_virt \
	  -device loader,file=$(EF_TOCK_BASEDIR)/../examples/littlefs/build/qemu_rv32_virt_ef_littlefs.tbf,addr=0x80100000

.PHONY: clean
clean:
	cargo clean
	make -C ../../littlefs/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH) clean
