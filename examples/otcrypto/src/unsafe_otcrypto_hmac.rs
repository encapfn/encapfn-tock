use core::cell::RefCell;

use kernel::debug;
use kernel::deferred_call::DeferredCall;
use kernel::hil::digest;
use kernel::hil::digest::DigestHash;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

use encapfn::branding::EFID;
use encapfn::rt::EncapfnRt;
use encapfn::types::{AccessScope, AllocScope, EFCopy, EFMutRef};

use crate::libotcrypto_bindings::{self, LibOtCrypto};

const SHA_256_OUTPUT_LEN_BYTES: usize = 32;

pub struct OtCryptoLibHMAC<'a> {
    hmac_context: RefCell<libotcrypto_bindings::otcrypto_hmac_context_t>,
    data_slice: OptionalCell<SubSlice<'static, u8>>,
    data_slice_mut: OptionalCell<SubSliceMut<'static, u8>>,
    digest_buf: TakeCell<'static, [u8; SHA_256_OUTPUT_LEN_BYTES]>,
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn digest::Client<SHA_256_OUTPUT_LEN_BYTES>>,
}

impl OtCryptoLibHMAC<'_> {
    pub fn new() -> Self {
        OtCryptoLibHMAC {
            hmac_context: RefCell::new(libotcrypto_bindings::otcrypto_hmac_context_t {
                inner: libotcrypto_bindings::otcrypto_hash_context_t {
                    mode: 0,
                    data: [0; 52],
                },
                outer: libotcrypto_bindings::otcrypto_hash_context_t {
                    mode: 0,
                    data: [0; 52],
                },
            }),
            data_slice: OptionalCell::empty(),
            data_slice_mut: OptionalCell::empty(),
            digest_buf: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
        }
    }

    fn with_hmac_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut libotcrypto_bindings::otcrypto_hmac_context_t) -> R,
    {
        let mut stored_hmac_context = self.hmac_context.borrow_mut();
        f(&mut *stored_hmac_context)
    }

    fn add_data_int(&self, data: &[u8]) -> Result<(), ErrorCode> {
        let res = self.with_hmac_context(|hmac_context| {
            let msg_buf = libotcrypto_bindings::otcrypto_const_byte_buf_t {
                data: data.as_ptr(),
                len: data.len(),
            };
            //panic!("Adding msg buf: {}, {}, {:x?}, {:?}", data.len(), data_slice.len(), &msg_buf, &*data_slice.validate(access).unwrap());

            unsafe { libotcrypto_bindings::otcrypto_hmac_update(hmac_context as *mut _, msg_buf) };
        });

        Ok(())
    }
}

use kernel::deferred_call::DeferredCallClient;

impl DeferredCallClient for OtCryptoLibHMAC<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        match (
            self.data_slice.take(),
            self.data_slice_mut.take(),
            self.digest_buf.take(),
        ) {
            (Some(data_slice), None, None) =>
            /* data slice */
            {
                self.client
                    .map(move |c| c.add_data_done(Ok(()), data_slice));
            }

            (None, Some(data_slice_mut), None) =>
            /* data slice mut */
            {
                self.client
                    .map(move |c| c.add_mut_data_done(Ok(()), data_slice_mut));
            }

            (None, None, Some(digest_buf)) =>
            /* hash done */
            {
                self.client.map(move |c| c.hash_done(Ok(()), digest_buf));
            }

            (None, None, None) => {
                unimplemented!("Unexpected deferred call!");
            }

            _ => {
                unimplemented!("Unhandled deferred call or multiple outstanding!");
            }
        }
    }
}

// HMAC Driver
impl<'a> digest::Digest<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OtCryptoLibHMAC<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<32>) {
        self.client.replace(client);
    }
}

impl<'a> digest::DigestData<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OtCryptoLibHMAC<'a> {
    fn set_data_client(&'a self, client: &'a dyn digest::ClientData<32>) {
        // we do not set a client for this (this is the lowest layer)
        // mirroring hmac.rs in `chips/lowrisc/src`
        unimplemented!()
    }

    fn add_data(
        &self,
        mut data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        match self.add_data_int(data.as_slice()) {
            Err(_) => Err((ErrorCode::FAIL, data)),
            Ok(()) => {
                self.data_slice.replace(data);
                self.deferred_call.set();
                Ok(())
            }
        }
    }

    fn add_mut_data(
        &self,
        mut data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        match self.add_data_int(data.as_slice()) {
            Err(_) => Err((ErrorCode::FAIL, data)),
            Ok(()) => {
                self.data_slice_mut.replace(data);
                self.deferred_call.set();
                Ok(())
            }
        }
    }

    /// Clear the keys and any other internal state. Any pending
    /// operations terminate and issue a callback with an
    /// `ErrorCode::CANCEL`. This call does not clear buffers passed
    /// through `add_mut_data`, those are up to the client clear.
    fn clear_data(&self) {
        // it is not clear what internal state exists for encapsulated
        // functions / ot-crpyto. For now this is empty.
        unimplemented!();
    }
}

impl<'a> digest::DigestHash<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OtCryptoLibHMAC<'a> {
    fn set_hash_client(&'a self, client: &'a dyn digest::ClientHash<32>) {
        // see comment for dataclient
        unimplemented!()
    }
    #[inline(never)]
    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        self.with_hmac_context(|hmac_context| {
            let mut tag_array = [0_u32; 256 / 32];
            let tag_buf = libotcrypto_bindings::otcrypto_word32_buf_t {
                data: &mut tag_array as *mut [u32; 256 / 32] as *mut u32,
                len: tag_array.len(),
            };

            unsafe { libotcrypto_bindings::otcrypto_hmac_final(hmac_context as *mut _, tag_buf) };

            // Copy the validated array's contents into the digest buffer,
            // converting the u32s to u8s in the process:
            //panic!("Hash done tag_array_val: {:x?}", &*tag_array_val);
            tag_array
                .iter()
                .flat_map(|w| u32::to_be_bytes(*w))
                .zip(digest.iter_mut())
                .for_each(|(src, dst)| *dst = src);
        });

        // Store the digest slice and request a deferred call:
        self.digest_buf.replace(digest);
        self.deferred_call.set();

        Ok(())
    }
}

impl<'a> digest::DigestVerify<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OtCryptoLibHMAC<'a> {
    fn set_verify_client(
        &'a self,
        client: &'a dyn digest::ClientVerify<{ SHA_256_OUTPUT_LEN_BYTES }>,
    ) {
        // see comment for dataclient
        unimplemented!()
    }
    fn verify(
        &'a self,
        compare: &'static mut [u8; SHA_256_OUTPUT_LEN_BYTES],
    ) -> Result<(), (ErrorCode, &'static mut [u8; SHA_256_OUTPUT_LEN_BYTES])> {
        //self.run(compare)
        unimplemented!();
    }
}

impl<'a> digest::HmacSha256 for OtCryptoLibHMAC<'a> {
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        assert!(key.len() == 32);

        unsafe { libotcrypto_bindings::entropy_complex_init() };

        self.with_hmac_context(|hmac_context| {
            let key_config_rust = libotcrypto_bindings::otcrypto_key_config {
                version: libotcrypto_bindings::otcrypto_lib_version_kOtcryptoLibVersion1,
                key_mode: libotcrypto_bindings::otcrypto_key_mode_kOtcryptoKeyModeHmacSha256,
                key_length: 32, // HMAC-SHA256
                hw_backed: libotcrypto_bindings::hardened_bool_kHardenedBoolFalse,
                //diversification_hw_backed: libotcrypto_bindings::crypto_const_uint8_buf_t {
                //    data: core::ptr::null(),
                //    len: 0,
                //},
                exportable: libotcrypto_bindings::hardened_bool_kHardenedBoolFalse,
                security_level:
                    libotcrypto_bindings::otcrypto_key_security_level_kOtcryptoKeySecurityLevelLow,
            };

            //blinded_key_config.write(key_config_rust, &mut access);

            // Create keyblob from key and mask:
            let keyblob_words = unsafe { libotcrypto_bindings::keyblob_num_words(key_config_rust) };
            assert!(keyblob_words == 16);

            let test_mask: [u32; 17] = [
                0x8cb847c3, 0xc6d34f36, 0x72edbf7b, 0x9bc0317f, 0x8f003c7f, 0x1d7ba049, 0xfd463b63,
                0xbb720c44, 0x784c215e, 0xeb101d65, 0x35beb911, 0xab481345, 0xa7ebc3e3, 0x04b2a1b9,
                0x764a9630, 0x78b8f9c5, 0x3f2a1d8e,
            ];

            let mut test_key = [0; 32];
            key.chunks(4)
                .map(|chunk| {
                    let mut ci = chunk.iter();
                    u32::from_be_bytes([
                        ci.next().copied().unwrap_or(0),
                        ci.next().copied().unwrap_or(0),
                        ci.next().copied().unwrap_or(0),
                        ci.next().copied().unwrap_or(0),
                    ])
                })
                .zip(test_key.iter_mut())
                .for_each(|(src, dst)| {
                    *dst = src;
                });

            let mut keyblob = [0_u32; 16];
            unsafe {
                libotcrypto_bindings::keyblob_from_key_and_mask(
                    &test_key as *const _ as *const _,
                    &test_mask as *const _ as *const _,
                    key_config_rust,
                    &mut keyblob as *mut _ as *mut _,
                )
            };

            let mut blinded_key = libotcrypto_bindings::otcrypto_blinded_key_t {
                config: key_config_rust,
                keyblob: &mut keyblob as *mut _ as *mut _,
                keyblob_length: keyblob_words * core::mem::size_of::<u32>(),
                checksum: 0,
            };

            let checksum = unsafe {
                libotcrypto_bindings::integrity_blinded_checksum(
                    &blinded_key as *const _ as *const _,
                )
            };

            blinded_key.checksum = checksum;
            //debug!("Blinded checksummed key: {:?}", &*blinded_key.validate(access).unwrap());

            let res = unsafe {
                libotcrypto_bindings::otcrypto_hmac_init(
                    hmac_context as *mut _,
                    &blinded_key as *const _ as *const _,
                )
            };
            //panic!("HMAC init res: {:?}", res.validate().unwrap());

            // todo: punting on error handling for now...
            //    }).unwrap();
            //}).unwrap();
        });

        Ok(())
    }
}

impl<'a> digest::HmacSha384 for OtCryptoLibHMAC<'a> {
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}

impl<'a> digest::HmacSha512 for OtCryptoLibHMAC<'a> {
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}
