//! Unit testing aid for the rstack library.
//!
//! Provides launcher for a dummy child process which stack can be captured.
use std::io;
use std::process::{Child, Command, Stdio};

include!(concat!(env!("OUT_DIR"), "/target.rs"));

pub struct CaptureTarget {
    process: Child,
}

impl CaptureTarget {
    /// Starts new capture target process.
    pub fn start() -> std::io::Result<Self> {
        let child = Command::new(get_path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Self { process: child })
    }

    /// Stops the capture-target process.
    pub fn stop(mut self) -> io::Result<()> {
        self.process.kill()?;
        let _ = self.process.try_wait()?;
        Ok(())
    }

    pub fn process_id(&self) -> u32 {
        self.process.id()
    }
}

impl Drop for CaptureTarget {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.try_wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_and_stopping_target_succeeds() {
        let target = CaptureTarget::start().expect("Starting capture-target failed.");
        assert!(target.process_id() > 0);
        target.stop().expect("Stopping capture-target failed.");
    }
}
