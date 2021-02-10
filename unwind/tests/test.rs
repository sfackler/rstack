#![cfg_attr(target_arch = "aarch64", feature(asm))]

use unwind::{get_context, Cursor, RegNum};

#[test]
fn local() {
    fn bar() {
        get_context!(context);
        let mut cursor = Cursor::local(context).unwrap();

        loop {
            let ip = cursor.register(RegNum::IP).unwrap();

            match (cursor.procedure_info(), cursor.procedure_name()) {
                (Ok(ref info), Ok(ref name)) if ip == info.start_ip() + name.offset() => {
                    println!(
                        "{:#016x} - {} ({:#016x}) + {:#x}",
                        ip,
                        name.name(),
                        info.start_ip(),
                        name.offset(),
                    );
                }
                _ => {
                    println!("{:#016x} - ????", ip);
                }
            }

            if !cursor.step().unwrap() {
                break;
            }
        }
    }

    fn foo() {
        bar();
    }
    foo();
}

#[test]
#[cfg(target_arch = "x86_64")]
fn x86_64() {
    macro_rules! dump_register {
        ($cursor:ident, $reg_num:expr, $reg_name:literal) => {
            if let Ok(reg_val) = $cursor.register($reg_num) {
                println!("{}: {:#016x}", $reg_name, reg_val);
            } else {
                println!("{} not stored!", $reg_name);
            }
        };
    }

    fn bar() {
        get_context!(context);
        let mut cursor = Cursor::local(context).unwrap();

        loop {
            if let Ok(procedure_name) = cursor.procedure_name() {
                println!("{}:", procedure_name.name());
            } else {
                println!("unknown:")
            }

            dump_register!(cursor, RegNum::RAX, "rax");
            dump_register!(cursor, RegNum::RDX, "rdx");
            dump_register!(cursor, RegNum::RCX, "rcx");
            dump_register!(cursor, RegNum::RBX, "rbx");
            dump_register!(cursor, RegNum::RSI, "rsi");
            dump_register!(cursor, RegNum::RDI, "rdi");
            dump_register!(cursor, RegNum::RBP, "rbp");
            dump_register!(cursor, RegNum::RSP, "rsp");
            dump_register!(cursor, RegNum::R8, "r8");
            dump_register!(cursor, RegNum::R9, "r9");
            dump_register!(cursor, RegNum::R10, "r10");
            dump_register!(cursor, RegNum::R11, "r11");
            dump_register!(cursor, RegNum::R12, "r12");
            dump_register!(cursor, RegNum::R13, "r13");
            dump_register!(cursor, RegNum::R14, "r14");
            dump_register!(cursor, RegNum::R15, "r15");
            dump_register!(cursor, RegNum::RIP, "rip");
            dump_register!(cursor, RegNum::CFA, "cfa");
            println!();

            if !cursor.step().unwrap() {
                break;
            }
        }
    }

    fn foo() {
        bar();
    }

    foo();
}

#[test]
#[cfg(feature = "ptrace")]
fn remote() {
    use std::io;
    use std::process::Command;
    use std::ptr;
    use std::thread;
    use std::time::Duration;
    use unwind::{Accessors, AddressSpace, Byteorder, PTraceState};

    let mut child = Command::new("sleep").arg("10").spawn().unwrap();
    thread::sleep(Duration::from_millis(10));
    unsafe {
        let ret = libc::ptrace(
            libc::PTRACE_ATTACH,
            child.id() as libc::pid_t,
            ptr::null_mut::<libc::c_void>(),
            ptr::null_mut::<libc::c_void>(),
        );
        if ret != 0 {
            panic!("{}", io::Error::last_os_error());
        }

        loop {
            let mut status = 0;
            let ret = libc::waitpid(child.id() as libc::pid_t, &mut status, 0);
            if ret < 0 {
                panic!("{}", io::Error::last_os_error());
            }
            if libc::WIFSTOPPED(status) {
                break;
            }
        }
    }
    let state = PTraceState::new(child.id() as _).unwrap();
    let address_space = AddressSpace::new(Accessors::ptrace(), Byteorder::DEFAULT).unwrap();
    let mut cursor = Cursor::remote(&address_space, &state).unwrap();

    loop {
        let ip = cursor.register(RegNum::IP).unwrap();

        match (cursor.procedure_info(), cursor.procedure_name()) {
            (Ok(ref info), Ok(ref name)) if ip == info.start_ip() + name.offset() => {
                println!(
                    "{:#016x} - {} ({:#016x}) + {:#x}",
                    ip,
                    name.name(),
                    info.start_ip(),
                    name.offset(),
                );
            }
            _ => println!("{:#016x} - ????", ip),
        }

        if !cursor.step().unwrap() {
            break;
        }
    }
    child.kill().unwrap();
}
