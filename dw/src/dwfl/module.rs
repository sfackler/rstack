use foreign_types::{ForeignTypeRef, Opaque};
use std::ffi::CStr;

pub struct ModuleRef(Opaque);

impl ForeignTypeRef for ModuleRef {
    type CType = dw_sys::Dwfl_Module;
}

impl ModuleRef {
    pub fn addr_name(&self, addr: u64) -> Option<&str> {
        unsafe {
            let ptr = dw_sys::dwfl_module_addrname(self.as_ptr(), addr);
            if ptr.is_null() {
                None
            } else {
                // FIXME
                Some(CStr::from_ptr(ptr).to_str().unwrap())
            }
        }
    }
}
