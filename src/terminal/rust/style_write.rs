use core::ffi::c_void;
use core::ptr;

use crate::palette_copy::*;
use crate::style::*;

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
