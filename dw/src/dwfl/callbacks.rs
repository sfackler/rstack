use libc::{c_char, c_int, c_void};
use std::ptr;

pub struct DwflCallbacks(dw_sys::Dwfl_Callbacks);

impl DwflCallbacks {
    pub fn new(find_elf: FindElf, find_debuginfo: FindDebuginfo) -> DwflCallbacks {
        DwflCallbacks(dw_sys::Dwfl_Callbacks {
            find_elf: Some(find_elf.0),
            find_debuginfo: Some(find_debuginfo.0),
            section_address: None,
            debuginfo_path: ptr::null_mut(),
        })
    }

    pub fn as_ptr(&self) -> *mut dw_sys::Dwfl_Callbacks {
        &self.0 as *const _ as *mut _
    }
}

#[derive(Copy, Clone)]
pub struct FindElf(
    unsafe extern "C" fn(
        *mut dw_sys::Dwfl_Module,
        *mut *mut c_void,
        *const c_char,
        dw_sys::Dwarf_Addr,
        *mut *mut c_char,
        *mut *mut dw_sys::Elf,
    ) -> c_int,
);

impl FindElf {
    pub const LINUX_PROC: FindElf = FindElf(dw_sys::dwfl_linux_proc_find_elf);
}

#[derive(Copy, Clone)]
pub struct FindDebuginfo(
    unsafe extern "C" fn(
        *mut dw_sys::Dwfl_Module,
        *mut *mut c_void,
        *const c_char,
        dw_sys::Dwarf_Addr,
        *const c_char,
        *const c_char,
        dw_sys::GElf_Word,
        *mut *mut c_char,
    ) -> c_int,
);

impl FindDebuginfo {
    pub const STANDARD: FindDebuginfo = FindDebuginfo(dw_sys::dwfl_standard_find_debuginfo);
}
