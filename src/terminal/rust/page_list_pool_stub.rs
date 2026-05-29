//! Std test/support pool bootstrap for Rust-owned terminals in cargo binaries.
//!
//! Implements the Zig `pin_bridge` symbols so `ghostty-vt` can link without a
//! libghostty-vt object when built with `feature = "std"` + `terminal-owned`.

use core::ffi::c_void;
use core::mem;
use core::ptr;

use crate::allocator::{alloc_alloc_impl, alloc_free_impl, GhosttyAllocator};
use crate::highlight::Pin;
use crate::page_core::{std_capacity, Page, PAGE_SIZE_MIN};
use crate::page_list_types::PageListNode;

fn std_page_size() -> usize {
    Page::layout(std_capacity()).total_size
}

fn alloc_aligned(_alloc: &GhosttyAllocator, size: usize, align: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }
    let layout = match std::alloc::Layout::from_size_align(size, align) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };
    unsafe { std::alloc::alloc(layout) }
}

fn free_aligned(alloc: &GhosttyAllocator, ptr: *mut u8, size: usize, align: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    if let Ok(layout) = std::alloc::Layout::from_size_align(size, align) {
        unsafe { std::alloc::dealloc(ptr, layout) };
    }
    let _ = alloc;
}

#[repr(C)]
struct TestMemoryPool {
    alloc: GhosttyAllocator,
}

fn pool_ref(pool_ptr: *mut c_void) -> Option<*mut TestMemoryPool> {
    if pool_ptr.is_null() {
        return None;
    }
    Some(pool_ptr as *mut TestMemoryPool)
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_memory_pool_create(
    alloc: *const GhosttyAllocator,
    _preheat: usize,
) -> *mut c_void {
    unsafe {
        if alloc.is_null() || (*alloc).is_null() {
            return ptr::null_mut();
        }
        let size = mem::size_of::<TestMemoryPool>();
        let raw = alloc_alloc_impl(alloc, size);
        if raw.is_null() {
            return ptr::null_mut();
        }
        let pool = raw as *mut TestMemoryPool;
        (*pool).alloc = ptr::read(alloc);
        pool as *mut c_void
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_memory_pool_destroy(
    alloc: *const GhosttyAllocator,
    pool_ptr: *mut c_void,
) {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return;
        };
        alloc_free_impl(alloc, pool as *mut u8, mem::size_of::<TestMemoryPool>());
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pool_create_node(pool_ptr: *mut c_void) -> *mut c_void {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return ptr::null_mut();
        };
        let size = mem::size_of::<PageListNode>();
        let align = mem::align_of::<PageListNode>();
        let raw = alloc_aligned(&(*pool).alloc, size, align);
        if raw.is_null() {
            return ptr::null_mut();
        }
        ptr::write_bytes(raw, 0, size);
        raw as *mut c_void
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pool_destroy_node(
    pool_ptr: *mut c_void,
    node_ptr: *mut c_void,
) {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return;
        };
        if node_ptr.is_null() {
            return;
        }
        free_aligned(
            &(*pool).alloc,
            node_ptr as *mut u8,
            mem::size_of::<PageListNode>(),
            mem::align_of::<PageListNode>(),
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pool_create_std_page(pool_ptr: *mut c_void) -> *mut u8 {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return ptr::null_mut();
        };
        alloc_aligned(&(*pool).alloc, std_page_size(), PAGE_SIZE_MIN)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pool_destroy_std_page(pool_ptr: *mut c_void, page: *mut u8) {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return;
        };
        if page.is_null() {
            return;
        }
        free_aligned(&(*pool).alloc, page, std_page_size(), PAGE_SIZE_MIN);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pin_create(
    pool_ptr: *mut c_void,
    node: *mut c_void,
    y: u16,
    x: u16,
    garbage: bool,
) -> *mut c_void {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return ptr::null_mut();
        };
        let size = mem::size_of::<Pin>();
        let raw = alloc_alloc_impl(&(*pool).alloc, size);
        if raw.is_null() {
            return ptr::null_mut();
        }
        let pin = raw as *mut Pin;
        ptr::write(
            pin,
            Pin {
                node: node as *mut PageListNode,
                y,
                x,
                garbage,
            },
        );
        pin as *mut c_void
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pin_destroy(pool_ptr: *mut c_void, pin: *mut c_void) {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return;
        };
        if pin.is_null() {
            return;
        }
        alloc_free_impl(&(*pool).alloc, pin as *mut u8, mem::size_of::<Pin>());
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pool_alloc(pool_ptr: *mut c_void, size: usize) -> *mut u8 {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return ptr::null_mut();
        };
        alloc_alloc_impl(&(*pool).alloc, size)
    }
}

#[no_mangle]
pub unsafe extern "C" fn ghostty_vt_pool_free(pool_ptr: *mut c_void, ptr: *mut u8, size: usize) {
    unsafe {
        let Some(pool) = pool_ref(pool_ptr) else {
            return;
        };
        if ptr.is_null() || size == 0 {
            return;
        }
        alloc_free_impl(&(*pool).alloc, ptr, size);
    }
}
