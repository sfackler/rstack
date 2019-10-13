use foreign_types::{foreign_type, ForeignType, ForeignTypeRef};
use libc::{c_int, c_void, pid_t};
use std::any::Any;
use std::ffi::CStr;
use std::panic::{self, AssertUnwindSafe};
use std::ptr;

use crate::dwfl::{cvt, Callbacks, Error, FrameRef, ModuleRef, ThreadRef};

foreign_type! {
    /// The base type used when interacting with libdwfl.
    pub unsafe type Dwfl<'a> {
        type CType = dw_sys::Dwfl;
        type PhantomData = &'a ();
        fn drop = dw_sys::dwfl_end;
    }
}

impl<'a> Dwfl<'a> {
    /// Creates a new `Dwfl` which will use the specified callbacks.
    pub fn begin(callbacks: &'a Callbacks) -> Result<Dwfl<'a>, Error> {
        unsafe {
            let ptr = dw_sys::dwfl_begin(callbacks.as_ptr());
            if ptr.is_null() {
                Err(Error::new())
            } else {
                Ok(Dwfl::from_ptr(ptr))
            }
        }
    }
}

impl<'a> DwflRef<'a> {
    /// Returns a string describing the version of libdw used.
    pub fn version(&self) -> &str {
        unsafe {
            let p = dw_sys::dwfl_version(self.as_ptr());
            CStr::from_ptr(p).to_str().unwrap()
        }
    }

    /// Starts a "reporting" session used to register new segments and modules.
    ///
    /// Existing segments and modules will be removed.
    pub fn report(&mut self) -> Report<'_, 'a> {
        unsafe {
            dw_sys::dwfl_report_begin(self.as_ptr());
            Report(self)
        }
    }

    /// Starts a "reporting" session used to register new segments and modules.
    ///
    /// Unlike the `report` method, this will not remove existing segments and modules.
    pub fn report_add(&mut self) -> Report<'_, 'a> {
        unsafe {
            dw_sys::dwfl_report_begin_add(self.as_ptr());
            Report(self)
        }
    }

    /// Configures the session to unwind the threads of a remote process via ptrace and the `/proc` pseudo-filesystem.
    ///
    /// Normally, the session will ptrace attach to threads being unwound, but if `assume_ptrace_stopped` is set to
    /// `true`, this will not happen. It's then the responsibility of the caller to ensure that the thread is already
    /// attached and stopped.
    pub fn linux_proc_attach(
        &mut self,
        pid: u32,
        assume_ptrace_stopped: bool,
    ) -> Result<(), Error> {
        unsafe {
            cvt(dw_sys::dwfl_linux_proc_attach(
                self.as_ptr(),
                pid as pid_t,
                assume_ptrace_stopped,
            ))
        }
    }

    /// Iterates through the threads of the attached process.
    ///
    /// The callback will be invoked for each thread in turn.
    pub fn threads<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnMut(&mut ThreadRef) -> Result<(), Error>,
    {
        unsafe {
            let mut state = ThreadsCallbackState {
                callback,
                panic: None,
                error: None,
            };
            let r = dw_sys::dwfl_getthreads(
                self.as_ptr(),
                Some(threads_cb::<F>),
                &mut state as *mut _ as *mut c_void,
            );

            if let Some(payload) = state.panic {
                panic::resume_unwind(payload);
            }
            if let Some(error) = state.error {
                return Err(error);
            }

            cvt(r)
        }
    }

    /// Iterates through the frames of a specific thread of the attached process.
    ///
    /// The callback will be invoked for each stack frame of the thread in turn.
    pub fn thread_frames<F>(&mut self, tid: u32, callback: F) -> Result<(), Error>
    where
        F: FnMut(&mut FrameRef) -> Result<(), Error>,
    {
        unsafe {
            let mut state = FramesCallbackState {
                callback,
                panic: None,
                error: None,
            };
            let r = dw_sys::dwfl_getthread_frames(
                self.as_ptr(),
                tid as pid_t,
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

    /// Looks up the module containing the address.
    pub fn addr_module(&self, address: u64) -> Result<&ModuleRef, Error> {
        unsafe {
            let ptr = dw_sys::dwfl_addrmodule(self.as_ptr(), address);
            if ptr.is_null() {
                Err(Error::new())
            } else {
                Ok(ModuleRef::from_ptr(ptr))
            }
        }
    }
}

/// A type used to register segments and modules with a DWFL session.
pub struct Report<'a, 'b>(&'a mut DwflRef<'b>);

impl<'a, 'b> Drop for Report<'a, 'b> {
    fn drop(&mut self) {
        unsafe {
            dw_sys::dwfl_report_end(self.0.as_ptr(), None, ptr::null_mut());
        }
    }
}

impl<'a, 'b> Report<'a, 'b> {
    /// Uses the `/proc` pseudo-filesystem to register the information for a specific running process.
    ///
    /// The `FindElf::LINUX_PROC` callback should be used with this method.
    pub fn linux_proc(&mut self, pid: u32) -> Result<(), Error> {
        unsafe {
            cvt(dw_sys::dwfl_linux_proc_report(
                self.0.as_ptr(),
                pid as pid_t,
            ))
        }
    }
}

struct ThreadsCallbackState<F> {
    callback: F,
    panic: Option<Box<dyn Any + Send>>,
    error: Option<Error>,
}

unsafe extern "C" fn threads_cb<F>(thread: *mut dw_sys::Dwfl_Thread, arg: *mut c_void) -> c_int
where
    F: FnMut(&mut ThreadRef) -> Result<(), Error>,
{
    let state = &mut *(arg as *mut ThreadsCallbackState<F>);
    let thread = ThreadRef::from_ptr_mut(thread);

    match panic::catch_unwind(AssertUnwindSafe(|| (state.callback)(thread))) {
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

struct FramesCallbackState<F> {
    callback: F,
    panic: Option<Box<dyn Any + Send>>,
    error: Option<Error>,
}

unsafe extern "C" fn frames_cb<F>(frame: *mut dw_sys::Dwfl_Frame, arg: *mut c_void) -> c_int
where
    F: FnMut(&mut FrameRef) -> Result<(), Error>,
{
    let state = &mut *(arg as *mut FramesCallbackState<F>);
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
