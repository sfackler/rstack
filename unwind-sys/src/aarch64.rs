use libc::ucontext_t;

use crate::*;

cfg_if! {
    if #[cfg(pre14)] {
        pub const UNW_TDEP_CURSOR_LEN: c_int = 4096;
    } else if #[cfg(pre13)] {
        pub const UNW_TDEP_CURSOR_LEN: c_int = 512;
    } else {
        pub const UNW_TDEP_CURSOR_LEN: c_int = 250;
    }
}

pub type unw_word_t = u64;
pub type unw_sword_t = i64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct unw_tdep_proc_info_t {}

pub const UNW_AARCH64_X0: c_int = 0;
pub const UNW_AARCH64_X1: c_int = 1;
pub const UNW_AARCH64_X2: c_int = 2;
pub const UNW_AARCH64_X3: c_int = 3;
pub const UNW_AARCH64_X4: c_int = 4;
pub const UNW_AARCH64_X5: c_int = 5;
pub const UNW_AARCH64_X6: c_int = 6;
pub const UNW_AARCH64_X7: c_int = 7;
pub const UNW_AARCH64_X8: c_int = 8;
pub const UNW_AARCH64_X9: c_int = 9;
pub const UNW_AARCH64_X10: c_int = 10;
pub const UNW_AARCH64_X11: c_int = 11;
pub const UNW_AARCH64_X12: c_int = 12;
pub const UNW_AARCH64_X13: c_int = 13;
pub const UNW_AARCH64_X14: c_int = 14;
pub const UNW_AARCH64_X15: c_int = 15;
pub const UNW_AARCH64_X16: c_int = 16;
pub const UNW_AARCH64_X17: c_int = 17;
pub const UNW_AARCH64_X18: c_int = 18;
pub const UNW_AARCH64_X19: c_int = 19;
pub const UNW_AARCH64_X20: c_int = 20;
pub const UNW_AARCH64_X21: c_int = 21;
pub const UNW_AARCH64_X22: c_int = 22;
pub const UNW_AARCH64_X23: c_int = 23;
pub const UNW_AARCH64_X24: c_int = 24;
pub const UNW_AARCH64_X25: c_int = 25;
pub const UNW_AARCH64_X26: c_int = 26;
pub const UNW_AARCH64_X27: c_int = 27;
pub const UNW_AARCH64_X28: c_int = 28;
pub const UNW_AARCH64_X29: c_int = 29;
pub const UNW_AARCH64_X30: c_int = 30;
pub const UNW_AARCH64_SP: c_int = 31;
pub const UNW_AARCH64_PC: c_int = 32;
pub const UNW_AARCH64_PSTATE: c_int = 33;
pub const UNW_AARCH64_V0: c_int = 64;
pub const UNW_AARCH64_V1: c_int = 65;
pub const UNW_AARCH64_V2: c_int = 66;
pub const UNW_AARCH64_V3: c_int = 67;
pub const UNW_AARCH64_V4: c_int = 68;
pub const UNW_AARCH64_V5: c_int = 69;
pub const UNW_AARCH64_V6: c_int = 70;
pub const UNW_AARCH64_V7: c_int = 71;
pub const UNW_AARCH64_V8: c_int = 72;
pub const UNW_AARCH64_V9: c_int = 73;
pub const UNW_AARCH64_V10: c_int = 74;
pub const UNW_AARCH64_V11: c_int = 75;
pub const UNW_AARCH64_V12: c_int = 76;
pub const UNW_AARCH64_V13: c_int = 77;
pub const UNW_AARCH64_V14: c_int = 78;
pub const UNW_AARCH64_V15: c_int = 79;
pub const UNW_AARCH64_V16: c_int = 80;
pub const UNW_AARCH64_V17: c_int = 81;
pub const UNW_AARCH64_V18: c_int = 82;
pub const UNW_AARCH64_V19: c_int = 83;
pub const UNW_AARCH64_V20: c_int = 84;
pub const UNW_AARCH64_V21: c_int = 85;
pub const UNW_AARCH64_V22: c_int = 86;
pub const UNW_AARCH64_V23: c_int = 87;
pub const UNW_AARCH64_V24: c_int = 88;
pub const UNW_AARCH64_V25: c_int = 89;
pub const UNW_AARCH64_V26: c_int = 90;
pub const UNW_AARCH64_V27: c_int = 91;
pub const UNW_AARCH64_V28: c_int = 92;
pub const UNW_AARCH64_V29: c_int = 93;
pub const UNW_AARCH64_V30: c_int = 94;
pub const UNW_AARCH64_V31: c_int = 95;
pub const UNW_AARCH64_FPSR: c_int = 96;
pub const UNW_AARCH64_FPCR: c_int = 97;
pub const UNW_AARCH64_CFA: c_int = UNW_AARCH64_SP;
pub const UNW_TDEP_LAST_REG: c_int = UNW_AARCH64_FPCR;
pub const UNW_TDEP_IP: c_int = UNW_AARCH64_X30;
pub const UNW_TDEP_SP: c_int = UNW_AARCH64_SP;
pub const UNW_TDEP_EH: c_int = UNW_AARCH64_X0;

pub const UNW_TDEP_NUM_EH_REGS: c_int = 4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct unw_tdep_save_loc_t {}

cfg_if! {
    if #[cfg(pre14)] {
        pub type unw_tdep_context_t = ucontext_t;
    } else {
        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct unw_sigcontext {
            pub fault_address: u64,
            pub regs: [u64; 31],
            pub sp: u64,
            pub pc: u64,
            pub pstate: u64,
            __reserved: unw_sigcontext_padding,
        }

        cfg_if! {
            if #[cfg(pre16)] {
                const PADDING_BYTES: usize = 34;
            } else {
                const PADDING_BYTES: usize = 66;
            }
        }

        #[derive(Copy, Clone)]
        #[repr(align(16))]
        struct unw_sigcontext_padding([u8; PADDING_BYTES * 8]);

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct unw_tdep_context_t {
            pub uc_flags: libc::c_ulong,
            // this should be a *mut ucontext, but libc doesn't define that
            uc_link: *mut ucontext_t,
            pub uc_stack: libc::stack_t,
            pub uc_sigmask: libc::sigset_t,
            pub uc_mcontext: unw_sigcontext,
        }

        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct unw_fpsimd_context_t {
            pub _ctx_magic: u32,
            pub _ctx_size: u32,
            pub fpsr: u32,
            pub fpcr: u32,
            pub vregs: [u64; 64],
        }
    }
}

cfg_if! {
    if #[cfg(pre16)] {
        #[macro_export]
        macro_rules! unw_tdep_getcontext {
            ($uc:expr) => {{
                let unw_ctx: *mut $crate::unw_tdep_context_t = $uc;
                core::arch::asm!(
                    "stp x0, x1, [x0, #0]",
                    "stp x2, x3, [x0, #16]",
                    "stp x4, x5, [x0, #32]",
                    "stp x6, x7, [x0, #48]",
                    "stp x8, x9, [x0, #64]",
                    "stp x10, x11, [x0, #80]",
                    "stp x12, x13, [x0, #96]",
                    "stp x14, x15, [x0, #112]",
                    "stp x16, x17, [x0, #128]",
                    "stp x18, x19, [x0, #144]",
                    "stp x20, x21, [x0, #160]",
                    "stp x22, x23, [x0, #176]",
                    "stp x24, x25, [x0, #192]",
                    "stp x26, x27, [x0, #208]",
                    "stp x28, x29, [x0, #224]",
                    "str x30, [x0, #240]",
                    "mov x1, sp",
                    "stp x1, x30, [x0, #248]",
                    in("x0") (*unw_ctx).uc_mcontext.regs.as_ptr(),
                    out("x1") _,
                    options(nostack),
                );

                0
            }};
        }
    } else {
        #[macro_export]
        macro_rules! unw_tdep_getcontext {
            ($uc:expr) => {{
                let unw_ctx: *mut $crate::unw_tdep_context_t = $uc;
                let mut unw_base = (*unw_ctx).uc_mcontext.regs.as_ptr();
                core::arch::asm!(
                    "stp x0, x1, [x0, #0]",
                    "stp x2, x3, [x0, #16]",
                    "stp x4, x5, [x0, #32]",
                    "stp x6, x7, [x0, #48]",
                    "stp x8, x9, [x0, #64]",
                    "stp x10, x11, [x0, #80]",
                    "stp x12, x13, [x0, #96]",
                    "stp x14, x15, [x0, #112]",
                    "stp x16, x17, [x0, #128]",
                    "stp x18, x19, [x0, #144]",
                    "stp x20, x21, [x0, #160]",
                    "stp x22, x23, [x0, #176]",
                    "stp x24, x25, [x0, #192]",
                    "stp x26, x27, [x0, #208]",
                    "stp x28, x29, [x0, #224]",
                    "mov x1, sp",
                    "stp x30, x1, [x0, #240]",
                    "adr x1, 2f",
                    "str x1, [x0, #256]",
                    "mov x0, #0",
                    "2:",
                    inout("x0") unw_base,
                    out("x1") _,
                    options(nostack),
                );

                unw_base as i32
            }};
        }
    }
}

extern "C" {
    #[link_name = "_Uaarch64_init_local"]
    pub fn unw_init_local(cur: *mut unw_cursor_t, ctx: *mut unw_context_t) -> c_int;

    #[link_name = "_Uaarch64_init_remote"]
    pub fn unw_init_remote(cur: *mut unw_cursor_t, spc: unw_addr_space_t, p: *mut c_void) -> c_int;

    #[link_name = "_Uaarch64_step"]
    pub fn unw_step(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Uaarch64_get_reg"]
    pub fn unw_get_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, valp: *mut unw_word_t) -> c_int;

    #[link_name = "_Uaarch64_set_reg"]
    pub fn unw_set_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, val: unw_word_t) -> c_int;

    #[link_name = "_Uaarch64_resume"]
    pub fn unw_resume(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Uaarch64_create_addr_space"]
    pub fn unw_create_addr_space(
        accessors: *mut unw_accessors_t,
        byteorder: c_int,
    ) -> unw_addr_space_t;

    #[link_name = "_Uaarch64_destroy_addr_space"]
    pub fn unw_destroy_addr_space(spc: unw_addr_space_t);

    #[link_name = "_Uaarch64_get_accessors"]
    pub fn unw_get_accessors(spc: unw_addr_space_t) -> *mut unw_accessors_t;

    #[link_name = "_Uaarch64_flush_cache"]
    pub fn unw_flush_cache(spc: unw_addr_space_t, lo: unw_word_t, hi: unw_word_t);

    #[link_name = "_Uaarch64_set_caching_policy"]
    pub fn unw_set_caching_policy(spc: unw_addr_space_t, policy: unw_caching_policy_t) -> c_int;

    #[link_name = "_Uaarch64_regname"]
    pub fn unw_regname(reg: unw_regnum_t) -> *const c_char;

    #[link_name = "_Uaarch64_get_proc_info"]
    pub fn unw_get_proc_info(cur: *mut unw_cursor_t, info: *mut unw_proc_info_t) -> c_int;

    #[link_name = "_Uaarch64_get_save_loc"]
    pub fn unw_get_save_loc(cur: *mut unw_cursor_t, a: c_int, p: *mut unw_save_loc_t) -> c_int;

    #[link_name = "_Uaarch64_is_fpreg"]
    pub fn unw_tdep_is_fpreg(reg: unw_regnum_t) -> c_int;

    #[link_name = "_Uaarch64_is_signal_frame"]
    pub fn unw_is_signal_frame(cur: *mut unw_cursor_t) -> c_int;

    #[link_name = "_Uaarch64_get_proc_name"]
    pub fn unw_get_proc_name(
        cur: *mut unw_cursor_t,
        buf: *mut c_char,
        len: size_t,
        offp: *mut unw_word_t,
    ) -> c_int;

    #[link_name = "_Uaarch64_strerror"]
    pub fn unw_strerror(err_code: c_int) -> *const c_char;

    #[link_name = "_Uaarch64_local_addr_space"]
    pub static unw_local_addr_space: unw_addr_space_t;
}
