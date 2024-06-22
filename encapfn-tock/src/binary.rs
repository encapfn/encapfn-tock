#[derive(Copy, Clone, Debug)]
pub struct EncapfnBinary {
    pub tbf_start: Option<*const ()>,
    pub binary_start: *const (),
    pub binary_length: usize,
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
}
