use libc::{c_char, c_int, c_void, size_t, ucontext_t};

use crate::*;

pub const UNW_TDEP_CURSOR_LEN: c_int = 127;

pub type unw_word_t = u32;
pub type unw_sword_t = i32;

pub const UNW_X86_EAX: c_int = 0;
pub const UNW_X86_EDX: c_int = 1;
pub const UNW_X86_ECX: c_int = 2;
pub const UNW_X86_EBX: c_int = 3;
pub const UNW_X86_ESI: c_int = 4;
pub const UNW_X86_EDI: c_int = 5;
pub const UNW_X86_EBP: c_int = 6;
pub const UNW_X86_ESP: c_int = 7;
pub const UNW_X86_EIP: c_int = 8;
pub const UNW_X86_EFLAGS: c_int = 9;
pub const UNW_X86_TRAPNO: c_int = 10;
pub const UNW_X86_ST0: c_int = 11;
pub const UNW_X86_ST1: c_int = 12;
pub const UNW_X86_ST2: c_int = 13;
pub const UNW_X86_ST3: c_int = 14;
pub const UNW_X86_ST4: c_int = 15;
pub const UNW_X86_ST5: c_int = 16;
pub const UNW_X86_ST6: c_int = 17;
pub const UNW_X86_ST7: c_int = 18;
pub const UNW_X86_FCW: c_int = 19;
pub const UNW_X86_FSW: c_int = 20;
pub const UNW_X86_FTW: c_int = 21;
pub const UNW_X86_FOP: c_int = 22;
pub const UNW_X86_FCS: c_int = 23;
pub const UNW_X86_FIP: c_int = 24;
pub const UNW_X86_FEA: c_int = 25;
pub const UNW_X86_FDS: c_int = 26;
pub const UNW_X86_XMM0_lo: c_int = 27;
pub const UNW_X86_XMM0_hi: c_int = 28;
pub const UNW_X86_XMM1_lo: c_int = 29;
pub const UNW_X86_XMM1_hi: c_int = 30;
pub const UNW_X86_XMM2_lo: c_int = 31;
pub const UNW_X86_XMM2_hi: c_int = 32;
pub const UNW_X86_XMM3_lo: c_int = 33;
pub const UNW_X86_XMM3_hi: c_int = 34;
pub const UNW_X86_XMM4_lo: c_int = 35;
pub const UNW_X86_XMM4_hi: c_int = 36;
pub const UNW_X86_XMM5_lo: c_int = 37;
pub const UNW_X86_XMM5_hi: c_int = 38;
pub const UNW_X86_XMM6_lo: c_int = 39;
pub const UNW_X86_XMM6_hi: c_int = 40;
pub const UNW_X86_XMM7_lo: c_int = 41;
pub const UNW_X86_XMM7_hi: c_int = 42;
pub const UNW_X86_MXCSR: c_int = 43;
pub const UNW_X86_GS: c_int = 44;
pub const UNW_X86_FS: c_int = 45;
pub const UNW_X86_ES: c_int = 46;
pub const UNW_X86_DS: c_int = 47;
pub const UNW_X86_SS: c_int = 48;
pub const UNW_X86_CS: c_int = 49;
pub const UNW_X86_TSS: c_int = 50;
pub const UNW_X86_LDT: c_int = 51;
pub const UNW_X86_CFA: c_int = 52;
pub const UNW_X86_XMM0: c_int = 53;
pub const UNW_X86_XMM1: c_int = 54;
pub const UNW_X86_XMM2: c_int = 55;
pub const UNW_X86_XMM3: c_int = 56;
pub const UNW_X86_XMM4: c_int = 57;
pub const UNW_X86_XMM5: c_int = 58;
pub const UNW_X86_XMM6: c_int = 59;
pub const UNW_X86_XMM7: c_int = 60;
pub const UNW_TDEP_LAST_REG: c_int = UNW_X86_XMM7;

pub const UNW_TDEP_IP: c_int = UNW_X86_EIP;
pub const UNW_TDEP_SP: c_int = UNW_X86_ESP;
pub const UNW_TDEP_EH: c_int = UNW_X86_EAX;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct unw_tdep_save_loc_t {}

pub type unw_tdep_context_t = ucontext_t;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct unw_tdep_proc_info_t {}

extern "C" {
    #[link_name = "_Ux86_getcontext"]
    pub fn unw_tdep_getcontext(ctx: *mut unw_tdep_context_t) -> c_int;

    #[link_name = "_Ux86_init_local"]
    pub fn unw_init_local(cur: *mut unw_cursor_t, ctx: *mut unw_context_t) -> c_int;

    #[link_name = "_Ux86_init_remote"]
    pub fn unw_init_remote(cur: *mut unw_cursor_t, spc: unw_addr_space_t, p: *mut c_void) -> c_int;

    #[link_name = "_Ux86_step"]
    pub fn unw_step(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Ux86_get_reg"]
    pub fn unw_get_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_word_t) -> c_int;

    #[link_name = "_Ux86_set_reg"]
    pub fn unw_set_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, val: unw_word_t) -> c_int;

    #[link_name = "_Ux86_resume"]
    pub fn unw_resume(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Ux86_create_addr_space"]
    pub fn unw_create_addr_space(
        accessors: *mut unw_accessors_t,
        byteorder: c_int,
    ) -> unw_addr_space_t;

    #[link_name = "_Ux86_destroy_addr_space"]
    pub fn unw_destroy_addr_space(spc: unw_addr_space_t);

    #[link_name = "_Ux86_get_accessors"]
    pub fn unw_get_accessors(spc: unw_addr_space_t) -> *mut unw_accessors_t;

    #[link_name = "_Ux86_flush_cache"]
    pub fn unw_flush_cache(spc: unw_addr_space_t, lo: unw_word_t, hi: unw_word_t);

    #[link_name = "_Ux86_set_caching_policy"]
    pub fn unw_set_caching_policy(spc: unw_addr_space_t, policy: unw_caching_policy_t) -> c_int;

    #[link_name = "_Ux86_regname"]
    pub fn unw_regname(reg: unw_regnum_t) -> *const c_char;

    #[link_name = "_Ux86_get_proc_info"]
    pub fn unw_get_proc_info(cur: *mut unw_cursor_t, info: *mut unw_proc_info_t) -> c_int;

    #[link_name = "_Ux86_get_save_loc"]
    pub fn unw_get_save_loc(cur: *mut unw_cursor_t, a: c_int, p: *mut unw_save_loc_t) -> c_int;

    #[link_name = "_Ux86_is_fpreg"]
    pub fn unw_tdep_is_fpreg(reg: unw_regnum_t) -> c_int;

    #[link_name = "_Ux86_is_signal_frame"]
    pub fn unw_is_signal_frame(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Ux86_get_proc_name"]
    pub fn unw_get_proc_name(
        cur: *mut unw_cursor_t,
        buf: *mut c_char,
        len: size_t,
        offp: *mut unw_word_t,
    ) -> c_int;

    #[link_name = "_Ux86_strerror"]
    pub fn unw_strerror(err_code: c_int) -> *const c_char;

    #[link_name = "_Ux86_local_addr_space"]
    pub static unw_local_addr_space: unw_addr_space_t;
}
