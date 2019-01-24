use libc::c_void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr::{self, NonNull};
use std::slice;

use crate::dw::{At, Attribute, Error, Tag};

// the repr may not be needed here, but better safe than sorry:
// https://github.com/rust-rfcs/unsafe-code-guidelines/issues/34
#[repr(transparent)]
pub struct Die<'a>(dw_sys::Dwarf_Die, PhantomData<&'a ()>);

impl<'a> Die<'a> {
    pub(crate) unsafe fn from_ptr<'b>(ptr: *mut dw_sys::Dwarf_Die) -> &'b Die<'a> {
        &*(ptr as *mut Die<'a>)
    }

    fn as_ptr(&self) -> *mut dw_sys::Dwarf_Die {
        &self.0 as *const _ as *mut _
    }

    pub fn tag(&self) -> Tag {
        unsafe {
            let tag = dw_sys::dwarf_tag(self.as_ptr());
            Tag::from_raw(tag)
        }
    }

    pub fn scopes(&self, addr: u64) -> Result<Scopes<'_>, Error> {
        unsafe {
            let mut ptr = ptr::null_mut();
            let len = dw_sys::dwarf_getscopes(self.as_ptr(), addr, &mut ptr);
            if len < 0 {
                Err(Error::new())
            } else {
                Ok(Scopes {
                    // ptr will probably be null if len is 0
                    ptr: NonNull::new(ptr as *mut Die).unwrap_or_else(NonNull::dangling),
                    len: len as usize,
                })
            }
        }
    }

    pub fn attr_integrate(&self, search_name: At) -> Result<Attribute<'_>, Error> {
        unsafe {
            let mut attribute = mem::zeroed::<dw_sys::Dwarf_Attribute>();

            let ptr =
                dw_sys::dwarf_attr_integrate(self.as_ptr(), search_name.as_raw(), &mut attribute);
            if ptr.is_null() {
                Err(Error::new())
            } else {
                Ok(Attribute::from_raw(attribute))
            }
        }
    }

    pub fn name(&self) -> Result<&CStr, Error> {
        unsafe {
            let ptr = dw_sys::dwarf_diename(self.as_ptr());
            if ptr.is_null() {
                Err(Error::new())
            } else {
                Ok(CStr::from_ptr(ptr))
            }
        }
    }
}

pub struct Scopes<'a> {
    ptr: NonNull<Die<'a>>,
    len: usize,
}

impl<'a> Drop for Scopes<'a> {
    fn drop(&mut self) {
        unsafe {
            if self.len > 0 {
                libc::free(self.ptr.as_ptr() as *mut c_void);
            }
        }
    }
}

impl<'a> Deref for Scopes<'a> {
    type Target = [Die<'a>];

    #[inline]
    fn deref(&self) -> &[Die<'a>] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}
