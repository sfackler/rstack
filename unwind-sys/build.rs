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
    let mut it = library.version.split(|c: char| !c.is_digit(10));
    let major = it.next().unwrap();
    let minor = it.next().unwrap();

    if major == "0" || (major == "1" && (minor == "0" || minor == "1")) {
        println!("cargo:rustc-cfg=unwind11x");
    }

    println!("cargo:version={}", library.version);
    let includedir = pkg_config::get_variable(lib, "includedir").unwrap();
    println!("cargo:includedir={}", includedir);
}
