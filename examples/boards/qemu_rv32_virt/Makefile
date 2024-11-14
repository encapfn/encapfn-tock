run:
	cargo build --release
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
