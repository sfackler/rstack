pub use dw_::dwfl::Error;
use dw_::dwfl::{Callbacks, Dwfl, FindDebuginfo, FindElf};
use lazy_static::lazy_static;

use crate::{Frame, Symbol, TraceOptions, TracedThread};

lazy_static! {
    static ref CALLBACKS: Callbacks = Callbacks::new(FindElf::LINUX_PROC, FindDebuginfo::STANDARD);
}

pub struct State(Dwfl<'static>);

impl State {
    pub fn new(pid: u32) -> Result<State, Error> {
        let mut dwfl = Dwfl::begin(&*CALLBACKS)?;
        dwfl.report().linux_proc(pid)?;
        dwfl.linux_proc_attach(pid, true)?;
        Ok(State(dwfl))
    }
}

impl TracedThread {
    pub fn dump_inner(
        &self,
        dwfl: &mut State,
        options: &TraceOptions,
        frames: &mut Vec<Frame>,
    ) -> Result<(), Error> {
        dwfl.0.thread_frames(self.0, |frame| {
            let mut is_signal = false;
            let ip = frame.pc(Some(&mut is_signal))?;

            let mut symbol = None;
            if options.symbols {
                let signal_adjust = if is_signal { 0 } else { 1 };

                if let Ok(i) = frame
                    .thread()
                    .dwfl()
                    .addr_module(ip - signal_adjust)
                    .and_then(|module| module.addr_info(ip - signal_adjust))
                {
                    symbol = Some(Symbol {
                        name: i.name().to_string_lossy().into_owned(),
                        offset: i.offset() + signal_adjust,
                        address: i.bias() + i.symbol().value(),
                        size: i.symbol().size(),
                    });
                }
            }

            frames.push(Frame {
                ip,
                is_signal,
                symbol,
            });

            Ok(())
        })
    }
}
