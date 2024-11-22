#![feature(naked_functions)]
#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use encapfn::types::EFType;

include!(concat!(env!("OUT_DIR"), "/liblwip_bindings.rs"));

unsafe impl EFType for netif {
    unsafe fn validate(_: *const Self) -> bool {
        true
    }
}

unsafe impl EFType for pbuf {
    unsafe fn validate(_: *const Self) -> bool {
        true
    }
}
