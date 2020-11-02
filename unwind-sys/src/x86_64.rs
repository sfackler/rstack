use libc::{c_char, c_int, c_void, size_t, ucontext_t};

use crate::*;

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
#[derive(Copy, Clone)]
pub struct unw_tdep_save_loc_t {
    #[cfg(not(pre12))]
    pub unused: c_char,
}

pub type unw_tdep_context_t = ucontext_t;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct unw_tdep_proc_info_t {
    #[cfg(not(pre12))]
    pub unused: c_char,
}

#[macro_export]
macro_rules! unw_tdep_getcontext {
    ($uc:expr) => {
        $crate::unw_tdep_getcontext($uc)
    };
}

extern "C" {
    #[link_name = "_Ux86_64_getcontext"]
    pub fn unw_tdep_getcontext(ctx: *mut unw_tdep_context_t) -> c_int;

    #[link_name = "_Ux86_64_init_local"]
    pub fn unw_init_local(cur: *mut unw_cursor_t, ctx: *mut unw_context_t) -> c_int;

    #[link_name = "_Ux86_64_init_remote"]
    pub fn unw_init_remote(cur: *mut unw_cursor_t, spc: unw_addr_space_t, p: *mut c_void) -> c_int;

    #[link_name = "_Ux86_64_step"]
    pub fn unw_step(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Ux86_64_get_reg"]
    pub fn unw_get_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_word_t) -> c_int;

    #[link_name = "_Ux86_64_set_reg"]
    pub fn unw_set_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, val: unw_word_t) -> c_int;

    #[link_name = "_Ux86_64_resume"]
    pub fn unw_resume(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Ux86_64_create_addr_space"]
    pub fn unw_create_addr_space(
        accessors: *mut unw_accessors_t,
        byteorder: c_int,
    ) -> unw_addr_space_t;

    #[link_name = "_Ux86_64_destroy_addr_space"]
    pub fn unw_destroy_addr_space(spc: unw_addr_space_t);

    #[link_name = "_Ux86_64_get_accessors"]
    pub fn unw_get_accessors(spc: unw_addr_space_t) -> *mut unw_accessors_t;

    #[link_name = "_Ux86_64_flush_cache"]
    pub fn unw_flush_cache(spc: unw_addr_space_t, lo: unw_word_t, hi: unw_word_t);

    #[link_name = "_Ux86_64_set_caching_policy"]
    pub fn unw_set_caching_policy(spc: unw_addr_space_t, policy: unw_caching_policy_t) -> c_int;

    #[link_name = "_Ux86_64_regname"]
    pub fn unw_regname(reg: unw_regnum_t) -> *const c_char;

    #[link_name = "_Ux86_64_get_proc_info"]
    pub fn unw_get_proc_info(cur: *mut unw_cursor_t, info: *mut unw_proc_info_t) -> c_int;

    #[link_name = "_Ux86_64_get_save_loc"]
    pub fn unw_get_save_loc(cur: *mut unw_cursor_t, a: c_int, p: *mut unw_save_loc_t) -> c_int;

    #[link_name = "_Ux86_64_is_fpreg"]
    pub fn unw_tdep_is_fpreg(reg: unw_regnum_t) -> c_int;

    #[link_name = "_Ux86_64_is_signal_frame"]
    pub fn unw_is_signal_frame(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Ux86_64_get_proc_name"]
    pub fn unw_get_proc_name(
        cur: *mut unw_cursor_t,
        buf: *mut c_char,
        len: size_t,
        offp: *mut unw_word_t,
    ) -> c_int;

    #[link_name = "_Ux86_64_strerror"]
    pub fn unw_strerror(err_code: c_int) -> *const c_char;

    #[link_name = "_Ux86_64_local_addr_space"]
    pub static unw_local_addr_space: unw_addr_space_t;
}
