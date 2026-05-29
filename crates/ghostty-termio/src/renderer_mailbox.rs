//! Renderer thread mailbox (`src/renderer/message.zig` subset).

use ghostty_foundation::{FoundationError, FoundationResult};
use ghostty_renderer::Size;

/// Messages delivered to the renderer thread.
#[derive(Debug, Clone, PartialEq)]
pub enum RendererMessage {
    Redraw,
    Focus(bool),
    Visible(bool),
    Resize(Size),
    ResetCursorBlink,
}

/// Fixed-capacity mailbox for renderer messages.
#[derive(Debug, Default)]
pub struct RendererMailbox {
    queue: std::collections::VecDeque<RendererMessage>,
    capacity: usize,
}

impl RendererMailbox {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: std::collections::VecDeque::with_capacity(capacity.min(64)),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, msg: RendererMessage) -> FoundationResult<()> {
        if self.queue.len() >= self.capacity {
            return Err(FoundationError::OutOfMemory);
        }
        self.queue.push_back(msg);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<RendererMessage> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghostty_renderer::{CellSize, Padding, ScreenSize, Size};

    #[test]
    fn resize_message() {
        let mut mb = RendererMailbox::new(2);
        let size = Size {
            screen: ScreenSize {
                width: 100,
                height: 50,
            },
            cell: CellSize {
                width: 8,
                height: 16,
            },
            padding: Padding::default(),
        };
        mb.push(RendererMessage::Resize(size)).unwrap();
        assert!(matches!(mb.pop(), Some(RendererMessage::Resize(_))));
    }
}
