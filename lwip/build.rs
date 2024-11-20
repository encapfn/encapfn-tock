use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=lwip");

    let cflags = std::env::var("EF_BINDGEN_CFLAGS").expect("Please set EF_BINDGEN_CFLAGS");

    println!("cargo:include=src/include");
    let bindings = bindgen::Builder::default()
        .header("src/include/lwip/init.h")
        .header("src/include/lwip/netif.h")
        .header("src/include/lwip/tcp.h")
        .header("src/include/lwip/udp.h")
        .header("src/include/lwip/ip_addr.h")
        .clang_arg("-I./src/include")
        .clang_arg("-I./config")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .encapfn_configuration_file(Some(
            PathBuf::from("./lwip.encapfn.toml").canonicalize().unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("liblwip_bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=config");
    cc::Build::new()
        .compiler("riscv32-none-elf-gcc")
        //.file("src/core/altcp_alloc.c")
        //.file("src/core/altcp.c")
        .file("src/core/def.c")
        .file("src/core/inet_chksum.c")
        .file("src/core/init.c")
        .file("src/core/ip.c")
        .file("src/core/ipv4/acd.c")
        .file("src/core/ipv4/autoip.c")
        .file("src/core/ipv4/dhcp.c")
        .file("src/core/ipv4/etharp.c")
        .file("src/core/ipv4/icmp.c")
        .file("src/core/ipv4/ip4.c")
        .file("src/core/ipv4/ip4_addr.c")
        .file("src/core/ipv4/ip4_frag.c")
        .file("src/core/ipv6/icmp6.c")
        .file("src/core/ipv6/ip6.c")
        .file("src/core/ipv6/ip6_addr.c")
        .file("src/core/ipv6/ip6_frag.c")
        .file("src/core/ipv6/mld6.c")
        .file("src/core/ipv6/nd6.c")
        .file("src/core/mem.c")
        .file("src/core/memp.c")
        .file("src/core/netif.c")
        .file("src/core/pbuf.c")
        .file("src/core/raw.c")
        //.file("src/core/stats.c")
        //.file("src/core/sys.c")
        .file("src/core/tcp.c")
        .file("src/core/tcp_in.c")
        .file("src/core/tcp_out.c")
        .file("src/core/timeouts.c")
        .file("src/core/udp.c")
        .file("src/netif/ethernet.c")
        .include("src/include")
        .include("config")
        .warnings(false)
        .compile("liblwip.a");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lwip.encapfn.toml");
}

