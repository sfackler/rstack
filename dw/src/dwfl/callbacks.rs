use libc::{c_char, c_int, c_void};
use std::ptr;

/// Callbacks used to configure the behavior of a `Dwfl`.
pub struct Callbacks(dw_sys::Dwfl_Callbacks);

unsafe impl Sync for Callbacks {}
unsafe impl Send for Callbacks {}

impl Callbacks {
    /// Creates a new callback set.
    ///
    /// The find_elf and find_debuginfo callbacks are required. The section address callback and debuginfo_path
    /// value are initialized to NULL.
    pub fn new(find_elf: FindElf, find_debuginfo: FindDebuginfo) -> Callbacks {
        Callbacks(dw_sys::Dwfl_Callbacks {
            find_elf: Some(find_elf.0),
            find_debuginfo: Some(find_debuginfo.0),
            section_address: None,
            debuginfo_path: ptr::null_mut(),
        })
    }

    /// Returns the pointer representation of the callbacks.
    pub fn as_ptr(&self) -> *mut dw_sys::Dwfl_Callbacks {
        &self.0 as *const _ as *mut _
    }
}

/// The callback responsible for locating the ELF images of a process.
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
    /// A standard callback which uses the `/proc` pseudo-filesystem to locate ELF images.
    pub const LINUX_PROC: FindElf = FindElf(dw_sys::dwfl_linux_proc_find_elf);
}

/// The callback responsible for locating the debuginfo of a process.
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
    /// The standard callback.
    pub const STANDARD: FindDebuginfo = FindDebuginfo(dw_sys::dwfl_standard_find_debuginfo);
}
