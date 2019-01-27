use std::env;
use std::process::Command;
use std::thread;

fn thread_2() {
    loop {
        thread::park();
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() > 1 {
        let _ = rstack_self::child();
        return;
    }

    thread::Builder::new()
        .name("thread 2".to_string())
        .spawn(thread_2)
        .unwrap();

    let exe = env::current_exe().unwrap();
    let trace = rstack_self::trace(Command::new(exe).arg("child")).unwrap();

    for thread in trace.threads() {
        println!("{} - {}", thread.id(), thread.name());
        for frame in thread.frames() {
            println!("{:#016x}", frame.ip());

            for symbol in frame.symbols() {
                print!("    - {}", symbol.name().unwrap_or("????"));
                if let Some(file) = symbol.file() {
                    print!(" {}:{}", file.display(), symbol.line().unwrap_or(0));
                }
                println!();
            }
        }
        println!();
    }
}
