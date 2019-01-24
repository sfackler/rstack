use libc::c_int;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tag(c_int);

impl Tag {
    pub const INLINED_SUBROUTINE: Tag = Tag(dw_sys::DW_TAG_inlined_subroutine);

    pub const SUBPROGRAM: Tag = Tag(dw_sys::DW_TAG_subprogram);

    pub fn from_raw(raw: c_int) -> Tag {
        Tag(raw)
    }

    pub fn as_raw(&self) -> c_int {
        self.0
    }
}
