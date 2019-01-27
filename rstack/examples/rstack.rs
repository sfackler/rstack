use std::env;
use std::process;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <pid>", args[0]);
        process::exit(1);
    }

    let pid = match args[1].parse() {
        Ok(pid) => pid,
        Err(e) => {
            eprintln!("error parsing PID: {}", e);
            process::exit(1);
        }
    };

    let process = match rstack::trace(pid) {
        Ok(threads) => threads,
        Err(e) => {
            eprintln!("error tracing threads: {}", e);
            process::exit(1);
        }
    };

    for thread in process.threads() {
        println!(
            "thread {} - {}",
            thread.id(),
            thread.name().unwrap_or("<unknown>")
        );
        for frame in thread.frames() {
            match frame.symbol() {
                Some(symbol) => println!(
                    "{:#016x} - {} + {:#x}",
                    frame.ip(),
                    symbol.name(),
                    symbol.offset(),
                ),
                None => println!("{:#016x} - ???", frame.ip()),
            }
        }
        println!();
    }
}
