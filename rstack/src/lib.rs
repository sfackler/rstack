//! Thread stack traces of remote processes.
//!
//! `rstack` (named after Java's `jstack`) uses ptrace to capture stack traces of the threads of a remote process. It
//! currently only supports Linux, and requires that the `/proc` pseudo-filesystem be mounted and accessible. Multiple
//! unwinding implementations are supported via Cargo features:
//!
//! * `unwind`: Uses [libunwind].
//! * `dw`: Uses libdw, part of the [elfutils] project.
//!
//! By default, the libunwind backend is used. You can switch to libdw via Cargo:
//!
//! ```toml
//! [dependencies]
//! rstack = { version = "0.1", features = ["dw"], default-features = false }
//! ```
//!
//! [libunwind]: http://www.nongnu.org/libunwind/
//! [elfutils]: https://sourceware.org/elfutils/
#![doc(html_root_url = "https://sfackler.github.io/rstack/doc")]
#![warn(missing_docs)]

use cfg_if::cfg_if;
use libc::{
    c_void, pid_t, ptrace, waitpid, ESRCH, PTRACE_ATTACH, PTRACE_CONT, PTRACE_DETACH,
    PTRACE_INTERRUPT, PTRACE_SEIZE, SIGSTOP, WIFSTOPPED, WSTOPSIG, __WALL,
};
use log::debug;
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::ptr;
use std::result;

cfg_if! {
    if #[cfg(feature = "dw")] {
        #[path = "imp/dw.rs"]
        mod imp;
    } else if #[cfg(feature = "unwind")] {
        #[path = "imp/unwind.rs"]
        mod imp;
    } else {
        compile_error!("You must select an unwinding implementation");
    }
}

/// The result type returned by methods in this crate.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
enum ErrorInner {
    Io(io::Error),
    Unwind(imp::Error),
}

/// The error type returned by methods in this crate.
#[derive(Debug)]
pub struct Error(ErrorInner);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ErrorInner::Io(ref e) => fmt::Display::fmt(e, fmt),
            ErrorInner::Unwind(ref e) => fmt::Display::fmt(e, fmt),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
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
    ip: u64,
    is_signal: bool,
    symbol: Option<Symbol>,
}

impl Frame {
    /// Returns the instruction pointer of the frame.
    #[inline]
    pub fn ip(&self) -> u64 {
        self.ip
    }

    /// Determines if the frame is from a signal handler.
    #[inline]
    pub fn is_signal(&self) -> bool {
        self.is_signal
    }

    /// Returns information about the symbol corresponding to this frame's instruction pointer, if known.
    #[inline]
    pub fn symbol(&self) -> Option<&Symbol> {
        self.symbol.as_ref()
    }
}

/// Information about the symbol corresponding to a stack frame.
#[derive(Debug, Clone)]
pub struct Symbol {
    name: String,
    offset: u64,
    address: u64,
    size: u64,
}

impl Symbol {
    /// Returns the name of the procedure.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the offset of the instruction pointer from the symbol's starting address.
    #[inline]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Returns the starting address of the symbol.
    #[inline]
    pub fn address(&self) -> u64 {
        self.address
    }

    /// Returns the size of the symbol.
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }
}

/// A convenience wrapper over `TraceOptions` which returns a maximally verbose trace.
pub fn trace(pid: u32) -> Result<Process> {
    TraceOptions::new()
        .thread_names(true)
        .symbols(true)
        .trace(pid)
}

/// Options controlling the behavior of tracing.
#[derive(Debug, Clone)]
pub struct TraceOptions {
    snapshot: bool,
    thread_names: bool,
    symbols: bool,
    ptrace_attach: bool,
}

impl Default for TraceOptions {
    fn default() -> TraceOptions {
        TraceOptions {
            snapshot: false,
            thread_names: false,
            symbols: false,
            ptrace_attach: true,
        }
    }
}

impl TraceOptions {
    /// Returns a new `TraceOptions` with default settings.
    pub fn new() -> TraceOptions {
        TraceOptions::default()
    }

    /// If set, the threads of the process will be traced in a consistent snapshot.
    ///
    /// A snapshot-mode trace ensures a consistent view of all threads, but requires that all
    /// threads be paused for the entire duration of the trace.
    ///
    /// Defaults to `false`.
    pub fn snapshot(&mut self, snapshot: bool) -> &mut TraceOptions {
        self.snapshot = snapshot;
        self
    }

    /// If set, the names of the process's threads will be recorded.
    ///
    /// Defaults to `false`.
    pub fn thread_names(&mut self, thread_names: bool) -> &mut TraceOptions {
        self.thread_names = thread_names;
        self
    }

    /// If set, information about the symbol at each frame will be recorded.
    ///
    /// Defaults to `false`.
    pub fn symbols(&mut self, symbols: bool) -> &mut TraceOptions {
        self.symbols = symbols;
        self
    }

    /// If set, `rstack` will automatically attach to threads via ptrace.
    ///
    /// If disabled, the calling process must already be attached to all traced threads, and the
    /// threads must be in the stopped state.
    ///
    /// Defaults to `true`.
    pub fn ptrace_attach(&mut self, ptrace_attach: bool) -> &mut TraceOptions {
        self.ptrace_attach = ptrace_attach;
        self
    }

    /// Traces the threads of the specified process.
    pub fn trace(&self, pid: u32) -> Result<Process> {
        let mut state = imp::State::new(pid).map_err(|e| Error(ErrorInner::Unwind(e)))?;

        let threads = if self.snapshot {
            self.trace_snapshot(pid, &mut state)?
        } else {
            self.trace_rolling(pid, &mut state)?
        };

        Ok(Process { id: pid, threads })
    }

    fn trace_snapshot(&self, pid: u32, state: &mut imp::State) -> Result<Vec<Thread>> {
        let threads = snapshot_threads(pid, self.ptrace_attach)?
            .iter()
            .map(|t| t.info(pid, state, self))
            .collect();

        Ok(threads)
    }

    fn trace_rolling(&self, pid: u32, state: &mut imp::State) -> Result<Vec<Thread>> {
        let mut threads = vec![];

        each_thread(pid, |tid| {
            let thread = if self.ptrace_attach {
                TracedThread::attach(tid)
            } else {
                TracedThread::traced(tid)
            };
            let thread = match thread {
                Ok(thread) => thread,
                Err(ref e) if e.raw_os_error() == Some(ESRCH) => {
                    debug!("error attaching to thread {}: {}", tid, e);
                    return Ok(());
                }
                Err(e) => return Err(Error(ErrorInner::Io(e))),
            };

            let trace = thread.info(pid, state, self);
            threads.push(trace);
            Ok(())
        })?;

        Ok(threads)
    }
}

fn snapshot_threads(pid: u32, ptrace_attach: bool) -> Result<BTreeSet<TracedThread>> {
    let mut threads = BTreeSet::new();

    // new threads may be created while we're in the process of stopping them all, so loop a couple
    // of times to hopefully converge
    for _ in 0..5 {
        let prev = threads.len();
        add_threads(&mut threads, pid, ptrace_attach)?;
        if prev == threads.len() {
            break;
        }
    }

    Ok(threads)
}

fn add_threads(threads: &mut BTreeSet<TracedThread>, pid: u32, ptrace_attach: bool) -> Result<()> {
    each_thread(pid, |tid| {
        if !threads.contains(&tid) {
            let thread = if ptrace_attach {
                TracedThread::attach(pid)
            } else {
                TracedThread::traced(pid)
            };
            let thread = match thread {
                Ok(thread) => thread,
                // ESRCH just means the thread died in the middle of things, which is fine
                Err(e) => {
                    if e.raw_os_error() == Some(ESRCH) {
                        debug!("error attaching to thread {}: {}", pid, e);
                        return Ok(());
                    } else {
                        return Err(Error(ErrorInner::Io(e)));
                    }
                }
            };
            threads.insert(thread);
        }

        Ok(())
    })
}

fn each_thread<F>(pid: u32, mut f: F) -> Result<()>
where
    F: FnMut(u32) -> Result<()>,
{
    let dir = format!("/proc/{}/task", pid);
    for entry in fs::read_dir(dir).map_err(|e| Error(ErrorInner::Io(e)))? {
        let entry = entry.map_err(|e| Error(ErrorInner::Io(e)))?;

        if let Some(tid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        {
            f(tid)?;
        }
    }
    Ok(())
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct TracedThread {
    id: u32,
    // True if TraceOptions::ptrace_attach was true (default value)
    // It means that Drop should perform detach
    should_detach: bool,
}

impl Drop for TracedThread {
    fn drop(&mut self) {
        if self.should_detach {
            unsafe {
                ptrace(
                    PTRACE_DETACH,
                    self.id as pid_t,
                    ptr::null_mut::<c_void>(),
                    ptr::null_mut::<c_void>(),
                );
            }
        }
    }
}

impl Borrow<u32> for TracedThread {
    fn borrow(&self) -> &u32 {
        &self.id
    }
}

impl TracedThread {
    fn attach(pid: u32) -> io::Result<TracedThread> {
        unsafe {
            let ret = ptrace(
                PTRACE_SEIZE,
                pid as pid_t,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            );
            if ret != 0 {
                let e = io::Error::last_os_error();
                // ptrace returns ESRCH if PTRACE_SEIZE isn't supported for some reason
                if e.raw_os_error() == Some(ESRCH as i32) {
                    return TracedThread::new_fallback(pid);
                }

                return Err(e);
            }

            let thread = TracedThread {
                id: pid,
                should_detach: true,
            };

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

    /// Creates `TracedThread` without attaching to process. Should not be used, if pid is not
    /// traced by current process
    fn traced(pid: u32) -> io::Result<TracedThread> {
        Ok(TracedThread {
            id: pid,
            should_detach: false,
        })
    }

    fn new_fallback(pid: u32) -> io::Result<TracedThread> {
        unsafe {
            let ret = ptrace(
                PTRACE_ATTACH,
                pid as pid_t,
                ptr::null_mut::<c_void>(),
                ptr::null_mut::<c_void>(),
            );
            if ret != 0 {
                return Err(io::Error::last_os_error());
            }

            let thread = TracedThread {
                id: pid,
                should_detach: true,
            };

            let mut status = 0;
            loop {
                let ret = waitpid(pid as pid_t, &mut status, __WALL);
                if ret < 0 {
                    let e = io::Error::last_os_error();
                    if e.kind() != io::ErrorKind::Interrupted {
                        return Err(e);
                    }

                    continue;
                }

                if !WIFSTOPPED(status) {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("unexpected wait status {}", status),
                    ));
                }

                let sig = WSTOPSIG(status);
                if sig == SIGSTOP {
                    return Ok(thread);
                }

                let ret = ptrace(
                    PTRACE_CONT,
                    pid as pid_t,
                    ptr::null_mut::<c_void>(),
                    sig as *const c_void,
                );
                if ret != 0 {
                    return Err(io::Error::last_os_error());
                }
            }
        }
    }

    fn info(&self, pid: u32, state: &mut imp::State, options: &TraceOptions) -> Thread {
        let name = if options.thread_names {
            self.name(pid)
        } else {
            None
        };

        let frames = self.dump(state, options);

        Thread {
            id: self.id,
            name,
            frames,
        }
    }

    fn dump(&self, state: &mut imp::State, options: &TraceOptions) -> Vec<Frame> {
        let mut frames = vec![];

        if let Err(e) = self.dump_inner(state, options, &mut frames) {
            debug!("error tracing thread {}: {}", self.id, e);
        }

        frames
    }

    fn name(&self, pid: u32) -> Option<String> {
        let path = format!("/proc/{}/task/{}/comm", pid, self.id);
        let mut name = vec![];
        match File::open(path).and_then(|mut f| f.read_to_end(&mut name)) {
            Ok(_) => Some(String::from_utf8_lossy(&name).trim().to_string()),
            Err(e) => {
                debug!("error getting name for thread {}: {}", self.id, e);
                None
            }
        }
    }
}
