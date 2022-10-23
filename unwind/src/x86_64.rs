use crate::RegNum;
use unwind_sys::*;

impl RegNum {
    /// An x86_64-specific identifier for the RAX register.
    pub const RAX: RegNum = RegNum(UNW_X86_64_RAX);
    /// An x86_64-specific identifier for the RDX register.
    pub const RDX: RegNum = RegNum(UNW_X86_64_RDX);
    /// An x86_64-specific identifier for the RCX register.
    pub const RCX: RegNum = RegNum(UNW_X86_64_RCX);
    /// An x86_64-specific identifier for the RBX register.
    pub const RBX: RegNum = RegNum(UNW_X86_64_RBX);
    /// An x86_64-specific identifier for the RSI register.
    pub const RSI: RegNum = RegNum(UNW_X86_64_RSI);
    /// An x86_64-specific identifier for the RDI register.
    pub const RDI: RegNum = RegNum(UNW_X86_64_RDI);
    /// An x86_64-specific identifier for the RBP register.
    pub const RBP: RegNum = RegNum(UNW_X86_64_RBP);
    /// An x86_64-specific identifier for the RSP register.
    pub const RSP: RegNum = RegNum(UNW_X86_64_RSP);
    /// An x86_64-specific identifier for the R8 register.
    pub const R8: RegNum = RegNum(UNW_X86_64_R8);
    /// An x86_64-specific identifier for the R9 register.
    pub const R9: RegNum = RegNum(UNW_X86_64_R9);
    /// An x86_64-specific identifier for the R10 register.
    pub const R10: RegNum = RegNum(UNW_X86_64_R10);
    /// An x86_64-specific identifier for the R11 register.
    pub const R11: RegNum = RegNum(UNW_X86_64_R11);
    /// An x86_64-specific identifier for the R12 register.
    pub const R12: RegNum = RegNum(UNW_X86_64_R12);
    /// An x86_64-specific identifier for the R13 register.
    pub const R13: RegNum = RegNum(UNW_X86_64_R13);
    /// An x86_64-specific identifier for the R14 register.
    pub const R14: RegNum = RegNum(UNW_X86_64_R14);
    /// An x86_64-specific identifier for the R15 register.
    pub const R15: RegNum = RegNum(UNW_X86_64_R15);
    /// An x86_64-specific identifier for the RIP register.
    pub const RIP: RegNum = RegNum(UNW_X86_64_RIP);
    /// An x86_64-specific identifier for the canonical frame address.
    pub const CFA: RegNum = RegNum(UNW_X86_64_CFA);
}
