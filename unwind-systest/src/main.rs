#![allow(bad_style, improper_ctypes)] // x86_64 libunwind has empty structs before 1.2

extern crate libc;
extern crate unwind_sys;

use libc::*;
use unwind_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
