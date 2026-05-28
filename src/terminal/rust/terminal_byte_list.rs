//! Growable byte buffer for terminal title/pwd (Zig `ArrayList(u8)` subset).

use core::ffi::c_void;
use core::ptr;

use crate::allocator::{alloc_alloc_impl, alloc_free_impl, GhosttyAllocator};

#[repr(C)]
pub struct ByteList {
    pub data: *mut u8,
    pub len: usize,
    pub cap: usize,
}

impl ByteList {
    pub unsafe fn create(alloc: *const GhosttyAllocator) -> Option<*mut Self> {
        unsafe {
            let mem = alloc_alloc_impl(alloc, core::mem::size_of::<Self>());
            if mem.is_null() {
                return None;
            }
            let list = mem as *mut Self;
            (*list).data = ptr::null_mut();
            (*list).len = 0;
            (*list).cap = 0;
            Some(list)
        }
    }

    pub unsafe fn destroy(alloc: *const GhosttyAllocator, list: *mut Self) {
        unsafe {
            if list.is_null() {
                return;
            }
            if !(*list).data.is_null() {
                alloc_free_impl(alloc, (*list).data, (*list).cap);
            }
            alloc_free_impl(alloc, list as *mut u8, core::mem::size_of::<Self>());
        }
    }

    pub unsafe fn clear_retaining_capacity(list: *mut Self) {
        unsafe {
            if list.is_null() {
                return;
            }
            (*list).len = 0;
        }
    }

    pub unsafe fn set_slice(
        alloc: *const GhosttyAllocator,
        list: *mut Self,
        value: &[u8],
    ) -> bool {
        unsafe {
            if list.is_null() {
                return false;
            }
            Self::clear_retaining_capacity(list);
            if value.is_empty() {
                return true;
            }
            let needed = value.len() + 1;
            if !Self::ensure_capacity(alloc, list, needed) {
                return false;
            }
            ptr::copy_nonoverlapping(value.as_ptr(), (*list).data, value.len());
            *(*list).data.add(value.len()) = 0;
            (*list).len = needed;
            true
        }
    }

    pub unsafe fn as_cstr_slice<'a>(list: *const Self) -> Option<&'a [u8]> {
        unsafe {
            if list.is_null() || (*list).len == 0 {
                return None;
            }
            let content_len = (*list).len.saturating_sub(1);
            Some(core::slice::from_raw_parts((*list).data, content_len))
        }
    }

    unsafe fn ensure_capacity(
        alloc: *const GhosttyAllocator,
        list: *mut Self,
        needed: usize,
    ) -> bool {
        unsafe {
            if (*list).cap >= needed {
                return true;
            }
            let new_cap = needed.max(16).next_power_of_two();
            let new_ptr = alloc_alloc_impl(alloc, new_cap);
            if new_ptr.is_null() {
                return false;
            }
            if !(*list).data.is_null() && (*list).len > 0 {
                ptr::copy_nonoverlapping((*list).data, new_ptr, (*list).len);
                alloc_free_impl(alloc, (*list).data, (*list).cap);
            }
            (*list).data = new_ptr;
            (*list).cap = new_cap;
            true
        }
    }
}

pub unsafe fn byte_list_from_void(ptr: *mut c_void) -> *mut ByteList {
    ptr as *mut ByteList
}
