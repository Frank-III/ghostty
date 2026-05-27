use core::ptr;

use crate::allocator::{GhosttyAllocator, alloc_alloc_impl, alloc_free_impl};
use crate::page_list_types::{PageList, PageListNode};
use crate::highlight::{HighlightFlattened, Pin};
use crate::search::search_sliding_window::{SlidingWindow, Direction};

pub struct PageListSearch {
    pub list: *mut PageList,
    pub window: *mut SlidingWindow,
    pub pin: *mut Pin,
    pub alloc: *const GhosttyAllocator,
}

impl PageListSearch {
    pub unsafe fn init(
        alloc: *const GhosttyAllocator,
        needle: &[u8],
        list: *mut PageList,
        start: *mut PageListNode,
    ) -> PageListSearch {
        if list.is_null() || start.is_null() {
            return PageListSearch {
                list: ptr::null_mut(),
                window: ptr::null_mut(),
                pin: ptr::null_mut(),
                alloc,
            };
        }

        unsafe {
            let pin_size = core::mem::size_of::<Pin>();
            let pin_ptr = alloc_alloc_impl(alloc, pin_size) as *mut Pin;
            if pin_ptr.is_null() {
                return PageListSearch {
                    list: ptr::null_mut(),
                    window: ptr::null_mut(),
                    pin: ptr::null_mut(),
                    alloc,
                };
            }

            let node_rows = (*start).data.size.rows;
            let node_cols = (*start).data.size.cols;
            ptr::write(pin_ptr, Pin {
                node: start,
                x: if node_cols > 0 { node_cols - 1 } else { 0 },
                y: if node_rows > 0 { node_rows - 1 } else { 0 },
                garbage: false,
            });

            let window = SlidingWindow::init(alloc, Direction::Reverse, needle);
            if window.is_null() {
                alloc_free_impl(alloc, pin_ptr as *mut u8, pin_size);
                return PageListSearch {
                    list: ptr::null_mut(),
                    window: ptr::null_mut(),
                    pin: ptr::null_mut(),
                    alloc,
                };
            }

            SlidingWindow::append(window, start);

            PageListSearch {
                list,
                window,
                pin: pin_ptr,
                alloc,
            }
        }
    }

    pub unsafe fn deinit(&mut self) {
        unsafe {
            if !self.window.is_null() {
                SlidingWindow::deinit(self.window);
                self.window = ptr::null_mut();
            }
            if !self.pin.is_null() {
                let pin_size = core::mem::size_of::<Pin>();
                alloc_free_impl(self.alloc, self.pin as *mut u8, pin_size);
                self.pin = ptr::null_mut();
            }
            self.list = ptr::null_mut();
        }
    }

    pub unsafe fn next(&mut self) -> HighlightFlattened {
        if self.window.is_null() {
            return HighlightFlattened::empty();
        }
        unsafe { SlidingWindow::next(self.window) }
    }

    pub unsafe fn feed(&mut self) -> bool {
        if self.pin.is_null() || self.window.is_null() {
            return false;
        }
        unsafe {
            if (*self.pin).garbage {
                return false;
            }

            let needle_len = (*self.window).needle_len;
            let mut rem = needle_len;

            let mut node_ptr = if !(*self.pin).node.is_null() {
                (*(*self.pin).node).prev
            } else {
                ptr::null_mut()
            };

            while !node_ptr.is_null() {
                let prev = (*node_ptr).prev;
                let added = SlidingWindow::append(self.window, node_ptr);
                (*self.pin).node = node_ptr;

                if added >= rem {
                    rem = 0;
                } else {
                    rem -= added;
                }

                if rem == 0 {
                    break;
                }
                node_ptr = prev;
            }

            rem < needle_len
        }
    }
}
