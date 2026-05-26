use core::ffi::{c_int, c_void};
use core::{mem, ptr};
use crate::early::*;
use crate::constants::*;
use crate::terminal::*;
use crate::render::*;
use crate::input::*;
use crate::selection::*;
use crate::kitty_graphics::*;
use crate::mouse_encode::*;
use crate::simple::*;
use crate::event_cell_style::*;
use crate::color::*;

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

const ALIGN_U8: u8 = 0;
const RETURN_ADDRESS_UNKNOWN: usize = 0;

#[no_mangle]
pub unsafe extern "C" fn ghostty_rust_alloc_alloc(
    alloc: *const GhosttyAllocator,
    len: usize,
) -> *mut u8 {
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
