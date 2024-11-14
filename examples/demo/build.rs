use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=./demo.encapfn.toml");
    println!("cargo:rerun-if-changed=./c_src/demo.c");
    println!("cargo:rerun-if-changed=./c_src/demo.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("c_src/demo.h")
        .clang_arg("-isystem ../../libtock-newlib-4.3.0.20230120/riscv/riscv64-unknown-elf/include")
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .encapfn_configuration_file(Some(
            PathBuf::from("./demo.encapfn.toml").canonicalize().unwrap(),
        ))
        .use_core()
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    //
    // We avoid using OUT_DIR as this does not allow us to view the
    // intermediate artifacts.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libdemo_bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .compiler("clang")
        .file("c_src/demo.c")
        .compile("libdemo");
}
