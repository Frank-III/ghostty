use core::ffi::c_void;
use core::ptr;

use crate::highlight::Pin;
use crate::page_core::{std_capacity, Page};
use crate::page_list_types::*;
use crate::page_types::*;
use crate::point::PointTag;
use crate::reflow_cursor::ReflowCursor;
use crate::size_types::OffsetBuf;
use crate::CellCountInt;

const PAGE_PREHEAT: usize = 4;

fn std_size() -> usize { Page::layout(std_capacity()).total_size }

#[repr(C)]
struct TrackedPinArray {
    keys: *mut *mut Pin,
    len: usize,
    capacity: usize,
}

extern "C" {
    fn ghostty_vt_pin_create(
        pool: *mut c_void,
        node: *mut c_void,
        y: u16,
        x: u16,
        garbage: bool,
    ) -> *mut c_void;
    fn ghostty_vt_pin_destroy(pool: *mut c_void, pin: *mut c_void);
    fn ghostty_vt_pool_alloc(pool: *mut c_void, size: usize) -> *mut u8;
    fn ghostty_vt_pool_free(pool: *mut c_void, ptr: *mut u8, size: usize);
}

pub const STD_CAPACITY_ROWS: CellCountInt = 215;
pub const STD_CAPACITY_COLS: CellCountInt = 215;

// ---------------------------------------------------------------------------
// PageListHead -- intrusive doubly-linked list operations
// ---------------------------------------------------------------------------

impl PageListHead {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.first.is_null()
    }

    pub fn append(&mut self, node: *mut PageListNode) {
        unsafe {
            let n = &mut *node;
            n.next = ptr::null_mut();
            n.prev = self.last;
            if !self.last.is_null() {
                (*self.last).next = node;
            } else {
                self.first = node;
            }
            self.last = node;
        }
    }

    pub fn prepend(&mut self, node: *mut PageListNode) {
        unsafe {
            let n = &mut *node;
            n.prev = ptr::null_mut();
            n.next = self.first;
            if !self.first.is_null() {
                (*self.first).prev = node;
            } else {
                self.last = node;
            }
            self.first = node;
        }
    }

    pub fn remove(&mut self, node: *mut PageListNode) {
        unsafe {
            let n = &*node;
            if !n.prev.is_null() {
                (*n.prev).next = n.next;
            } else {
                self.first = n.next;
            }
            if !n.next.is_null() {
                (*n.next).prev = n.prev;
            } else {
                self.last = n.prev;
            }
        }
    }

    pub fn insert_after(&mut self, existing: *mut PageListNode, node: *mut PageListNode) {
        unsafe {
            let e = &mut *existing;
            let n = &mut *node;
            n.prev = existing;
            n.next = e.next;
            if !e.next.is_null() {
                (*e.next).prev = node;
            } else {
                self.last = node;
            }
            e.next = node;
        }
    }

    pub fn insert_before(&mut self, existing: *mut PageListNode, node: *mut PageListNode) {
        unsafe {
            let e = &mut *existing;
            let n = &mut *node;
            n.next = existing;
            n.prev = e.prev;
            if !e.prev.is_null() {
                (*e.prev).next = node;
            } else {
                self.first = node;
            }
            e.prev = node;
        }
    }

    pub fn pop_first(&mut self) -> *mut PageListNode {
        let node = self.first;
        if !node.is_null() {
            self.remove(node);
        }
        node
    }

    pub fn pop_last(&mut self) -> *mut PageListNode {
        let node = self.last;
        if !node.is_null() {
            self.remove(node);
        }
        node
    }
}

// ---------------------------------------------------------------------------
// Pin -- movement and query methods
// ---------------------------------------------------------------------------

pub enum PinMoveResult {
    Offset(Pin),
    Overflow(Pin, usize),
}

impl Pin {
    pub fn down(self, n: usize) -> Option<Pin> {
        match self.down_overflow(n) {
            PinMoveResult::Offset(p) => Some(p),
            PinMoveResult::Overflow(_, _) => None,
        }
    }

    pub fn up(self, n: usize) -> Option<Pin> {
        match self.up_overflow(n) {
            PinMoveResult::Offset(p) => Some(p),
            PinMoveResult::Overflow(_, _) => None,
        }
    }

    pub fn down_overflow(self, n: usize) -> PinMoveResult {
        unsafe {
            let node = self.node;
            let node_ref = &*node;
            let rows_after = (node_ref.data.size.rows as usize) - (self.y as usize + 1);
            if n <= rows_after {
                return PinMoveResult::Offset(Pin {
                    node: self.node,
                    y: (self.y as usize + n) as CellCountInt,
                    x: self.x,
                    garbage: false,
                });
            }

            let mut cur = node;
            let mut n_left = n - rows_after;
            loop {
                let next = (*cur).next;
                if next.is_null() {
                    return PinMoveResult::Overflow(
                        Pin {
                            node: cur,
                            y: (*cur).data.size.rows - 1,
                            x: self.x,
                            garbage: false,
                        },
                        n_left,
                    );
                }
                let next_rows = (*next).data.size.rows as usize;
                if n_left <= next_rows {
                    return PinMoveResult::Offset(Pin {
                        node: next,
                        y: (n_left - 1) as CellCountInt,
                        x: self.x,
                        garbage: false,
                    });
                }
                n_left -= next_rows;
                cur = next;
            }
        }
    }

    pub fn up_overflow(self, n: usize) -> PinMoveResult {
        if n <= self.y as usize {
            return PinMoveResult::Offset(Pin {
                node: self.node,
                y: (self.y as usize - n) as CellCountInt,
                x: self.x,
                garbage: false,
            });
        }

        unsafe {
            let mut cur = self.node;
            let mut n_left = n - self.y as usize;
            loop {
                let prev = (*cur).prev;
                if prev.is_null() {
                    return PinMoveResult::Overflow(
                        Pin {
                            node: cur,
                            y: 0,
                            x: self.x,
                            garbage: false,
                        },
                        n_left,
                    );
                }
                let prev_rows = (*prev).data.size.rows as usize;
                if n_left <= prev_rows {
                    return PinMoveResult::Offset(Pin {
                        node: prev,
                        y: (prev_rows - n_left) as CellCountInt,
                        x: self.x,
                        garbage: false,
                    });
                }
                n_left -= prev_rows;
                cur = prev;
            }
        }
    }

    #[inline]
    pub fn left(self, n: usize) -> Pin {
        debug_assert!(n <= self.x as usize);
        Pin {
            node: self.node,
            y: self.y,
            x: self.x.saturating_sub(n as CellCountInt),
            garbage: self.garbage,
        }
    }

    #[inline]
    pub fn right(self, n: usize) -> Pin {
        unsafe {
            debug_assert!(self.x as usize + n < (*self.node).data.size.cols as usize);
        }
        Pin {
            node: self.node,
            y: self.y,
            x: self.x.saturating_add(n as CellCountInt),
            garbage: self.garbage,
        }
    }

    #[inline]
    pub fn left_clamp(self, n: CellCountInt) -> Pin {
        Pin {
            node: self.node,
            y: self.y,
            x: self.x.saturating_sub(n),
            garbage: self.garbage,
        }
    }

    #[inline]
    pub fn right_clamp(self, n: CellCountInt) -> Pin {
        unsafe {
            let max_x = (*self.node).data.size.cols - 1;
            Pin {
                node: self.node,
                y: self.y,
                x: core::cmp::min(self.x.saturating_add(n), max_x),
                garbage: self.garbage,
            }
        }
    }

    pub fn left_wrap(self, n: usize) -> Option<Pin> {
        if n <= self.x as usize {
            return Some(self.left(n));
        }
        unsafe {
            let cols = (*self.node).data.size.cols as usize;
            let extra = n - self.x as usize;
            let rows_off = 1 + extra / cols;
            match self.up_overflow(rows_off) {
                PinMoveResult::Offset(mut p) => {
                    p.x = (cols - extra % cols) as CellCountInt;
                    Some(p)
                }
                PinMoveResult::Overflow(_, _) => None,
            }
        }
    }

    pub fn right_wrap(self, n: usize) -> Option<Pin> {
        unsafe {
            let cols = (*self.node).data.size.cols as usize;
            let remaining_in_row = cols - self.x as usize - 1;
            if n <= remaining_in_row {
                return Some(self.right(n));
            }
            let extra = n - remaining_in_row;
            let rows_off = 1 + extra / cols;
            match self.down_overflow(rows_off) {
                PinMoveResult::Offset(mut p) => {
                    p.x = (extra % cols - 1) as CellCountInt;
                    Some(p)
                }
                PinMoveResult::Overflow(_, _) => None,
            }
        }
    }

    pub fn before(self, other: Pin) -> bool {
        if self.node == other.node {
            if self.y < other.y {
                return true;
            }
            if self.y > other.y {
                return false;
            }
            return self.x < other.x;
        }
        unsafe {
            let mut cur = (*self.node).next;
            while !cur.is_null() {
                if cur == other.node {
                    return true;
                }
                cur = (*cur).next;
            }
        }
        false
    }

    pub fn is_between(self, top: Pin, bottom: Pin) -> bool {
        if self.node == top.node {
            if (self.y as usize) < (top.y as usize) {
                return false;
            }
            if self.y > top.y {
                return if self.node == bottom.node {
                    self.y <= bottom.y
                } else {
                    true
                };
            }
            if self.x < top.x {
                return false;
            }
        }

        if self.node == bottom.node {
            if self.y > bottom.y {
                return false;
            }
            if self.y < bottom.y {
                return true;
            }
            return self.x <= bottom.x;
        }

        if top.node == bottom.node {
            return false;
        }

        unsafe {
            let mut cur = (*top.node).next;
            while !cur.is_null() {
                if cur == bottom.node {
                    break;
                }
                if cur == self.node {
                    return true;
                }
                cur = (*cur).next;
            }
        }
        false
    }

    #[inline]
    pub fn row_and_cell_ptr(self) -> (*mut Row, *mut Cell) {
        unsafe {
            let page = &(*self.node).data;
            let row = page.get_row(self.y as usize);
            let cells = page.row_cells_ptr(row);
            let cell = cells.add(self.x as usize);
            (row, cell)
        }
    }

    #[inline]
    pub fn row_ptr(self) -> *mut Row {
        unsafe { (*self.node).data.get_row(self.y as usize) }
    }
}

// ---------------------------------------------------------------------------
// PageIterator / RowIterator / CellIterator / PromptIterator types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageIteratorChunk {
    pub node: *mut PageListNode,
    pub start: CellCountInt,
    pub end: CellCountInt,
}

impl PageIteratorChunk {
    #[inline]
    pub fn full_page(&self) -> bool {
        unsafe { self.start == 0 && self.end == (*self.node).data.size.rows }
    }

    pub fn overlaps(&self, other: &PageIteratorChunk) -> bool {
        if self.node != other.node {
            return false;
        }
        if self.end <= other.start {
            return false;
        }
        if self.start >= other.end {
            return false;
        }
        true
    }

    pub fn row_count(&self) -> usize {
        (self.end - self.start) as usize
    }
}

pub(crate) enum PageIteratorLimit {
    None,
    Count(usize),
    Row(Pin),
}

pub struct PageIterator {
    row: Option<Pin>,
    limit: PageIteratorLimit,
    direction: PageListDirection,
}

impl PageIterator {
    pub fn new_empty() -> Self {
        Self {
            row: None,
            limit: PageIteratorLimit::None,
            direction: PageListDirection::RightDown,
        }
    }

    pub fn new_at_pin(start: Pin, direction: PageListDirection, limit: Option<Pin>) -> Self {
        Self {
            row: Some(start),
            limit: match limit {
                Some(p) => PageIteratorLimit::Row(p),
                None => PageIteratorLimit::None,
            },
            direction,
        }
    }

    pub fn new_unlimited(start: Pin, direction: PageListDirection) -> Self {
        Self {
            row: Some(start),
            limit: PageIteratorLimit::None,
            direction,
        }
    }

    pub fn next(&mut self) -> Option<PageIteratorChunk> {
        match self.direction {
            PageListDirection::RightDown => self.next_down(),
            PageListDirection::LeftUp => self.next_up(),
        }
    }

    fn next_down(&mut self) -> Option<PageIteratorChunk> {
        let row = self.row?;
        match &mut self.limit {
            PageIteratorLimit::None => unsafe {
                let next_ptr = (*row.node).next;
                self.row = if next_ptr.is_null() {
                    None
                } else {
                    Some(Pin {
                        node: next_ptr,
                        y: 0,
                        x: 0,
                        garbage: false,
                    })
                };
                let end = (*row.node).data.size.rows;
                Some(PageIteratorChunk {
                    node: row.node,
                    start: row.y,
                    end,
                })
            },
            PageIteratorLimit::Count(ref mut limit) => unsafe {
                let avail = ((*row.node).data.size.rows as usize) - (row.y as usize);
                let len = if avail < *limit { avail } else { *limit };
                if len < *limit {
                    let next_ptr = (*row.node).next;
                    self.row = if next_ptr.is_null() {
                        None
                    } else {
                        Some(Pin {
                            node: next_ptr,
                            y: 0,
                            x: 0,
                            garbage: false,
                        })
                    };
                    *limit -= len;
                } else {
                    self.row = None;
                }
                Some(PageIteratorChunk {
                    node: row.node,
                    start: row.y,
                    end: row.y + len as CellCountInt,
                })
            },
            PageIteratorLimit::Row(limit_row) => unsafe {
                if limit_row.node != row.node {
                    let next_ptr = (*row.node).next;
                    self.row = if next_ptr.is_null() {
                        None
                    } else {
                        Some(Pin {
                            node: next_ptr,
                            y: 0,
                            x: 0,
                            garbage: false,
                        })
                    };
                    let end = (*row.node).data.size.rows;
                    Some(PageIteratorChunk {
                        node: row.node,
                        start: row.y,
                        end,
                    })
                } else {
                    self.row = None;
                    if row.y > limit_row.y {
                        return None;
                    }
                    Some(PageIteratorChunk {
                        node: row.node,
                        start: row.y,
                        end: limit_row.y + 1,
                    })
                }
            },
        }
    }

    fn next_up(&mut self) -> Option<PageIteratorChunk> {
        let row = self.row?;
        match &mut self.limit {
            PageIteratorLimit::None => unsafe {
                let prev_ptr = (*row.node).prev;
                self.row = if prev_ptr.is_null() {
                    None
                } else {
                    let prev_rows = (*prev_ptr).data.size.rows;
                    Some(Pin {
                        node: prev_ptr,
                        y: prev_rows - 1,
                        x: 0,
                        garbage: false,
                    })
                };
                Some(PageIteratorChunk {
                    node: row.node,
                    start: 0,
                    end: row.y + 1,
                })
            },
            PageIteratorLimit::Count(ref mut limit) => {
                let avail = row.y as usize + 1;
                let len = if avail < *limit { avail } else { *limit };
                if len < *limit {
                    self.row = match row.up(len) {
                        Some(p) => Some(p),
                        None => None,
                    };
                    *limit -= len;
                } else {
                    self.row = None;
                }
                let start = if row.y as usize >= len {
                    row.y - len as CellCountInt
                } else {
                    0
                };
                Some(PageIteratorChunk {
                    node: row.node,
                    start,
                    end: row.y + 1,
                })
            }
            PageIteratorLimit::Row(limit_row) => unsafe {
                if limit_row.node != row.node {
                    let prev_ptr = (*row.node).prev;
                    self.row = if prev_ptr.is_null() {
                        None
                    } else {
                        let prev_rows = (*prev_ptr).data.size.rows;
                        Some(Pin {
                            node: prev_ptr,
                            y: prev_rows - 1,
                            x: 0,
                            garbage: false,
                        })
                    };
                    Some(PageIteratorChunk {
                        node: row.node,
                        start: 0,
                        end: row.y + 1,
                    })
                } else {
                    self.row = None;
                    if row.y < limit_row.y {
                        return None;
                    }
                    Some(PageIteratorChunk {
                        node: row.node,
                        start: limit_row.y,
                        end: row.y + 1,
                    })
                }
            },
        }
    }
}

pub struct RowIterator {
    pub page_it: PageIterator,
    chunk: Option<PageIteratorChunk>,
    offset: CellCountInt,
}

impl RowIterator {
    pub fn new_empty() -> Self {
        Self {
            page_it: PageIterator::new_empty(),
            chunk: None,
            offset: 0,
        }
    }

    pub fn new_from_pin(start: Pin, direction: PageListDirection) -> Self {
        Self::new_from_pin_with_limit(start, direction, None)
    }

    pub fn new_from_pin_with_limit(
        start: Pin,
        direction: PageListDirection,
        limit: Option<Pin>,
    ) -> Self {
        let mut page_it = PageIterator::new_at_pin(start, direction, limit);
        let chunk = page_it.next();
        let offset = match &chunk {
            Some(c) => match direction {
                PageListDirection::RightDown => c.start,
                PageListDirection::LeftUp => c.end.saturating_sub(1),
            },
            None => 0,
        };
        Self {
            page_it,
            chunk,
            offset,
        }
    }

    pub fn next(&mut self) -> Option<Pin> {
        let chunk = self.chunk?;
        let row_pin = Pin {
            node: chunk.node,
            y: self.offset,
            x: 0,
            garbage: false,
        };

        match self.page_it.direction {
            PageListDirection::RightDown => {
                self.offset += 1;
                if self.offset >= chunk.end {
                    self.chunk = self.page_it.next();
                    if let Some(c) = &self.chunk {
                        self.offset = c.start;
                    }
                }
            }
            PageListDirection::LeftUp => {
                if self.offset == 0 {
                    self.chunk = self.page_it.next();
                    if let Some(c) = &self.chunk {
                        self.offset = c.end - 1;
                    }
                } else if self.offset == chunk.start {
                    self.chunk = None;
                } else {
                    self.offset -= 1;
                }
            }
        }

        Some(row_pin)
    }
}

pub struct CellIterator {
    pub row_it: RowIterator,
    cell: Option<Pin>,
}

pub fn cell_iterator_at_pin(
    start: Pin,
    direction: PageListDirection,
    limit: Option<Pin>,
) -> CellIterator {
    let mut row_it = RowIterator::new_from_pin_with_limit(start, direction, limit);
    let mut cell = row_it.next();
    if let Some(ref mut c) = cell {
        c.x = start.x;
    }
    CellIterator { row_it, cell }
}

impl CellIterator {
    pub fn next(&mut self) -> Option<Pin> {
        let cell = self.cell?;
        match self.row_it.page_it.direction {
            PageListDirection::RightDown => unsafe {
                if (cell.x as usize + 1) < (*cell.node).data.size.cols as usize {
                    let mut copy = cell;
                    copy.x += 1;
                    self.cell = Some(copy);
                } else {
                    self.cell = self.row_it.next();
                }
            },
            PageListDirection::LeftUp => unsafe {
                if cell.x > 0 {
                    let mut copy = cell;
                    copy.x -= 1;
                    self.cell = Some(copy);
                } else if let Some(mut next_cell) = self.row_it.next() {
                    next_cell.x = (*next_cell.node).data.size.cols - 1;
                    self.cell = Some(next_cell);
                } else {
                    self.cell = None;
                }
            },
        }
        Some(cell)
    }
}

pub struct PromptIterator {
    current: Option<Pin>,
    limit: Option<Pin>,
    direction: PageListDirection,
}

impl PromptIterator {
    pub fn new_empty() -> Self {
        Self {
            current: None,
            limit: None,
            direction: PageListDirection::LeftUp,
        }
    }

    pub fn new(current: Option<Pin>, limit: Option<Pin>, direction: PageListDirection) -> Self {
        Self {
            current,
            limit,
            direction,
        }
    }

    pub fn next(&mut self) -> Option<Pin> {
        match self.direction {
            PageListDirection::RightDown => self.next_right_down(),
            PageListDirection::LeftUp => self.next_left_up(),
        }
    }

    fn next_right_down(&mut self) -> Option<Pin> {
        let start = self.current?;
        let mut current = Some(start);
        while let Some(p) = current {
            let at_limit = self.limit.map_or(false, |l| l.eql(p));
            let row = p.row_ptr();

            unsafe {
                let sp = (*row).semantic_prompt();
                match sp {
                    SemanticPrompt::None => {
                        if at_limit {
                            break;
                        }
                    }
                    SemanticPrompt::Prompt | SemanticPrompt::PromptContinuation => {
                        if at_limit {
                            self.current = None;
                            return Some(p.left(p.x as usize));
                        }
                        let mut end_pin = p;
                        while let Some(next_pin) = end_pin.down(1) {
                            let next_row = next_pin.row_ptr();
                            match (*next_row).semantic_prompt() {
                                SemanticPrompt::PromptContinuation => {
                                    if let Some(lim) = self.limit {
                                        if lim.eql(next_pin) {
                                            break;
                                        }
                                    }
                                }
                                SemanticPrompt::Prompt | SemanticPrompt::None => {
                                    self.current = Some(next_pin);
                                    return Some(p.left(p.x as usize));
                                }
                            }
                            end_pin = next_pin;
                        }
                        self.current = None;
                        return Some(p.left(p.x as usize));
                    }
                }
            }

            current = p.down(1);
        }
        self.current = None;
        None
    }

    fn next_left_up(&mut self) -> Option<Pin> {
        let start = self.current?;
        let mut current = Some(start);
        while let Some(p) = current {
            let at_limit = self.limit.map_or(false, |l| l.eql(p));
            let row = p.row_ptr();

            unsafe {
                let sp = (*row).semantic_prompt();
                match sp {
                    SemanticPrompt::None => {
                        if at_limit {
                            break;
                        }
                    }
                    SemanticPrompt::Prompt => {
                        self.current = if at_limit { None } else { p.up(1) };
                        return Some(p.left(p.x as usize));
                    }
                    SemanticPrompt::PromptContinuation => {
                        if at_limit {
                            self.current = None;
                            return Some(p.left(p.x as usize));
                        }
                        let mut end_pin = p;
                        while let Some(prior) = end_pin.up(1) {
                            if let Some(lim) = self.limit {
                                if lim.eql(prior) {
                                    break;
                                }
                            }
                            let prior_row = prior.row_ptr();
                            match (*prior_row).semantic_prompt() {
                                SemanticPrompt::None => {
                                    self.current = Some(prior);
                                    return Some(end_pin.left(end_pin.x as usize));
                                }
                                SemanticPrompt::PromptContinuation => {}
                                SemanticPrompt::Prompt => {
                                    self.current = prior.up(1);
                                    return Some(prior.left(prior.x as usize));
                                }
                            }
                            end_pin = prior;
                        }
                        self.current = None;
                        return Some(p.left(p.x as usize));
                    }
                }
            }

            current = p.up(1);
        }
        self.current = None;
        None
    }
}

// ---------------------------------------------------------------------------
// Scroll command for PageList
// ---------------------------------------------------------------------------

pub enum PageListScroll {
    Active,
    Top,
    Row(usize),
    DeltaRow(isize),
    DeltaPrompt(isize),
    Pin(Pin),
}

// ---------------------------------------------------------------------------
// PageList -- main impl
// ---------------------------------------------------------------------------

impl PageList {
    // -- Integrity (no-ops in release, matching page_core pattern) ----------

    #[inline]
    pub fn assert_integrity(&self) {}

    #[inline]
    pub fn pause_integrity_checks(&mut self, _pause: bool) {}

    // -- Size / capacity queries -------------------------------------------

    #[inline]
    pub fn max_size(&self) -> usize {
        if self.explicit_max_size > self.min_max_size {
            self.explicit_max_size
        } else {
            self.min_max_size
        }
    }

    pub fn total_rows_count(&self) -> usize {
        let mut rows: usize = 0;
        let mut cur = self.pages.first;
        while !cur.is_null() {
            unsafe {
                rows += (*cur).data.size.rows as usize;
                cur = (*cur).next;
            }
        }
        rows
    }

    pub fn total_pages(&self) -> usize {
        let mut count: usize = 0;
        let mut cur = self.pages.first;
        while !cur.is_null() {
            count += 1;
            unsafe {
                cur = (*cur).next;
            }
        }
        count
    }

    // -- Positioning: get_top_left / get_bottom_right -----------------------

    pub fn get_top_left(&self, tag: PointTag) -> Pin {
        match tag {
            PointTag::SCREEN | PointTag::HISTORY => Pin {
                node: self.pages.first,
                y: 0,
                x: 0,
                garbage: false,
            },
            PointTag::VIEWPORT => match self.viewport {
                PageListViewport::Active => self.get_top_left(PointTag::ACTIVE),
                PageListViewport::Top => self.get_top_left(PointTag::SCREEN),
                PageListViewport::Pin => unsafe { *((self.viewport_pin) as *const Pin) },
            },
            PointTag::ACTIVE => {
                let mut rem = self.rows as usize;
                let mut it = self.pages.last;
                while !it.is_null() {
                    unsafe {
                        let rows = (*it).data.size.rows as usize;
                        if rem <= rows {
                            return Pin {
                                node: it,
                                y: (rows - rem) as CellCountInt,
                                x: 0,
                                garbage: false,
                            };
                        }
                        rem -= rows;
                        it = (*it).prev;
                    }
                }
                // Should never happen; fall back to first page
                Pin {
                    node: self.pages.first,
                    y: 0,
                    x: 0,
                    garbage: false,
                }
            }
        }
    }

    pub fn get_bottom_right(&self, tag: PointTag) -> Option<Pin> {
        match tag {
            PointTag::SCREEN | PointTag::ACTIVE => unsafe {
                let node = self.pages.last;
                if node.is_null() {
                    return None;
                }
                Some(Pin {
                    node,
                    y: (*node).data.size.rows - 1,
                    x: (*node).data.size.cols - 1,
                    garbage: false,
                })
            },
            PointTag::VIEWPORT => {
                let mut br = self.get_top_left(PointTag::VIEWPORT);
                br = br.down(self.rows as usize - 1)?;
                unsafe {
                    br.x = (*br.node).data.size.cols - 1;
                }
                Some(br)
            }
            PointTag::HISTORY => {
                let mut br = self.get_top_left(PointTag::ACTIVE);
                br = br.up(1)?;
                unsafe {
                    br.x = (*br.node).data.size.cols - 1;
                }
                Some(br)
            }
        }
    }

    // -- Pin / unpin -------------------------------------------------------

    pub fn pin(&self, tag: PointTag, x: CellCountInt, y: u32) -> Option<Pin> {
        if x >= self.cols {
            return None;
        }
        let tl = self.get_top_left(tag);
        let moved = tl.down(y as usize)?;
        let mut result = Pin {
            node: moved.node,
            y: moved.y,
            x,
            garbage: false,
        };
        result.x = x;
        Some(result)
    }

    pub fn pin_is_active(&self, p: Pin) -> bool {
        let active = self.get_top_left(PointTag::ACTIVE);
        if p.node == active.node {
            return (p.y as usize) >= (active.y as usize);
        }
        unsafe {
            let mut cur = (*active.node).next;
            while !cur.is_null() {
                if cur == p.node {
                    return true;
                }
                cur = (*cur).next;
            }
        }
        false
    }

    pub fn pin_is_top(&self, p: Pin) -> bool {
        p.y == 0 && p.node == self.pages.first && !self.pages.first.is_null()
    }

    pub fn pin_is_valid(&self, p: Pin) -> bool {
        let mut it = self.pages.first;
        while !it.is_null() {
            unsafe {
                if it == p.node {
                    return (p.y as usize) < ((*it).data.size.rows as usize)
                        && (p.x as usize) < ((*it).data.size.cols as usize);
                }
                it = (*it).next;
            }
        }
        false
    }

    /// Track a pin so it remains valid across page mutations.
    ///
    /// MEMORYPOOL-BOUND: Zig's `trackPin` (PageList.zig:3973) calls
    /// `self.pool.pins.create()` to allocate a `*Pin` from the pin pool,
    /// Track a pin so it survives reflow/scroll/resize mutations.
    ///
    /// Allocates a `Pin` from `self.pool.pins` via the Zig FFI bridge
    /// (`ghostty_vt_pin_create`), then inserts the resulting pointer into
    /// `self.tracked_pins`. The tracked-pins keys array grows lazily via
    /// `ghostty_vt_pool_alloc`/`ghostty_vt_pool_free`.
    pub fn track_pin(&mut self, p: Pin) -> *mut Pin {
        if self.pool.is_null() {
            return ptr::null_mut();
        }
        let pool_raw = self.pool as *mut c_void;
        let pin_raw: *mut Pin = unsafe {
            ghostty_vt_pin_create(
                pool_raw,
                p.node as *mut c_void,
                p.y,
                p.x,
                p.garbage,
            ) as *mut Pin
        };
        if pin_raw.is_null() {
            return ptr::null_mut();
        }
        unsafe {
            if self.tracked_pins.is_null() {
                let tp_size = core::mem::size_of::<PageListTrackedPinSet>();
                let tp_ptr = ghostty_vt_pool_alloc(pool_raw, tp_size)
                    as *mut PageListTrackedPinSet;
                if tp_ptr.is_null() {
                    ghostty_vt_pin_destroy(pool_raw, pin_raw as *mut c_void);
                    return ptr::null_mut();
                }
                (*tp_ptr).keys = ptr::null_mut();
                (*tp_ptr).len = 0;
                (*tp_ptr).capacity = 0;
                self.tracked_pins = tp_ptr;
            }
            let tp = &mut *self.tracked_pins;
            if tp.len == tp.capacity {
                let new_cap = if tp.capacity == 0 { 8 } else { tp.capacity * 2 };
                let elem = core::mem::size_of::<*mut Pin>();
                let new_size = new_cap * elem;
                let new_keys = ghostty_vt_pool_alloc(pool_raw, new_size)
                    as *mut *mut Pin;
                if new_keys.is_null() {
                    ghostty_vt_pin_destroy(pool_raw, pin_raw as *mut c_void);
                    return ptr::null_mut();
                }
                if !tp.keys.is_null() && tp.len > 0 {
                    core::ptr::copy_nonoverlapping(tp.keys, new_keys, tp.len);
                    ghostty_vt_pool_free(
                        pool_raw,
                        tp.keys as *mut u8,
                        tp.capacity * elem,
                    );
                }
                tp.keys = new_keys;
                tp.capacity = new_cap;
            }
            *tp.keys.add(tp.len) = pin_raw;
            tp.len += 1;
            pin_raw
        }
    }

    /// Untrack a previously tracked pin and free it back to the pool.
    ///
    /// Removes the entry from `self.tracked_pins` via swap-remove, then
    /// calls `ghostty_vt_pin_destroy` to return the `Pin` memory to the
    /// Zig `PageListMemoryPool`.
    pub fn untrack_pin(&mut self, p: *mut Pin) {
        if p.is_null() {
            return;
        }
        if !self.tracked_pins.is_null() {
            unsafe {
                let tp = &mut *self.tracked_pins;
                if !tp.keys.is_null() && tp.len > 0 {
                    let mut i = 0usize;
                    while i < tp.len {
                        if *tp.keys.add(i) == p {
                            let last = *tp.keys.add(tp.len - 1);
                            *tp.keys.add(i) = last;
                            tp.len -= 1;
                            break;
                        }
                        i += 1;
                    }
                }
            }
        }
        if !self.pool.is_null() {
            unsafe {
                ghostty_vt_pin_destroy(self.pool as *mut c_void, p as *mut c_void);
            }
        }
    }

    pub fn point_from_pin(&self, tag: PointTag, p: Pin) -> Option<(CellCountInt, u32)> {
        let tl = self.get_top_left(tag);
        if p.node == tl.node {
            if (p.y as usize) < (tl.y as usize) {
                return None;
            }
            Some((p.x, (p.y - tl.y) as u32))
        } else {
            let mut y: u32 = (tl.node as *mut PageListNode != p.node) as u32
                * unsafe { (*tl.node).data.size.rows as u32 - tl.y as u32 };
            let mut cur = unsafe { (*tl.node).next };
            loop {
                if cur.is_null() {
                    return None;
                }
                if cur == p.node {
                    y += p.y as u32;
                    break;
                }
                unsafe {
                    y += (*cur).data.size.rows as u32;
                    cur = (*cur).next;
                }
            }
            Some((p.x, y))
        }
    }

    // -- Iterator constructors ----------------------------------------------

    pub fn page_iterator(
        &self,
        direction: PageListDirection,
        tl_tag: PointTag,
        tl_x: CellCountInt,
        tl_y: u32,
        bl_tag: Option<PointTag>,
        bl_x: Option<CellCountInt>,
        bl_y: Option<u32>,
    ) -> PageIterator {
        let tl_pin = match self.pin(tl_tag, tl_x, tl_y) {
            Some(p) => p,
            None => return PageIterator::new_empty(),
        };
        let bl_pin = if let (Some(bt), Some(bx), Some(by)) = (bl_tag, bl_x, bl_y) {
            match self.pin(bt, bx, by) {
                Some(p) => p,
                None => match self.get_bottom_right(tl_tag) {
                    Some(p) => p,
                    None => return PageIterator::new_empty(),
                },
            }
        } else {
            match self.get_bottom_right(tl_tag) {
                Some(p) => p,
                None => return PageIterator::new_empty(),
            }
        };

        match direction {
            PageListDirection::RightDown => PageIterator {
                row: Some(tl_pin),
                limit: PageIteratorLimit::Row(bl_pin),
                direction: PageListDirection::RightDown,
            },
            PageListDirection::LeftUp => PageIterator {
                row: Some(bl_pin),
                limit: PageIteratorLimit::Row(tl_pin),
                direction: PageListDirection::LeftUp,
            },
        }
    }

    pub fn page_iterator_all(&self, direction: PageListDirection, tag: PointTag) -> PageIterator {
        let start = match direction {
            PageListDirection::RightDown => self.get_top_left(tag),
            PageListDirection::LeftUp => match self.get_bottom_right(tag) {
                Some(p) => p,
                None => return PageIterator::new_empty(),
            },
        };
        PageIterator {
            row: Some(start),
            limit: PageIteratorLimit::None,
            direction,
        }
    }

    pub fn row_iterator(
        &self,
        direction: PageListDirection,
        tl_tag: PointTag,
        tl_x: CellCountInt,
        tl_y: u32,
        bl_tag: Option<PointTag>,
        bl_x: Option<CellCountInt>,
        bl_y: Option<u32>,
    ) -> RowIterator {
        let mut page_it = self.page_iterator(direction, tl_tag, tl_x, tl_y, bl_tag, bl_x, bl_y);
        let chunk = page_it.next();
        let offset = match &chunk {
            Some(c) => match direction {
                PageListDirection::RightDown => c.start,
                PageListDirection::LeftUp => c.end.saturating_sub(1),
            },
            None => 0,
        };
        RowIterator {
            page_it,
            chunk,
            offset,
        }
    }

    pub fn cell_iterator(
        &self,
        direction: PageListDirection,
        tl_tag: PointTag,
        tl_x: CellCountInt,
        tl_y: u32,
        bl_tag: Option<PointTag>,
        bl_x: Option<CellCountInt>,
        bl_y: Option<u32>,
    ) -> CellIterator {
        let mut row_it = self.row_iterator(direction, tl_tag, tl_x, tl_y, bl_tag, bl_x, bl_y);
        let cell = row_it.next().map(|mut c| {
            c.x = tl_x;
            c
        });
        CellIterator { row_it, cell }
    }

    // -- Scrollbar ----------------------------------------------------------

    pub fn scrollbar(&self) -> PageListScrollbar {
        if self.explicit_max_size == 0 {
            return PageListScrollbar {
                total: self.rows as usize,
                offset: 0,
                len: self.rows as usize,
            };
        }
        PageListScrollbar {
            total: self.total_rows,
            offset: self.viewport_row_offset(),
            len: self.rows as usize,
        }
    }

    pub fn viewport_row_offset(&self) -> usize {
        match self.viewport {
            PageListViewport::Top => 0,
            PageListViewport::Active => self.total_rows.saturating_sub(self.rows as usize),
            PageListViewport::Pin => {
                let cached = self.viewport_pin_row_offset;
                if cached > 0 {
                    return cached - 1;
                }
                let offset = self.compute_viewport_pin_row_offset();
                offset
            }
        }
    }

    fn compute_viewport_pin_row_offset(&self) -> usize {
        unsafe {
            let vp_pin = &*(self.viewport_pin as *const Pin);
            let mut offset: usize = 0;
            let mut node = self.pages.last;
            while !node.is_null() {
                let rows = (*node).data.size.rows as usize;
                offset += rows;
                if node == vp_pin.node {
                    offset -= vp_pin.y as usize;
                    return self.total_rows - offset;
                }
                node = (*node).prev;
            }
            // Fallback: pin not found
            self.total_rows.saturating_sub(self.rows as usize)
        }
    }

    // -- Grow: add exactly one row to the active area -----------------------

    pub fn grow(&mut self) -> *mut PageListNode {
        unsafe {
            let last = self.pages.last;
            if last.is_null() {
                return ptr::null_mut();
            }

            let last_ref = &mut *last;
            if last_ref.data.capacity.rows > last_ref.data.size.rows {
                last_ref.data.size.rows += 1;
                self.total_rows += 1;
                return ptr::null_mut();
            }

            let cap = std_capacity();
            let new_page_size = Page::layout(cap).total_size;

            if !self.pages.first.is_null()
                && self.pages.first != self.pages.last
                && self.page_size + new_page_size > self.max_size()
            {
                let first = self.pages.pop_first();
                if !first.is_null() && first != last {
                    let first_rows = (*first).data.size.rows as usize;
                    self.total_rows = self.total_rows.saturating_sub(first_rows);

                    if self.total_rows + 1 < self.rows as usize {
                        self.pages.prepend(first);
                        self.total_rows += first_rows;
                    } else {
                        if self.viewport == PageListViewport::Pin
                            && self.viewport_pin_row_offset > 0
                        {
                            let v = self.viewport_pin_row_offset - 1;
                            if v < first_rows {
                                self.viewport = PageListViewport::Top;
                            } else {
                                self.viewport_pin_row_offset = v - first_rows + 1;
                            }
                        }

                        if !self.tracked_pins.is_null() {
                            let tp = &*(self.tracked_pins as *const TrackedPinArray);
                            if !tp.keys.is_null() {
                                let new_first = self.pages.first;
                                let mut i = 0usize;
                                while i < tp.len {
                                    let p = *tp.keys.add(i);
                                    if !p.is_null() && (*p).node == first {
                                        (*p).node = new_first;
                                        (*p).y = 0;
                                        (*p).x = 0;
                                        (*p).garbage = true;
                                    }
                                    i += 1;
                                }
                            }
                        }
                        ptr::write(
                            self.viewport_pin as *mut Pin,
                            Pin {
                                node: self.pages.first,
                                y: 0,
                                x: 0,
                                garbage: false,
                            },
                        );

                        if (*first).data.memory_len > std_size() {
                            (*first).data.deinit();
                            return ptr::null_mut();
                        }

                        let buf = (*first).data.memory;
                        let buf_len = (*first).data.memory_len;
                        let u64_ptr = buf as *mut u64;
                        let u64_len = buf_len / 8;
                        let mut i = 0usize;
                        while i < u64_len {
                            ptr::write(u64_ptr.add(i), 0);
                            i += 1;
                        }

                        let layout = Page::layout(cap);
                        let obuf = OffsetBuf::init(buf);
                        (*first).data = Page::init_buf(obuf, layout);
                        (*first).data.size.rows = 1;
                        self.pages.insert_after(last, first);
                        self.total_rows += 1;

                        self.page_serial_min = (*first).serial + 1;
                        (*first).serial = self.page_serial;
                        self.page_serial += 1;

                        return first;
                    }
                }
            }

            match Page::init(cap) {
                Ok(page) => {
                    let node_size = core::mem::size_of::<PageListNode>();
                    let aligned = (node_size + 7) & !7;
                    match page_alloc(aligned) {
                        Ok(node_mem) => {
                            let node_ptr = node_mem.as_mut_ptr() as *mut PageListNode;
                            ptr::write(
                                node_ptr,
                                PageListNode {
                                    prev: ptr::null_mut(),
                                    next: ptr::null_mut(),
                                    data: page,
                                    serial: self.page_serial,
                                },
                            );
                            (*node_ptr).data.size.rows = 1;
                            self.pages.append(node_ptr);
                            self.page_serial += 1;
                            self.page_size += (*node_ptr).data.memory_len;
                            self.total_rows += 1;
                            node_ptr
                        }
                        Err(()) => ptr::null_mut(),
                    }
                }
                Err(()) => ptr::null_mut(),
            }
        }
    }

    pub fn grow_rows(&mut self, n: usize) {
        for _ in 0..n {
            self.grow();
        }
    }

    // -- Scroll -------------------------------------------------------------

    pub fn scroll(&mut self, behavior: PageListScroll) {
        if self.explicit_max_size == 0 {
            self.viewport = PageListViewport::Active;
            return;
        }

        match behavior {
            PageListScroll::Active => {
                self.viewport = PageListViewport::Active;
            }
            PageListScroll::Top => {
                self.viewport = PageListViewport::Top;
            }
            PageListScroll::Pin(p) => {
                if self.pin_is_active(p) {
                    self.viewport = PageListViewport::Active;
                    return;
                } else if self.pin_is_top(p) {
                    self.viewport = PageListViewport::Top;
                    return;
                }
                unsafe {
                    ptr::write(self.viewport_pin as *mut Pin, p);
                }
                self.viewport = PageListViewport::Pin;
                self.viewport_pin_row_offset = 0;
            }
            PageListScroll::Row(n) => {
                if n == 0 {
                    self.viewport = PageListViewport::Top;
                    return;
                }
                if n >= self.total_rows - self.rows as usize {
                    self.viewport = PageListViewport::Active;
                    return;
                }
                match self.viewport {
                    PageListViewport::Pin if self.viewport_pin_row_offset > 0 => {
                        let cached = self.viewport_pin_row_offset - 1;
                        let delta = n as isize - cached as isize;
                        self.scroll(PageListScroll::DeltaRow(delta));
                        return;
                    }
                    _ => {}
                }
                self.viewport_pin_row_offset = n + 1;
                self.viewport = PageListViewport::Pin;

                let midpoint = self.total_rows / 2;
                if n < midpoint {
                    let mut node_it = self.pages.first;
                    let mut rem = n as usize;
                    while !node_it.is_null() {
                        unsafe {
                            let nr = (*node_it).data.size.rows as usize;
                            if rem < nr {
                                ptr::write(
                                    self.viewport_pin as *mut Pin,
                                    Pin {
                                        node: node_it,
                                        y: rem as CellCountInt,
                                        x: 0,
                                        garbage: false,
                                    },
                                );
                                return;
                            }
                            rem -= nr;
                            node_it = (*node_it).next;
                        }
                    }
                } else {
                    let mut node_it = self.pages.last;
                    let mut rem = self.total_rows - n;
                    while !node_it.is_null() {
                        unsafe {
                            let nr = (*node_it).data.size.rows as usize;
                            if rem <= nr {
                                ptr::write(
                                    self.viewport_pin as *mut Pin,
                                    Pin {
                                        node: node_it,
                                        y: (nr - rem) as CellCountInt,
                                        x: 0,
                                        garbage: false,
                                    },
                                );
                                return;
                            }
                            rem -= nr;
                            node_it = (*node_it).prev;
                        }
                    }
                }

                self.viewport = PageListViewport::Active;
            }
            PageListScroll::DeltaRow(n) => {
                match self.viewport {
                    PageListViewport::Top if n <= 0 => return,
                    PageListViewport::Active if n >= 0 => return,
                    PageListViewport::Pin => {
                        if n == 0 {
                            return;
                        }
                        unsafe {
                            let vp_pin = *(self.viewport_pin as *const Pin);
                            if n < 0 {
                                match vp_pin.up_overflow((-n) as usize) {
                                    PinMoveResult::Offset(new_pin) => {
                                        ptr::write(self.viewport_pin as *mut Pin, new_pin);
                                        if self.viewport_pin_row_offset > 0 {
                                            self.viewport_pin_row_offset -= (-n) as usize;
                                        }
                                        return;
                                    }
                                    PinMoveResult::Overflow(_, _) => {
                                        self.viewport = PageListViewport::Top;
                                        return;
                                    }
                                }
                            } else {
                                match vp_pin.down_overflow(n as usize) {
                                    PinMoveResult::Offset(new_pin) => {
                                        if self.pin_is_active(new_pin) {
                                            self.viewport = PageListViewport::Active;
                                        } else {
                                            ptr::write(self.viewport_pin as *mut Pin, new_pin);
                                            if self.viewport_pin_row_offset > 0 {
                                                self.viewport_pin_row_offset += n as usize;
                                            }
                                        }
                                        return;
                                    }
                                    PinMoveResult::Overflow(_, _) => {
                                        self.viewport = PageListViewport::Active;
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                let top = self.get_top_left(PointTag::VIEWPORT);
                let p = if n < 0 {
                    match top.up_overflow((-n) as usize) {
                        PinMoveResult::Offset(v) => v,
                        PinMoveResult::Overflow(end, _) => end,
                    }
                } else {
                    match top.down_overflow(n as usize) {
                        PinMoveResult::Offset(v) => v,
                        PinMoveResult::Overflow(end, _) => end,
                    }
                };

                if self.pin_is_active(p) {
                    self.viewport = PageListViewport::Active;
                    return;
                }
                if self.pin_is_top(p) {
                    self.viewport = PageListViewport::Top;
                    return;
                }
                unsafe {
                    ptr::write(self.viewport_pin as *mut Pin, p);
                }
                self.viewport = PageListViewport::Pin;
                self.viewport_pin_row_offset = 0;
            }
            PageListScroll::DeltaPrompt(n) => {
                if n == 0 {
                    return;
                }
                let delta_rem = if n > 0 { n as usize } else { (-n) as usize };
                let start_pin = {
                    let tl = self.get_top_left(PointTag::VIEWPORT);
                    if n <= 0 {
                        match tl.up(1) {
                            Some(p) => p,
                            None => return,
                        }
                    } else {
                        let mut adjusted = match tl.down(1) {
                            Some(p) => p,
                            None => return,
                        };
                        unsafe {
                            let row = adjusted.row_ptr();
                            if (*row).semantic_prompt() != SemanticPrompt::None {
                                loop {
                                    let row2 = adjusted.row_ptr();
                                    if (*row2).semantic_prompt()
                                        != SemanticPrompt::PromptContinuation
                                    {
                                        break;
                                    }
                                    match adjusted.down(1) {
                                        Some(p) => adjusted = p,
                                        None => break,
                                    }
                                }
                            }
                        }
                        adjusted
                    }
                };

                let dir = if n > 0 {
                    PageListDirection::RightDown
                } else {
                    PageListDirection::LeftUp
                };
                let mut it = PromptIterator::new(Some(start_pin), None, dir);
                let mut prompt_pin: Option<Pin> = None;
                let mut rem = delta_rem;
                while let Some(next) = it.next() {
                    prompt_pin = Some(next);
                    rem -= 1;
                    if rem == 0 {
                        break;
                    }
                }

                if let Some(p) = prompt_pin {
                    if self.pin_is_active(p) {
                        self.viewport = PageListViewport::Active;
                    } else {
                        unsafe {
                            ptr::write(self.viewport_pin as *mut Pin, p);
                        }
                        self.viewport = PageListViewport::Pin;
                        self.viewport_pin_row_offset = 0;
                    }
                }
            }
        }
    }

    pub fn scroll_clear(&mut self) -> Result<(), ()> {
        unsafe {
            let last = self.pages.last;
            if last.is_null() {
                return Ok(());
            }

            let mut page = last;
            let mut n: usize = 0;
            loop {
                let page_ref = &(*page).data;
                let rows = page_ref.size.rows as usize;
                let cols = self.cols as usize;
                let rows_base = page_ref.rows_ptr();

                for i in 0..rows {
                    let rev_i = rows - i - 1;
                    let row = rows_base.add(rev_i);
                    let cells = page_ref.row_cells_ptr(row);
                    let mut found_nonempty = false;
                    for c in 0..cols {
                        if !(*cells.add(c)).is_empty() {
                            found_nonempty = true;
                            break;
                        }
                    }
                    if found_nonempty {
                        let non_empty = self.rows as usize - n;
                        for _ in 0..non_empty {
                            self.grow();
                        }
                        return Ok(());
                    }
                    n += 1;
                    if n > self.rows as usize {
                        return Ok(());
                    }
                }

                let prev = (*page).prev;
                if prev.is_null() {
                    return Ok(());
                }
                page = prev;
            }
        }
    }

    // -- Clear dirty --------------------------------------------------------

    pub fn clear_dirty(&mut self) {
        let mut page = self.pages.first;
        while !page.is_null() {
            unsafe {
                (*page).data.dirty = false;
                let rows = (*page).data.size.rows as usize;
                let base = (*page).data.rows_ptr();
                for i in 0..rows {
                    let row = base.add(i);
                    (*row).set_dirty(false);
                }
                page = (*page).next;
            }
        }
    }

    // -- Resize ---------------------------------------------------------------

    pub fn resize(&mut self, opts: &PageListResize) {
        self.viewport_pin_row_offset = 0;

        let new_cols = if opts.cols != 0 { opts.cols } else { self.cols };
        let new_rows = if opts.rows != 0 { opts.rows } else { self.rows };

        if opts.reflow {
            self.min_max_size = self.calc_min_max_size(new_cols, new_rows);

            if new_cols == self.cols {
                self.resize_without_reflow_no_cols(new_rows);
            } else if new_cols > self.cols {
                self.resize_cols(new_cols);
                self.resize_without_reflow_no_cols(new_rows);
            } else {
                self.resize_without_reflow_no_cols(new_rows);
                self.resize_cols(new_cols);
            }

            match self.viewport {
                PageListViewport::Pin => unsafe {
                    let vp_pin = *(self.viewport_pin as *const Pin);
                    if self.pin_is_active(vp_pin) {
                        self.viewport = PageListViewport::Active;
                    }
                },
                _ => {}
            }
        } else {
            self.min_max_size = self.calc_min_max_size(new_cols, new_rows);

            if new_cols != self.cols && new_cols < self.cols {
                let old_cols = self.cols;
                unsafe {
                    let mut it = self.pages.first;
                    while !it.is_null() {
                        let page = &mut (*it).data;
                        let rows = page.size.rows as usize;
                        let base = page.rows_ptr();
                        for i in 0..rows {
                            let row = base.add(i);
                            let cells = page.row_cells_ptr(row);
                            for c in (new_cols as usize)..(old_cols as usize) {
                                *cells.add(c) = Cell::default();
                            }
                        }
                        page.size.cols = new_cols;
                        it = (*it).next;
                    }
                }
                self.cols = new_cols;
            } else if new_cols != self.cols && new_cols > self.cols {
                self.resize_without_reflow_grow_cols(new_cols);
            }

            self.resize_without_reflow_no_cols(new_rows);
        }
    }

    fn resize_without_reflow_no_cols(&mut self, new_rows: CellCountInt) {
        if new_rows < self.rows {
            let trimmed = self.trim_trailing_blank_rows(self.rows - new_rows);
            self.total_rows -= trimmed as usize;
            self.rows = new_rows;
        } else if new_rows > self.rows {
            self.rows = new_rows;
            let mut count: usize = 0;
            let mut page = self.pages.first;
            while !page.is_null() {
                unsafe {
                    count += (*page).data.size.rows as usize;
                    if count >= new_rows as usize {
                        break;
                    }
                    page = (*page).next;
                }
            }
            if count < new_rows as usize {
                let needed = new_rows as usize - count;
                for _ in 0..needed {
                    self.grow();
                }
            }
            match self.viewport {
                PageListViewport::Pin => unsafe {
                    let vp_pin = *(self.viewport_pin as *const Pin);
                    if self.pin_is_active(vp_pin) {
                        self.viewport = PageListViewport::Active;
                    }
                },
                _ => {}
            }
        }
    }

    fn resize_cols(&mut self, new_cols: CellCountInt) {
        unsafe {
            let first_old = self.pages.first;
            if first_old.is_null() {
                return;
            }

            let cap: PageCapacity = match (*first_old).data.capacity.adjust(&CapacityAdjustment {
                cols: Some(new_cols),
            }) {
                Ok(c) => c,
                Err(()) => {
                    let mut fallback = (*first_old).data.capacity;
                    fallback.cols = new_cols;
                    let std_rows = std_capacity().rows;
                    fallback.rows = if (*first_old).data.size.rows < std_rows {
                        (*first_old).data.size.rows
                    } else {
                        std_rows
                    };
                    fallback
                }
            };

            self.cols = new_cols;

            let first_new_node = match self.alloc_page_node(cap) {
                Some(n) => n,
                None => return,
            };
            (*first_new_node).data.size.rows = 1;

            let mut row_it = self.row_iterator(
                PageListDirection::RightDown,
                PointTag::SCREEN,
                0,
                0,
                None,
                None,
                None,
            );

            self.pages.first = first_new_node;
            self.pages.last = first_new_node;

            let mut reflow_cursor = ReflowCursor::init(first_new_node);
            while let Some(row_pin) = row_it.next() {
                let row_node = row_pin.node;
                let row_y = row_pin.y;
                let last_row_in_page = (*row_node).data.size.rows - 1;

                let _ = reflow_cursor.reflow_row(
                    self as *mut PageList,
                    row_pin,
                    None,
                );

                if row_y == last_row_in_page {
                    self.destroy_page_node_full(row_node);
                }
            }

            self.total_rows = reflow_cursor.total_rows;

            let mut total: usize = 0;
            let mut node_it = self.pages.first;
            while !node_it.is_null() {
                total += (*node_it).data.size.rows as usize;
                if total >= self.rows as usize {
                    break;
                }
                node_it = (*node_it).next;
            }
            if total < self.rows as usize {
                let needed = self.rows as usize - total;
                for _ in 0..needed {
                    self.grow();
                }
            }
        }
    }

    fn resize_without_reflow_grow_cols(&mut self, new_cols: CellCountInt) {
        let old_cols = self.cols;
        unsafe {
            let mut it = self.pages.first;
            while !it.is_null() {
                let node = it;
                it = (*node).next;
                let page: *mut Page = &mut (*node).data;

                if (*page).capacity.cols >= new_cols {
                    let rows_ptr_val = (*page).rows_ptr();
                    let rows = (*page).size.rows as usize;
                    let old_last_x = if old_cols > 0 {
                        (old_cols - 1) as usize
                    } else {
                        0
                    };
                    let mut spacer_found = false;
                    let mut i = 0usize;
                    while i < rows {
                        let row = rows_ptr_val.add(i);
                        let cells = (*page).row_cells_ptr(row);
                        let cell = cells.add(old_last_x);
                        if (*cell).wide() == Wide::SpacerHead {
                            spacer_found = true;
                            break;
                        }
                        i += 1;
                    }

                    if spacer_found {
                        i = 0;
                        while i < rows {
                            let row = rows_ptr_val.add(i);
                            let cells = (*page).row_cells_ptr(row);
                            let cell = cells.add(old_last_x);
                            if (*cell).wide() == Wide::SpacerHead {
                                ptr::write(cell, Cell::default());
                            }
                            i += 1;
                        }
                    }

                    (*page).size.cols = new_cols;
                }
            }
        }
        self.cols = new_cols;
    }

    unsafe fn alloc_page_node(&mut self, cap: PageCapacity) -> Option<*mut PageListNode> {
        unsafe {
            let page = match Page::init(cap) {
                Ok(p) => p,
                Err(()) => return None,
            };
            let node_size = core::mem::size_of::<PageListNode>();
            let aligned = (node_size + 7) & !7;
            let node_mem = match page_alloc(aligned) {
                Ok(m) => m,
                Err(()) => {
                    let mut orphan = page;
                    orphan.deinit();
                    return None;
                }
            };
            let node_ptr = node_mem.as_mut_ptr() as *mut PageListNode;
            ptr::write(
                node_ptr,
                PageListNode {
                    prev: ptr::null_mut(),
                    next: ptr::null_mut(),
                    data: page,
                    serial: self.page_serial,
                },
            );
            self.page_serial += 1;
            self.page_size += (*node_ptr).data.memory_len;
            Some(node_ptr)
        }
    }

    unsafe fn destroy_page_node_full(&mut self, node: *mut PageListNode) {
        unsafe {
            let node_page: *mut Page = &mut (*node).data;
            let page_mem_len = (*node_page).memory_len;
            self.page_size = self.page_size.saturating_sub(page_mem_len);
            (*node_page).deinit();

            let node_size = core::mem::size_of::<PageListNode>();
            let aligned = (node_size + 7) & !7;
            let node_slice = core::slice::from_raw_parts_mut(node as *mut u8, aligned);
            page_free(node_slice);
        }
    }

    fn trim_trailing_blank_rows(&mut self, max: CellCountInt) -> CellCountInt {
        let mut trimmed: CellCountInt = 0;
        let bl = match self.get_bottom_right(PointTag::SCREEN) {
            Some(p) => p,
            None => return 0,
        };
        let mut it_row = bl;
        loop {
            unsafe {
                let page = &(*it_row.node).data;
                let row = page.get_row(it_row.y as usize);
                let cells = page.row_cells_ptr(row);
                let cols = page.size.cols as usize;
                let mut has_text = false;
                for c in 0..cols {
                    if !(*cells.add(c)).is_empty() {
                        has_text = true;
                        break;
                    }
                }
                if has_text {
                    return trimmed;
                }

                if !self.tracked_pins.is_null() {
                    let tp = &*(self.tracked_pins as *const TrackedPinArray);
                    if !tp.keys.is_null() {
                        let node = it_row.node;
                        let y = it_row.y;
                        let mut i = 0usize;
                        while i < tp.len {
                            let p = *tp.keys.add(i);
                            if !p.is_null() && (*p).node == node && (*p).y == y {
                                return trimmed;
                            }
                            i += 1;
                        }
                    }
                }

                let node = it_row.node;
                (*node).data.size.rows -= 1;
                if (*node).data.size.rows == 0 {
                    self.erase_page(node);
                }
            }

            trimmed += 1;
            if trimmed >= max {
                return trimmed;
            }

            match it_row.up(1) {
                Some(p) => it_row = p,
                None => break,
            }
        }
        trimmed
    }

    fn erase_page(&mut self, node: *mut PageListNode) {
        unsafe {
            let has_prev = !(*node).prev.is_null();
            let has_next = !(*node).next.is_null();
            if !has_prev && !has_next {
                return;
            }
            if !has_prev {
                if let Some(next) = (*node).next.as_ref() {
                    self.page_serial_min = next.serial;
                }
            }

            let target = if has_prev { (*node).prev } else { (*node).next };
            if !self.tracked_pins.is_null() {
                let tp = &*(self.tracked_pins as *const TrackedPinArray);
                if !tp.keys.is_null() {
                    let mut i = 0usize;
                    while i < tp.len {
                        let p = *tp.keys.add(i);
                        if !p.is_null() && (*p).node == node {
                            (*p).node = target;
                            (*p).y = 0;
                            (*p).x = 0;
                        }
                        i += 1;
                    }
                }
            }

            self.pages.remove(node);
            self.page_size = self.page_size.saturating_sub((*node).data.memory_len);
            (*node).data.deinit();
        }
    }

    // -- Init / deinit / reset (init_new is MEMORYPOOL-BOUND) ---------------

    /// Construct a zeroed PageList shell.
    ///
    /// MEMORYPOOL-BOUND: Zig's `PageList.init` (PageList.zig:358) allocates
    /// a `MemoryPool`, initial pages via the pool, a `viewport_pin` from
    /// `pool.pins.create()`, and a `tracked_pins` hash map. All of these
    /// require the Zig allocator and cannot be replicated in pure Rust.
    /// This constructor produces a structurally valid but pool-less shell
    /// that callers must wire up through the Zig FFI before use.
    pub fn init_new(cols: CellCountInt, rows: CellCountInt) -> PageList {
        PageList {
            pool: ptr::null_mut(),
            pages: PageListHead {
                first: ptr::null_mut(),
                last: ptr::null_mut(),
            },
            page_serial: 0,
            page_serial_min: 0,
            page_size: 0,
            explicit_max_size: usize::MAX,
            min_max_size: 0,
            total_rows: rows as usize,
            tracked_pins: ptr::null_mut(),
            viewport: PageListViewport::Active,
            viewport_pin: ptr::null_mut(),
            viewport_pin_row_offset: 0,
            cols,
            rows,
        }
    }

    pub fn deinit_pages(&mut self) {
        let mut it = self.pages.first;
        while !it.is_null() {
            unsafe {
                let next = (*it).next;
                self.destroy_page_node_full(it);
                it = next;
            }
        }
        self.pages.first = ptr::null_mut();
        self.pages.last = ptr::null_mut();
        self.page_size = 0;
    }

    pub fn reset(&mut self) {
        self.page_serial_min = self.page_serial;

        let cap = std_capacity();
        let rows_per_page = cap.rows as usize;
        let pages_needed = if self.rows as usize == 0 {
            1
        } else {
            (self.rows as usize + rows_per_page - 1) / rows_per_page
        };

        let mut page_count = 0usize;
        let mut it = self.pages.first;
        let mut rows_left = self.rows as usize;

        while !it.is_null() {
            unsafe {
                let next = (*it).next;
                page_count += 1;

                if page_count <= pages_needed {
                    (*it).data.reinit();
                    let page_rows = if rows_left >= rows_per_page {
                        rows_per_page
                    } else {
                        rows_left
                    };
                    (*it).data.size.rows = page_rows as CellCountInt;
                    (*it).data.size.cols = self.cols;
                    rows_left = rows_left.saturating_sub(page_rows);
                } else {
                    self.pages.remove(it);
                    self.destroy_page_node_full(it);
                }

                it = next;
            }
        }

        self.total_rows = self.rows as usize;

        if !self.tracked_pins.is_null() {
            unsafe {
                let tp = &*(self.tracked_pins as *const TrackedPinArray);
                if !tp.keys.is_null() {
                    let first = self.pages.first;
                    let mut i = 0usize;
                    while i < tp.len {
                        let p = *tp.keys.add(i);
                        if !p.is_null() {
                            (*p).node = first;
                            (*p).y = 0;
                            (*p).x = 0;
                            (*p).garbage = true;
                        }
                        i += 1;
                    }
                }
            }
        }

        if !self.viewport_pin.is_null() {
            unsafe {
                ptr::write(
                    self.viewport_pin as *mut Pin,
                    Pin {
                        node: self.pages.first,
                        y: 0,
                        x: 0,
                        garbage: false,
                    },
                );
            }
        }

        self.viewport = PageListViewport::Active;
        self.viewport_pin_row_offset = 0;
    }

    // -- Helper: min max size calculation ------------------------------------

    fn calc_min_max_size(&self, _cols: CellCountInt, rows: CellCountInt) -> usize {
        let cap = std_capacity();
        let pages_exact = if cap.rows >= rows {
            1usize
        } else {
            (rows as usize + cap.rows as usize - 1) / cap.rows as usize
        };
        let pages = pages_exact + 1;
        // Use the standard page size * number of pages
        // Since we don't have the exact pool item size, approximate:
        let layout = Page::layout(cap);
        layout.total_size * pages
    }

    // -- Fixup viewport after row removal -----------------------------------

    pub fn fixup_viewport(&mut self, removed: usize) {
        match self.viewport {
            PageListViewport::Active => {}
            PageListViewport::Pin => unsafe {
                let vp_pin = *(self.viewport_pin as *const Pin);
                if self.pin_is_active(vp_pin) {
                    self.viewport = PageListViewport::Active;
                } else if self.viewport_pin_row_offset > 0 {
                    let v = self.viewport_pin_row_offset - 1;
                    if v < removed {
                        self.viewport = PageListViewport::Top;
                    } else {
                        self.viewport_pin_row_offset = v - removed + 1;
                    }
                }
            },
            PageListViewport::Top => {
                let first = self.pages.first;
                if !first.is_null()
                    && self.pin_is_active(Pin {
                        node: first,
                        y: 0,
                        x: 0,
                        garbage: false,
                    })
                {
                    self.viewport = PageListViewport::Active;
                }
            }
        }
    }
}
