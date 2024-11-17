use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=./otcrypto.encapfn.toml");
    println!("cargo:rerun-if-changed=./c_src/encapfn_otcrypto_tbf.h");

    let cflags = std::env::var("EF_BINDGEN_CFLAGS").expect("Please set EF_BINDGEN_CFLAGS");
    // panic!("CFLAGS: {}", cflags);
    // panic!("CFLAGS: {:?}", cflags.split(" ").collect::<Vec<_>>());

    let bindings = bindgen::Builder::default()
        .header("c_src/encapfn_otcrypto_tbf.h")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .encapfn_configuration_file(Some(
            PathBuf::from("./otcrypto.encapfn.toml")
                .canonicalize()
                .unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libotcrypto_bindings.rs"))
        .expect("Couldn't write bindings!");

    println!(
        "cargo::rustc-link-search=/home/leons/proj/encapfn/code/encapfn-tock/opentitan-cryptolib"
    );
    println!("cargo::rustc-link-lib=otcrypto");
}
