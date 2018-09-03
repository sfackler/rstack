extern crate pkg_config;

fn main() {
    pkg_config::probe_library("libdw").unwrap();

    let includedir = pkg_config::get_variable("libdw", "includedir").unwrap();
    println!("cargo:includedir={}", includedir);
}
