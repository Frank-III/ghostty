use core::ffi::c_void;
use core::ptr;

use crate::allocator::GhosttyAllocator;
use crate::early::*;
use crate::page_core::{std_capacity, Page};
use crate::page_list_types::{
    PageList, PageListHead, PageListMemoryPool, PageListNode, PageListTrackedPinSet,
    PageListViewport,
};
use crate::size_types::CellCountInt;
use crate::size_types::OffsetBuf;

extern "C" {
    fn ghostty_vt_memory_pool_create(alloc: *const GhosttyAllocator, preheat: usize)
        -> *mut c_void;
    fn ghostty_vt_memory_pool_destroy(alloc: *const GhosttyAllocator, pool: *mut c_void);
    fn ghostty_vt_pool_create_node(pool: *mut c_void) -> *mut c_void;
    fn ghostty_vt_pool_destroy_node(pool: *mut c_void, node: *mut c_void);
    fn ghostty_vt_pool_create_std_page(pool: *mut c_void) -> *mut u8;
    fn ghostty_vt_pool_destroy_std_page(pool: *mut c_void, page: *mut u8);
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

fn std_size() -> usize {
    Page::layout(std_capacity()).total_size
}

fn min_max_size(_cols: CellCountInt, rows: CellCountInt) -> usize {
    let cap = std_capacity();
    let rows_per_page = cap.rows as usize;
    let pages_needed = if rows as usize == 0 {
        1
    } else {
        (rows as usize + rows_per_page - 1) / rows_per_page
    };
    pages_needed * std_size()
}

impl PageList {
    /// Initialize a PageList with a fresh memory pool from the Zig bridge.
    pub unsafe fn init_full(
        alloc: *const GhosttyAllocator,
        cols: CellCountInt,
        rows: CellCountInt,
        max_scrollback: usize,
    ) -> Option<PageList> {
        unsafe {
            if cols == 0 || rows == 0 || alloc.is_null() {
                return None;
            }

            let pool_raw = ghostty_vt_memory_pool_create(alloc, 4);
            if pool_raw.is_null() {
                return None;
            }

            let mut pl = PageList {
                pool: pool_raw as *mut PageListMemoryPool,
                pages: PageListHead {
                    first: ptr::null_mut(),
                    last: ptr::null_mut(),
                },
                page_serial: 0,
                page_serial_min: 0,
                page_size: 0,
                explicit_max_size: if max_scrollback == 0 {
                    usize::MAX
                } else {
                    max_scrollback
                },
                min_max_size: min_max_size(cols, rows),
                total_rows: rows as usize,
                tracked_pins: ptr::null_mut(),
                viewport: PageListViewport::Active,
                viewport_pin: ptr::null_mut(),
                viewport_pin_row_offset: 0,
                cols,
                rows,
            };

            if !init_pages(&mut pl, pool_raw, rows) {
                ghostty_vt_memory_pool_destroy(alloc, pool_raw);
                return None;
            }

            let first = pl.pages.first;
            if first.is_null() {
                ghostty_vt_memory_pool_destroy(alloc, pool_raw);
                return None;
            }

            let pin_raw = ghostty_vt_pin_create(pool_raw, first as *mut c_void, 0, 0, false);
            if pin_raw.is_null() {
                pl.deinit_pages();
                ghostty_vt_memory_pool_destroy(alloc, pool_raw);
                return None;
            }
            pl.viewport_pin = pin_raw as *mut crate::highlight::Pin;

            let tp_size = core::mem::size_of::<PageListTrackedPinSet>();
            let tp_ptr = ghostty_vt_pool_alloc(pool_raw, tp_size) as *mut PageListTrackedPinSet;
            if tp_ptr.is_null() {
                pl.deinit_pages();
                ghostty_vt_memory_pool_destroy(alloc, pool_raw);
                return None;
            }
            (*tp_ptr).keys = ptr::null_mut();
            (*tp_ptr).len = 0;
            (*tp_ptr).capacity = 0;
            pl.tracked_pins = tp_ptr;

            Some(pl)
        }
    }

    pub unsafe fn deinit_full(&mut self, alloc: *const GhosttyAllocator) {
        unsafe {
            if self.pool.is_null() {
                return;
            }
            let pool_raw = self.pool as *mut c_void;
            self.deinit_pages();
            if !self.tracked_pins.is_null() {
                let tp = self.tracked_pins;
                if !(*tp).keys.is_null() && (*tp).capacity > 0 {
                    let size = (*tp).capacity * core::mem::size_of::<*mut crate::highlight::Pin>();
                    ghostty_vt_pool_free(pool_raw, (*tp).keys as *mut u8, size);
                }
                ghostty_vt_pool_free(
                    pool_raw,
                    tp as *mut u8,
                    core::mem::size_of::<PageListTrackedPinSet>(),
                );
                self.tracked_pins = ptr::null_mut();
            }
            ghostty_vt_memory_pool_destroy(alloc, pool_raw);
            self.pool = ptr::null_mut();
        }
    }
}

unsafe fn init_pages(pl: &mut PageList, pool_raw: *mut c_void, rows: CellCountInt) -> bool {
    unsafe {
        let cap = std_capacity();
        let layout = Page::layout(cap);
        let pooled = layout.total_size <= std_size();

        let mut rem = rows as usize;
        while rem > 0 {
            let node_raw = ghostty_vt_pool_create_node(pool_raw);
            if node_raw.is_null() {
                return false;
            }
            let node = node_raw as *mut PageListNode;

            let page_buf = if pooled {
                ghostty_vt_pool_create_std_page(pool_raw)
            } else {
                ptr::null_mut()
            };
            if page_buf.is_null() {
                ghostty_vt_pool_destroy_node(pool_raw, node_raw);
                return false;
            }

            core::ptr::write_bytes(page_buf, 0, layout.total_size);
            let buf = OffsetBuf::init(page_buf);
            let page_layout = Page::layout(cap);
            (*node).data = Page::init_buf(buf, page_layout);
            (*node).data.size.rows = if rem >= cap.rows as usize {
                cap.rows
            } else {
                rem as u16
            };
            (*node).serial = pl.page_serial;
            (*node).prev = ptr::null_mut();
            (*node).next = ptr::null_mut();

            if pl.pages.last.is_null() {
                pl.pages.first = node;
                pl.pages.last = node;
            } else {
                (*pl.pages.last).next = node;
                (*node).prev = pl.pages.last;
                pl.pages.last = node;
            }

            pl.page_size += layout.total_size;
            pl.page_serial += 1;
            rem = rem.saturating_sub((*node).data.size.rows as usize);
        }

        !pl.pages.first.is_null()
    }
}
