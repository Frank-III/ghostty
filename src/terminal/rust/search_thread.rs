use core::ffi::c_void;
use core::ptr;

use crate::allocator::{GhosttyAllocator, alloc_alloc_impl, alloc_free_impl};
use crate::highlight::HighlightFlattened;
use crate::search::search_screen::{ScreenSearch, SelectDirection};
use crate::search::search_viewport::ViewportSearch;

pub const REFRESH_INTERVAL: u32 = 24;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchTickResult {
    Complete = 0,
    Progress = 1,
    Blocked = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchEventType {
    Quit = 0,
    Complete = 1,
    TotalMatches = 2,
    SelectedMatch = 3,
    ViewportMatches = 4,
}

pub struct SearchEvent {
    pub event_type: SearchEventType,
    pub total_matches: usize,
    pub selected_idx: usize,
    pub selected_highlight: HighlightFlattened,
    pub viewport_matches_ptr: *const HighlightFlattened,
    pub viewport_matches_len: usize,
}

impl SearchEvent {
    pub fn quit() -> Self {
        Self {
            event_type: SearchEventType::Quit,
            total_matches: 0,
            selected_idx: 0,
            selected_highlight: HighlightFlattened::empty(),
            viewport_matches_ptr: ptr::null(),
            viewport_matches_len: 0,
        }
    }

    pub fn complete() -> Self {
        Self {
            event_type: SearchEventType::Complete,
            total_matches: 0,
            selected_idx: 0,
            selected_highlight: HighlightFlattened::empty(),
            viewport_matches_ptr: ptr::null(),
            viewport_matches_len: 0,
        }
    }

    pub fn total_matches(n: usize) -> Self {
        Self {
            event_type: SearchEventType::TotalMatches,
            total_matches: n,
            selected_idx: 0,
            selected_highlight: HighlightFlattened::empty(),
            viewport_matches_ptr: ptr::null(),
            viewport_matches_len: 0,
        }
    }
}

pub type EventCallback = unsafe extern "C" fn(event: *const SearchEvent, userdata: *mut c_void);

pub struct SearchThreadOptions {
    pub mutex: *mut c_void,
    pub terminal: *mut c_void,
    pub event_cb: Option<EventCallback>,
    pub event_userdata: *mut c_void,
}

pub struct ScreenState {
    pub key: u8,
    pub total: usize,
    pub has_total: bool,
    pub selected_idx: usize,
    pub has_selected: bool,
}

impl ScreenState {
    pub fn default_primary() -> Self {
        Self {
            key: 0,
            total: 0,
            has_total: false,
            selected_idx: 0,
            has_selected: false,
        }
    }
}

pub struct SearchState {
    pub viewport: ViewportSearch,
    pub primary_screen: *mut ScreenSearch,
    pub alt_screen: *mut ScreenSearch,
    pub last_screen: ScreenState,
    pub last_complete: bool,
    pub stale_viewport_matches: bool,
    pub alloc: *const GhosttyAllocator,
}

impl SearchState {
    pub unsafe fn init(alloc: *const GhosttyAllocator, needle: &[u8]) -> SearchState {
        let vp = unsafe { ViewportSearch::init(alloc, needle) };
        SearchState {
            viewport: vp,
            primary_screen: ptr::null_mut(),
            alt_screen: ptr::null_mut(),
            last_screen: ScreenState::default_primary(),
            last_complete: false,
            stale_viewport_matches: true,
            alloc,
        }
    }

    pub unsafe fn deinit(&mut self) {
        unsafe {
            self.viewport.deinit();
            if !self.primary_screen.is_null() {
                (*self.primary_screen).deinit();
                let ss_size = core::mem::size_of::<ScreenSearch>();
                alloc_free_impl(self.alloc, self.primary_screen as *mut u8, ss_size);
                self.primary_screen = ptr::null_mut();
            }
            if !self.alt_screen.is_null() {
                (*self.alt_screen).deinit();
                let ss_size = core::mem::size_of::<ScreenSearch>();
                alloc_free_impl(self.alloc, self.alt_screen as *mut u8, ss_size);
                self.alt_screen = ptr::null_mut();
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        let primary_done = if self.primary_screen.is_null() {
            true
        } else {
            unsafe { (*self.primary_screen).state.is_complete() }
        };
        let alt_done = if self.alt_screen.is_null() {
            true
        } else {
            unsafe { (*self.alt_screen).state.is_complete() }
        };
        primary_done && alt_done
    }

    pub unsafe fn tick(&mut self) -> SearchTickResult {
        let mut result = SearchTickResult::Complete;

        if !self.primary_screen.is_null() {
            unsafe {
                let r = (*self.primary_screen).tick();
                match r {
                    crate::search::search_screen::TickResult::Ok => {
                        result = SearchTickResult::Progress;
                    }
                    crate::search::search_screen::TickResult::FeedRequired => {
                        if result == SearchTickResult::Complete {
                            result = SearchTickResult::Blocked;
                        }
                    }
                    _ => {}
                }
            }
        }

        if !self.alt_screen.is_null() {
            unsafe {
                let r = (*self.alt_screen).tick();
                match r {
                    crate::search::search_screen::TickResult::Ok => {
                        result = SearchTickResult::Progress;
                    }
                    crate::search::search_screen::TickResult::FeedRequired => {
                        if result == SearchTickResult::Complete {
                            result = SearchTickResult::Blocked;
                        }
                    }
                    _ => {}
                }
            }
        }

        result
    }

    pub unsafe fn feed(&mut self, _terminal: *mut c_void) {
        unsafe {
            if !self.primary_screen.is_null() {
                let needs = (*self.primary_screen).state.needs_feed();
                if needs {
                    (*self.primary_screen).feed();
                }
            }
            if !self.alt_screen.is_null() {
                let needs = (*self.alt_screen).state.needs_feed();
                if needs {
                    (*self.alt_screen).feed();
                }
            }
        }
    }
}

pub struct SearchThread {
    pub alloc: *const GhosttyAllocator,
    pub mailbox: *mut c_void,
    pub search: *mut SearchState,
    pub opts: SearchThreadOptions,
    pub refresh_active: bool,
}

impl SearchThread {
    pub unsafe fn init(alloc: *const GhosttyAllocator, opts: SearchThreadOptions) -> SearchThread {
        SearchThread {
            alloc,
            mailbox: ptr::null_mut(),
            search: ptr::null_mut(),
            opts,
            refresh_active: false,
        }
    }

    pub unsafe fn deinit(&mut self) {
        unsafe {
            if !self.search.is_null() {
                (*self.search).deinit();
                let ss_size = core::mem::size_of::<SearchState>();
                alloc_free_impl(self.alloc, self.search as *mut u8, ss_size);
                self.search = ptr::null_mut();
            }
        }
    }

    pub unsafe fn change_needle(&mut self, needle: &[u8]) {
        unsafe {
            if !self.search.is_null() {
                let current = (*self.search).viewport.needle();
                if ascii_equal_ignore_case(current, needle) {
                    return;
                }

                (*self.search).deinit();
                let ss_size = core::mem::size_of::<SearchState>();
                alloc_free_impl(self.alloc, self.search as *mut u8, ss_size);
                self.search = ptr::null_mut();

                if let Some(cb) = self.opts.event_cb {
                    let ev = SearchEvent::total_matches(0);
                    cb(&ev, self.opts.event_userdata);
                }
            }

            if needle.is_empty() {
                return;
            }

            let ss_size = core::mem::size_of::<SearchState>();
            let new_search = alloc_alloc_impl(self.alloc, ss_size) as *mut SearchState;
            if new_search.is_null() {
                return;
            }
            ptr::write(new_search, SearchState::init(self.alloc, needle));
            self.search = new_search;
        }
    }

    pub unsafe fn select(&mut self, direction: SelectDirection) {
        if self.search.is_null() {
            return;
        }
        unsafe {
            let screen = (*self.search).primary_screen;
            if !screen.is_null() {
                (*screen).select(direction);
            }
        }
    }
}

fn ascii_equal_ignore_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        let ca = if a[i] >= b'A' && a[i] <= b'Z' { a[i] + 32 } else { a[i] };
        let cb = if b[i] >= b'A' && b[i] <= b'Z' { b[i] + 32 } else { b[i] };
        if ca != cb {
            return false;
        }
        i += 1;
    }
    true
}
