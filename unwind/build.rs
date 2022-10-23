use std::env;

fn main() {
    let version = env::var("DEP_UNWIND_VERSION").unwrap();
    let mut it = version.split(&['.', '-'][..]);
    let major = it.next().unwrap().parse::<u32>().unwrap();
    let minor = it.next().unwrap().parse::<u32>().unwrap();
    if major < 1 || (major == 1 && minor < 2) {
        println!("cargo:rustc-cfg=pre12");
    }
    if major < 1 || (major == 1 && minor < 6) {
        println!("cargo:rustc-cfg=pre16");
    }
}
