use foreign_types::{ForeignType, ForeignTypeRef, Opaque};
use libc::{c_int, c_void, pid_t};
use std::any::Any;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::panic::{self, AssertUnwindSafe};
use std::ptr;

use crate::dwfl::{cvt, Callbacks, Error, FrameRef, ModuleRef, ThreadRef};

pub struct Dwfl<'a>(*mut dw_sys::Dwfl, PhantomData<&'a ()>);

impl<'a> Drop for Dwfl<'a> {
    fn drop(&mut self) {
        unsafe {
            dw_sys::dwfl_end(self.as_ptr());
        }
    }
}

impl<'a> ForeignType for Dwfl<'a> {
    type CType = dw_sys::Dwfl;
    type Ref = DwflRef<'a>;

    #[inline]
    unsafe fn from_ptr(ptr: *mut dw_sys::Dwfl) -> Dwfl<'a> {
        Dwfl(ptr, PhantomData)
    }

    #[inline]
    fn as_ptr(&self) -> *mut dw_sys::Dwfl {
        self.0
    }
}

impl<'a> Deref for Dwfl<'a> {
    type Target = DwflRef<'a>;

    #[inline]
    fn deref(&self) -> &DwflRef<'a> {
        unsafe { &*(self.as_ptr() as *mut DwflRef<'a>) }
    }
}

impl<'a> DerefMut for Dwfl<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut DwflRef<'a> {
        unsafe { &mut *(self.as_ptr() as *mut DwflRef<'a>) }
    }
}

impl<'a> Dwfl<'a> {
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

pub struct DwflRef<'a>(Opaque, PhantomData<&'a ()>);

impl<'a> ForeignTypeRef for DwflRef<'a> {
    type CType = dw_sys::Dwfl;
}

impl<'a> DwflRef<'a> {
    pub fn version(&self) -> &str {
        unsafe {
            let p = dw_sys::dwfl_version(self.as_ptr());
            CStr::from_ptr(p).to_str().unwrap()
        }
    }

    pub fn report(&mut self) -> Report<'_, 'a> {
        unsafe {
            dw_sys::dwfl_report_begin(self.as_ptr());
            Report(self)
        }
    }

    pub fn report_add(&mut self) -> Report<'_, 'a> {
        unsafe {
            dw_sys::dwfl_report_begin_add(self.as_ptr());
            Report(self)
        }
    }

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

pub struct Report<'a, 'b>(&'a mut DwflRef<'b>);

impl<'a, 'b> Drop for Report<'a, 'b> {
    fn drop(&mut self) {
        unsafe {
            dw_sys::dwfl_report_end(self.0.as_ptr(), None, ptr::null_mut());
        }
    }
}

impl<'a, 'b> Report<'a, 'b> {
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
    panic: Option<Box<Any + Send>>,
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
    panic: Option<Box<Any + Send>>,
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
