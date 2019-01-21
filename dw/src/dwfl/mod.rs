use libc::c_int;

pub use callbacks::*;
pub use dwfl::*;
pub use dwfl_frame::*;
pub use dwfl_thread::*;
pub use error::*;
pub use module::*;

mod callbacks;
mod dwfl;
mod dwfl_frame;
mod dwfl_thread;
mod error;
mod module;

fn cvt(r: c_int) -> Result<(), Error> {
    if r == 0 {
        Ok(())
    } else {
        Err(Error::new())
    }
}
