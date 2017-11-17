extern crate addr2line;
extern crate antidote;
extern crate bincode;
extern crate cpp_demangle;
extern crate fallible_iterator;
extern crate gimli;
extern crate libc;
extern crate memmap;
extern crate object;
extern crate rstack;
extern crate rustc_demangle;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate env_logger;

use antidote::Mutex;
use fallible_iterator::FallibleIterator;
use libc::{c_ulong, getppid, prctl, PR_SET_PTRACER};
use std::process::{Command, Stdio};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io::{self, BufReader, Read, Write};

mod dylibs;

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
    pub library: Option<&'static Path>,
    pub symbols: Vec<Symbol>,
}

pub struct Symbol {
    pub name: Option<String>,
    pub file: Option<PathBuf>,
    pub line: Option<u64>,
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
            library: None,
            symbols: vec![],
        };

        let current_ip = if raw_frame.is_signal {
            raw_frame.ip
        } else {
            raw_frame.ip - 1
        };
        if let Some(info) = dylibs::query(current_ip) {
            frame.library = info.library;

            let symbols = info.frames.and_then(|it| {
                it.map(|raw_symbol| {
                    let mut symbol = Symbol {
                        name: None,
                        file: None,
                        line: None,
                    };
                    if let Some(function) = raw_symbol.function {
                        symbol.name = Some(function.to_string());
                    }
                    if let Some(location) = raw_symbol.location {
                        symbol.file = location.file;
                        symbol.line = location.line;
                    }

                    symbol
                }).collect()
            });

            match symbols {
                Ok(symbols) => frame.symbols = symbols,
                Err(e) => {
                    debug!("error querying debug info for {:#016}: {}", current_ip, e);
                }
            }

            if frame.symbols.is_empty() {
                if let Some(name) = info.symbol {
                    frame.symbols.push(Symbol {
                        name: Some(name.to_string()),
                        file: None,
                        line: None,
                    });
                }
            }
        }

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

pub fn child() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut buf = [0];
    stdin.read_exact(&mut buf).expect("first wait");
    let trace = child_trace();
    bincode::serialize_into(&mut stdout, &trace, bincode::Infinite).expect("serialize");
    stdout.flush().expect("flush");
    stdin.read_exact(&mut buf).expect("second wait");
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
                            .trace()
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
