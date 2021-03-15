use std::env;

fn main() {
    let mut cfg = ctest2::TestGenerator::new();

    let includedir = env::var_os("DEP_DW_INCLUDEDIR").unwrap();
    cfg.include(includedir);

    cfg.header("elfutils/libdwfl.h")
        .header("elfutils/libdwelf.h")
        .header("dwarf.h")
        .type_name(|t, _, _| t.to_string())
        .skip_signededness(|t| match t {
            "GElf_Phdr" | "GElf_Shdr" | "GElf_Sym" | "Dwarf_OOM" => true,
            _ => false,
        })
        .generate("../dw-sys/src/lib.rs", "all.rs");
}
