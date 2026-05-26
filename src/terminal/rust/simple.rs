use core::ffi::c_void;
use core::{mem, ptr};

pub(crate) use crate::simple_write::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyString {
    pub(crate) ptr: *const u8,
    pub(crate) len: usize,
}

pub(crate) fn struct_sized_field_fits<T>(size: usize, offset: usize) -> bool {
    size >= offset.saturating_add(mem::size_of::<T>())
}

pub(crate) unsafe fn write_string(out: *mut c_void, bytes: &'static [u8]) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), bytes.as_ptr());
        ptr::write(core::ptr::addr_of_mut!((*string).len), bytes.len());
    }
}

pub(crate) unsafe fn write_borrowed_string(out: *mut c_void, ptr: *const u8, len: usize) {
    let string = out.cast::<GhosttyString>();
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*string).ptr), ptr);
        ptr::write(core::ptr::addr_of_mut!((*string).len), len);
    }
}
