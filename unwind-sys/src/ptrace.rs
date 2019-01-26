use libc::{c_void, pid_t};

use crate::*;

extern "C" {
    pub fn _UPT_create(pid: pid_t) -> *mut c_void;
    pub fn _UPT_destroy(p: *mut c_void);

    pub static _UPT_accessors: unw_accessors_t;
}
