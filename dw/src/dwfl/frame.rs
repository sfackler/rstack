use foreign_types::{ForeignTypeRef, Opaque};
use std::ptr;

use crate::dwfl::{ThreadRef, Error};

pub struct FrameRef(Opaque);

impl ForeignTypeRef for FrameRef {
    type CType = dw_sys::Dwfl_Frame;
}

impl FrameRef {
    pub fn thread(&self) -> &ThreadRef {
        unsafe {
            let ptr = dw_sys::dwfl_frame_thread(self.as_ptr());
            ThreadRef::from_ptr(ptr)
        }
    }

    pub fn pc(&self, is_activation: Option<&mut bool>) -> Result<u64, Error> {
        unsafe {
            let mut pc = 0;
            let isactivation = is_activation.map_or(ptr::null_mut(), |b| b as *mut bool);
            if dw_sys::dwfl_frame_pc(self.as_ptr(), &mut pc, isactivation) {
                Ok(pc)
            } else {
                Err(Error::new())
            }
        }
    }
}
