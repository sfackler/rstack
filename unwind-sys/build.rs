extern crate pkg_config;

use std::env;

fn main() {
    let lib = if env::var_os("CARGO_FEATURE_PTRACE").is_some() {
        "libunwind-ptrace"
    } else {
        "libunwind-generic"
    };
    let library = pkg_config::probe_library(lib).unwrap();

    // There were some ABI changes from 1.1 to 1.2 on x86_64
    let mut it = library.version.split(".");
    let major = it.next().unwrap().parse::<u32>().unwrap();
    let minor = it.next().unwrap().parse::<u32>().unwrap();
    if major < 1 || (major == 1 && minor < 2) {
        println!("cargo:rustc-cfg=pre12");
    }

    println!("cargo:version={}", library.version);
    let includedir = pkg_config::get_variable(lib, "includedir").unwrap();
    println!("cargo:includedir={}", includedir);
}
