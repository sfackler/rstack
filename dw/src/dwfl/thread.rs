use crate::dwfl::{cvt, DwflRef, Error, FrameRef};
use foreign_types::{ForeignTypeRef, Opaque};
use libc::{c_int, c_uint, c_void};
use std::any::Any;
use std::convert::TryFrom;
use std::panic::{self, AssertUnwindSafe};

/// A reference to a thread.
pub struct ThreadRef(Opaque);

unsafe impl ForeignTypeRef for ThreadRef {
    type CType = dw_sys::Dwfl_Thread;
}

impl ThreadRef {
    /// Returns the base session associated with this thread.
    pub fn dwfl(&self) -> &DwflRef<'_> {
        unsafe {
            let ptr = dw_sys::dwfl_thread_dwfl(self.as_ptr());
            DwflRef::from_ptr(ptr)
        }
    }

    /// Returns the thread's ID.
    pub fn tid(&self) -> u32 {
        unsafe { dw_sys::dwfl_thread_tid(self.as_ptr()) as u32 }
    }

    /// Used by the [`ThreadCallbacks::set_initial_registers`](crate::dwfl::ThreadCallbacks::set_initial_registers)
    /// method to initialize the thread's registers.
    ///
    /// Registers are set in a contiguous block starting at `firstreg`.
    pub fn state_registers(&mut self, firstreg: usize, regs: &[u64]) -> bool {
        let firstreg = c_int::try_from(firstreg).expect("firstreg overflow");
        let nregs = c_uint::try_from(regs.len()).expect("regs length overflow");
        unsafe {
            dw_sys::dwfl_thread_state_registers(self.as_ptr(), firstreg, nregs, regs.as_ptr())
        }
    }

    /// Used by the [`ThreadCallbacks::set_initial_registers`](crate::dwfl::ThreadCallbacks::set_initial_registers)
    /// method to initialize the thread's PC if not covered in the registers set by [`Self::state_registers`].
    pub fn state_register_pc(&mut self, pc: u64) {
        unsafe { dw_sys::dwfl_thread_state_register_pc(self.as_ptr(), pc) }
    }

    /// Iterates through the frames of the thread.
    ///
    /// The callback will be invoked for each stack frame of the thread in turn.
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
    panic: Option<Box<dyn Any + Send>>,
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
