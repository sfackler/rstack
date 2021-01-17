use foreign_types::foreign_type;

foreign_type! {
    /// An ELF image.
    pub unsafe type Elf {
        type CType = dw_sys::Elf;
        fn drop = dw_sys::elf_end;
    }
}
