extern crate pkg_config;

use std::env;

fn main() {
    pkg_config::probe_library("libunwind").unwrap();

    if env::var_os("CARGO_FEATURE_NATIVE").is_some() {
        let target = env::var("TARGET").unwrap();
        let arch = if target.starts_with("x86_64-") {
            "x86_64"
        } else {
            panic!("unsupported target");
        };
        println!("cargo:rustc-link-lib=unwind-{}", arch);
    }
    if env::var_os("CARGO_FEATURE_PTRACE").is_some() {
        println!("cargo:rustc-link-lib=unwind-ptrace");
    }
}
