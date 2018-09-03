use libc::{
    Elf64_Addr, Elf64_Half, Elf64_Off, Elf64_Phdr, Elf64_Shdr, Elf64_Sxword, Elf64_Sym, Elf64_Word,
    Elf64_Xword,
};

pub type GElf_Word = Elf64_Word;
pub type GElf_Addr = Elf64_Addr;
pub type GElf_Phdr = Elf64_Phdr;
pub type GElf_Shdr = Elf64_Shdr;
pub type GElf_Sxword = Elf64_Sxword;
pub type GElf_Sym = Elf64_Sym;
pub type GElf_Off = Elf64_Off;
pub type GElf_Xword = Elf64_Xword;
pub type GElf_Half = Elf64_Half;
