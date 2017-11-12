use addr2line::{Context, FullContext, IterFrames};
use gimli::{self, EndianBuf, RunTimeEndian};
use libc::{c_int, c_void, dl_iterate_phdr, dl_phdr_info, size_t, PT_LOAD};
use memmap::Mmap;
use object;
use std::any::Any;
use std::cmp::Ordering;
use std::env;
use std::ffi::{CStr, OsStr};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::slice;

lazy_static! {
    static ref STATE: State = load_state();
}

pub fn query(
    addr: usize,
) -> Result<IterFrames<'static, EndianBuf<'static, RunTimeEndian>>, gimli::Error> {
    let state = &*STATE;

    let idx = match state.segments.binary_search_by(|s| {
        if s.start > addr {
            Ordering::Greater
        } else if s.end <= addr {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }) {
        Ok(idx) => idx,
        Err(_) => {
            warn!(
                "unable to find address {:#016x} in known dynamic segments",
                addr
            );
            0
        }
    };

    let idx = state.segments[idx].dylib;
    let dylib = &state.dylibs[idx];
    let shifted = addr - dylib.offset;
    dylib.context.query(shifted as u64)
}

struct CallbackState {
    state: State,
    panic: Option<Box<Any + Send>>,
}

unsafe extern "C" fn callback(info: *mut dl_phdr_info, _: size_t, state: *mut c_void) -> c_int {
    let state = &mut *(state as *mut CallbackState);

    match panic::catch_unwind(AssertUnwindSafe(|| state.state.add(&*info))) {
        Ok(()) => 0,
        Err(e) => {
            state.panic = Some(e);
            1
        }
    }
}

fn load_state() -> State {
    let mut state = CallbackState {
        state: State {
            dylibs: vec![],
            segments: vec![],
        },
        panic: None,
    };
    unsafe {
        dl_iterate_phdr(Some(callback), &mut state as *mut _ as *mut _);
    }
    if let Some(e) = state.panic {
        panic::resume_unwind(e);
    }

    state.state.segments.sort_by_key(|s| s.start);
    state.state
}

struct Dylib {
    name: Option<PathBuf>,
    map: Mmap,
    object: Box<object::File<'static>>,
    context: FullContext<EndianBuf<'static, RunTimeEndian>>,
    offset: usize,
}

struct Segment {
    dylib: usize,
    start: usize,
    end: usize,
}

struct State {
    dylibs: Vec<Dylib>,
    segments: Vec<Segment>,
}

impl State {
    unsafe fn add(&mut self, info: &dl_phdr_info) {
        let name = CStr::from_ptr(info.dlpi_name);
        let name = OsStr::from_bytes(name.to_bytes());
        let name = if name.is_empty() {
            None
        } else {
            Some(PathBuf::from(name))
        };

        let file = match name {
            Some(ref name) => File::open(name),
            None => env::current_exe().and_then(|p| File::open(&p)),
        };

        let map = match file.and_then(|f| Mmap::map(&f)) {
            Ok(map) => map,
            Err(e) => {
                warn!("error mapping file {:?}: {}", name, e);
                return;
            }
        };

        let static_slice = &*(&*map as *const [u8]);

        let object = match object::File::parse(&static_slice) {
            Ok(object) => Box::new(object),
            Err(e) => {
                warn!("error parsing object file {:?}: {}", name, e);
                return;
            }
        };

        let static_object = &*(&*object as *const _);

        let context = match Context::new(static_object).and_then(|c| c.parse_functions()) {
            Ok(context) => context,
            Err(e) => {
                warn!("error loading debug info for {:?}: {}", name, e);
                return;
            }
        };

        let offset = info.dlpi_addr as usize;

        let dylib = self.dylibs.len();
        self.dylibs.push(Dylib {
            name,
            map,
            object,
            context,
            offset,
        });

        let segments = slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum as usize);
        for segment in segments {
            if segment.p_type != PT_LOAD {
                continue;
            }

            let start = offset + segment.p_vaddr as usize;
            let end = start + segment.p_memsz as usize;
            self.segments.push(Segment { dylib, start, end });
        }
    }
}

#[cfg(test)]
mod test {
    use env_logger;
    use fallible_iterator::FallibleIterator;
    use unwind::{Cursor, RegNum};

    use super::*;

    #[test]
    fn load() {
        let state = &*STATE;

        for (i, dylib) in state.dylibs.iter().enumerate() {
            println!("{} - {:#016x}: {:?}", i, dylib.offset, dylib.name);
        }

        println!();

        for segment in &state.segments {
            println!(
                "{:#016x}-{:#016x}: {}",
                segment.start,
                segment.end,
                segment.dylib
            );
        }
    }

    #[test]
    fn local() {
        let _ = env_logger::init();

        Cursor::local(|mut cursor| {
            loop {
                let ip = cursor.register(RegNum::IP)?;
                println!("{:#016x} - {}", ip, cursor.procedure_name()?.name);

                let mut it = query(ip as usize).unwrap();
                while let Some(frame) = it.next().unwrap() {
                    println!(
                        "{}, {:?}, {:?}",
                        frame.function.map_or("??".to_string(), |f| f.to_string()),
                        frame.location.as_ref().map(|l| &l.file),
                        frame.location.as_ref().map(|l| &l.line)
                    );
                }

                if !cursor.step()? {
                    break;
                }
            }

            Ok(())
        }).unwrap();
    }
}
