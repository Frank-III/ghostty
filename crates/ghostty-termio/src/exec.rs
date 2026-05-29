//! Subprocess lifecycle for termio (`src/termio/Exec.zig` subset).
//!
//! Owns child PID watching and exit reporting; PTY I/O remains in [`crate::harness`].

use crate::command::{process_exit_from_wait_status, CommandSpec, ProcessExit};
use crate::spawn::{spawn_pty_command, SpawnPtyError};
use crate::winsize::Winsize;

/// Non-blocking child exit watcher (`Exec` subprocess exit path).
#[derive(Debug)]
pub struct ChildWatcher {
    pid: libc::pid_t,
    /// Exit code after reap; delivered at most once via [`Self::poll_exit`].
    reaped_exit: Option<u32>,
    delivered: bool,
}

impl ChildWatcher {
    pub fn new(pid: libc::pid_t) -> Self {
        Self {
            pid,
            reaped_exit: None,
            delivered: false,
        }
    }

    pub fn pid(&self) -> libc::pid_t {
        self.pid
    }

    /// Poll for child exit with `WNOHANG`. Returns `Some(exit_code)` at most once.
    pub fn poll_exit(&mut self) -> Option<u32> {
        if self.delivered || self.pid <= 0 {
            return None;
        }
        if self.reaped_exit.is_none() {
            let mut status: i32 = 0;
            let result = unsafe { libc::waitpid(self.pid, &mut status, libc::WNOHANG) };
            if result > 0 {
                self.reaped_exit = Some(exit_code_from_status(status));
            } else if result < 0 {
                self.delivered = true;
                return None;
            }
        }
        if let Some(code) = self.reaped_exit {
            self.delivered = true;
            return Some(code);
        }
        None
    }

    /// Send SIGTERM then SIGKILL if the child is still running.
    pub fn terminate(&mut self) {
        if self.delivered || self.pid <= 0 || self.reaped_exit.is_some() {
            return;
        }
        let _ = unsafe { libc::kill(self.pid, libc::SIGTERM) };
        let mut status: i32 = 0;
        for _ in 0..20 {
            let result = unsafe { libc::waitpid(self.pid, &mut status, libc::WNOHANG) };
            if result > 0 {
                self.reaped_exit = Some(exit_code_from_status(status));
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let _ = unsafe { libc::kill(self.pid, libc::SIGKILL) };
        let result = unsafe { libc::waitpid(self.pid, &mut status, 0) };
        if result > 0 {
            self.reaped_exit = Some(exit_code_from_status(status));
        }
    }
}

fn exit_code_from_status(status: i32) -> u32 {
    match process_exit_from_wait_status(status) {
        ProcessExit::Exited(code) => u32::from(code),
        ProcessExit::Signaled(sig) => sig,
        ProcessExit::Stopped(sig) => sig,
        ProcessExit::Unknown(raw) => raw,
    }
}

/// Spawn result: PTY master fd wrapper + child watcher.
#[cfg(unix)]
pub struct ExecSpawn {
    pub pty: crate::pty::PosixPty,
    pub child: ChildWatcher,
}

#[cfg(unix)]
impl ExecSpawn {
    pub fn spawn(spec: &CommandSpec, winsize: Winsize) -> Result<Self, SpawnPtyError> {
        let (pty, pid) = spawn_pty_command(spec, winsize)?;
        Ok(Self {
            pty,
            child: ChildWatcher::new(pid),
        })
    }

    pub fn pid(&self) -> libc::pid_t {
        self.child.pid()
    }
}

#[cfg(all(unix, test))]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::command::CommandBuilder;

    #[test]
    fn child_exit_reported_once() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("exit 42")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut exec = ExecSpawn::spawn(&spec, winsize).expect("spawn");
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(code) = exec.child.poll_exit() {
                assert_eq!(code, 42);
                assert!(exec.child.poll_exit().is_none());
                return;
            }
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("expected child exit");
    }

    #[test]
    fn terminate_reaps_short_lived_shell() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("sleep 60")
            .build()
            .expect("spec");
        let winsize = Winsize {
            cols: 80,
            rows: 24,
            x_pixels: 0,
            y_pixels: 0,
        };
        let mut exec = ExecSpawn::spawn(&spec, winsize).expect("spawn");
        exec.child.terminate();
        assert!(exec.child.poll_exit().is_some());
        assert!(exec.child.poll_exit().is_none());
    }
}
