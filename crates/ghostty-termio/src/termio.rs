//! Termio thread mailbox (Phase 3 skeleton).
//!
//! Mirrors the Zig `Termio` message loop shape; full PTY/renderer wiring is deferred.

use ghostty_foundation::{FoundationError, FoundationResult};

/// Messages delivered to the termio thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermioMessage {
    Write(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Shutdown,
    /// Renderer/surface bridge placeholder (`Termio.zig` surface mailbox).
    RedrawRequested,
    /// Title change from child process (`Exec` → surface).
    SetTitle(String),
}

/// Fixed-capacity mailbox for termio messages.
pub struct TermioMailbox {
    queue: std::collections::VecDeque<TermioMessage>,
    capacity: usize,
}

impl TermioMailbox {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: std::collections::VecDeque::with_capacity(capacity.min(64)),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, msg: TermioMessage) -> FoundationResult<()> {
        if self.queue.len() >= self.capacity {
            return Err(FoundationError::OutOfMemory);
        }
        self.queue.push_back(msg);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<TermioMessage> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{TermioMailbox, TermioMessage};

    #[test]
    fn mailbox_fifo() {
        let mut mb = TermioMailbox::new(4);
        mb.push(TermioMessage::Write(b"hi".to_vec())).unwrap();
        mb.push(TermioMessage::Resize { cols: 80, rows: 24 })
            .unwrap();
        assert_eq!(
            mb.pop(),
            Some(TermioMessage::Write(b"hi".to_vec()))
        );
        assert_eq!(
            mb.pop(),
            Some(TermioMessage::Resize { cols: 80, rows: 24 })
        );
    }
}
