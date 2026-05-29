//! Production termio session (`Termio.zig`) via background thread + event drain.

use ghostty_foundation::FoundationResult;

use crate::command::CommandSpec;
use crate::spawn::SpawnPtyError;
use crate::termio::TermioMessage;
use crate::thread::{TermioThreadEvent, TermioThreadHandle};
use crate::winsize::Winsize;
use crate::TermioSink;

/// PTY-backed termio with a background I/O thread (production path).
pub struct TermioLoop {
    thread: TermioThreadHandle,
    winsize: Winsize,
}

impl TermioLoop {
    pub fn spawn(spec: &CommandSpec, winsize: Winsize) -> Result<Self, SpawnPtyError> {
        let thread = TermioThreadHandle::spawn(spec, winsize)?;
        Ok(Self { thread, winsize })
    }

    pub fn winsize(&self) -> Winsize {
        self.winsize
    }

    pub fn pid(&self) -> libc::pid_t {
        self.thread.pid()
    }

    pub fn is_shutdown(&self) -> bool {
        self.thread.is_shutdown()
    }

    pub fn push(&mut self, msg: TermioMessage) -> FoundationResult<()> {
        if let TermioMessage::Resize { cols, rows } = &msg {
            self.winsize = Winsize {
                cols: *cols,
                rows: *rows,
                x_pixels: self.winsize.x_pixels,
                y_pixels: self.winsize.y_pixels,
            };
        }
        self.thread.push(msg)
    }

    /// Drain thread events into `sink`; returns PTY bytes delivered to the sink.
    pub fn tick(&mut self, sink: &mut dyn TermioSink) -> FoundationResult<usize> {
        let mut bytes = 0usize;
        while let Some(event) = self.thread.try_recv_event() {
            match event {
                TermioThreadEvent::PtyOutput(data) => {
                    bytes += data.len();
                    sink.write_terminal(&data);
                }
                TermioThreadEvent::ResizeAck { cols, rows } => {
                    sink.resize_terminal(cols, rows);
                }
                TermioThreadEvent::SetTitle(_) | TermioThreadEvent::RedrawRequested => {}
                TermioThreadEvent::ChildExit(_) => {}
            }
        }
        Ok(bytes)
    }

    pub fn poll_child_exit(&mut self) -> Option<u32> {
        self.thread.poll_child_exit()
    }

    pub fn shutdown(&mut self) -> FoundationResult<()> {
        self.push(TermioMessage::Shutdown)
    }
}
