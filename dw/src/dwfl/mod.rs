use libc::c_int;

pub use self::callbacks::*;
pub use self::dwfl::*;
pub use self::frame::*;
pub use self::thread::*;
pub use self::error::*;
pub use self::module::*;

mod callbacks;
mod dwfl;
mod frame;
mod thread;
mod error;
mod module;

fn cvt(r: c_int) -> Result<(), Error> {
    if r == 0 {
        Ok(())
    } else {
        Err(Error::new())
    }
}
