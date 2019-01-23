pub struct Symbol(pub(crate) dw_sys::GElf_Sym);

impl Symbol {
    pub fn value(&self) -> u64 {
        self.0.st_value
    }

    pub fn size(&self) -> u64 {
        self.0.st_size
    }
}
