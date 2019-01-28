#![doc(html_root_url = "https://sfackler.github.io/rstack/doc")]
#![allow(bad_style)]

pub use crate::dwarf::*;
pub use crate::elfutils::*;
pub use crate::libelf::*;

macro_rules! c_enum {
    ($name:ident { $($variant:ident = $value:expr,)*}) => {
        pub type $name = libc::c_uint;

        $(
            pub const $variant: $name = $value;
        )*
    }
}

mod dwarf;
mod elfutils;
mod libelf;
