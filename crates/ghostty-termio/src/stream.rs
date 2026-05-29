//! PTY byte stream: mailbox messages → PTY I/O → terminal sink.
//!
//! Port target: `src/termio/stream_handler.zig` read/write path (without full parser).

use ghostty_foundation::FoundationResult;

use crate::pty::{PosixPty, PtyIoError};
use crate::termio::TermioMessage;
use crate::winsize::Winsize;
use crate::TermioSink;

/// Non-blocking PTY pump used by the termio harness and background thread.
pub struct PtyStream {
    pty: PosixPty,
    winsize: Winsize,
    shutdown: bool,
}

impl PtyStream {
    pub fn new(pty: PosixPty, winsize: Winsize) -> Self {
        Self {
            pty,
            winsize,
            shutdown: false,
        }
    }

    pub fn winsize(&self) -> Winsize {
        self.winsize
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub fn pty(&self) -> &PosixPty {
        &self.pty
    }

    pub fn pty_mut(&mut self) -> &mut PosixPty {
        &mut self.pty
    }

    /// Apply one mailbox message (Write → PTY, Resize → PTY + sink, etc.).
    pub fn apply_message(
        &mut self,
        msg: &TermioMessage,
        sink: &mut dyn TermioSink,
    ) -> FoundationResult<()> {
        match msg {
            TermioMessage::Write(data) => {
                self.pty
                    .write(data)
                    .map_err(|_| ghostty_foundation::FoundationError::Unsupported)?;
            }
            TermioMessage::Resize { cols, rows } => {
                let size = Winsize {
                    cols: *cols,
                    rows: *rows,
                    x_pixels: self.winsize.x_pixels,
                    y_pixels: self.winsize.y_pixels,
                };
                self.pty
                    .set_size(size)
                    .map_err(|_| ghostty_foundation::FoundationError::Unsupported)?;
                self.winsize = size;
                sink.resize_terminal(*cols, *rows);
            }
            TermioMessage::Shutdown => {
                self.shutdown = true;
            }
            TermioMessage::RedrawRequested | TermioMessage::SetTitle(_) => {}
            TermioMessage::ResizeAck { cols, rows } => {
                sink.resize_terminal(*cols, *rows);
            }
        }
        Ok(())
    }

    /// Drain a batch of messages.
    pub fn apply_messages<I>(
        &mut self,
        messages: I,
        sink: &mut dyn TermioSink,
    ) -> FoundationResult<()>
    where
        I: IntoIterator<Item = TermioMessage>,
    {
        for msg in messages {
            self.apply_message(&msg, sink)?;
        }
        Ok(())
    }

    /// Read available PTY output into the terminal sink.
    pub fn pump_into(&mut self, sink: &mut dyn TermioSink) -> Result<usize, PtyIoError> {
        let mut buf = [0u8; 4096];
        let n = self.pty.read(&mut buf)?;
        if n > 0 {
            sink.write_terminal(&buf[..n]);
        }
        Ok(n)
    }

    pub fn poll_readable(&self, timeout_ms: i32) -> Result<bool, PtyIoError> {
        self.pty.poll_readable(timeout_ms)
    }
}
