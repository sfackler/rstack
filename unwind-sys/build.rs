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
    if library.version.starts_with("1.2.") {
        println!("cargo:rustc-cfg=unwind12x");
    }

    println!("cargo:version={}", library.version);
    let includedir = pkg_config::get_variable("libunwind", "includedir").unwrap();
    println!("cargo:includedir={}", includedir);
}
