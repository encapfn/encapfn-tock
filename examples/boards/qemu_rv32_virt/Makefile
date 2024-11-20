EF_TOCK_BASEDIR := $(shell pwd)/../../../encapfn-tock
EF_BIN_NAME     := efdemo
EF_ARCH         := rv32imac
EF_LAYOUT_LD    := $(shell pwd)/ef_layout.ld
EF_TARGET       := qemu_rv32_virt

include $(EF_TOCK_BASEDIR)/encapfn_c_rt/Configuration.mk

.PHONY: all
all: run

.PHONY: ../../demo/build/qemu_rv32_virt_efdemo.tbf
../../demo/build/qemu_rv32_virt_efdemo.tbf:
	make -C ../../demo/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH)

foreign-libs: ../../demo/build/qemu_rv32_virt_efdemo.tbf

.PHONY: build
build: ../../demo/build/qemu_rv32_virt_efdemo.tbf
	EF_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release

.PHONY: run
run: build
	qemu-system-riscv32 \
	  -machine virt \
	  -semihosting \
	  -global driver=riscv-cpu,property=smepmp,value=true \
	  -global virtio-mmio.force-legacy=false \
	  -device virtio-rng-device \
	  -netdev user,id=n0,net=192.168.1.0/24,dhcpstart=192.168.1.255 \
	  -device virtio-net-device,netdev=n0 \
	  -nographic \
	  -bios ../../../target/riscv32imac-unknown-none-elf/release/encapfn-demo-qemu_rv32_virt \
	  -device loader,file=../../demo/build/qemu_rv32_virt_efdemo.tbf,addr=0x80100000

.PHONY: clean
clean:
	cargo clean
	make -C ../../demo/ EF_LAYOUT_LD=$(EF_LAYOUT_LD) EF_TARGET=$(EF_TARGET) EF_ARCH=$(EF_ARCH) clean
