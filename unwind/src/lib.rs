extern crate libc;
extern crate unwind_sys;

#[macro_use]
#[allow(unused_imports)]
extern crate foreign_types;

use foreign_types::Opaque;
use libc::{c_char, c_int, c_void};
use std::fmt;
use std::error;
use std::mem;
use std::result;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use unwind_sys::*;

pub type Result<T> = result::Result<T, Error>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Error(c_int);

impl Error {
    pub const UNSPEC: Error = Error(-UNW_EUNSPEC);
    pub const NOMEM: Error = Error(-UNW_ENOMEM);
    pub const BADREG: Error = Error(-UNW_EBADREG);
    pub const READONLYREG: Error = Error(-UNW_EREADONLYREG);
    pub const STOPUNWIND: Error = Error(-UNW_ESTOPUNWIND);
    pub const INVALIDIP: Error = Error(-UNW_EINVALIDIP);
    pub const BADFRAME: Error = Error(-UNW_EBADFRAME);
    pub const INVAL: Error = Error(-UNW_EINVAL);
    pub const BADVERSION: Error = Error(-UNW_EBADVERSION);
    pub const NOINFO: Error = Error(-UNW_ENOINFO);
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let s = match -self.0 {
            UNW_EUNSPEC => "unspecified error",
            UNW_ENOMEM => "out of memory",
            UNW_EBADREG => "bad register number",
            UNW_EREADONLYREG => "attempt to write read-only register",
            UNW_ESTOPUNWIND => "stop unwinding",
            UNW_EINVALIDIP => "invalid IP",
            UNW_EBADFRAME => "bad frame",
            UNW_EINVAL => "unsupported operation or bad value",
            UNW_EBADVERSION => "unwind info has unsupported version",
            UNW_ENOINFO => "no unwind info found",
            _ => return write!(fmt, "unknown error {}", self.0),
        };
        fmt.write_str(s)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "libunwind error"
    }
}

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

    /// The endianness of the target architecture.
    pub const NATIVE_ENDIAN: Byteorder = Byteorder::_NATIVE_ENDIAN;

    #[cfg(target_endian = "little")]
    const _NATIVE_ENDIAN: Byteorder = Byteorder::LITTLE_ENDIAN;
    #[cfg(target_endian = "big")]
    const _NATIVE_ENDIAN: Byteorder = Byteorder::BIG_ENDIAN;
}

#[cfg(feature = "ptrace")]
foreign_type! {
    type CType = c_void;
    fn drop = _UPT_destroy;

    pub struct PTraceState;

    pub struct PTraceStateRef;
}

#[cfg(feature = "ptrace")]
impl PTraceState {
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

pub struct Accessors<T>(unw_accessors_t, PhantomData<T>);

#[cfg(feature = "ptrace")]
impl Accessors<PTraceStateRef> {
    pub fn ptrace() -> &'static Accessors<PTraceStateRef> {
        unsafe { &*(&_UPT_accessors as *const unw_accessors_t as *const Accessors<PTraceStateRef>) }
    }
}

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

pub struct AddressSpaceRef<T>(Opaque, PhantomData<T>);

impl<T> AddressSpaceRef<T> {
    fn as_ptr(&self) -> unw_addr_space_t {
        unsafe { &mut *(self as *const _ as *mut _) }
    }
}

#[derive(Copy, Clone)]
pub struct RegNum(c_int);

impl RegNum {
    pub const IP: RegNum = RegNum(UNW_REG_IP);
    pub const SP: RegNum = RegNum(UNW_REG_SP);
}

#[derive(Copy, Clone)]
pub struct ProcedureInfo {
    pub start_ip: u64,
    pub end_ip: u64,
    _p: (),
}

#[derive(Clone)]
pub struct Cursor<'a>(unw_cursor_t, PhantomData<(&'a ())>);

impl<'a> Cursor<'a> {
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

    pub fn step(&mut self) -> Result<bool> {
        unsafe {
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

    pub fn register(&self, num: RegNum) -> Result<u64> {
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

    pub fn procedure_info(&self) -> Result<ProcedureInfo> {
        unsafe {
            let mut info = mem::uninitialized();
            let ret = unw_get_proc_info(&self.0 as *const _ as *mut _, &mut info);
            if ret == UNW_ESUCCESS {
                Ok(ProcedureInfo {
                    start_ip: info.start_ip as u64,
                    end_ip: info.end_ip as u64,
                    _p: (),
                })
            } else {
                Err(Error(ret))
            }
        }
    }

    pub fn procedure_name(&self, buf: &mut [u8], offset: &mut u64) -> Result<()> {
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
}

#[cfg(test)]
mod test {
    use std::str;

    use super::*;

    #[test]
    fn local() {
        fn bar() {
            Cursor::local(|mut cursor| {
                let mut buf = [0; 256];
                loop {
                    let ip = cursor.register(RegNum::IP).unwrap();
                    let info = cursor.procedure_info().unwrap();
                    let mut offset = 0;
                    let ok = match cursor.procedure_name(&mut buf, &mut offset) {
                        Ok(()) => true,
                        Err(Error::NOMEM) => true,
                        Err(_) => false,
                    };

                    if ok {
                        let len = buf.iter().position(|b| *b == 0).unwrap();
                        let name = str::from_utf8(&buf[..len]).unwrap();
                        println!(
                            "{:#x} - {} ({:#x}) + {:#x}",
                            ip,
                            name,
                            info.start_ip,
                            offset
                        );
                    } else {
                        println!("{:#x} - ????", ip);
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

        let mut buf = [0; 256];
        loop {
            let ip = cursor.register(RegNum::IP).unwrap();
            let info = cursor.procedure_info().unwrap();
            let mut offset = 0;
            match cursor.procedure_name(&mut buf, &mut offset) {
                Ok(()) => {}
                Err(Error::NOMEM) => {}
                Err(e) => panic!("{}", e),
            }

            let len = buf.iter().position(|b| *b == 0).unwrap();
            let name = str::from_utf8(&buf[..len]).unwrap();
            println!(
                "{:#x} - {} ({:#x}) + {:#x}",
                ip,
                name,
                info.start_ip,
                offset
            );

            if !cursor.step().unwrap() {
                break;
            }
        }

        child.kill().unwrap();
    }
}
