use std::process::Command;

use crate::dwfl::{Dwfl, DwflCallbacks, FindDebuginfo, FindElf};

#[test]
fn trace_self() {
    let child = Command::new("sleep").arg("10").spawn().unwrap();

    let callbacks = DwflCallbacks::new(FindElf::LINUX_PROC, FindDebuginfo::STANDARD);
    let mut dwfl = Dwfl::begin(&callbacks).unwrap();
    dwfl.report().linux_proc(child.id()).unwrap();
    dwfl.linux_proc_attach(child.id(), false).unwrap();

    dwfl.threads(|thread| {
        println!("thread {}", thread.tid());

        let _ = thread.frames(|frame| {
            let mut activation = false;
            let mut ip = frame.pc(Some(&mut activation))?;
            if !activation {
                ip -= 1;
            }

            match frame
                .thread()
                .dwfl()
                .addr_module(ip)
                .and_then(|module| module.addr_name(ip))
            {
                Some(name) => println!("    {:#016x} - {}", ip, name),
                None => println!("    {:#016x} - ????", ip),
            }

            Ok(())
        });

        Ok(())
    })
    .unwrap();
}
