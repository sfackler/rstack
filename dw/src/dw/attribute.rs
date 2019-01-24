use std::ffi::CStr;
use std::marker::PhantomData;

use crate::dw::Error;

// the repr may not be needed here, but better safe than sorry:
// https://github.com/rust-rfcs/unsafe-code-guidelines/issues/34
#[repr(transparent)]
pub struct Attribute<'a>(dw_sys::Dwarf_Attribute, PhantomData<&'a ()>);

impl<'a> Attribute<'a> {
    pub(crate) unsafe fn from_raw(raw: dw_sys::Dwarf_Attribute) -> Attribute<'a> {
        Attribute(raw, PhantomData)
    }

    fn as_ptr(&self) -> *mut dw_sys::Dwarf_Attribute {
        &self.0 as *const _ as *mut _
    }

    pub fn form_string(&self) -> Result<&CStr, Error> {
        unsafe {
            let ptr = dw_sys::dwarf_formstring(self.as_ptr());
            if ptr.is_null() {
                Err(Error::new())
            } else {
                Ok(CStr::from_ptr(ptr))
            }
        }
    }
}
