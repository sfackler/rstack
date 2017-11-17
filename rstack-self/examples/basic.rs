extern crate rstack_self;

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
        rstack_self::child();
        return;
    }

    thread::Builder::new()
        .name("thread 2".to_string())
        .spawn(thread_2)
        .unwrap();

    let exe = env::current_exe().unwrap();
    let trace = rstack_self::trace(Command::new(exe).arg("child")).unwrap();

    for thread in trace {
        println!("{} - {}", thread.id, thread.name);
        for frame in thread.frames {
            print!("{:#016x}", frame.ip);
            if let Some(path) = frame.library {
                print!(" ({})", path.display());
            }
            println!();

            for symbol in frame.symbols {
                print!("    - {}", symbol.name.unwrap_or("????".to_string()));
                if let Some(file) = symbol.file {
                    print!(" {}:{}", file.display(), symbol.line.unwrap_or(0));
                }
                println!();
            }
        }
        println!();
    }
}
