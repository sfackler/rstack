pub use dw_::dwfl::Error;
use dw_::dwfl::{Callbacks, Dwfl, FindDebuginfo, FindElf};
use lazy_static::lazy_static;

use crate::{Frame, ProcedureInfo, ProcedureName, TraceOptions, TracedThread};

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

            let mut name = None;
            let mut info = None;
            if options.procedure_names || options.procedure_info {
                let signal_adjust = if is_signal { 0 } else { 1 };

                if let Ok(i) = frame
                    .thread()
                    .dwfl()
                    .addr_module(ip - signal_adjust)
                    .and_then(|module| module.addr_info(ip - signal_adjust))
                {
                    if options.procedure_names {
                        name = Some(ProcedureName {
                            name: i.name().to_string_lossy().into_owned(),
                            offset: i.offset() + signal_adjust,
                        });
                    }
                    if options.procedure_info {
                        let start_ip = i.bias() + i.symbol().value();
                        info = Some(ProcedureInfo {
                            start_ip,
                            end_ip: start_ip + i.symbol().size(),
                        });
                    }
                }
            }

            frames.push(Frame {
                ip,
                is_signal: Some(is_signal),
                name,
                info,
            });

            Ok(())
        })
    }
}
