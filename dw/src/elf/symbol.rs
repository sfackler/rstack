/// An ELF symbol.
pub struct Symbol(pub(crate) dw_sys::GElf_Sym);

impl Symbol {
    /// Returns the value of the symbol.
    pub fn value(&self) -> u64 {
        self.0.st_value
    }

    /// Returns the size of the symbol.
    pub fn size(&self) -> u64 {
        self.0.st_size
    }
}
