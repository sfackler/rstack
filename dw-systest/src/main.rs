#![allow(bad_style)]

extern crate dw_sys;
extern crate libc;

use dw_sys::*;
use libc::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));
