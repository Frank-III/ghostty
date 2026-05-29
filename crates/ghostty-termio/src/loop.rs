//! Production termio pump loop (`src/termio/Termio.zig` drain/pump cycle).
//!
//! Wraps [`TermioHarness`] for the synchronous path used by [`ghostty_core::SurfaceSession`].

use ghostty_foundation::FoundationResult;

use crate::command::CommandSpec;
use crate::harness::{TermioHarness, TermioSink};
use crate::spawn::SpawnPtyError;
use crate::termio::TermioMessage;
use crate::winsize::Winsize;

/// PTY-backed termio session with child exit reporting from [`crate::exec`].
pub struct TermioLoop {
    harness: TermioHarness,
}

impl TermioLoop {
    pub fn spawn(spec: &CommandSpec, winsize: Winsize) -> Result<Self, SpawnPtyError> {
        TermioHarness::spawn(spec, winsize).map(|harness| Self { harness })
    }

    pub fn harness(&self) -> &TermioHarness {
        &self.harness
    }

    pub fn harness_mut(&mut self) -> &mut TermioHarness {
        &mut self.harness
    }

    pub fn pid(&self) -> libc::pid_t {
        self.harness.pid()
    }

    pub fn winsize(&self) -> Winsize {
        self.harness.winsize()
    }

    pub fn is_shutdown(&self) -> bool {
        self.harness.is_shutdown()
    }

    pub fn push(&mut self, msg: TermioMessage) -> FoundationResult<()> {
        self.harness.mailbox_mut().push(msg)
    }

    pub fn tick(&mut self, sink: &mut dyn TermioSink) -> FoundationResult<usize> {
        self.harness.drain_mailbox(sink)?;
        self.harness
            .pump_pty(sink)
            .map_err(|_| ghostty_foundation::FoundationError::Unsupported)
    }

    pub fn poll_child_exit(&mut self) -> Option<u32> {
        self.harness.poll_child_exit()
    }

    pub fn shutdown(&mut self, sink: &mut dyn TermioSink) -> FoundationResult<()> {
        self.push(TermioMessage::Shutdown)?;
        self.harness.drain_mailbox(sink)
    }
}

impl Drop for TermioLoop {
    fn drop(&mut self) {
        self.harness.terminate_child();
    }
}
