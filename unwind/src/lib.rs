//! An interface to [libunwind].
//!
//! libunwind provides access to the call chain of a process. It supports both local and remote
//! processes.
//!
//! # Examples
//!
//! Printing a backtrace of the current thread:
//!
//! ```
//! use unwind::{Cursor, RegNum};
//!
//! Cursor::local(|mut cursor| {
//!     loop {
//!         let ip = cursor.register(RegNum::IP)?;
//!
//!         match (cursor.procedure_info(), cursor.procedure_name()) {
//!             (Ok(ref info), Ok(ref name)) if ip == info.start_ip() + name.offset() => {
//!                 println!(
//!                     "{:#016x} - {} ({:#016x}) + {:#x}",
//!                     ip,
//!                     name.name(),
//!                     info.start_ip(),
//!                     name.offset()
//!                 );
//!             }
//!             _ => println!("{:#016x} - ????", ip),
//!         }
//!
//!         if !cursor.step()? {
//!             break;
//!         }
//!     }
//!
//!     Ok(())
//! }).unwrap();
//! ```
//!
//! [libunwind]: http://www.nongnu.org/libunwind/
#![warn(missing_docs)]
extern crate libc;
extern crate unwind_sys;

#[macro_use]
#[allow(unused_imports)]
extern crate foreign_types;

use foreign_types::Opaque;
use libc::{c_char, c_int, c_void};
use std::ffi::CStr;
use std::fmt;
use std::error;
use std::mem;
use std::result;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use unwind_sys::*;

/// The result type returned by functions in this crate.
pub type Result<T> = result::Result<T, Error>;

/// An error returned from libunwind.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Error(c_int);

impl Error {
    /// Unspecified error.
    pub const UNSPEC: Error = Error(-UNW_EUNSPEC);

    /// Out of memory.
    pub const NOMEM: Error = Error(-UNW_ENOMEM);

    /// Bad register number.
    pub const BADREG: Error = Error(-UNW_EBADREG);

    /// Attempt to write read-only register.
    pub const READONLYREG: Error = Error(-UNW_EREADONLYREG);

    /// Stop unwinding.
    pub const STOPUNWIND: Error = Error(-UNW_ESTOPUNWIND);

    /// Invalid IP.
    pub const INVALIDIP: Error = Error(-UNW_EINVALIDIP);

    /// Bad frame.
    pub const BADFRAME: Error = Error(-UNW_EBADFRAME);

    /// Unsupported operation or bad value.
    pub const INVAL: Error = Error(-UNW_EINVAL);

    /// Unwind info has unsupported value.
    pub const BADVERSION: Error = Error(-UNW_EBADVERSION);

    /// No unwind info found.
    pub const NOINFO: Error = Error(-UNW_ENOINFO);
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let err = unw_strerror(self.0);
            let err = CStr::from_ptr(err).to_string_lossy();
            fmt.write_str(&err)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "libunwind error"
    }
}

/// The byteorder of an address space.
#[derive(Copy, Clone)]
pub struct Byteorder(c_int);

impl Byteorder {
    /// The default byte order of the unwind target.
    pub const DEFAULT: Byteorder = Byteorder(0);

    /// Little endian.
    pub const LITTLE_ENDIAN: Byteorder = Byteorder(1234);

    /// Big endian.
    pub const BIG_ENDIAN: Byteorder = Byteorder(4321);

    /// PDP endian.
    pub const PDP_ENDIAN: Byteorder = Byteorder(3412);
}

#[cfg(feature = "ptrace")]
foreign_type! {
    type CType = c_void;
    fn drop = _UPT_destroy;

    /// The unwind state used by the ptrace accessors.
    ///
    /// The `ptrace` Cargo feature must be enabled to use this type.
    pub struct PTraceState;

    /// A borrowed reference to a [`PTraceState`].
    ///
    /// [`PTraceState`]: struct.PTraceState.html
    pub struct PTraceStateRef;
}

#[cfg(feature = "ptrace")]
impl PTraceState {
    /// Constructs a new `PTraceState` for the specified PID.
    ///
    /// The process must already be attached and suspended before unwinding can be performed.
    pub fn new(pid: u32) -> Result<PTraceState> {
        unsafe {
            let ptr = _UPT_create(pid as _);
            if ptr.is_null() {
                // this is documented to only fail on OOM
                Err(Error(-UNW_ENOMEM))
            } else {
                Ok(PTraceState(ptr))
            }
        }
    }
}

/// A collection of functions used to unwind an arbitrary process.
pub struct Accessors<T>(unw_accessors_t, PhantomData<T>);

#[cfg(feature = "ptrace")]
impl Accessors<PTraceStateRef> {
    /// Returns `Accessors` which use the ptrace system call to unwind a remote process.
    ///
    /// The `ptrace` Cargo feature must be enabled to use this type.
    pub fn ptrace() -> &'static Accessors<PTraceStateRef> {
        unsafe { &*(&_UPT_accessors as *const unw_accessors_t as *const Accessors<PTraceStateRef>) }
    }
}

/// An address space upon which unwinding can be performed.
pub struct AddressSpace<T>(unw_addr_space_t, PhantomData<T>);

impl<T> Drop for AddressSpace<T> {
    fn drop(&mut self) {
        unsafe {
            unw_destroy_addr_space(self.0);
        }
    }
}

impl<T> Deref for AddressSpace<T> {
    type Target = AddressSpaceRef<T>;

    fn deref(&self) -> &AddressSpaceRef<T> {
        unsafe { &*(self.0 as *const AddressSpaceRef<T>) }
    }
}

impl<T> DerefMut for AddressSpace<T> {
    fn deref_mut(&mut self) -> &mut AddressSpaceRef<T> {
        unsafe { &mut *(self.0 as *mut AddressSpaceRef<T>) }
    }
}

impl<T> AddressSpace<T> {
    /// Creates a new `AddressSpace`.
    pub fn new(accessors: &Accessors<T>, byteorder: Byteorder) -> Result<AddressSpace<T>> {
        unsafe {
            let ptr = unw_create_addr_space(
                &accessors.0 as *const unw_accessors_t as *mut unw_accessors_t,
                byteorder.0,
            );
            if ptr.is_null() {
                Err(Error(-UNW_EUNSPEC))
            } else {
                Ok(AddressSpace(ptr, PhantomData))
            }
        }
    }
}

/// A borrowed reference to an [`AddressSpace`].
///
/// [`AddressSpace`]: struct.AddressSpace.html
pub struct AddressSpaceRef<T>(Opaque, PhantomData<T>);

impl<T> AddressSpaceRef<T> {
    fn as_ptr(&self) -> unw_addr_space_t {
        unsafe { &mut *(self as *const _ as *mut _) }
    }
}

/// An identifier of a processor register.
#[derive(Copy, Clone)]
pub struct RegNum(c_int);

impl RegNum {
    /// A generic identifier for the register storing the instruction pointer.
    pub const IP: RegNum = RegNum(UNW_REG_IP);

    /// A generic identifier for the register storing the stack pointer.
    pub const SP: RegNum = RegNum(UNW_REG_SP);
}

/// Information about a procedure.
#[derive(Copy, Clone)]
pub struct ProcedureInfo {
    start_ip: u64,
    end_ip: u64,
}

impl ProcedureInfo {
    /// Returns the starting address of the procedure.
    pub fn start_ip(&self) -> u64 {
        self.start_ip
    }

    /// Returns the ending address of the procedure.
    pub fn end_ip(&self) -> u64 {
        self.end_ip
    }
}

/// The name of a procedure.
#[derive(Clone)]
pub struct ProcedureName {
    name: String,
    offset: u64,
}

impl ProcedureName {
    /// Returns the name of the procedure.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the offset of the frame's instruction pointer from the starting address of the named
    /// procedure.
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

/// A cursor into a frame of a stack.
///
/// The cursor starts at the current (topmost) frame, and can be advanced downwards through the
/// stack. While a cursor cannot be run "backwards", it can be cloned, and one of the copies
/// advanced while the other continues to refer to the previous frame.
#[derive(Clone)]
pub struct Cursor<'a>(unw_cursor_t, PhantomData<(&'a ())>);

impl<'a> Cursor<'a> {
    /// Creates a cursor over the stack of the calling thread.
    ///
    /// The cursor is provided to a closure rather than being returned because the stack frame being
    /// referenced by the frame must remain alive.
    pub fn local<F, T>(f: F) -> Result<T>
    where
        F: FnOnce(Cursor) -> Result<T>,
    {
        unsafe {
            let mut context = mem::uninitialized();
            let ret = unw_tdep_getcontext(&mut context);
            if ret != UNW_ESUCCESS {
                return Err(Error(ret));
            }

            let mut cursor = mem::uninitialized();
            let ret = unw_init_local(&mut cursor, &mut context);
            if ret != UNW_ESUCCESS {
                return Err(Error(ret));
            }

            f(Cursor(cursor, PhantomData))
        }
    }

    /// Creates a cursor over the stack of a "remote" process.
    pub fn remote<T>(address_space: &'a AddressSpaceRef<T>, state: &'a T) -> Result<Cursor<'a>> {
        unsafe {
            let mut cursor = mem::uninitialized();
            let ret = unw_init_remote(
                &mut cursor,
                address_space.as_ptr(),
                state as *const T as *mut c_void,
            );
            if ret == UNW_ESUCCESS {
                Ok(Cursor(cursor, PhantomData))
            } else {
                Err(Error(ret))
            }
        }
    }

    /// Steps the cursor into the next older stack frame.
    ///
    /// A return value of `false` indicates that the cursor is at the last frame of the stack.
    pub fn step(&mut self) -> Result<bool> {
        unsafe {
            // libunwind 1.1 seems to get confused and walks off the end of the stack. The last IP
            // it reports is 0, so we'll stop if we're there.
            if cfg!(pre12) && self.register(RegNum::IP).unwrap_or(1) == 0 {
                return Ok(false);
            }

            let ret = unw_step(&mut self.0);
            if ret > 0 {
                Ok(true)
            } else if ret == 0 {
                Ok(false)
            } else {
                Err(Error(ret))
            }
        }
    }

    /// Returns the value of an integral register at the current frame.
    ///
    /// Based on the calling convention, some registers may not be available in a stack frame.
    pub fn register(&mut self, num: RegNum) -> Result<u64> {
        unsafe {
            let mut val = 0;
            let ret = unw_get_reg(&self.0 as *const _ as *mut _, num.0, &mut val);
            if ret == UNW_ESUCCESS {
                Ok(val)
            } else {
                Err(Error(ret))
            }
        }
    }

    /// Returns information about the procedure at the current frame.
    pub fn procedure_info(&mut self) -> Result<ProcedureInfo> {
        unsafe {
            let mut info = mem::uninitialized();
            let ret = unw_get_proc_info(&self.0 as *const _ as *mut _, &mut info);
            if ret == UNW_ESUCCESS {
                Ok(ProcedureInfo {
                    start_ip: info.start_ip as u64,
                    end_ip: info.end_ip as u64,
                })
            } else {
                Err(Error(ret))
            }
        }
    }

    /// Returns the name of the procedure of the current frame.
    ///
    /// The name is copied into the provided buffer, and is null-terminated. If the buffer is too
    /// small to hold the full name, [`Error::NOMEM`] is returned and the buffer contains the
    /// portion of the name that fits (including the null terminator).
    ///
    /// The offset of the instruction pointer from the beginning of the identified procedure is
    /// copied into the `offset` parameter.
    ///
    /// The `procedure_name` method provides a higher level wrapper over this method.
    ///
    /// In certain contexts, particularly when the binary being unwound has been stripped, the
    /// unwinder may not have enough information to properly identify the procedure and will simply
    /// return the first label before the frame's instruction pointer. The offset will always be
    /// relative to this label.
    ///
    /// [`Error::NOMEM`]: struct.Error.html#associatedconstant.NOMEM
    pub fn procedure_name_raw(&mut self, buf: &mut [u8], offset: &mut u64) -> Result<()> {
        unsafe {
            let mut raw_off = 0;
            let ret = unw_get_proc_name(
                &self.0 as *const _ as *mut _,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
                &mut raw_off,
            );
            *offset = raw_off as u64;
            if ret == UNW_ESUCCESS {
                Ok(())
            } else {
                Err(Error(ret))
            }
        }
    }

    /// Returns the name of the procedure of the current frame.
    ///
    /// In certain contexts, particularly when the binary being unwound has been stripped, the
    /// unwinder may not have enough information to properly identify the procedure and will simply
    /// return the first label before the frame's instruction pointer. The offset will always be
    /// relative to this label.
    pub fn procedure_name(&mut self) -> Result<ProcedureName> {
        let mut buf = vec![0; 256];
        loop {
            let mut offset = 0;
            match self.procedure_name_raw(&mut buf, &mut offset) {
                Ok(()) => {
                    let len = buf.iter().position(|b| *b == 0).unwrap();
                    buf.truncate(len);
                    let name = String::from_utf8_lossy(&buf).into_owned();
                    return Ok(ProcedureName {
                        name,
                        offset: offset as u64,
                    });
                }
                Err(Error::NOMEM) => {
                    let len = buf.len() * 2;
                    buf.resize(len, 0);
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Determines if the current frame is a signal frame.
    ///
    /// Signal frames are unique in several ways. More register state is available than normal, and
    /// the instruction pointer references the currently executing instruction rather than the next
    /// instruction.
    pub fn is_signal_frame(&mut self) -> Result<bool> {
        unsafe {
            let ret = unw_is_signal_frame(&self.0 as *const _ as *mut _);
            if ret < 0 {
                Err(Error(ret))
            } else {
                Ok(ret != 0)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn local() {
        fn bar() {
            Cursor::local(|mut cursor| {
                loop {
                    let ip = cursor.register(RegNum::IP).unwrap();

                    match (cursor.procedure_info(), cursor.procedure_name()) {
                        (Ok(ref info), Ok(ref name)) if ip == info.start_ip + name.offset => {
                            println!(
                                "{:#016x} - {} ({:#016x}) + {:#x}",
                                ip,
                                name.name,
                                info.start_ip,
                                name.offset
                            );
                        }
                        _ => {
                            println!("{:#016x} - ????", ip);
                        }
                    }

                    if !cursor.step().unwrap() {
                        break;
                    }
                }

                Ok(())
            }).unwrap();
        }
        fn foo() {
            bar();
        }
        foo();
    }

    #[test]
    #[cfg(feature = "ptrace")]
    fn remote() {
        use std::process::Command;
        use std::io;
        use std::ptr;

        let mut child = Command::new("sleep").arg("10").spawn().unwrap();
        unsafe {
            let ret = libc::ptrace(
                libc::PTRACE_ATTACH,
                child.id() as libc::pid_t,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            );
            if ret != 0 {
                panic!("{}", io::Error::last_os_error());
            }

            loop {
                let mut status = 0;
                let ret = libc::waitpid(child.id() as libc::pid_t, &mut status, 0);
                if ret < 0 {
                    panic!("{}", io::Error::last_os_error());
                }
                if libc::WIFSTOPPED(status) {
                    break;
                }
            }
        }
        let state = PTraceState::new(child.id() as _).unwrap();
        let address_space = AddressSpace::new(Accessors::ptrace(), Byteorder::DEFAULT).unwrap();
        let mut cursor = Cursor::remote(&address_space, &state).unwrap();

        loop {
            let ip = cursor.register(RegNum::IP).unwrap();

            match (cursor.procedure_info(), cursor.procedure_name()) {
                (Ok(ref info), Ok(ref name)) if ip == info.start_ip + name.offset => {
                    println!(
                        "{:#016x} - {} ({:#016x}) + {:#x}",
                        ip,
                        name.name,
                        info.start_ip,
                        name.offset
                    );
                }
                _ => println!("{:#016x} - ????", ip),
            }

            if !cursor.step().unwrap() {
                break;
            }
        }

        child.kill().unwrap();
    }
}
