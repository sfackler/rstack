//! Safe bindings to elfutils's libdw.
#![doc(html_root_url = "https://sfackler.github.io/rstack/doc")]
#![warn(missing_docs)]

pub mod dwfl;
pub mod elf;

#[cfg(test)]
mod test;
