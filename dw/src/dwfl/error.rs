use libc::c_int;
use std::error;
use std::ffi::CStr;
use std::fmt;

pub struct Error(c_int);

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Error")
            .field("code", &self.0)
            .field("message", &self.as_str())
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}

impl error::Error for Error {}

impl Error {
    pub(crate) fn new() -> Error {
        unsafe { Error(dw_sys::dwfl_errno()) }
    }

    fn as_str(&self) -> &str {
        unsafe {
            let s = dw_sys::dwfl_errmsg(self.0);
            assert!(!s.is_null());
            CStr::from_ptr(s).to_str().unwrap()
        }
    }
}
