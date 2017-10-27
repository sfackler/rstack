extern crate ctest;

fn main() {
    let mut cfg = ctest::TestGenerator::new();

    if cfg!(feature = "local") {
        cfg.cfg("feature", Some("native"));
    }
    if cfg!(feature = "ptrace") {
        cfg.cfg("feature", Some("ptrace"))
            .cfg("feature", Some("native"))
            .header("libunwind-ptrace.h");
    }

    cfg.header("libunwind.h")
        .type_name(|t, _| t.to_string())
        .skip_signededness(|t| match t {
            "unw_tdep_fpreg_t" | "unw_tdep_context_t" | "unw_context_t" | "unw_addr_space_t" => {
                true
            }
            _ => false,
        })
        .field_name(|s, f| match (s, f) {
            ("unw_save_loc_t", "type_") => "type".to_string(),
            (_, f) => f.to_string(),
        })
        .skip_struct(|s| match s {
            "unw_save_loc_t_u" => true,
            _ => false,
        })
        .skip_field_type(|s, f| match (s, f) {
            ("unw_save_loc_t", "u") => true,
            _ => false,
        })
        .generate("../unwind-sys/src/lib.rs", "all.rs");
}
