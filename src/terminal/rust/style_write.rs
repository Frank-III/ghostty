use core::ffi::{c_int, c_void};
use core::ptr;

use crate::early::*;
use crate::style::*;

pub(crate) unsafe fn write_rgb_value(out: *mut GhosttyColorRgb, r: u8, g: u8, b: u8) {
    unsafe {
        ptr::write(core::ptr::addr_of_mut!((*out).r), r);
        ptr::write(core::ptr::addr_of_mut!((*out).g), g);
        ptr::write(core::ptr::addr_of_mut!((*out).b), b);
    }
}

pub(crate) unsafe fn copy_palette(dst: *mut GhosttyColorRgb, src: *const GhosttyColorRgb) -> c_int {
    if dst.is_null() || src.is_null() {
        return GHOSTTY_INVALID_VALUE;
    }

    let mut i = 0usize;
    while i < 256 {
        unsafe {
            let src_rgb = src.add(i);
            write_rgb_value(
                dst.add(i),
                ptr::read(core::ptr::addr_of!((*src_rgb).r)),
                ptr::read(core::ptr::addr_of!((*src_rgb).g)),
                ptr::read(core::ptr::addr_of!((*src_rgb).b)),
            );
        }
        i += 1;
    }

    GHOSTTY_SUCCESS
}

pub(crate) unsafe fn write_rgb(out: *mut c_void, src: *const GhosttyColorRgb) {
    let rgb = out.cast::<GhosttyColorRgb>();
    unsafe {
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).r),
            ptr::read(core::ptr::addr_of!((*src).r)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).g),
            ptr::read(core::ptr::addr_of!((*src).g)),
        );
        ptr::write(
            core::ptr::addr_of_mut!((*rgb).b),
            ptr::read(core::ptr::addr_of!((*src).b)),
        );
    }
}

pub(crate) unsafe fn write_out<T>(out: *mut c_void, value: T) {
    unsafe {
        ptr::write(out.cast::<T>(), value);
    }
}
