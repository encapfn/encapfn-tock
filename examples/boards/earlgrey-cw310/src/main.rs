#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use earlgrey_board_lib::{ChipConfig, EarlGreyChip};
use encapfn::rt::EncapfnRt;
use kernel::platform::mpu;
use kernel::{capabilities, create_capability, static_init};

// // Must only be constructed once, which is what we guarantee with the "unsafe impl" below:
// struct OtCryptoLibHMACID;
// unsafe impl encapfn::branding::EFID for OtCryptoLibHMACID {}

// //type OTEncapfnRt = encapfn::rt::mock::MockRt<OtCryptoLibHMACID>;
// type OTEncapfnRt = encapfn_tock::rv32i_c_rt::TockRv32iCRt<
//     OtCryptoLibHMACID,
//     <EarlGreyChip as kernel::platform::chip::Chip>::MPU,
// >;

// type CryptolibHmacImpl = encapfn_example_otcrypto::ef_otcrypto_hmac::OtCryptoLibHMAC<
//     'static,
//     OtCryptoLibHMACID,
//     OTEncapfnRt,
//     encapfn_example_otcrypto::libotcrypto_bindings::LibOtCryptoRt<
//         'static,
//         OtCryptoLibHMACID,
//         OTEncapfnRt,
//     >,
// >;
type CryptolibHmacImpl = encapfn_example_otcrypto::unsafe_otcrypto_hmac::OtCryptoLibHMAC<'static>;

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    extern "C" {
        static _sapps: u8;
        static _eapps: u8;
        // /// Beginning of the RAM region for app memory.
        // static mut _sappmem: u8;
        // /// End of the RAM region for app memory.
        // static _eappmem: u8;
        // /// The start of the kernel text (Included only for kernel PMP)
        // static _stext: u8;
        // /// The end of the kernel text (Included only for kernel PMP)
        // static _etext: u8;
        // /// The start of the kernel / app / storage flash (Included only for kernel PMP)
        // static _sflash: u8;
        // /// The end of the kernel / app / storage flash (Included only for kernel PMP)
        // static _eflash: u8;
        // /// The start of the kernel / app RAM (Included only for kernel PMP)
        // static _ssram: u8;
        // /// The end of the kernel / app RAM (Included only for kernel PMP)
        // static _esram: u8;
        // /// The start of the OpenTitan manifest
        // static _manifest: u8;

        static _efram_start: u8;
        static _efram_end: u32;
    }

    let (board_kernel, earlgrey, chip, _peripherals) = earlgrey_board_lib::start();

    // //// Try to load the efdemo Encapsulated Functions TBF binary:
    // let ef_cryptolib_binary = encapfn_tock::binary::EncapfnBinary::find(
    //     "efotcrypto",
    //     core::slice::from_raw_parts(
    //         core::ptr::addr_of!(_sapps) as *const u8,
    //         core::ptr::addr_of!(_eapps) as *const u8 as usize
    //             - core::ptr::addr_of!(_sapps) as *const u8 as usize,
    //     ),
    // )
    // .unwrap();

    // // Additional MPU regions to expose to the Encapsulated Function:
    // let mpu_regions: [(mpu::Region, mpu::Permissions); 1] = [
    //     (
    //         // OpenTitan MMIO peripherals:
    //         mpu::Region::new(0x40000000 as *const _, 0x10000000),
    //         mpu::Permissions::ReadWriteOnly,
    //     ),
    //     // (
    //     //     // OpenTitan debug manager (RVDM) memory:
    //     //     mpu::Region::new(
    //     //         0x00010000 as *const _,
    //     //         0x00001000,
    //     //     ),
    //     //     mpu::Permissions::ReadWriteExecute,
    //     // )
    // ];

    // // This is unsafe, as it instantiates a runtime that can be used to run
    // // foreign functions without memory protection:
    // let (rt, alloc, access) = static_init!(
    //     (
    //         OTEncapfnRt,
    //         encapfn::types::AllocScope<
    //             'static,
    //             <OTEncapfnRt as EncapfnRt>::AllocTracker<'static>,
    //             OtCryptoLibHMACID,
    //         >,
    //         encapfn::types::AccessScope<OtCryptoLibHMACID>,
    //     ),
    //     encapfn_tock::rv32i_c_rt::TockRv32iCRt::new(
    //         kernel::platform::chip::Chip::mpu(chip),
    //         ef_cryptolib_binary,
    //         core::ptr::addr_of!(_efram_start) as *const () as *mut (),
    //         core::ptr::addr_of!(_efram_end) as usize - core::ptr::addr_of!(_efram_start) as usize,
    //         mpu_regions.into_iter(),
    //         OtCryptoLibHMACID,
    //     )
    //     .unwrap(),
    // );

    // let bound_rt = static_init!(
    //     encapfn_example_otcrypto::libotcrypto_bindings::LibOtCryptoRt<
    //         'static,
    //         OtCryptoLibHMACID,
    //         OTEncapfnRt,
    //     >,
    //     encapfn_example_otcrypto::libotcrypto_bindings::LibOtCryptoRt::new(rt).unwrap(),
    // );

    // SwitchImpls:
    // let ot_cryptolib_hmac = static_init!(
    //     CryptolibHmacImpl,
    //     encapfn_example_otcrypto::ef_otcrypto_hmac::OtCryptoLibHMAC::new(bound_rt, alloc, access)
    // );
    let ot_cryptolib_hmac = static_init!(
        CryptolibHmacImpl,
        encapfn_example_otcrypto::unsafe_otcrypto_hmac::OtCryptoLibHMAC::new()
    );
    kernel::deferred_call::DeferredCallClient::register(ot_cryptolib_hmac);

    let digest_buf = static_init!([u8; 32], [0xff; 32]);

    // TODO: this is bad! This is creating a second instance of this
    // hardware alarm, over the same hardware peripheral. It should be
    // OK for now, as we're currently just using it to same the
    // current time, which does not incur any register writes.
    let hardware_alarm = static_init!(
        earlgrey::timer::RvTimer<ChipConfig>,
        earlgrey::timer::RvTimer::new()
    );

    let hmac_bench = static_init!(
        encapfn_example_otcrypto::hmac_bench::HmacBench<
            'static,
            32,
            CryptolibHmacImpl,
            earlgrey::timer::RvTimer<'_, ChipConfig>,
        >,
        encapfn_example_otcrypto::hmac_bench::HmacBench::new(
            "Unsafe",
            ot_cryptolib_hmac,
            &[42; 512],
            8, // how many times to add the above buffer
            digest_buf,
            100, // how many overall benchmark iterations
            hardware_alarm,
        ),
    );
    kernel::hil::digest::Digest::set_client(ot_cryptolib_hmac, hmac_bench);

    hmac_bench.start();

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(earlgrey, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
}
