//! Production termio session (`Termio.zig`) via background thread + event drain.

use ghostty_foundation::FoundationResult;

use crate::command::CommandSpec;
use crate::spawn::SpawnPtyError;
use crate::termio::TermioMessage;
use crate::thread::{TermioThreadEvent, TermioThreadHandle};
use crate::winsize::Winsize;
use crate::TermioSink;

/// Side effects from one termio drain besides PTY bytes into the terminal sink.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TermioDrain {
    pub pty_bytes: usize,
    pub set_title: Option<String>,
    pub redraw_requested: bool,
}

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

    /// Drain thread events into `sink`; returns PTY bytes and surface side effects.
    pub fn tick(&mut self, sink: &mut dyn TermioSink) -> FoundationResult<TermioDrain> {
        let mut drain = TermioDrain::default();
        while let Some(event) = self.thread.try_recv_event() {
            match event {
                TermioThreadEvent::PtyOutput(data) => {
                    drain.pty_bytes += data.len();
                    sink.write_terminal(&data);
                }
                TermioThreadEvent::ResizeAck { cols, rows } => {
                    sink.resize_terminal(cols, rows);
                }
                TermioThreadEvent::SetTitle(title) => {
                    drain.set_title = Some(title);
                }
                TermioThreadEvent::RedrawRequested => {
                    drain.redraw_requested = true;
                }
                TermioThreadEvent::ChildExit(_) => {}
            }
        }
        Ok(drain)
    }

    pub fn poll_child_exit(&mut self) -> Option<u32> {
        self.thread.poll_child_exit()
    }

    pub fn shutdown(&mut self) -> FoundationResult<()> {
        self.push(TermioMessage::Shutdown)
    }
}
