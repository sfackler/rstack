#![allow(bad_style)]

extern crate unwind_sys;
extern crate libc;

use libc::*;
use unwind_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
