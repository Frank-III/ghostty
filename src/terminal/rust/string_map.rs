use crate::early::*;
use crate::constants::*;
use crate::allocator::*;

#[repr(C)]
pub struct StringMap {
    pub string_ptr: *mut u8,
    pub string_len: usize,
    pub map_ptr: *mut u8,
    pub map_len: usize,
}

impl StringMap {
    pub fn string(&self) -> &[u8] {
        if self.string_ptr.is_null() || self.string_len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.string_ptr, self.string_len) }
    }

    pub unsafe fn deinit(&self, alloc: *const GhosttyAllocator) {
        if !self.string_ptr.is_null() && self.string_len > 0 {
            unsafe { alloc_free_impl(alloc, self.string_ptr, self.string_len + 1); }
        }
        if !self.map_ptr.is_null() && self.map_len > 0 {
            unsafe { alloc_free_impl(alloc, self.map_ptr, self.map_len * core::mem::size_of::<usize>()); }
        }
    }
}

impl Default for StringMap {
    fn default() -> Self {
        StringMap {
            string_ptr: core::ptr::null_mut(),
            string_len: 0,
            map_ptr: core::ptr::null_mut(),
            map_len: 0,
        }
    }
}
