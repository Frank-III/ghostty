use core::ffi::c_void;
use core::ptr;

use crate::allocator::{GhosttyAllocator, alloc_alloc_impl, alloc_free_impl};
use crate::page_list_types::{PageList, PageListNode};
use crate::highlight::HighlightFlattened;
use crate::search::search_sliding_window::{SlidingWindow, Direction};

pub struct Fingerprint {
    pub nodes_ptr: *mut *mut PageListNode,
    pub nodes_len: usize,
    pub nodes_cap: usize,
}

impl Fingerprint {
    pub fn empty() -> Self {
        Self {
            nodes_ptr: ptr::null_mut(),
            nodes_len: 0,
            nodes_cap: 0,
        }
    }

    pub fn nodes(&self) -> &[*mut PageListNode] {
        if self.nodes_ptr.is_null() || self.nodes_len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.nodes_ptr, self.nodes_len) }
    }

    pub unsafe fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if !self.nodes_ptr.is_null() && self.nodes_cap > 0 {
            let byte_size = self.nodes_cap * core::mem::size_of::<*mut PageListNode>();
            unsafe {
                alloc_free_impl(alloc, self.nodes_ptr as *mut u8, byte_size);
            }
            self.nodes_ptr = ptr::null_mut();
            self.nodes_len = 0;
            self.nodes_cap = 0;
        }
    }

    pub fn eql(&self, other: &Fingerprint) -> bool {
        let a = self.nodes();
        let b = other.nodes();
        if a.len() != b.len() {
            return false;
        }
        let mut i = 0;
        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

pub struct ViewportSearch {
    pub window: *mut SlidingWindow,
    pub fingerprint: *mut Fingerprint,
    pub active_dirty: *mut bool,
    pub alloc: *const GhosttyAllocator,
}

impl ViewportSearch {
    pub unsafe fn init(
        alloc: *const GhosttyAllocator,
        needle: &[u8],
    ) -> ViewportSearch {
        let window = unsafe { SlidingWindow::init(alloc, Direction::Forward, needle) };
        ViewportSearch {
            window,
            fingerprint: ptr::null_mut(),
            active_dirty: ptr::null_mut(),
            alloc,
        }
    }

    pub unsafe fn deinit(&mut self) {
        unsafe {
            if !self.fingerprint.is_null() {
                (*self.fingerprint).deinit(self.alloc);
                let fp_size = core::mem::size_of::<Fingerprint>();
                alloc_free_impl(self.alloc, self.fingerprint as *mut u8, fp_size);
                self.fingerprint = ptr::null_mut();
            }
            if !self.active_dirty.is_null() {
                alloc_free_impl(self.alloc, self.active_dirty as *mut u8, 1);
                self.active_dirty = ptr::null_mut();
            }
            if !self.window.is_null() {
                SlidingWindow::deinit(self.window);
                self.window = ptr::null_mut();
            }
        }
    }

    pub unsafe fn reset(&mut self) {
        unsafe {
            if !self.fingerprint.is_null() {
                (*self.fingerprint).deinit(self.alloc);
                let fp_size = core::mem::size_of::<Fingerprint>();
                alloc_free_impl(self.alloc, self.fingerprint as *mut u8, fp_size);
                self.fingerprint = ptr::null_mut();
            }
            if !self.window.is_null() {
                SlidingWindow::clear_and_retain_capacity(self.window);
            }
        }
    }

    pub fn needle(&self) -> &[u8] {
        unsafe { SlidingWindow::needle_slice(self.window) }
    }

    pub unsafe fn update(&mut self, list: *const PageList) -> bool {
        if self.window.is_null() || list.is_null() {
            return false;
        }
        true
    }

    pub unsafe fn next(&mut self) -> HighlightFlattened {
        if self.window.is_null() {
            return HighlightFlattened::empty();
        }
        unsafe { SlidingWindow::next(self.window) }
    }
}
