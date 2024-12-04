use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=./ef_littlefs.encapfn.toml");
    println!("cargo:rerun-if-changed=./c_src/ef_littlefs.h");

    let cflags = std::env::var("EF_BINDGEN_CFLAGS").expect("Please set EF_BINDGEN_CFLAGS");
    // panic!("CFLAGS: {}", cflags);
    // panic!("CFLAGS: {:?}", cflags.split(" ").collect::<Vec<_>>());

    let bindings = bindgen::Builder::default()
        .header("c_src/ef_littlefs.h")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .encapfn_configuration_file(Some(
            PathBuf::from("./ef_littlefs.encapfn.toml")
                .canonicalize()
                .unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ef_littlefs_bindings.rs"))
        .expect("Couldn't write bindings!");

    println!(
        "cargo::rustc-link-search=third-party/littlefs/build-riscv32"
    );
    println!(
        "cargo::rustc-link-search=third-party/libtock-c/lib/libtock-newlib-4.3.0.20230120/riscv/riscv64-unknown-elf/lib/rv32imac/ilp32/"
    );
    println!(
        "cargo::rustc-link-search=third-party/libtock-c/lib/libtock-libc++-13.2.0/riscv/riscv64-unknown-elf/lib/rv32imac/ilp32/"
    );
    println!(
        "cargo::rustc-link-search=third-party/libtock-c/lib/libtock-libc++-13.2.0/riscv/lib/gcc/riscv64-unknown-elf/13.2.0/rv32imac/ilp32/"
    );

    cc::Build::new()
        .compiler("riscv32-none-elf-gcc")
        .file("c_src/ef_littlefs.c")
        // I know this is the grossest fix in history
        .file("../../encapfn-tock/encapfn_c_rt/sys.c")
        .include("../../third-party/littlefs/")
        .compile("libeflfs");

    println!("cargo::rustc-link-lib=lfs");
    println!("cargo::rustc-link-lib=c");
    println!("cargo::rustc-link-lib=m");
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rustc-link-lib=supc++");
    println!("cargo::rustc-link-lib=gcc");

    // println!("cargo::rustc-link-lib=eflfs");
}
