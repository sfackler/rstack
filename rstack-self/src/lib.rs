//! Retrieve stack traces of all threads of the process.
//!
//! This is implemented using the [rstack] crate, which itself uses [libunwind]'s ability to
//! trace remote processes using ptrace. Because processes cannot ptrace themselves, we're forced to
//! use spawn a child process which does that work.
//!
//! [rstack]: https://sfackler.github.io/rstack/doc/rstack
//! [libunwind]: http://www.nongnu.org/libunwind/
//!
//! # Example
//!
//! ```
//! extern crate rstack_self;
//!
//! use std::env;
//! use std::process::Command;
//! use std::thread;
//!
//! fn main() {
//!     if env::args_os().count() > 1 {
//!         let _ = rstack_self::child();
//!         return;
//!     }
//!
//!     // spawn a second thread just for fun
//!     thread::spawn(background_thread);
//!
//!     let exe = env::current_exe().unwrap();
//!     let trace = rstack_self::trace(Command::new(exe).arg("child")).unwrap();
//!
//!     println!("{:#?}", trace);
//! }
//!
//! fn background_thread() {
//!     loop {
//!         thread::park();
//!     }
//! }
//! ```
#![doc(html_root_url = "https://sfackler.github.io/rstack/doc")]
#![warn(missing_docs)]
extern crate antidote;
extern crate backtrace;
extern crate bincode;
extern crate libc;
extern crate rstack;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate env_logger;

use antidote::Mutex;
use libc::{c_ulong, getppid, prctl, PR_SET_PTRACER};
use std::error;
use std::fmt;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::result;

lazy_static! {
    static ref TRACE_LOCK: Mutex<()> = Mutex::new(());
}

/// The result type returned by methods in this crate.
pub type Result<T> = result::Result<T, Error>;

/// The error type returned by methods in this crate.
#[derive(Debug)]
pub struct Error(Box<error::Error + Sync + Send>);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, fmt)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        error::Error::description(&*self.0)
    }

    fn cause(&self) -> Option<&error::Error> {
        error::Error::cause(&*self.0)
    }
}

/// Information about a thread.
#[derive(Debug, Clone)]
pub struct Thread {
    id: u32,
    name: String,
    frames: Vec<Frame>,
}

impl Thread {
    /// Returns the thread's ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the thread's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the stack frames of the thread.
    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
}

/// Information about a stack frame.
#[derive(Debug, Clone)]
pub struct Frame {
    ip: usize,
    symbols: Vec<Symbol>,
}

impl Frame {
    /// Returns the instruction pointer of the frame.
    pub fn ip(&self) -> usize {
        self.ip
    }

    /// Returns the symbols resolved to this frame.
    ///
    /// Multiple symbols can be returned due to inlining.
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }
}

/// Information about a symbol.
#[derive(Debug, Clone)]
pub struct Symbol {
    name: Option<String>,
    file: Option<PathBuf>,
    line: Option<u32>,
}

impl Symbol {
    /// Returns the name of the symbol, if known.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| &**n)
    }

    /// Returns the file in which this symbol is defined, if known.
    pub fn file(&self) -> Option<&Path> {
        self.file.as_ref().map(|f| &**f)
    }

    /// Returns the line at which the address which resolved to this symbol corresponds, if known.
    pub fn line(&self) -> Option<u32> {
        self.line
    }
}

/// Returns stack traces of all of the threads the calling process.
///
/// The provided `Command` should be configured to spawn a process which will call the [`child`]
/// function. It must not use standard input or standard output, but standard error will be
/// inherited and can be used. The spawned process must "directly" call `child` rather than
/// spawning another process to call it. That is, the parent of the process that calls `child` is
/// the one that will be traced.
///
/// [`child`]: fn.child.html
pub fn trace(child: &mut Command) -> Result<Vec<Thread>> {
    let raw = trace_raw(child)?;
    let threads = symbolicate(raw);
    Ok(threads)
}

fn trace_raw(child: &mut Command) -> Result<Vec<RawThread>> {
    let _guard = TRACE_LOCK.lock();

    let child = child
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| Error(e.into()))?;
    let mut child = ChildGuard(child);

    let mut stdin = child.0.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.0.stdout.take().unwrap());

    let _guard = PtracerGuard::new(child.0.id()).map_err(|e| Error(e.into()))?;

    stdin.write_all(&[0]).map_err(|e| Error(e.into()))?;

    let raw: result::Result<Vec<RawThread>, String> =
        bincode::deserialize_from(&mut stdout, bincode::Infinite).map_err(|e| Error(e.into()))?;
    let raw = raw.map_err(|e| Error(e.into()))?;

    Ok(raw)
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct PtracerGuard(bool);

impl Drop for PtracerGuard {
    fn drop(&mut self) {
        if self.0 {
            let _ = set_ptracer(0);
        }
    }
}

impl PtracerGuard {
    fn new(pid: u32) -> io::Result<PtracerGuard> {
        match set_ptracer(pid) {
            Ok(()) => Ok(PtracerGuard(true)),
            Err(ref e) if e.kind() == io::ErrorKind::InvalidInput => Ok(PtracerGuard(false)),
            Err(e) => Err(e),
        }
    }
}

fn set_ptracer(pid: u32) -> io::Result<()> {
    unsafe {
        let r = prctl(PR_SET_PTRACER, pid as c_ulong, 0, 0, 0);
        if r != 0 {
            Err(io::Error::last_os_error().into())
        } else {
            Ok(())
        }
    }
}

fn symbolicate(raw: Vec<RawThread>) -> Vec<Thread> {
    raw.into_iter().map(symbolicate_thread).collect()
}

fn symbolicate_thread(raw: RawThread) -> Thread {
    let mut thread = Thread {
        id: raw.id,
        name: raw.name,
        frames: vec![],
    };

    for raw_frame in raw.frames {
        let mut frame = Frame {
            ip: raw_frame.ip,
            symbols: vec![],
        };

        let current_ip = if raw_frame.is_signal || raw_frame.ip == 0 {
            raw_frame.ip
        } else {
            raw_frame.ip - 1
        };
        backtrace::resolve(current_ip as *mut _, |symbol| {
            frame.symbols.push(Symbol {
                name: symbol.name().map(|s| s.to_string()),
                file: symbol.filename().map(|p| p.to_owned()),
                line: symbol.lineno(),
            });
        });

        thread.frames.push(frame);
    }

    thread
}

#[derive(Serialize, Deserialize)]
struct RawThread {
    id: u32,
    name: String,
    frames: Vec<RawFrame>,
}

#[derive(Serialize, Deserialize)]
struct RawFrame {
    ip: usize,
    is_signal: bool,
}

/// The function called by process spawned by a call to [`trace`].
///
/// [`trace`]: fn.trace.html
pub fn child() -> Result<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // wait for the parent to tell us it's ready
    let mut buf = [0];
    stdin.read_exact(&mut buf).map_err(|e| Error(e.into()))?;

    let trace = child_trace();
    bincode::serialize_into(&mut stdout, &trace, bincode::Infinite).map_err(|e| Error(e.into()))?;
    stdout.flush().map_err(|e| Error(e.into()))?;

    // wait around for the parent to kill us, or die
    let _ = stdin.read_exact(&mut buf);
    Ok(())
}

fn child_trace() -> result::Result<Vec<RawThread>, String> {
    let parent = unsafe { getppid() } as u32;

    match rstack::TraceOptions::new().thread_names(true).trace(parent) {
        Ok(process) => Ok(process
            .threads()
            .iter()
            .map(|thread| RawThread {
                id: thread.id(),
                name: thread
                    .name()
                    .map_or_else(|| "<unknown>".to_string(), |s| s.to_string()),
                frames: thread
                    .frames()
                    .iter()
                    .map(|f| RawFrame {
                        ip: f.ip(),
                        is_signal: f.is_signal().unwrap_or(false),
                    })
                    .collect(),
            })
            .collect()),
        Err(e) => Err(e.to_string()),
    }
}
