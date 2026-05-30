//! Production termio session (`Termio.zig`) via background thread + event drain.

use ghostty_config::DerivedStreamConfig;
use ghostty_foundation::FoundationResult;

use crate::command::CommandSpec;
use crate::spawn::SpawnPtyError;
use crate::stream_handler::StreamHandler;
use crate::surface_mailbox::SurfaceMessage;
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
    stream: StreamHandler,
}

impl TermioLoop {
    pub fn spawn(
        spec: &CommandSpec,
        winsize: Winsize,
        stream_config: DerivedStreamConfig,
    ) -> Result<Self, SpawnPtyError> {
        let thread = TermioThreadHandle::spawn(spec, winsize)?;
        Ok(Self {
            thread,
            winsize,
            stream: StreamHandler::new(stream_config),
        })
    }

    pub fn stream_handler(&self) -> &StreamHandler {
        &self.stream
    }

    pub fn stream_handler_mut(&mut self) -> &mut StreamHandler {
        &mut self.stream
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
                    drain.set_title = Some(title.clone());
                    let _ = self.stream.on_set_title(title);
                }
                TermioThreadEvent::RedrawRequested => {
                    drain.redraw_requested = true;
                    let _ = self.stream.on_redraw_requested();
                }
                TermioThreadEvent::ChildExit(exit_code) => {
                    let _ = self
                        .stream
                        .send_surface(SurfaceMessage::ChildExited { exit_code });
                }
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

    /// Drain surface-thread messages queued by the stream handler.
    pub fn drain_surface_mailbox(&mut self) -> Vec<SurfaceMessage> {
        let mut out = Vec::new();
        while let Some(msg) = self.stream.surface_mailbox().pop() {
            out.push(msg);
        }
        out
    }

    /// Wire rust-owned VT side effects to this termio's stream handler and PTY write path.
    #[cfg(feature = "rust-vt")]
    pub fn bind_rust_terminal(
        &mut self,
        terminal: &mut crate::vt_sink::rust_owned::RustOwnedTerminalSink,
    ) {
        let termio_ptr = self as *mut Self;
        // SAFETY: `attach_vt_bridge` only reads `stream` then stores `termio` as a raw pointer.
        unsafe {
            terminal.attach_vt_bridge(&mut (*termio_ptr).stream, &mut *termio_ptr);
        }
    }
}
