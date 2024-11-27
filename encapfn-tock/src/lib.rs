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
