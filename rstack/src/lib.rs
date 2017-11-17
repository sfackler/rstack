extern crate libc;
extern crate unwind;

#[macro_use]
extern crate log;

#[cfg(test)]
extern crate env_logger;

use libc::{c_void, pid_t, ptrace, waitpid, EPERM, PTRACE_DETACH, PTRACE_INTERRUPT, PTRACE_SEIZE,
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

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
enum ErrorInner {
    Io(io::Error),
    Unwind(unwind::Error),
}

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

pub struct Thread {
    id: u32,
    name: Option<String>,
    trace: Vec<Frame>,
}

impl Thread {
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| &**s)
    }

    #[inline]
    pub fn trace(&self) -> &[Frame] {
        &self.trace
    }
}

pub struct Frame {
    ip: usize,
    is_signal: Option<bool>,
    name: Option<ProcedureName>,
    info: Option<ProcedureInfo>,
}

impl Frame {
    #[inline]
    pub fn ip(&self) -> usize {
        self.ip
    }

    #[inline]
    pub fn is_signal(&self) -> Option<bool> {
        self.is_signal
    }

    #[inline]
    pub fn name(&self) -> Option<&ProcedureName> {
        self.name.as_ref()
    }

    #[inline]
    pub fn info(&self) -> Option<&ProcedureInfo> {
        self.info.as_ref()
    }
}

pub struct ProcedureName {
    name: String,
    offset: usize,
}

impl ProcedureName {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub struct ProcedureInfo {
    start_ip: usize,
    end_ip: usize,
}

impl ProcedureInfo {
    #[inline]
    pub fn start_ip(&self) -> usize {
        self.start_ip
    }

    #[inline]
    pub fn end_ip(&self) -> usize {
        self.end_ip
    }
}

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
    pub fn new() -> TraceOptions {
        TraceOptions::default()
    }

    pub fn thread_names(&mut self, thread_names: bool) -> &mut TraceOptions {
        self.thread_names = thread_names;
        self
    }

    pub fn procedure_names(&mut self, procedure_names: bool) -> &mut TraceOptions {
        self.procedure_names = procedure_names;
        self
    }

    pub fn procedure_info(&mut self, procedure_info: bool) -> &mut TraceOptions {
        self.procedure_info = procedure_info;
        self
    }

    pub fn trace(&self, pid: u32) -> Result<Vec<Thread>> {
        let space = AddressSpace::new(&Accessors::ptrace(), Byteorder::DEFAULT)
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
                Ok(trace) => traces.push(Thread {
                    id: thread.0,
                    name,
                    trace,
                }),
                Err(e) => debug!("error tracing thread {}: {}", thread.0, e),
            }
        }

        Ok(traces)
    }
}

fn get_threads(pid: u32) -> Result<BTreeSet<TracedThread>> {
    let mut threads = BTreeSet::new();

    let path = format!("/proc/{}/task", pid);

    // new threads may be create while we're in the process of stopping them all, so loop a couple
    // of times to converge
    for _ in 0..4 {
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
                // some errors can legitimately happen since we're racing with the thread exiting
                // but we do want to report permission errors
                Err(e) => if e.raw_os_error() == Some(EPERM) {
                    return Err(Error(ErrorInner::Io(e)));
                } else {
                    debug!("error attaching to thread {}: {}", pid, e);
                    continue;
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

            let thread = TracedThread(pid);

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
                        name: n.name,
                        offset: n.offset as usize,
                    }
                })
            } else {
                None
            };

            let info = if options.procedure_info {
                cursor.procedure_info().ok().map(|i| {
                    ProcedureInfo {
                        start_ip: i.start_ip as usize,
                        end_ip: i.end_ip as usize,
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

#[cfg(test)]
mod test {
    use std::process::Command;

    use super::*;

    #[test]
    fn traced_thread() {
        let _ = env_logger::init();

        let space = AddressSpace::new(Accessors::ptrace(), Byteorder::DEFAULT).unwrap();

        let mut child = Command::new("sleep").arg("10").spawn().unwrap();

        let thread = TracedThread::new(child.id()).unwrap();
        let trace = thread.dump(&space).unwrap();
        drop(thread);

        for frame in &trace {
            println!(
                "{:#x} - {} + {:#x}",
                frame.ip(),
                frame.name().ok().map_or("<unknown>", |s| s.name()),
                frame.name().ok().map_or(0, |s| s.offset())
            );
        }

        child.kill().unwrap();
    }
}
