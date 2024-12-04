#![no_std]
#![feature(naked_functions)]

use core::ptr;

use kernel::hil::time::Time;

// Magic:
use encapfn::branding::EFID;
use encapfn::rt::EncapfnRt;
use encapfn::types::{AccessScope, AllocScope};
use encapfn::efmutref_get_field;

pub fn print_result<T: Time>(
    label: &str,
    elements: Option<usize>,
    measurement: (usize, T::Ticks, T::Ticks),
    time: &T,
) {
    use kernel::hil::time::{Ticks, ConvertTicks};

    let (iters, start, end) = measurement;
    assert!(end > start);
    let ticks = end.wrapping_sub(start);
    let us = time.ticks_to_us(ticks);
    kernel::debug!(
        "[{}({:?})]: {:?} ticks ({} us) for {} iters, {} ticks / iter, {} us / iter",
        label,
        elements,
        ticks,
        us,
        iters,
        (ticks.into_u32() as f32) / iters as f32,
        (us as f32) / iters as f32
    );
}

// Includes bindgen magic:
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
pub mod littlefs_bindings {
    include!(concat!(env!("OUT_DIR"), "/ef_littlefs_bindings.rs"));
}

use encapfn::types::EFType;
use encapfn::types::EFCopy;

use littlefs_bindings::LibLittleFS;

const FILE_CONTENTS: &str = include_str!("filecontents1024.txt");

#[inline(never)]
pub fn test_littlefs<ID: EFID, RT: EncapfnRt<ID = ID>, L: LibLittleFS<ID, RT, RT = RT>, T: Time>(
    label: &str,
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
) {
    kernel::debug!("in littlefs library");

    /* This is what I expect will be the basis of our benchmark */
    /* The next three calls write an array into a file, and then read it back */
    // const filecontents : &str = match core::str::from_utf8(include_bytes!("filecontents8.txt")) {
    //     Ok(v) => v,
    //     Err(e) => panic!("Filecontents panic"),
    // };

    /* Number of iterations of creating, writing, rewinding, reading, closing, and deleting a file */
    const num_iters: usize = 1000;


    // If weird bugs arise due to the contents of lfs (I don't expect this to happen) uncomment the next two comments
    // let mut lfs = lib.getEmptyFilesystem(alloc, access).unwrap().validate().unwrap();

    let start = time.now();
    for i in 0..num_iters {

        lib.rt().allocate_stacked_t_mut::<littlefs_bindings::lfs, _, _>(alloc, |c_lfs, alloc|{
            // THIS HAS TO REMAIN ALIVE EVEN THOUGH IT'S NOT DIRECTLY USED SINCE A POINTER TO IT IS STORED IN LFS AFTER LFS_FORMAT.
            lib.rt().allocate_stacked_t_mut::<littlefs_bindings::lfs_config, _, _>(alloc, |c_cfg, alloc| {

                lib.rt().allocate_stacked_t_mut::<[u8; littlefs_bindings::CACHE_SIZE as usize], _, _>(alloc, |c_readbuf, alloc| { // Use this line for an EncapFN allocated buffer
                    lib.rt().allocate_stacked_t_mut::<[u8; littlefs_bindings::CACHE_SIZE as usize], _, _>(alloc, |c_progbuffer, alloc| { // Use this line for an EncapFN allocated buffer
                        lib.rt().allocate_stacked_t_mut::<[u8; littlefs_bindings::CACHE_SIZE as usize], _, _>(alloc, |c_lookaheadbuffer, alloc| { // Use this line for an EncapFN allocated buffer
                            // Write the base "filled" config to the ref:
                            c_cfg.write_copy(&lib.getFilledCFG(alloc, access).unwrap(), access);

                            // Now, change some of the fields to point to the buffers:
                            let read_buffer_ref = unsafe { efmutref_get_field!(
                                littlefs_bindings::lfs_config,
                                *mut ::core::ffi::c_void,
                                c_cfg,
                                read_buffer
                            ) };
                            read_buffer_ref.write((&c_readbuf).as_ptr().cast::<core::ffi::c_void>().into(), access);

                            let prog_buffer_ref = unsafe { efmutref_get_field!(
                                littlefs_bindings::lfs_config,
                                *mut ::core::ffi::c_void,
                                c_cfg,
                                prog_buffer
                            ) };
                            prog_buffer_ref.write((&c_progbuffer).as_ptr().cast::<core::ffi::c_void>().into(), access);

                            let lookahead_buffer_ref = unsafe { efmutref_get_field!(
                                littlefs_bindings::lfs_config,
                                *mut ::core::ffi::c_void,
                                c_cfg,
                                lookahead_buffer
                            ) };
                            lookahead_buffer_ref.write((&c_lookaheadbuffer).as_ptr().cast::<core::ffi::c_void>().into(), access);

                            // Benchmark start!!!

                            assert_eq!(0, lib.lfs_format(
                                c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                c_cfg.as_ptr().cast::<littlefs_bindings::lfs_config>().into(),
                                alloc,
                                access
                            ).unwrap().validate().unwrap());



                            let mount_error = lib.lfs_mount(
                                c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                c_cfg.as_ptr().cast::<littlefs_bindings::lfs_config>().into(),
                                alloc,
                                access
                            ).unwrap().validate().unwrap();

                            // kernel::debug!("Mount return: {}", mount_error);
                            if mount_error != 0 {
                                panic!("Mount error");
                            }

                            let filename = "Thenameofthefile";
                            lib.rt().allocate_stacked_slice_mut::<u8, _, _>(filename.as_bytes().len(), alloc, |c_filename, alloc|{
                                c_filename.copy_from_slice(filename.as_bytes(), access);

                                // (I don't expect this to happen) Uncomment the
                                // next two comments if errors start to arise due to
                                // not properly default values in the file struct
                                //
                                // let mut file = lib.getEmptyFileType(0, alloc, access).unwrap().validate().unwrap();

                                lib.rt().allocate_stacked_t_mut::<littlefs_bindings::lfs_file_t, _, _>(alloc, |c_file, alloc|{
                                    // Uncomment these to use the buffer defined in
                                    // the ef_littlefs.c file
                                    //
                                    // let mut fileconfig_buffer = lib.getFileConfigBufferAddr(0, alloc, access).unwrap().validate().unwrap();
                                    // kernel::debug!("File config received: {:?}", fileconfig_buffer);

                                    lib.rt().allocate_stacked_t_mut::<[u8; littlefs_bindings::CACHE_SIZE as usize], _, _>(alloc, |c_fileconfig_buffer, alloc| { // Use this line for an EncapFN allocated buffer


                                        let filecfg = littlefs_bindings::lfs_file_config {
                                            // buffer: fileconfig_buffer, // Use this line for the buffer in the ef_littlefs.c file
                                            buffer: (&c_fileconfig_buffer).as_ptr().cast::<core::ffi::c_void>().into(), // Use this line for the buffer allocated above
                                            attrs: ptr::null_mut(),
                                            attr_count: 0,
                                        };

                                        lib.rt().allocate_stacked_t_mut::<littlefs_bindings::lfs_file_config, _, _>(alloc, |c_fileconfig, alloc|{

                                            c_fileconfig.write(filecfg, access);

                                            // Create and open
                                            assert_eq!(0, lib.lfs_file_opencfg(
                                                c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                                c_file.as_ptr().cast::<littlefs_bindings::lfs_file_t>().into(),
                                                c_filename.as_ptr().cast::<i8>().into(),
                                                3 | 0x0100, // Read and create
                                                c_fileconfig.as_ptr().cast::<littlefs_bindings::lfs_file_config>().into(),
                                                alloc,
                                                access,
                                            ).unwrap().validate().unwrap());


                                            // Write
                                            lib.rt().allocate_stacked_t_mut::<[u8; FILE_CONTENTS.as_bytes().len()], _, _>(alloc, |c_filecontents, alloc| {
                                                c_filecontents.copy_from_slice(FILE_CONTENTS.as_bytes(), access);

                                                // kernel::debug!("Contents buffer reads as (should be the file contents): {:?}", c_filecontents.copy(access).validate().unwrap());
                                                let file_write_error = lib.lfs_file_write(
                                                    c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                                    c_file.as_ptr().cast::<littlefs_bindings::lfs_file_t>().into(),
                                                    c_filecontents.as_ptr().cast::<core::ffi::c_void>().into(),
                                                    FILE_CONTENTS.as_bytes().len() as u32,
                                                    alloc,
                                                    access,
                                                ).unwrap().validate().unwrap();

                                                // kernel::debug!("File write return: {}", file_write_error);
                                                if file_write_error != FILE_CONTENTS.as_bytes().len() as i32 {
                                                    panic!("File write error");
                                                }

                                            });

                                            // Rewind
                                            assert_eq!(0, lib.lfs_file_rewind(
                                                c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                                c_file.as_ptr().cast::<littlefs_bindings::lfs_file_t>().into(),
                                                alloc,
                                                access,
                                            ).unwrap().validate().unwrap());

                                            // Read
                                            lib.rt().allocate_stacked_slice_mut::<u8, _, _>(FILE_CONTENTS.as_bytes().len(), alloc, |c_filecontents, alloc| {
                                                // kernel::debug!("Contents buffer reads as (could be garbage): {:?}", c_filecontents.copy(access).validate().unwrap());
                                                let file_read_error = lib.lfs_file_read(
                                                    c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                                    c_file.as_ptr().cast::<littlefs_bindings::lfs_file_t>().into(),
                                                    c_filecontents.as_ptr().cast::<core::ffi::c_void>().into(),
                                                    FILE_CONTENTS.as_bytes().len() as u32,
                                                    alloc,
                                                    access,
                                                ).unwrap().validate().unwrap();

                                                // kernel::debug!("File read return: {}", file_read_error);
                                                if file_read_error != FILE_CONTENTS.as_bytes().len() as i32 {
                                                    panic!("File read error");
                                                }

                                                assert_eq!(
                                                    &*c_filecontents.as_immut().validate_as_str(access).unwrap(),
                                                    FILE_CONTENTS,
                                                );

                                                // kernel::debug!("Contents buffer reads as (should be the file contents): {:?}", c_filecontents.copy(access).validate().unwrap());

                                            });

                                            assert_eq!(0, lib.lfs_file_close(
                                                c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                                c_file.as_ptr().cast::<littlefs_bindings::lfs_file_t>().into(),
                                                alloc,
                                                access,
                                            ).unwrap().validate().unwrap());
                                        });


                                        // // Delete the file
                                        // assert_eq!(0, lib.lfs_remove(
                                        //     c_lfs.as_ptr().cast::<littlefs_bindings::lfs>().into(),
                                        //     c_filename.as_ptr().cast::<i8>().into(),
                                        //     alloc,
                                        //     access,
                                        // ).unwrap().validate().unwrap());

                                        // TODO Test timing on OT
                                        // let end_time = hardware_alarm.now();
                                        // kernel::debug!("start: {:?}, end: {:?}",
                                        //     start_time,
                                        //     end_time
                                        // );
                                        // let runtime = end_time - start_time;

                                    });
                                });
                            });
                        });
                    });
                });
            });
        });

    }

    let end = time.now();
    print_result(label, Some(FILE_CONTENTS.as_bytes().len()), (num_iters, start, end), time);

}









// I don't think we need this anymore but if I delete it I'm sure I'll suddenly need it like 5 mins later

// let cfg = littlefs_bindings::lfs_config {

//     context: ptr::null_mut(),
//     read: Option::Some(littlefs_bindings::read),
//     prog: Option::Some(littlefs_bindings::prog),
//     erase: Option::Some(littlefs_bindings::erase),
//     sync: Option::Some(littlefs_bindings::sync),

//     read_size: 1,
//     prog_size: 1,
//     block_size: littlefs_bindings::BLOCK_SIZE,
//     block_count: littlefs_bindings::BLOCK_COUNT,
//     cache_size: littlefs_bindings::CACHE_SIZE,
//     lookahead_size: littlefs_bindings::CACHE_SIZE,
//     block_cycles: littlefs_bindings::BLOCK_CYCLES as i32,

//     compact_thresh: 0,

//     read_buffer: (&c_readbuf).as_ptr().cast::<core::ffi::c_void>().into(),
//     prog_buffer: (&c_progbuffer).as_ptr().cast::<core::ffi::c_void>().into(),
//     lookahead_buffer: (&c_lookaheadbuffer).as_ptr().cast::<core::ffi::c_void>().into(),

//     name_max: 0,
//     file_max: 0,
//     attr_max: 0,
//     metadata_max: 0,
//     inline_max: 0,
// };
