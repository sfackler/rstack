use libc::{
    c_char, c_int, c_long, c_uint, c_ulong, c_void, gid_t, mode_t, size_t, time_t, uid_t,
    Elf32_Chdr, Elf32_Ehdr, Elf32_Off, Elf32_Phdr, Elf32_Shdr, Elf64_Chdr, Elf64_Ehdr, Elf64_Off,
    Elf64_Phdr, Elf64_Shdr,
};

c_enum! {
    Elf_Type {
        ELF_T_BYTE = 0,
        ELF_T_ADDR = 1,
        ELF_T_DYN = 2,
        ELF_T_EHDR = 3,
        ELF_T_HALF = 4,
        ELF_T_OFF = 5,
        ELF_T_PHDR = 6,
        ELF_T_RELA = 7,
        ELF_T_REL = 8,
        ELF_T_SHDR = 9,
        ELF_T_SWORD = 10,
        ELF_T_SYM = 11,
        ELF_T_WORD = 12,
        ELF_T_XWORD = 13,
        ELF_T_SXWORD = 14,
        ELF_T_VDEF = 15,
        ELF_T_VDAUX = 16,
        ELF_T_VNEED = 17,
        ELF_T_VNAUX = 18,
        ELF_T_NHDR = 19,
        ELF_T_SYMINFO = 20,
        ELF_T_MOVE = 21,
        ELF_T_LIB = 22,
        ELF_T_GNUHASH = 23,
        ELF_T_AUXV = 24,
        ELF_T_CHDR = 25,
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Elf_Data {
    pub d_buf: *mut c_void,
    pub d_type: Elf_Type,
    pub d_version: c_uint,
    pub d_size: size_t,
    pub d_off: i64,
    pub d_align: size_t,
}

c_enum! {
    Elf_Cmd {
        ELF_C_NULL = 0,
        ELF_C_READ = 1,
        ELF_C_RDWR = 2,
        ELF_C_WRITE = 3,
        ELF_C_CLR = 4,
        ELF_C_SET = 5,
        ELF_C_FDDONE = 6,
        ELF_C_FDREAD = 7,
        ELF_C_READ_MMAP = 8,
        ELF_C_RDWR_MMAP = 9,
        ELF_C_WRITE_MMAP = 10,
        ELF_C_READ_MMAP_PRIVATE = 11,
        ELF_C_EMPTY = 12,
    }
}

pub const ELF_F_DIRTY: c_uint = 0x1;
pub const ELF_F_LAYOUT: c_uint = 0x4;
pub const ELF_F_PERMISSIVE: c_uint = 0x8;

pub const ELF_CHF_FORCE: c_uint = 0x1;

c_enum! {
    Elf_Kind {
        ELF_K_NONE = 0,
        ELF_K_AR = 1,
        ELF_K_COFF = 2,
        ELF_K_ELF = 3,
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Elf_Arhdr {
    pub ar_name: *mut c_char,
    pub ar_date: time_t,
    pub ar_uid: uid_t,
    pub ar_gid: gid_t,
    pub ar_mode: mode_t,
    pub ar_size: i64,
    pub ar_rawname: *mut c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Elf_Arsym {
    pub as_name: *mut c_char,
    pub as_off: size_t,
    pub as_hash: c_ulong,
}

pub enum Elf {}

pub enum Elf_Scn {}

extern "C" {
    pub fn elf_begin(__fildes: c_int, __cmd: Elf_Cmd, __ref: *mut Elf) -> *mut Elf;

    pub fn elf_clone(__elf: *mut Elf, __cmd: Elf_Cmd) -> *mut Elf;

    pub fn elf_memory(__image: *mut c_char, __size: size_t) -> *mut Elf;

    pub fn elf_next(__elf: *mut Elf) -> Elf_Cmd;

    pub fn elf_end(__elf: *mut Elf) -> c_int;

    pub fn elf_update(__elf: *mut Elf, __cmd: Elf_Cmd) -> i64;

    pub fn elf_kind(__elf: *mut Elf) -> Elf_Kind;

    pub fn elf_getbase(__elf: *mut Elf) -> i64;

    pub fn elf_getident(__elf: *mut Elf, __nbytes: *mut size_t) -> *mut c_char;

    pub fn elf32_getehdr(__elf: *mut Elf) -> *mut Elf32_Ehdr;

    pub fn elf64_getehdr(__elf: *mut Elf) -> *mut Elf64_Ehdr;

    pub fn elf32_newehdr(__elf: *mut Elf) -> *mut Elf32_Ehdr;

    pub fn elf64_newehdr(__elf: *mut Elf) -> *mut Elf64_Ehdr;

    pub fn elf_getphdrnum(__elf: *mut Elf, __dst: *mut size_t) -> c_int;

    pub fn elf32_getphdr(__elf: *mut Elf) -> *mut Elf32_Phdr;

    pub fn elf64_getphdr(__elf: *mut Elf) -> *mut Elf64_Phdr;

    pub fn elf32_newphdr(__elf: *mut Elf, __cnt: size_t) -> *mut Elf32_Phdr;

    pub fn elf64_newphdr(__elf: *mut Elf, __cnt: size_t) -> *mut Elf64_Phdr;

    pub fn elf_getscn(__elf: *mut Elf, __index: size_t) -> *mut Elf_Scn;

    pub fn elf32_offscn(__elf: *mut Elf, __offset: Elf32_Off) -> *mut Elf_Scn;

    pub fn elf64_offscn(__elf: *mut Elf, __offset: Elf64_Off) -> *mut Elf_Scn;

    pub fn elf_ndxscn(__scn: *mut Elf_Scn) -> size_t;

    pub fn elf_nextscn(__elf: *mut Elf, __scn: *mut Elf_Scn) -> *mut Elf_Scn;

    pub fn elf_newscn(__elf: *mut Elf) -> *mut Elf_Scn;

    pub fn elf_scnshndx(__scn: *mut Elf_Scn) -> c_int;

    pub fn elf_getshdrnum(__elf: *mut Elf, __dst: *mut size_t) -> c_int;

    pub fn elf_getshdrstrndx(__elf: *mut Elf, __dst: *mut size_t) -> c_int;

    pub fn elf32_getshdr(__scn: *mut Elf_Scn) -> *mut Elf32_Shdr;

    pub fn elf64_getshdr(__scn: *mut Elf_Scn) -> *mut Elf64_Shdr;

    pub fn elf32_getchdr(__scn: *mut Elf_Scn) -> *mut Elf32_Chdr;

    pub fn elf64_getchdr(__scn: *mut Elf_Scn) -> *mut Elf64_Chdr;

    pub fn elf_compress(scn: *mut Elf_Scn, type_: c_int, flags: c_uint) -> c_int;

    pub fn elf_flagelf(__elf: *mut Elf, __cmd: Elf_Cmd, __flags: c_uint) -> c_uint;

    pub fn elf_flagehdr(__elf: *mut Elf, __cmd: Elf_Cmd, __flags: c_uint) -> c_uint;

    pub fn elf_flagphdr(__elf: *mut Elf, __cmd: Elf_Cmd, __flags: c_uint) -> c_uint;

    pub fn elf_flagscn(__scn: *mut Elf_Scn, __cmd: Elf_Cmd, __flags: c_uint) -> c_uint;

    pub fn elf_flagdata(__data: *mut Elf_Data, __cmd: Elf_Cmd, __flags: c_uint) -> c_uint;

    pub fn elf_flagshdr(__scn: *mut Elf_Scn, __cmd: Elf_Cmd, __flags: c_uint) -> c_uint;

    pub fn elf_getdata(__scn: *mut Elf_Scn, __data: *mut Elf_Data) -> *mut Elf_Data;

    pub fn elf_rawdata(__scn: *mut Elf_Scn, __data: *mut Elf_Data) -> *mut Elf_Data;

    pub fn elf_newdata(__scn: *mut Elf_Scn) -> *mut Elf_Data;

    pub fn elf_getdata_rawchunk(
        __elf: *mut Elf,
        __offset: i64,
        __size: size_t,
        __type: Elf_Type,
    ) -> *mut Elf_Data;

    pub fn elf_getarhdr(__elf: *mut Elf) -> *mut Elf_Arhdr;

    pub fn elf_getaroff(__elf: *mut Elf) -> i64;

    pub fn elf_rand(__elf: *mut Elf, __offset: size_t) -> size_t;

    pub fn elf_getarsym(__elf: *mut Elf, __narsyms: *mut size_t) -> *mut Elf_Arsym;

    pub fn elf_cntl(__elf: *mut Elf, __cmd: Elf_Cmd) -> c_int;

    pub fn elf_rawfile(__elf: *mut Elf, __nbytes: *mut size_t) -> *mut c_char;

    pub fn elf32_fsize(__type: Elf_Type, __count: size_t, __version: c_uint) -> size_t;

    pub fn elf64_fsize(__type: Elf_Type, __count: size_t, __version: c_uint) -> size_t;

    pub fn elf32_xlatetom(
        __dest: *mut Elf_Data,
        __src: *const Elf_Data,
        __encode: c_uint,
    ) -> *mut Elf_Data;

    pub fn elf64_xlatetom(
        __dest: *mut Elf_Data,
        __src: *const Elf_Data,
        __encode: c_uint,
    ) -> *mut Elf_Data;

    pub fn elf32_xlatetof(
        __dest: *mut Elf_Data,
        __src: *const Elf_Data,
        __encode: c_uint,
    ) -> *mut Elf_Data;

    pub fn elf64_xlatetof(
        __dest: *mut Elf_Data,
        __src: *const Elf_Data,
        __encode: c_uint,
    ) -> *mut Elf_Data;

    pub fn elf_errno() -> c_int;

    pub fn elf_errmsg(__error: c_int) -> *const c_char;

    pub fn elf_version(__version: c_uint) -> c_uint;

    pub fn elf_fill(__fill: c_int);

    pub fn elf_hash(__string: *const c_char) -> c_ulong;

    pub fn elf_gnu_hash(__string: *const c_char) -> c_ulong;

    pub fn elf32_checksum(__elf: *mut Elf) -> c_long;

    pub fn elf64_checksum(__elf: *mut Elf) -> c_long;
}
