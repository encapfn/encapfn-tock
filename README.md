# Encapsulated Functions for the Tock OS Kernel

To run the [demo](examples/demo) library on a QEMU rv32i virt target
(both using a direct, unsafe FFI and through Encapsulated Functions),
run the following commands:

```
$ nix-shell
[nix-shell]$ cd examples/boards/qemu_rv32_virt/
[nix-shell:examples/boards/qemu_rv32_virt]$ make run
    Finished `release` profile [optimized + debuginfo] target(s) in 7.26s
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
QEMU RISC-V 32-bit "virt" machine, initialization complete.
Entering main loop.
allocated array [false, ..., false]
demo_nop returned 1379
allocated array after invoke[true, ..., false]
Ran test_libdemo with the MockRt!
allocated array [false, ...., false]
demo_nop returned 1379
allocated array after invoke[true, ..., false]
Ran test_libdemo with the Rv32iCRt!
tock$
```

To exit QEMU, either press `C-a x` or type `panic` on the Tock prompt.
