#![allow(bad_style)]
#![allow(improper_ctypes)] // unw_proc_info_t is zero sized on x86_64

extern crate unwind_sys;
extern crate libc;

use libc::*;
use unwind_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
