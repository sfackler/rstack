//! DWFL types.

use libc::c_int;

pub use self::callbacks::*;
pub use self::dwfl::*;
pub use self::error::*;
pub use self::frame::*;
pub use self::module::*;
pub use self::thread::*;
pub use self::thread_callbacks::*;

mod callbacks;
mod dwfl;
mod error;
mod frame;
mod module;
mod thread;
mod thread_callbacks;

fn cvt(r: c_int) -> Result<(), Error> {
    if r == 0 {
        Ok(())
    } else {
        Err(Error::new())
    }
}
