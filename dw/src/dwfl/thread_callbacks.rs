use crate::dwfl::{DwflRef, ThreadRef};
use foreign_types::ForeignTypeRef;
use libc::{c_void, pid_t};

pub trait ThreadCallbacks {
    fn next_thread(&mut self, dwfl: &DwflRef<'_>) -> Option<u32>;

    fn get_thread(&mut self, dwfl: &DwflRef<'_>, tid: u32) -> bool;

    fn memory_read(&mut self, dwfl: &DwflRef<'_>, addr: u64) -> Option<u64>;

    fn set_initial_registers(&mut self, thread: &mut ThreadRef) -> bool;

    fn detach(&mut self, dwfl: &DwflRef<'_>) {
        let _ = dwfl;
    }

    fn thread_detach(&mut self, thread: &mut ThreadRef) {
        let _ = thread;
    }
}

pub(crate) fn make_dwfl_thread_callbacks<T>() -> &'static dw_sys::Dwfl_Thread_Callbacks
where
    T: ThreadCallbacks,
{
    &dw_sys::Dwfl_Thread_Callbacks {
        next_thread: Some(next_thread_shim::<T>),
        get_thread: Some(get_thread_shim::<T>),
        memory_read: Some(memory_read_shim::<T>),
        set_initial_registers: Some(set_initial_registers_shim::<T>),
        detach: Some(detach_shim::<T>),
        thread_detach: Some(thread_detach_shim::<T>),
    }
}

unsafe extern "C" fn next_thread_shim<T>(
    dwfl: *mut dw_sys::Dwfl,
    dwfl_arg: *mut c_void,
    thread_argp: *mut *mut c_void,
) -> pid_t
where
    T: ThreadCallbacks,
{
    let dwfl = DwflRef::from_ptr(dwfl);
    let callbacks = &mut *dwfl_arg.cast::<T>();

    *thread_argp = dwfl_arg;
    match callbacks.next_thread(dwfl) {
        Some(pid) => pid as pid_t,
        None => 0,
    }
}

unsafe extern "C" fn get_thread_shim<T>(
    dwfl: *mut dw_sys::Dwfl,
    tid: pid_t,
    dwfl_arg: *mut c_void,
    thread_argp: *mut *mut c_void,
) -> bool
where
    T: ThreadCallbacks,
{
    let dwfl = DwflRef::from_ptr(dwfl);
    let callbacks = &mut *dwfl_arg.cast::<T>();

    *thread_argp = dwfl_arg;
    callbacks.get_thread(dwfl, tid as u32)
}

unsafe extern "C" fn memory_read_shim<T>(
    dwfl: *mut dw_sys::Dwfl,
    addr: dw_sys::Dwarf_Addr,
    result: *mut dw_sys::Dwarf_Word,
    dwfl_arg: *mut c_void,
) -> bool
where
    T: ThreadCallbacks,
{
    let dwfl = DwflRef::from_ptr(dwfl);
    let callbacks = &mut *dwfl_arg.cast::<T>();

    match callbacks.memory_read(dwfl, addr) {
        Some(v) => {
            *result = v;
            true
        }
        None => false,
    }
}

unsafe extern "C" fn set_initial_registers_shim<T>(
    thread: *mut dw_sys::Dwfl_Thread,
    thread_arg: *mut c_void,
) -> bool
where
    T: ThreadCallbacks,
{
    let thread = ThreadRef::from_ptr_mut(thread);
    let callbacks = &mut *thread_arg.cast::<T>();

    callbacks.set_initial_registers(thread)
}

unsafe extern "C" fn detach_shim<T>(dwfl: *mut dw_sys::Dwfl, dwfl_arg: *mut c_void)
where
    T: ThreadCallbacks,
{
    let dwfl = DwflRef::from_ptr(dwfl);
    let callbacks = &mut *dwfl_arg.cast::<T>();

    callbacks.detach(dwfl)
}

unsafe extern "C" fn thread_detach_shim<T>(
    thread: *mut dw_sys::Dwfl_Thread,
    thread_arg: *mut c_void,
) where
    T: ThreadCallbacks,
{
    let thread = ThreadRef::from_ptr_mut(thread);
    let callbacks = &mut *thread_arg.cast::<T>();

    callbacks.thread_detach(thread)
}
