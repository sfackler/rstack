use foreign_types::{ForeignTypeRef, Opaque};
use std::ffi::CStr;
use std::mem;
use std::ptr;

use crate::elf::Symbol;

pub struct ModuleRef(Opaque);

impl ForeignTypeRef for ModuleRef {
    type CType = dw_sys::Dwfl_Module;
}

impl ModuleRef {
    pub fn addr_name(&self, addr: u64) -> Option<&CStr> {
        unsafe {
            let ptr = dw_sys::dwfl_module_addrname(self.as_ptr(), addr);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr))
            }
        }
    }

    pub fn addr_info(&self, addr: u64) -> Option<AddrInfo<'_>> {
        unsafe {
            let mut offset = 0;
            let mut sym = mem::zeroed::<dw_sys::GElf_Sym>();
            let mut bias = 0;

            let ptr = dw_sys::dwfl_module_addrinfo(
                self.as_ptr(),
                addr,
                &mut offset,
                &mut sym,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut bias,
            );

            if ptr.is_null() {
                None
            } else {
                Some(AddrInfo {
                    name: CStr::from_ptr(ptr),
                    offset,
                    sym: Symbol(sym),
                    bias,
                })
            }
        }
    }
}

pub struct AddrInfo<'a> {
    name: &'a CStr,
    offset: u64,
    sym: Symbol,
    bias: u64,
}

impl<'a> AddrInfo<'a> {
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn symbol(&self) -> &Symbol {
        &self.sym
    }

    pub fn bias(&self) -> u64 {
        self.bias
    }
}
