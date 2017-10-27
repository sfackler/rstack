#![allow(improper_ctypes)] // zero sized structs are UB in C D:

use libc::{c_char, c_int, c_void, size_t, ucontext_t};

pub use ::*;

make_api!();

pub const UNW_TDEP_CURSOR_LEN: c_int = 127;

pub type unw_word_t = u64;
pub type unw_sword_t = i64;

pub const UNW_X86_64_RAX: c_int = 0;
pub const UNW_X86_64_RDX: c_int = 1;
pub const UNW_X86_64_RCX: c_int = 2;
pub const UNW_X86_64_RBX: c_int = 3;
pub const UNW_X86_64_RSI: c_int = 4;
pub const UNW_X86_64_RDI: c_int = 5;
pub const UNW_X86_64_RBP: c_int = 6;
pub const UNW_X86_64_RSP: c_int = 7;
pub const UNW_X86_64_R8: c_int = 8;
pub const UNW_X86_64_R9: c_int = 9;
pub const UNW_X86_64_R10: c_int = 10;
pub const UNW_X86_64_R11: c_int = 11;
pub const UNW_X86_64_R12: c_int = 12;
pub const UNW_X86_64_R13: c_int = 13;
pub const UNW_X86_64_R14: c_int = 14;
pub const UNW_X86_64_R15: c_int = 15;
pub const UNW_X86_64_RIP: c_int = 16;
pub const UNW_TDEP_LAST_REG: c_int = UNW_X86_64_RIP;
pub const UNW_X86_64_CFA: c_int = 17;

pub const UNW_TDEP_IP: c_int = UNW_X86_64_RIP;
pub const UNW_TDEP_SP: c_int = UNW_X86_64_RSP;
pub const UNW_TDEP_BP: c_int = UNW_X86_64_RBP;
pub const UNW_TDEP_EH: c_int = UNW_X86_64_RAX;

#[repr(C)]
pub struct unw_tdep_save_loc_t {}

pub type unw_tdep_context_t = ucontext_t;

#[repr(C)]
pub struct unw_tdep_proc_info_t {}

extern "C" {
    pub fn _Ux86_64_getcontext(ctx: *mut unw_tdep_context_t) -> c_int;

    pub fn _Ux86_64_init_local(cur: *mut unw_cursor_t, ctx: *mut unw_context_t) -> c_int;
    pub fn _Ux86_64_init_remote(
        cur: *mut unw_cursor_t,
        spc: unw_addr_space_t,
        p: *mut c_void,
    ) -> c_int;
    pub fn _Ux86_64_step(cur: *mut unw_cursor_t) -> c_int;
    pub fn _Ux86_64_resume(cur: *mut unw_cursor_t) -> c_int;
    pub fn _Ux86_64_get_proc_info(cur: *mut unw_cursor_t, info: *mut unw_proc_info_t) -> c_int;
}
