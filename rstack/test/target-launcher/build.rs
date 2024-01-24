use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Compile the "capture-target" executable as part of the build process of this library to gain path to its location.
    // This location is then fed to the library as part of the build process of this library so that the library
    // can start the "capture-target" process upon request.

    let out_dir = std::env::var("OUT_DIR").expect("Output directory unavailable.");
    let run = escargot::CargoBuild::new()
        .current_release()
        .current_target()
        .manifest_path("../capture-target/Cargo.toml")
        .target_dir(&out_dir)
        .run()
        .expect("Compiling capture-target failed.");

    let target_template = r#"
    fn get_path() -> &'static str {
        "PATH"
    }
    "#;
    let target_template = target_template.replace(
        "PATH",
        run.path().to_str().expect("Unexpected characters in path."),
    );

    let dest_path = Path::new(&out_dir).join("target.rs");
    let mut f = File::create(dest_path).expect("Opening target.rs failed.");
    f.write_all(target_template.as_bytes())
        .expect("Writing child.rs failed.");

    println!("cargo:rerun-if-changed=build.rs");
}
