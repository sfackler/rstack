//! Thread stack traces of remote processes.
//!
//! `rstack` (named after Java's `jstack`) uses [libunwind]'s ptrace interface to capture stack
//! traces of the threads of a remote process. It currently only supports Linux with a kernel
//! version of 3.4 or higher, and requires that the `/proc` pseudo-filesystem be mounted and
//! accessible.
//!
//! [libunwind]: http://www.nongnu.org/libunwind/

#![warn(missing_docs)]
extern crate libc;
extern crate unwind;

#[macro_use]
extern crate log;

use libc::{c_void, pid_t, ptrace, waitpid, ESRCH, PTRACE_DETACH, PTRACE_INTERRUPT, PTRACE_SEIZE,
           WIFSTOPPED, __WALL};
use std::borrow::Borrow;
use std::result;
use std::io::{self, Read};
use std::fmt;
use std::fs::{self, File};
use std::error;
use std::collections::BTreeSet;
use std::ptr;
use unwind::{Accessors, AddressSpace, Byteorder, Cursor, PTraceState, PTraceStateRef, RegNum};

/// The result type returned by methods in this crate.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
enum ErrorInner {
    Io(io::Error),
    Unwind(unwind::Error),
}

/// The error type returned by methods in this crate.
#[derive(Debug)]
pub struct Error(ErrorInner);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ErrorInner::Io(ref e) => fmt::Display::fmt(e, fmt),
            ErrorInner::Unwind(ref e) => fmt::Display::fmt(e, fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "rstack error"
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.0 {
            ErrorInner::Io(ref e) => Some(e),
            ErrorInner::Unwind(ref e) => Some(e),
        }
    }
}

/// Information about a remote process.
#[derive(Debug, Clone)]
pub struct Process {
    id: u32,
    threads: Vec<Thread>,
}

impl Process {
    /// Returns the process's ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns information about the threads of the process.
    pub fn threads(&self) -> &[Thread] {
        &self.threads
    }
}

/// Information about a thread of a remote process.
#[derive(Debug, Clone)]
pub struct Thread {
    id: u32,
    name: Option<String>,
    frames: Vec<Frame>,
}

impl Thread {
    /// Returns the thread's ID.
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the thread's name, if known.
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| &**s)
    }

    /// Returns the frames of the stack trace representing the state of the thread.
    #[inline]
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
}

/// Information about a stack frame of a remote process.
#[derive(Debug, Clone)]
pub struct Frame {
    ip: usize,
    is_signal: Option<bool>,
    name: Option<ProcedureName>,
    info: Option<ProcedureInfo>,
}

impl Frame {
    /// Returns the instruction pointer of the frame.
    #[inline]
    pub fn ip(&self) -> usize {
        self.ip
    }

    /// Determines if the frame is from a signal handler, if known.
    #[inline]
    pub fn is_signal(&self) -> Option<bool> {
        self.is_signal
    }

    /// Returns the name of the procedure that this frame is running, if known.
    ///
    /// In certain contexts, particularly when the binary being traced or its dynamic libraries have
    /// been stripped, the unwinder may not have enough information to properly identify the
    /// procedure and will simply return the first label before the frame's instruction pointer. The
    /// offset will always be relative to this label.
    #[inline]
    pub fn name(&self) -> Option<&ProcedureName> {
        self.name.as_ref()
    }

    /// Returns information about the procedure that this frame is running, if known.
    #[inline]
    pub fn info(&self) -> Option<&ProcedureInfo> {
        self.info.as_ref()
    }
}

/// Information about a name of a procedure.
#[derive(Debug, Clone)]
pub struct ProcedureName {
    name: String,
    offset: usize,
}

impl ProcedureName {
    /// Returns the name of the procedure.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the offset of the instruction pointer from this procedure's starting address.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }
}

/// Information about a procedure.
#[derive(Debug, Clone)]
pub struct ProcedureInfo {
    start_ip: usize,
    end_ip: usize,
}

impl ProcedureInfo {
    /// Returns the starting address of this procedure.
    #[inline]
    pub fn start_ip(&self) -> usize {
        self.start_ip
    }

    /// Returns the ending address of this procedure.
    #[inline]
    pub fn end_ip(&self) -> usize {
        self.end_ip
    }
}

/// A struct controlling the behavior of tracing.
#[derive(Debug, Clone)]
pub struct TraceOptions {
    thread_names: bool,
    procedure_names: bool,
    procedure_info: bool,
}

impl Default for TraceOptions {
    fn default() -> TraceOptions {
        TraceOptions {
            thread_names: false,
            procedure_names: false,
            procedure_info: false,
        }
    }
}

impl TraceOptions {
    /// Returns a new `TraceOptions` with default settings.
    pub fn new() -> TraceOptions {
        TraceOptions::default()
    }

    /// If set, the names of the process's threads will be recorded.
    ///
    /// Defaults to `false`.
    pub fn thread_names(&mut self, thread_names: bool) -> &mut TraceOptions {
        self.thread_names = thread_names;
        self
    }

    /// If set, the names of the procedures running in the frames of the process's threads will be
    /// recorded.
    ///
    /// Defaults to `false`.
    pub fn procedure_names(&mut self, procedure_names: bool) -> &mut TraceOptions {
        self.procedure_names = procedure_names;
        self
    }

    /// If set, information about the procedures running in the frames of the process's threads will
    /// be recorded.
    ///
    /// Defaults to `false`.
    pub fn procedure_info(&mut self, procedure_info: bool) -> &mut TraceOptions {
        self.procedure_info = procedure_info;
        self
    }

    /// Traces the threads of the specified process.
    pub fn trace(&self, pid: u32) -> Result<Process> {
        let space = AddressSpace::new(Accessors::ptrace(), Byteorder::DEFAULT)
            .map_err(|e| Error(ErrorInner::Unwind(e)))?;
        let threads = get_threads(pid)?;

        let mut traces = vec![];

        for thread in &threads {
            let name = if self.thread_names {
                get_name(pid, thread.0)
            } else {
                None
            };

            match thread.dump(&space, self) {
                Ok(frames) => traces.push(Thread {
                    id: thread.0,
                    name,
                    frames,
                }),
                Err(e) => debug!("error tracing thread {}: {}", thread.0, e),
            }
        }

        Ok(Process {
            id: pid,
            threads: traces,
        })
    }
}

/// A convenience wrapper over `TraceOptions` which returns a maximally verbose trace.
pub fn trace(pid: u32) -> Result<Process> {
    TraceOptions::new()
        .thread_names(true)
        .procedure_names(true)
        .procedure_info(true)
        .trace(pid)
}

fn get_threads(pid: u32) -> Result<BTreeSet<TracedThread>> {
    let mut threads = BTreeSet::new();

    let path = format!("/proc/{}/task", pid);

    // new threads may be created while we're in the process of stopping them all, so loop a couple
    // of times to hopefully converge
    for _ in 0..5 {
        let prev = threads.len();
        add_threads(&mut threads, &path)?;
        if prev == threads.len() {
            break;
        }
    }

    Ok(threads)
}

fn add_threads(threads: &mut BTreeSet<TracedThread>, dir: &str) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| Error(ErrorInner::Io(e)))? {
        let entry = entry.map_err(|e| Error(ErrorInner::Io(e)))?;

        let pid = match entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        {
            Some(pid) => pid,
            None => continue,
        };

        if !threads.contains(&pid) {
            let thread = match TracedThread::new(pid) {
                Ok(thread) => thread,
                // ESRCH just means the thread died in the middle of things, which is fine
                Err(e) => if e.raw_os_error() == Some(ESRCH) {
                    debug!("error attaching to thread {}: {}", pid, e);
                    continue;
                } else {
                    return Err(Error(ErrorInner::Io(e)));
                },
            };
            threads.insert(thread);
        }
    }

    Ok(())
}

fn get_name(pid: u32, tid: u32) -> Option<String> {
    let path = format!("/proc/{}/task/{}/comm", pid, tid);
    let mut name = vec![];
    match File::open(path).and_then(|mut f| f.read_to_end(&mut name)) {
        Ok(_) => Some(String::from_utf8_lossy(&name).trim().to_string()),
        Err(e) => {
            debug!("error getting name for thread {}: {}", tid, e);
            None
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct TracedThread(u32);

impl Drop for TracedThread {
    fn drop(&mut self) {
        unsafe {
            ptrace(
                PTRACE_DETACH,
                self.0 as pid_t,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            );
        }
    }
}

impl Borrow<u32> for TracedThread {
    fn borrow(&self) -> &u32 {
        &self.0
    }
}

impl TracedThread {
    fn new(pid: u32) -> io::Result<TracedThread> {
        unsafe {
            let ret = ptrace(
                PTRACE_SEIZE,
                pid as pid_t,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            );
            if ret != 0 {
                return Err(io::Error::last_os_error());
            }

            let thread = TracedThread(pid);

            let ret = ptrace(
                PTRACE_INTERRUPT,
                pid as pid_t,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            );
            if ret != 0 {
                return Err(io::Error::last_os_error());
            }

            let mut status = 0;
            while waitpid(pid as pid_t, &mut status, __WALL) < 0 {
                let e = io::Error::last_os_error();
                if e.kind() != io::ErrorKind::Interrupted {
                    return Err(e);
                }
            }

            if !WIFSTOPPED(status) {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("unexpected wait status {}", status),
                ));
            }

            Ok(thread)
        }
    }

    fn dump(
        &self,
        space: &AddressSpace<PTraceStateRef>,
        options: &TraceOptions,
    ) -> unwind::Result<Vec<Frame>> {
        let state = PTraceState::new(self.0)?;
        let mut cursor = Cursor::remote(&space, &state)?;

        let mut trace = vec![];
        loop {
            let ip = cursor.register(RegNum::IP)? as usize;
            let is_signal = cursor.is_signal_frame().ok();

            let name = if options.procedure_names {
                cursor.procedure_name().ok().map(|n| {
                    ProcedureName {
                        name: n.name().to_string(),
                        offset: n.offset() as usize,
                    }
                })
            } else {
                None
            };

            let info = if options.procedure_info {
                cursor.procedure_info().ok().map(|i| {
                    ProcedureInfo {
                        start_ip: i.start_ip() as usize,
                        end_ip: i.end_ip() as usize,
                    }
                })
            } else {
                None
            };

            trace.push(Frame {
                ip,
                is_signal,
                name,
                info,
            });

            if !cursor.step()? {
                break;
            }
        }

        Ok(trace)
    }
}
