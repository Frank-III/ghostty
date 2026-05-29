//! Synchronous termio harness for tests only (no background thread).
//!
//! Production code uses [`crate::TermioLoop`].

use ghostty_foundation::FoundationResult;

use crate::command::CommandSpec;
#[cfg(unix)]
use crate::exec::ExecSpawn;
use crate::stream::PtyStream;
use crate::termio::{TermioMailbox, TermioMessage};
use crate::winsize::Winsize;

/// Bytes read from the PTY are fed here (terminal emulator, collector, etc.).
pub trait TermioSink {
    fn write_terminal(&mut self, bytes: &[u8]);
    fn resize_terminal(&mut self, cols: u16, rows: u16);
}

impl TermioSink for Vec<u8> {
    fn write_terminal(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn resize_terminal(&mut self, _cols: u16, _rows: u16) {}
}

/// Synchronous PTY harness for unit/integration tests.
pub struct TermioHarness {
    stream: PtyStream,
    #[cfg(unix)]
    child: crate::exec::ChildWatcher,
    mailbox: TermioMailbox,
}

impl TermioHarness {
    pub fn spawn(
        spec: &CommandSpec,
        winsize: Winsize,
    ) -> Result<Self, crate::spawn::SpawnPtyError> {
        let exec = ExecSpawn::spawn(spec, winsize)?;
        exec.pty
            .set_nonblocking(true)
            .map_err(|_| crate::spawn::SpawnPtyError::SpawnFailed)?;
        Ok(Self {
            stream: PtyStream::new(exec.pty, winsize),
            child: exec.child,
            mailbox: TermioMailbox::new(64),
        })
    }

    pub fn mailbox(&self) -> &TermioMailbox {
        &self.mailbox
    }

    pub fn mailbox_mut(&mut self) -> &mut TermioMailbox {
        &mut self.mailbox
    }

    pub fn pid(&self) -> libc::pid_t {
        self.child.pid()
    }

    pub fn poll_child_exit(&mut self) -> Option<u32> {
        self.child.poll_exit()
    }

    pub fn terminate_child(&mut self) {
        self.child.terminate();
    }

    pub fn winsize(&self) -> Winsize {
        self.stream.winsize()
    }

    pub fn is_shutdown(&self) -> bool {
        self.stream.is_shutdown()
    }

    pub fn drain_mailbox(&mut self, sink: &mut dyn TermioSink) -> FoundationResult<()> {
        while let Some(msg) = self.mailbox.pop() {
            if matches!(msg, TermioMessage::SetTitle(_)) {
                continue;
            }
            if matches!(msg, TermioMessage::RedrawRequested) {
                continue;
            }
            self.stream.apply_message(&msg, sink)?;
        }
        Ok(())
    }

    pub fn pump_pty(&mut self, sink: &mut dyn TermioSink) -> Result<usize, crate::pty::PtyIoError> {
        self.stream.pump_into(sink)
    }

    pub fn poll_readable(&self, timeout_ms: i32) -> Result<bool, crate::pty::PtyIoError> {
        self.stream.poll_readable(timeout_ms)
    }

    pub fn run_until<F>(
        &mut self,
        sink: &mut dyn TermioSink,
        deadline: std::time::Instant,
        mut done: F,
    ) -> FoundationResult<()>
    where
        F: FnMut(&mut Self, &mut dyn TermioSink) -> bool,
    {
        while !self.is_shutdown() {
            self.drain_mailbox(sink)?;
            let _ = self
                .pump_pty(sink)
                .map_err(|_| ghostty_foundation::FoundationError::Unsupported)?;
            if done(self, sink) {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            let timeout_ms = remaining.as_millis().min(i32::MAX as u128) as i32;
            if !self
                .poll_readable(timeout_ms)
                .map_err(|_| ghostty_foundation::FoundationError::Unsupported)?
            {
                continue;
            }
        }
        Ok(())
    }
}

impl Drop for TermioHarness {
    fn drop(&mut self) {
        self.terminate_child();
    }
}

#[cfg(all(unix, test))]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::command::CommandBuilder;

    #[test]
    fn mailbox_write_reaches_pty_child() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("cat")
            .build()
            .expect("command spec");

        let winsize = Winsize {
            rows: 24,
            cols: 80,
            x_pixels: 0,
            y_pixels: 0,
        };

        let mut harness = TermioHarness::spawn(&spec, winsize).expect("spawn harness");
        harness
            .mailbox_mut()
            .push(TermioMessage::Write(b"ping\n".to_vec()))
            .expect("queue write");

        let mut out = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(2);
        harness
            .run_until(&mut out, deadline, |h, sink| {
                h.drain_mailbox(sink).ok();
                false
            })
            .expect("run loop");

        assert!(out.windows(4).any(|w| w == b"ping"), "output={out:?}");

        harness
            .mailbox_mut()
            .push(TermioMessage::Shutdown)
            .expect("shutdown");
        harness.drain_mailbox(&mut out).expect("drain shutdown");
        assert!(harness.is_shutdown());
    }

    #[test]
    fn child_output_reaches_sink() {
        let spec = CommandBuilder::new()
            .path("/bin/sh")
            .arg("sh")
            .arg("-c")
            .arg("printf 'termio-e2e'")
            .build()
            .expect("command spec");

        let winsize = Winsize {
            rows: 24,
            cols: 80,
            x_pixels: 0,
            y_pixels: 0,
        };

        let mut harness = TermioHarness::spawn(&spec, winsize).expect("spawn harness");
        let mut out = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(2);
        harness
            .run_until(&mut out, deadline, |_h, _sink| false)
            .expect("run loop");

        assert!(
            out.windows(10).any(|w| w == b"termio-e2e"),
            "output={out:?}"
        );
    }
}
