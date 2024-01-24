use std::time::Duration;

fn main() {
    // Keep running until target-launcher forcibly kills this process.
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
