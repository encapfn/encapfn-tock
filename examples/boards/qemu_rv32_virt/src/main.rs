// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use core::ptr::addr_of_mut;
use core::slice;

use encapfn::types::EFPtr;
use kernel::debug;
use kernel::{capabilities, create_capability};
use qemu_rv32_virt_lib::{self, PROCESSES};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

const ICMP_ECHO_ETH: [u8; 52] = [
0x00, 0x0c, 0x29, 0x7d, 0xae, 0xc7, 0x00, 0x50, 0x56, 0xc0, 0x00, 0x08, 0x08, 0x00, 0x45, 0x00,
0x00, 0x26, 0xba, 0x96, 0x00, 0x00, 0x40, 0x01, 0x3a, 0x6f, 0xc0, 0xa8, 0x02, 0x01, 0xc0, 0xa8,
0x02, 0x80, 0x08, 0x00, 0xee, 0xdd, 0x12, 0x04, 0x00, 0x00, 0x56, 0xdd, 0x33, 0xe1, 0x00, 0x01,
0x6b, 0x5c, 0x01, 0x02];

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
        use encapfn::rt::EncapfnRt;
        use lwip::LibLwip;
        // This is unsafe, as it instantiates a runtime that can be used to run
        // foreign functions without memory protection:
        let (rt, mut alloc, mut access) = unsafe {
            encapfn::rt::mock::MockRt::new(
                false,
                encapfn::rt::mock::stack_alloc::StackAllocator::<
                    encapfn::rt::mock::stack_alloc::StackFrameAllocRiscv,
                >::new(),
                brand,
            )
        };

        // Create a "bound" runtime, which implements the LibLwip API:
        let lw = lwip::LibLwipRt::new(&rt).unwrap();

        lw.lwip_init(&mut access).unwrap();

        extern "C" fn netif_output(_netif: *mut lwip::netif, buf: *mut lwip::pbuf, ipaddr: *const lwip::ip4_addr) -> i8 {
	    debug!("Output!!!");
	    0
	}

        extern "C" fn netif_linkoutput(_netif: *mut lwip::netif, buf: *mut lwip::pbuf) -> i8 {
            let buf = unsafe { &mut *buf };
            let payload =
                unsafe { slice::from_raw_parts(buf.payload as *const u8, buf.len as usize) };
            debug!("Payload {:?}", payload);
            0
        }

        extern "C" fn netif_init(netif: *mut lwip::netif) -> i8 {
            debug!("Initializing {:?}", netif);
            let netif = unsafe { &mut *netif };
	    //netif.output = Some(lwip::etharp_output);
	    netif.output = Some(netif_output);
            netif.linkoutput = Some(netif_linkoutput);
	    netif.hwaddr = [0x00, 0x0c, 0x29, 0x7d, 0xae, 0xc7];
	    netif.hwaddr_len = 6;
            netif.name = [b'e' as i8, b'0' as i8];
	    netif.flags      = (lwip::NETIF_FLAG_BROADCAST | lwip::NETIF_FLAG_ETHARP | lwip::NETIF_FLAG_ETHERNET | lwip::NETIF_FLAG_IGMP | lwip::NETIF_FLAG_MLD6) as u8;
	    netif.mtu = 1500;
            0
        }

        lw.rt()
            .allocate_stacked_t_mut::<lwip::netif, _, _>(&mut alloc, |netif, mut alloc2| {
                lw.rt()
                    .allocate_stacked_t_mut::<lwip::ip4_addr, _, _>(
                        &mut alloc2,
                        |ipaddr, _alloc3| {
                            ipaddr.write(lwip::ip4_addr { addr: 0 }, &mut access);
                            let state: *mut core::ffi::c_void = 0 as *mut _;
                            let result = lw
                                .netif_add(
                                    netif.as_ptr().into(),
                                    ipaddr.as_ptr().into(),
                                    ipaddr.as_ptr().into(),
                                    ipaddr.as_ptr().into(),
                                    state,
                                    Some(netif_init),
                                    Some(lwip::netif_input),
                                    &mut access,
                                )
                                .unwrap();
                            debug!("{:?}", result.validate());
                        },
                    )
                    .unwrap();
                let set_default_result = lw
                    .netif_set_default(netif.as_ptr().into(), &mut access)
                    .unwrap();
                debug!("netif_set_default {:?}", set_default_result.validate());

                let set_up_result = lw.netif_set_up(netif.as_ptr().into(), &mut access).unwrap();
                debug!("netif_set_up {:?}", set_up_result.validate());
		debug!("DHCP {:?}", lw.dhcp_start(netif.as_ptr().into(), &mut access).unwrap().validate());
		let pbuf = lw.pbuf_alloc(lwip::pbuf_layer_PBUF_RAW, 52, lwip::pbuf_type_PBUF_POOL, &mut access).unwrap().validate().unwrap();


                lw.rt()
                    .allocate_stacked_t_mut::<[u8; 52], _, _>(
                        &mut alloc2,
                        |buf, _alloc3| {
			    buf.write(ICMP_ECHO_ETH, &mut access);
			    lw.pbuf_take(pbuf, buf.as_ptr().0 as *const _, ICMP_ECHO_ETH.len() as u16, &mut access).unwrap();
			}).unwrap();

		debug!("{:?}", lw.netif_input(pbuf, netif.as_ptr().into(), &mut access).unwrap().validate());

            })
            .unwrap();

        let netif = EFPtr::<lwip::netif>::from(
            lw.netif_get_by_index(1, &mut access)
                .unwrap()
                .validate()
                .unwrap(),
        )
        .upgrade_unchecked()
        .validate(&mut access)
        .unwrap();
        debug!("{:?}", netif.name.map(|b| b as u8 as char));
        let netif = EFPtr::<lwip::netif>::from(
            lw.netif_get_by_index(2, &mut access)
                .unwrap()
                .validate()
                .unwrap(),
        )
        .upgrade_unchecked()
        .validate(&mut access)
        .unwrap();
        debug!("{:?}", netif.name.map(|b| b as u8 as char));

	#[no_mangle]
	pub extern "C" fn sys_now() -> u32 {
	    1000
	}

	lw.sys_check_timeouts(&mut access);
    });

    /*encapfn::branding::new(|brand| {
        // This is unsafe, as it instantiates a runtime that can be used to run
        // foreign functions without memory protection:
        let (rt, mut alloc, mut access) = unsafe {
            encapfn::rt::mock::MockRt::new(
                false,
                encapfn::rt::mock::stack_alloc::StackAllocator::<
                    encapfn::rt::mock::stack_alloc::StackFrameAllocRiscv,
                >::new(),
                brand,
            )
        };

        // Create a "bound" runtime, which implements the LibDemo API:
        let bound_rt = encapfn_example_demo::libdemo::LibDemoRt::new(&rt).unwrap();

        // Run a test:
        encapfn_example_demo::test_libdemo(&bound_rt, &mut alloc, &mut access);
        debug!("Ran test_libdemo with the MockRt!");
    });

    encapfn::branding::new(|brand| {
        // Try to load the efdemo Encapsulated Functions TBF binary:
        let efdemo_binary = encapfn_tock::binary::EncapfnBinary::find(
            "efdemo",
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

        // Create a "bound" runtime, which implements the LibDemo API:
        let bound_rt = encapfn_example_demo::libdemo::LibDemoRt::new(&rt).unwrap();

        // Run a test:
        encapfn_example_demo::test_libdemo(&bound_rt, &mut alloc, &mut access);
        debug!("Ran test_libdemo with the Rv32iCRt!");
    });*/

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
