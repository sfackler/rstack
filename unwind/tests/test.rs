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
