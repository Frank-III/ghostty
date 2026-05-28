//! Synchronous termio harness: mailbox → PTY writes, PTY reads → terminal sink.
//!
//! Mirrors the Zig `termio.Thread` drain/pump shape for tests; no xev thread yet.

use ghostty_foundation::FoundationResult;

use crate::command::CommandSpec;
use crate::pty::{PosixPty, PtyIoError};
use crate::spawn::{spawn_pty_command, SpawnPtyError};
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

/// Minimal PTY-backed termio session for integration tests.
pub struct TermioHarness {
    pty: PosixPty,
    pid: libc::pid_t,
    mailbox: TermioMailbox,
    winsize: Winsize,
    shutdown: bool,
}

impl TermioHarness {
    pub fn spawn(spec: &CommandSpec, winsize: Winsize) -> Result<Self, SpawnPtyError> {
        let (pty, pid) = spawn_pty_command(spec, winsize)?;
        pty.set_nonblocking(true).map_err(|_| SpawnPtyError::SpawnFailed)?;
        Ok(Self {
            pty,
            pid,
            mailbox: TermioMailbox::new(64),
            winsize,
            shutdown: false,
        })
    }

    pub fn mailbox(&self) -> &TermioMailbox {
        &self.mailbox
    }

    pub fn mailbox_mut(&mut self) -> &mut TermioMailbox {
        &mut self.mailbox
    }

    pub fn pid(&self) -> libc::pid_t {
        self.pid
    }

    pub fn winsize(&self) -> Winsize {
        self.winsize
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    /// Process queued mailbox messages (Write → PTY, Resize → PTY + sink).
    pub fn drain_mailbox(&mut self, sink: &mut dyn TermioSink) -> FoundationResult<()> {
        while let Some(msg) = self.mailbox.pop() {
            match msg {
                TermioMessage::Write(data) => {
                    self.pty
                        .write(&data)
                        .map_err(|_| ghostty_foundation::FoundationError::Unsupported)?;
                }
                TermioMessage::Resize { cols, rows } => {
                    let size = Winsize {
                        cols,
                        rows,
                        x_pixels: self.winsize.x_pixels,
                        y_pixels: self.winsize.y_pixels,
                    };
                    self.pty
                        .set_size(size)
                        .map_err(|_| ghostty_foundation::FoundationError::Unsupported)?;
                    self.winsize = size;
                    sink.resize_terminal(cols, rows);
                }
                TermioMessage::Shutdown => {
                    self.shutdown = true;
                }
            }
        }
        Ok(())
    }

    /// Read available PTY output and forward to the terminal sink.
    pub fn pump_pty(&mut self, sink: &mut dyn TermioSink) -> Result<usize, PtyIoError> {
        let mut buf = [0u8; 4096];
        let n = self.pty.read(&mut buf)?;
        if n > 0 {
            sink.write_terminal(&buf[..n]);
        }
        Ok(n)
    }

    /// Poll PTY, pump reads, and drain mailbox until `deadline` or shutdown.
    pub fn run_until<F>(
        &mut self,
        sink: &mut dyn TermioSink,
        deadline: std::time::Instant,
        mut done: F,
    ) -> FoundationResult<()>
    where
        F: FnMut(&mut Self, &mut dyn TermioSink) -> bool,
    {
        while !self.shutdown {
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
                .pty
                .poll_readable(timeout_ms)
                .map_err(|_| ghostty_foundation::FoundationError::Unsupported)?
            {
                continue;
            }
        }
        Ok(())
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
