use foreign_types::{ForeignTypeRef, Opaque};
use std::ffi::CStr;
use std::mem;
use std::ptr;

use crate::dwfl::Error;
use crate::elf::Symbol;

/// A reference to a module.
pub struct ModuleRef(Opaque);

impl ForeignTypeRef for ModuleRef {
    type CType = dw_sys::Dwfl_Module;
}

impl ModuleRef {
    /// Returns the name of the containing the address.
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

    /// Returns information about the symbol containing the address.
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
}

/// Information about a symbol.
pub struct AddrInfo<'a> {
    name: &'a CStr,
    offset: u64,
    sym: Symbol,
    bias: u64,
}

impl<'a> AddrInfo<'a> {
    /// Returns the name of the symbol.
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    /// Returns the offset of the address from the base of the symbol.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Returns the (unadjusted) symbol itself.
    pub fn symbol(&self) -> &Symbol {
        &self.sym
    }

    /// Returns the offset of the symbol's address to where it was loaded in memory.
    pub fn bias(&self) -> u64 {
        self.bias
    }
}
