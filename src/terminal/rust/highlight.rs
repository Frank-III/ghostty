use core::ffi::c_void;
use crate::early::*;
use crate::constants::*;
use crate::size_types::*;
use crate::page_list_types::PageListNode;

/// A stable reference to a specific cell position within the PageList.
/// Pins are tracked and automatically updated when pages change.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pin {
    pub node: *mut PageListNode,
    pub y: CellCountInt,
    pub x: CellCountInt,
    /// Flipped to `true` when the tracked page was pruned and the pin
    /// could not be moved to a meaningful location.
    pub garbage: bool,
}

impl Default for Pin {
    fn default() -> Self {
        Self {
            node: core::ptr::null_mut(),
            y: 0,
            x: 0,
            garbage: false,
        }
    }
}

impl Pin {
    pub fn eql(self, other: Pin) -> bool {
        self.node == other.node && self.x == other.x && self.y == other.y
    }
}

/// Opaque placeholder retained for ABI compatibility with older call sites.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PageNode {
    _opaque: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PageChunk {
    pub node: *mut PageListNode,
    pub start: CellCountInt,
    pub end: CellCountInt,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FlattenedChunk {
    pub node: *mut PageListNode,
    pub serial: u64,
    pub start: CellCountInt,
    pub end: CellCountInt,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HighlightUntracked {
    pub start: Pin,
    pub end: Pin,
}

impl HighlightUntracked {
    pub fn eql(self, other: HighlightUntracked) -> bool {
        self.start.eql(other.start) && self.end.eql(other.end)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HighlightTracked {
    pub start: *mut Pin,
    pub end: *mut Pin,
}

impl HighlightTracked {
    pub fn init_assume(start: *mut Pin, end: *mut Pin) -> Self {
        Self { start, end }
    }
}

#[repr(C)]
pub struct HighlightFlattened {
    pub chunks_ptr: *mut FlattenedChunk,
    pub chunks_len: usize,
    pub chunks_cap: usize,
    pub top_x: CellCountInt,
    pub bot_x: CellCountInt,
}

impl HighlightFlattened {
    pub fn empty() -> Self {
        Self {
            chunks_ptr: core::ptr::null_mut(),
            chunks_len: 0,
            chunks_cap: 0,
            top_x: 0,
            bot_x: 0,
        }
    }

    pub fn chunks(&self) -> &[FlattenedChunk] {
        if self.chunks_ptr.is_null() || self.chunks_len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.chunks_ptr, self.chunks_len) }
    }

    pub fn start_pin(&self) -> Pin {
        let chunks = self.chunks();
        Pin {
            node: chunks[0].node,
            x: self.top_x,
            y: chunks[0].start,
            garbage: false,
        }
    }

    pub fn end_pin(&self) -> Pin {
        let chunks = self.chunks();
        let last = chunks.len() - 1;
        Pin {
            node: chunks[last].node,
            x: self.bot_x,
            y: chunks[last].end - 1,
            garbage: false,
        }
    }

    pub fn untracked(&self) -> HighlightUntracked {
        let chunks = self.chunks();
        let last = chunks.len() - 1;
        HighlightUntracked {
            start: Pin {
                node: chunks[0].node,
                x: self.top_x,
                y: chunks[0].start,
                garbage: false,
            },
            end: Pin {
                node: chunks[last].node,
                x: self.bot_x,
                y: chunks[last].end - 1,
                garbage: false,
            },
        }
    }
}

pub fn highlight_tracked_deinit(
    _tracked: *const HighlightTracked,
    _screen: *mut c_void,
) {
}

pub fn highlight_untracked_track(
    _untracked: *const HighlightUntracked,
    _screen: *mut c_void,
) -> *mut HighlightTracked {
    core::ptr::null_mut()
}
