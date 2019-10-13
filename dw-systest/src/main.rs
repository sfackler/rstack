#![allow(bad_style, invalid_value, unknown_lints)]

extern crate dw_sys;
extern crate libc;

use dw_sys::*;
use libc::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
