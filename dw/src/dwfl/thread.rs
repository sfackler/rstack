use foreign_types::{ForeignTypeRef, Opaque};
use libc::{c_int, c_void};
use std::any::Any;
use std::panic::{self, AssertUnwindSafe};

use crate::dwfl::{cvt, FrameRef, DwflRef, Error};

pub struct ThreadRef(Opaque);

impl ForeignTypeRef for ThreadRef {
    type CType = dw_sys::Dwfl_Thread;
}

impl ThreadRef {
    pub fn dwfl(&self) -> &DwflRef<'_> {
        unsafe {
            let ptr = dw_sys::dwfl_thread_dwfl(self.as_ptr());
            DwflRef::from_ptr(ptr)
        }
    }

    pub fn tid(&self) -> u32 {
        unsafe { dw_sys::dwfl_thread_tid(self.as_ptr()) as u32 }
    }

    pub fn frames<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(&mut FrameRef) -> Result<(), Error>,
    {
        unsafe {
            let mut state = CallbackState {
                callback,
                panic: None,
                error: None,
            };
            let r = dw_sys::dwfl_thread_getframes(
                self.as_ptr(),
                Some(frames_cb::<F>),
                &mut state as *mut _ as *mut c_void,
            );

            if let Some(payload) = state.panic {
                panic::resume_unwind(payload);
            }
            if let Some(e) = state.error {
                return Err(e);
            }

            cvt(r)
        }
    }
}

struct CallbackState<F> {
    callback: F,
    panic: Option<Box<Any + Send>>,
    error: Option<Error>,
}

unsafe extern "C" fn frames_cb<F>(frame: *mut dw_sys::Dwfl_Frame, arg: *mut c_void) -> c_int
where
    F: FnMut(&mut FrameRef) -> Result<(), Error>,
{
    let state = &mut *(arg as *mut CallbackState<F>);
    let frame = FrameRef::from_ptr_mut(frame);

    match panic::catch_unwind(AssertUnwindSafe(|| (state.callback)(frame))) {
        Ok(Ok(())) => dw_sys::DWARF_CB_OK,
        Ok(Err(e)) => {
            state.error = Some(e);
            dw_sys::DWARF_CB_ABORT
        }
        Err(e) => {
            state.panic = Some(e);
            dw_sys::DWARF_CB_ABORT
        }
    }
}
