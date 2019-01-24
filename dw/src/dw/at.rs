use libc::c_uint;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct At(c_uint);

impl At {
    pub const MIPS_LINKAGE_NAME: At = At(dw_sys::DW_AT_MIPS_linkage_name);

    pub const LINKAGE_NAME: At = At(dw_sys::DW_AT_linkage_name);

    pub fn from_raw(raw: c_uint) -> At {
        At(raw)
    }

    pub fn as_raw(&self) -> c_uint {
        self.0
    }
}
