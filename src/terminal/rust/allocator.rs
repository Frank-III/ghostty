use crate::color::*;
use crate::constants::*;
use crate::early::*;
use crate::input::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::render::*;
use crate::selection::*;
use crate::simple::*;
use crate::style::*;
use crate::terminal::*;
use core::ffi::{c_int, c_void};
use core::{mem, ptr};

#[repr(C)]
pub struct GhosttyAllocatorVtable {
    alloc: unsafe extern "C" fn(*mut c_void, usize, u8, usize) -> *mut u8,
    resize: unsafe extern "C" fn(*mut c_void, *mut u8, usize, u8, usize, usize) -> bool,
    remap: unsafe extern "C" fn(*mut c_void, *mut u8, usize, u8, usize, usize) -> *mut u8,
    free: unsafe extern "C" fn(*mut c_void, *mut u8, usize, u8, usize),
}

#[repr(C)]
pub struct GhosttyAllocator {
    ctx: *mut c_void,
    vtable: *const GhosttyAllocatorVtable,
}

impl GhosttyAllocator {
    pub fn null() -> Self {
        GhosttyAllocator {
            ctx: ptr::null_mut(),
            vtable: ptr::null(),
        }
    }

    pub fn is_null(&self) -> bool {
        self.ctx.is_null() || self.vtable.is_null()
    }
}

const ALIGN_U8: u8 = 0;
const RETURN_ADDRESS_UNKNOWN: usize = 0;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_alloc_alloc(
    alloc: *const GhosttyAllocator,
    len: usize,
) -> *mut u8 {
    unsafe { alloc_alloc_impl(alloc, len) }
}

pub(crate) unsafe fn alloc_alloc_impl(alloc: *const GhosttyAllocator, len: usize) -> *mut u8 {
    if alloc.is_null() {
        return ptr::null_mut();
    }
    if len == 0 {
        return ptr::NonNull::<u8>::dangling().as_ptr();
    }

    unsafe {
        let vtable = (*alloc).vtable;
        if vtable.is_null() {
            return ptr::null_mut();
        }

        ((*vtable).alloc)((*alloc).ctx, len, ALIGN_U8, RETURN_ADDRESS_UNKNOWN)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_alloc_free(
    alloc: *const GhosttyAllocator,
    ptr: *mut u8,
    len: usize,
) {
    unsafe { alloc_free_impl(alloc, ptr, len) }
}

pub(crate) unsafe fn alloc_free_impl(alloc: *const GhosttyAllocator, ptr: *mut u8, len: usize) {
    if alloc.is_null() || ptr.is_null() || len == 0 {
        return;
    }

    unsafe {
        let vtable = (*alloc).vtable;
        if vtable.is_null() {
            return;
        }

        ((*vtable).free)((*alloc).ctx, ptr, len, ALIGN_U8, RETURN_ADDRESS_UNKNOWN);
    }
}

#[cfg(feature = "std")]
pub fn test_support_allocator() -> GhosttyAllocator {
    static mut CTX: u8 = 0;
    GhosttyAllocator {
        ctx: core::ptr::addr_of_mut!(CTX).cast(),
        vtable: &TEST_VTABLE,
    }
}

#[cfg(feature = "std")]
mod test_alloc {
    use super::*;
    use core::ffi::c_void;

    unsafe extern "C" fn test_alloc_fn(
        _ctx: *mut c_void,
        len: usize,
        _align: u8,
        _ra: usize,
    ) -> *mut u8 {
        if len == 0 {
            return ptr::NonNull::<u8>::dangling().as_ptr();
        }
        let layout = std::alloc::Layout::from_size_align(len, 1).unwrap();
        unsafe { std::alloc::alloc(layout) }
    }

    unsafe extern "C" fn test_realloc_fn(
        _ctx: *mut c_void,
        ptr: *mut u8,
        old_len: usize,
        _align: u8,
        new_len: usize,
        _ra: usize,
    ) -> *mut u8 {
        if ptr.is_null() || old_len == 0 {
            let layout = std::alloc::Layout::from_size_align(new_len, 1).unwrap();
            return unsafe { std::alloc::alloc(layout) };
        }
        if new_len == 0 {
            let layout = std::alloc::Layout::from_size_align(old_len, 1).unwrap();
            unsafe { std::alloc::dealloc(ptr, layout) };
            return ptr::NonNull::<u8>::dangling().as_ptr();
        }
        let layout = std::alloc::Layout::from_size_align(old_len, 1).unwrap();
        unsafe { std::alloc::realloc(ptr, layout, new_len) }
    }

    unsafe extern "C" fn test_resize_fn(
        _ctx: *mut c_void,
        _ptr: *mut u8,
        _old_len: usize,
        _align: u8,
        _new_len: usize,
        _ra: usize,
    ) -> bool {
        false
    }

    unsafe extern "C" fn test_free_fn(
        _ctx: *mut c_void,
        ptr: *mut u8,
        len: usize,
        _align: u8,
        _ra: usize,
    ) {
        if !ptr.is_null() && len > 0 {
            let layout = std::alloc::Layout::from_size_align(len, 1).unwrap();
            unsafe { std::alloc::dealloc(ptr, layout) };
        }
    }

    pub(super) static TEST_VTABLE: GhosttyAllocatorVtable = GhosttyAllocatorVtable {
        alloc: test_alloc_fn,
        resize: test_resize_fn,
        remap: test_realloc_fn,
        free: test_free_fn,
    };
}

#[cfg(feature = "std")]
use test_alloc::TEST_VTABLE;
