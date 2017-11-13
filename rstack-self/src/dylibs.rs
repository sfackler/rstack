use addr2line::{Context, FullContext, IterFrames};
use cpp_demangle;
use gimli::{self, EndianBuf, RunTimeEndian};
use libc::{c_int, c_void, dl_iterate_phdr, dl_phdr_info, size_t, PT_LOAD};
use memmap::Mmap;
use object::{self, SymbolKind};
use rustc_demangle;
use std::any::Any;
use std::cmp::Ordering;
use std::env;
use std::ffi::{CStr, OsStr};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::slice;

lazy_static! {
    static ref STATE: State = load_state();
}

pub struct FrameInfo {
    pub library: Option<&'static Path>,
    pub symbol: Option<&'static str>,
    pub frames: Result<IterFrames<'static, EndianBuf<'static, RunTimeEndian>>, gimli::Error>,
}

pub fn query(addr: usize) -> Option<FrameInfo> {
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
        Err(_) => return None,
    };

    let idx = state.segments[idx].dylib;
    let dylib = &state.dylibs[idx];
    let shifted = addr - dylib.offset;

    let symbol = match dylib.symbols.binary_search_by(|s| {
        if s.start > addr {
            Ordering::Greater
        } else if s.end <= addr {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }) {
        Ok(idx) => Some(&*dylib.symbols[idx].name),
        Err(_) => None,
    };

    Some(FrameInfo {
        library: dylib.name.as_ref().map(|p| &**p),
        frames: dylib.context.query(shifted as u64),
        symbol,
    })
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
    _map: Mmap,
    object: Box<object::File<'static>>,
    context: FullContext<EndianBuf<'static, RunTimeEndian>>,
    symbols: Vec<Symbol>,
    offset: usize,
}

struct Symbol {
    name: String,
    start: usize,
    end: usize,
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
                debug!("error mapping file {:?}: {}", name, e);
                return;
            }
        };

        let static_slice = &*(&*map as *const [u8]);

        let object = match object::File::parse(&static_slice) {
            Ok(object) => Box::new(object),
            Err(e) => {
                debug!("error parsing object file {:?}: {}", name, e);
                return;
            }
        };

        let static_object = &*(&*object as *const _);

        let context = match Context::new(static_object).and_then(|c| c.parse_functions()) {
            Ok(context) => context,
            Err(e) => {
                debug!("error loading debug info for {:?}: {}", name, e);
                return;
            }
        };

        let offset = info.dlpi_addr as usize;

        let mut symbols = vec![];
        for symbol in object.get_symbols() {
            if symbol.kind() != SymbolKind::Text || symbol.address() == 0 || symbol.size() == 0 {
                continue;
            }

            let name = match cpp_demangle::Symbol::new(symbol.name()) {
                Ok(name) => name.to_string(),
                Err(_) => String::from_utf8_lossy(symbol.name()).into_owned(),
            };
            let name = rustc_demangle::demangle(&name).to_string();
            let start = offset + symbol.address() as usize;
            let end = start + symbol.size() as usize;

            symbols.push(Symbol { name, start, end });
        }
        symbols.sort_by_key(|s| s.start);

        let dylib = self.dylibs.len();
        self.dylibs.push(Dylib {
            name,
            _map: map,
            object,
            context,
            offset,
            symbols,
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

    use super::*;

    #[test]
    fn load() {
        let _ = env_logger::init();

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
}
