// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use core::ptr::addr_of_mut;

use kernel::debug;
use kernel::{capabilities, create_capability};
use qemu_rv32_virt_lib::{self, PROCESSES};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/// Main function.
///
/// This function is called from the arch crate after some very basic
/// RISC-V setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    // These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let (board_kernel, platform, chip, _default_peripherals) = qemu_rv32_virt_lib::start();

    // Special encapsulated functions linker symbols:
    extern "C" {
        static mut _efram_start: u8;
        static mut _efram_end: u8;
    }

    encapfn::branding::new(|brand| {
        // This is unsafe, as it instantiates a runtime that can be used to run
        // foreign functions without memory protection:
        let (rt, mut alloc, mut access) = unsafe {
            encapfn::rt::mock::MockRt::new(
                false, // zero_copy_immutable
		true, // all_upgrades_valid
                encapfn::rt::mock::stack_alloc::StackAllocator::<
                    encapfn::rt::mock::stack_alloc::StackFrameAllocRiscv,
                >::new(),
                brand,
            )
        };

        // Create a "bound" runtime
        let bound_rt = encapfn_tock_littlefs::ef_littlefs_bindings::LibLittleFSRt::new(&rt).unwrap();
        debug!("About to test_libdemo with the MockRT!");

        // Run a test:
        encapfn_tock_littlefs::test_littlefs(&bound_rt, &mut alloc, &mut access);
        debug!("Ran test_libdemo with the MockRt!");
        debug!("");

        // Test callbacks!
        // encapfn_example_demo::test_libdemo_callback(&bound_rt, &mut alloc, &mut access);
    });

    encapfn::branding::new(|brand| {
        // Try to load the efdemo Encapsulated Functions TBF binary:
        let efdemo_binary = encapfn_tock::binary::EncapfnBinary::find(
            "ef_littlefs",
            core::slice::from_raw_parts(
                &_sapps as *const u8,
                &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
            ),
        )
        .unwrap();

        let (rt, mut alloc, mut access) = unsafe {
            encapfn_tock::rv32i_c_rt::TockRv32iCRt::new(
                kernel::platform::chip::Chip::mpu(chip),
                efdemo_binary,
                core::ptr::addr_of_mut!(_efram_start) as *mut (),
                core::ptr::addr_of!(_efram_end) as usize
                    - core::ptr::addr_of!(_efram_start) as usize,
                // Expose no addl. MPU regions:
                [].into_iter(),
                brand,
            )
        }
        .unwrap();

        // Create a "bound" runtime
        let bound_rt = encapfn_tock_littlefs::ef_littlefs_bindings::LibLittleFSRt::new(&rt).unwrap();

        debug!("About to test_libdemo with the Rv32iCRt!");

        // Run a test:
        encapfn_tock_littlefs::test_littlefs(&bound_rt, &mut alloc, &mut access);
        debug!("Ran test_libdemo with the Rv32iCRt!");
        debug!("");
    });

    // Acquire required capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    // ---------- PROCESS LOADING, SCHEDULER LOOP ----------

    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &mut *addr_of_mut!(PROCESSES),
        &FAULT_RESPONSE,
        &process_mgmt_cap,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_cap);
}
