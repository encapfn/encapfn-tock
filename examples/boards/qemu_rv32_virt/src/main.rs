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
use core::cell::Cell;

use encapfn::efmutref_get_field;
use encapfn::rt::{CallbackContext, CallbackReturn};
use encapfn::types::{EFMutRef, EFPtr};
use kernel::debug;
use kernel::{capabilities, create_capability};
use qemu_rv32_virt_lib::{self, PROCESSES};

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

/* const ICMP_ECHO_ETH: [u8; 52] = [
0x00, 0x0c, 0x29, 0x7d, 0xae, 0xc7, 0x00, 0x50, 0x56, 0xc0, 0x00, 0x08, 0x08, 0x00, 0x45, 0x00,
0x00, 0x26, 0xba, 0x96, 0x00, 0x00, 0x40, 0x01, 0x3a, 0x6f, 0xc0, 0xa8, 0x02, 0x01, 0xc0, 0xa8,
0x02, 0x80, 0x08, 0x00, 0xee, 0xdd, 0x12, 0x04, 0x00, 0x00, 0x56, 0xdd, 0x33, 0xe1, 0x00, 0x01,
0x6b, 0x5c, 0x01, 0x02]; */


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
        // let (rt, mut alloc, mut access) = unsafe {
        //     encapfn::rt::mock::MockRt::new(
        //         false, // zero_copy_immutable
	//         true, // all_upgrades_valid
        //         encapfn::rt::mock::stack_alloc::StackAllocator::<
        //             encapfn::rt::mock::stack_alloc::StackFrameAllocRiscv,
        //         >::new(),
        //         brand,
        //     )
        // };

	let eflwip_binary = encapfn_tock::binary::EncapfnBinary::find(
            "eflwip",
            core::slice::from_raw_parts(
                &_sapps as *const u8,
                &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
            ),
        ).unwrap();

	let (rt, mut alloc, mut access) = unsafe {
            encapfn_tock::rv32i_c_rt::TockRv32iCRt::new(
                kernel::platform::chip::Chip::mpu(chip),
                eflwip_binary,
                core::ptr::addr_of_mut!(_efram_start) as *mut (),
                core::ptr::addr_of!(_efram_end) as usize
                    - core::ptr::addr_of!(_efram_start) as usize,
                // Expose no addl. MPU regions:
                [].into_iter(),
                brand,
            )
        }
        .unwrap();


        // Create a "bound" runtime, which implements the LibLwip API:
        let lw = lwip::LibLwipRt::new(&rt).unwrap();
        lw.lwip_init(&mut alloc, &mut access).unwrap();

        // Allocate space for a network packet:
        lw.rt().allocate_stacked_t_mut::<lwip::netif, _, _>(&mut alloc, |netif, mut alloc| {
            // Setup a received callback:
	    let received_icmp_echo_response_packets = Cell::new(0);
            lw.rt().setup_callback(&mut |ctx, _alloc, _access, _arg| {
                let pbuf_reg = ctx.get_argument_register(1).unwrap() as *mut lwip::pbuf;
                let buffer_len = (*pbuf_reg).len;
                let buf = unsafe {
                    slice::from_raw_parts(
                        (*pbuf_reg).payload as *const u8,
                        buffer_len as usize,
                    )
                };
		// Verify that we received an echo response packet:
		if buf != ICMP_ECHO_RESPONSE {
		    panic!("Received unknown packet {:?}: {:?}", buffer_len, buf);
		} else {
                    // debug!("Received packet of length {:?}: {:?}", buffer_len, buf);
		    received_icmp_echo_response_packets.set(
			received_icmp_echo_response_packets.get()
			    + 1
		    );
		}
            },
            alloc,
            |linkoutput_cb, alloc| {
		// Prepare a "netif_init_ callback:
		lw.rt().setup_callback(&mut |ctx, ret, alloc, access| {
		    // netif_init callback:
		    let netif: EFMutRef<_, lwip::netif> =
			EFPtr::from(ctx.get_argument_register(0).unwrap() as *mut _)
			.upgrade_mut(alloc).unwrap();

		    let netif: *mut lwip::netif = netif.as_ptr().into();
		    let netif: &mut lwip::netif = unsafe { &mut *(netif) };
		    // debug!("Initializing netif {:p}", netif);

		    netif.hwaddr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
		    netif.hwaddr_len = 6;
		    netif.name = [b'e' as i8, b'0' as i8];
		    netif.flags = (lwip::NETIF_FLAG_BROADCAST
				   | lwip::NETIF_FLAG_ETHARP
				   | lwip::NETIF_FLAG_ETHERNET
				   | lwip::NETIF_FLAG_IGMP
				   | lwip::NETIF_FLAG_MLD6) as u8;
		    netif.mtu = 1500;


		    lw.make_ip_addr_t(&mut netif.ip_addr as *mut _, 192, 168, 1, 50, alloc, access).unwrap();
		    lw.make_ip_addr_t(&mut netif.netmask as *mut _, 255, 255, 255, 0, alloc, access).unwrap();
		    lw.make_ip_addr_t(&mut netif.gw as *mut _, 192, 168, 1, 1, alloc, access).unwrap();

		    // let netif_output_ref: *mut lwip::netif_output_fn =
                    //     efmutref_get_field!(
                    //         lwip::netif,
                    //         lwip::netif_output_fn,
                    //         netif,
                    //         output
                    //     )
                    //     .as_ptr()
                    //     .into();
                    // netif_output_ref = Some(lwip::etharp_output);
		    // netif.output = Some(lwip::etharp_output);
		    let etharp_output_symbol: *const () = lw.lookup_symbol(14).unwrap();
		    netif.output = unsafe {
			core::mem::transmute::<
		            _,
			    Option<unsafe extern "C" fn(
				*mut lwip::netif, *mut lwip::pbuf, *const lwip::ip4_addr) -> i8>
		        >(etharp_output_symbol)
		    };

                    // let netif_linkout_ref: *mut lwip::netif_linkoutput_fn =
                    //     efmutref_get_field!(
                    //         lwip::netif,
                    //         lwip::netif_linkoutput_fn,
                    //         netif,
                    //         linkoutput
                    //     )
                    //     .as_ptr()
                    //     .into();
		    // netif_linkout_ref.write(...)
                    netif.linkoutput = core::mem::transmute::<
                        *const extern "C" fn(),
                        Option<unsafe extern "C" fn(*mut lwip::netif, *mut lwip::pbuf) -> i8>,
	            >(linkoutput_cb as *const _);

		    ret.set_return_register(0, 0);
		}, alloc, |netif_init_cb, alloc| {
		    let netif_ptr: *mut lwip::netif = netif.as_ptr().into();
		    debug!("Adding netif: {:?}", netif_ptr);
                    let result = lw.netif_add(
                        netif.as_ptr().into(),
                        core::ptr::null_mut(), // ipaddr
                        core::ptr::null_mut(), // netmask
                        core::ptr::null_mut(), // gateway
                        core::ptr::null_mut(), // state
                        unsafe {
			    core::mem::transmute::<
				*const extern "C" fn(),
				Option<unsafe extern "C" fn(*mut lwip::netif) -> i8>
		            >(netif_init_cb as *const _)
			},
                        Some(lwip::netif_input),
                        alloc,
                        &mut access,
                    ).unwrap();
                    debug!("netif_add result: {:?}", result.validate());
                }).unwrap();

                let set_default_result = lw
                    .netif_set_default(netif.as_ptr().into(), alloc, &mut access)
                    .unwrap();
                debug!("netif_set_default {:?}", set_default_result.validate());

                let set_up_result = lw
                    .netif_set_up(netif.as_ptr().into(), alloc, &mut access)
                    .unwrap();
                debug!("netif_set_up {:?}", set_up_result.validate());

		// This would normally be in the init callback, but the PMP Rt
		// currently doesn't handle nested invokes well, so put it here
		// for now:
		let set_link_up_result = lw
                    .netif_set_link_up(netif.as_ptr().into(), alloc, &mut access)
                    .unwrap();
                debug!("netif_set_up {:?}", set_link_up_result.validate());


		lw.rt().allocate_stacked_t_mut::<lwip::ip4_addr, _, _>(alloc, |ip4addr, alloc| {
		    lw.make_ip4_addr_t(ip4addr.as_ptr().into(), 192, 168, 1, 2, alloc, &mut access).unwrap();
		    lw.rt().allocate_stacked_t_mut::<lwip::eth_addr, _, _>(alloc, |ethaddr, alloc| {
			ethaddr.write(lwip::eth_addr { addr: [0x02, 0x00, 0x00, 0x00, 0x00, 0x02] }, &mut access);
			debug!("Add static arp entry result: {:?}", lw.etharp_add_static_entry(
			    ip4addr.as_ptr().into(),
			    ethaddr.as_ptr().into(),
			    alloc,
			    &mut access
			).unwrap().validate());
		    }).unwrap();
		}).unwrap();

		const ICMP_ECHO_REQ_CNT: usize = 10000;
		for _ in 0..ICMP_ECHO_REQ_CNT {
                    let pbuf = lw
			.pbuf_alloc(
                            lwip::pbuf_layer_PBUF_RAW,
                            42,
                            lwip::pbuf_type_PBUF_POOL,
                            alloc,
                            &mut access,
			)
			.unwrap()
			.validate()
			.unwrap();

                    lw.rt()
			.allocate_stacked_t_mut::<[u8; 42], _, _>(alloc, |buf, alloc| {
                            buf.copy_from_slice(ICMP_ECHO_ETH, &mut access);
                            lw.pbuf_take(
				pbuf,
				buf.as_ptr().0 as *const _,
				ICMP_ECHO_ETH.len() as u16,
				alloc,
				&mut access,
                            )
				.unwrap();
			})
			.unwrap();

                    assert_eq!(
			0,
			lw.netif_input(pbuf, netif.as_ptr().into(), alloc, &mut access)
                            .unwrap()
                            .validate()
			    .unwrap(),
                    );
		}
		panic!("ICMP echo requests sent: {}, responses received: {}",
		       ICMP_ECHO_REQ_CNT,
		       received_icmp_echo_response_packets.get(),
		);

                // let netif = EFPtr::<lwip::netif>::from(
                //     lw.netif_get_by_index(1, alloc, &mut access)
                //         .unwrap()
                //         .validate()
                //         .unwrap(),
                // )
                //     .upgrade_unchecked()
                //     .validate(&mut access)
                //     .unwrap();
                // debug!("{:?}", netif.name.map(|b| b as u8 as char));

                // let netif = EFPtr::<lwip::netif>::from(
                //     lw.netif_get_by_index(2, alloc, &mut access)
                //         .unwrap()
                //         .validate()
                //         .unwrap(),
                // )
                //     .upgrade_unchecked()
                //     .validate(&mut access)
                //     .unwrap();
                // debug!("{:?}", netif.name.map(|b| b as u8 as char));

                #[no_mangle]
                pub extern "C" fn sys_now() -> u32 {
                    1000
                }

                // lw.sys_check_timeouts(alloc, &mut access);
            })
        })
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
