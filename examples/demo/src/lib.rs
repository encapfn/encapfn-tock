#![no_std]
#![feature(naked_functions)]

// Magic:
use encapfn::branding::EFID;
use encapfn::rt::EncapfnRt;
use encapfn::types::{AccessScope, AllocScope};

// Includes bindgen magic:
#[allow(non_upper_case_globals)]
pub mod libdemo {
    include!(concat!(env!("OUT_DIR"), "/libdemo_bindings.rs"));
}

use libdemo::LibDemo;

#[inline(never)]
pub fn test_libdemo<ID: EFID, RT: EncapfnRt<ID = ID>, L: LibDemo<ID, RT, RT = RT>>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
) {
    lib.rt()
        .allocate_stacked_t_mut::<[bool; 32], _, _>(alloc, |allocation, _alloc| {
            //let bool_array_ref = allocation.into_ref(alloc);
            let bool_array_val = allocation.write([false; 32], access);
            kernel::debug!("allocated array {:?}", *bool_array_val);
            let bool_array_ref = bool_array_val.as_ref();

            let bool_array_efptr = bool_array_ref.as_ptr();
            let bool_array_ptr: *mut [bool; 32] = bool_array_efptr.into();

            let ret = lib
                .demo_nop(1337, bool_array_ptr as *mut bool, access)
                .unwrap()
                .validate()
                .unwrap();
            kernel::debug!("demo_nop returned {}", ret);

            let bool_array_val = bool_array_ref.validate(access).unwrap();
            kernel::debug!("allocated array after invoke{:?}", *bool_array_val);
        })
        .unwrap();
    // prev alloc is valid again
}
