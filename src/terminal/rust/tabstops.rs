use crate::allocator::*;
use crate::constants::*;
use crate::early::*;

const UNIT_BITS: usize = 8;
const PREALLOC_COLUMNS: usize = 512;
const PREALLOC_COUNT: usize = PREALLOC_COLUMNS / UNIT_BITS;

const MASKS: [u8; UNIT_BITS] = [
    0b00000001, 0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000,
];

#[repr(C)]
pub struct Tabstops {
    pub cols: usize,
    pub prealloc_stops: [u8; PREALLOC_COUNT],
    pub dynamic_stops_ptr: *mut u8,
    pub dynamic_stops_len: usize,
}

impl Tabstops {
    #[inline]
    fn entry(col: usize) -> usize {
        col / UNIT_BITS
    }

    #[inline]
    fn index(col: usize) -> usize {
        col % UNIT_BITS
    }

    pub fn capacity(&self) -> usize {
        (PREALLOC_COUNT + self.dynamic_stops_len) * UNIT_BITS
    }

    pub fn set(&mut self, col: usize) {
        let i = Self::entry(col);
        let idx = Self::index(col);
        let mask = unsafe { *MASKS.as_ptr().add(idx) };
        if i < PREALLOC_COUNT {
            unsafe {
                let p = self.prealloc_stops.as_mut_ptr().add(i);
                *p |= mask;
            }
        } else {
            let dynamic_i = i - PREALLOC_COUNT;
            unsafe {
                *self.dynamic_stops_ptr.add(dynamic_i) |= mask;
            }
        }
    }

    pub fn unset(&mut self, col: usize) {
        let i = Self::entry(col);
        let idx = Self::index(col);
        let mask = unsafe { *MASKS.as_ptr().add(idx) };
        if i < PREALLOC_COUNT {
            unsafe {
                let p = self.prealloc_stops.as_mut_ptr().add(i);
                *p &= !mask;
            }
        } else {
            let dynamic_i = i - PREALLOC_COUNT;
            unsafe {
                *self.dynamic_stops_ptr.add(dynamic_i) &= !mask;
            }
        }
    }

    pub fn get(&self, col: usize) -> bool {
        let i = Self::entry(col);
        let idx = Self::index(col);
        let mask = unsafe { *MASKS.as_ptr().add(idx) };
        let unit = if i < PREALLOC_COUNT {
            unsafe { *self.prealloc_stops.as_ptr().add(i) }
        } else {
            let dynamic_i = i - PREALLOC_COUNT;
            unsafe { *self.dynamic_stops_ptr.add(dynamic_i) }
        };
        unit & mask == mask
    }

    pub unsafe fn init(
        alloc: *const GhosttyAllocator,
        cols: usize,
        interval: usize,
    ) -> Option<Tabstops> {
        let mut t = Tabstops {
            cols: 0,
            prealloc_stops: [0u8; PREALLOC_COUNT],
            dynamic_stops_ptr: core::ptr::null_mut(),
            dynamic_stops_len: 0,
        };
        if !unsafe { Self::resize_impl(&mut t, alloc, cols) } {
            return None;
        }
        t.reset(interval);
        Some(t)
    }

    pub unsafe fn deinit(&mut self, alloc: *const GhosttyAllocator) {
        if self.dynamic_stops_len > 0 && !self.dynamic_stops_ptr.is_null() {
            unsafe {
                alloc_free_impl(alloc, self.dynamic_stops_ptr, self.dynamic_stops_len);
            }
        }
        self.dynamic_stops_ptr = core::ptr::null_mut();
        self.dynamic_stops_len = 0;
    }

    pub unsafe fn resize(&mut self, alloc: *const GhosttyAllocator, cols: usize) -> bool {
        unsafe { Self::resize_impl(self, alloc, cols) }
    }

    unsafe fn resize_impl(t: &mut Tabstops, alloc: *const GhosttyAllocator, cols: usize) -> bool {
        if cols <= PREALLOC_COLUMNS {
            t.cols = cols;
            return true;
        }
        let size = cols - PREALLOC_COLUMNS;
        if size <= t.dynamic_stops_len {
            t.cols = cols;
            return true;
        }
        let new = unsafe { alloc_alloc_impl(alloc, size) };
        if new.is_null() {
            return false;
        }
        unsafe {
            core::ptr::write_bytes(new, 0, size);
        }
        if t.dynamic_stops_len > 0 && !t.dynamic_stops_ptr.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(t.dynamic_stops_ptr, new, t.dynamic_stops_len);
                alloc_free_impl(alloc, t.dynamic_stops_ptr, t.dynamic_stops_len);
            }
        }
        t.dynamic_stops_ptr = new;
        t.dynamic_stops_len = size;
        t.cols = cols;
        true
    }

    pub fn clear(&mut self) {
        unsafe {
            core::ptr::write_bytes(self.prealloc_stops.as_mut_ptr(), 0, PREALLOC_COUNT);
        }
        if self.dynamic_stops_len > 0 && !self.dynamic_stops_ptr.is_null() {
            unsafe {
                core::ptr::write_bytes(self.dynamic_stops_ptr, 0, self.dynamic_stops_len);
            }
        }
    }

    pub fn reset(&mut self, interval: usize) {
        self.clear();
        if interval > 0 && self.cols > 1 {
            let mut i = interval;
            let limit = self.cols - 1;
            while i < limit {
                self.set(i);
                i += interval;
            }
        }
    }
}

impl Default for Tabstops {
    fn default() -> Self {
        Tabstops {
            cols: 0,
            prealloc_stops: [0u8; PREALLOC_COUNT],
            dynamic_stops_ptr: core::ptr::null_mut(),
            dynamic_stops_len: 0,
        }
    }
}
