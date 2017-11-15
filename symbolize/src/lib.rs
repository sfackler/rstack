extern crate goblin;
extern crate libc;
extern crate memmap;
extern crate rustc_demangle;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate env_logger;

use goblin::elf::{Elf, Syms};
use goblin::elf::sym::{STB_GLOBAL, STB_LOCAL, STB_WEAK};
use goblin::strtab::Strtab;
use libc::{c_int, c_void, dl_iterate_phdr, dl_phdr_info, size_t, PT_LOAD};
use memmap::Mmap;
use std::any::Any;
use std::collections::hash_map::{Entry, HashMap};
use std::cmp::Ordering;
use std::ffi::{CStr, OsStr};
use std::fs::File;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::slice;

lazy_static! {
    static ref STATE: State = load_state();
}

struct Dylib {
    name: Option<PathBuf>,
    offset: usize,
    symbols: Vec<Symbol>,
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

pub struct FrameInfo {
    pub library: Option<&'static Path>,
    pub symbol: Option<&'static str>,
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
        if s.start > shifted {
            Ordering::Greater
        } else if s.end <= shifted {
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
        symbol,
    })
}

struct CallbackState {
    state: State,
    panic: Option<Box<Any + Send>>,
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

unsafe extern "C" fn callback(info: *mut dl_phdr_info, _: size_t, state: *mut c_void) -> c_int {
    let state = &mut *(state as *mut CallbackState);

    match panic::catch_unwind(AssertUnwindSafe(|| load_lib(&mut state.state, &*info))) {
        Ok(()) => 0,
        Err(e) => {
            state.panic = Some(e);
            1
        }
    }
}

unsafe fn load_lib(state: &mut State, info: &dl_phdr_info) {
    let dylib = match load_dylib(info) {
        Some(dylib) => dylib,
        None => return,
    };

    let offset = dylib.offset;
    state.dylibs.push(dylib);
    let dylib = state.dylibs.len() - 1;

    let segments = slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum as usize);
    for segment in segments {
        if segment.p_type != PT_LOAD {
            continue;
        }

        let start = offset + segment.p_vaddr as usize;
        let end = start + segment.p_memsz as usize;
        state.segments.push(Segment { dylib, start, end })
    }
}

unsafe fn load_dylib(info: &dl_phdr_info) -> Option<Dylib> {
    let name = CStr::from_ptr(info.dlpi_name);
    let name = OsStr::from_bytes(name.to_bytes());
    let name = if name.is_empty() {
        None
    } else {
        Some(PathBuf::from(name))
    };

    let elf = {
        let path = match name {
            Some(ref name) => &**name,
            None => Path::new("/proc/self/exe"),
        };

        let map = match File::open(path).and_then(|f| Mmap::map(&f)) {
            Ok(map) => map,
            Err(e) => {
                debug!("error mapping file {}: {}", path.display(), e);
                return None;
            }
        };

        let static_slice = &*(&*map as *const [u8]);
        mem::forget(map);

        match Elf::parse(static_slice) {
            Ok(elf) => elf,
            Err(e) => {
                debug!("error parsing file {}: {}", path.display(), e);
                return None;
            }
        }
    };

    let offset = info.dlpi_addr as usize;

    let symbols = load_symbols(&elf);

    Some(Dylib {
        name,
        offset,
        symbols,
    })
}

fn load_symbols(elf: &Elf) -> Vec<Symbol> {
    // .symtab is more detailed, so use it if available
    if !elf.syms.is_empty() {
        load_symbols_inner(&elf.syms, &elf.strtab)
    } else {
        load_symbols_inner(&elf.dynsyms, &elf.dynstrtab)
    }
}

#[derive(PartialOrd, PartialEq, Ord, Eq)]
enum Bind {
    Other,
    Local,
    Global,
    Weak,
}

struct TempSymbol<'a> {
    name: &'a str,
    start: usize,
    end: usize,
    bind: Bind,
}

impl<'a> PartialOrd for TempSymbol<'a> {
    fn partial_cmp(&self, other: &TempSymbol<'a>) -> Option<Ordering> {
        Some(
            self.bind
                .cmp(&other.bind)
                .then_with(|| other.name.len().cmp(&self.name.len())),
        )
    }
}

impl<'a> PartialEq for TempSymbol<'a> {
    fn eq(&self, other: &TempSymbol<'a>) -> bool {
        self.bind == other.bind && self.name.len() == other.name.len()
    }
}

fn load_symbols_inner(syms: &Syms, strs: &Strtab) -> Vec<Symbol> {
    let mut map = HashMap::with_capacity(syms.len());

    // Some things (in particular glibc) will have multiple symbols corresponding to the same
    // address. For example, libc's close is called all of `__GI__close`, `__libc__close`,
    // `__close`, and `close`. The first two are locally bound, the third is globally bound, and
    // the last is weakly bound.
    //
    // Following that, we prefer weak to global binding and global to local when picking a name.
    // Ties are broken by taking the shorter name. That will hopefully leave us with the least weird
    // choice.
    for sym in syms {
        if !sym.is_function() || sym.st_value == 0 {
            continue;
        }

        let name = match strs.get(sym.st_name) {
            Some(Ok(name)) => name,
            _ => continue,
        };

        let start = sym.st_value as usize;
        let end = start + sym.st_size as usize;

        let bind = match sym.st_bind() {
            STB_WEAK => Bind::Weak,
            STB_GLOBAL => Bind::Global,
            STB_LOCAL => Bind::Local,
            _ => Bind::Other,
        };

        let temp = TempSymbol {
            name,
            start,
            end,
            bind,
        };

        match map.entry(start) {
            Entry::Vacant(e) => {
                e.insert(temp);
            }
            Entry::Occupied(mut e) => if temp > *e.get() {
                e.insert(temp);
            },
        }
    }

    let mut symbols = map.values()
        .map(|s| {
            Symbol {
                name: rustc_demangle::demangle(s.name).to_string(),
                start: s.start,
                end: s.end,
            }
        })
        .collect::<Vec<_>>();
    symbols.sort_by_key(|s| s.start);

    symbols
}

#[cfg(test)]
mod test {
    use libc::close;

    use super::*;

    #[test]
    fn simple() {
        let info = query(query as usize).unwrap();
        assert!(info.library.is_none());
        assert!(info.symbol.unwrap().contains("symbolize::query"));

        let info = query(close as usize).unwrap();
        assert_eq!(info.symbol.unwrap(), "close");
    }
}
