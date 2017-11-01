extern crate libc;
extern crate unwind;

#[macro_use]
extern crate log;

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
    name: String,
    trace: Vec<Frame>,
}

impl Thread {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn trace(&self) -> &[Frame] {
        &self.trace
    }
}

pub struct Frame {
    ip: usize,
    name: Option<ProcedureName>,
    info: Option<ProcedureInfo>,
}

impl Frame {
    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn name(&self) -> Option<&ProcedureName> {
        self.name.as_ref()
    }

    pub fn info(&self) -> Option<&ProcedureInfo> {
        self.info.as_ref()
    }
}

pub struct ProcedureName {
    name: String,
    offset: usize,
}

impl ProcedureName {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub struct ProcedureInfo {
    start_ip: usize,
    end_ip: usize,
}

impl ProcedureInfo {
    pub fn start_ip(&self) -> usize {
        self.start_ip
    }

    pub fn end_ip(&self) -> usize {
        self.end_ip
    }
}

pub fn trace_threads(pid: u32) -> Result<Vec<Thread>> {
    let space = AddressSpace::new(&Accessors::ptrace(), Byteorder::DEFAULT)
        .map_err(|e| Error(ErrorInner::Unwind(e)))?;
    let threads = get_threads(pid)?;

    let mut traces = vec![];

    for thread in &threads {
        let path = format!("/proc/{}/task/{}/comm", pid, thread.0);
        let mut name = vec![];
        if let Err(e) = File::open(path).and_then(|mut r| r.read_to_end(&mut name)) {
            debug!("error getting name for thread {}: {}", thread.0, e);
            continue;
        }

        match thread.dump(&space) {
            Ok(trace) => traces.push(Thread {
                id: thread.0,
                name: String::from_utf8_lossy(&name).trim().to_string(),
                trace,
            }),
            Err(e) => debug!("error tracing thread {}: {}", thread.0, e),
        }
    }

    Ok(traces)
}

fn get_threads(pid: u32) -> Result<BTreeSet<TracedThread>> {
    let mut threads = BTreeSet::new();

    let path = format!("/proc/{}/task", pid);

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

    fn dump(&self, space: &AddressSpace<PTraceStateRef>) -> unwind::Result<Vec<Frame>> {
        let state = PTraceState::new(self.0)?;
        let mut cursor = Cursor::remote(&space, &state)?;

        let mut trace = vec![];
        loop {
            let mut frame = Frame {
                ip: cursor.register(RegNum::IP)? as usize,
                name: None,
                info: None,
            };

            let mut buf = vec![0; 256];
            loop {
                let mut offset = 0;
                match cursor.procedure_name(&mut buf, &mut offset) {
                    Ok(()) => {
                        let end = buf.iter().position(|b| *b == 0).unwrap();
                        buf.truncate(end);
                        frame.name = Some(ProcedureName {
                            name: String::from_utf8(buf).unwrap(),
                            offset: offset as usize,
                        });
                        break;
                    }
                    Err(unwind::Error::NOMEM) => {
                        let len = buf.len() * 2;
                        buf.resize(len, 0);
                    }
                    Err(e) => debug!(
                        "error retreiving procedure name for thread {}: {}",
                        self.0,
                        e
                    ),
                }
            }

            match cursor.procedure_info() {
                Ok(info) => {
                    frame.info = Some(ProcedureInfo {
                        start_ip: info.start_ip as usize,
                        end_ip: info.end_ip as usize,
                    });
                }
                Err(e) => debug!(
                    "error retreiving procedure info for thread {}: {}",
                    self.0,
                    e
                ),
            }

            trace.push(frame);

            match cursor.step() {
                Ok(true) => {}
                Ok(false) => break,
                Err(e) => {
                    debug!("error stepping frame for thread {}: {}", self.0, e);
                    break;
                }
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
        let space = AddressSpace::new(Accessors::ptrace(), Byteorder::DEFAULT).unwrap();

        let mut child = Command::new("sleep").arg("10").spawn().unwrap();

        let thread = TracedThread::new(child.id()).unwrap();
        let trace = thread.dump(&space).unwrap();
        drop(thread);

        for frame in &trace.trace {
            println!(
                "{:#x} - {} + {:#x}",
                frame.ip().unwrap_or(0),
                frame.name().unwrap_or("<unknown>"),
                frame.offset().unwrap_or(0)
            );
        }

        child.kill().unwrap();
    }
}
