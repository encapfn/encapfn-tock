#![no_std]
#![feature(
    naked_functions,
    maybe_uninit_as_bytes,
    maybe_uninit_write_slice,
    offset_of_enum
)]

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TockEFError {
    BinaryLengthInvalid {
        min_expected: usize,
        actual: usize,
        desc: &'static str,
    },

    BinaryAlignError {
        expected: usize,
        actual: usize,
    },

    BinaryMagicInvalid,

    BinarySizeOverflow,

    MPUConfigError,

    EFError(encapfn::EFError),
}

impl From<encapfn::EFError> for TockEFError {
    fn from(ef_error: encapfn::EFError) -> Self {
        TockEFError::EFError(ef_error)
    }
}

pub mod binary;
pub mod rv32i_c_rt;

// Helper for benchmarks:
pub fn print_ogbench_result<T: kernel::hil::time::Time, E: core::fmt::Debug>(
    label: &str,
    elements: Option<E>,
    measurement: (usize, T::Ticks, T::Ticks),
    time: &T,
) {
    use kernel::hil::time::{Ticks, ConvertTicks};

    struct CondDisplay<T: core::fmt::Display>(bool, T);
    impl<T: core::fmt::Display> core::fmt::Display for CondDisplay<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if self.0 {
                core::fmt::Display::fmt(&self.1, f)
            } else {
                Ok(())
            }
        }
    }
    
    struct CondDebug<T: core::fmt::Debug, F: Fn() -> T>(bool, F, core::marker::PhantomData<T>);
    impl<T: core::fmt::Debug, F: Fn() -> T> core::fmt::Debug for CondDebug<T, F> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if self.0 {
                core::fmt::Debug::fmt(&self.1(), f)
            } else {
                Ok(())
            }
        }
    }


    let (iters, start, end) = measurement;
    assert!(end > start);
    let ticks = end.wrapping_sub(start);
    let us = time.ticks_to_us(ticks);

    let elements_ref = &elements;
    kernel::debug!(
        "OGBENCH[{}{}{:?}]: TICKS={} US={} ITERS={} TICKSPERITER={} USPERITER={}",
        label,
        CondDisplay(elements_ref.is_some(), " ELEMS="),
        CondDebug(elements_ref.is_some(), || elements_ref.as_ref().unwrap(), core::marker::PhantomData),
        ticks.into_u32(),
        us,
        iters,
        (ticks.into_u32() as f32) / iters as f32,
        (us as f32) / iters as f32
    );
}

