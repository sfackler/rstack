#![allow(bad_style)]
extern crate libc;

use libc::c_int;

#[cfg(feature = "native")]
pub use native::*;
#[cfg(feature = "ptrace")]
pub use ptrace::*;

#[macro_use]
mod macros;

#[cfg(feature = "native")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64.rs")]
mod native;

#[cfg(feature = "ptrace")]
mod ptrace;

pub const UNW_ESUCCESS: c_int = 0;
pub const UNW_EUNSPEC: c_int = 1;
pub const UNW_ENOMEM: c_int = 2;
pub const UNW_EBADREG: c_int = 3;
pub const UNW_EREADONLYREG: c_int = 4;
pub const UNW_ESTOPUNWIND: c_int = 5;
pub const UNW_EINVALIDIP: c_int = 6;
pub const UNW_EBADFRAME: c_int = 7;
pub const UNW_EINVAL: c_int = 8;
pub const UNW_EBADVERSION: c_int = 9;
pub const UNW_ENOINFO: c_int = 10;

pub const UNW_CACHE_NONE: c_int = 0;
pub const UNW_CACHE_GLOBAL: c_int = 1;
pub const UNW_CACHE_PER_THREAD: c_int = 2;

pub type unw_regnum_t = c_int;

pub enum unw_addr_space {}
pub type unw_addr_space_t = *mut unw_addr_space;

#[repr(C)]
pub enum unw_save_loc_type_t {
    UNW_SLT_NONE,
    UNW_SLT_MEMORY,
    UNW_SLT_REG,
}
