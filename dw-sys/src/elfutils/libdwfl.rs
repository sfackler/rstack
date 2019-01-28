use libc::{c_char, c_int, c_uchar, c_uint, c_void, pid_t, ptrdiff_t, size_t, FILE};

use crate::*;

pub enum Dwfl {}
pub enum Dwfl_Module {}
pub enum Dwfl_Thread {}
pub enum Dwfl_Frame {}
pub enum Dwfl_Line {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwfl_Callbacks {
    pub find_elf: Option<
        unsafe extern "C" fn(
            mod_: *mut Dwfl_Module,
            userdata: *mut *mut c_void,
            modname: *const c_char,
            base: Dwarf_Addr,
            file_name: *mut *mut c_char,
            elfp: *mut *mut Elf,
        ) -> c_int,
    >,
    pub find_debuginfo: Option<
        unsafe extern "C" fn(
            mod_: *mut Dwfl_Module,
            userdata: *mut *mut c_void,
            modname: *const c_char,
            base: Dwarf_Addr,
            file_name: *const c_char,
            debuglink_file: *const c_char,
            debuglink_crc: GElf_Word,
            debuginfo_file_name: *mut *mut c_char,
        ) -> c_int,
    >,
    pub section_address: Option<
        unsafe extern "C" fn(
            mod_: *mut Dwfl_Module,
            userdata: *mut *mut c_void,
            modname: *const c_char,
            base: Dwarf_Addr,
            secname: *const c_char,
            shndx: GElf_Word,
            shdr: *const GElf_Shdr,
            addr: *mut Dwarf_Addr,
        ) -> c_int,
    >,
    pub debuginfo_path: *mut *mut c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Dwfl_Thread_Callbacks {
    pub next_thread: Option<
        unsafe extern "C" fn(dwfl: *mut Dwfl, dwfl_arg: *mut c_void, thread_argp: *mut *mut c_void)
            -> pid_t,
    >,
    pub get_thread: Option<
        unsafe extern "C" fn(
            dwfl: *mut Dwfl,
            tid: pid_t,
            dwfl_arg: *mut c_void,
            thread_argp: *mut *mut c_void,
        ) -> bool,
    >,
    pub memory_read: Option<
        unsafe extern "C" fn(
            dwfl: *mut Dwfl,
            addr: Dwarf_Addr,
            result: *mut Dwarf_Word,
            dwfl_arg: *mut c_void,
        ) -> bool,
    >,
    pub set_initial_registers:
        Option<unsafe extern "C" fn(thread: *mut Dwfl_Thread, thread_arg: *mut c_void) -> bool>,
    pub detach: Option<unsafe extern "C" fn(dwfl: *mut Dwfl, dwfl_arg: *mut c_void)>,
    pub thread_detach:
        Option<unsafe extern "C" fn(thread: *mut Dwfl_Thread, thread_arg: *mut c_void)>,
}

extern "C" {
    pub fn dwfl_begin(callbacks: *const Dwfl_Callbacks) -> *mut Dwfl;

    pub fn dwfl_end(dwfl: *mut Dwfl);

    pub fn dwfl_version(dwfl: *mut Dwfl) -> *const c_char;

    pub fn dwfl_errno() -> c_int;

    pub fn dwfl_errmsg(err: c_int) -> *const c_char;

    pub fn dwfl_report_begin(dwfl: *mut Dwfl);

    pub fn dwfl_report_segment(
        dwfl: *mut Dwfl,
        ndx: c_int,
        phdr: *const GElf_Phdr,
        bias: GElf_Addr,
        ident: *const c_void,
    ) -> c_int;

    pub fn dwfl_report_module(
        dwfl: *mut Dwfl,
        name: *const c_char,
        start: Dwarf_Addr,
        end: Dwarf_Addr,
    ) -> *mut Dwfl_Module;

    pub fn dwfl_report_elf(
        dwfl: *mut Dwfl,
        name: *const c_char,
        file_name: *const c_char,
        fd: c_int,
        base: GElf_Addr,
        add_p_vaddr: bool,
    ) -> *mut Dwfl_Module;

    pub fn dwfl_report_offline(
        dwfl: *mut Dwfl,
        name: *const c_char,
        file_name: *const c_char,
        fd: c_int,
    ) -> *mut Dwfl_Module;

    pub fn dwfl_report_end(
        dwfl: *mut Dwfl,
        removed: Option<
            unsafe extern "C" fn(
                _: *mut Dwfl_Module,
                _: *mut c_void,
                _: *const c_char,
                _: Dwarf_Addr,
                arg: *mut c_void,
            ) -> c_int,
        >,
        arg: *mut c_void,
    ) -> c_int;

    pub fn dwfl_report_begin_add(dwfl: *mut Dwfl);

    pub fn dwfl_module_info(
        mod_: *mut Dwfl_Module,
        userdata: *mut *mut *mut c_void,
        start: *mut Dwarf_Addr,
        end: *mut Dwarf_Addr,
        dwbias: *mut Dwarf_Addr,
        symbias: *mut Dwarf_Addr,
        mainfile: *mut *const c_char,
        debugfile: *mut *const c_char,
    ) -> *const c_char;

    pub fn dwfl_getmodules(
        dwfl: *mut Dwfl,
        callback: Option<
            unsafe extern "C" fn(
                _: *mut Dwfl_Module,
                _: *mut *mut c_void,
                _: *const c_char,
                _: Dwarf_Addr,
                arg: *mut c_void,
            ) -> c_int,
        >,
        arg: *mut c_void,
        offset: ptrdiff_t,
    ) -> ptrdiff_t;

    pub fn dwfl_addrmodule(dwfl: *mut Dwfl, address: Dwarf_Addr) -> *mut Dwfl_Module;

    pub fn dwfl_addrsegment(
        dwfl: *mut Dwfl,
        address: Dwarf_Addr,
        mod_: *mut *mut Dwfl_Module,
    ) -> c_int;

    pub fn dwfl_module_report_build_id(
        mod_: *mut Dwfl_Module,
        bits: *const c_uchar,
        len: size_t,
        vaddr: GElf_Addr,
    ) -> c_int;

    pub fn dwfl_module_build_id(
        mod_: *mut Dwfl_Module,
        bits: *mut *const c_uchar,
        vaddr: *mut GElf_Addr,
    ) -> c_int;

    pub fn dwfl_build_id_find_elf(
        _: *mut Dwfl_Module,
        _: *mut *mut c_void,
        _: *const c_char,
        _: Dwarf_Addr,
        _: *mut *mut c_char,
        _: *mut *mut Elf,
    ) -> c_int;

    pub fn dwfl_build_id_find_debuginfo(
        _: *mut Dwfl_Module,
        _: *mut *mut c_void,
        _: *const c_char,
        _: Dwarf_Addr,
        _: *const c_char,
        _: *const c_char,
        _: GElf_Word,
        _: *mut *mut c_char,
    ) -> c_int;

    pub fn dwfl_standard_find_debuginfo(
        _: *mut Dwfl_Module,
        _: *mut *mut c_void,
        _: *const c_char,
        _: Dwarf_Addr,
        _: *const c_char,
        _: *const c_char,
        _: GElf_Word,
        _: *mut *mut c_char,
    ) -> c_int;

    pub fn dwfl_linux_kernel_find_elf(
        _: *mut Dwfl_Module,
        _: *mut *mut c_void,
        _: *const c_char,
        _: Dwarf_Addr,
        _: *mut *mut c_char,
        _: *mut *mut Elf,
    ) -> c_int;

    pub fn dwfl_linux_kernel_module_section_address(
        _: *mut Dwfl_Module,
        _: *mut *mut c_void,
        _: *const c_char,
        _: Dwarf_Addr,
        _: *const c_char,
        _: GElf_Word,
        _: *const GElf_Shdr,
        addr: *mut Dwarf_Addr,
    ) -> c_int;

    pub fn dwfl_linux_kernel_report_kernel(dwfl: *mut Dwfl) -> c_int;

    pub fn dwfl_linux_kernel_report_modules(dwfl: *mut Dwfl) -> c_int;

    pub fn dwfl_linux_kernel_report_offline(
        dwfl: *mut Dwfl,
        release: *const c_char,
        predicate: Option<unsafe extern "C" fn(_: *const c_char, *const c_char) -> c_int>,
    ) -> c_int;

    pub fn dwfl_core_file_report(
        dwfl: *mut Dwfl,
        elf: *mut Elf,
        executable: *const c_char,
    ) -> c_int;

    pub fn dwfl_linux_proc_report(dwfl: *mut Dwfl, pid: pid_t) -> c_int;

    pub fn dwfl_linux_proc_maps_report(dwfl: *mut Dwfl, file: *mut FILE) -> c_int;

    pub fn dwfl_linux_proc_find_elf(
        mod_: *mut Dwfl_Module,
        userdata: *mut *mut c_void,
        module_name: *const c_char,
        base: Dwarf_Addr,
        file_name: *mut *mut c_char,
        _: *mut *mut Elf,
    ) -> c_int;

    pub fn dwfl_module_relocations(mod_: *mut Dwfl_Module) -> c_int;

    pub fn dwfl_module_relocate_address(mod_: *mut Dwfl_Module, address: *mut Dwarf_Addr) -> c_int;

    pub fn dwfl_module_relocation_info(
        mod_: *mut Dwfl_Module,
        idx: c_uint,
        shndxp: *mut GElf_Word,
    ) -> *const c_char;

    pub fn dwfl_module_getelf(_: *mut Dwfl_Module, bias: *mut GElf_Addr) -> *mut Elf;

    pub fn dwfl_module_getsymtab(mod_: *mut Dwfl_Module) -> c_int;

    pub fn dwfl_module_getsymtab_first_global(mod_: *mut Dwfl_Module) -> c_int;

    pub fn dwfl_module_getsym(
        mod_: *mut Dwfl_Module,
        ndx: c_int,
        sym: *mut GElf_Sym,
        shndxp: *mut GElf_Word,
    ) -> *const c_char;

    pub fn dwfl_module_getsym_info(
        mod_: *mut Dwfl_Module,
        ndx: c_int,
        sym: *mut GElf_Sym,
        addr: *mut GElf_Addr,
        shndxp: *mut GElf_Word,
        elfp: *mut *mut Elf,
        bias: *mut Dwarf_Addr,
    ) -> *const c_char;

    pub fn dwfl_module_addrname(mod_: *mut Dwfl_Module, address: GElf_Addr) -> *const c_char;

    pub fn dwfl_module_addrinfo(
        mod_: *mut Dwfl_Module,
        address: GElf_Addr,
        offset: *mut GElf_Off,
        sym: *mut GElf_Sym,
        shndxp: *mut GElf_Word,
        elfp: *mut *mut Elf,
        bias: *mut Dwarf_Addr,
    ) -> *const c_char;

    pub fn dwfl_module_addrsym(
        mod_: *mut Dwfl_Module,
        address: GElf_Addr,
        sym: *mut GElf_Sym,
        shndxp: *mut GElf_Word,
    ) -> *const c_char;

    pub fn dwfl_module_address_section(
        mod_: *mut Dwfl_Module,
        address: *mut Dwarf_Addr,
        bias: *mut Dwarf_Addr,
    ) -> *mut Elf_Scn;

    pub fn dwfl_module_getdwarf(_: *mut Dwfl_Module, bias: *mut Dwarf_Addr) -> *mut Dwarf;

    pub fn dwfl_getdwarf(
        _: *mut Dwfl,
        callback: Option<
            unsafe extern "C" fn(
                _: *mut Dwfl_Module,
                _: *mut *mut c_void,
                _: *const c_char,
                _: Dwarf_Addr,
                _: *mut Dwarf,
                _: Dwarf_Addr,
                _: *mut c_void,
            ) -> c_int,
        >,
        arg: *mut c_void,
        offset: ptrdiff_t,
    ) -> ptrdiff_t;

    pub fn dwfl_addrdwarf(dwfl: *mut Dwfl, addr: Dwarf_Addr, bias: *mut Dwarf_Addr) -> *mut Dwarf;

    pub fn dwfl_addrdie(dwfl: *mut Dwfl, addr: Dwarf_Addr, bias: *mut Dwarf_Addr)
        -> *mut Dwarf_Die;

    pub fn dwfl_module_addrdie(
        mod_: *mut Dwfl_Module,
        addr: Dwarf_Addr,
        bias: *mut Dwarf_Addr,
    ) -> *mut Dwarf_Die;

    pub fn dwfl_nextcu(
        dwfl: *mut Dwfl,
        lastcu: *mut Dwarf_Die,
        bias: *mut Dwarf_Addr,
    ) -> *mut Dwarf_Die;

    pub fn dwfl_module_nextcu(
        mod_: *mut Dwfl_Module,
        lastcu: *mut Dwarf_Die,
        bias: *mut Dwarf_Addr,
    ) -> *mut Dwarf_Die;

    pub fn dwfl_cumodule(cudie: *mut Dwarf_Die) -> *mut Dwfl_Module;

    pub fn dwfl_getsrclines(cudie: *mut Dwarf_Die, nlines: *mut size_t) -> c_int;

    pub fn dwfl_module_getsrc(mod_: *mut Dwfl_Module, addr: Dwarf_Addr) -> *mut Dwfl_Line;

    pub fn dwfl_getsrc(dwfl: *mut Dwfl, addr: Dwarf_Addr) -> *mut Dwfl_Line;

    pub fn dwfl_module_getsrc_file(
        mod_: *mut Dwfl_Module,
        fname: *const c_char,
        lineno: c_int,
        column: c_int,
        srcp: *mut *mut *mut Dwfl_Line,
        nsrcs: *mut size_t,
    ) -> c_int;

    pub fn dwfl_linemodule(line: *mut Dwfl_Line) -> *mut Dwfl_Module;

    pub fn dwfl_linecu(line: *mut Dwfl_Line) -> *mut Dwarf_Die;

    pub fn dwfl_lineinfo(
        line: *mut Dwfl_Line,
        addr: *mut Dwarf_Addr,
        linep: *mut c_int,
        colp: *mut c_int,
        mtime: *mut Dwarf_Word,
        length: *mut Dwarf_Word,
    ) -> *const c_char;

    pub fn dwfl_dwarf_line(line: *mut Dwfl_Line, bias: *mut Dwarf_Addr) -> *mut Dwarf_Line;

    pub fn dwfl_line_comp_dir(line: *mut Dwfl_Line) -> *const c_char;

    pub fn dwfl_module_return_value_location(
        mod_: *mut Dwfl_Module,
        functypedie: *mut Dwarf_Die,
        locops: *mut *const Dwarf_Op,
    ) -> c_int;

    pub fn dwfl_module_register_names(
        mod_: *mut Dwfl_Module,
        callback: Option<
            unsafe extern "C" fn(
                arg: *mut c_void,
                regno: c_int,
                setname: *const c_char,
                prefix: *const c_char,
                regname: *const c_char,
                bits: c_int,
                type_: c_int,
            ) -> c_int,
        >,
        arg: *mut c_void,
    ) -> c_int;

    pub fn dwfl_module_dwarf_cfi(mod_: *mut Dwfl_Module, bias: *mut Dwarf_Addr) -> *mut Dwarf_CFI;

    pub fn dwfl_module_eh_cfi(mod_: *mut Dwfl_Module, bias: *mut Dwarf_Addr) -> *mut Dwarf_CFI;

    pub fn dwfl_attach_state(
        dwfl: *mut Dwfl,
        elf: *mut Elf,
        pid: pid_t,
        thread_callbacks: *const Dwfl_Thread_Callbacks,
        dwfl_arg: *mut c_void,
    ) -> bool;

    pub fn dwfl_core_file_attach(dwfl: *mut Dwfl, elf: *mut Elf) -> c_int;

    pub fn dwfl_linux_proc_attach(
        dwfl: *mut Dwfl,
        pid: pid_t,
        assume_ptrace_stopped: bool,
    ) -> c_int;

    pub fn dwfl_pid(dwfl: *mut Dwfl) -> pid_t;

    pub fn dwfl_thread_dwfl(thread: *mut Dwfl_Thread) -> *mut Dwfl;

    pub fn dwfl_thread_tid(thread: *mut Dwfl_Thread) -> pid_t;

    pub fn dwfl_frame_thread(state: *mut Dwfl_Frame) -> *mut Dwfl_Thread;

    pub fn dwfl_thread_state_registers(
        thread: *mut Dwfl_Thread,
        firstreg: c_int,
        nregs: c_uint,
        regs: *const Dwarf_Word,
    ) -> bool;

    pub fn dwfl_thread_state_register_pc(thread: *mut Dwfl_Thread, pc: Dwarf_Word);

    pub fn dwfl_getthreads(
        dwfl: *mut Dwfl,
        callback: Option<unsafe extern "C" fn(thread: *mut Dwfl_Thread, arg: *mut c_void) -> c_int>,
        arg: *mut c_void,
    ) -> c_int;

    pub fn dwfl_thread_getframes(
        thread: *mut Dwfl_Thread,
        callback: Option<unsafe extern "C" fn(state: *mut Dwfl_Frame, arg: *mut c_void) -> c_int>,
        arg: *mut c_void,
    ) -> c_int;

    pub fn dwfl_getthread_frames(
        dwl: *mut Dwfl,
        tid: pid_t,
        callback: Option<unsafe extern "C" fn(state: *mut Dwfl_Frame, arg: *mut c_void) -> c_int>,
        arg: *mut c_void,
    ) -> c_int;

    pub fn dwfl_frame_pc(
        state: *mut Dwfl_Frame,
        pc: *mut Dwarf_Addr,
        isactivation: *mut bool,
    ) -> bool;
}
