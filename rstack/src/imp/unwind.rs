pub use unwind_::Error;
use unwind_::{Accessors, AddressSpace, Byteorder, Cursor, PTraceState, PTraceStateRef, RegNum};

use crate::{Frame, Symbol, TraceOptions, TracedThread};

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
            let is_signal = cursor.is_signal_frame()?;

            let mut symbol = None;
            if options.symbols {
                match (cursor.procedure_name(), cursor.procedure_info()) {
                    (Ok(ref name), Ok(ref info)) if info.start_ip() + name.offset() == ip => {
                        symbol = Some(Symbol {
                            name: name.name().to_string(),
                            offset: name.offset(),
                            address: info.start_ip(),
                            size: info.end_ip() - info.start_ip(),
                        });
                    }
                    _ => {}
                }
            }

            frames.push(Frame {
                ip,
                is_signal,
                symbol,
            });

            if !cursor.step()? {
                break;
            }
        }

        Ok(())
    }
}
