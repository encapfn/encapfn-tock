#![feature(naked_functions)]
#![no_std]

use core::slice;
use core::cell::Cell;

use kernel::hil::time::Time;
use kernel::debug;

use encapfn::efmutref_get_field;
use encapfn::branding::EFID;
use encapfn::types::{AccessScope, AllocScope, EFPtr, EFType, EFMutRef, EFCopy};
use encapfn::rt::{CallbackReturn, CallbackContext, EncapfnRt};

pub fn print_result<T: Time>(
    label: &str,
    elements: Option<usize>,
    measurement: (usize, T::Ticks, T::Ticks),
    time: &T,
) {
    use kernel::hil::time::{Ticks, ConvertTicks};

    let (iters, start, end) = measurement;
    assert!(end > start);
    let ticks = end.wrapping_sub(start);
    let us = time.ticks_to_us(ticks);
    kernel::debug!(
        "[{}({:?})]: {:?} ticks ({} us) for {} iters, {} ticks / iter, {} us / iter",
        label,
        elements,
        ticks,
        us,
        iters,
        (ticks.into_u32() as f32) / iters as f32,
        (us as f32) / iters as f32
    );
}

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod lwip_bindings {
    include!(concat!(env!("OUT_DIR"), "/liblwip_bindings.rs"));

    // TODO: integrate into the base library itself!
    impl<'a, ID: ::encapfn::branding::EFID, RT: ::encapfn::rt::EncapfnRt> LibLwipRt<'a, ID, RT> {
        pub fn lookup_symbol(&self, fixed_offset_symbol: usize) -> Option<*const ()> {
            self.rt.lookup_symbol(0, fixed_offset_symbol, &self.symbols)
        }
    }
}

use lwip_bindings::{LibLwip, LibLwipRt};

#[no_mangle]
pub extern "C" fn sys_now() -> u32 {
    1000
}

const ICMP_ECHO_ETH: &'static [u8] = b"\x02\x00\x00\x00\x00\x01\x02\x00\x00\x00\x00\x02\x08\x00E\x00\x00\x1c\x00\x01\x00\x00@\x01\xf7[\xc0\xa8\x01\x02\xc0\xa8\x012\x08\x00\xf7\xff\x00\x00\x00\x00";
const ICMP_ECHO_RESPONSE: [u8; 42] = [2, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 1, 8, 0, 69, 0, 0, 28, 0, 1, 0, 0, 255, 1, 56, 91, 192, 168, 1, 50, 192, 168, 1, 2, 0, 0, 255, 255, 0, 0, 0, 0];


#[rustfmt::skip]
#[inline(never)]
pub fn run<ID: EFID, RT: EncapfnRt<ID = ID>, L: LibLwip<ID, RT, RT = RT>, T: Time>(
    lw: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
    netif_input_symbol: *const (),
    etharp_output_symbol: *const (),
    label: &str,
) {
    lw.lwip_init(alloc, access).unwrap();

    // Allocate space for a network packet:
    lw.rt().allocate_stacked_t_mut::<lwip_bindings::netif, _, _>(alloc, |netif, alloc| {
        // Zero out the netif struct:
        netif.write_copy(&EFCopy::zeroed(), access);

        // Setup a received callback:
        let received_icmp_echo_response_packets = Cell::new(0);
        lw.rt().setup_callback(&mut |ctx, _ret, alloc, access| {
            let pbuf_ptr = ctx.get_argument_register(1).unwrap() as *mut lwip_bindings::pbuf;
            let pbuf = EFPtr::from(pbuf_ptr).upgrade_mut(alloc).unwrap();

            let buffer_len: u16 = *(
                unsafe { efmutref_get_field!(
                    lwip_bindings::pbuf,
                    u16,
                    pbuf,
                    len
                ) }
            ).validate(access).unwrap();

            let payload_c_void_ptr: *mut core::ffi::c_void = *(
                unsafe { efmutref_get_field!(
                    lwip_bindings::pbuf,
                    *mut ::core::ffi::c_void,
                    pbuf,
                    payload
                ) }
            ).validate(access).unwrap();

            let payload_slice = EFPtr::from(payload_c_void_ptr as *mut u8)
                .upgrade_slice(buffer_len as usize, alloc)
                .unwrap()
                .validate(access)
                .unwrap();

            // Verify that we received an echo response packet:
            if &*payload_slice != ICMP_ECHO_RESPONSE {
                panic!("Received unknown packet {:?}: {:x?}", buffer_len, &*payload_slice);
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
                let netif_ref: EFMutRef<_, lwip_bindings::netif> =
                    EFPtr::from(ctx.get_argument_register(0).unwrap() as *mut _)
                    .upgrade_mut(alloc).unwrap();

                let netif_hwaddr_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        [u8; 6],
                        netif_ref,
                        hwaddr
                    )
                };
                netif_hwaddr_ref.write([0x02, 0x00, 0x00, 0x00, 0x00, 0x01], access);

                let netif_hwaddr_len_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        u8,
                        netif_ref,
                        hwaddr_len
                    )
                };
                netif_hwaddr_len_ref.write(6, access);

                let netif_name_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        [u8; 2],
                        netif_ref,
                        name
                    )
                };
                netif_name_ref.copy_from_slice(b"e0", access);

                let netif_flags_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        u8,
                        netif_ref,
                        flags
                    )
                };
                netif_flags_ref.write(
                    (lwip_bindings::NETIF_FLAG_BROADCAST
                     | lwip_bindings::NETIF_FLAG_ETHARP
                     | lwip_bindings::NETIF_FLAG_ETHERNET
                     | lwip_bindings::NETIF_FLAG_IGMP
                     | lwip_bindings::NETIF_FLAG_MLD6) as u8,
                    access,
                );

                let netif_mtu_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        u16,
                        netif_ref,
                        mtu
                    )
                };
                netif_mtu_ref.write(1500, access);

                let netif_ip_addr_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        lwip_bindings::ip_addr_t,
                        netif_ref,
                        ip_addr
                    )
                };
                lw.make_ip_addr_t(
                    netif_ip_addr_ref.as_ptr().into(),
                    192, 168, 1, 50,
                    alloc, access
                ).unwrap();

                let netif_netmask_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        lwip_bindings::ip_addr_t,
                        netif_ref,
                        netmask
                    )
                };
                lw.make_ip_addr_t(
                    netif_netmask_ref.as_ptr().into(),
                    255, 255, 255, 0,
                    alloc, access
                ).unwrap();

                let netif_gw_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        lwip_bindings::ip_addr_t,
                        netif_ref,
                        gw
                    )
                };
                lw.make_ip_addr_t(
                    netif_gw_ref.as_ptr().into(),
                    192, 168, 1, 1,
                    alloc, access
                ).unwrap();

                let netif_output_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        lwip_bindings::netif_output_fn,
                        netif_ref,
                        output
                    )
                };
                netif_output_ref.write(
                    unsafe {
                        core::mem::transmute::<
                            _,
                            Option<unsafe extern "C" fn(
                                *mut lwip_bindings::netif,
                                *mut lwip_bindings::pbuf,
                                *const lwip_bindings::ip4_addr
                            ) -> i8>
                        >(etharp_output_symbol)
                    },
                    access
                );

                let netif_linkoutput_ref = unsafe {
                    efmutref_get_field!(
                        lwip_bindings::netif,
                        lwip_bindings::netif_linkoutput_fn,
                        netif_ref,
                        linkoutput
                    )
                };
                netif_linkoutput_ref.write(
                    unsafe {
                        core::mem::transmute::<
                            *const extern "C" fn(),
                            Option<unsafe extern "C" fn(
                                *mut lwip_bindings::netif,
                                *mut lwip_bindings::pbuf
                            ) -> i8>,
                        >(linkoutput_cb as *const _)
                    },
                    access
                );

                debug!("Called netif_init callback!");

                ret.set_return_register(0, 0);
            }, alloc, |netif_init_cb, alloc| {
                let netif_ptr: *mut lwip_bindings::netif = netif.as_ptr().into();
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
                            Option<unsafe extern "C" fn(
                                *mut lwip_bindings::netif
                            ) -> i8>
                        >(netif_init_cb as *const _)
                    },
                    unsafe {
                        core::mem::transmute::<
                            *const extern "C" fn(),
                            Option<unsafe extern "C" fn(
                                *mut lwip_bindings::pbuf,
                                *mut lwip_bindings::netif
                            ) -> i8>,
                        >(netif_input_symbol as *const _)
                    },
                    alloc,
                    access,
                ).unwrap();

                debug!("netif_add result: {:?}", result.validate());
            }).unwrap();

            let set_default_result = lw
                .netif_set_default(netif.as_ptr().into(), alloc, access)
                .unwrap();
            debug!("netif_set_default {:?}", set_default_result.validate());

            let set_up_result = lw
                .netif_set_up(netif.as_ptr().into(), alloc, access)
                .unwrap();
            debug!("netif_set_up {:?}", set_up_result.validate());

            // This would normally be in the init callback, but the PMP Rt
            // currently doesn't handle nested invokes well, so put it here
            // for now:
            let set_link_up_result = lw
                .netif_set_link_up(netif.as_ptr().into(), alloc, access)
                .unwrap();
            debug!("netif_set_link_up {:?}", set_link_up_result.validate());


            lw.rt().allocate_stacked_t_mut::<lwip_bindings::ip4_addr, _, _>(alloc, |ip4addr, alloc| {
                lw.make_ip4_addr_t(ip4addr.as_ptr().into(), 192, 168, 1, 2, alloc, access).unwrap();
                lw.rt().allocate_stacked_t_mut::<lwip_bindings::eth_addr, _, _>(alloc, |ethaddr, alloc| {
                    ethaddr.write(lwip_bindings::eth_addr { addr: [0x02, 0x00, 0x00, 0x00, 0x00, 0x02] }, access);
                    debug!("Add static arp entry result: {:?}", lw.etharp_add_static_entry(
                        ip4addr.as_ptr().into(),
                        ethaddr.as_ptr().into(),
                        alloc,
                        access
                    ).unwrap().validate());
                }).unwrap();
            }).unwrap();

            const ICMP_ECHO_REQ_CNT: usize = 10_000;
            let start = time.now();
            for _ in 0..ICMP_ECHO_REQ_CNT {
                let pbuf = lw
                    .pbuf_alloc(
                        lwip_bindings::pbuf_layer_PBUF_RAW,
                        42,
                        lwip_bindings::pbuf_type_PBUF_POOL,
                        alloc,
                        access,
                    )
                    .unwrap()
                    .validate()
                    .unwrap();

                lw.rt()
                    .allocate_stacked_t_mut::<[u8; 42], _, _>(alloc, |buf, alloc| {
                        buf.copy_from_slice(ICMP_ECHO_ETH, access);
                        lw.pbuf_take(
                            pbuf,
                            buf.as_ptr().0 as *const _,
                            ICMP_ECHO_ETH.len() as u16,
                            alloc,
                            access,
                        )
                            .unwrap();
                    })
                    .unwrap();

                assert_eq!(
                    0,
                    lw.netif_input(pbuf, netif.as_ptr().into(), alloc, access)
                        .unwrap()
                        .validate()
                        .unwrap(),
                );
            }
            let end = time.now();

            assert_eq!(ICMP_ECHO_REQ_CNT,
                   received_icmp_echo_response_packets.get(),);
            print_result(label, None, (ICMP_ECHO_REQ_CNT, start, end), time);
        }).unwrap();
    }).unwrap();
}
