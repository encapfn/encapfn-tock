#![no_std]
#![feature(naked_functions)]

// Magic:
use encapfn::branding::EFID;
use encapfn::rt::EncapfnRt;
use encapfn::types::{AccessScope, AllocScope};

// Includes bindgen magic:
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
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
        .allocate_stacked_t_mut::<[bool; 32], _, _>(alloc, |allocation, alloc| {
            //let bool_array_ref = allocation.into_ref(alloc);
            let bool_array_val = allocation.write([false; 32], access);
            kernel::debug!("allocated array {:?}", *bool_array_val);
            let bool_array_ref = bool_array_val.as_ref();

            let bool_array_efptr = bool_array_ref.as_ptr();
            let bool_array_ptr: *mut [bool; 32] = bool_array_efptr.into();

            let ret = lib
                .demo_nop(1337, bool_array_ptr as *mut bool, alloc, access)
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

#[inline(never)]
pub fn test_libdemo_callback<ID: EFID, RT: EncapfnRt<ID = ID>, L: LibDemo<ID, RT, RT = RT>>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
) {
    let mut callback_called_counter: usize = 0;

    lib.rt()
        .setup_callback(
            &mut |ctx, _ret, _alloc, _access| {
                callback_called_counter += 1;
                if callback_called_counter == 3 {
                    panic!("Callback called third time, context: {:x?}", ctx);
                }
            },
            alloc,
            |callback_ptr, alloc| {
                lib.demo_invoke_callback(
                    // Change to a 3 to trigger the above panic:
                    2,
                    // 3,
                    unsafe {
                        // TODO: provide a safe method to perform this cast
                        core::mem::transmute::<
                            *const extern "C" fn(),
                            Option<unsafe extern "C" fn(usize, usize, usize)>,
                        >(callback_ptr as *const _)
                    },
                    // None,
                    alloc,
                    access,
                )
                .unwrap()
                .validate()
                .unwrap();
            },
        )
        .unwrap();

    kernel::debug!("Callback called {} times", callback_called_counter);
}
