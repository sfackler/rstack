pub use unwind_::Error;
use unwind_::{Accessors, AddressSpace, Byteorder, Cursor, PTraceState, PTraceStateRef, RegNum};

use crate::{Frame, ProcedureInfo, ProcedureName, TraceOptions, TracedThread};

pub struct State(AddressSpace<PTraceStateRef>);

impl State {
    pub fn new(_: u32) -> Result<State, Error> {
        AddressSpace::new(Accessors::ptrace(), Byteorder::DEFAULT).map(State)
    }
}

impl TracedThread {
    pub fn dump_inner(
        &self,
        space: &mut State,
        options: &TraceOptions,
        frames: &mut Vec<Frame>,
    ) -> Result<(), Error> {
        let state = PTraceState::new(self.0)?;
        let mut cursor = Cursor::remote(&space.0, &state)?;

        loop {
            let ip = cursor.register(RegNum::IP)?;
            let is_signal = cursor.is_signal_frame().ok();

            let name = if options.procedure_names {
                cursor.procedure_name().ok().map(|n| ProcedureName {
                    name: n.name().to_string(),
                    offset: n.offset(),
                })
            } else {
                None
            };

            let info = if options.procedure_info {
                cursor.procedure_info().ok().map(|i| ProcedureInfo {
                    start_ip: i.start_ip(),
                    end_ip: i.end_ip(),
                })
            } else {
                None
            };

            frames.push(Frame {
                ip,
                is_signal,
                name,
                info,
            });

            if !cursor.step()? {
                break;
            }
        }

        Ok(())
    }
}
