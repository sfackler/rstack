extern crate addr2line;
extern crate bincode;
extern crate fallible_iterator;
extern crate gimli;
extern crate libc;
extern crate memmap;
extern crate object;
extern crate rstack;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
extern crate unwind;

use fallible_iterator::FallibleIterator;
use libc::{c_ulong, getppid, prctl, PR_SET_PTRACER};
use std::process::{Command, Stdio};
use std::error::Error;
use std::path::PathBuf;
use std::io::{self, Read, Write};

mod dylibs;

pub struct Thread {
    pub id: u32,
    pub name: String,
    pub frames: Vec<Frame>,
}

pub struct Frame {
    pub ip: usize,
    pub name: Option<String>,
    pub file: Option<PathBuf>,
    pub line: Option<u64>,
}

pub fn trace_threads(child: &mut Command) -> Result<Vec<Thread>, Box<Error + Sync + Send>> {
    let mut child = child
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    set_ptracer(child.id())?;
    let mut bomb = PtracerBomb(true);

    stdin.write_all(&[0])?;
    stdin.flush()?;

    let raw: Result<Vec<RawThread>, String> =
        bincode::deserialize_from(&mut stdout, bincode::Infinite)?;
    let raw = raw?;

    set_ptracer(0)?;
    bomb.0 = false;

    stdin.write_all(&[0])?;
    stdin.flush()?;

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

    for ip in raw.frames {
        let mut it = dylibs::query(ip)?;
        let mut any = false;
        while let Some(frame) = it.next()? {
            any = true;
            let mut full = Frame {
                ip,
                name: None,
                file: None,
                line: None,
            };
            if let Some(function) = frame.function {
                full.name = Some(function.to_string());
            }
            if let Some(location) = frame.location {
                full.file = location.file;
                full.line = location.line;
            }

            thread.frames.push(full);
        }

        if !any {
            thread.frames.push(Frame {
                ip,
                name: None,
                file: None,
                line: None,
            });
        }
    }

    Ok(thread)
}

#[derive(Serialize, Deserialize)]
struct RawThread {
    id: u32,
    name: String,
    frames: Vec<usize>,
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

    match rstack::trace_threads(parent) {
        Ok(threads) => Ok(
            threads
                .into_iter()
                .map(|thread| {
                    RawThread {
                        id: thread.id(),
                        name: thread.name().to_string(),
                        frames: thread.trace().iter().map(|f| f.ip()).collect(),
                    }
                })
                .collect(),
        ),
        Err(e) => Err(e.to_string()),
    }
}
