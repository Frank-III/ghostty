use core::ptr;

use crate::allocator::GhosttyAllocator;
use crate::highlight::HighlightFlattened;
use crate::page_list_types::{PageList, PageListNode};
use crate::search::search_sliding_window::{Direction, SlidingWindow};

pub struct ActiveSearch {
    pub window: *mut SlidingWindow,
}

impl ActiveSearch {
    pub unsafe fn init(alloc: *const GhosttyAllocator, needle: &[u8]) -> ActiveSearch {
        let window = unsafe { SlidingWindow::init(alloc, Direction::Forward, needle) };
        ActiveSearch { window }
    }

    pub unsafe fn deinit(&mut self) {
        if !self.window.is_null() {
            unsafe {
                SlidingWindow::deinit(self.window);
            }
            self.window = ptr::null_mut();
        }
    }

    pub unsafe fn update(&mut self, list: *const PageList) -> *mut PageListNode {
        if self.window.is_null() || list.is_null() {
            return ptr::null_mut();
        }
        unsafe {
            SlidingWindow::clear_and_retain_capacity(self.window);

            let rows = (*list).rows as usize;
            let mut rem = rows;
            let mut node_ptr = (*list).pages.last;
            let mut last_node: *mut PageListNode = ptr::null_mut();

            while !node_ptr.is_null() {
                let prev = (*node_ptr).prev;
                SlidingWindow::append(self.window, node_ptr);
                last_node = node_ptr;

                let node_rows = (*node_ptr).data.size.rows as usize;
                if rem <= node_rows {
                    node_ptr = prev;
                    break;
                }
                rem -= node_rows;
                node_ptr = prev;
            }

            let needle_len = (*self.window).needle_len;
            if needle_len > 1 {
                let mut added: usize = 0;
                while !node_ptr.is_null() {
                    let prev = (*node_ptr).prev;
                    let node_rows = (*node_ptr).data.size.rows as usize;
                    let last_row_idx = if node_rows > 0 { node_rows - 1 } else { 0 };
                    let row_ptr = (*node_ptr).data.get_row(last_row_idx);
                    let wrap = if !row_ptr.is_null() {
                        (*row_ptr).wrap()
                    } else {
                        false
                    };
                    if !wrap {
                        break;
                    }

                    added += SlidingWindow::append(self.window, node_ptr);
                    if added >= needle_len - 1 {
                        break;
                    }
                    node_ptr = prev;
                }
            }

            last_node
        }
    }

    pub unsafe fn next(&mut self) -> HighlightFlattened {
        if self.window.is_null() {
            return HighlightFlattened::empty();
        }
        unsafe { SlidingWindow::next(self.window) }
    }
}
