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
use std::process::{Command, Stdio};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io::{self, BufReader, Read, Write};

lazy_static! {
    static ref TRACE_LOCK: Mutex<()> = Mutex::new(());
}

pub struct Thread {
    id: u32,
    name: String,
    frames: Vec<Frame>,
}

impl Thread {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }
}

pub struct Frame {
    ip: usize,
    symbols: Vec<Symbol>,
}

impl Frame {
    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }
}

pub struct Symbol {
    name: Option<String>,
    file: Option<PathBuf>,
    line: Option<u32>,
}

impl Symbol {
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| &**n)
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_ref().map(|f| &**f)
    }

    pub fn line(&self) -> Option<u32> {
        self.line
    }
}

pub fn trace(child: &mut Command) -> Result<Vec<Thread>, Box<Error + Sync + Send>> {
    let _guard = TRACE_LOCK.lock();

    let mut child = child
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    set_ptracer(child.id())?;
    let mut bomb = PtracerBomb(true);

    stdin.write_all(&[0])?;

    let raw: Result<Vec<RawThread>, String> =
        bincode::deserialize_from(&mut stdout, bincode::Infinite)?;
    let raw = raw?;

    set_ptracer(0)?;
    bomb.0 = false;

    stdin.write_all(&[0])?;

    let threads = symbolicate(raw);

    Ok(threads)
}

struct PtracerBomb(bool);

impl Drop for PtracerBomb {
    fn drop(&mut self) {
        if self.0 {
            let _ = set_ptracer(0);
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

        let current_ip = if raw_frame.is_signal {
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

pub fn child() -> Result<(), Box<Error + Sync + Send>> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut buf = [0];
    stdin.read_exact(&mut buf)?;
    let trace = child_trace();
    bincode::serialize_into(&mut stdout, &trace, bincode::Infinite)?;
    stdout.flush()?;
    stdin.read_exact(&mut buf)?;

    Ok(())
}

fn child_trace() -> Result<Vec<RawThread>, String> {
    let parent = unsafe { getppid() } as u32;

    match rstack::TraceOptions::new().thread_names(true).trace(parent) {
        Ok(process) => Ok(
            process
                .threads()
                .iter()
                .map(|thread| {
                    RawThread {
                        id: thread.id(),
                        name: thread
                            .name()
                            .map_or_else(|| "<unknown>".to_string(), |s| s.to_string()),
                        frames: thread
                            .frames()
                            .iter()
                            .map(|f| {
                                RawFrame {
                                    ip: f.ip(),
                                    is_signal: f.is_signal().unwrap_or(false),
                                }
                            })
                            .collect(),
                    }
                })
                .collect(),
        ),
        Err(e) => Err(e.to_string()),
    }
}
