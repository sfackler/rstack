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
use std::path::PathBuf;
use std::io::{self, BufReader, Read, Write};

lazy_static! {
    static ref TRACE_LOCK: Mutex<()> = Mutex::new(());
}

pub struct Thread {
    pub id: u32,
    pub name: String,
    pub frames: Vec<Frame>,
}

pub struct Frame {
    pub ip: usize,
    pub symbols: Vec<Symbol>,
}

pub struct Symbol {
    pub name: Option<String>,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
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

    symbolicate(raw)
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

fn symbolicate(raw: Vec<RawThread>) -> Result<Vec<Thread>, Box<Error + Sync + Send>> {
    raw.into_iter().map(symbolicate_thread).collect()
}

fn symbolicate_thread(raw: RawThread) -> Result<Thread, Box<Error + Sync + Send>> {
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

    Ok(thread)
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
        Ok(threads) => Ok(
            threads
                .into_iter()
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
