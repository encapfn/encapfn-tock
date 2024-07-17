use crate::TockEFError;

// Word offsets:
pub const ENCAPFN_HEADER_MAGIC_WOFFSET: usize = 0;
pub const ENCAPFN_HEADER_RTHDR_PTR_WOFFSET: usize = 1;
pub const ENCAPFN_HEADER_INIT_PTR_WOFFSET: usize = 2;
pub const ENCAPFN_HEADER_FNTAB_PTR_WOFFSET: usize = 3;
pub const ENCAPFN_HEADER_FNTAB_LEN_WOFFSET: usize = 4;
pub const ENCAPFN_HEADER_WLEN: usize = 5;
pub const ENCAPFN_HEADER_MAGIC: u32 = 0x454E4350;

#[derive(Copy, Clone, Debug)]
pub struct EncapfnBinary {
    pub tbf_start: Option<*const ()>,
    pub binary_start: *const (),
    pub binary_length: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct EncapfnBinaryParsed {
    pub rthdr_addr: *const (),
    pub init_addr: *const (),
    pub fntab_addr: *const (),
    pub fntab_length: usize,
}

impl EncapfnBinary {
    // TODO: change to raw pointer slice, remove 'static lifetime
    // requirement in parse_tbf_header_lengths
    pub fn find(svc_name: &str, app_flash: &'static [u8]) -> Result<Self, ()> {
        let mut remaining_flash = app_flash;

        loop {
            // Get the first eight bytes of flash to check if there is another
            // app.
            let test_header_slice = match remaining_flash.get(0..8) {
                Some(s) => s,
                None => {
                    // Not enough flash to test for another app. This just means
                    // we are at the end of flash, and there are no more apps to
                    // load.
                    return Err(());
                }
            };

            // Pass the first eight bytes to tbfheader to parse out the length of
            // the tbf header and app. We then use those values to see if we have
            // enough flash remaining to parse the remainder of the header.
            let (version, header_length, entry_length) =
                match tock_tbf::parse::parse_tbf_header_lengths(
                    test_header_slice.try_into().or(Err(()))?,
                ) {
                    Ok((v, hl, el)) => (v, hl, el),
                    Err(tock_tbf::types::InitialTbfParseError::InvalidHeader(entry_length)) => {
                        // If we could not parse the header, then we want to skip over
                        // this app and look for the next one.
                        (0, 0, entry_length)
                    }
                    Err(tock_tbf::types::InitialTbfParseError::UnableToParse) => {
                        // Since Tock apps use a linked list, it is very possible the
                        // header we started to parse is intentionally invalid to signal
                        // the end of apps. This is ok and just means we have finished
                        // loading apps.
                        return Err(());
                    }
                };

            // Now we can get a slice which only encompasses the length of flash
            // described by this tbf header.  We will either parse this as an actual
            // app, or skip over this region.
            let entry_flash = remaining_flash.get(0..entry_length as usize).ok_or(())?;

            // Advance the flash slice for process discovery beyond this last entry.
            // This will be the start of where we look for a new process since Tock
            // processes are allocated back-to-back in flash.
            remaining_flash = remaining_flash.get(entry_flash.len()..).ok_or(())?;

            if header_length > 0 {
                // If we found an actual app header, try to create a `Process`
                // object. We also need to shrink the amount of remaining memory
                // based on whatever is assigned to the new process if one is
                // created.

                // Get a slice for just the app header.
                let header_flash = entry_flash.get(0..header_length as usize).ok_or(())?;

                // Parse the full TBF header to see if this is a valid app. If the
                // header can't parse, we will error right here.
                if let Ok(tbf_header) = tock_tbf::parse::parse_tbf_header(header_flash, version) {
                    let process_name = tbf_header.get_package_name().unwrap();

                    // If the app is enabled, it's a real app and not what we are looking for.
                    if tbf_header.enabled() {
                        continue;
                    }

                    if svc_name != process_name {
                        continue;
                    }

                    return Ok(EncapfnBinary {
                        tbf_start: Some(entry_flash.as_ptr() as *const ()),
                        binary_start: unsafe {
                            entry_flash
                                .as_ptr()
                                .byte_offset(tbf_header.get_protected_size() as isize)
                                as *const ()
                        },
                        binary_length: entry_length as usize
                            - tbf_header.get_protected_size() as usize,
                    });
                }
            };
        }
    }

    pub fn parse(&self) -> Result<EncapfnBinaryParsed, TockEFError> {
        // Each Encapsulated Functions binary must start with a header indicating
        // relevant data to the loader (Tock). Check that the binary can at
        // least fit the header, ensure that the magic bytes match, and perform
        // other sanity checks.
        //
        // Encapsulated Functions binary header layout:
        //
        // 0             2             4             6             8
        // +---------------------------+---------------------------+
        // | 0x454E4350 (ENCP) MAGIC   | Runtime Header Offset     |
        // +---------------------------+---------------------------+
        // | `init` Function Offset    | Function Table Offset     |
        // +---------------------------+---------------------------+
        // | Function Table Length (in pointers)
        // +---------------------------+
        //
        // We will try to load these sections into the provided RAM region, with
        // a layout as follows:
        //
        // +---------------------------+ <- `ram_region_start`
        // | Loader-initialized data   | -\
        // | - (optional) padding      |  |
        // | - .data                   |  |
        // | - .bss                    |  |
        // +---------------------------+  |
        // | Rust "remote memory"    | |  |
        // | stack allocator         | |  |
        // |                         v |  |- R/W permissions for foreign code
        // +---------------------------+  |
        // | Return trampoline stack   |  |
        // | frame                     |  |
        // +---------------------------+  |
        // | Encapsulated functions  | |  |
        // | service stack           | |  |
        // |                         v | -/
        // +---------------------------+ <- `ram_region_start` + `ram_region_len`
        //
        // The entire Encapsulated Functions binary will further be made
        // available with read-execute permissions.

        // Make sure we have at least enough data to parse the header:
        if self.binary_length < ENCAPFN_HEADER_WLEN * core::mem::size_of::<u32>() {
            return Err(TockEFError::BinaryLengthInvalid {
		min_expected: ENCAPFN_HEADER_WLEN * core::mem::size_of::<u32>(),
		actual: self.binary_length,
		desc: "Required space for the EF header",
	    });
        }

        // We require the Encapsulated Functions header to be aligned to a
        // word-boundary, such that we can create a u32-slice to it and support
        // efficient loads.
        if self.binary_start as usize % core::mem::align_of::<u32>() != 0 {
	    return Err(TockEFError::BinaryAlignError {
		expected: core::mem::align_of::<u32>(),
		actual: self.binary_start as usize % core::mem::align_of::<u32>(),
	    });
	}

        // We generally try to avoid retaining Rust slices to the containerized
        // service binary (to avoid unsoundness, in case this memory should
        // change). However, for parsing the header, we can create an ephemeral
        // slice given that we verified the length:
        let header_slice = unsafe {
            core::slice::from_raw_parts(self.binary_start as *const u32, ENCAPFN_HEADER_WLEN) };

        // Read the header fields in native endianness. First, check the magic:
        if header_slice[ENCAPFN_HEADER_MAGIC_WOFFSET] != ENCAPFN_HEADER_MAGIC {
            return Err(TockEFError::BinaryMagicInvalid);
        }

        // Extract the runtime header pointer and ensure that it is fully
        // contained in contained within the binary:
        let rthdr_offset = header_slice[ENCAPFN_HEADER_RTHDR_PTR_WOFFSET] as usize;
        if rthdr_offset > self.binary_length.checked_sub(core::mem::size_of::<u32>()).ok_or(TockEFError::BinarySizeOverflow)? {
            return Err(TockEFError::BinaryLengthInvalid {
		actual: self.binary_length,
		min_expected: rthdr_offset.saturating_add(core::mem::size_of::<u32>()),
		desc: "Required space for the RT header (as indicated by rthdr_offset)",
	    });
        }
        let rthdr_addr = unsafe { self.binary_start.byte_add(rthdr_offset) };
        assert!(rthdr_addr as usize % core::mem::size_of::<u32>() == 0);

        // Extract the init function pointer pointer and ensure that it is fully
        // contained in contained within the binary:
        let init_offset = header_slice[ENCAPFN_HEADER_INIT_PTR_WOFFSET] as usize;
        if init_offset > self.binary_length.checked_sub(core::mem::size_of::<u32>())
	    .ok_or(TockEFError::BinarySizeOverflow)? {
		return Err(TockEFError::BinaryLengthInvalid {
		    actual: self.binary_length,
		    min_expected: init_offset.saturating_add(core::mem::size_of::<u32>()),
		    desc: "Required space for the init function (as indicated by init_offset)",
		});
            }

        // May be a compressed instruction, in which case it'll be aligned on a
        // 2-byte boundary:
        let init_addr = unsafe { self.binary_start.byte_add(init_offset) };
        assert!(init_addr as usize % core::mem::size_of::<u16>() == 0);

        // Extract the function table pointer and ensure that it is fully
        // contained in contained within the binary:
        let fntab_offset = header_slice[ENCAPFN_HEADER_FNTAB_PTR_WOFFSET] as usize;
        let fntab_length = header_slice[ENCAPFN_HEADER_FNTAB_LEN_WOFFSET] as usize;
        if fntab_length.checked_mul(core::mem::size_of::<*const ()>())
	    .and_then(|fl| fntab_offset.checked_add(fl))
	    .ok_or(TockEFError::BinarySizeOverflow)?
            > self.binary_length
	    .checked_sub(core::mem::size_of::<u32>())
	    .ok_or(TockEFError::BinarySizeOverflow)?
        {
            return Err(TockEFError::BinaryLengthInvalid {
		actual: self.binary_length,
		min_expected: fntab_offset.saturating_add(fntab_length * core::mem::size_of::<*const ()>()).saturating_add(core::mem::size_of::<u32>()),
		desc: "Required space for the function table (as indicated by fntab_offset + fntab_len * size_of::<*const ()>)",
	    });
        }
        let fntab_addr = unsafe { self.binary_start.byte_add(fntab_offset) };
        assert!(fntab_addr as usize % core::mem::size_of::<u32>() == 0);

	Ok(EncapfnBinaryParsed {
	    rthdr_addr,
	    init_addr,
	    fntab_addr,
	    fntab_length,
	})
    }
}

