use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=./demo.encapfn.toml");
    println!("cargo:rerun-if-changed=./c_src/demo.c");
    println!("cargo:rerun-if-changed=./c_src/demo.h");

    let cflags = std::env::var("EF_BINDGEN_CFLAGS")
	.expect("Please set EF_BINDGEN_CFLAGS");

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
        .compiler("clang")
        .file("c_src/demo.c")
        .compile("libdemo");
}
