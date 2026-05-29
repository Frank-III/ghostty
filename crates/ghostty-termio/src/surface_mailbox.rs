//! Surface thread mailbox (`src/apprt/surface.zig` `Message` subset).

use ghostty_foundation::{FoundationError, FoundationResult};

/// OSC 52 clipboard target (`apprt.Clipboard` subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardKind {
    Standard,
    Selection,
    Primary,
}

impl ClipboardKind {
    pub fn from_osc(kind: u8) -> Self {
        match kind {
            b's' => Self::Selection,
            b'p' => Self::Primary,
            _ => Self::Standard,
        }
    }
}

/// Messages delivered to the surface / apprt thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceMessage {
    SetTitle(String),
    RedrawRequested,
    Close,
    ChildExited { exit_code: u32 },
    ReportTitle,
    RingBell,
    ClipboardRead { clipboard: ClipboardKind },
    ClipboardWrite {
        clipboard: ClipboardKind,
        data: Vec<u8>,
    },
}

/// Fixed-capacity mailbox for surface messages.
#[derive(Debug, Default)]
pub struct SurfaceMailbox {
    queue: std::collections::VecDeque<SurfaceMessage>,
    capacity: usize,
}

impl SurfaceMailbox {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: std::collections::VecDeque::with_capacity(capacity.min(64)),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, msg: SurfaceMessage) -> FoundationResult<()> {
        if self.queue.len() >= self.capacity {
            return Err(FoundationError::OutOfMemory);
        }
        self.queue.push_back(msg);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<SurfaceMessage> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_set_title() {
        let mut mb = SurfaceMailbox::new(4);
        mb.push(SurfaceMessage::SetTitle("hi".into())).unwrap();
        assert_eq!(
            mb.pop(),
            Some(SurfaceMessage::SetTitle("hi".to_string()))
        );
    }
}
