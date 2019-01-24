use foreign_types::{ForeignTypeRef, Opaque};
use std::ffi::CStr;
use std::mem;
use std::ptr;

use crate::dw::Die;
use crate::dwfl::Error;
use crate::elf::Symbol;

pub struct ModuleRef(Opaque);

impl ForeignTypeRef for ModuleRef {
    type CType = dw_sys::Dwfl_Module;
}

impl ModuleRef {
    pub fn addr_name(&self, addr: u64) -> Result<&CStr, Error> {
        unsafe {
            let ptr = dw_sys::dwfl_module_addrname(self.as_ptr(), addr);
            if ptr.is_null() {
                Err(Error::new())
            } else {
                Ok(CStr::from_ptr(ptr))
            }
        }
    }

    pub fn addr_info(&self, addr: u64) -> Result<AddrInfo<'_>, Error> {
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
                Err(Error::new())
            } else {
                Ok(AddrInfo {
                    name: CStr::from_ptr(ptr),
                    offset,
                    sym: Symbol(sym),
                    bias,
                })
            }
        }
    }

    pub fn addr_die(&self, addr: u64) -> Result<(&Die<'_>, u64), Error> {
        unsafe {
            let mut bias = 0;
            let ptr = dw_sys::dwfl_module_addrdie(self.as_ptr(), addr, &mut bias);
            if ptr.is_null() {
                Err(Error::new())
            } else {
                Ok((Die::from_ptr(ptr), bias))
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
