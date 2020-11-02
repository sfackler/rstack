use pkg_config;

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
    let mut minor = it.next().unwrap().parse::<u32>().unwrap();
    // the pkg-config version is messed up in old versions and reports e.g. 1.21 for 1.2.1!
    if it.next().is_none() {
        minor /= 10;
    }
    if major < 1 || (major == 1 && minor < 2) {
        println!("cargo:rustc-cfg=pre12");
    }
    if major < 1 || (major == 1 && minor < 3) {
        println!("cargo:rustc-cfg=pre13");
    }
    if major < 1 || (major == 1 && minor < 4) {
        println!("cargo:rustc-cfg=pre14");
    }

    println!("cargo:version={}", library.version);
    let includedir = pkg_config::get_variable(lib, "includedir").unwrap();
    println!("cargo:includedir={}", includedir);
}
