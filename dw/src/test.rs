use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::dwfl::{Dwfl, DwflCallbacks, Error, FindDebuginfo, FindElf, FrameRef};

fn frame_callback(frame: &mut FrameRef) -> Result<(), Error> {
    let mut activation = false;
    let mut ip = frame.pc(Some(&mut activation))?;
    if !activation {
        ip -= 1;
    }

    match frame
        .thread()
        .dwfl()
        .addr_module(ip)
        .and_then(|module| module.addr_info(ip))
    {
        Some(info) => println!(
            "    {:#016x} - {} ({:#016x}) + {:#x}",
            ip,
            info.name().to_string_lossy(),
            info.bias() + info.symbol().value(),
            info.offset()
        ),
        None => println!("    {:#016x} - ????", ip),
    }

    Ok(())
}

#[test]
fn trace_sleep() {
    let child = Command::new("sleep").arg("10").spawn().unwrap();
    thread::sleep(Duration::from_millis(10));

    let callbacks = DwflCallbacks::new(FindElf::LINUX_PROC, FindDebuginfo::STANDARD);
    let mut dwfl = Dwfl::begin(&callbacks).unwrap();
    dwfl.report().linux_proc(child.id()).unwrap();
    dwfl.linux_proc_attach(child.id(), false).unwrap();

    dwfl.threads(|thread| {
        println!("thread {}", thread.tid());
        thread.frames(frame_callback)
    })
    .unwrap();
}

#[test]
fn trace_sleep_thread() {
    let child = Command::new("sleep").arg("10").spawn().unwrap();
    thread::sleep(Duration::from_millis(10));

    let callbacks = DwflCallbacks::new(FindElf::LINUX_PROC, FindDebuginfo::STANDARD);
    let mut dwfl = Dwfl::begin(&callbacks).unwrap();
    dwfl.report().linux_proc(child.id()).unwrap();
    dwfl.linux_proc_attach(child.id(), false).unwrap();

    dwfl.thread_frames(child.id(), frame_callback).unwrap();
}
