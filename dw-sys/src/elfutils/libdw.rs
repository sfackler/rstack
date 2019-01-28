use libc::{c_char, c_int, c_long, c_uchar, c_uint, c_void, ptrdiff_t, size_t};

use crate::*;

c_enum! {
    Dwarf_Cmd {
        DWARF_C_READ = 0,
        DWARF_C_RDWR = 1,
        DWARF_C_WRITE = 2,
    }
}

pub const DWARF_CB_OK: c_int = 0;
pub const DWARF_CB_ABORT: c_int = 1;

pub const DW_TAG_invalid: c_int = 0;

pub type Dwarf_Off = GElf_Off;

pub type Dwarf_Addr = GElf_Addr;

pub type Dwarf_Word = GElf_Xword;
pub type Dwarf_Sword = GElf_Sxword;
pub type Dwarf_Half = GElf_Half;

pub enum Dwarf_Abbrev {}

pub const DWARF_END_ABBREV: *mut Dwarf_Abbrev = -1isize as *mut Dwarf_Abbrev;

pub enum Dwarf_Lines {}

pub enum Dwarf_Line {}

pub enum Dwarf_Files {}

pub enum Dwarf_Arange {}

pub enum Dwarf_Aranges {}

pub enum Dwarf_CU {}

pub enum Dwarf_Macro {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwarf_Attribute {
    pub code: c_uint,
    pub form: c_uint,
    pub valp: *mut c_uchar,
    pub cu: *mut Dwarf_CU,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwarf_Block {
    pub length: Dwarf_Word,
    pub data: *mut c_uchar,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwarf_Die {
    pub addr: *mut c_void,
    pub cu: *mut Dwarf_CU,
    pub abbrev: *mut Dwarf_Abbrev,
    pub padding__: c_long,
}

pub const DWARF_END_DIE: *mut Dwarf_Die = -1isize as *mut Dwarf_Die;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwarf_Global {
    pub cu_offset: Dwarf_Off,
    pub die_offset: Dwarf_Off,
    pub name: *const c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwarf_Op {
    pub atom: u8,
    pub number: Dwarf_Word,
    pub number2: Dwarf_Word,
    pub offset: Dwarf_Word,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwarf_CIE {
    pub CIE_id: Dwarf_Off,
    pub initial_instructions: *const u8,
    pub initial_instructions_end: *const u8,
    pub code_alignment_factor: Dwarf_Word,
    pub data_alignment_factor: Dwarf_Sword,
    pub return_address_register: Dwarf_Word,
    pub augmentation: *const c_char,
    pub augmentation_data: *const u8,
    pub augmentation_data_size: size_t,
    pub fde_augmentation_data_size: size_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwarf_FDE {
    pub CIE_pointer: Dwarf_Off,
    pub start: *const u8,
    pub end: *const u8,
}

#[repr(C)]
pub union Dwarf_CFI_Entry {
    pub CIE_id: Dwarf_Off,
    pub cie: Dwarf_CIE,
    pub fde: Dwarf_FDE,
}

pub enum Dwarf_Frame {}

pub enum Dwarf_CFI {}

pub enum Dwarf {}

pub type Dwarf_OOM = unsafe extern "C" fn() -> !;

extern "C" {
    pub fn dwarf_begin(fildes: c_int, cmd: Dwarf_Cmd) -> *mut Dwarf;

    pub fn dwarf_begin_elf(elf: *mut Elf, cmd: Dwarf_Cmd, scngrp: *mut Elf_Scn) -> *mut Dwarf;

    pub fn dwarf_getelf(dwarf: *mut Dwarf) -> *mut Elf;

    pub fn dwarf_cu_getdwarf(dcu: *mut Dwarf_CU) -> *mut Dwarf;

    pub fn dwarf_getalt(main: *mut Dwarf) -> *mut Dwarf;

    pub fn dwarf_setalt(main: *mut Dwarf, alt: *mut Dwarf);

    pub fn dwarf_end(dwarf: *mut Dwarf) -> c_int;

    pub fn dwarf_nextcu(
        dwarf: *mut Dwarf,
        off: Dwarf_Off,
        next_off: *mut Dwarf_Off,
        header_sizep: *mut size_t,
        abbrev_offsetp: *mut Dwarf_Off,
        address_sizep: *mut u8,
        offset_sizep: *mut u8,
    ) -> c_int;

    pub fn dwarf_next_unit(
        dwarf: *mut Dwarf,
        off: Dwarf_Off,
        next_off: *mut Dwarf_Off,
        header_sizep: *mut size_t,
        versionp: *mut Dwarf_Half,
        abbrev_offsetp: *mut Dwarf_Off,
        address_sizep: *mut u8,
        offset_sizep: *mut u8,
        type_signaturep: *mut u64,
        type_offsetp: *mut Dwarf_Off,
    ) -> c_int;

    pub fn dwarf_next_cfi(
        e_ident: *const c_uchar,
        data: *mut Elf_Data,
        eh_frame_p: bool,
        offset: Dwarf_Off,
        next_offset: *mut Dwarf_Off,
        entry: *mut Dwarf_CFI_Entry,
    ) -> c_int;

    pub fn dwarf_getcfi(dwarf: *mut Dwarf) -> *mut Dwarf_CFI;

    pub fn dwarf_getcfi_elf(elf: *mut Elf) -> *mut Dwarf_CFI;

    pub fn dwarf_cfi_end(cache: *mut Dwarf_CFI) -> c_int;

    pub fn dwarf_offdie(
        dbg: *mut Dwarf,
        offset: Dwarf_Off,
        result: *mut Dwarf_Die,
    ) -> *mut Dwarf_Die;

    pub fn dwarf_offdie_types(
        dbg: *mut Dwarf,
        offset: Dwarf_Off,
        result: *mut Dwarf_Die,
    ) -> *mut Dwarf_Die;

    pub fn dwarf_dieoffset(die: *mut Dwarf_Die) -> Dwarf_Off;

    pub fn dwarf_cuoffset(die: *mut Dwarf_Die) -> Dwarf_Off;

    pub fn dwarf_diecu(
        die: *mut Dwarf_Die,
        result: *mut Dwarf_Die,
        address_sizep: *mut u8,
        offset_sizep: *mut u8,
    ) -> *mut Dwarf_Die;

    pub fn dwarf_cu_die(
        cu: *mut Dwarf_CU,
        result: *mut Dwarf_Die,
        versionp: *mut Dwarf_Half,
        abbrev_offsetp: *mut Dwarf_Off,
        address_sizep: *mut u8,
        offset_sizep: *mut u8,
        type_signaturep: *mut u64,
        type_offsetp: *mut Dwarf_Off,
    ) -> *mut Dwarf_Die;

    pub fn dwarf_addrdie(
        dbg: *mut Dwarf,
        addr: Dwarf_Addr,
        result: *mut Dwarf_Die,
    ) -> *mut Dwarf_Die;

    pub fn dwarf_child(die: *mut Dwarf_Die, result: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_siblingof(die: *mut Dwarf_Die, result: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_peel_type(die: *mut Dwarf_Die, result: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_haschildren(die: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_getattrs(
        die: *mut Dwarf_Die,
        callback: Option<unsafe extern "C" fn(_: *mut Dwarf_Attribute, _: *mut c_void) -> c_int>,
        arg: *mut c_void,
        offset: ptrdiff_t,
    ) -> ptrdiff_t;

    pub fn dwarf_tag(die: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_attr(
        die: *mut Dwarf_Die,
        search_name: c_uint,
        result: *mut Dwarf_Attribute,
    ) -> *mut Dwarf_Attribute;

    pub fn dwarf_hasattr(die: *mut Dwarf_Die, search_name: c_uint) -> c_int;

    pub fn dwarf_attr_integrate(
        die: *mut Dwarf_Die,
        search_name: c_uint,
        result: *mut Dwarf_Attribute,
    ) -> *mut Dwarf_Attribute;

    pub fn dwarf_hasattr_integrate(die: *mut Dwarf_Die, search_name: c_uint) -> c_int;

    pub fn dwarf_hasform(attr: *mut Dwarf_Attribute, search_form: c_uint) -> c_int;

    pub fn dwarf_whatattr(attr: *mut Dwarf_Attribute) -> c_uint;

    pub fn dwarf_whatform(attr: *mut Dwarf_Attribute) -> c_uint;

    pub fn dwarf_formstring(attrp: *mut Dwarf_Attribute) -> *const c_char;

    pub fn dwarf_formudata(attr: *mut Dwarf_Attribute, return_uval: *mut Dwarf_Word) -> c_int;

    pub fn dwarf_formsdata(attr: *mut Dwarf_Attribute, return_uval: *mut Dwarf_Sword) -> c_int;

    pub fn dwarf_formaddr(attr: *mut Dwarf_Attribute, return_addr: *mut Dwarf_Addr) -> c_int;

    pub fn dwarf_formref_die(attr: *mut Dwarf_Attribute, die_mem: *mut Dwarf_Die)
        -> *mut Dwarf_Die;

    pub fn dwarf_formblock(attr: *mut Dwarf_Attribute, return_block: *mut Dwarf_Block) -> c_int;

    pub fn dwarf_formflag(attr: *mut Dwarf_Attribute, return_bool: *mut bool) -> c_int;

    pub fn dwarf_diename(die: *mut Dwarf_Die) -> *const c_char;

    pub fn dwarf_highpc(die: *mut Dwarf_Die, return_addr: *mut Dwarf_Addr) -> c_int;

    pub fn dwarf_lowpc(die: *mut Dwarf_Die, return_addr: *mut Dwarf_Addr) -> c_int;

    pub fn dwarf_entrypc(die: *mut Dwarf_Die, return_addr: *mut Dwarf_Addr) -> c_int;

    pub fn dwarf_haspc(die: *mut Dwarf_Die, pc: Dwarf_Addr) -> c_int;

    pub fn dwarf_ranges(
        die: *mut Dwarf_Die,
        offset: ptrdiff_t,
        basep: *mut Dwarf_Addr,
        startp: *mut Dwarf_Addr,
        endp: *mut Dwarf_Addr,
    ) -> ptrdiff_t;

    pub fn dwarf_bytesize(die: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_bitsize(die: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_bitoffset(die: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_arrayorder(die: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_srclang(die: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_getabbrev(
        die: *mut Dwarf_Die,
        offset: Dwarf_Off,
        lengthp: *mut size_t,
    ) -> *mut Dwarf_Abbrev;

    pub fn dwarf_offabbrev(
        dbg: *mut Dwarf,
        offset: Dwarf_Off,
        lengthp: *mut size_t,
        abbrevp: *mut Dwarf_Abbrev,
    ) -> c_int;

    pub fn dwarf_getabbrevcode(abbrev: *mut Dwarf_Abbrev) -> c_uint;

    pub fn dwarf_getabbrevtag(abbrev: *mut Dwarf_Abbrev) -> c_uint;

    pub fn dwarf_abbrevhaschildren(abbrev: *mut Dwarf_Abbrev) -> c_int;

    pub fn dwarf_getattrcnt(abbrev: *mut Dwarf_Abbrev, attrcntp: *mut size_t) -> c_int;

    pub fn dwarf_getabbrevattr(
        abbrev: *mut Dwarf_Abbrev,
        idx: size_t,
        namep: *mut c_uint,
        formp: *mut c_uint,
        offset: *mut Dwarf_Off,
    ) -> c_int;

    pub fn dwarf_getstring(dbg: *mut Dwarf, offset: Dwarf_Off, lep: *mut size_t) -> *const c_char;

    pub fn dwarf_getpubnames(
        dbg: *mut Dwarf,
        callback: Option<
            unsafe extern "C" fn(_: *mut Dwarf, _: *mut Dwarf_Global, _: *mut c_void) -> c_int,
        >,
        arg: *mut c_void,
        offset: ptrdiff_t,
    ) -> ptrdiff_t;

    pub fn dwarf_getsrclines(
        cudie: *mut Dwarf_Die,
        lines: *mut *mut Dwarf_Lines,
        nlines: *mut size_t,
    ) -> c_int;

    pub fn dwarf_onesrcline(lines: *mut Dwarf_Lines, idx: size_t) -> *mut Dwarf_Line;

    pub fn dwarf_getsrcfiles(
        cudie: *mut Dwarf_Die,
        files: *mut *mut Dwarf_Files,
        nfiles: *mut size_t,
    ) -> c_int;

    pub fn dwarf_getsrc_die(cudie: *mut Dwarf_Die, addr: Dwarf_Addr) -> *mut Dwarf_Line;

    pub fn dwarf_getsrc_file(
        dbg: *mut Dwarf,
        fname: *const c_char,
        line: c_int,
        col: c_int,
        srcp: *mut *mut *mut Dwarf_Line,
        nsrcs: *mut size_t,
    ) -> c_int;

    pub fn dwarf_lineaddr(line: *mut Dwarf_Line, addrp: *mut Dwarf_Addr) -> c_int;

    pub fn dwarf_lineop_index(line: *mut Dwarf_Line, op_indexp: *mut c_uint) -> c_int;

    pub fn dwarf_lineno(line: *mut Dwarf_Line, linep: *mut c_int) -> c_int;

    pub fn dwarf_linecol(line: *mut Dwarf_Line, colp: *mut c_int) -> c_int;

    pub fn dwarf_linebeginstatement(line: *mut Dwarf_Line, flagp: *mut bool) -> c_int;

    pub fn dwarf_lineendsequence(line: *mut Dwarf_Line, flagp: *mut bool) -> c_int;

    pub fn dwarf_lineblock(line: *mut Dwarf_Line, flagp: *mut bool) -> c_int;

    pub fn dwarf_lineprologueend(line: *mut Dwarf_Line, flagp: *mut bool) -> c_int;

    pub fn dwarf_lineepiloguebegin(line: *mut Dwarf_Line, flagp: *mut bool) -> c_int;

    pub fn dwarf_lineisa(line: *mut Dwarf_Line, isap: *mut c_uint) -> c_int;

    pub fn dwarf_linediscriminator(line: *mut Dwarf_Line, discp: *mut c_uint) -> c_int;

    pub fn dwarf_linesrc(
        line: *mut Dwarf_Line,
        mtime: *mut Dwarf_Word,
        length: *mut Dwarf_Word,
    ) -> *const c_char;

    pub fn dwarf_filesrc(
        file: *mut Dwarf_Files,
        idx: size_t,
        mtime: *mut Dwarf_Word,
        length: *mut Dwarf_Word,
    ) -> *const c_char;

    pub fn dwarf_getsrcdirs(
        files: *mut Dwarf_Files,
        result: *mut *const *const c_char,
        ndirs: *mut size_t,
    ) -> c_int;

    pub fn dwarf_getlocation(
        attr: *mut Dwarf_Attribute,
        expr: *mut *mut Dwarf_Op,
        exprlen: *mut size_t,
    ) -> c_int;

    pub fn dwarf_getlocation_addr(
        attr: *mut Dwarf_Attribute,
        address: Dwarf_Addr,
        exprs: *mut *mut Dwarf_Op,
        exprlens: *mut size_t,
        nlocs: size_t,
    ) -> c_int;

    pub fn dwarf_getlocations(
        attr: *mut Dwarf_Attribute,
        offset: ptrdiff_t,
        basep: *mut Dwarf_Addr,
        startp: *mut Dwarf_Addr,
        endp: *mut Dwarf_Addr,
        expr: *mut *mut Dwarf_Op,
        exprlen: *mut size_t,
    ) -> ptrdiff_t;

    pub fn dwarf_getlocation_implicit_value(
        attr: *mut Dwarf_Attribute,
        op: *const Dwarf_Op,
        return_block: *mut Dwarf_Block,
    ) -> c_int;

    pub fn dwarf_getlocation_implicit_pointer(
        attr: *mut Dwarf_Attribute,
        op: *const Dwarf_Op,
        result: *mut Dwarf_Attribute,
    ) -> c_int;

    pub fn dwarf_getlocation_die(
        attr: *mut Dwarf_Attribute,
        op: *const Dwarf_Op,
        result: *mut Dwarf_Die,
    ) -> c_int;

    pub fn dwarf_getlocation_attr(
        attr: *mut Dwarf_Attribute,
        op: *const Dwarf_Op,
        result: *mut Dwarf_Attribute,
    ) -> c_int;

    pub fn dwarf_aggregate_size(die: *mut Dwarf_Die, size: *mut Dwarf_Word) -> c_int;

    pub fn dwarf_getscopes(
        cudie: *mut Dwarf_Die,
        pc: Dwarf_Addr,
        scopes: *mut *mut Dwarf_Die,
    ) -> c_int;

    pub fn dwarf_getscopes_die(die: *mut Dwarf_Die, scopes: *mut *mut Dwarf_Die) -> c_int;

    pub fn dwarf_getscopevar(
        scopes: *mut Dwarf_Die,
        nscopes: c_int,
        name: *const c_char,
        skip_shadows: c_int,
        match_file: *const c_char,
        match_lineno: c_int,
        match_linecol: c_int,
        result: *mut Dwarf_Die,
    ) -> c_int;

    pub fn dwarf_getaranges(
        dbg: *mut Dwarf,
        aranges: *mut *mut Dwarf_Aranges,
        naranges: *mut size_t,
    ) -> c_int;

    pub fn dwarf_onearange(aranges: *mut Dwarf_Aranges, idx: size_t) -> *mut Dwarf_Arange;

    pub fn dwarf_getarangeinfo(
        arange: *mut Dwarf_Arange,
        addrp: *mut Dwarf_Addr,
        lengthp: *mut Dwarf_Word,
        offsetp: *mut Dwarf_Off,
    ) -> c_int;

    pub fn dwarf_getarange_addr(aranges: *mut Dwarf_Aranges, addr: Dwarf_Addr)
        -> *mut Dwarf_Arange;

    pub fn dwarf_getfuncs(
        cudie: *mut Dwarf_Die,
        callback: Option<unsafe extern "C" fn(_: *mut Dwarf_Die, _: *mut c_void) -> c_int>,
        arg: *mut c_void,
        offset: ptrdiff_t,
    ) -> ptrdiff_t;

    pub fn dwarf_decl_file(decl: *mut Dwarf_Die) -> *const c_char;

    pub fn dwarf_decl_line(decl: *mut Dwarf_Die, linep: *mut c_int) -> c_int;

    pub fn dwarf_decl_column(decl: *mut Dwarf_Die, colp: *mut c_int) -> c_int;

    pub fn dwarf_func_inline(func: *mut Dwarf_Die) -> c_int;

    pub fn dwarf_func_inline_instances(
        func: *mut Dwarf_Die,
        callback: Option<unsafe extern "C" fn(_: *mut Dwarf_Die, _: *mut c_void) -> c_int>,
        arg: *mut c_void,
    ) -> c_int;

    pub fn dwarf_entry_breakpoints(die: *mut Dwarf_Die, bkpts: *mut *mut Dwarf_Addr) -> c_int;

    pub fn dwarf_getmacros(
        cudie: *mut Dwarf_Die,
        callback: Option<unsafe extern "C" fn(_: *mut Dwarf_Macro, _: *mut c_void) -> c_int>,
        arg: *mut c_void,
        token: ptrdiff_t,
    ) -> ptrdiff_t;

    pub fn dwarf_getmacros_off(
        dbg: *mut Dwarf,
        macoff: Dwarf_Off,
        callback: Option<unsafe extern "C" fn(_: *mut Dwarf_Macro, _: *mut c_void) -> c_int>,
        arg: *mut c_void,
        token: ptrdiff_t,
    ) -> ptrdiff_t;

    pub fn dwarf_macro_getsrcfiles(
        dbg: *mut Dwarf,
        macro_: *mut Dwarf_Macro,
        files: *mut *mut Dwarf_Files,
        nfiles: *mut size_t,
    ) -> c_int;

    pub fn dwarf_macro_opcode(macro_: *mut Dwarf_Macro, opcodeop: *mut c_uint) -> c_int;

    pub fn dwarf_macro_getparamcnt(macro_: *mut Dwarf_Macro, paramcntp: *mut size_t) -> c_int;

    pub fn dwarf_macro_param(
        macro_: *mut Dwarf_Macro,
        idx: size_t,
        attribute: *mut Dwarf_Attribute,
    ) -> c_int;

    pub fn dwarf_macro_param1(macro_: *mut Dwarf_Macro, paramp: *mut Dwarf_Word) -> c_int;

    pub fn dwarf_macro_param2(
        macro_: *mut Dwarf_Macro,
        paramp: *mut Dwarf_Word,
        strp: *mut *const c_char,
    ) -> c_int;

    pub fn dwarf_cfi_addrframe(
        cache: *mut Dwarf_CFI,
        address: Dwarf_Addr,
        frame: *mut *mut Dwarf_Frame,
    ) -> c_int;

    pub fn dwarf_frame_info(
        frame: *mut Dwarf_Frame,
        start: *mut Dwarf_Addr,
        end: *mut Dwarf_Addr,
        signalp: *mut bool,
    ) -> c_int;

    pub fn dwarf_frame_cfa(
        frame: *mut Dwarf_Frame,
        ops: *mut *mut Dwarf_Op,
        nops: *mut size_t,
    ) -> c_int;

    pub fn dwarf_frame_register(
        frame: *mut Dwarf_Frame,
        regno: c_int,
        ops_mem: *mut [Dwarf_Op; 3],
        ops: *mut *mut Dwarf_Op,
        nops: *mut size_t,
    ) -> c_int;

    pub fn dwarf_errno() -> c_int;

    pub fn dwarf_errmsg(err: c_int) -> *const c_char;

    pub fn dwarf_new_oom_handler(dbg: *mut Dwarf, handler: Dwarf_OOM) -> Dwarf_OOM;
}
