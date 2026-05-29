//! Surface thread mailbox (`src/apprt/surface.zig` `Message` subset).

use ghostty_foundation::{FoundationError, FoundationResult};

/// Messages delivered to the surface / apprt thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceMessage {
    SetTitle(String),
    RedrawRequested,
    Close,
    ChildExited { exit_code: u32 },
    ReportTitle,
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
