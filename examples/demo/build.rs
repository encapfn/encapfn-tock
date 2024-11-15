use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=./demo.encapfn.toml");
    println!("cargo:rerun-if-changed=./c_src/demo.c");
    println!("cargo:rerun-if-changed=./c_src/demo.h");

    let cflags = std::env::var("EF_BINDGEN_CFLAGS").expect("Please set EF_BINDGEN_CFLAGS");
    // panic!("CFLAGS: {:?}", cflags.split(" ").collect::<Vec<_>>());

    let bindings = bindgen::Builder::default()
        .header("c_src/demo.h")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .encapfn_configuration_file(Some(
            PathBuf::from("./demo.encapfn.toml").canonicalize().unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libdemo_bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .compiler("riscv32-none-elf-gcc")
	.try_flags_from_environment("EF_BINDGEN_CFLAGS")
	.unwrap()
        .file("c_src/demo.c")
        .compile("libdemo");

    println!("cargo::rustc-link-search=/home/leons/proj/encapfn/code/encapfn-tock/opentitan-cryptolib");
    println!("cargo::rustc-link-lib=otcrypto");
}
