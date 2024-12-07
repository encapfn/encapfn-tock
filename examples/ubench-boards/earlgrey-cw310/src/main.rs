#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use earlgrey_board_lib::{ChipConfig, EarlGreyChip};
use encapfn::rt::EncapfnRt;
use kernel::platform::mpu;
use kernel::{capabilities, create_capability, static_init};

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

    encapfn::branding::new(|brand| {
        // Try to load the efdemo Encapsulated Functions TBF binary:
        let efdemo_binary = encapfn_tock::binary::EncapfnBinary::find(
            "ef_ubench",
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
        let bound_rt = encapfn_tock_ubench::libdemo::LibDemoRt::new(&rt).unwrap();

        // TODO: this is bad! This is creating a second instance of this
        // hardware alarm, over the same hardware peripheral. It should be
        // OK for now, as we're currently just using it to same the
        // current time, which does not incur any register writes.
        let hardware_alarm = static_init!(
            earlgrey::timer::RvTimer<ChipConfig>,
            earlgrey::timer::RvTimer::new()
        );

        // Invoke benchmarks:
        // encapfn_tock_ubench::run_ubench_invoke(&bound_rt, &mut alloc, &mut access, hardware_alarm, &mut || {
        //     kernel::platform::chip::Chip::mpu(chip).request_reconfiguration()
        // });

        // Validate benchmarks:
        // encapfn_tock_ubench::run_ubench_validate_bytes(&bound_rt, &mut alloc, &mut access, hardware_alarm);
        // encapfn_tock_ubench::run_ubench_validate_str(&bound_rt, &mut alloc, &mut access, hardware_alarm);

        // Upgrade benchmark:
        // encapfn_tock_ubench::run_ubench_upgrade(&bound_rt, &mut alloc, &mut access, hardware_alarm);

        // Callback benchmark:
        encapfn_tock_ubench::run_ubench_callback(
            &bound_rt,
            &mut alloc,
            &mut access,
            hardware_alarm,
        );
    });

    // -------------------------------------------------------------------------
    // Setup time benchmark:
    // use kernel::hil::time::Time;

    // // TODO: this is bad! This is creating a second instance of this
    // // hardware alarm, over the same hardware peripheral. It should be
    // // OK for now, as we're currently just using it to same the
    // // current time, which does not incur any register writes.
    // let hardware_alarm = static_init!(
    //     earlgrey::timer::RvTimer<ChipConfig>,
    //     earlgrey::timer::RvTimer::new()
    // );

    // const SETUP_ITERS: usize = 10_000;

    // let start_unsafe = hardware_alarm.now();
    // for _ in 0..SETUP_ITERS {
    // 	encapfn_tock_ubench::bench_args_unsafe::<0>();
    // }
    // let end_unsafe = hardware_alarm.now();

    // let start_ef = hardware_alarm.now();
    // for _ in 0..SETUP_ITERS {
    // 	encapfn::branding::new(|brand| {
    //         // Try to load the efdemo Encapsulated Functions TBF binary:
    //         let efdemo_binary = encapfn_tock::binary::EncapfnBinary::find(
    // 		"ef_ubench",
    // 		core::slice::from_raw_parts(
    //                 &_sapps as *const u8,
    //                 &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
    // 		),
    //         )
    // 		.unwrap();

    //         let (rt, mut alloc, mut access) = unsafe {
    // 		encapfn_tock::rv32i_c_rt::TockRv32iCRt::new(
    //                 kernel::platform::chip::Chip::mpu(chip),
    //                 efdemo_binary,
    //                 core::ptr::addr_of_mut!(_efram_start) as *mut (),
    //                 core::ptr::addr_of!(_efram_end) as usize
    // 			- core::ptr::addr_of!(_efram_start) as usize,
    //                 // Expose no addl. MPU regions:
    //                 [].into_iter(),
    //                 brand,
    // 		)
    //         }
    //         .unwrap();

    //         // Create a "bound" runtime
    //         let bound_rt = encapfn_tock_ubench::libdemo::LibDemoRt::new(&rt).unwrap();

    //         // Run a single function:
    // 	    encapfn_tock_ubench::bench_args_ef::<0, _, _, _>(&bound_rt, &mut alloc, &mut access);
    // 	});
    // }
    // let end_ef = hardware_alarm.now();

    // encapfn_tock_ubench::print_result("setup_unsafe", None, (SETUP_ITERS, start_unsafe, end_unsafe), hardware_alarm);
    // encapfn_tock_ubench::print_result("setup_ef", None, (SETUP_ITERS, start_ef, end_ef), hardware_alarm);
    // -------------------------------------------------------------------------

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(earlgrey, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
}
