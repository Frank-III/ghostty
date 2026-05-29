//! Port of type definitions from `PageList.zig`.
//!
//! Contains the structural types that make up Ghostty's terminal PageList:
//! a doubly-linked list of pages representing the screen and scrollback.

use crate::highlight::Pin;
use crate::page_core::Page;
use crate::size_types::CellCountInt;
use core::ffi::c_void;

/// A single node within the PageList linked list.
///
/// Each node wraps a `Page` and participates in a doubly-linked list
/// that represents the terminal's screen + scrollback in order.
/// The `serial` field is a monotonically increasing number used to
/// detect stale references.
#[repr(C)]
pub struct PageListNode {
    pub prev: *mut PageListNode,
    pub next: *mut PageListNode,
    pub data: Page,
    pub serial: u64,
}

impl PageListNode {
    /// The sentinel value representing the end of the list.
    pub const NULL: *mut PageListNode = core::ptr::null_mut();
}

/// Memory pool used to allocate PageList nodes, page buffers, and pins.
///
/// TYPE-OPAQUE / Zig-owned: The real `MemoryPool` (PageList.zig:85-115)
/// contains typed sub-pools (`NodePool`, `PagePool`, `PinPool`) backed by
/// Zig's `MemoryPool` / `ArenaAllocator` infrastructure. Fields are exposed
/// as raw `*mut c_void` here so Rust can hold and pass pool pointers
/// without depending on Zig's internal layout. All pool operations
/// (create/destroy nodes, pins, pages) must go through the Zig FFI.
#[repr(C)]
pub struct PageListMemoryPool {
    pub alloc: *mut c_void,
    pub nodes: *mut c_void,
    pub pages: *mut c_void,
    pub pins: *mut c_void,
}

/// Viewport location within the PageList.
///
/// Determines where the user is currently "looking" in the terminal.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageListViewport {
    /// Pinned to the active (visible) area — the bottom of the screen.
    Active = 0,
    /// Pinned to the top of the scrollback history.
    Top = 1,
    /// Pinned to a tracked pin at an arbitrary scrollback position.
    Pin = 2,
}

/// The linked-list head used by PageList.
///
/// This mirrors Zig's `IntrusiveDoublyLinkedList(Node)` with just
/// first/last pointers (the intrusive list has no sentinel node).
#[repr(C)]
pub struct PageListHead {
    pub first: *mut PageListNode,
    pub last: *mut PageListNode,
}

/// Scrollbar state for the PageList.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageListScrollbar {
    /// Total size of the scrollable area (rows).
    pub total: usize,
    /// Offset into the total area where the viewport begins.
    pub offset: usize,
    /// Length of the visible area (viewport rows).
    pub len: usize,
}

impl PageListScrollbar {
    pub const ZERO: Self = Self {
        total: 0,
        offset: 0,
        len: 0,
    };
}

impl Default for PageListScrollbar {
    fn default() -> Self {
        Self::ZERO
    }
}

/// C-ABI variant of `PageListScrollbar` (matches `ghostty_action_scrollbar_s`).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PageListScrollbarC {
    pub total: u64,
    pub offset: u64,
    pub len: u64,
}

impl PageListScrollbar {
    #[inline]
    pub fn to_c(self) -> PageListScrollbarC {
        PageListScrollbarC {
            total: self.total as u64,
            offset: self.offset as u64,
            len: self.len as u64,
        }
    }
}

/// Options for resizing the PageList.
///
/// `cols`/`rows` use 0 as a sentinel for "no change" (Zig uses `?CellCountInt`).
/// `reflow` controls whether text is reflowed or truncated on shrink.
#[repr(C)]
pub struct PageListResize {
    pub cols: CellCountInt,
    pub rows: CellCountInt,
    pub reflow: bool,
    pub cursor_x: CellCountInt,
    pub cursor_y: CellCountInt,
    pub cursor_pin: *mut Pin,
    pub has_cursor: bool,
}

impl Default for PageListResize {
    fn default() -> Self {
        Self {
            cols: 0,
            rows: 0,
            reflow: true,
            cursor_x: 0,
            cursor_y: 0,
            cursor_pin: core::ptr::null_mut(),
            has_cursor: false,
        }
    }
}

/// Direction for iterating over the PageList.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageListDirection {
    /// Toward the beginning (left/up in scrollback).
    LeftUp = 0,
    /// Toward the end (right/down in scrollback).
    RightDown = 1,
}

/// Set of tracked pins on a PageList. Backed by Zig's
/// `AutoArrayHashMapUnmanaged(*Pin, void)`; exposed as a raw pointer array
/// in Rust for layout-stable access.
///
/// `keys[i]` is a pointer to a tracked `Pin`; iteration walks the array to
/// find or remove entries.
#[repr(C)]
pub struct PageListTrackedPinSet {
    pub keys: *mut *mut Pin,
    pub len: usize,
    pub capacity: usize,
}

/// The PageList itself: a linked list of pages with viewport, pool,
/// and bookkeeping state.
#[repr(C)]
pub struct PageList {
    /// The pool from which nodes and pages are allocated.
    pub pool: *mut PageListMemoryPool,
    /// The linked list head (first = oldest, last = newest).
    pub pages: PageListHead,
    /// Monotonically increasing serial for freshly allocated pages.
    pub page_serial: u64,
    /// Lowest valid serial; serials below this are stale.
    pub page_serial_min: u64,
    /// Total byte size of allocated pages (excludes pool overhead).
    pub page_size: usize,
    /// Maximum bytes allowed for scrollback-only pages.
    pub explicit_max_size: usize,
    /// Minimum allowed `explicit_max_size` given current cols/rows.
    pub min_max_size: usize,
    /// Total number of rows (scrollback + active) in the PageList.
    pub total_rows: usize,
    /// Set of tracked pins.
    pub tracked_pins: *mut PageListTrackedPinSet,
    /// Current viewport location.
    pub viewport: PageListViewport,
    /// Pre-allocated pin used when viewport is `Pin` (avoids alloc on scroll).
    pub viewport_pin: *mut Pin,
    /// Cached row offset of `viewport_pin` from the top; null if uncomputed.
    /// Encoded as `usize + 1`, with 0 meaning "not calculated".
    pub viewport_pin_row_offset: usize,
    /// Current desired number of columns.
    pub cols: CellCountInt,
    /// Current desired number of rows.
    pub rows: CellCountInt,
}
