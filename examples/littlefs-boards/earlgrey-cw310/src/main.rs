#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use earlgrey_board_lib::{ChipConfig, EarlGreyChip};
use encapfn::rt::EncapfnRt;
use kernel::platform::mpu;
use kernel::{capabilities, create_capability, static_init};
use kernel::debug;

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    extern "C" {
        static _sapps: u8;
        static _eapps: u8;

        static mut _efram_start: u8;
        static mut _efram_end: u32;
    }

    let (board_kernel, earlgrey, chip, _peripherals) = earlgrey_board_lib::start();

    // TODO: this is bad! This is creating a second instance of this
    // hardware alarm, over the same hardware peripheral. It should be
    // OK for now, as we're currently just using it to same the
    // current time, which does not incur any register writes.
    let hardware_alarm = static_init!(
        earlgrey::timer::RvTimer<ChipConfig>,
        earlgrey::timer::RvTimer::new()
    );

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
        let bound_rt = encapfn_tock_littlefs::littlefs_bindings::LibLittleFSRt::new(&rt).unwrap();

        // Run a test:
        encapfn_tock_littlefs::test_littlefs("mock", &bound_rt, &mut alloc, &mut access, hardware_alarm);
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
        let bound_rt = encapfn_tock_littlefs::littlefs_bindings::LibLittleFSRt::new(&rt).unwrap();

        // Run a test:
        encapfn_tock_littlefs::test_littlefs("ef_mpk", &bound_rt, &mut alloc, &mut access, hardware_alarm);
    });

    // Load-bearing, otherwise the binary doesn't fit in flash
    panic!();

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(earlgrey, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
}
