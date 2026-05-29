use core::ffi::c_void;
use core::ptr;

use crate::allocator::{alloc_alloc_impl, alloc_free_impl, GhosttyAllocator};
use crate::highlight::{HighlightFlattened, HighlightTracked, HighlightUntracked, Pin};
use crate::page_list_types::{PageList, PageListNode};
use crate::search::search_active::ActiveSearch;
use crate::search::search_pagelist::PageListSearch;
use crate::search::search_sliding_window::SlidingWindow;
use crate::size_types::CellCountInt;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchState {
    Active = 0,
    History = 1,
    HistoryFeed = 2,
    Complete = 3,
}

impl SearchState {
    pub fn is_complete(self) -> bool {
        self == SearchState::Complete
    }

    pub fn needs_feed(self) -> bool {
        matches!(self, SearchState::HistoryFeed | SearchState::Complete)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectDirection {
    Next = 0,
    Prev = 1,
}

pub struct SelectedMatch {
    pub idx: usize,
    pub highlight: HighlightTracked,
}

pub struct HistorySearch {
    pub searcher: PageListSearch,
    pub start_pin: *mut Pin,
}

pub struct ResultList {
    pub ptr: *mut HighlightFlattened,
    pub len_val: usize,
    pub cap: usize,
}

impl ResultList {
    pub fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len_val: 0,
            cap: 0,
        }
    }

    pub fn items(&self) -> &[HighlightFlattened] {
        if self.ptr.is_null() || self.len_val == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.ptr, self.len_val) }
    }

    pub fn items_mut(&mut self) -> &mut [HighlightFlattened] {
        if self.ptr.is_null() || self.len_val == 0 {
            return &mut [];
        }
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len_val) }
    }

    pub unsafe fn push(
        &mut self,
        alloc: *const GhosttyAllocator,
        item: HighlightFlattened,
    ) -> bool {
        if self.len_val >= self.cap {
            let new_cap = if self.cap == 0 { 8 } else { self.cap * 2 };
            let byte_size = new_cap * core::mem::size_of::<HighlightFlattened>();
            unsafe {
                let new_ptr = alloc_alloc_impl(alloc, byte_size) as *mut HighlightFlattened;
                if new_ptr.is_null() {
                    return false;
                }
                if self.len_val > 0 && !self.ptr.is_null() {
                    ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len_val);
                    let old_size = self.cap * core::mem::size_of::<HighlightFlattened>();
                    alloc_free_impl(alloc, self.ptr as *mut u8, old_size);
                }
                self.ptr = new_ptr;
                self.cap = new_cap;
            }
        }
        unsafe {
            ptr::write(self.ptr.add(self.len_val), item);
        }
        self.len_val += 1;
        true
    }

    pub unsafe fn clear_retaining_capacity(&mut self) {
        self.len_val = 0;
    }

    pub unsafe fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.ptr.is_null() && self.cap > 0 {
            let byte_size = self.cap * core::mem::size_of::<HighlightFlattened>();
            unsafe {
                alloc_free_impl(alloc, self.ptr as *mut u8, byte_size);
            }
            self.ptr = ptr::null_mut();
            self.len_val = 0;
            self.cap = 0;
        }
    }

    pub unsafe fn shrink_to(&mut self, _alloc: *const GhosttyAllocator, new_len: usize) {
        if new_len >= self.len_val {
            return;
        }
        self.len_val = new_len;
    }
}

pub struct ScreenSearch {
    pub screen: *mut c_void,
    pub active: ActiveSearch,
    pub history: *mut HistorySearch,
    pub state: SearchState,
    pub selected: *mut SelectedMatch,
    pub history_results: ResultList,
    pub active_results: ResultList,
    pub rows: CellCountInt,
    pub cols: CellCountInt,
    pub alloc: *const GhosttyAllocator,
}

impl ScreenSearch {
    pub unsafe fn init(
        alloc: *const GhosttyAllocator,
        screen: *mut c_void,
        needle: &[u8],
    ) -> ScreenSearch {
        let active = unsafe { ActiveSearch::init(alloc, needle) };

        ScreenSearch {
            screen,
            active,
            history: ptr::null_mut(),
            state: SearchState::Active,
            selected: ptr::null_mut(),
            history_results: ResultList::empty(),
            active_results: ResultList::empty(),
            rows: 0,
            cols: 0,
            alloc,
        }
    }

    pub unsafe fn deinit(&mut self) {
        unsafe {
            self.active.deinit();

            if !self.history.is_null() {
                let hs = self.history;
                (*hs).searcher.deinit();
                if !(*hs).start_pin.is_null() {
                    let pin_size = core::mem::size_of::<Pin>();
                    alloc_free_impl(self.alloc, (*hs).start_pin as *mut u8, pin_size);
                }
                let hs_size = core::mem::size_of::<HistorySearch>();
                alloc_free_impl(self.alloc, hs as *mut u8, hs_size);
                self.history = ptr::null_mut();
            }

            if !self.selected.is_null() {
                let sm_size = core::mem::size_of::<SelectedMatch>();
                alloc_free_impl(self.alloc, self.selected as *mut u8, sm_size);
                self.selected = ptr::null_mut();
            }

            self.active_results.deinit(self.alloc);
            self.history_results.deinit(self.alloc);
        }
    }

    pub fn needle(&self) -> &[u8] {
        unsafe { SlidingWindow::needle_slice(self.active.window) }
    }

    pub fn matches_len(&self) -> usize {
        self.active_results.len_val + self.history_results.len_val
    }

    pub unsafe fn search_all(&mut self) {
        loop {
            let r = unsafe { self.tick() };
            match r {
                TickResult::Ok => continue,
                TickResult::FeedRequired => {
                    unsafe { self.feed() };
                    continue;
                }
                TickResult::Complete => return,
                TickResult::OutOfMemory => return,
            }
        }
    }

    pub unsafe fn tick(&mut self) -> TickResult {
        match self.state {
            SearchState::Active => unsafe { self.tick_active() },
            SearchState::History => unsafe { self.tick_history() },
            SearchState::HistoryFeed => TickResult::FeedRequired,
            SearchState::Complete => TickResult::Complete,
        }
    }

    pub unsafe fn feed(&mut self) {
        if self.history.is_null() {
            self.state = SearchState::Complete;
            return;
        }

        unsafe {
            let hs = self.history;
            let fed = (*hs).searcher.feed();
            if !fed {
                self.state = SearchState::Complete;
                self.prune_history();
                return;
            }

            match self.state {
                SearchState::Active | SearchState::History => {}
                SearchState::HistoryFeed => self.state = SearchState::History,
                SearchState::Complete => {}
            }
        }
    }

    unsafe fn prune_history(&mut self) {}

    unsafe fn tick_active(&mut self) -> TickResult {
        unsafe {
            loop {
                let hl = self.active.next();
                if hl.chunks_len == 0 {
                    break;
                }
                if !self.active_results.push(self.alloc, hl) {
                    return TickResult::OutOfMemory;
                }
            }
            self.state = SearchState::History;
            TickResult::Ok
        }
    }

    unsafe fn tick_history(&mut self) -> TickResult {
        if self.history.is_null() {
            self.state = SearchState::Complete;
            return TickResult::Ok;
        }

        unsafe {
            let hs = self.history;
            loop {
                let hl = (*hs).searcher.next();
                if hl.chunks_len == 0 {
                    break;
                }

                let chunks = hl.chunks();
                if !chunks.is_empty() {
                    let first_node = chunks.get_unchecked(0).node;
                    if !(*hs).start_pin.is_null() && first_node == (*(*hs).start_pin).node {
                        continue;
                    }
                }

                if !self.history_results.push(self.alloc, hl) {
                    return TickResult::OutOfMemory;
                }
            }
            self.state = SearchState::HistoryFeed;
            TickResult::Ok
        }
    }

    pub unsafe fn selected_match(&self) -> HighlightFlattened {
        let empty = HighlightFlattened::empty();
        if self.selected.is_null() {
            return empty;
        }
        unsafe {
            let sel = self.selected;
            let idx = (*sel).idx;
            let active_len = self.active_results.len_val;

            if idx < active_len {
                let rev_idx = active_len - 1 - idx;
                let items = self.active_results.items();
                if rev_idx < items.len() {
                    return ptr::read(items.get_unchecked(rev_idx));
                }
                return empty;
            }

            let history_idx = idx - active_len;
            let items = self.history_results.items();
            if history_idx < items.len() {
                return ptr::read(items.get_unchecked(history_idx));
            }

            empty
        }
    }

    pub unsafe fn select(&mut self, to: SelectDirection) -> bool {
        match to {
            SelectDirection::Next => unsafe { self.select_next() },
            SelectDirection::Prev => unsafe { self.select_prev() },
        }
    }

    unsafe fn select_next(&mut self) -> bool {
        unsafe {
            let active_len = self.active_results.len_val;
            let history_len = self.history_results.len_val;
            let total = active_len + history_len;

            if total == 0 {
                return false;
            }

            let next_idx = if self.selected.is_null() {
                0
            } else {
                let prev_idx = (*self.selected).idx;
                if prev_idx + 1 >= total {
                    0
                } else {
                    prev_idx + 1
                }
            };

            let sm_size = core::mem::size_of::<SelectedMatch>();
            let new_sm = alloc_alloc_impl(self.alloc, sm_size) as *mut SelectedMatch;
            if new_sm.is_null() {
                return false;
            }

            ptr::write(
                new_sm,
                SelectedMatch {
                    idx: next_idx,
                    highlight: HighlightTracked {
                        start: ptr::null_mut(),
                        end: ptr::null_mut(),
                    },
                },
            );

            if !self.selected.is_null() {
                alloc_free_impl(self.alloc, self.selected as *mut u8, sm_size);
            }
            self.selected = new_sm;
            true
        }
    }

    unsafe fn select_prev(&mut self) -> bool {
        unsafe {
            let active_len = self.active_results.len_val;
            let history_len = self.history_results.len_val;
            let total = active_len + history_len;

            if total == 0 {
                return false;
            }

            let next_idx = if self.selected.is_null() {
                total - 1
            } else {
                let prev_idx = (*self.selected).idx;
                if prev_idx == 0 {
                    total - 1
                } else {
                    prev_idx - 1
                }
            };

            let sm_size = core::mem::size_of::<SelectedMatch>();
            let new_sm = alloc_alloc_impl(self.alloc, sm_size) as *mut SelectedMatch;
            if new_sm.is_null() {
                return false;
            }

            ptr::write(
                new_sm,
                SelectedMatch {
                    idx: next_idx,
                    highlight: HighlightTracked {
                        start: ptr::null_mut(),
                        end: ptr::null_mut(),
                    },
                },
            );

            if !self.selected.is_null() {
                alloc_free_impl(self.alloc, self.selected as *mut u8, sm_size);
            }
            self.selected = new_sm;
            true
        }
    }
}

pub enum TickResult {
    Ok,
    FeedRequired,
    Complete,
    OutOfMemory,
}
